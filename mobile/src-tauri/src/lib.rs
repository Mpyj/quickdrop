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
// Tauri Commands (Mobile-specific)
// ============================================

/// گرفتن IP محلی
#[tauri::command]
async fn get_local_ip() -> Result<String, String> {
    match network::get_lan_ip() {
        Ok(ip) => Ok(ip.to_string()),
        Err(_) => Ok("127.0.0.1".to_string()),
    }
}

/// گرفتن Pair Code
#[tauri::command]
async fn get_pair_code(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.pair_code.to_string())
}

/// گرفتن توکن
#[tauri::command]
async fn get_token(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.token.to_string())
}

/// گرفتن Mobile URL
#[tauri::command]
async fn get_mobile_url(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.mobile_url())
}

// ============================================
// Main Entry
// ============================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("🌐 QuickDrop Mobile starting...");

    // IP
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

    // Config
    let config = ConfigManager::new();

    let saved_path = tauri::async_runtime::block_on(config.get_save_path());
    let saved_received: Vec<ReceivedFile> =
        tauri::async_runtime::block_on(config.get_received_files());
    let saved_shared: Vec<SharedFile> =
        tauri::async_runtime::block_on(config.get_shared_files());

    println!("📥 {} فایل دریافتی", saved_received.len());
    println!("📤 {} فایل اشتراکی", saved_shared.len());

    // TransferManager
    let transfer = TransferManager::new(saved_path);

    // Token + Pair Code
    let token = auth::generate_token();
    let pair_code = auth::generate_pair_code();

    println!("🔐 Pair Code: {}", pair_code);
    println!("🔑 Token: {}...", &token[..8]);

    // State
    let state = AppState::new(
        token,
        pair_code,
        lan_ip.clone(),
        transfer,
        config,
        saved_received,
        saved_shared,
    );

    println!("📱 Mobile URL: {}", state.mobile_url());

    // Server
    let server_state = state.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = server::start_server(server_state).await {
            eprintln!("❌ خطا در سرور: {}", e);
        }
    });

    // Tauri
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_local_ip,
            get_pair_code,
            get_token,
            get_mobile_url,
        ])
        .setup(move |_app| {
            println!("🌐 QuickDrop Mobile آماده‌ست");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("خطا در اجرای Tauri Mobile");
}