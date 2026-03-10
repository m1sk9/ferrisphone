use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub user_id: String,
    pub model: String,
    pub user_message: String,
    pub model_response: String,
    pub timestamp: DateTime<Utc>,
    pub guild_id: String,
    pub channel_id: String,
    pub estimated_tokens: u32,
}

pub struct ChatLogger {
    dir: PathBuf,
}

impl ChatLogger {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub async fn append(&self, entry: &LogEntry) -> anyhow::Result<()> {
        let date = entry.timestamp.format("%Y-%m-%d").to_string();
        let path = self.dir.join(format!("{date}.json"));

        let mut entries = load_entries(&path).await?;
        entries.push(entry.clone());

        let json = serde_json::to_string_pretty(&entries)?;
        tokio::fs::create_dir_all(&self.dir).await?;
        super::atomic_write(&path, json.as_bytes()).await?;

        tracing::info!(
            user_id = %entry.user_id,
            model = %entry.model,
            guild_id = %entry.guild_id,
            channel_id = %entry.channel_id,
            estimated_tokens = entry.estimated_tokens,
            "Chat log recorded"
        );
        Ok(())
    }
}

async fn load_entries(path: &Path) -> anyhow::Result<Vec<LogEntry>> {
    if path.exists() {
        let content = tokio::fs::read_to_string(path).await?;
        let entries = serde_json::from_str(&content)?;
        Ok(entries)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(timestamp: DateTime<Utc>) -> LogEntry {
        LogEntry {
            user_id: "586824421470109716".to_string(),
            model: "claude-haiku".to_string(),
            user_message: "こんにちは".to_string(),
            model_response: "こんにちは！何かお手伝いできることはありますか？".to_string(),
            timestamp,
            guild_id: "123456789".to_string(),
            channel_id: "987654321".to_string(),
            estimated_tokens: 42,
        }
    }

    #[tokio::test]
    async fn append_creates_file_with_date_name() {
        let dir = tempfile::tempdir().unwrap();
        let logger = ChatLogger::new(dir.path());

        let ts = DateTime::parse_from_rfc3339("2026-03-10T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let entry = sample_entry(ts);
        logger.append(&entry).await.unwrap();

        let path = dir.path().join("2026-03-10.json");
        assert!(path.exists());

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        let entries: Vec<LogEntry> = serde_json::from_str(&content).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].user_message, "こんにちは");
    }

    #[tokio::test]
    async fn append_adds_to_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let logger = ChatLogger::new(dir.path());

        let ts = DateTime::parse_from_rfc3339("2026-03-10T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        logger.append(&sample_entry(ts)).await.unwrap();
        logger.append(&sample_entry(ts)).await.unwrap();

        let path = dir.path().join("2026-03-10.json");
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        let entries: Vec<LogEntry> = serde_json::from_str(&content).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn different_dates_create_separate_files() {
        let dir = tempfile::tempdir().unwrap();
        let logger = ChatLogger::new(dir.path());

        let ts1 = DateTime::parse_from_rfc3339("2026-03-10T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let ts2 = DateTime::parse_from_rfc3339("2026-03-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        logger.append(&sample_entry(ts1)).await.unwrap();
        logger.append(&sample_entry(ts2)).await.unwrap();

        assert!(dir.path().join("2026-03-10.json").exists());
        assert!(dir.path().join("2026-03-11.json").exists());
    }
}
