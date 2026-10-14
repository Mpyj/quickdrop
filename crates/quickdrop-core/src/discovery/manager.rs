//! Discovery Manager
//! 
//! این struct مسئول:
//! - مدیریت scan
//! - نگه‌داشتن لیست دستگاه‌ها
//! - فرستادن رویدادها به UI

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

use super::DiscoveryError;
use super::events::DiscoveryEvent;
use crate::platform::{
    DiscoveryProvider,
    types::{ConnectionInfo, DiscoveredDevice},
};

/// مدیر Discovery
pub struct DiscoveryManager {
    /// Provider (Windows/Android/...)
    provider: Arc<dyn DiscoveryProvider>,
    
    /// لیست دستگاه‌ها
    devices: Arc<RwLock<HashMap<String, DiscoveredDevice>>>,
    
    /// وضعیت scan
    is_scanning: Arc<RwLock<bool>>,
    
    /// اطلاعات خودمون
    my_info: Arc<RwLock<Option<ConnectionInfo>>>,
    
    /// Channel برای رویدادها
    event_tx: broadcast::Sender<DiscoveryEvent>,
}

impl DiscoveryManager {
    /// ساخت DiscoveryManager جدید
    pub fn new(provider: Arc<dyn DiscoveryProvider>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            provider,
            devices: Arc::new(RwLock::new(HashMap::new())),
            is_scanning: Arc::new(RwLock::new(false)),
            my_info: Arc::new(RwLock::new(None)),
            event_tx,
        }
    }
    
    /// گرفتن receiver برای رویدادها
    pub fn subscribe(&self) -> broadcast::Receiver<DiscoveryEvent> {
        self.event_tx.subscribe()
    }
    
    /// شروع scan
    pub async fn start_scan(&self) -> Result<(), DiscoveryError> {
        if *self.is_scanning.read().await {
            return Ok(()); // از قبل شروع شده
        }
        
        // چک کن Bluetooth روشنه
        if !self.provider.is_bluetooth_enabled().await
            .map_err(|e| DiscoveryError::Internal(e.to_string()))? 
        {
            return Err(DiscoveryError::BluetoothDisabled);
        }
        
        // شروع scan
        self.provider.start_scan().await
            .map_err(|e| DiscoveryError::ScanFailed(e.to_string()))?;
        
        *self.is_scanning.write().await = true;
        
        let _ = self.event_tx.send(DiscoveryEvent::ScanStarted);
        
        Ok(())
    }
    
    /// توقف scan
    pub async fn stop_scan(&self) -> Result<(), DiscoveryError> {
        if !*self.is_scanning.read().await {
            return Ok(());
        }
        
        self.provider.stop_scan().await
            .map_err(|e| DiscoveryError::ScanFailed(e.to_string()))?;
        
        *self.is_scanning.write().await = false;
        
        let _ = self.event_tx.send(DiscoveryEvent::ScanStopped);
        
        Ok(())
    }
    
    /// آپدیت لیست دستگاه‌ها از provider
    pub async fn refresh_devices(&self) -> Result<(), DiscoveryError> {
        let devices = self.provider.get_devices().await
            .map_err(|e| DiscoveryError::ScanFailed(e.to_string()))?;
        
        let mut map = self.devices.write().await;
        
        // آپدیت
        for device in devices {
            map.insert(device.id.clone(), device);
        }
        
        // فرستادن رویداد
        let list: Vec<DiscoveredDevice> = map.values().cloned().collect();
        let _ = self.event_tx.send(DiscoveryEvent::DevicesUpdated {
            devices: list,
        });
        
        let _ = self.event_tx.send(DiscoveryEvent::StatusChanged {
            is_scanning: *self.is_scanning.read().await,
            device_count: map.len(),
        });
        
        Ok(())
    }
    
    /// لیست دستگاه‌ها
    pub async fn get_devices(&self) -> Vec<DiscoveredDevice> {
        let map = self.devices.read().await;
        map.values().cloned().collect()
    }
    
    /// اتصال به یه دستگاه و تبادل اطلاعات
    pub async fn connect_and_exchange(
        &self,
        device_id: &str,
        my_info: ConnectionInfo,
    ) -> Result<ConnectionInfo, DiscoveryError> {
        let their_info = self.provider
            .connect_and_exchange(device_id, my_info)
            .await
            .map_err(|e| DiscoveryError::ConnectionFailed(e.to_string()))?;
        
        Ok(their_info)
    }
    
    /// شروع Advertising (برای Host)
    pub async fn start_advertising(
        &self,
        my_info: ConnectionInfo,
    ) -> Result<(), DiscoveryError> {
        self.provider.start_advertising(my_info.clone()).await
            .map_err(|e| DiscoveryError::Internal(e.to_string()))?;
        
        *self.my_info.write().await = Some(my_info);
        
        Ok(())
    }
    
    /// توقف Advertising
    pub async fn stop_advertising(&self) -> Result<(), DiscoveryError> {
        self.provider.stop_advertising().await
            .map_err(|e| DiscoveryError::Internal(e.to_string()))?;
        
        *self.my_info.write().await = None;
        
        Ok(())
    }
    
    /// وضعیت scan
    pub async fn is_scanning(&self) -> bool {
        *self.is_scanning.read().await
    }
    
    /// پاک کردن لیست دستگاه‌ها
    pub async fn clear_devices(&self) {
        self.devices.write().await.clear();
    }
}