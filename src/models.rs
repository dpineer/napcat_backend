use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use pgvector::Vector;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// --- Configuration Models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub llm_api_key: String,
    pub llm_base_url: String,
    pub llm_model: String,
}

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
    pub message: Option<Vec<MessageElement>>,
    pub raw: Option<serde_json::Value>,
}

// 消息元素结构，支持多种消息类型
#[derive(Deserialize, Debug, Clone)]
pub struct MessageElement {
    pub r#type: String,
    pub data: serde_json::Value,
}

// 转发消息内容结构
#[derive(Deserialize, Debug)]
pub struct ForwardMessageContent {
    pub xml_content: String,
    pub res_id: String,
    pub file_name: String,
}

// 图片元素结构
#[derive(Deserialize, Debug)]
pub struct ImageElement {
    pub file: String,
    pub url: Option<String>,
    pub file_id: Option<String>,
    pub file_size: Option<i64>,
    pub pic_type: Option<String>,
    pub summary: Option<String>,
}

// 回复元素结构
#[derive(Deserialize, Debug)]
pub struct ReplyElement {
    pub id: String,
    pub source_msg_text: Option<String>,
    pub source_msg_text_elems: Option<Vec<ReplyTextElement>>,
}

#[derive(Deserialize, Debug)]
pub struct ReplyTextElement {
    pub reply_abs_elem_type: i32,
    pub text_elem_content: Option<String>,
}

// 图片处理结果
#[derive(Debug, Clone)]
pub struct ImageProcessingResult {
    pub has_image: bool,
    pub image_descriptions: Vec<String>,
    pub image_urls: Vec<String>,
    pub processing_status: ImageProcessingStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageProcessingStatus {
    Success,
    Failed(String),
    NotProcessed,
    ServiceUnavailable,
}

impl Default for ImageProcessingResult {
    fn default() -> Self {
        Self {
            has_image: false,
            image_descriptions: Vec::new(),
            image_urls: Vec::new(),
            processing_status: ImageProcessingStatus::NotProcessed,
        }
    }
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