use anyhow::Result;
use fastembed::{TextEmbedding, InitOptions};
use tracing::info;
use crate::db::DatabaseRepository;

pub struct KnowledgeBase {
    embedder: TextEmbedding,
    db: DatabaseRepository,
}

impl KnowledgeBase {
    pub fn new(db: DatabaseRepository) -> Result<Self> {
        info!("Loading embedding model (this may take a while on first run)...");
        // FastEmbed v5 initialization
        let model = TextEmbedding::try_new(InitOptions::default())?;
        Ok(Self { embedder: model, db })
    }

    pub async fn add_document(&mut self, text: &str) -> Result<()> {
        // Embed returns a generic error, map it to anyhow
        let embeddings = self.embedder.embed(vec![text], None)?;
        let vector = embeddings.first().ok_or_else(|| anyhow::anyhow!("No embedding generated"))?.clone();
        
        self.db.add_document(text, vector).await
    }

    pub async fn search(&mut self, query: &str, top_k: usize) -> Result<Vec<String>> {
        let embeddings = self.embedder.embed(vec![query], None)?;
        let vector = embeddings.first().ok_or_else(|| anyhow::anyhow!("No embedding generated"))?.clone();

        // Use a threshold of 0.6 to filter out irrelevant noise
        let results: Vec<crate::models::SearchResult> = self.db.search_documents(vector, top_k as i32, 0.6).await?;
        
        Ok(results.into_iter().map(|r| r.content).collect())
    }
}
