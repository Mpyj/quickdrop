//! پیدا کردن IP مناسب LAN برای اتصال دستگاه‌های دیگه

use std::net::IpAddr;

/// IP مناسب شبکه‌ی محلی رو برمی‌گردونه.
/// 
/// این تابع همه‌ی interfaceهای شبکه رو بررسی می‌کنه و 
/// اولین IP معتبر (نه loopback، نه IPv6 link-local) رو برمی‌گردونه.
/// 
/// # مثال
/// ```
/// let ip = get_lan_ip();
/// println!("آدرس سرور: http://{}:8080", ip);
/// ```
pub fn get_lan_ip() -> Result<IpAddr, String> {
    match local_ip_address::local_ip() {
        Ok(ip) => {
            // چک می‌کنیم که loopback نباشه
            if ip.is_loopback() {
                return Err("فقط loopback پیدا شد - احتمالاً شبکه وصل نیست".to_string());
            }
            Ok(ip)
        }
        Err(e) => Err(format!("نتونستم IP پیدا کنم: {}", e)),
    }
}