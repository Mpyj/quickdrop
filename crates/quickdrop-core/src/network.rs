//! پیدا کردن IP مناسب LAN — بدون نیاز به اینترنت
//! 
//! این ماژول از `if-addrs` استفاده می‌کنه که interfaceهای شبکه رو
//! مستقیماً از سیستم می‌خونه — بدون نیاز به اینترنت یا اتصال خارجی.

use std::net::{IpAddr, Ipv4Addr};

/// IP مناسب شبکه‌ی محلی رو برمی‌گردونه.
/// 
/// **بدون نیاز به اینترنت** — فقط از interfaceهای شبکه استفاده می‌کنه.
pub fn get_lan_ip() -> Result<IpAddr, String> {
    // ============================================
    // روش 1: if-addrs (بدون اینترنت) — اولویت اول
    // ============================================
    match find_lan_ip_via_if_addrs() {
        Ok(ip) => {
            println!("✅ IP پیدا شد: {}", ip);
            return Ok(ip);
        }
        Err(e) => {
            eprintln!("⚠️ if-addrs نشد: {}", e);
        }
    }

    // ============================================
    // روش 2: local-ip-address (fallback)
    // ============================================
    match local_ip_address::local_ip() {
        Ok(ip) => {
            if ip.is_loopback() {
                return Err("فقط loopback پیدا شد".to_string());
            }
            println!("✅ IP (fallback): {}", ip);
            Ok(ip)
        }
        Err(e) => Err(format!("نتونستم IP پیدا کنم: {}", e)),
    }
}

/// پیدا کردن IP از interfaceهای شبکه (بدون اینترنت)
fn find_lan_ip_via_if_addrs() -> Result<IpAddr, String> {
    let interfaces = if_addrs::get_if_addrs()
        .map_err(|e| format!("خطا در گرفتن interfaceها: {}", e))?;

    let mut candidates: Vec<(String, Ipv4Addr)> = Vec::new();

    println!("🔍 بررسی interfaceهای شبکه:");

    for iface in interfaces {
        if let if_addrs::IfAddr::V4(addr) = iface.addr {
            let ip = addr.ip;

            // فیلتر 1: loopback (127.0.0.1)
            if ip.is_loopback() {
                println!("  ❌ {} → {} (loopback)", iface.name, ip);
                continue;
            }

            // فیلتر 2: link-local (169.254.x.x)
            if ip.is_link_local() {
                println!("  ❌ {} → {} (link-local)", iface.name, ip);
                continue;
            }

            println!("  ✅ {} → {}", iface.name, ip);
            candidates.push((iface.name.clone(), ip));
        }
    }

    if candidates.is_empty() {
        return Err("هیچ IP معتبری پیدا نشد".to_string());
    }

    // ============================================
    // اولویت‌بندی: Wi-Fi > Hotspot > Ethernet
    // ============================================
    
    // اولویت ۱: 192.168.x.x (Wi-Fi خونه یا Hotspot اندروید)
    for (name, ip) in &candidates {
        let octets = ip.octets();
        if octets[0] == 192 && octets[1] == 168 {
            println!("🎯 انتخاب (192.168.x.x): {} ({})", ip, name);
            return Ok(IpAddr::V4(*ip));
        }
    }

    // اولویت ۲: 10.x.x.x (Hotspot یا شبکه‌ی اداری)
    for (name, ip) in &candidates {
        let octets = ip.octets();
        if octets[0] == 10 {
            println!("🎯 انتخاب (10.x.x.x): {} ({})", ip, name);
            return Ok(IpAddr::V4(*ip));
        }
    }

    // اولویت ۳: 172.16-31.x.x
    for (name, ip) in &candidates {
        let octets = ip.octets();
        if octets[0] == 172 && (16..=31).contains(&octets[1]) {
            println!("🎯 انتخاب (172.16-31.x.x): {} ({})", ip, name);
            return Ok(IpAddr::V4(*ip));
        }
    }

    // فالبک: اولین IP
    let (name, ip) = &candidates[0];
    println!("🎯 انتخاب (فالبک): {} ({})", ip, name);
    Ok(IpAddr::V4(*ip))
}

/// چک می‌کنه که آدرس private هست یا نه
#[allow(dead_code)]
pub fn is_private_ip(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();

    // 10.0.0.0/8
    if octets[0] == 10 {
        return true;
    }

    // 172.16.0.0/12
    if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        return true;
    }

    // 192.168.0.0/16
    if octets[0] == 192 && octets[1] == 168 {
        return true;
    }

    false
}