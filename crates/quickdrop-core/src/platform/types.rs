//! انواع داده‌های Platform

use serde::{Deserialize, Serialize};

/// نوع دستگاه
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Desktop,
    Mobile,
    Unknown,
}

impl Default for DeviceType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "desktop"),
            Self::Mobile => write!(f, "mobile"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// یه دستگاه QuickDrop که پیدا شده
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub rssi: i32,
    pub is_paired: bool,
    pub last_seen: u64,
}

/// اطلاعات اتصال — از طریق BLE مبادله میشه
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub ip: String,
    pub port: u16,
    pub token: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub hotspot_ssid: Option<String>,
    pub hotspot_password: Option<String>,
    pub protocol_version: u32,
    pub timestamp: u64,
}

impl ConnectionInfo {
    pub const PROTOCOL_VERSION: u32 = 1;
    
    pub fn new(
        ip: String,
        port: u16,
        token: String,
        device_name: String,
        device_type: DeviceType,
    ) -> Self {
        Self {
            ip,
            port,
            token,
            device_name,
            device_type,
            hotspot_ssid: None,
            hotspot_password: None,
            protocol_version: Self::PROTOCOL_VERSION,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
    
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("خطا در serialize: {}", e))
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| format!("خطا در deserialize: {}", e))
    }
}

/// اطلاعات Hotspot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HotspotInfo {
    pub ssid: String,
    pub password: String,
    pub ip: String,
    pub is_active: bool,
}

/// وضعیت Hotspot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HotspotStatus {
    pub is_active: bool,
    pub ssid: Option<String>,
    pub connected_devices: u32,
}

/// پیام‌های BLE Protocol
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BleMessage {
    Hello {
        device_name: String,
        device_type: DeviceType,
        protocol_version: u32,
    },
    RequestConnection,
    ConnectionInfo {
        info: ConnectionInfo,
    },
    Ack,
    Error {
        message: String,
    },
    Goodbye,
}

impl BleMessage {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("خطا در serialize: {}", e))
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| format!("خطا در deserialize: {}", e))
    }
}