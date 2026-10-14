//! Platform Abstraction Layer
//! 
//! این ماژول یه interface مشترک برای همه‌ی پلتفرم‌ها تعریف می‌کنه.
//! هر پلتفرم (Windows, Android, macOS, Linux) implementation خودش رو داره.
//! 
//! طراحی:
//! - `DiscoveryProvider` → پیدا کردن دستگاه‌ها
//! - `HotspotProvider` → مدیریت Hotspot
//! - `PermissionProvider` → مدیریت دسترسی‌ها
//! - `SystemStateProvider` → وضعیت Bluetooth/Wi-Fi

pub mod permissions;
pub mod types;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "android")]
pub mod android;

use async_trait::async_trait;

// ============================================
// Re-exports
// ============================================

pub use permissions::{
    BluetoothState,
    PermissionProvider,
    PermissionStatus,
    PermissionType,
    SystemStateProvider,
    WifiState,
};

pub use types::{
    BleMessage,
    ConnectionInfo,
    DeviceType,
    DiscoveredDevice,
    HotspotInfo,
    HotspotStatus,
};

// ============================================
// Trait — DiscoveryProvider
// ============================================

/// Trait مشترک Discovery
/// 
/// این trait مسئول پیدا کردن دستگاه‌های QuickDrop اطراف است.
/// از Bluetooth LE برای Discovery استفاده می‌کنه.
#[async_trait]
pub trait DiscoveryProvider: Send + Sync {
    /// شروع scan برای پیدا کردن دستگاه‌ها
    async fn start_scan(&self) -> Result<(), PlatformError>;
    
    /// توقف scan
    async fn stop_scan(&self) -> Result<(), PlatformError>;
    
    /// لیست دستگاه‌های پیدا شده
    async fn get_devices(&self) -> Result<Vec<DiscoveredDevice>, PlatformError>;
    
    /// اتصال به یه دستگاه + دریافت اطلاعات اتصال
    async fn connect_and_exchange(
        &self,
        device_id: &str,
        my_info: ConnectionInfo,
    ) -> Result<ConnectionInfo, PlatformError>;
    
    /// شروع Advertising (برای Host)
    async fn start_advertising(
        &self,
        my_info: ConnectionInfo,
    ) -> Result<(), PlatformError>;
    
    /// توقف Advertising
    async fn stop_advertising(&self) -> Result<(), PlatformError>;
    
    /// چک کن که Bluetooth روشنه
    async fn is_bluetooth_enabled(&self) -> Result<bool, PlatformError>;
    
    /// درخواست روشن کردن Bluetooth
    async fn request_enable_bluetooth(&self) -> Result<(), PlatformError>;
}

// ============================================
// Trait — HotspotProvider
// ============================================

/// Trait مشترک Hotspot
/// 
/// این trait مسئول مدیریت Wi-Fi Hotspot است.
/// برای انتقال فایل از Wi-Fi استفاده می‌شه.
#[async_trait]
pub trait HotspotProvider: Send + Sync {
    /// روشن کردن Hotspot
    async fn start(&self) -> Result<HotspotInfo, PlatformError>;
    
    /// خاموش کردن Hotspot
    async fn stop(&self) -> Result<(), PlatformError>;
    
    /// وضعیت فعلی Hotspot
    async fn status(&self) -> Result<HotspotStatus, PlatformError>;
    
    /// چک کن که Wi-Fi روشنه
    async fn is_wifi_enabled(&self) -> Result<bool, PlatformError>;
    
    /// درخواست روشن کردن Wi-Fi
    async fn request_enable_wifi(&self) -> Result<(), PlatformError>;
}

// ============================================
// Platform Error
// ============================================

/// خطای Platform
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PlatformError {
    /// Bluetooth خاموشه
    BluetoothDisabled,
    /// Wi-Fi خاموشه
    WifiDisabled,
    /// Permission نداریم
    PermissionDenied(String),
    /// دستگاه پیدا نشد
    DeviceNotFound(String),
    /// خطای اتصال
    ConnectionFailed(String),
    /// خطای داخلی
    Internal(String),
    /// پشتیبانی نمیشه
    NotSupported(String),
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BluetoothDisabled => write!(f, "Bluetooth خاموش است"),
            Self::WifiDisabled => write!(f, "Wi-Fi خاموش است"),
            Self::PermissionDenied(msg) => write!(f, "دسترسی رد شد: {}", msg),
            Self::DeviceNotFound(msg) => write!(f, "دستگاه پیدا نشد: {}", msg),
            Self::ConnectionFailed(msg) => write!(f, "خطا در اتصال: {}", msg),
            Self::Internal(msg) => write!(f, "خطای داخلی: {}", msg),
            Self::NotSupported(msg) => write!(f, "پشتیبانی نمی‌شود: {}", msg),
        }
    }
}

impl std::error::Error for PlatformError {}