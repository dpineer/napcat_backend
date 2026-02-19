use anyhow::Result;
use pgvector::Vector;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tracing::{info, warn};
use crate::models::{Conversation, GroupContext, SearchResult, UserContext};

#[derive(Clone)]
pub struct DatabaseRepository {
    pool: PgPool,
}

impl DatabaseRepository {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Retry logic to prevent immediate failure if DB is starting up
        let mut pool = None;
        for i in 1..=5 {
            match PgPoolOptions::new().max_connections(10).connect(database_url).await {
                Ok(p) => {
                    pool = Some(p);
                    break;
                }
                Err(e) => {
                    warn!("Attempt {}/5: Failed to connect to DB: {}. Retrying in 5s...", i, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }

        let pool = pool.ok_or_else(|| anyhow::anyhow!("Failed to connect to database after retries"))?;
        
        let repo = Self { pool };
        repo.init_schema().await?;
        Ok(repo)
    }

    async fn init_schema(&self) -> Result<()> {
        info!("Running database migrations...");
        
        // 1. Enable Extension
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector").execute(&self.pool).await?;

        // 2. Create Tables
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS documents (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                content TEXT NOT NULL,
                embedding vector(384),
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id BIGINT,
                group_id BIGINT,
                message_type VARCHAR(20),
                raw_message TEXT NOT NULL,
                bot_response TEXT,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS user_contexts (
                user_id BIGINT PRIMARY KEY,
                context_data JSONB DEFAULT '{}'::jsonb,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS group_contexts (
                group_id BIGINT PRIMARY KEY,
                context_data JSONB DEFAULT '{}'::jsonb,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS system_config (
                config_key VARCHAR(100) PRIMARY KEY,
                config_value JSONB NOT NULL
            )
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // --- Document Operations ---

    pub async fn add_document(&self, content: &str, embedding: Vec<f32>) -> Result<()> {
        let vector = Vector::from(embedding);
        sqlx::query("INSERT INTO documents (content, embedding) VALUES ($1, $2)")
            .bind(content)
            .bind(vector)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn search_documents(&self, embedding: Vec<f32>, limit: i32, threshold: f32) -> Result<Vec<SearchResult>> {
        let vector = Vector::from(embedding);
        // Using Cosine Distance (<=>). 
        // 1 - distance = similarity. We want similarity > threshold, so distance < (1 - threshold).
        let distance_threshold = 1.0 - threshold;

        let rows = sqlx::query(r#"
            SELECT content, (1 - (embedding <=> $1)) as similarity
            FROM documents
            WHERE (embedding <=> $1) < $2
            ORDER BY embedding <=> $1 ASC
            LIMIT $3
        "#)
        .bind(vector)
        .bind(distance_threshold)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let results = rows.into_iter().map(|row| SearchResult {
            content: row.get("content"),
            similarity: row.get::<f64, _>("similarity") as f32, // 转换为f32
        }).collect();

        Ok(results)
    }

    // --- Conversation Operations ---

    pub async fn add_conversation(&self, user_id: Option<i64>, group_id: Option<i64>, msg: &str, response: Option<&str>) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO conversations (user_id, group_id, raw_message, bot_response)
            VALUES ($1, $2, $3, $4)
        "#)
        .bind(user_id)
        .bind(group_id)
        .bind(msg)
        .bind(response)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_recent_conversations(&self, user_id: Option<i64>, group_id: Option<i64>, limit: i64) -> Result<Vec<Conversation>> {
        // Simple logic: if group_id exists, fetch group history, else fetch user history
        let query = if let Some(gid) = group_id {
            sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE group_id = $1 ORDER BY created_at DESC LIMIT $2").bind(gid).bind(limit)
        } else if let Some(uid) = user_id {
            sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2").bind(uid).bind(limit)
        } else {
            return Ok(vec![]);
        };

        let mut conversations = query.fetch_all(&self.pool).await?;
        conversations.reverse(); // Return in chronological order
        Ok(conversations)
    }

    // --- Context Operations ---

    pub async fn get_user_context(&self, user_id: i64) -> Result<Option<serde_json::Value>> {
        let res: Option<UserContext> = sqlx::query_as("SELECT * FROM user_contexts WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res.map(|c| c.context_data))
    }

    pub async fn get_group_context(&self, group_id: i64) -> Result<Option<serde_json::Value>> {
        let res: Option<GroupContext> = sqlx::query_as("SELECT * FROM group_contexts WHERE group_id = $1")
            .bind(group_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res.map(|c| c.context_data))
    }

    // --- 知识库管理功能 ---

    pub async fn list_documents(&self) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query(r#"
            SELECT id, content, created_at 
            FROM documents 
            ORDER BY created_at DESC
        "#)
        .fetch_all(&self.pool)
        .await?;

        let documents = rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.get::<uuid::Uuid, _>("id").to_string(),
                "content": row.get::<String, _>("content"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_string()
            })
        }).collect();

        Ok(documents)
    }

    pub async fn get_document_by_id(&self, id: &str) -> Result<Option<serde_json::Value>> {
        let uuid = uuid::Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        let row = sqlx::query(r#"
            SELECT id, content, created_at 
            FROM documents 
            WHERE id = $1
        "#)
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let document = serde_json::json!({
                    "id": row.get::<uuid::Uuid, _>("id").to_string(),
                    "content": row.get::<String, _>("content"),
                    "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_string()
                });
                Ok(Some(document))
            }
            None => Ok(None)
        }
    }

    pub async fn update_document(&self, id: &str, content: &str) -> Result<()> {
        let uuid = uuid::Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        // 首先获取旧的embedding（如果有的话）或者重新生成
        let old_doc = sqlx::query("SELECT embedding FROM documents WHERE id = $1")
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await?;

        if old_doc.is_none() {
            return Err(anyhow::anyhow!("Document not found"));
        }

        // 更新文档内容，保持原有的embedding（简化处理）
        // 在实际应用中，可能需要重新生成embedding
        sqlx::query("UPDATE documents SET content = $1 WHERE id = $2")
            .bind(content)
            .bind(uuid)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_document(&self, id: &str) -> Result<()> {
        let uuid = uuid::Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        let result = sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Document not found"));
        }

        Ok(())
    }
}
