// QuickDrop Mobile - Frontend Logic

import { invoke } from "@tauri-apps/api/core";
import { checkPermissions } from "@mnlphlp/plugin-blec";

// ============================================
// Types
// ============================================

interface InfoResponse {
    lan_ip: string;
    pair_code: string;
}

interface Device {
    id: string;
    name: string;
    type: "desktop" | "mobile" | "unknown";
    rssi: number;
}

interface SelectedFile {
    name: string;
    path: string;
    size: number;
    icon: string;
}

interface PermissionsState {
    location: boolean;
    wifi: boolean;
    bluetooth: boolean;
}

// ============================================
// State
// ============================================

const state = {
    lanIp: "---.---.---.---",
    pairCode: "----",
    token: "",
    serverOnline: false,
};

const selectedFiles: SelectedFile[] = [];
const scannedDevices: Device[] = [];

const permissionsState: PermissionsState = {
    location: false,
    wifi: false,
    bluetooth: false,
};

// ============================================
// DOM Helpers
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
// Screen Management
// ============================================

function showScreen(id: string): void {
    document.querySelectorAll('.screen').forEach(s => s.classList.remove('active'));
    const screen = document.getElementById(id);
    if (screen) screen.classList.add('active');
}

// ============================================
// Wizard Navigation
// ============================================

function setupWizardNavigation(): void {
    document.querySelectorAll('.wizard-back-btn').forEach((btn) => {
        btn.addEventListener('click', () => {
            const target = (btn as HTMLElement).dataset.back;
            if (target) showScreen(target + '-screen');
        });
    });

    $("landing-send-btn")?.addEventListener('click', () => {
        showScreen('permissions-screen');
        requestBlePermissions();
    });

    $("landing-receive-btn")?.addEventListener('click', () => {
        showScreen('select-files-screen');
    });

    $("landing-qr-btn")?.addEventListener('click', () => {
        showScreen('find-device-screen');
        scanDevices();
    });

    $("perm-next-btn")?.addEventListener('click', () => {
        showScreen('select-files-screen');
    });

    $("files-next-btn")?.addEventListener('click', () => {
        showScreen('find-device-screen');
        scanDevices();
    });

    $("select-files-btn")?.addEventListener('click', handleSelectFiles);

    document.querySelectorAll('.discovery-tab').forEach((tab) => {
        tab.addEventListener('click', () => {
            const tabId = (tab as HTMLElement).dataset.tab;
            document.querySelectorAll('.discovery-tab').forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            document.querySelectorAll('.discovery-panel').forEach(p => p.classList.remove('active'));
            const panel = document.getElementById(tabId + '-panel');
            if (panel) panel.classList.add('active');

            if (tabId === "bluetooth") scanDevices();
        });
    });

    $("refresh-devices-btn")?.addEventListener('click', () => {
        scanDevices();
    });

    $("cancel-transfer-btn")?.addEventListener('click', () => {
        showScreen('landing-screen');
    });
}

// ============================================
// Permissions — با تأیید دستی
// ============================================

/// مرحله ۱: گرفتن Permission های BLE
async function requestBlePermissions(): Promise<void> {
    console.log("🔍 گرفتن BLE Permissions...");

    try {
        // این تابع خودش از کاربر می‌پرسه و به Settings می‌فرسته
        await checkPermissions(true);
        console.log("✅ BLE Permissions گرفته شد");

        // حالا UI رو آپدیت کن (اول همه false)
        updatePermissionsUI();
    } catch (e) {
        console.error("❌ خطا در گرفتن permissions:", e);
    }
}

/// مرحله ۲: کاربر روی «روشن کن» می‌زنه
async function handleTurnOnBluetooth(): Promise<void> {
    console.log("🔵 درخواست روشن کردن Bluetooth");

    try {
        // این تابع یه Intent اندروید باز می‌کنه
        // که پنجره‌ی روشن کردن Bluetooth رو نشون میده
        await invoke("request_enable_bluetooth");
    } catch (e) {
        console.error("خطا:", e);
    }
}

async function handleTurnOnWifi(): Promise<void> {
    console.log("📶 درخواست روشن کردن Wi-Fi");

    try {
        await invoke("request_enable_wifi");
    } catch (e) {
        console.error("خطا:", e);
    }
}

async function handleLocationSettings(): Promise<void> {
    console.log("📍 باز کردن Settings Location");

    try {
        await invoke("open_location_settings");
    } catch (e) {
        console.error("خطا:", e);
    }
}

/// کاربر تأیید می‌کنه
function togglePermission(perm: "location" | "wifi" | "bluetooth"): void {
    permissionsState[perm] = !permissionsState[perm];
    updatePermissionsUI();
}

/// بعد از اینکه کاربر از Settings برگشت، چک می‌کنه
/// که کدوم دکمه‌ها باید تیک بخورن
function markAsOn(perm: "location" | "wifi" | "bluetooth"): void {
    permissionsState[perm] = true;
    updatePermissionsUI();
}

// ============================================
// Permissions UI
// ============================================

function updatePermissionsUI(): void {
    // Location
    const locEl = $("perm-location");
    if (permissionsState.location) {
        locEl?.classList.add("granted");
    } else {
        locEl?.classList.remove("granted");
    }

    // Wi-Fi
    const wifiEl = $("perm-wifi");
    if (permissionsState.wifi) {
        wifiEl?.classList.add("granted");
    } else {
        wifiEl?.classList.remove("granted");
    }

    // Bluetooth
    const btEl = $("perm-bluetooth");
    if (permissionsState.bluetooth) {
        btEl?.classList.add("granted");
    } else {
        btEl?.classList.remove("granted");
    }

    // Next button
    const nextBtn = $("perm-next-btn") as HTMLButtonElement;
    if (nextBtn) {
        const allOk = permissionsState.location &&
                      permissionsState.wifi &&
                      permissionsState.bluetooth;
        nextBtn.disabled = !allOk;
    }
}

function setupPermissionHandlers(): void {
    // دکمه‌ی Location → Settings
    document.querySelector('.permission-btn[data-perm="location"]')?.addEventListener("click", () => {
        handleLocationSettings();
        // بعد از برگشت از Settings، خودکار تأیید کن
        setTimeout(() => markAsOn("location"), 2000);
    });

    // دکمه‌ی Wi-Fi → Intent
    document.querySelector('.permission-btn[data-perm="wifi"]')?.addEventListener("click", async () => {
        await handleTurnOnWifi();
        // بعد از برگشت، تأیید کن
        setTimeout(() => markAsOn("wifi"), 2000);
    });

    // دکمه‌ی Bluetooth → Intent
    document.querySelector('.permission-btn[data-perm="bluetooth"]')?.addEventListener("click", async () => {
        await handleTurnOnBluetooth();
        // بعد از برگشت، تأیید کن
        setTimeout(() => markAsOn("bluetooth"), 2000);
    });
}

// ============================================
// Device Scan
// ============================================

async function scanDevices(): Promise<void> {
    const deviceList = $("device-list");
    if (!deviceList) return;

    deviceList.innerHTML = `
        <div class="empty-state">
            <div class="empty-icon">📡</div>
            <p>در حال جستجو...</p>
        </div>
    `;

    try {
        const devices = await invoke<Device[]>("start_device_scan");
        scannedDevices.length = 0;
        scannedDevices.push(...devices);

        if (devices.length === 0) {
            deviceList.innerHTML = `
                <div class="empty-state">
                    <div class="empty-icon">📡</div>
                    <p>هیچ دستگاهی پیدا نشد</p>
                </div>
            `;
            return;
        }

        deviceList.innerHTML = devices.map((device) => {
            const icon = device.type === "desktop" ? "💻" : "📱";
            const typeLabel = device.type === "desktop" ? "کامپیوتر" : "گوشی";

            return `
                <div class="device-item" data-device-id="${device.id}">
                    <div class="device-item-icon">${icon}</div>
                    <div class="device-item-body">
                        <div class="device-item-name">${escapeHtml(device.name)}</div>
                        <div class="device-item-type">${typeLabel} • ${device.rssi} dBm</div>
                    </div>
                </div>
            `;
        }).join("");

        deviceList.querySelectorAll(".device-item").forEach((item) => {
            item.addEventListener("click", async () => {
                const deviceId = (item as HTMLElement).dataset.deviceId;
                if (!deviceId) return;

                try {
                    await invoke("connect_to_device", { deviceId });
                    showScreen("transfer-screen");
                } catch (e) {
                    console.error("خطا در اتصال:", e);
                }
            });
        });
    } catch (e) {
        console.error("خطا در scan:", e);
    }
}

// ============================================
// File Selection
// ============================================

function renderSelectedFiles(): void {
    const container = $("selected-files");
    const countEl = $("files-count");
    const nextBtn = $("files-next-btn") as HTMLButtonElement;

    if (!container) return;

    if (countEl) countEl.textContent = selectedFiles.length.toString();
    if (nextBtn) nextBtn.disabled = selectedFiles.length === 0;

    if (selectedFiles.length === 0) {
        container.innerHTML = "";
        return;
    }

    container.innerHTML = selectedFiles.map((file, index) => `
        <div class="selected-file-item" data-index="${index}">
            <div class="selected-file-icon">${file.icon}</div>
            <div class="selected-file-info">
                <div class="selected-file-name" title="${escapeHtml(file.name)}">${escapeHtml(file.name)}</div>
                <div class="selected-file-size">${formatBytes(file.size)}</div>
            </div>
            <button class="selected-file-remove" data-index="${index}" title="حذف">✕</button>
        </div>
    `).join("");

    container.querySelectorAll(".selected-file-remove").forEach((btn) => {
        btn.addEventListener("click", (e) => {
            e.stopPropagation();
            const idx = parseInt((btn as HTMLElement).dataset.index || "0", 10);
            selectedFiles.splice(idx, 1);
            renderSelectedFiles();
        });
    });
}

async function handleSelectFiles(): Promise<void> {
    const fileInput = $("file-input") as HTMLInputElement;
    if (!fileInput) return;
    fileInput.click();
}

function setupFileInput(): void {
    const fileInput = $("file-input") as HTMLInputElement;
    if (!fileInput) return;

    fileInput.addEventListener("change", async (e) => {
        const files = (e.target as HTMLInputElement).files;
        if (!files) return;

        for (const file of Array.from(files)) {
            const ext = file.name.split('.').pop()?.toLowerCase() || '';
            const icon = getFileIcon(ext);

            selectedFiles.push({
                name: file.name,
                path: file.name,
                size: file.size,
                icon,
            });
        }

        renderSelectedFiles();
        fileInput.value = "";
    });
}

function getFileIcon(ext: string): string {
    const icons: Record<string, string> = {
        jpg: "🖼️", jpeg: "🖼️", png: "🖼️", gif: "🖼️", webp: "🖼️",
        mp4: "🎬", mkv: "🎬", avi: "🎬", mov: "🎬",
        mp3: "🎵", wav: "🎵", flac: "🎵", m4a: "🎵",
        pdf: "📕", doc: "📘", docx: "📘",
        xls: "📗", xlsx: "📗",
        ppt: "📙", pptx: "📙",
        zip: "📦", rar: "📦", "7z": "📦",
        txt: "📄", md: "📄",
        apk: "📱",
    };
    return icons[ext] || "📄";
}

// ============================================
// Init
// ============================================

document.addEventListener("DOMContentLoaded", () => {
    console.log("🌐 QuickDrop Mobile loaded");
    showScreen('landing-screen');

    setupWizardNavigation();
    setupPermissionHandlers();
    setupFileInput();

    setTimeout(async () => {
        try {
            const info = await invoke<InfoResponse>("get_app_info");
            setText("local-ip", info.lan_ip);
            setText("pair-code", info.pair_code);
        } catch (e) {
            console.error("خطا در لود اطلاعات:", e);
        }
    }, 100);
});