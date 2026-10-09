//! QuickDrop Desktop - Entry Point

use quickdrop_core::{
    auth,
    config::ConfigManager,
    network,
    server,
    state::{AppState, ReceivedFile, SharedFile},
    transfer::TransferManager,
};

// ============================================
// Tauri Commands (فقط Desktop)
// ============================================

/// باز کردن فایل با اپلیکیشن پیش‌فرض سیستم
#[tauri::command]
async fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("خطا: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("خطا: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("خطا: {}", e))?;
    }

    Ok(())
}

/// نمایش فایل در پوشه (select شده)
#[tauri::command]
async fn reveal_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| format!("خطا: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| format!("خطا: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("خطا: {}", e))?;
        }
    }

    Ok(())
}

// ============================================
// Main Entry
// ============================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("🌐 Using system WebView2 (default)");

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

    if saved_path.is_some() {
        println!(
            "📂 مسیر ذخیره‌ی سفارشی لود شد: {}",
            saved_path.as_ref().unwrap()
        );
    }
    println!("📥 {} فایل دریافتی از قبل", saved_received.len());
    println!("📤 {} فایل اشتراکی از قبل", saved_shared.len());

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

    println!("📱 Mobile URL: {}", state.mobile_url());

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
        .manage(state)
        .invoke_handler(tauri::generate_handler![open_file, reveal_file])
        .setup(move |_app| {
            println!("🌐 QuickDrop Desktop آماده‌ست");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("خطا در اجرای Tauri");
}