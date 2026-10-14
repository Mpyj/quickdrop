//! Windows Implementation

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

pub struct WindowsDiscovery;

impl WindowsDiscovery {
    pub fn new() -> Self { Self }
}

impl Default for WindowsDiscovery {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl DiscoveryProvider for WindowsDiscovery {
    async fn start_scan(&self) -> Result<(), PlatformError> {
        log::info!("🔍 Windows: start BLE scan");
        Ok(())
    }
    
    async fn stop_scan(&self) -> Result<(), PlatformError> {
        log::info!("🛑 Windows: stop scan");
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
        Err(PlatformError::NotSupported("Windows خودکار روشن نمی‌کنه".to_string()))
    }
}

// ============================================
// Hotspot
// ============================================

pub struct WindowsHotspot;

impl WindowsHotspot {
    pub fn new() -> Self { Self }
}

impl Default for WindowsHotspot {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl HotspotProvider for WindowsHotspot {
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

pub struct WindowsPermissions;

impl WindowsPermissions {
    pub fn new() -> Self { Self }
}

impl Default for WindowsPermissions {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl PermissionProvider for WindowsPermissions {
    async fn check(&self, _perm: PermissionType) -> Result<PermissionStatus, PlatformError> {
        Ok(PermissionStatus::Granted)
    }
    
    async fn request(&self, _perm: PermissionType) -> Result<PermissionStatus, PlatformError> {
        Ok(PermissionStatus::Granted)
    }
    
    async fn open_app_settings(&self) -> Result<(), PlatformError> {
        Ok(())
    }
    
    async fn required_permissions(&self) -> Vec<PermissionType> {
        vec![]
    }
}

// ============================================
// System State
// ============================================

pub struct WindowsSystemState;

impl WindowsSystemState {
    pub fn new() -> Self { Self }
}

impl Default for WindowsSystemState {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl SystemStateProvider for WindowsSystemState {
    async fn bluetooth_state(&self) -> Result<BluetoothState, PlatformError> {
        Ok(BluetoothState::Off)
    }
    
    async fn wifi_state(&self) -> Result<WifiState, PlatformError> {
        Ok(WifiState::On)
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