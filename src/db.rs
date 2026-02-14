use std::path::Path;
use turso::Value;

/// A single transcript row from the `items` table.
#[derive(Debug, Clone)]
pub struct TranscriptRow {
    pub identifier: i64,
    pub model: String,
    pub host: String,
    pub original_source_link: String,
    pub output_language: String,
    pub summary: String,
    pub summary_done: bool,
    pub summary_input_tokens: i64,
    pub summary_output_tokens: i64,
    pub transcript: String,
    pub timestamps: String,
    pub timestamped_summary_in_youtube_format: String,
    pub cost: f64,
    pub has_embedding: bool,
    pub embedding_model: String,
}

/// Lightweight row for list display (avoids loading full transcript text).
#[derive(Debug, Clone)]
pub struct TranscriptListItem {
    pub identifier: i64,
    pub host: String,
    pub summary_preview: String,
    pub cost: f64,
    pub has_embedding: bool,
    pub model: String,
    pub original_source_link: String, // Added to cache for in-memory filtering
}

/// Result of a vector similarity search.
#[derive(Debug, Clone)]
pub struct SimilarResult {
    pub identifier: i64,
    pub host: String,
    pub summary_preview: String,
    pub distance: f64,
    pub original_source_link: String,
}

// ── Value extraction helpers ──

fn val_i64(v: &Value) -> i64 {
    match v {
        Value::Integer(i) => *i,
        _ => 0,
    }
}

fn val_f64(v: &Value) -> f64 {
    match v {
        Value::Real(f) => *f,
        Value::Integer(i) => *i as f64,
        _ => 0.0,
    }
}

fn val_string(v: &Value) -> String {
    match v {
        Value::Text(s) => s.clone(),
        _ => String::new(),
    }
}

fn val_bool(v: &Value) -> bool {
    match v {
        Value::Integer(i) => *i != 0,
        _ => false,
    }
}

/// Database handle wrapping a turso connection.
pub struct Database {
    conn: turso::Connection,
}

impl Database {
    /// Open a local SQLite database file via Turso.
    pub async fn open(path: &Path) -> turso::Result<Self> {
        let path_str = path.to_string_lossy().to_string();
        let db = turso::Builder::new_local(&path_str).build().await?;
        let conn = db.connect()?;
        Ok(Database { conn })
    }

    /// Load all transcripts metadata for in-memory caching.
    pub async fn list_all_transcripts(&self) -> turso::Result<Vec<TranscriptListItem>> {
        let mut items = Vec::new();
        let mut rows = self.conn.query(
            "SELECT identifier, host, substr(summary, 1, 500), cost, \
             CASE WHEN embedding IS NOT NULL THEN 1 ELSE 0 END, model, \
             COALESCE(original_source_link, '') \
             FROM items ORDER BY identifier",
            (),
        ).await?;

        while let Some(row) = rows.next().await? {
            items.push(TranscriptListItem {
                identifier: val_i64(&row.get_value(0)?),
                host: val_string(&row.get_value(1)?),
                summary_preview: val_string(&row.get_value(2)?),
                cost: val_f64(&row.get_value(3)?),
                has_embedding: val_i64(&row.get_value(4)?) == 1,
                model: val_string(&row.get_value(5)?),
                original_source_link: val_string(&row.get_value(6)?),
            });
        }
        Ok(items)
    }

    /// Get a single transcript by identifier.
    pub async fn get_transcript(&self, id: i64) -> turso::Result<Option<TranscriptRow>> {
        let mut rows = self
            .conn
            .query(
                "SELECT identifier, model, host, \
                 COALESCE(original_source_link, ''), COALESCE(output_language, ''), \
                 COALESCE(summary, ''), summary_done, \
                 COALESCE(summary_input_tokens, 0), COALESCE(summary_output_tokens, 0), \
                 COALESCE(transcript, ''), COALESCE(timestamps, ''), \
                 COALESCE(timestamped_summary_in_youtube_format, ''), \
                 COALESCE(cost, 0), \
                 CASE WHEN embedding IS NOT NULL THEN 1 ELSE 0 END, \
                 COALESCE(embedding_model, '') \
                 FROM items WHERE identifier = ?1",
                turso::params::Params::Positional(vec![Value::Integer(id)]),
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(TranscriptRow {
                identifier: val_i64(&row.get_value(0)?),
                model: val_string(&row.get_value(1)?),
                host: val_string(&row.get_value(2)?),
                original_source_link: val_string(&row.get_value(3)?),
                output_language: val_string(&row.get_value(4)?),
                summary: val_string(&row.get_value(5)?),
                summary_done: val_bool(&row.get_value(6)?),
                summary_input_tokens: val_i64(&row.get_value(7)?),
                summary_output_tokens: val_i64(&row.get_value(8)?),
                transcript: val_string(&row.get_value(9)?),
                timestamps: val_string(&row.get_value(10)?),
                timestamped_summary_in_youtube_format: val_string(&row.get_value(11)?),
                cost: val_f64(&row.get_value(12)?),
                has_embedding: val_i64(&row.get_value(13)?) == 1,
                embedding_model: val_string(&row.get_value(14)?),
            }))
        } else {
            Ok(None)
        }
    }

    /// Find transcripts similar to the given one using cosine distance.
    /// Uses vector_slice(..., 0, 768) to handle Matryoshka dimension mismatch.
    pub async fn find_similar(
        &self,
        source_id: i64,
        limit: i64,
    ) -> turso::Result<Vec<SimilarResult>> {
        let mut results = Vec::new();

        // Slice both source and target embeddings to 768 dimensions for comparison.
        // This allows 3072-dim and 768-dim embeddings to be compared.
        let mut rows = self
            .conn
            .query(
                "SELECT t.identifier, t.host, substr(t.summary, 1, 500), \
                 vector_distance_cos(vector_slice(t.embedding, 0, 768), vector_slice(s.embedding, 0, 768)) AS dist, \
                 COALESCE(t.original_source_link, '') \
                 FROM items t, (SELECT embedding FROM items WHERE identifier = ?1) s \
                 WHERE t.embedding IS NOT NULL AND t.identifier != ?1 \
                 ORDER BY dist \
                 LIMIT ?2",
                turso::params::Params::Positional(vec![
                    Value::Integer(source_id),
                    Value::Integer(limit),
                ]),
            )
            .await?;

        while let Some(row) = rows.next().await? {
            results.push(SimilarResult {
                identifier: val_i64(&row.get_value(0)?),
                host: val_string(&row.get_value(1)?),
                summary_preview: val_string(&row.get_value(2)?),
                distance: val_f64(&row.get_value(3)?),
                original_source_link: val_string(&row.get_value(4)?),
            });
        }

        Ok(results)
    }
}
