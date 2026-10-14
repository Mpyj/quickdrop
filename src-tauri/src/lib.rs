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
// Tauri Commands — File Operations
// ============================================

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
// Platform State — Windows
// ============================================

#[cfg(target_os = "windows")]
mod windows_platform {
    use windows::Devices::Radios::{Radio, RadioKind, RadioState};

    pub fn get_bluetooth_state() -> String {
        match check_radio_state(RadioKind::Bluetooth) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("⚠️ خطا در چک Bluetooth: {}", e);
                "unsupported".to_string()
            }
        }
    }

    pub fn get_wifi_state() -> String {
        match check_radio_state(RadioKind::WiFi) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("⚠️ خطا در چک Wi-Fi: {}", e);
                "unsupported".to_string()
            }
        }
    }

    fn check_radio_state(kind: RadioKind) -> Result<String, String> {
        let radios = Radio::GetRadiosAsync()
            .map_err(|e| format!("خطا: {}", e))?
            .get()
            .map_err(|e| format!("خطا: {}", e))?;

        for radio in radios {
            let radio_kind = radio.Kind().map_err(|e| format!("خطا: {}", e))?;
            if radio_kind == kind {
                let state = radio.State().map_err(|e| format!("خطا: {}", e))?;
                return Ok(match state {
                    RadioState::On => "on".to_string(),
                    RadioState::Off => "off".to_string(),
                    RadioState::Disabled => "off".to_string(),
                    _ => "unknown".to_string(),
                });
            }
        }
        Err(format!("Radio {:?} پیدا نشد", kind))
    }

    pub fn open_bluetooth_settings() -> Result<(), String> {
        std::process::Command::new("cmd")
            .args(["/C", "start", "ms-settings:bluetooth"])
            .spawn()
            .map_err(|e| format!("خطا: {}", e))?;
        Ok(())
    }

    pub fn open_wifi_settings() -> Result<(), String> {
        std::process::Command::new("cmd")
            .args(["/C", "start", "ms-settings:network-wifi"])
            .spawn()
            .map_err(|e| format!("خطا: {}", e))?;
        Ok(())
    }

    pub fn open_location_settings() -> Result<(), String> {
        std::process::Command::new("cmd")
            .args(["/C", "start", "ms-settings:privacy-location"])
            .spawn()
            .map_err(|e| format!("خطا: {}", e))?;
        Ok(())
    }
}

// ============================================
// Tauri Commands — Platform
// ============================================

#[tauri::command]
async fn get_bluetooth_state() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let state = tokio::task::spawn_blocking(|| {
            windows_platform::get_bluetooth_state()
        })
        .await
        .map_err(|e| format!("خطا: {}", e))?;
        println!("🔵 Bluetooth: {}", state);
        return Ok(state);
    }

    #[cfg(target_os = "android")]
    {
        return Ok("off".to_string());
    }

    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        Ok("unsupported".to_string())
    }
}

#[tauri::command]
async fn get_wifi_state() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let state = tokio::task::spawn_blocking(|| {
            windows_platform::get_wifi_state()
        })
        .await
        .map_err(|e| format!("خطا: {}", e))?;
        println!("📶 Wi-Fi: {}", state);
        return Ok(state);
    }

    #[cfg(target_os = "android")]
    {
        return Ok("off".to_string());
    }

    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        Ok("unsupported".to_string())
    }
}

#[tauri::command]
async fn check_location_permission() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        return Ok("granted".to_string());
    }

    #[cfg(target_os = "android")]
    {
        return Ok("not_requested".to_string());
    }

    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        Ok("granted".to_string())
    }
}

#[tauri::command]
async fn request_enable_bluetooth() -> Result<(), String> {
    println!("📡 باز کردن Settings Bluetooth");

    #[cfg(target_os = "windows")]
    {
        tokio::task::spawn_blocking(|| {
            windows_platform::open_bluetooth_settings()
        })
        .await
        .map_err(|e| format!("خطا: {}", e))??;
    }

    Ok(())
}

#[tauri::command]
async fn request_enable_wifi() -> Result<(), String> {
    println!("📶 باز کردن Settings Wi-Fi");

    #[cfg(target_os = "windows")]
    {
        tokio::task::spawn_blocking(|| {
            windows_platform::open_wifi_settings()
        })
        .await
        .map_err(|e| format!("خطا: {}", e))??;
    }

    Ok(())
}

#[tauri::command]
async fn open_location_settings() -> Result<(), String> {
    println!("📍 باز کردن Settings Location");

    #[cfg(target_os = "windows")]
    {
        tokio::task::spawn_blocking(|| {
            windows_platform::open_location_settings()
        })
        .await
        .map_err(|e| format!("خطا: {}", e))??;
    }

    Ok(())
}

#[tauri::command]
async fn open_system_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "ms-settings:"])
            .spawn()
            .map_err(|e| format!("خطا: {}", e))?;
    }
    Ok(())
}

// ============================================
// Tauri Commands — Discovery
// ============================================

#[tauri::command]
async fn start_device_scan() -> Result<Vec<serde_json::Value>, String> {
    println!("🔍 شروع scan دستگاه‌ها...");
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    let devices = vec![
        serde_json::json!({
            "id": "demo-1",
            "name": "Parsa-Laptop",
            "type": "desktop",
            "rssi": -45,
        }),
        serde_json::json!({
            "id": "demo-2",
            "name": "Samsung-A52",
            "type": "mobile",
            "rssi": -62,
        }),
    ];

    println!("✅ {} دستگاه پیدا شد", devices.len());
    Ok(devices)
}

#[tauri::command]
async fn stop_device_scan() -> Result<(), String> {
    println!("🛑 توقف scan");
    Ok(())
}

#[tauri::command]
async fn connect_to_device(device_id: String) -> Result<serde_json::Value, String> {
    println!("🔗 اتصال به دستگاه: {}", device_id);
    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

    Ok(serde_json::json!({
        "success": true,
        "device_id": device_id,
    }))
}

// ============================================
// Tauri Commands — File Selection
// ============================================

/// خوندن اطلاعات یه فایل (اسم، اندازه، مسیر)
#[tauri::command]
async fn get_file_info(path: String) -> Result<serde_json::Value, String> {
    let file_path = std::path::Path::new(&path);

    if !file_path.exists() {
        return Err(format!("فایل پیدا نشد: {}", path));
    }

    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("خطا در خواندن فایل: {}", e))?;

    let name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let size = metadata.len();

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let icon = match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" => "🖼️",
        "mp4" | "mkv" | "avi" | "mov" | "webm" => "🎬",
        "mp3" | "wav" | "flac" | "ogg" | "m4a" => "🎵",
        "pdf" => "📕",
        "doc" | "docx" => "📘",
        "xls" | "xlsx" => "📗",
        "ppt" | "pptx" => "📙",
        "zip" | "rar" | "7z" | "tar" | "gz" => "📦",
        "txt" | "md" | "log" => "📄",
        "exe" | "msi" => "⚙️",
        "apk" => "📱",
        _ => "📄",
    };

    Ok(serde_json::json!({
        "name": name,
        "path": path,
        "size": size,
        "extension": ext,
        "icon": icon,
    }))
}

/// خوندن اطلاعات چند فایل با هم
#[tauri::command]
async fn get_files_info(paths: Vec<String>) -> Result<Vec<serde_json::Value>, String> {
    let mut result = Vec::new();

    for path in paths {
        match get_file_info(path).await {
            Ok(info) => result.push(info),
            Err(e) => eprintln!("⚠️ {}", e),
        }
    }

    Ok(result)
}

// ============================================
// Main Entry
// ============================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Fixed WebView2
    #[cfg(all(debug_assertions, target_os = "windows"))]
    {
        let fixed_path = std::path::Path::new("webview2-fixed");
        if fixed_path.exists() {
            if let Ok(abs_path) = std::fs::canonicalize(fixed_path) {
                let path_str = abs_path
                    .to_string_lossy()
                    .trim_start_matches(r"\\?\")
                    .to_string();
                std::env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", &path_str);
                println!("✅ Using fixed WebView2: {}", path_str);
            }
        }
    }

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

    if saved_path.is_some() {
        println!("📂 مسیر ذخیره: {}", saved_path.as_ref().unwrap());
    }
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
            open_file,
            reveal_file,
            get_bluetooth_state,
            get_wifi_state,
            check_location_permission,
            request_enable_bluetooth,
            request_enable_wifi,
            open_location_settings,
            open_system_settings,
            start_device_scan,
            stop_device_scan,
            connect_to_device,
            get_file_info,
            get_files_info,
        ])
        .setup(move |_app| {
            println!("🌐 QuickDrop Desktop آماده‌ست");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("خطا در اجرای Tauri");
}