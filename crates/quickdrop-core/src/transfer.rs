//! Transfer Engine - مدیریت انتقال فایل

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// اندازه‌ی هر تیکه (۱ مگابایت)
pub const CHUNK_SIZE: usize = 1024 * 1024;

/// پسوند فایل موقت
const TEMP_EXTENSION: &str = "part";

/// یه session آپلود فعال
#[derive(Debug, Clone)]
pub struct UploadSession {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub received: u64,
    pub temp_path: PathBuf,
    pub final_path: PathBuf,
    pub token: String,
    pub started_at: Instant,
}

impl UploadSession {
    pub fn progress_percent(&self) -> f64 {
        if self.file_size == 0 {
            return 0.0;
        }
        (self.received as f64 / self.file_size as f64) * 100.0
    }

    pub fn average_speed(&self) -> f64 {
        let elapsed = self.started_at.elapsed().as_secs_f64();
        if elapsed < 0.1 {
            return 0.0;
        }
        self.received as f64 / elapsed
    }
}

/// مدیر Transfer
#[derive(Clone)]
pub struct TransferManager {
    sessions: Arc<RwLock<HashMap<String, UploadSession>>>,
    save_path: Arc<RwLock<PathBuf>>,
}

impl TransferManager {
    /// ساخت TransferManager با مسیر سفارشی (یا پیش‌فرض)
    pub fn new(custom_save_path: Option<String>) -> Self {
        let default_path = match custom_save_path {
            Some(p) => PathBuf::from(p),
            None => Self::default_save_path(),
        };

        if let Err(e) = std::fs::create_dir_all(&default_path) {
            eprintln!("⚠️ نتونستم پوشه‌ی ذخیره رو بسازم: {}", e);
        }

        println!("📁 مسیر ذخیره: {}", default_path.display());

        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            save_path: Arc::new(RwLock::new(default_path)),
        }
    }

    fn default_save_path() -> PathBuf {
        if let Some(downloads) = dirs::download_dir() {
            downloads.join("QuickDrop")
        } else {
            PathBuf::from("./QuickDrop")
        }
    }

    pub async fn get_save_path(&self) -> PathBuf {
        self.save_path.read().await.clone()
    }

    pub async fn set_save_path(&self, new_path: PathBuf) -> Result<(), String> {
        std::fs::create_dir_all(&new_path)
            .map_err(|e| format!("نتونستم پوشه رو بسازم: {}", e))?;

        let mut path = self.save_path.write().await;
        *path = new_path.clone();

        println!("📁 مسیر ذخیره تغییر کرد: {}", new_path.display());
        Ok(())
    }

    pub async fn create_session(
        &self,
        file_name: String,
        file_size: u64,
        token: String,
    ) -> Result<UploadSession, String> {
        let id = uuid::Uuid::new_v4().to_string();

        let save_dir = self.get_save_path().await;
        let final_path = Self::resolve_filename(&save_dir, &file_name);

        let temp_path = final_path.with_extension(
            format!(
                "{}.{}",
                final_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or(""),
                TEMP_EXTENSION
            ),
        );

        let session = UploadSession {
            id: id.clone(),
            file_name,
            file_size,
            received: 0,
            temp_path,
            final_path,
            token,
            started_at: Instant::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(id.clone(), session.clone());

        println!(
            "📥 آپلود جدید: {} ({} بایت) - ID: {}",
            session.file_name,
            file_size,
            &id[..8]
        );

        Ok(session)
    }

    pub async fn get_session(&self, id: &str) -> Option<UploadSession> {
        let sessions = self.sessions.read().await;
        sessions.get(id).cloned()
    }

    pub async fn get_latest_active(&self) -> Option<UploadSession> {
        let sessions = self.sessions.read().await;
        sessions.values().max_by_key(|s| s.started_at).cloned()
    }

    pub async fn append_chunk(
        &self,
        id: &str,
        data: &[u8],
    ) -> Result<(u64, bool), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| format!("session {} پیدا نشد", id))?;

        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&session.temp_path)
            .await
            .map_err(|e| format!("خطا در باز کردن فایل موقت: {}", e))?;

        file.write_all(data)
            .await
            .map_err(|e| format!("خطا در نوشتن: {}", e))?;

        file.flush()
            .await
            .map_err(|e| format!("خطا در flush: {}", e))?;

        session.received += data.len() as u64;

        let received = session.received;
        let total = session.file_size;
        let is_complete = received >= total;

        Ok((received, is_complete))
    }

    pub async fn complete_session(&self, id: &str) -> Result<PathBuf, String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .remove(id)
            .ok_or_else(|| format!("session {} پیدا نشد", id))?;

        tokio::fs::rename(&session.temp_path, &session.final_path)
            .await
            .map_err(|e| format!("خطا در تغییر نام: {}", e))?;

        println!("✅ آپلود کامل شد: {}", session.final_path.display());
        println!(
            "   📦 {} بایت در {:.1} ثانیه ({:.1} MB/s)",
            session.received,
            session.started_at.elapsed().as_secs_f64(),
            session.average_speed() / 1_048_576.0
        );

        Ok(session.final_path)
    }

    #[allow(dead_code)]
    pub async fn cancel_session(&self, id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(id) {
            let _ = tokio::fs::remove_file(&session.temp_path).await;
            println!("🗑 آپلود لغو شد: {}", session.file_name);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn active_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    fn resolve_filename(dir: &Path, file_name: &str) -> PathBuf {
        let base_path = dir.join(file_name);

        if !base_path.exists() {
            return base_path;
        }

        let path = Path::new(file_name);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        for i in 1..10000 {
            let new_name = if ext.is_empty() {
                format!("{} ({})", stem, i)
            } else {
                format!("{} ({}).{}", stem, i, ext)
            };

            let candidate = dir.join(&new_name);
            if !candidate.exists() {
                return candidate;
            }
        }

        let new_name = format!("{}_{}", stem, uuid::Uuid::new_v4().simple());
        dir.join(new_name)
    }
}

impl Default for TransferManager {
    fn default() -> Self {
        Self::new(None)
    }
}