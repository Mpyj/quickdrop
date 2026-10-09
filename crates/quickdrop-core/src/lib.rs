//! QuickDrop Core

pub mod auth;
pub mod config;
pub mod network;
pub mod qr;
pub mod server;
pub mod state;
pub mod transfer;

// Re-exports
pub use auth::{generate_token, generate_pair_code, verify_token};
pub use config::ConfigManager;
pub use network::get_lan_ip;
pub use qr::generate_qr_svg;
pub use server::{start_server, DEFAULT_PORT};
pub use state::{AppState, ReceivedFile, SharedFile};
pub use transfer::{TransferManager, UploadSession, CHUNK_SIZE};