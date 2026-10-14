//! Connection Module
//! 
//! این ماژول مسئول:
//! - مدیریت اتصال‌های فعال
//! - نگه‌داشتن لیست دستگاه‌های متصل
//! - مدیریت Hotspot
//! - آماده‌سازی برای Transfer

pub mod events;
pub mod manager;

// Re-exports
pub use events::{ConnectionEvent, ConnectionState};
pub use manager::{ActiveConnection, ConnectionManager};

use serde::{Deserialize, Serialize};

/// خطای Connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionError {
    DeviceNotFound(String),
    ConnectionFailed(String),
    HotspotFailed(String),
    Timeout,
    Internal(String),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound(msg) => write!(f, "دستگاه پیدا نشد: {}", msg),
            Self::ConnectionFailed(msg) => write!(f, "خطا در اتصال: {}", msg),
            Self::HotspotFailed(msg) => write!(f, "Hotspot فعال نشد: {}", msg),
            Self::Timeout => write!(f, "زمان به پایان رسید"),
            Self::Internal(msg) => write!(f, "خطای داخلی: {}", msg),
        }
    }
}

impl std::error::Error for ConnectionError {}