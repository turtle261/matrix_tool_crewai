use actix_web::{get, post, web, HttpResponse, Responder};
use matrix_sdk::{Client, config::SyncSettings, room::MessagesOptions};
use matrix_sdk::ruma::{OwnedRoomId, UInt};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;
use crate::{config::Config, error::ApiError};
use url::Url;

#[derive(Clone)]
pub struct ApiState {
    pub sessions: Arc<RwLock<HashMap<String, Session>>>,
    pub config: Config,
}

#[derive(Clone)]
pub struct Session {
    pub client: Option<Client>,
    pub error: Option<String>,
}

#[post("/login/sso/start")]
pub async fn login_sso_start(state: web::Data<ApiState>) -> Result<impl Responder, ApiError> {
    let session_id = Uuid::new_v4().to_string();
    let homeserver_url = Url::parse(&state.config.homeserver.url).map_err(|e| ApiError::MatrixError(e.to_string()))?;
    let client = Client::new(homeserver_url).await.map_err(|e| ApiError::Http(e))?;
    
    // Make sure the redirect URL exactly matches what Matrix expects for the SSO callback
    let redirect_url = format!("http://localhost:8080/login/sso/callback?session_id={}", session_id);
    
    let sso_url = client
        .matrix_auth()
        .get_sso_login_url(&redirect_url, None)
        .await
        .map_err(|e| ApiError::MatrixError(e.to_string()))?;

    let mut sessions = state.sessions.write().await;
    sessions.insert(session_id.clone(), Session { client: Some(client), error: None });
    Ok(HttpResponse::Ok().json(json!({
        "session_id": session_id,
        "sso_url": sso_url,
    })))
}

#[get("/login/sso/callback")]
pub async fn login_sso_callback(
    state: web::Data<ApiState>,
    query: web::Query<CallbackQuery>,
) -> impl Responder {
    let session_id = &query.session_id;
    let login_token = &query.login_token;
    
    let mut sessions = state.sessions.write().await;
    
    if !sessions.contains_key(session_id) {
        return HttpResponse::BadRequest().body(format!("Invalid session ID: {}", session_id));
    }
    
    let session = sessions.get_mut(session_id).unwrap();

    if let Some(client) = session.client.as_ref() {
        match client.matrix_auth().login_token(login_token).await {
            Ok(_) => {
                session.error = None;
                HttpResponse::Ok().body("Login successful! You can now close this window and return to the application.")
            }
            Err(e) => {
                let error_msg = e.to_string();
                session.error = Some(error_msg.clone());
                HttpResponse::BadRequest().body(format!("Login failed: {}", error_msg))
            }
        }
    } else {
        HttpResponse::BadRequest().body("Session doesn't have a client")
    }
}

#[get("/login/status/{session_id}")]
pub async fn login_status(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> Result<impl Responder, ApiError> {
    let session_id = path.into_inner();
    let sessions = state.sessions.read().await;
    let session = sessions.get(&session_id).ok_or(ApiError::InvalidSession)?;
    if let Some(client) = &session.client {
        if client.logged_in() {
            Ok(HttpResponse::Ok().json(json!({"status": "logged_in"})))
        } else if let Some(error) = &session.error {
            Ok(HttpResponse::Ok().json(json!({"status": "error", "error": error})))
        } else {
            Ok(HttpResponse::Ok().json(json!({"status": "pending"})))
        }
    } else {
        Err(ApiError::InvalidSession)
    }
}

#[get("/sync/{session_id}")]
pub async fn sync(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> Result<impl Responder, ApiError> {
    let session_id = path.into_inner();
    let sessions = state.sessions.read().await;
    let session = sessions.get(&session_id).ok_or(ApiError::InvalidSession)?;
    let client = session.client.as_ref().ok_or(ApiError::NotLoggedIn)?;
    
    // Create sync settings with a shorter timeout
    let sync_settings = SyncSettings::default().timeout(std::time::Duration::from_secs(20));
    
    // Use tokio timeout as an additional safety measure
    let sync_future = client.sync_once(sync_settings);
    let sync_result = tokio::time::timeout(
        std::time::Duration::from_secs(30), // 15 seconds timeout
        sync_future
    ).await;
    
    // Handle both timeout and matrix errors
    match sync_result {
        Ok(Ok(sync_response)) => {
            // Return a JSON object with rooms and other relevant info
            let mut rooms_data = Vec::new();
            
            for (room_id, room_info) in sync_response.rooms.join {
                rooms_data.push(json!({
                    "room_id": room_id.to_string(),
                    "unread_notifications": room_info.unread_notifications,
                    "timeline_events": room_info.timeline.events.len()
                }));
            }
            
            Ok(HttpResponse::Ok().json(json!({
                "rooms": rooms_data,
                "next_batch": sync_response.next_batch
            })))
        },
        Ok(Err(e)) => {
            // Matrix SDK error occurred
            // If sync fails, still try to return the list of joined rooms
            // This ensures the API continues to work even if sync times out
            let joined_rooms = client.joined_rooms();
            
            let mut room_infos = Vec::new();
            for room in joined_rooms {
                room_infos.push(json!({
                    "room_id": room.room_id().to_string()
                }));
            }
            
            // Return the rooms we could get, along with the error
            Ok(HttpResponse::Ok().json(json!({
                "rooms": room_infos,
                "error": format!("Sync warning (continuing with basic room list): {}", e)
            })))
        },
        Err(_) => {
            // Tokio timeout error occurred
            // Still try to return the list of joined rooms
            let joined_rooms = client.joined_rooms();
            
            let mut room_infos = Vec::new();
            for room in joined_rooms {
                room_infos.push(json!({
                    "room_id": room.room_id().to_string()
                }));
            }
            
            Ok(HttpResponse::Ok().json(json!({
                "rooms": room_infos,
                "error": "Sync timed out (continuing with basic room list)"
            })))
        }
    }
}

#[get("/rooms/{session_id}")]
pub async fn rooms(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> Result<impl Responder, ApiError> {
    let session_id = path.into_inner();
    let sessions = state.sessions.read().await;
    let session = sessions.get(&session_id).ok_or(ApiError::InvalidSession)?;
    let client = session.client.as_ref().ok_or(ApiError::NotLoggedIn)?;
    let joined_rooms = client.joined_rooms();
    
    let mut room_infos = Vec::new();
    for room in joined_rooms {
        let display_name = room.display_name().await;
        let name = match display_name {
            Ok(name) => name.to_string(),
            Err(_) => "Unknown".to_string(),
        };
        room_infos.push(json!({
            "room_id": room.room_id().to_string(),
            "name": name,
        }));
    }
    
    Ok(HttpResponse::Ok().json(room_infos))
}

#[get("/rooms/{session_id}/{room_id}/messages")]
pub async fn room_messages(
    state: web::Data<ApiState>,
    path: web::Path<(String, String)>,
) -> Result<impl Responder, ApiError> {
    let (session_id, room_id_str) = path.into_inner();
    let sessions = state.sessions.read().await;
    let session = sessions.get(&session_id).ok_or(ApiError::InvalidSession)?;
    let client = session.client.as_ref().ok_or(ApiError::NotLoggedIn)?;
    
    let room_id = OwnedRoomId::try_from(room_id_str)
        .map_err(|_| ApiError::MatrixError("Invalid room ID format".to_string()))?;
    
    let room = client
        .get_room(&room_id)
        .ok_or(ApiError::MatrixError("Room not found".to_string()))?;
    
    // Create options for requesting messages with a limited count to avoid timeouts
    let mut options = MessagesOptions::backward();
    options.limit = UInt::from(20u32); // Limit to 20 messages
    
    // Set a tokio timeout to ensure we don't hang for too long
    let messages_future = room.messages(options);
    let messages_response = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        messages_future
    ).await;
    
    // Handle timeout and other potential errors
    match messages_response {
        Ok(Ok(response)) => {
            // Extract and format messages for response
            let mut messages = Vec::new();
            
            // Process messages
            for chunk in response.chunk {
                // Get the raw event as a value we can work with
                let value = chunk.event.deserialize_as::<serde_json::Value>().ok();
                
                if let Some(value) = value {
                    // Try to extract message details from common fields
                    let sender = value.get("sender").and_then(|s| s.as_str()).unwrap_or("Unknown");
                    
                    // Try to get the message body from content
                    let body = if let Some(content) = value.get("content") {
                        content.get("body").and_then(|b| b.as_str()).unwrap_or("No content")
                    } else {
                        "No content"
                    };
                    
                    let event_id = value.get("event_id").and_then(|e| e.as_str()).unwrap_or("Unknown");
                    let timestamp = value.get("origin_server_ts").and_then(|t| t.as_u64()).unwrap_or(0);
                    
                    messages.push(json!({
                        "sender": sender,
                        "body": body,
                        "event_id": event_id,
                        "timestamp": timestamp
                    }));
                }
            }
            
            Ok(HttpResponse::Ok().json(messages))
        },
        Ok(Err(e)) => {
            // Matrix SDK error
            Err(ApiError::MatrixSdk(e))
        },
        Err(_) => {
            // Timeout error
            Err(ApiError::MatrixError("Request for messages timed out".to_string()))
        }
    }
}

#[post("/rooms/{session_id}/{room_id}/join")]
pub async fn join_room(
    state: web::Data<ApiState>,
    path: web::Path<(String, String)>,
) -> Result<impl Responder, ApiError> {
    let (session_id, room_id_str) = path.into_inner();
    let sessions = state.sessions.read().await;
    let session = sessions.get(&session_id).ok_or(ApiError::InvalidSession)?;
    let client = session.client.as_ref().ok_or(ApiError::NotLoggedIn)?;
    
    let room_id = OwnedRoomId::try_from(room_id_str.clone())
        .map_err(|_| ApiError::MatrixError("Invalid room ID format".to_string()))?;
    
    // Set a tokio timeout to ensure we don't hang for too long
    let join_future = client.join_room_by_id(&room_id);
    let join_result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        join_future
    ).await;
    
    match join_result {
        Ok(Ok(room)) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "room_id": room.room_id().to_string(),
                "message": format!("Successfully joined room {}", room_id_str)
            })))
        },
        Ok(Err(e)) => {
            Err(ApiError::MatrixError(format!("Failed to join room: {}", e)))
        },
        Err(_) => {
            Err(ApiError::MatrixError("Request to join room timed out".to_string()))
        }
    }
}

#[post("/rooms/{session_id}/{room_id}/leave")]
pub async fn leave_room(
    state: web::Data<ApiState>,
    path: web::Path<(String, String)>,
) -> Result<impl Responder, ApiError> {
    let (session_id, room_id_str) = path.into_inner();
    let sessions = state.sessions.read().await;
    let session = sessions.get(&session_id).ok_or(ApiError::InvalidSession)?;
    let client = session.client.as_ref().ok_or(ApiError::NotLoggedIn)?;
    
    let room_id = OwnedRoomId::try_from(room_id_str.clone())
        .map_err(|_| ApiError::MatrixError("Invalid room ID format".to_string()))?;
    
    // Get the room first
    let room = client
        .get_room(&room_id)
        .ok_or(ApiError::MatrixError(format!("Room {} not found", room_id_str)))?;
    
    // Set a tokio timeout to ensure we don't hang for too long
    let leave_future = room.leave();
    let leave_result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        leave_future
    ).await;
    
    match leave_result {
        Ok(Ok(_)) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "room_id": room_id.to_string(),
                "message": format!("Successfully left room {}", room_id_str)
            })))
        },
        Ok(Err(e)) => {
            Err(ApiError::MatrixError(format!("Failed to leave room: {}", e)))
        },
        Err(_) => {
            Err(ApiError::MatrixError("Request to leave room timed out".to_string()))
        }
    }
}

#[post("/rooms/{session_id}/{room_id}/send")]
pub async fn send_message(
    state: web::Data<ApiState>,
    path: web::Path<(String, String)>,
    message_body: web::Json<MessageBody>,
) -> Result<impl Responder, ApiError> {
    let (session_id, room_id_str) = path.into_inner();
    let sessions = state.sessions.read().await;
    let session = sessions.get(&session_id).ok_or(ApiError::InvalidSession)?;
    let client = session.client.as_ref().ok_or(ApiError::NotLoggedIn)?;
    
    let room_id = OwnedRoomId::try_from(room_id_str)
        .map_err(|_| ApiError::MatrixError("Invalid room ID format".to_string()))?;
    
    let room = client
        .get_room(&room_id)
        .ok_or(ApiError::MatrixError("Room not found".to_string()))?;
    
    // Create the plain text message content
    use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEventContent};
    let content = RoomMessageEventContent::new(MessageType::Text(
        matrix_sdk::ruma::events::room::message::TextMessageEventContent::plain(
            message_body.body.clone(),
        ),
    ));
    
    // Set a tokio timeout to ensure we don't hang for too long
    let send_future = room.send(content);
    let send_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        send_future
    ).await;
    
    match send_result {
        Ok(Ok(response)) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "event_id": response.event_id.to_string()
            })))
        },
        Ok(Err(e)) => {
            Err(ApiError::MatrixError(format!("Failed to send message: {}", e)))
        },
        Err(_) => {
            Err(ApiError::MatrixError("Request to send message timed out".to_string()))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct MessageBody {
    body: String,
}

#[derive(serde::Deserialize)]
pub struct CallbackQuery {
    session_id: String,
    #[serde(rename = "loginToken")]
    login_token: String,
}

#[get("/status")]
pub async fn status() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "running"
    }))
}