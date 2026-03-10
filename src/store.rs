pub mod config;
pub mod log;
pub mod memory;
pub mod user;

use std::path::{Path, PathBuf};

use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::RwLock;

pub struct JsonStore<T> {
    data: RwLock<T>,
    path: PathBuf,
}

impl<T: Serialize + DeserializeOwned + Default> JsonStore<T> {
    pub async fn load(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let data = if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            serde_json::from_str(&content)?
        } else {
            T::default()
        };
        Ok(Self {
            data: RwLock::new(data),
            path,
        })
    }

    pub async fn read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.data.read().await;
        f(&guard)
    }

    pub async fn write<R>(&self, f: impl FnOnce(&mut T) -> R) -> anyhow::Result<R> {
        let result = {
            let mut guard = self.data.write().await;
            f(&mut guard)
        };
        self.flush().await?;
        Ok(result)
    }

    pub async fn flush(&self) -> anyhow::Result<()> {
        let guard = self.data.read().await;
        let json = serde_json::to_string_pretty(&*guard)?;
        drop(guard);

        atomic_write(&self.path, json.as_bytes()).await
    }
}

pub(crate) async fn atomic_write(path: &Path, data: &[u8]) -> anyhow::Result<()> {
    // 親ディレクトリが存在しなければ作成
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let tmp_path = path.with_extension("tmp");
    tokio::fs::write(&tmp_path, data).await?;
    tokio::fs::rename(&tmp_path, path).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn load_creates_default_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.json");

        let store: JsonStore<HashMap<String, String>> = JsonStore::load(&path).await.unwrap();
        let is_empty = store.read(|data| data.is_empty()).await;
        assert!(is_empty);
    }

    #[tokio::test]
    async fn load_reads_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("existing.json");
        tokio::fs::write(&path, r#"{"key":"value"}"#).await.unwrap();

        let store: JsonStore<HashMap<String, String>> = JsonStore::load(&path).await.unwrap();
        let val = store.read(|data| data.get("key").cloned()).await;
        assert_eq!(val, Some("value".to_string()));
    }

    #[tokio::test]
    async fn write_and_flush_persists_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.json");

        let store: JsonStore<HashMap<String, String>> = JsonStore::load(&path).await.unwrap();
        store
            .write(|data| {
                data.insert("hello".to_string(), "world".to_string());
            })
            .await
            .unwrap();

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        let reloaded: HashMap<String, String> = serde_json::from_str(&content).unwrap();
        assert_eq!(reloaded.get("hello").unwrap(), "world");
    }

    #[tokio::test]
    async fn atomic_write_does_not_leave_tmp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("atomic.json");

        let store: JsonStore<HashMap<String, i32>> = JsonStore::load(&path).await.unwrap();
        store
            .write(|data| {
                data.insert("x".to_string(), 42);
            })
            .await
            .unwrap();

        let tmp_path = path.with_extension("tmp");
        assert!(!tmp_path.exists());
        assert!(path.exists());
    }

    #[tokio::test]
    async fn reload_after_write_returns_same_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reload.json");

        let store: JsonStore<HashMap<String, String>> = JsonStore::load(&path).await.unwrap();
        store
            .write(|data| {
                data.insert("a".to_string(), "b".to_string());
            })
            .await
            .unwrap();

        let store2: JsonStore<HashMap<String, String>> = JsonStore::load(&path).await.unwrap();
        let val = store2.read(|data| data.get("a").cloned()).await;
        assert_eq!(val, Some("b".to_string()));
    }
}
