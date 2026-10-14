// QuickDrop Desktop - Frontend Logic

import { invoke } from "@tauri-apps/api/core";

const SERVER_URL = "http://localhost:8080";

// ============================================
// Types
// ============================================

interface InfoResponse {
    lan_ip: string;
    pair_code: string;
    port: number;
}

interface ConfigResponse {
    save_path: string;
    chunk_size: number;
}

interface ReceivedFile {
    name: string;
    size: number;
    path: string;
    received_at: number;
    duration_secs: number;
}

interface SharedFile {
    id: string;
    name: string;
    size: number;
    path: string;
    shared_at: number;
}

interface ActiveUpload {
    file_name: string;
    file_size: number;
    received: number;
    progress: number;
    speed_mbps: number;
}

interface Device {
    id: string;
    name: string;
    type: "desktop" | "mobile" | "unknown";
    rssi: number;
}

interface PermissionsState {
    location: "granted" | "denied" | "not_requested";
    wifi: "on" | "off" | "unsupported";
    bluetooth: "on" | "off" | "unsupported";
}

interface SelectedFile {
    name: string;
    path: string;
    size: number;
    extension: string;
    icon: string;
}

// ============================================
// State
// ============================================

const state = {
    lanIp: "---.---.---.---",
    pairCode: "----",
    savePath: "",
    serverOnline: false,
    token: "",
    activeUpload: null as ActiveUpload | null,
};

const receivedFiles: ReceivedFile[] = [];
const sharedFiles: SharedFile[] = [];
const scannedDevices: Device[] = [];
const selectedFiles: SelectedFile[] = [];

const permissionsState: PermissionsState = {
    location: "not_requested",
    wifi: "off",
    bluetooth: "off",
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

function authHeaders(): HeadersInit {
    return {
        "Authorization": `Bearer ${state.token}`,
        "Content-Type": "application/json",
    };
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
    // Back buttons
    document.querySelectorAll('.wizard-back-btn').forEach((btn) => {
        btn.addEventListener('click', () => {
            const target = (btn as HTMLElement).dataset.back;
            if (target) showScreen(target + '-screen');
        });
    });

    // Landing: Send
    $("landing-send-btn")?.addEventListener('click', () => {
        showScreen('permissions-screen');
        checkPermissions();
    });

    // Landing: Receive
    $("landing-receive-btn")?.addEventListener('click', () => {
        showScreen('select-files-screen');
    });

    // Landing: QR
    $("landing-qr-btn")?.addEventListener('click', () => {
        showScreen('find-device-screen');
        scanDevices();
    });

    // Permissions: Next
    $("perm-next-btn")?.addEventListener('click', () => {
        showScreen('select-files-screen');
    });

    // Select Files: Next
    $("files-next-btn")?.addEventListener('click', () => {
        showScreen('find-device-screen');
        scanDevices();
    });

    // Select Files: Select Files
    $("select-files-btn")?.addEventListener('click', handleSelectFiles);

    // Discovery Tabs
    document.querySelectorAll('.discovery-tab').forEach((tab) => {
        tab.addEventListener('click', () => {
            const tabId = (tab as HTMLElement).dataset.tab;
            document.querySelectorAll('.discovery-tab').forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            document.querySelectorAll('.discovery-panel').forEach(p => p.classList.remove('active'));
            const panel = document.getElementById(tabId + '-panel');
            if (panel) panel.classList.add('active');

            if (tabId === "bluetooth") {
                scanDevices();
            }
        });
    });

    // Refresh devices
    $("refresh-devices-btn")?.addEventListener('click', () => {
        scanDevices();
    });

    // Cancel transfer
    $("cancel-transfer-btn")?.addEventListener('click', () => {
        showScreen('landing-screen');
    });
}

// ============================================
// Permissions
// ============================================

async function checkPermissions(): Promise<void> {
    try {
        const [loc, wifi, bt] = await Promise.all([
            invoke<string>("check_location_permission"),
            invoke<string>("get_wifi_state"),
            invoke<string>("get_bluetooth_state"),
        ]);

        permissionsState.location = loc as any;
        permissionsState.wifi = wifi as any;
        permissionsState.bluetooth = bt as any;

        updatePermissionsUI();
    } catch (e) {
        console.error("خطا در چک کردن permissions:", e);
    }
}

function updatePermissionsUI(): void {
    // Location
    const locEl = $("perm-location");
    const locBtn = locEl?.querySelector(".permission-btn") as HTMLElement;
    if (permissionsState.location === "granted") {
        locEl?.classList.add("granted");
        if (locBtn) locBtn.textContent = "✓";
    } else {
        locEl?.classList.remove("granted");
        if (locBtn) locBtn.textContent = "Settings";
    }

    // Wi-Fi
    const wifiEl = $("perm-wifi");
    const wifiBtn = wifiEl?.querySelector(".permission-btn") as HTMLElement;
    if (permissionsState.wifi === "on") {
        wifiEl?.classList.add("granted");
        if (wifiBtn) {
            wifiBtn.textContent = "✓";
            wifiBtn.classList.remove("primary");
        }
    } else {
        wifiEl?.classList.remove("granted");
        if (wifiBtn) {
            wifiBtn.textContent = "روشن کن";
            wifiBtn.classList.add("primary");
        }
    }

    // Bluetooth
    const btEl = $("perm-bluetooth");
    const btBtn = btEl?.querySelector(".permission-btn") as HTMLElement;
    if (permissionsState.bluetooth === "on") {
        btEl?.classList.add("granted");
        if (btBtn) {
            btBtn.textContent = "✓";
            btBtn.classList.remove("primary");
        }
    } else {
        btEl?.classList.remove("granted");
        if (btBtn) {
            btBtn.textContent = "روشن کن";
            btBtn.classList.add("primary");
        }
    }

    // Next button
    const nextBtn = $("perm-next-btn") as HTMLButtonElement;
    if (nextBtn) {
        const allOk =
            permissionsState.location === "granted" &&
            permissionsState.wifi === "on" &&
            permissionsState.bluetooth === "on";
        nextBtn.disabled = !allOk;
    }
}

function setupPermissionHandlers(): void {
    document.querySelectorAll(".permission-btn").forEach((btn) => {
        btn.addEventListener("click", async () => {
            const perm = (btn as HTMLElement).dataset.perm;

            try {
                if (perm === "location") {
                    await invoke("open_location_settings");
                } else if (perm === "wifi") {
                    await invoke("request_enable_wifi");
                } else if (perm === "bluetooth") {
                    await invoke("request_enable_bluetooth");
                }

                setTimeout(checkPermissions, 1500);
            } catch (e) {
                console.error("خطا:", e);
            }
        });
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

                const device = scannedDevices.find(d => d.id === deviceId);
                console.log("🎯 اتصال به:", device?.name);

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
        deviceList.innerHTML = `
            <div class="empty-state">
                <div class="empty-icon">❌</div>
                <p>خطا در جستجو</p>
            </div>
        `;
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

async function addFiles(paths: string[]): Promise<void> {
    try {
        const infos = await invoke<SelectedFile[]>("get_files_info", { paths });

        for (const info of infos) {
            if (!selectedFiles.some(f => f.path === info.path)) {
                selectedFiles.push(info);
            }
        }

        renderSelectedFiles();
        console.log(`✅ ${infos.length} فایل اضافه شد`);
    } catch (e) {
        console.error("خطا در افزودن فایل‌ها:", e);
    }
}

async function handleSelectFiles(): Promise<void> {
    try {
        const { open } = await import("@tauri-apps/plugin-dialog");
        const selected = await open({
            multiple: true,
            title: "انتخاب فایل برای ارسال",
        });

        if (!selected) return;

        const paths = Array.isArray(selected) ? selected : [selected];
        await addFiles(paths);
    } catch (e) {
        console.error("خطا در انتخاب فایل:", e);
    }
}

function setupDropZone(): void {
    const dropZone = $("send-drop-zone");
    if (!dropZone) return;

    ["dragenter", "dragover", "dragleave", "drop"].forEach((eventName) => {
        dropZone.addEventListener(eventName, (e) => {
            e.preventDefault();
            e.stopPropagation();
        });
    });

    dropZone.addEventListener("dragenter", () => {
        dropZone.classList.add("drag-over");
    });

    dropZone.addEventListener("dragleave", () => {
        dropZone.classList.remove("drag-over");
    });

    dropZone.addEventListener("drop", async (e) => {
        dropZone.classList.remove("drag-over");

        const dt = (e as DragEvent).dataTransfer;
        if (!dt) return;

        const files = Array.from(dt.files);
        const paths = files
            .map((f) => (f as any).path)
            .filter((p): p is string => !!p);

        if (paths.length > 0) {
            await addFiles(paths);
        }
    });
}

// ============================================
// Load Functions
// ============================================

async function loadQrCode(): Promise<void> {
    const qrContent = $("qr-content");
    try {
        const res = await fetch(`${SERVER_URL}/qr`);
        if (!res.ok) throw new Error("QR fetch failed");
        const svg = await res.text();
        if (qrContent) qrContent.innerHTML = svg;
    } catch (e) {
        setTimeout(loadQrCode, 2000);
    }
}

async function loadToken(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/token`);
        if (!res.ok) throw new Error("Token fetch failed");
        state.token = (await res.text()).trim();
        console.log("🔑 Token loaded");
    } catch (e) {
        console.error("خطا در لود توکن:", e);
        setTimeout(loadToken, 2000);
    }
}

async function loadAppInfo(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/info`);
        if (!res.ok) throw new Error("Info fetch failed");
        const data: InfoResponse = await res.json();
        state.lanIp = data.lan_ip;
        state.pairCode = data.pair_code;
        state.serverOnline = true;
        setText("local-ip", state.lanIp);
        setText("pair-code", state.pairCode);
        updateStatus();
    } catch (e) {
        state.serverOnline = false;
        updateStatus();
        setTimeout(loadAppInfo, 3000);
    }
}

async function loadConfig(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/config`);
        if (!res.ok) throw new Error("Config fetch failed");
        const data: ConfigResponse = await res.json();
        state.savePath = data.save_path;
    } catch (e) {
        setTimeout(loadConfig, 3000);
    }
}

// ============================================
// Polling
// ============================================

const POLLING_INTERVAL = 1500;
let pollingTimer: number | null = null;

async function pollFiles(): Promise<void> {
    await Promise.all([loadReceivedFiles(), loadSharedFiles(), loadActiveUpload()]);
}

function startPolling(): void {
    if (pollingTimer !== null) clearInterval(pollingTimer);
    pollingTimer = window.setInterval(pollFiles, POLLING_INTERVAL);
}

async function loadReceivedFiles(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/files`);
        if (!res.ok) return;
        const files: ReceivedFile[] = await res.json();
        receivedFiles.length = 0;
        receivedFiles.push(...files);
    } catch (e) {}
}

async function loadSharedFiles(): Promise<void> {
    if (!state.token) return;
    try {
        const res = await fetch(`${SERVER_URL}/share`, { headers: authHeaders() });
        if (!res.ok) return;
        const files: SharedFile[] = await res.json();
        sharedFiles.length = 0;
        sharedFiles.push(...files);
        const count = $("shared-count");
        if (count) count.textContent = files.length.toString();
    } catch (e) {}
}

async function loadActiveUpload(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/upload/active`);
        if (!res.ok) return;
        const data = await res.json();
        if (data.active) {
            state.activeUpload = data;
        } else {
            state.activeUpload = null;
        }
    } catch (e) {}
}

// ============================================
// Status
// ============================================

function updateStatus(): void {
    const text = $("status-text");
    if (state.serverOnline) {
        if (text) text.textContent = "آماده";
    } else {
        if (text) text.textContent = "در حال راه‌اندازی";
    }
}

// ============================================
// Setup Handlers
// ============================================

function setupHandlers(): void {
    // placeholder
}

// ============================================
// Init
// ============================================

document.addEventListener("DOMContentLoaded", () => {
    // ۱. فوری: صفحه‌ی Landing
    showScreen('landing-screen');

    // ۲. UI setup (سریع)
    setupHandlers();
    setupWizardNavigation();
    setupPermissionHandlers();
    setupDropZone();

    // ۳. Backend (تو background)
    setTimeout(async () => {
        try {
            await loadToken();
            await loadAppInfo();
            await loadConfig();

            loadQrCode().catch(console.error);
            startPolling();
            checkPermissions();
        } catch (e) {
            console.error("خطا در لود:", e);
        }
    }, 50);
});