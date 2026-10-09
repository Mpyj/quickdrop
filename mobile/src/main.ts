// QuickDrop Mobile - Frontend Logic

import { invoke } from "@tauri-apps/api/core";

// ============================================
// Types
// ============================================

interface SharedFile {
    id: string;
    name: string;
    size: number;
    path: string;
    shared_at: number;
}

// ============================================
// State
// ============================================

const state = {
    localIp: "---.---.---.---",
    pairCode: "----",
    token: "",
    mobileUrl: "",
    sharedFiles: [] as SharedFile[],
};

// ============================================
// Helpers
// ============================================

function $(id: string): HTMLElement | null {
    return document.getElementById(id);
}

function setText(id: string, text: string): void {
    const el = $(id);
    if (el) el.textContent = text;
}

function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

function escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// ============================================
// Load Info from Rust
// ============================================

async function loadInfo(): Promise<void> {
    try {
        state.localIp = await invoke<string>("get_local_ip");
        state.pairCode = await invoke<string>("get_pair_code");
        state.token = await invoke<string>("get_token");
        state.mobileUrl = await invoke<string>("get_mobile_url");

        setText("local-ip", state.localIp);
        setText("pair-code", state.pairCode);

        console.log("📱 IP:", state.localIp);
        console.log("🔐 Pair Code:", state.pairCode);
        console.log("🔑 Token:", state.token.substring(0, 8) + "...");
        console.log("🔗 Mobile URL:", state.mobileUrl);
    } catch (e) {
        console.error("خطا در لود اطلاعات:", e);
    }
}

// ============================================
// Actions (موقت)
// ============================================

function handleUpload(): void {
    console.log("📤 Upload clicked");
    alert("به‌زودی: ارسال فایل به PC");
}

function handleDownload(): void {
    console.log("📥 Download clicked");
    alert("به‌زودی: دریافت فایل از PC");
}

// ============================================
// Setup
// ============================================

function setupHandlers(): void {
    $("upload-btn")?.addEventListener("click", handleUpload);
    $("download-btn")?.addEventListener("click", handleDownload);
}

// ============================================
// Init
// ============================================

document.addEventListener("DOMContentLoaded", async () => {
    console.log("🌐 QuickDrop Mobile loaded");
    setupHandlers();
    await loadInfo();
});