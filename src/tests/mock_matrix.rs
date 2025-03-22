//! Mock Matrix client for testing
//!
//! This module provides a mock implementation of the Matrix client for testing purposes.

use std::collections::HashMap;
use std::sync::Arc;
use matrix_sdk::{Client, Room};
use matrix_sdk::ruma::{OwnedRoomId, OwnedEventId, OwnedUserId, RoomId, UserId};
use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEventContent};
use tokio::sync::{Mutex, RwLock};
use url::Url;
use async_trait::async_trait;

/// Mock Matrix client for testing
pub struct MockMatrixClient {
    /// Whether the client is logged in
    pub logged_in: bool,
    /// The rooms the client is in
    pub rooms: HashMap<String, MockRoom>,
    /// The messages in each room
    pub messages: HashMap<String, Vec<MockMessage>>,
}

/// Mock room for testing
pub struct MockRoom {
    /// The room ID
    pub room_id: String,
    /// The room name
    pub name: String,
}

/// Mock message for testing
pub struct MockMessage {
    /// The sender of the message
    pub sender: String,
    /// The message body
    pub body: String,
    /// The event ID
    pub event_id: String,
    /// The timestamp
    pub timestamp: u64,
}

impl MockMatrixClient {
    /// Create a new mock Matrix client
    pub fn new() -> Self {
        let mut rooms = HashMap::new();
        let mut messages = HashMap::new();
        
        // Add some test rooms
        rooms.insert(
            "#test:example.org".to_string(),
            MockRoom {
                room_id: "#test:example.org".to_string(),
                name: "Test Room".to_string(),
            },
        );
        
        rooms.insert(
            "#general:example.org".to_string(),
            MockRoom {
                room_id: "#general:example.org".to_string(),
                name: "General".to_string(),
            },
        );
        
        // Add some test messages
        messages.insert(
            "#test:example.org".to_string(),
            vec![
                MockMessage {
                    sender: "@user1:example.org".to_string(),
                    body: "Hello, world!".to_string(),
                    event_id: "$event1".to_string(),
                    timestamp: 1620000000000,
                },
                MockMessage {
                    sender: "@user2:example.org".to_string(),
                    body: "Hi there!".to_string(),
                    event_id: "$event2".to_string(),
                    timestamp: 1620000001000,
                },
            ],
        );
        
        messages.insert(
            "#general:example.org".to_string(),
            vec![
                MockMessage {
                    sender: "@user1:example.org".to_string(),
                    body: "Welcome to the general room!".to_string(),
                    event_id: "$event3".to_string(),
                    timestamp: 1620000002000,
                },
            ],
        );
        
        Self {
            logged_in: false,
            rooms,
            messages,
        }
    }
    
    /// Get a list of joined rooms
    pub fn joined_rooms(&self) -> Vec<MockRoom> {
        self.rooms.values().cloned().collect()
    }
    
    /// Get a room by ID
    pub fn get_room(&self, room_id: &str) -> Option<MockRoom> {
        self.rooms.get(room_id).cloned()
    }
    
    /// Get messages from a room
    pub fn get_messages(&self, room_id: &str) -> Vec<MockMessage> {
        self.messages.get(room_id).cloned().unwrap_or_default()
    }
    
    /// Send a message to a room
    pub fn send_message(&mut self, room_id: &str, message: &str) -> String {
        let event_id = format!("$event{}", self.messages.len() + 1);
        
        let mock_message = MockMessage {
            sender: "@test_user:example.org".to_string(),
            body: message.to_string(),
            event_id: event_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };
        
        if let Some(messages) = self.messages.get_mut(room_id) {
            messages.push(mock_message);
        } else {
            self.messages.insert(room_id.to_string(), vec![mock_message]);
        }
        
        event_id
    }
    
    /// Join a room
    pub fn join_room(&mut self, room_id: &str) {
        if !self.rooms.contains_key(room_id) {
            self.rooms.insert(
                room_id.to_string(),
                MockRoom {
                    room_id: room_id.to_string(),
                    name: format!("Room {}", room_id),
                },
            );
        }
    }
    
    /// Leave a room
    pub fn leave_room(&mut self, room_id: &str) {
        self.rooms.remove(room_id);
        self.messages.remove(room_id);
    }
}

impl Clone for MockRoom {
    fn clone(&self) -> Self {
        Self {
            room_id: self.room_id.clone(),
            name: self.name.clone(),
        }
    }
}

impl Clone for MockMessage {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            body: self.body.clone(),
            event_id: self.event_id.clone(),
            timestamp: self.timestamp,
        }
    }
}

/// Create a mock API state for testing
pub fn create_mock_api_state() -> crate::api::ApiState {
    let config = crate::config::Config {
        homeserver: crate::config::HomeserverConfig {
            url: "https://example.org".to_string(),
        },
    };
    
    let sessions = Arc::new(RwLock::new(HashMap::new()));
    
    crate::api::ApiState {
        sessions,
        config,
    }
}

/// Create a test session with a mock client
pub async fn create_test_session(state: &crate::api::ApiState) -> String {
    let session_id = uuid::Uuid::new_v4().to_string();
    
    let mut sessions = state.sessions.write().await;
    sessions.insert(
        session_id.clone(),
        crate::api::Session {
            client: None, // We'll mock the client in the tests
            error: None,
        },
    );
    
    session_id
}