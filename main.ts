// QuickDrop Desktop - Frontend Logic

import { toast } from "./ts/toast";
import { celebrate } from "./ts/confetti";
import { playIncoming, playOutgoing } from "./ts/sound";
import { enableDragDrop } from "./ts/dragdrop";
import { showContextMenu } from "./ts/contextmenu";

const SERVER_URL = "http://localhost:8080";

interface InfoResponse {
    lan_ip: string;
    pair_code: string;
    port: number;
}

interface ConfigResponse {
    save_path: string;
    chunk_size: number;
}

interface ResetPathResponse {
    save_path: string;
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

const state = {
    lanIp: "---.---.---.---",
    pairCode: "- - - -",
    savePath: "",
    serverOnline: false,
    token: "",
    activeUpload: null as ActiveUpload | null,
    lastActiveUpload: null as ActiveUpload | null,
};

const receivedFiles: ReceivedFile[] = [];
const sharedFiles: SharedFile[] = [];

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

function formatSpeed(bytes: number, seconds: number): string {
    if (seconds < 0.1) return '--';
    return formatBytes(bytes / seconds) + '/s';
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

// === QR ===
async function loadQrCode(): Promise<void> {
    const qrContent = $("qr-content");
    const qrLoading = $("qr-loading");
    try {
        const res = await fetch(`${SERVER_URL}/qr`);
        if (!res.ok) throw new Error("QR fetch failed");
        const svg = await res.text();
        if (qrContent) qrContent.innerHTML = svg;
        qrLoading?.classList.add("hidden");
    } catch (e) {
        setTimeout(loadQrCode, 2000);
    }
}

// === Token ===
async function loadToken(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/token`);
        if (!res.ok) throw new Error("Token fetch failed");
        state.token = (await res.text()).trim();
        console.log("🔑 Token loaded:", state.token.substring(0, 8) + "...");
    } catch (e) {
        console.error("خطا در لود توکن:", e);
        setTimeout(loadToken, 2000);
    }
}

// === App Info ===
async function loadAppInfo(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/info`);
        if (!res.ok) throw new Error("Info fetch failed");
        const data: InfoResponse = await res.json();
        state.lanIp = data.lan_ip;
        state.pairCode = data.pair_code;
        state.serverOnline = true;
        setText("lan-ip", state.lanIp);
        setText("pair-code", state.pairCode);
        updateStatus();
    } catch (e) {
        state.serverOnline = false;
        updateStatus();
        setTimeout(loadAppInfo, 3000);
    }
}

// === Config ===
async function loadConfig(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/config`);
        if (!res.ok) throw new Error("Config fetch failed");
        const data: ConfigResponse = await res.json();
        state.savePath = data.save_path;
        setText("save-path", data.save_path);
    } catch (e) {
        setTimeout(loadConfig, 3000);
    }
}

// ============================================
// Polling (هوشمند — موقع انتقال، سبک‌تر)
// ============================================

const POLLING_INTERVAL = 1500;
let pollingTimer: number | null = null;

async function pollFiles(): Promise<void> {
    // اگه انتقال فعال هست، فقط active رو چک کن (سبک‌تر)
    if (state.activeUpload) {
        await loadActiveUpload();
        return;
    }

    // در حالت عادی، همه‌چیز رو چک کن
    await Promise.all([loadReceivedFiles(), loadSharedFiles(), loadActiveUpload()]);
}

function startPolling(): void {
    if (pollingTimer !== null) {
        clearInterval(pollingTimer);
    }
    pollingTimer = window.setInterval(pollFiles, POLLING_INTERVAL);
    console.log("✅ Polling started");
}

function stopPolling(): void {
    if (pollingTimer !== null) {
        clearInterval(pollingTimer);
        pollingTimer = null;
        console.log("⏹ Polling stopped");
    }
}

// === Load Received Files ===
async function loadReceivedFiles(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/files`);
        if (!res.ok) return;
        const files: ReceivedFile[] = await res.json();

        // چک محتوا، نه فقط تعداد
        const changed =
            files.length !== receivedFiles.length ||
            files.some((f, i) => receivedFiles[i]?.path !== f.path);

        if (changed) {
            receivedFiles.length = 0;
            receivedFiles.push(...files);
            renderReceivedFiles();
        }
    } catch (e) {}
}

// === Load Shared Files ===
async function loadSharedFiles(): Promise<void> {
    if (!state.token) return;
    try {
        const res = await fetch(`${SERVER_URL}/share`, {
            headers: authHeaders(),
        });
        if (!res.ok) return;
        const files: SharedFile[] = await res.json();

        const changed =
            files.length !== sharedFiles.length ||
            files.some((f, i) => sharedFiles[i]?.id !== f.id);

        if (changed) {
            sharedFiles.length = 0;
            sharedFiles.push(...files);
            renderSharedFiles();
        }
    } catch (e) {}
}

// === Active Upload ===
async function loadActiveUpload(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/upload/active`);
        if (!res.ok) return;
        const data = await res.json();
        if (data.active) {
            state.activeUpload = data;
            state.lastActiveUpload = data;
            showLiveTransfer();
        } else {
            if (state.activeUpload) {
                state.activeUpload = null;
                hideLiveTransfer();
            }
        }
    } catch (e) {}
}

function showLiveTransfer(): void {
    if (!state.activeUpload) return;
    const banner = $("live-transfer");
    if (!banner) return;

    banner.classList.add("active");

    const icon = $("live-icon");
    const title = $("live-title");
    const percent = $("live-percent");
    const fill = $("live-progress-fill");
    const size = $("live-size");
    const speed = $("live-speed");

    if (icon) icon.textContent = "📥";
    if (title) title.textContent = `در حال دریافت: ${state.activeUpload.file_name}`;
    if (percent) percent.textContent = `${state.activeUpload.progress.toFixed(1)}%`;
    if (fill) fill.style.width = `${state.activeUpload.progress}%`;
    if (size) size.textContent = `${formatBytes(state.activeUpload.received)} / ${formatBytes(state.activeUpload.file_size)}`;
    if (speed) speed.textContent = `${state.activeUpload.speed_mbps.toFixed(1)} MB/s`;

    const badge = $("status-badge");
    const text = $("status-text");
    badge?.classList.remove("online");
    badge?.classList.add("transferring");
    if (text) text.textContent = `دریافت: ${state.activeUpload.progress.toFixed(0)}%`;
}

function hideLiveTransfer(): void {
    const banner = $("live-transfer");
    banner?.classList.remove("active");

    const badge = $("status-badge");
    badge?.classList.remove("transferring");
    badge?.classList.add("online");
    const text = $("status-text");
    if (text) text.textContent = "آماده‌ی دریافت";

    const fileName = state.lastActiveUpload?.file_name || "فایل";
    celebrate();
    playIncoming();
    toast.success("📥 فایل دریافت شد", fileName);

    loadReceivedFiles();
}

// === Render ===
function renderReceivedFiles(): void {
    const container = $("incoming-content");
    const badge = $("incoming-count");
    if (badge) badge.textContent = receivedFiles.length.toString();
    if (!container) return;

    if (receivedFiles.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-icon">✨</div>
                <p>هنوز فایلی دریافت نشده</p>
            </div>
        `;
        return;
    }

    container.innerHTML = receivedFiles.map((file) => `
        <div class="file-item" data-path="${escapeHtml(file.path)}">
            <div class="file-icon">📄</div>
            <div class="file-info">
                <div class="file-name" title="${escapeHtml(file.name)}">${escapeHtml(file.name)}</div>
                <div class="file-meta">${formatBytes(file.size)} • ⚡ ${formatSpeed(file.size, file.duration_secs)}</div>
            </div>
            <button class="file-delete" data-path="${escapeHtml(file.path)}" title="حذف از لیست">✕</button>
        </div>
    `).join("");

    container.querySelectorAll<HTMLElement>(".file-item").forEach((item) => {
        item.addEventListener("contextmenu", (e) => {
            e.preventDefault();
            const path = item.dataset.path;
            if (!path) return;

            showContextMenu(e.clientX, e.clientY, [
                { id: "open", icon: "📂", label: "باز کردن فایل", action: () => openPath(path) },
                { id: "show", icon: "📁", label: "نمایش در پوشه", action: () => revealPath(path) },
                {
                    id: "copy", icon: "📋", label: "کپی مسیر",
                    action: () => {
                        navigator.clipboard.writeText(path);
                        toast.success("📋 کپی شد", "مسیر فایل کپی شد", false);
                    },
                },
                {
                    id: "delete", icon: "🗑", label: "حذف از لیست",
                    action: () => deleteReceived(path),
                    danger: true,
                },
            ]);
        });
    });

    container.querySelectorAll<HTMLElement>(".file-delete").forEach((btn) => {
        btn.addEventListener("click", (e) => {
            e.stopPropagation();
            const path = btn.dataset.path;
            if (path) deleteReceived(path);
        });
    });
}

function renderSharedFiles(): void {
    const container = $("shared-content");
    const badge = $("shared-count");
    if (badge) badge.textContent = sharedFiles.length.toString();
    if (!container) return;

    if (sharedFiles.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-icon">📭</div>
                <p>هنوز فایلی برای ارسال نیست</p>
            </div>
        `;
        return;
    }

    container.innerHTML = sharedFiles.map((file) => `
        <div class="file-item" data-id="${file.id}" data-path="${escapeHtml(file.path)}">
            <div class="file-icon">📤</div>
            <div class="file-info">
                <div class="file-name" title="${escapeHtml(file.name)}">${escapeHtml(file.name)}</div>
                <div class="file-meta">${formatBytes(file.size)}</div>
            </div>
            <button class="file-delete" data-id="${file.id}" title="حذف از لیست">✕</button>
        </div>
    `).join("");

    container.querySelectorAll<HTMLElement>(".file-item").forEach((item) => {
        item.addEventListener("contextmenu", (e) => {
            e.preventDefault();
            const path = item.dataset.path;
            const id = item.dataset.id;
            if (!path || !id) return;

            showContextMenu(e.clientX, e.clientY, [
                { id: "open", icon: "📂", label: "باز کردن فایل", action: () => openPath(path) },
                { id: "show", icon: "📁", label: "نمایش در پوشه", action: () => revealPath(path) },
                {
                    id: "copy", icon: "📋", label: "کپی مسیر",
                    action: () => {
                        navigator.clipboard.writeText(path);
                        toast.success("📋 کپی شد", "مسیر فایل کپی شد", false);
                    },
                },
                {
                    id: "delete", icon: "🗑", label: "حذف از لیست",
                    action: () => deleteShared(id),
                    danger: true,
                },
            ]);
        });
    });

    container.querySelectorAll<HTMLElement>(".file-delete").forEach((btn) => {
        btn.addEventListener("click", (e) => {
            e.stopPropagation();
            const id = btn.dataset.id;
            if (id) deleteShared(id);
        });
    });
}

// === Delete Functions ===
async function deleteReceived(path: string): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/files/delete`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ path }),
        });

        if (!res.ok) throw new Error("خطا در حذف");

        const idx = receivedFiles.findIndex(f => f.path === path);
        if (idx !== -1) receivedFiles.splice(idx, 1);
        renderReceivedFiles();

        toast.success("🗑 حذف شد", "فایل از لیست حذف شد", false);
    } catch (e) {
        console.error("خطا در حذف:", e);
        toast.error("خطا", "نتونستم حذف کنم", false);
    }
}

async function deleteShared(id: string): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/share/${id}`, {
            method: "DELETE",
            headers: authHeaders(),
        });

        if (!res.ok) throw new Error("خطا در حذف");

        await loadSharedFiles();
        toast.success("🗑 حذف شد", "فایل از لیست حذف شد", false);
    } catch (e) {
        console.error("خطا در حذف:", e);
        toast.error("خطا", "نتونستم حذف کنم", false);
    }
}

async function clearAllReceived(): Promise<void> {
    if (receivedFiles.length === 0) {
        toast.info("لیست خالیه", "چیزی برای حذف نیست", false);
        return;
    }

    const confirmed = confirm(`آیا مطمئنی می‌خوای همه‌ی ${receivedFiles.length} فایل دریافتی رو از لیست پاک کنی؟\n\n(فایل‌ها از روی دیسک پاک نمیشن)`);
    if (!confirmed) return;

    try {
        const res = await fetch(`${SERVER_URL}/files/clear`, {
            method: "POST",
        });

        if (!res.ok) throw new Error("خطا در پاک کردن");

        const data = await res.json();
        receivedFiles.length = 0;
        renderReceivedFiles();

        toast.success("🗑 پاک شد", `${data.removed} فایل از لیست حذف شد`, false);
    } catch (e) {
        console.error("خطا:", e);
        toast.error("خطا", "نتونستم پاک کنم", false);
    }
}

async function clearAllShared(): Promise<void> {
    if (sharedFiles.length === 0) {
        toast.info("لیست خالیه", "چیزی برای حذف نیست", false);
        return;
    }

    const confirmed = confirm(`آیا مطمئنی می‌خوای همه‌ی ${sharedFiles.length} فایل اشتراکی رو پاک کنی؟`);
    if (!confirmed) return;

    try {
        const res = await fetch(`${SERVER_URL}/share/clear`, {
            method: "POST",
            headers: authHeaders(),
        });

        if (!res.ok) throw new Error("خطا در پاک کردن");

        const data = await res.json();
        sharedFiles.length = 0;
        renderSharedFiles();

        toast.success("🗑 پاک شد", `${data.removed} فایل از لیست حذف شد`, false);
    } catch (e) {
        console.error("خطا:", e);
        toast.error("خطا", "نتونستم پاک کنم", false);
    }
}

function updateStatus(): void {
    const badge = $("status-badge");
    const text = $("status-text");
    if (state.activeUpload) return;
    if (state.serverOnline) {
        badge?.classList.add("online");
        if (text) text.textContent = "آماده‌ی دریافت";
    } else {
        badge?.classList.remove("online");
        if (text) text.textContent = "در حال راه‌اندازی...";
    }
}

async function copyToClipboard(text: string, button: HTMLElement): Promise<void> {
    try {
        await navigator.clipboard.writeText(text);
        button.classList.add("copied");
        setTimeout(() => button.classList.remove("copied"), 1500);
    } catch (e) {}
}

// === Open / Reveal Path ===
async function openPath(path: string): Promise<void> {
    try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("open_file", { path });
    } catch (e) {
        console.error("خطا در باز کردن:", e);
        toast.error("خطا", "نتونستم فایل رو باز کنم", false);
    }
}

async function revealPath(path: string): Promise<void> {
    try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("reveal_file", { path });
    } catch (e) {
        console.error("خطا در reveal:", e);
        toast.error("خطا", "نتونستم پوشه رو باز کنم", false);
    }
}

// === Change Save Path ===
async function handleChangePath(): Promise<void> {
    try {
        const { open } = await import("@tauri-apps/plugin-dialog");
        const selected = await open({
            directory: true,
            multiple: false,
            title: "انتخاب پوشه‌ی ذخیره",
            defaultPath: state.savePath,
        });

        if (!selected || typeof selected !== "string") return;

        const res = await fetch(`${SERVER_URL}/config/save-path`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ path: selected }),
        });

        if (!res.ok) {
            const err = await res.text();
            throw new Error(err);
        }

        state.savePath = selected;
        setText("save-path", selected);

        toast.success("📂 مسیر تغییر کرد", "مسیر ذخیره‌ی جدید اعمال شد");
    } catch (e) {
        console.error("خطا در تغییر مسیر:", e);
        toast.error("خطا", "نتونستم مسیر رو تغییر بدم", false);
    }
}

// === Reset Save Path ===
async function handleResetPath(): Promise<void> {
    try {
        const res = await fetch(`${SERVER_URL}/config/reset-path`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
        });

        if (!res.ok) {
            const err = await res.text();
            throw new Error(err);
        }

        const data: ResetPathResponse = await res.json();
        state.savePath = data.save_path;
        setText("save-path", data.save_path);

        toast.success("🔄 مسیر ریست شد", "مسیر به حالت پیش‌فرض برگشت");
    } catch (e) {
        console.error("خطا در ریست مسیر:", e);
        toast.error("خطا", "نتونستم مسیر رو ریست کنم", false);
    }
}

// === Share Handler ===
async function addSharedFiles(paths: string[]): Promise<void> {
    if (!state.token) {
        toast.error("خطا", "توکن لود نشده", false);
        return;
    }

    let addedCount = 0;
    for (const path of paths) {
        try {
            const res = await fetch(`${SERVER_URL}/share`, {
                method: "POST",
                headers: authHeaders(),
                body: JSON.stringify({ path }),
            });
            if (res.ok) {
                addedCount++;
            } else {
                const err = await res.text();
                console.error("خطا در share:", err);
            }
        } catch (e) {
            console.error("خطا:", e);
        }
    }

    await loadSharedFiles();

    if (addedCount > 0) {
        playOutgoing();
        toast.success(
            "📤 فایل اضافه شد",
            addedCount === 1
                ? "۱ فایل آماده‌ی ارسال به گوشی"
                : `${addedCount} فایل آماده‌ی ارسال به گوشی`
        );
    }
}

async function handleShareClick(): Promise<void> {
    try {
        const { open } = await import("@tauri-apps/plugin-dialog");
        const selected = await open({
            multiple: true,
            title: "انتخاب فایل برای ارسال به گوشی",
        });

        if (!selected) return;

        const paths = Array.isArray(selected) ? selected : [selected];
        await addSharedFiles(paths);
    } catch (e) {
        console.error("خطا در باز کردن dialog:", e);
    }
}

function setupHandlers(): void {
    $("copy-pair-btn")?.addEventListener("click", (e) => {
        const btn = e.currentTarget as HTMLElement;
        copyToClipboard(state.pairCode.replace(/\s/g, ""), btn);
    });

    document.querySelectorAll<HTMLElement>(".copy-btn.small[data-copy]").forEach((btn) => {
        btn.addEventListener("click", () => {
            if (btn.dataset.copy) {
                copyToClipboard(state.lanIp, btn);
            }
        });
    });

    $("change-path-btn")?.addEventListener("click", handleChangePath);
    $("reset-path-btn")?.addEventListener("click", handleResetPath);

    $("share-btn")?.addEventListener("click", handleShareClick);

    // دکمه‌های پاک کردن همه
    $("clear-incoming-btn")?.addEventListener("click", clearAllReceived);
    $("clear-shared-btn")?.addEventListener("click", clearAllShared);
}

// === Drag & Drop Setup ===
async function setupDragDrop(): Promise<void> {
    try {
        await enableDragDrop({
            onFiles: async (paths) => {
                console.log("📥 فایل‌های drop شده:", paths);
                await addSharedFiles(paths);
            },
        });
        console.log("✅ Drag & Drop فعال شد");
    } catch (e) {
        console.error("خطا در راه‌اندازی Drag & Drop:", e);
    }
}

// === Init ===
document.addEventListener("DOMContentLoaded", async () => {
    setupHandlers();
    renderReceivedFiles();
    renderSharedFiles();

    await loadToken();
    await loadAppInfo();
    await loadConfig();
    await loadQrCode();
    await pollFiles();

    await setupDragDrop();

    startPolling();

    setTimeout(() => {
        toast.success("QuickDrop آماده‌ست", "فایل رو بکش رو پنجره یا دکمه‌ی + رو بزن");
    }, 1000);
});