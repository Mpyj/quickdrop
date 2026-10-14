//! QuickDrop Core
//! 
//! این crate شامل همه‌ی منطق مشترک بین Desktop و Mobile هست.
//! 
//! ساختار ماژول‌ها:
//! - `auth` — توکن و Pair Code
//! - `config` — persistence (ذخیره/لود تنظیمات)
//! - `network` — IP discovery
//! - `qr` — تولید QR Code
//! - `server` — وب‌سرور (Axum)
//! - `state` — State مشترک
//! - `transfer` — انتقال فایل (chunked + streaming)
//! - `platform` — abstraction layer (Bluetooth, Hotspot, Permissions)
//! - `discovery` — پیدا کردن دستگاه‌ها
//! - `connection` — مدیریت اتصال

// ============================================
// Modules
// ============================================

pub mod auth;
pub mod config;
pub mod connection;
pub mod discovery;
pub mod network;
pub mod platform;
pub mod qr;
pub mod server;
pub mod state;
pub mod transfer;

// ============================================
// Re-exports — Core
// ============================================

pub use auth::{generate_token, generate_pair_code, verify_token};
pub use config::ConfigManager;
pub use network::get_lan_ip;
pub use qr::generate_qr_svg;
pub use server::{start_server, DEFAULT_PORT};
pub use state::{AppState, ReceivedFile, SharedFile};
pub use transfer::{TransferManager, UploadSession, CHUNK_SIZE};

// ============================================
// Re-exports — Platform
// ============================================

pub use platform::{
    DiscoveryProvider,
    HotspotProvider,
    PlatformError,
    PermissionProvider,
    PermissionStatus,
    PermissionType,
    SystemStateProvider,
    BluetoothState,
    WifiState,
    types::{
        BleMessage,
        ConnectionInfo,
        DeviceType,
        DiscoveredDevice,
        HotspotInfo,
        HotspotStatus,
    },
};

// ============================================
// Re-exports — Discovery
// ============================================

pub use discovery::{
    DiscoveryError,
    DiscoveryEvent,
    DiscoveryManager,
};

// ============================================
// Re-exports — Connection
// ============================================

pub use connection::{
    ActiveConnection,
    ConnectionError,
    ConnectionEvent,
    ConnectionManager,
    ConnectionState,
};

// ============================================
// Version Info
// ============================================

/// نسخه‌ی QuickDrop Core
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// نام پروتکل
pub const PROTOCOL_NAME: &str = "QuickDrop";

/// نسخه‌ی پروتکل
pub const PROTOCOL_VERSION: u32 = 1;