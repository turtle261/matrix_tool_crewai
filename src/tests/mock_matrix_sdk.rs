//! Mock implementation of the Matrix SDK client
//!
//! This module provides a mock implementation of the Matrix SDK client for testing purposes.

use std::collections::HashMap;
use std::sync::Arc;
use matrix_sdk::{Client, Room};
use matrix_sdk::ruma::{OwnedRoomId, OwnedEventId, OwnedUserId, RoomId, UserId};
use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEventContent};
use matrix_sdk::ruma::events::SyncMessageEvent;
use matrix_sdk::ruma::events::AnyMessageEvent;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::room::{MessagesOptions, RoomMember};
use matrix_sdk::sync::SyncResponse;
use tokio::sync::{Mutex, RwLock};
use url::Url;
use async_trait::async_trait;

/// A mock implementation of the Matrix SDK Client
pub struct MockClient {
    /// Rooms in the client, keyed by room ID
    rooms: RwLock<HashMap<OwnedRoomId, MockRoom>>,
    /// User ID of the client
    user_id: OwnedUserId,
}

/// A mock implementation of a Matrix room
pub struct MockRoom {
    /// Room ID
    room_id: OwnedRoomId,
    /// Messages in the room
    messages: Vec<MockMessage>,
    /// Members in the room
    members: HashMap<OwnedUserId, MockRoomMember>,
}

/// A mock implementation of a Matrix message
pub struct MockMessage {
    /// Sender of the message
    sender: OwnedUserId,
    /// Content of the message
    content: RoomMessageEventContent,
    /// Event ID
    event_id: OwnedEventId,
    /// Timestamp of the message
    timestamp: u64,
}

/// A mock implementation of a Matrix room member
pub struct MockRoomMember {
    /// User ID of the member
    user_id: OwnedUserId,
    /// Display name of the member
    display_name: Option<String>,
}

impl MockClient {
    /// Create a new mock client
    pub fn new(user_id: &str) -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            user_id: user_id.parse().unwrap(),
        }
    }

    /// Add a room to the client
    pub async fn add_room(&self, room_id: &str) -> MockRoom {
        let room_id: OwnedRoomId = room_id.parse().unwrap();
        let room = MockRoom {
            room_id: room_id.clone(),
            messages: Vec::new(),
            members: HashMap::new(),
        };
        
        self.rooms.write().await.insert(room_id.clone(), room.clone());
        room
    }

    /// Get a room from the client
    pub async fn get_room(&self, room_id: &str) -> Option<MockRoom> {
        let room_id: OwnedRoomId = room_id.parse().unwrap();
        self.rooms.read().await.get(&room_id).cloned()
    }
}

impl Clone for MockRoom {
    fn clone(&self) -> Self {
        Self {
            room_id: self.room_id.clone(),
            messages: self.messages.clone(),
            members: self.members.clone(),
        }
    }
}

impl Clone for MockMessage {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            content: self.content.clone(),
            event_id: self.event_id.clone(),
            timestamp: self.timestamp,
        }
    }
}

/// Create a mock Matrix client
pub fn create_mock_client() -> Client {
    // This is a placeholder. In a real implementation, we would need to mock the matrix_sdk::Client
    // which is quite complex. For now, we'll just create a real client with a fake homeserver URL.
    let homeserver_url = Url::parse("https://example.org").unwrap();
    Client::new(homeserver_url).unwrap()
}

/// Create a mock room
pub fn create_mock_room(client: &Client, room_id: &str) -> Room {
    // This is a placeholder. In a real implementation, we would need to mock the matrix_sdk::Room
    // which is quite complex. For now, we'll just return a dummy value.
    unimplemented!("Mock room creation not implemented")
}

/// Create a mock message
pub fn create_mock_message(_sender: &str, _body: &str, _event_id: &str, _timestamp: u64) {
    // This is a placeholder. In a real implementation, we would need to mock the matrix_sdk message events
    // which is quite complex. For now, we'll just return a dummy value.
    unimplemented!("Mock message creation not implemented")
}

/// Create a mock sync response
pub fn create_mock_sync_response() -> SyncResponse {
    // This is a placeholder. In a real implementation, we would need to mock the matrix_sdk::SyncResponse
    // which is quite complex. For now, we'll just return a dummy value.
    unimplemented!("Mock sync response creation not implemented")
}