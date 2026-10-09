//! State مشترک برنامه

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::ConfigManager;
use crate::transfer::TransferManager;

/// اطلاعات یه فایل دریافتی (برای نمایش تو UI)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReceivedFile {
    pub name: String,
    pub size: u64,
    pub path: String,
    pub received_at: u64,
    pub duration_secs: f64,
}

/// اطلاعات یه فایل آماده برای ارسال به گوشی
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SharedFile {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub path: String,
    pub shared_at: u64,
}

/// State کلی برنامه
#[derive(Clone)]
pub struct AppState {
    pub token: Arc<String>,
    pub pair_code: Arc<String>,
    pub lan_ip: Arc<String>,
    pub transfer: TransferManager,
    pub config: ConfigManager,
    pub received_files: Arc<RwLock<VecDeque<ReceivedFile>>>,
    pub shared_files: Arc<RwLock<VecDeque<SharedFile>>>,
}

impl AppState {
    pub fn new(
        token: String,
        pair_code: String,
        lan_ip: String,
        transfer: TransferManager,
        config: ConfigManager,
        received_files: Vec<ReceivedFile>,
        shared_files: Vec<SharedFile>,
    ) -> Self {
        Self {
            token: Arc::new(token),
            pair_code: Arc::new(pair_code),
            lan_ip: Arc::new(lan_ip),
            transfer,
            config,
            received_files: Arc::new(RwLock::new(VecDeque::from(received_files))),
            shared_files: Arc::new(RwLock::new(VecDeque::from(shared_files))),
        }
    }

    pub fn mobile_url(&self) -> String {
        format!("http://{}:8080/m?t={}", self.lan_ip, self.token)
    }

    /// ذخیره‌ی لیست فایل‌های دریافتی
    pub async fn persist_received_files(&self) {
        let files: Vec<ReceivedFile> = self.received_files.read().await.iter().cloned().collect();
        if let Err(e) = self.config.set_received_files(files).await {
            eprintln!("⚠️ خطا در ذخیره‌ی فایل‌های دریافتی: {}", e);
        }
    }

    /// ذخیره‌ی لیست فایل‌های اشتراکی
    pub async fn persist_shared_files(&self) {
        let files: Vec<SharedFile> = self.shared_files.read().await.iter().cloned().collect();
        if let Err(e) = self.config.set_shared_files(files).await {
            eprintln!("⚠️ خطا در ذخیره‌ی فایل‌های اشتراکی: {}", e);
        }
    }
}