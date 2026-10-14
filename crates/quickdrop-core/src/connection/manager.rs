//! Connection Manager

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

use super::{ConnectionError, events::*};
use crate::platform::{
    DiscoveryProvider, HotspotProvider,
    types::{ConnectionInfo, HotspotInfo},
};

/// یه اتصال فعال
#[derive(Clone, Debug)]
pub struct ActiveConnection {
    /// شناسه‌ی دستگاه
    pub device_id: String,
    /// اسم دستگاه
    pub device_name: String,
    /// IP
    pub ip: String,
    /// پورت
    pub port: u16,
    /// توکن
    pub token: String,
    /// وضعیت
    pub state: ConnectionState,
}

/// مدیر Connection
pub struct ConnectionManager {
    /// Discovery provider
    discovery: Arc<dyn DiscoveryProvider>,
    
    /// Hotspot provider
    hotspot: Arc<dyn HotspotProvider>,
    
    /// اتصال‌های فعال
    connections: Arc<RwLock<HashMap<String, ActiveConnection>>>,
    
    /// Hotspot فعلی
    current_hotspot: Arc<RwLock<Option<HotspotInfo>>>,
    
    /// Channel برای رویدادها
    event_tx: broadcast::Sender<ConnectionEvent>,
}

impl ConnectionManager {
    pub fn new(
        discovery: Arc<dyn DiscoveryProvider>,
        hotspot: Arc<dyn HotspotProvider>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            discovery,
            hotspot,
            connections: Arc::new(RwLock::new(HashMap::new())),
            current_hotspot: Arc::new(RwLock::new(None)),
            event_tx,
        }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<ConnectionEvent> {
        self.event_tx.subscribe()
    }
    
    /// شروع اتصال به یه دستگاه
    /// 
    /// این تابع:
    /// 1. اطلاعات خودمون رو از طریق BLE می‌فرسته
    /// 2. اطلاعات دستگاه مقابل رو می‌گیره
    /// 3. اتصال رو ثبت می‌کنه
    pub async fn connect(
        &self,
        device_id: &str,
        my_info: ConnectionInfo,
    ) -> Result<ActiveConnection, ConnectionError> {
        let _ = self.event_tx.send(ConnectionEvent::ConnectingStarted {
            device_id: device_id.to_string(),
            device_name: my_info.device_name.clone(),
        });
        
        // تبادل اطلاعات از طریق BLE
        let their_info = self.discovery
            .connect_and_exchange(device_id, my_info.clone())
            .await
            .map_err(|e| {
                let msg = e.to_string();
                let _ = self.event_tx.send(ConnectionEvent::Error {
                    device_id: device_id.to_string(),
                    message: msg.clone(),
                });
                ConnectionError::ConnectionFailed(msg)
            })?;
        
        // ساخت ActiveConnection
        let conn = ActiveConnection {
            device_id: device_id.to_string(),
            device_name: their_info.device_name.clone(),
            ip: their_info.ip.clone(),
            port: their_info.port,
            token: their_info.token.clone(),
            state: ConnectionState::Connected,
        };
        
        // ثبت
        self.connections.write().await
            .insert(device_id.to_string(), conn.clone());
        
        let _ = self.event_tx.send(ConnectionEvent::Connected {
            device_id: device_id.to_string(),
            device_name: their_info.device_name,
            ip: their_info.ip,
            port: their_info.port,
        });
        
        Ok(conn)
    }
    
    /// قطع اتصال
    pub async fn disconnect(&self, device_id: &str) -> Result<(), ConnectionError> {
        let removed = self.connections.write().await
            .remove(device_id);
        
        if removed.is_some() {
            let _ = self.event_tx.send(ConnectionEvent::Disconnected {
                device_id: device_id.to_string(),
                reason: None,
            });
        }
        
        Ok(())
    }
    
    /// لیست اتصال‌های فعال
    pub async fn get_connections(&self) -> Vec<ActiveConnection> {
        self.connections.read().await
            .values()
            .cloned()
            .collect()
    }
    
    /// روشن کردن Hotspot
    /// 
    /// برای انتقال فایل، یکی از دستگاه‌ها Hotspot روشن می‌کنه.
    pub async fn start_hotspot(&self) -> Result<HotspotInfo, ConnectionError> {
        let info = self.hotspot.start().await
            .map_err(|e| ConnectionError::HotspotFailed(e.to_string()))?;
        
        *self.current_hotspot.write().await = Some(info.clone());
        
        Ok(info)
    }
    
    /// خاموش کردن Hotspot
    pub async fn stop_hotspot(&self) -> Result<(), ConnectionError> {
        self.hotspot.stop().await
            .map_err(|e| ConnectionError::HotspotFailed(e.to_string()))?;
        
        *self.current_hotspot.write().await = None;
        
        Ok(())
    }
    
    /// وضعیت Hotspot
    pub async fn hotspot_info(&self) -> Option<HotspotInfo> {
        self.current_hotspot.read().await.clone()
    }
}