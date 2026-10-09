//! وب‌سرور محلی با axum

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::auth;
use crate::qr;
use crate::state::{AppState, ReceivedFile, SharedFile};
use crate::transfer::CHUNK_SIZE;

pub const DEFAULT_PORT: u16 = 8080;

// ============================================
// Router
// ============================================

fn build_router(state: AppState) -> Router {
    let static_service = ServeDir::new("static");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/token", get(token_handler))
        .route("/qr", get(qr_handler))
        .route("/pair", get(pair_handler))
        .route("/m", get(mobile_handler))
        .route("/api/verify", get(verify_handler))
        .route("/config", get(config_handler))
        .route("/config/save-path", post(set_save_path_handler))
        .route("/config/reset-path", post(reset_save_path_handler))
        // فایل‌های دریافتی
        .route("/files", get(files_handler))
        .route("/files/delete", post(delete_received_handler))
        .route("/files/clear", post(clear_received_handler))
        // فایل‌های اشتراکی
        .route("/share", post(share_handler))
        .route("/share", get(shared_list_handler))
        .route("/share/clear", post(clear_shared_handler))
        .route("/share/{id}", delete(shared_delete_handler))
        .route("/download/{id}", get(download_handler))
        // آپلود
        .route("/upload/init", post(upload_init_handler))
        .route("/upload/chunk/{id}", post(upload_chunk_handler))
        .route("/upload/complete/{id}", post(upload_complete_handler))
        .route("/upload/status/{id}", get(upload_status_handler))
        .route("/upload/active", get(upload_active_handler))
        .nest_service("/static", static_service)
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
        .layer(cors)
        .with_state(state)
}

// ============================================
// Helpers
// ============================================

fn check_auth(headers: &HeaderMap, state: &AppState) -> Result<(), (StatusCode, String)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, "هدر Authorization پیدا نشد".to_string())
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, "فرمت Authorization اشتباهه".to_string())
        })?;

    if !auth::verify_token(token, &state.token) {
        return Err((StatusCode::UNAUTHORIZED, "توکن نامعتبره".to_string()));
    }

    Ok(())
}

fn check_auth_query(token: Option<&str>, state: &AppState) -> Result<(), (StatusCode, String)> {
    match token {
        Some(t) if auth::verify_token(t, &state.token) => Ok(()),
        Some(_) => Err((StatusCode::UNAUTHORIZED, "توکن نامعتبره".to_string())),
        None => Err((StatusCode::UNAUTHORIZED, "توکن داده نشده".to_string())),
    }
}

fn extract_token(headers: &HeaderMap, query_token: Option<&str>) -> Option<String> {
    if let Some(auth_header) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            return Some(token.to_string());
        }
    }
    query_token.map(|s| s.to_string())
}

// ============================================
// Handlers - صفحات
// ============================================

async fn root_handler() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>QuickDrop</title></head><body style="font-family: system-ui; padding: 40px;"><h1>✅ QuickDrop Server</h1></body></html>"#)
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn info_handler(State(state): State<AppState>) -> impl IntoResponse {
    let json = format!(
        r#"{{"lan_ip":"{}","pair_code":"{}","port":{}}}"#,
        state.lan_ip, state.pair_code, DEFAULT_PORT
    );
    (StatusCode::OK, [("content-type", "application/json")], json)
}

async fn token_handler(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, [("content-type", "text/plain")], state.token.to_string())
}

async fn qr_handler(State(state): State<AppState>) -> impl IntoResponse {
    let url = state.mobile_url();
    match qr::generate_qr_svg(&url) {
        Ok(svg) => (StatusCode::OK, [("content-type", "image/svg+xml")], svg).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("خطا: {}", e)).into_response(),
    }
}

#[derive(Deserialize)]
struct PairQuery {
    code: Option<String>,
}

async fn pair_handler(
    State(state): State<AppState>,
    Query(query): Query<PairQuery>,
) -> impl IntoResponse {
    match query.code {
        Some(code) if code == *state.pair_code => (StatusCode::OK, format!("OK:{}", state.token)),
        Some(_) => (StatusCode::UNAUTHORIZED, "کد اشتباه".to_string()),
        None => (StatusCode::BAD_REQUEST, "کد وارد نشده".to_string()),
    }
}

#[derive(Deserialize)]
struct MobileQuery {
    #[allow(dead_code)]
    t: Option<String>,
}

async fn mobile_handler(
    State(_state): State<AppState>,
    Query(_query): Query<MobileQuery>,
) -> Response {
    let html = match tokio::fs::read_to_string("static/mobile.html").await {
        Ok(h) => h,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("خطا: {}", e)).into_response();
        }
    };
    (StatusCode::OK, Html(html)).into_response()
}

#[derive(Deserialize)]
struct VerifyQuery {
    t: Option<String>,
}

async fn verify_handler(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    match check_auth_query(query.t.as_deref(), &state) {
        Ok(()) => (StatusCode::OK, "OK".to_string()),
        Err((code, msg)) => (code, msg),
    }
}

// ============================================
// Config
// ============================================

async fn config_handler(State(state): State<AppState>) -> impl IntoResponse {
    let save_path = state.transfer.get_save_path().await;
    let json = serde_json::json!({
        "save_path": save_path.to_string_lossy(),
        "chunk_size": CHUNK_SIZE,
    });
    (StatusCode::OK, axum::Json(json))
}

#[derive(Deserialize)]
struct SetSavePathRequest {
    path: String,
}

async fn set_save_path_handler(
    State(state): State<AppState>,
    axum::Json(req): axum::Json<SetSavePathRequest>,
) -> impl IntoResponse {
    let path = std::path::Path::new(&req.path);

    if !path.exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            return (StatusCode::BAD_REQUEST, format!("خطا در ساخت پوشه: {}", e)).into_response();
        }
    }

    if let Err(e) = state.config.set_save_path(req.path.clone()).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
    }

    if let Err(e) = state.transfer.set_save_path(std::path::PathBuf::from(&req.path)).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
    }

    println!("📂 مسیر ذخیره تغییر کرد: {}", req.path);

    (StatusCode::OK, "OK").into_response()
}

#[derive(Serialize)]
struct ResetPathResponse {
    save_path: String,
}

async fn reset_save_path_handler(State(state): State<AppState>) -> impl IntoResponse {
    let default_path = if let Some(downloads) = dirs::download_dir() {
        downloads.join("QuickDrop")
    } else {
        std::path::PathBuf::from("./QuickDrop")
    };

    if let Err(e) = std::fs::create_dir_all(&default_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("خطا: {}", e)).into_response();
    }

    let default_str = default_path.to_string_lossy().to_string();

    if let Err(e) = state.config.set_save_path(default_str.clone()).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
    }

    if let Err(e) = state.transfer.set_save_path(default_path).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
    }

    println!("🔄 مسیر ذخیره ریست شد به: {}", default_str);

    let response = ResetPathResponse {
        save_path: default_str,
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

// ============================================
// Files (Received)
// ============================================

async fn files_handler(State(state): State<AppState>) -> impl IntoResponse {
    let files = state.received_files.read().await;
    let list: Vec<ReceivedFile> = files.iter().cloned().collect();
    (StatusCode::OK, axum::Json(list))
}

#[derive(Deserialize)]
struct DeleteReceivedRequest {
    path: String,
}

/// حذف یه فایل دریافتی از لیست
async fn delete_received_handler(
    State(state): State<AppState>,
    axum::Json(req): axum::Json<DeleteReceivedRequest>,
) -> impl IntoResponse {
    let removed = {
        let mut files = state.received_files.write().await;
        let len_before = files.len();
        files.retain(|f| f.path != req.path);
        len_before - files.len()
    };

    if removed > 0 {
        state.persist_received_files().await;
        println!("🗑 فایل دریافتی حذف شد از لیست: {}", req.path);
        (StatusCode::OK, "حذف شد").into_response()
    } else {
        (StatusCode::NOT_FOUND, "پیدا نشد").into_response()
    }
}

/// حذف همه‌ی فایل‌های دریافتی از لیست
async fn clear_received_handler(State(state): State<AppState>) -> impl IntoResponse {
    let count = {
        let mut files = state.received_files.write().await;
        let len = files.len();
        files.clear();
        len
    };

    state.persist_received_files().await;
    println!("🗑 همه‌ی فایل‌های دریافتی حذف شدن ({} فایل)", count);

    let json = serde_json::json!({ "removed": count });
    (StatusCode::OK, axum::Json(json)).into_response()
}

// ============================================
// Shared Files
// ============================================

#[derive(Deserialize)]
struct ShareRequest {
    path: String,
}

#[derive(Serialize)]
struct ShareResponse {
    id: String,
    name: String,
    size: u64,
}

async fn share_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<ShareRequest>,
) -> impl IntoResponse {
    if let Err((code, msg)) = check_auth(&headers, &state) {
        return (code, msg).into_response();
    }

    let path = std::path::PathBuf::from(&req.path);

    if !path.exists() {
        return (StatusCode::NOT_FOUND, "فایل پیدا نشد").into_response();
    }
    if !path.is_file() {
        return (StatusCode::BAD_REQUEST, "مسیر یه فایل نیست").into_response();
    }

    let metadata = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("خطا: {}", e)).into_response(),
    };

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let id = uuid::Uuid::new_v4().to_string();

    let shared_file = SharedFile {
        id: id.clone(),
        name: file_name.clone(),
        size: metadata.len(),
        path: path.to_string_lossy().to_string(),
        shared_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };

    {
        let mut files = state.shared_files.write().await;
        files.push_front(shared_file);
        while files.len() > 50 {
            files.pop_back();
        }
    }

    state.persist_shared_files().await;

    println!("📤 فایل اشتراکی: {} ({} بایت)", file_name, metadata.len());

    let response = ShareResponse {
        id,
        name: file_name,
        size: metadata.len(),
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

async fn shared_list_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err((code, msg)) = check_auth(&headers, &state) {
        return (code, msg).into_response();
    }

    let files = state.shared_files.read().await;
    let list: Vec<SharedFile> = files.iter().cloned().collect();
    (StatusCode::OK, axum::Json(list)).into_response()
}

async fn shared_delete_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err((code, msg)) = check_auth(&headers, &state) {
        return (code, msg).into_response();
    }

    let removed;
    {
        let mut files = state.shared_files.write().await;
        let len_before = files.len();
        files.retain(|f| f.id != id);
        removed = len_before - files.len();
    }

    if removed > 0 {
        state.persist_shared_files().await;
        println!("🗑 فایل اشتراکی حذف شد: {}", id);
        (StatusCode::OK, "حذف شد").into_response()
    } else {
        (StatusCode::NOT_FOUND, "پیدا نشد").into_response()
    }
}

/// حذف همه‌ی فایل‌های اشتراکی
async fn clear_shared_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err((code, msg)) = check_auth(&headers, &state) {
        return (code, msg).into_response();
    }

    let count = {
        let mut files = state.shared_files.write().await;
        let len = files.len();
        files.clear();
        len
    };

    state.persist_shared_files().await;
    println!("🗑 همه‌ی فایل‌های اشتراکی حذف شدن ({} فایل)", count);

    let json = serde_json::json!({ "removed": count });
    (StatusCode::OK, axum::Json(json)).into_response()
}

#[derive(Deserialize)]
struct DownloadQuery {
    t: Option<String>,
}

async fn download_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<DownloadQuery>,
) -> Response {
    let token = extract_token(&headers, query.t.as_deref());
    match token {
        Some(t) if auth::verify_token(&t, &state.token) => {}
        _ => return (StatusCode::UNAUTHORIZED, "توکن نامعتبره").into_response(),
    }

    let file = {
        let files = state.shared_files.read().await;
        files.iter().find(|f| f.id == id).cloned()
    };

    let file = match file {
        Some(f) => f,
        None => return (StatusCode::NOT_FOUND, "فایل پیدا نشد").into_response(),
    };

    let file_handle = match tokio::fs::File::open(&file.path).await {
        Ok(f) => f,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("خطا: {}", e)).into_response(),
    };

    let stream = tokio_util::io::ReaderStream::new(file_handle);
    let body = axum::body::Body::from_stream(stream);

    let content_disposition = format!("attachment; filename=\"{}\"", file.name.replace('"', ""));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, file.size.to_string())
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(body)
        .unwrap()
}

// ============================================
// Upload
// ============================================

#[derive(Deserialize)]
struct UploadInitRequest {
    file_name: String,
    file_size: u64,
}

#[derive(Serialize)]
struct UploadInitResponse {
    session_id: String,
    chunk_size: usize,
}

async fn upload_init_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<UploadInitRequest>,
) -> impl IntoResponse {
    if let Err((code, msg)) = check_auth(&headers, &state) {
        return (code, msg).into_response();
    }

    if req.file_name.is_empty() {
        return (StatusCode::BAD_REQUEST, "اسم فایل خالیه").into_response();
    }
    if req.file_size == 0 {
        return (StatusCode::BAD_REQUEST, "اندازه‌ی فایل صفره").into_response();
    }

    let token = state.token.to_string();
    match state.transfer.create_session(req.file_name, req.file_size, token).await {
        Ok(session) => {
            let response = UploadInitResponse {
                session_id: session.id,
                chunk_size: CHUNK_SIZE,
            };
            (StatusCode::OK, axum::Json(response)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[derive(Serialize)]
struct ChunkResponse {
    received: u64,
    total: u64,
    progress: f64,
    complete: bool,
}

async fn upload_chunk_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    if let Err((code, msg)) = check_auth(&headers, &state) {
        return (code, msg).into_response();
    }

    if body.is_empty() {
        return (StatusCode::BAD_REQUEST, "بدنه‌ی خالی").into_response();
    }

    match state.transfer.append_chunk(&session_id, &body).await {
        Ok((received, complete)) => {
            if let Some(session) = state.transfer.get_session(&session_id).await {
                let response = ChunkResponse {
                    received,
                    total: session.file_size,
                    progress: session.progress_percent(),
                    complete,
                };
                (StatusCode::OK, axum::Json(response)).into_response()
            } else {
                (StatusCode::NOT_FOUND, "session پیدا نشد").into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[derive(Serialize)]
struct CompleteResponse {
    saved_to: String,
    file_name: String,
    file_size: u64,
}

async fn upload_complete_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    if let Err((code, msg)) = check_auth(&headers, &state) {
        return (code, msg).into_response();
    }

    let (file_name, file_size, elapsed_secs) = match state.transfer.get_session(&session_id).await {
        Some(s) => (
            s.file_name.clone(),
            s.received,
            s.started_at.elapsed().as_secs_f64(),
        ),
        None => return (StatusCode::NOT_FOUND, "session پیدا نشد").into_response(),
    };

    match state.transfer.complete_session(&session_id).await {
        Ok(path) => {
            let received_file = ReceivedFile {
                name: file_name.clone(),
                size: file_size,
                path: path.to_string_lossy().to_string(),
                received_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                duration_secs: elapsed_secs,
            };

            {
                let mut files = state.received_files.write().await;
                files.push_front(received_file);
                while files.len() > 100 {
                    files.pop_back();
                }
            }

            state.persist_received_files().await;

            let response = CompleteResponse {
                saved_to: path.to_string_lossy().to_string(),
                file_name,
                file_size,
            };
            (StatusCode::OK, axum::Json(response)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[derive(Serialize)]
struct StatusResponse {
    session_id: String,
    file_name: String,
    file_size: u64,
    received: u64,
    progress: f64,
    speed_mbps: f64,
}

async fn upload_status_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    if let Err((code, msg)) = check_auth(&headers, &state) {
        return (code, msg).into_response();
    }

    match state.transfer.get_session(&session_id).await {
        Some(session) => {
            let response = StatusResponse {
                session_id: session.id.clone(),
                file_name: session.file_name.clone(),
                file_size: session.file_size,
                received: session.received,
                progress: session.progress_percent(),
                speed_mbps: session.average_speed() / 1_048_576.0,
            };
            (StatusCode::OK, axum::Json(response)).into_response()
        }
        None => (StatusCode::NOT_FOUND, "session پیدا نشد").into_response(),
    }
}

// ============================================
// Active Upload
// ============================================

#[derive(Serialize)]
struct ActiveUploadResponse {
    active: bool,
    file_name: String,
    file_size: u64,
    received: u64,
    progress: f64,
    speed_mbps: f64,
}

async fn upload_active_handler(State(state): State<AppState>) -> impl IntoResponse {
    let session_opt = state.transfer.get_latest_active().await;

    match session_opt {
        Some(s) => {
            let response = ActiveUploadResponse {
                active: true,
                file_name: s.file_name.clone(),
                file_size: s.file_size,
                received: s.received,
                progress: s.progress_percent(),
                speed_mbps: s.average_speed() / 1_048_576.0,
            };
            (StatusCode::OK, axum::Json(response)).into_response()
        }
        None => {
            let response = ActiveUploadResponse {
                active: false,
                file_name: String::new(),
                file_size: 0,
                received: 0,
                progress: 0.0,
                speed_mbps: 0.0,
            };
            (StatusCode::OK, axum::Json(response)).into_response()
        }
    }
}

// ============================================
// Server Launcher
// ============================================

pub async fn start_server(state: AppState) -> Result<(), String> {
    let router = build_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT));

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("نتونستم به {} گوش بدم: {}", addr, e))?;

    println!("✅ QuickDrop Server running on http://0.0.0.0:{}", DEFAULT_PORT);

    axum::serve(listener, router)
        .await
        .map_err(|e| format!("خطا در سرور: {}", e))
}