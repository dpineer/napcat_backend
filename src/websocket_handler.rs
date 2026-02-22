use axum::extract::ws::{WebSocket, Message};
use axum::extract::WebSocketUpgrade;
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

/// Message type enum for WebSocket communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Group,
    History,
    Conversation,
    Config,
    Control,
    Data,
}

/// WebSocket message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WSMessage {
    pub msg_type: String,
    pub action: Option<String>,
    pub text: Option<String>,
    pub audio: Option<Vec<f32>>,
    pub images: Option<Vec<String>>,
    pub history_uid: Option<String>,
    pub file: Option<String>,
    pub display_text: Option<DisplayText>,
}

/// Display text structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayText {
    pub text: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
}

/// WebSocket handler for managing connections and message routing
pub struct WebSocketHandler {
    /// Store active connections
    pub connections: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>,
    /// Broadcast channel for group messages
    pub broadcast_tx: broadcast::Sender<String>,
}

impl WebSocketHandler {
    /// Create a new WebSocket handler
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// Handle new WebSocket connection
    pub async fn handle_connection(&self, ws: WebSocket) {
        // Accept the WebSocket connection
        let (mut sender, mut receiver) = ws.split();
        
        // Create a channel for this connection
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Add to connections
        let client_id = {
            let mut connections = self.connections.lock().await;
            // Generate a unique ID for this connection
            let client_id = uuid::Uuid::new_v4().to_string();
            connections.insert(client_id.clone(), tx);
            client_id
        };

        // Forward messages from broadcast to this connection
        let mut broadcast_rx = self.broadcast_tx.subscribe();
        
        // Create a channel to merge broadcast and internal messages
        let (merged_tx, mut merged_rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Forward broadcast messages to merged channel
        let merged_tx_clone = merged_tx.clone();
        let broadcast_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle broadcast messages
                    Ok(msg) = broadcast_rx.recv() => {
                        if merged_tx_clone.send(Message::Text(msg)).is_err() {
                            break;
                        }
                    }
                    // Handle internal messages
                    Some(msg) = rx.recv() => {
                        if merged_tx_clone.send(msg).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        
        // Handle sending all messages to the client
        let send_task = tokio::spawn(async move {
            while let Some(msg) = merged_rx.recv().await {
                if sender.send(msg).await.is_err() {
                    break;
                }
            }
        });
        
        // Handle receiving messages from the client
        let recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Text(text) = msg {
                    // Process the received message
                    println!("Received message: {}", text);
                }
            }
        });
        
        // Wait for tasks to complete
        let _handle = tokio::spawn(async move {
            let _ = tokio::join!(broadcast_task, send_task, recv_task);
        });
    }

    /// Send a message to a specific client
    pub async fn send_to_client(&self, client_id: &str, message: Message) -> Result<(), ()> {
        let connections = self.connections.lock().await;
        if let Some(sender) = connections.get(client_id) {
            sender.send(message).map_err(|_| ())
        } else {
            Err(())
        }
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, message: String) -> Result<(), ()> {
        self.broadcast_tx.send(message).map(|_| ()).map_err(|_| ())
    }

    /// Remove a client from connections
    pub async fn remove_client(&self, client_id: &str) {
        let mut connections = self.connections.lock().await;
        connections.remove(client_id);
    }
}