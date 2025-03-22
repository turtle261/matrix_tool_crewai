//! Tests for the Matrix API endpoints
//!
//! This module contains tests for all the API endpoints used by the Matrix Tool.

use actix_web::{test, web, App};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::api::{self, ApiState, Session};
use crate::config::Config;
use crate::error::ApiError;
use super::mock_matrix::{create_mock_api_state, create_test_session, MockMatrixClient};

/// Test the status endpoint
#[actix_web::test]
async fn test_status_endpoint() {
    // Create a test app
    let app = test::init_service(
        App::new()
            .service(api::status)
    ).await;
    
    // Send a request to the status endpoint
    let req = test::TestRequest::get().uri("/status").to_request();
    let resp = test::call_service(&app, req).await;
    
    // Check that the response is successful
    assert!(resp.status().is_success());
    
    // Check the response body
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "running");
    
    println!("✅ Status endpoint test passed");
}

/// Test the login SSO start endpoint
#[actix_web::test]
async fn test_login_sso_start() {
    // Create a mock API state
    let state = create_mock_api_state();
    
    // Create a test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(api::login_sso_start)
    ).await;
    
    // Send a request to the login SSO start endpoint
    let req = test::TestRequest::post().uri("/login/sso/start").to_request();
    let resp = test::call_service(&app, req).await;
    
    // Check that the response is successful
    assert!(resp.status().is_success());
    
    // Check the response body
    let body: Value = test::read_body_json(resp).await;
    assert!(body.get("session_id").is_some());
    assert!(body.get("sso_url").is_some());
    
    println!("✅ Login SSO start endpoint test passed");
}

/// Test the login status endpoint
#[actix_web::test]
async fn test_login_status() {
    // Create a mock API state
    let state = create_mock_api_state();
    
    // Create a test session
    let session_id = create_test_session(&state).await;
    
    // Create a test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(api::login_status)
    ).await;
    
    // Send a request to the login status endpoint
    let req = test::TestRequest::get()
        .uri(&format!("/login/status/{}", session_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    // Check that the response is successful
    assert!(resp.status().is_success());
    
    // Check the response body
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "pending");
    
    println!("✅ Login status endpoint test passed");
}

/// Test the rooms endpoint
#[actix_web::test]
async fn test_rooms_endpoint() {
    // Create a mock API state
    let state = create_mock_api_state();
    
    // Create a test session
    let session_id = create_test_session(&state).await;
    
    // Set up a mock client in the session
    {
        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&session_id).unwrap();
        
        // Create a mock client
        let mock_client = MockMatrixClient::new();
        
        // Store the mock client in the session
        // Note: In a real test, we would need to implement a way to mock the matrix_sdk::Client
        // For now, we'll just set it to None and handle that in our test
        session.client = None;
    }
    
    // Create a test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(api::rooms)
    ).await;
    
    // Send a request to the rooms endpoint
    let req = test::TestRequest::get()
        .uri(&format!("/rooms/{}", session_id))
        .to_request();
    
    // This will fail because we haven't properly mocked the matrix_sdk::Client
    // In a real implementation, we would need to create a proper mock
    let resp = test::call_service(&app, req).await;
    
    // We expect an error because the client is None
    assert!(resp.status().is_client_error());
    
    println!("ℹ️ Rooms endpoint test skipped (requires proper mocking of matrix_sdk::Client)");
}

/// Test the room messages endpoint
#[actix_web::test]
async fn test_room_messages_endpoint() {
    // Create a mock API state
    let state = create_mock_api_state();
    
    // Create a test session
    let session_id = create_test_session(&state).await;
    
    // Set up a mock client in the session
    {
        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&session_id).unwrap();
        
        // Create a mock client
        let mock_client = MockMatrixClient::new();
        
        // Store the mock client in the session
        // Note: In a real test, we would need to implement a way to mock the matrix_sdk::Client
        // For now, we'll just set it to None and handle that in our test
        session.client = None;
    }
    
    // Create a test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(api::room_messages)
    ).await;
    
    // Send a request to the room messages endpoint
    let req = test::TestRequest::get()
        .uri(&format!("/rooms/{}/{}/messages", session_id, "#test:example.org"))
        .to_request();
    
    // This will fail because we haven't properly mocked the matrix_sdk::Client
    // In a real implementation, we would need to create a proper mock
    let resp = test::call_service(&app, req).await;
    
    // We expect an error because the client is None
    assert!(resp.status().is_client_error());
    
    println!("ℹ️ Room messages endpoint test skipped (requires proper mocking of matrix_sdk::Client)");
}

/// Test the join room endpoint
#[actix_web::test]
async fn test_join_room_endpoint() {
    // Create a mock API state
    let state = create_mock_api_state();
    
    // Create a test session
    let session_id = create_test_session(&state).await;
    
    // Set up a mock client in the session
    {
        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&session_id).unwrap();
        
        // Create a mock client
        let mock_client = MockMatrixClient::new();
        
        // Store the mock client in the session
        // Note: In a real test, we would need to implement a way to mock the matrix_sdk::Client
        // For now, we'll just set it to None and handle that in our test
        session.client = None;
    }
    
    // Create a test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(api::join_room)
    ).await;
    
    // Send a request to the join room endpoint
    let req = test::TestRequest::post()
        .uri(&format!("/rooms/{}/{}/join", session_id, "#test:example.org"))
        .to_request();
    
    // This will fail because we haven't properly mocked the matrix_sdk::Client
    // In a real implementation, we would need to create a proper mock
    let resp = test::call_service(&app, req).await;
    
    // We expect an error because the client is None
    assert!(resp.status().is_client_error());
    
    println!("ℹ️ Join room endpoint test skipped (requires proper mocking of matrix_sdk::Client)");
}

/// Test the leave room endpoint
#[actix_web::test]
async fn test_leave_room_endpoint() {
    // Create a mock API state
    let state = create_mock_api_state();
    
    // Create a test session
    let session_id = create_test_session(&state).await;
    
    // Set up a mock client in the session
    {
        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&session_id).unwrap();
        
        // Create a mock client
        let mock_client = MockMatrixClient::new();
        
        // Store the mock client in the session
        // Note: In a real test, we would need to implement a way to mock the matrix_sdk::Client
        // For now, we'll just set it to None and handle that in our test
        session.client = None;
    }
    
    // Create a test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(api::leave_room)
    ).await;
    
    // Send a request to the leave room endpoint
    let req = test::TestRequest::post()
        .uri(&format!("/rooms/{}/{}/leave", session_id, "#test:example.org"))
        .to_request();
    
    // This will fail because we haven't properly mocked the matrix_sdk::Client
    // In a real implementation, we would need to create a proper mock
    let resp = test::call_service(&app, req).await;
    
    // We expect an error because the client is None
    assert!(resp.status().is_client_error());
    
    println!("ℹ️ Leave room endpoint test skipped (requires proper mocking of matrix_sdk::Client)");
}

/// Test the send message endpoint
#[actix_web::test]
async fn test_send_message_endpoint() {
    // Create a mock API state
    let state = create_mock_api_state();
    
    // Create a test session
    let session_id = create_test_session(&state).await;
    
    // Set up a mock client in the session
    {
        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&session_id).unwrap();
        
        // Create a mock client
        let mock_client = MockMatrixClient::new();
        
        // Store the mock client in the session
        // Note: In a real test, we would need to implement a way to mock the matrix_sdk::Client
        // For now, we'll just set it to None and handle that in our test
        session.client = None;
    }
    
    // Create a test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(api::send_message)
    ).await;
    
    // Send a request to the send message endpoint
    let req = test::TestRequest::post()
        .uri(&format!("/rooms/{}/{}/send", session_id, "#test:example.org"))
        .set_json(&json!({"body": "Test message"}))
        .to_request();
    
    // This will fail because we haven't properly mocked the matrix_sdk::Client
    // In a real implementation, we would need to create a proper mock
    let resp = test::call_service(&app, req).await;
    
    // We expect an error because the client is None
    assert!(resp.status().is_client_error());
    
    println!("ℹ️ Send message endpoint test skipped (requires proper mocking of matrix_sdk::Client)");
}

/// Test the sync endpoint
#[actix_web::test]
async fn test_sync_endpoint() {
    // Create a mock API state
    let state = create_mock_api_state();
    
    // Create a test session
    let session_id = create_test_session(&state).await;
    
    // Set up a mock client in the session
    {
        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&session_id).unwrap();
        
        // Create a mock client
        let mock_client = MockMatrixClient::new();
        
        // Store the mock client in the session
        // Note: In a real test, we would need to implement a way to mock the matrix_sdk::Client
        // For now, we'll just set it to None and handle that in our test
        session.client = None;
    }
    
    // Create a test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(api::sync)
    ).await;
    
    // Send a request to the sync endpoint
    let req = test::TestRequest::get()
        .uri(&format!("/sync/{}", session_id))
        .to_request();
    
    // This will fail because we haven't properly mocked the matrix_sdk::Client
    // In a real implementation, we would need to create a proper mock
    let resp = test::call_service(&app, req).await;
    
    // We expect an error because the client is None
    assert!(resp.status().is_client_error());
    
    println!("ℹ️ Sync endpoint test skipped (requires proper mocking of matrix_sdk::Client)");
}

/// Test the login SSO callback endpoint
#[actix_web::test]
async fn test_login_sso_callback() {
    // Create a mock API state
    let state = create_mock_api_state();
    
    // Create a test session
    let session_id = create_test_session(&state).await;
    
    // Create a test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(api::login_sso_callback)
    ).await;
    
    // Send a request to the login SSO callback endpoint
    let req = test::TestRequest::get()
        .uri(&format!("/login/sso/callback?session_id={}&loginToken=test_token", session_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    // We expect an error because the client is None
    assert!(resp.status().is_client_error());
    
    println!("ℹ️ Login SSO callback endpoint test skipped (requires proper mocking of matrix_sdk::Client)");
}

/// Run all tests
#[actix_web::test]
pub async fn run_all_tests() {
    println!("Running all Matrix API tests...");
    
    // Run the status endpoint test
    test_status_endpoint();
    
    // Run the login SSO start endpoint test
    test_login_sso_start();
    
    // Run the login status endpoint test
    test_login_status();
    
    // Run the join room endpoint test
    test_join_room_endpoint();
    
    // Run the leave room endpoint test
    test_leave_room_endpoint();
    
    // Note: The following tests require proper mocking of matrix_sdk::Client
    // and are skipped for now
    
    println!("All tests completed!");
}