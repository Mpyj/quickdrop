// QuickDrop Mobile - Entry Point

import { invoke } from "@tauri-apps/api/core";

// ============================================
// State
// ============================================

interface SharedFile {
    id: string;
    name: string;
    size: number;
    path: string;
    shared_at: number;
}

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

function showScreen(id: string): void {
    document.querySelectorAll('.screen').forEach(s => s.classList.remove('active'));
    const screen = $(id);
    if (screen) screen.classList.add('active');
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
// Toast
// ============================================

function showToast(type: 'success' | 'error' | 'info', title: string, message?: string): void {
    const container = $("toast-container");
    if (!container) return;

    const icons = { success: '✅', error: '❌', info: 'ℹ️' };

    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.innerHTML = `
        <div class="toast-icon">${icons[type]}</div>
        <div class="toast-body">
            <div class="toast-title">${escapeHtml(title)}</div>
            ${message ? `<div class="toast-message">${escapeHtml(message)}</div>` : ''}
        </div>
    `;

    container.appendChild(toast);

    setTimeout(() => {
        toast.classList.add('leaving');
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}

// ============================================
// Load Info
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
    } catch (e) {
        console.error("خطا در لود اطلاعات:", e);
        showToast('error', 'خطا', 'نتونستم اطلاعات رو لود کنم');
    }
}

// ============================================
// Handlers
// ============================================

function handleSendClick(): void {
    console.log("📤 Send clicked");
    showScreen('send-screen');
}

function handleReceiveClick(): void {
    console.log("📥 Receive clicked");
    showScreen('receive-screen');
}

function handleSettingsClick(): void {
    console.log("⚙️ Settings clicked");
    showToast('info', 'به‌زودی', 'تنظیمات در نسخه‌های بعدی');
}

function handlePickFile(): void {
    console.log("📁 Pick file clicked");
    showToast('info', 'به‌زودی', 'انتخاب فایل در حال ساخت');
}

function goHome(): void {
    showScreen('home-screen');
}

// ============================================
// Setup
// ============================================

function setupHandlers(): void {
    $("send-btn")?.addEventListener("click", handleSendClick);
    $("receive-btn")?.addEventListener("click", handleReceiveClick);
    $("settings-btn")?.addEventListener("click", handleSettingsClick);
    $("pick-file-btn")?.addEventListener("click", handlePickFile);
    $("send-back")?.addEventListener("click", goHome);
    $("receive-back")?.addEventListener("click", goHome);
}

// ============================================
// Init
// ============================================

document.addEventListener("DOMContentLoaded", async () => {
    console.log("🌐 QuickDrop Mobile loaded");
    setupHandlers();
    await loadInfo();

    // بعد از لود، برو به Home
    setTimeout(() => {
        showScreen('home-screen');
    }, 800);
});