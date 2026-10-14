//! رویدادهای Discovery
//! 
//! این رویدادها از طریق channel به UI فرستاده میشن.

use super::super::platform::types::DiscoveredDevice;
use serde::{Deserialize, Serialize};

/// رویداد Discovery
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiscoveryEvent {
    /// Scan شروع شد
    ScanStarted,
    
    /// Scan متوقف شد
    ScanStopped,
    
    /// دستگاه جدید پیدا شد
    DeviceFound {
        device: DiscoveredDevice,
    },
    
    /// دستگاه رفت (از دسترس خارج شد)
    DeviceLost {
        device_id: String,
    },
    
    /// لیست دستگاه‌ها آپدیت شد
    DevicesUpdated {
        devices: Vec<DiscoveredDevice>,
    },
    
    /// خطا
    Error {
        message: String,
    },
    
    /// وضعیت تغییر کرد
    StatusChanged {
        is_scanning: bool,
        device_count: usize,
    },
}