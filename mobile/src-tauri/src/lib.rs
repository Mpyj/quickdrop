//! QuickDrop Mobile - Entry Point

use quickdrop_core::{
    auth,
    config::ConfigManager,
    network,
    server,
    state::{AppState, ReceivedFile, SharedFile},
    transfer::TransferManager,
};

// ============================================
// Tauri Commands — App Info
// ============================================

#[tauri::command]
async fn get_app_info(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "lan_ip": state.lan_ip.to_string(),
        "pair_code": state.pair_code.to_string(),
    }))
}

// ============================================
// Tauri Commands — Platform
// ============================================

/// چک کردن وضعیت Bluetooth
/// 
/// روی Android، خود پلاگین blec این رو مدیریت می‌کنه.
/// این command فقط برای سازگاری با frontend هست.
#[tauri::command]
async fn get_bluetooth_state() -> Result<String, String> {
    println!("🔵 چک Bluetooth (از طریق blec)");
    // وضعیت واقعی از frontend با checkPermissions گرفته میشه
    Ok("unknown".to_string())
}

/// چک کردن وضعیت Wi-Fi
#[tauri::command]
async fn get_wifi_state() -> Result<String, String> {
    println!("📶 چک Wi-Fi");
    Ok("unknown".to_string())
}

/// چک کردن permission Location
#[tauri::command]
async fn check_location_permission() -> Result<String, String> {
    println!("📍 چک Location");
    Ok("granted".to_string())
}

/// درخواست روشن کردن Bluetooth
#[tauri::command]
async fn request_enable_bluetooth() -> Result<(), String> {
    println!("📡 درخواست روشن کردن Bluetooth");
    Ok(())
}

/// درخواست روشن کردن Wi-Fi
#[tauri::command]
async fn request_enable_wifi() -> Result<(), String> {
    println!("📶 درخواست روشن کردن Wi-Fi");
    Ok(())
}

/// باز کردن Settings Location
#[tauri::command]
async fn open_location_settings() -> Result<(), String> {
    println!("📍 باز کردن Settings Location");
    Ok(())
}

// ============================================
// Tauri Commands — Discovery
// ============================================

/// شروع scan دستگاه‌ها
/// 
/// روی Android، از پلاگین blec استفاده می‌کنیم که خودش
/// دستگاه‌های BLE اطراف رو پیدا می‌کنه.
#[tauri::command]
async fn start_device_scan() -> Result<Vec<serde_json::Value>, String> {
    println!("🔍 شروع scan...");

    // TODO: پیاده‌سازی واقعی با blec
    // فعلاً mock برای تست UI
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    let devices = vec![
        serde_json::json!({
            "id": "demo-1",
            "name": "Parsa-PC",
            "type": "desktop",
            "rssi": -42,
        }),
        serde_json::json!({
            "id": "demo-2",
            "name": "Samsung-A52",
            "type": "mobile",
            "rssi": -58,
        }),
    ];

    println!("✅ {} دستگاه پیدا شد", devices.len());
    Ok(devices)
}

/// اتصال به دستگاه
#[tauri::command]
async fn connect_to_device(device_id: String) -> Result<serde_json::Value, String> {
    println!("🔗 اتصال به: {}", device_id);
    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

    Ok(serde_json::json!({
        "success": true,
        "device_id": device_id,
    }))
}

// ============================================
// Main Entry
// ============================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("🌐 QuickDrop Mobile starting...");

    // ============================================
    // IP
    // ============================================
    let lan_ip = match network::get_lan_ip() {
        Ok(ip) => {
            println!("✅ LAN IP: {}", ip);
            ip.to_string()
        }
        Err(e) => {
            eprintln!("⚠️ خطا در IP: {}", e);
            "127.0.0.1".to_string()
        }
    };

    // ============================================
    // Config
    // ============================================
    let config = ConfigManager::new();
    let saved_path = tauri::async_runtime::block_on(config.get_save_path());
    let saved_received: Vec<ReceivedFile> =
        tauri::async_runtime::block_on(config.get_received_files());
    let saved_shared: Vec<SharedFile> =
        tauri::async_runtime::block_on(config.get_shared_files());

    println!("📥 {} فایل دریافتی", saved_received.len());
    println!("📤 {} فایل اشتراکی", saved_shared.len());

    // ============================================
    // TransferManager
    // ============================================
    let transfer = TransferManager::new(saved_path);

    // ============================================
    // Token + Pair Code
    // ============================================
    let token = auth::generate_token();
    let pair_code = auth::generate_pair_code();

    println!("🔐 Pair Code: {}", pair_code);
    println!("🔑 Token: {}...", &token[..8]);

    // ============================================
    // State
    // ============================================
    let state = AppState::new(
        token,
        pair_code,
        lan_ip.clone(),
        transfer,
        config,
        saved_received,
        saved_shared,
    );

    // ============================================
    // Server
    // ============================================
    let server_state = state.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = server::start_server(server_state).await {
            eprintln!("❌ خطا در سرور: {}", e);
        }
    });

    // ============================================
    // Tauri
    // ============================================
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_blec::init()) // 👈 پلاگین BLE
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            get_bluetooth_state,
            get_wifi_state,
            check_location_permission,
            request_enable_bluetooth,
            request_enable_wifi,
            open_location_settings,
            start_device_scan,
            connect_to_device,
        ])
        .setup(move |_app| {
            println!("🌐 QuickDrop Mobile آماده‌ست");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("خطا در اجرای Tauri Mobile");
}