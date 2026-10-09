// QuickDrop Mobile - Frontend Logic

import { invoke } from "@tauri-apps/api/core";

// ============================================
// State
// ============================================

const state = {
    localIp: "---.---.---.---",
    pairCode: "----",
    token: "",
    mobileUrl: "",
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
        console.log("🔗 URL:", state.mobileUrl);
    } catch (e) {
        console.error("خطا در لود اطلاعات:", e);
    }
}

// ============================================
// Actions
// ============================================

function handleSend(): void {
    console.log("📤 Send clicked");
    alert("به‌زودی: انتخاب فایل و ارسال");
}

function handleReceive(): void {
    console.log("📥 Receive clicked");
    alert("به‌زودی: دریافت فایل از دستگاه‌های دیگه");
}

function handleSettings(): void {
    console.log("⚙️ Settings clicked");
    alert("به‌زودی: تنظیمات");
}

// ============================================
// Setup
// ============================================

function setupHandlers(): void {
    $("send-btn")?.addEventListener("click", handleSend);
    $("receive-btn")?.addEventListener("click", handleReceive);
    $("settings-btn")?.addEventListener("click", handleSettings);
}

// ============================================
// Init
// ============================================

document.addEventListener("DOMContentLoaded", async () => {
    console.log("🌐 QuickDrop Mobile loaded");
    setupHandlers();
    await loadInfo();
    
    // نمایش صفحه‌ی Home بعد از لود
    setTimeout(() => {
        showScreen('home-screen');
    }, 500);
});