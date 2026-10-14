//! Permission Management
//! 
//! مدیریت دسترسی‌ها برای Android 12+

use async_trait::async_trait;
use super::PlatformError;
use serde::{Deserialize, Serialize};

/// نوع Permission
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PermissionType {
    Location,
    Bluetooth,
    BluetoothScan,
    BluetoothConnect,
    BluetoothAdvertise,
    NearbyWifiDevices,
    WifiState,
    ChangeWifiState,
}

impl PermissionType {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Location => "برای پیدا کردن دستگاه‌های نزدیک",
            Self::Bluetooth => "برای استفاده از Bluetooth",
            Self::BluetoothScan => "برای پیدا کردن دستگاه‌های Bluetooth",
            Self::BluetoothConnect => "برای اتصال به دستگاه‌های Bluetooth",
            Self::BluetoothAdvertise => "برای قابل دیدن شدن توسط دیگران",
            Self::NearbyWifiDevices => "برای پیدا کردن دستگاه‌های Wi-Fi",
            Self::WifiState => "برای چک کردن وضعیت Wi-Fi",
            Self::ChangeWifiState => "برای روشن/خاموش کردن Wi-Fi",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Location => "موقعیت",
            Self::Bluetooth => "Bluetooth",
            Self::BluetoothScan => "Scan Bluetooth",
            Self::BluetoothConnect => "اتصال Bluetooth",
            Self::BluetoothAdvertise => "Advertising Bluetooth",
            Self::NearbyWifiDevices => "دستگاه‌های Wi-Fi نزدیک",
            Self::WifiState => "وضعیت Wi-Fi",
            Self::ChangeWifiState => "تغییر Wi-Fi",
        }
    }
}

/// وضعیت Permission
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    Granted,
    Denied,
    DeniedCanAsk,
    PermanentlyDenied,
    NotRequested,
}

impl PermissionStatus {
    pub fn is_granted(&self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Trait Permission Provider
#[async_trait]
pub trait PermissionProvider: Send + Sync {
    async fn check(&self, perm: PermissionType) -> Result<PermissionStatus, PlatformError>;
    async fn request(&self, perm: PermissionType) -> Result<PermissionStatus, PlatformError>;
    
    async fn request_multiple(
        &self,
        perms: Vec<PermissionType>,
    ) -> Result<Vec<(PermissionType, PermissionStatus)>, PlatformError> {
        let mut results = Vec::new();
        for perm in perms {
            let status = self.request(perm.clone()).await?;
            results.push((perm, status));
        }
        Ok(results)
    }
    
    async fn open_app_settings(&self) -> Result<(), PlatformError>;
    async fn required_permissions(&self) -> Vec<PermissionType>;
}

/// وضعیت Bluetooth
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BluetoothState {
    On,
    Off,
    TurningOn,
    TurningOff,
    Unsupported,
    PermissionDenied,
}

/// وضعیت Wi-Fi
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WifiState {
    On,
    Off,
    TurningOn,
    TurningOff,
    Unsupported,
    PermissionDenied,
}

/// Trait System State Provider
#[async_trait]
pub trait SystemStateProvider: Send + Sync {
    async fn bluetooth_state(&self) -> Result<BluetoothState, PlatformError>;
    async fn wifi_state(&self) -> Result<WifiState, PlatformError>;
    async fn request_enable_bluetooth(&self) -> Result<(), PlatformError>;
    async fn request_enable_wifi(&self) -> Result<(), PlatformError>;
    async fn open_system_settings(&self) -> Result<(), PlatformError>;
}