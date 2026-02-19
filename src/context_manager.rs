use anyhow::Result;
use crate::db::DatabaseRepository;

pub struct ContextManager {
    db: DatabaseRepository,
}

impl ContextManager {
    pub fn new(db: DatabaseRepository) -> Self {
        Self { db }
    }

    pub async fn record_interaction(&self, user_id: Option<i64>, group_id: Option<i64>, input: &str, output: &str) -> Result<()> {
        self.db.add_conversation(user_id, group_id, input, Some(output)).await
    }

    pub async fn get_rag_context(&self, user_id: Option<i64>, group_id: Option<i64>) -> Result<String> {
        let mut context_parts = Vec::new();

        // 1. Get recent conversation history
        let history: Vec<crate::models::Conversation> = self.db.get_recent_conversations(user_id, group_id, 5).await?;
        if !history.is_empty() {
            context_parts.push("Recent Conversation History:".to_string());
            for msg in history {
                context_parts.push(format!("User: {}\nAssistant: {}", msg.raw_message, msg.bot_response.unwrap_or_default()));
            }
        }

        // 2. Get static user context (if exists)
        if let Some(uid) = user_id {
            if let Some(data) = self.db.get_user_context(uid).await? {
                context_parts.push(format!("User Profile: {}", data));
            }
        }

        Ok(context_parts.join("\n---\n"))
    }
}
