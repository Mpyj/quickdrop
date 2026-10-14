//! Discovery Module
//! 
//! این ماژول مسئول پیدا کردن دستگاه‌های QuickDrop در اطراف است.
//! 
//! استراتژی:
//! 1. Bluetooth LE (اولویت اول)
//! 2. mDNS (فالبک)
//! 3. UDP Broadcast (فالبک)
//! 4. QR Code (فالبک نهایی)

pub mod events;
pub mod manager;

pub use events::*;
pub use manager::DiscoveryManager;

use serde::{Deserialize, Serialize};

/// خطای Discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryError {
    /// Bluetooth خاموشه
    BluetoothDisabled,
    /// Wi-Fi خاموشه
    WifiDisabled,
    /// Permission نداریم
    PermissionDenied(String),
    /// خطای scan
    ScanFailed(String),
    /// خطای اتصال
    ConnectionFailed(String),
    /// Timeout
    Timeout,
    /// خطای داخلی
    Internal(String),
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BluetoothDisabled => write!(f, "Bluetooth خاموش است"),
            Self::WifiDisabled => write!(f, "Wi-Fi خاموش است"),
            Self::PermissionDenied(msg) => write!(f, "دسترسی رد شد: {}", msg),
            Self::ScanFailed(msg) => write!(f, "خطا در scan: {}", msg),
            Self::ConnectionFailed(msg) => write!(f, "خطا در اتصال: {}", msg),
            Self::Timeout => write!(f, "زمان به پایان رسید"),
            Self::Internal(msg) => write!(f, "خطای داخلی: {}", msg),
        }
    }
}

impl std::error::Error for DiscoveryError {}