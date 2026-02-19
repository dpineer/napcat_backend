use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use pgvector::Vector;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// --- Database Models ---

#[derive(FromRow, Debug)]
pub struct Document {
    pub id: Uuid,
    pub content: String,
    // pgvector maps 'vector' column to pgvector::Vector
    pub embedding: Option<Vector>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub content: String,
    pub similarity: f32,
}

#[derive(FromRow, Debug)]
pub struct Conversation {
    pub id: Uuid,
    pub user_id: Option<i64>,
    pub group_id: Option<i64>,
    pub raw_message: String,
    pub bot_response: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(FromRow, Debug)]
pub struct UserContext {
    pub user_id: i64,
    pub context_data: serde_json::Value,
}

#[derive(FromRow, Debug)]
pub struct GroupContext {
    pub group_id: i64,
    pub context_data: serde_json::Value,
}

#[derive(FromRow, Debug)]
pub struct SystemConfig {
    pub config_key: String,
    pub config_value: serde_json::Value,
}

// --- OneBot V11 Models ---

#[derive(Deserialize, Debug)]
pub struct OneBotEvent {
    pub post_type: Option<String>,
    pub message_type: Option<String>,
    pub raw_message: Option<String>,
    pub user_id: Option<i64>,
    pub group_id: Option<i64>,
}

// Array format support for NapCat
#[derive(Deserialize, Debug)]
pub struct NapCatArrayEvent(
    pub String, // Event type (e.g., "message")
    pub serde_json::Value // Event data
);

#[derive(Serialize)]
pub struct ReplyPayload {
    pub action: String,
    pub params: ReplyParams,
}

#[derive(Serialize)]
pub struct ReplyParams {
    pub user_id: Option<i64>,
    pub group_id: Option<i64>,
    pub message: String,
}