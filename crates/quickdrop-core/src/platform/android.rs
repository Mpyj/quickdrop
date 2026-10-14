//! Android Implementation
//! 
//! از طریق Tauri Plugin با Kotlin ارتباط برقرار می‌کنه.

use async_trait::async_trait;
use super::{
    DiscoveryProvider, HotspotProvider, PlatformError,
    PermissionProvider, PermissionStatus, PermissionType,
    SystemStateProvider, BluetoothState, WifiState,
    types::*,
};

// ============================================
// Discovery
// ============================================

pub struct AndroidDiscovery;

impl AndroidDiscovery {
    pub fn new() -> Self { Self }
}

impl Default for AndroidDiscovery {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl DiscoveryProvider for AndroidDiscovery {
    async fn start_scan(&self) -> Result<(), PlatformError> {
        log::info!("🔍 Android: start BLE scan");
        Ok(())
    }
    
    async fn stop_scan(&self) -> Result<(), PlatformError> {
        Ok(())
    }
    
    async fn get_devices(&self) -> Result<Vec<DiscoveredDevice>, PlatformError> {
        Ok(vec![])
    }
    
    async fn connect_and_exchange(
        &self,
        _device_id: &str,
        _my_info: ConnectionInfo,
    ) -> Result<ConnectionInfo, PlatformError> {
        Err(PlatformError::NotSupported("به‌زودی".to_string()))
    }
    
    async fn start_advertising(&self, _my_info: ConnectionInfo) -> Result<(), PlatformError> {
        Ok(())
    }
    
    async fn stop_advertising(&self) -> Result<(), PlatformError> {
        Ok(())
    }
    
    async fn is_bluetooth_enabled(&self) -> Result<bool, PlatformError> {
        Ok(false)
    }
    
    async fn request_enable_bluetooth(&self) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported("به‌زودی".to_string()))
    }
}

// ============================================
// Hotspot
// ============================================

pub struct AndroidHotspot;

impl AndroidHotspot {
    pub fn new() -> Self { Self }
}

impl Default for AndroidHotspot {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl HotspotProvider for AndroidHotspot {
    async fn start(&self) -> Result<HotspotInfo, PlatformError> {
        Err(PlatformError::NotSupported("به‌زودی".to_string()))
    }
    
    async fn stop(&self) -> Result<(), PlatformError> {
        Ok(())
    }
    
    async fn status(&self) -> Result<HotspotStatus, PlatformError> {
        Ok(HotspotStatus {
            is_active: false,
            ssid: None,
            connected_devices: 0,
        })
    }
    
    async fn is_wifi_enabled(&self) -> Result<bool, PlatformError> {
        Ok(false)
    }
    
    async fn request_enable_wifi(&self) -> Result<(), PlatformError> {
        Ok(())
    }
}

// ============================================
// Permissions
// ============================================

pub struct AndroidPermissions;

impl AndroidPermissions {
    pub fn new() -> Self { Self }
}

impl Default for AndroidPermissions {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl PermissionProvider for AndroidPermissions {
    async fn check(&self, _perm: PermissionType) -> Result<PermissionStatus, PlatformError> {
        Ok(PermissionStatus::NotRequested)
    }
    
    async fn request(&self, _perm: PermissionType) -> Result<PermissionStatus, PlatformError> {
        Err(PlatformError::NotSupported("به‌زودی".to_string()))
    }
    
    async fn open_app_settings(&self) -> Result<(), PlatformError> {
        Ok(())
    }
    
    async fn required_permissions(&self) -> Vec<PermissionType> {
        vec![
            PermissionType::BluetoothScan,
            PermissionType::BluetoothConnect,
            PermissionType::BluetoothAdvertise,
            PermissionType::NearbyWifiDevices,
            PermissionType::Location,
        ]
    }
}

// ============================================
// System State
// ============================================

pub struct AndroidSystemState;

impl AndroidSystemState {
    pub fn new() -> Self { Self }
}

impl Default for AndroidSystemState {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl SystemStateProvider for AndroidSystemState {
    async fn bluetooth_state(&self) -> Result<BluetoothState, PlatformError> {
        Ok(BluetoothState::Off)
    }
    
    async fn wifi_state(&self) -> Result<WifiState, PlatformError> {
        Ok(WifiState::Off)
    }
    
    async fn request_enable_bluetooth(&self) -> Result<(), PlatformError> {
        Ok(())
    }
    
    async fn request_enable_wifi(&self) -> Result<(), PlatformError> {
        Ok(())
    }
    
    async fn open_system_settings(&self) -> Result<(), PlatformError> {
        Ok(())
    }
}