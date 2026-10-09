// QuickDrop - Toast Notification System

import { playSuccess, playError, playInfo, playWarning } from './sound';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface ToastOptions {
    title: string;
    message?: string;
    type?: ToastType;
    duration?: number;
    icon?: string;
    sound?: boolean;  // پیش‌فرض true
}

const DEFAULT_DURATION = 3000;

const ICONS: Record<ToastType, string> = {
    success: '✅',
    error: '❌',
    info: 'ℹ️',
    warning: '⚠️',
};

const SOUNDS: Record<ToastType, () => void> = {
    success: playSuccess,
    error: playError,
    info: playInfo,
    warning: playWarning,
};

/// نمایش یه Toast
export function showToast(options: ToastOptions): void {
    const container = document.getElementById('toast-container');
    if (!container) {
        console.warn('Toast container not found');
        return;
    }

    const type = options.type || 'info';
    const duration = options.duration ?? DEFAULT_DURATION;
    const icon = options.icon ?? ICONS[type];
    const playSound = options.sound !== false;

    // پخش صدا
    if (playSound) {
        try {
            SOUNDS[type]();
        } catch (e) {
            // بی‌صدا نادیده بگیر
        }
    }

    // ساخت المان
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.innerHTML = `
        <div class="toast-icon">${icon}</div>
        <div class="toast-body">
            <div class="toast-title">${escapeHtml(options.title)}</div>
            ${options.message ? `<div class="toast-message">${escapeHtml(options.message)}</div>` : ''}
        </div>
        <button class="toast-close" title="بستن">×</button>
        <div class="toast-progress" style="animation-duration: ${duration}ms"></div>
    `;

    container.appendChild(toast);

    // دکمه‌ی بستن
    const closeBtn = toast.querySelector('.toast-close');
    closeBtn?.addEventListener('click', () => removeToast(toast));

    // حذف خودکار
    const timeoutId = window.setTimeout(() => removeToast(toast), duration);
    (toast as any)._timeoutId = timeoutId;

    // حداکثر ۵ تا Toast همزمان
    const toasts = container.querySelectorAll('.toast');
    if (toasts.length > 5) {
        removeToast(toasts[0] as HTMLElement);
    }
}

/// حذف یه Toast با انیمیشن
function removeToast(toast: HTMLElement): void {
    if ((toast as any)._removing) return;
    (toast as any)._removing = true;

    const timeoutId = (toast as any)._timeoutId;
    if (timeoutId) clearTimeout(timeoutId);

    toast.classList.add('toast-leaving');

    setTimeout(() => {
        toast.remove();
    }, 300);
}

/// Escape کردن HTML
function escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

/// Shortcuts
export const toast = {
    success: (title: string, message?: string, sound?: boolean) =>
        showToast({ title, message, type: 'success', sound }),
    error: (title: string, message?: string, sound?: boolean) =>
        showToast({ title, message, type: 'error', duration: 5000, sound }),
    info: (title: string, message?: string, sound?: boolean) =>
        showToast({ title, message, type: 'info', sound }),
    warning: (title: string, message?: string, sound?: boolean) =>
        showToast({ title, message, type: 'warning', sound }),
};