//! مدیریت config و persistence
//! 
//! config رو تو `%APPDATA%\com.parsa.quickdrop\config.json` ذخیره می‌کنه.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::{ReceivedFile, SharedFile};

/// یه config قابل ذخیره
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub save_path: Option<String>,
    pub received_files: Vec<ReceivedFile>,
    pub shared_files: Vec<SharedFile>,
}

/// مدیر config
#[derive(Clone)]
pub struct ConfigManager {
    path: PathBuf,
    data: Arc<RwLock<Config>>,
}

impl ConfigManager {
    /// ساخت ConfigManager و لود کردن config موجود
    pub fn new() -> Self {
        let path = Self::config_path();
        let data = Self::load_from_disk(&path).unwrap_or_default();

        Self {
            path,
            data: Arc::new(RwLock::new(data)),
        }
    }

    /// مسیر فایل config
    fn config_path() -> PathBuf {
        let app_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.parsa.quickdrop");
        app_dir.join("config.json")
    }

    /// لود کردن از دیسک
    fn load_from_disk(path: &PathBuf) -> Option<Config> {
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// ذخیره به دیسک
    async fn save_to_disk(&self) -> Result<(), String> {
        let data = self.data.read().await.clone();

        // اطمینان از وجود پوشه
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("خطا در ساخت پوشه: {}", e))?;
        }

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("خطا در تبدیل JSON: {}", e))?;

        std::fs::write(&self.path, json)
            .map_err(|e| format!("خطا در نوشتن فایل: {}", e))?;

        Ok(())
    }

    /// گرفتن مسیر ذخیره
    pub async fn get_save_path(&self) -> Option<String> {
        self.data.read().await.save_path.clone()
    }

    /// تنظیم مسیر ذخیره + ذخیره
    pub async fn set_save_path(&self, path: String) -> Result<(), String> {
        {
            let mut data = self.data.write().await;
            data.save_path = Some(path);
        }
        self.save_to_disk().await
    }

    /// گرفتن فایل‌های دریافتی
    pub async fn get_received_files(&self) -> Vec<ReceivedFile> {
        self.data.read().await.received_files.clone()
    }

    /// آپدیت فایل‌های دریافتی + ذخیره
    pub async fn set_received_files(&self, files: Vec<ReceivedFile>) -> Result<(), String> {
        {
            let mut data = self.data.write().await;
            data.received_files = files;
        }
        self.save_to_disk().await
    }

    /// گرفتن فایل‌های اشتراکی
    pub async fn get_shared_files(&self) -> Vec<SharedFile> {
        self.data.read().await.shared_files.clone()
    }

    /// آپدیت فایل‌های اشتراکی + ذخیره
    pub async fn set_shared_files(&self, files: Vec<SharedFile>) -> Result<(), String> {
        {
            let mut data = self.data.write().await;
            data.shared_files = files;
        }
        self.save_to_disk().await
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}