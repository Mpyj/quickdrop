//! رویدادهای Connection

use serde::{Deserialize, Serialize};

/// وضعیت اتصال
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    /// قطع
    Disconnected,
    /// در حال اتصال
    Connecting,
    /// متصل
    Connected,
    /// خطا
    Failed(String),
}

/// رویداد Connection
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConnectionEvent {
    /// شروع اتصال
    ConnectingStarted {
        device_id: String,
        device_name: String,
    },
    
    /// اتصال برقرار شد
    Connected {
        device_id: String,
        device_name: String,
        ip: String,
        port: u16,
    },
    
    /// اتصال قطع شد
    Disconnected {
        device_id: String,
        reason: Option<String>,
    },
    
    /// خطا
    Error {
        device_id: String,
        message: String,
    },
}