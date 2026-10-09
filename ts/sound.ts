// QuickDrop - Sound Effects (Web Audio API)

let audioContext: AudioContext | null = null;

/// گرفتن (یا ساختن) AudioContext
function getContext(): AudioContext | null {
    try {
        if (!audioContext) {
            audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
        }
        // اگه suspend بود، resume کن (مرورگرها بعد از تعامل کاربر اجازه می‌دن)
        if (audioContext.state === 'suspended') {
            audioContext.resume();
        }
        return audioContext;
    } catch (e) {
        console.warn('AudioContext پشتیبانی نمیشه:', e);
        return null;
    }
}

/// پخش یه بوق ساده
/// 
/// @param frequency - فرکانس (هرتز)
/// @param duration - مدت (ثانیه)
/// @param type - نوع موج
/// @param volume - حجم (0.0 تا 1.0)
function playTone(
    frequency: number,
    duration: number,
    type: OscillatorType = 'sine',
    volume: number = 0.15,
): void {
    const ctx = getContext();
    if (!ctx) return;

    const oscillator = ctx.createOscillator();
    const gainNode = ctx.createGain();

    oscillator.type = type;
    oscillator.frequency.value = frequency;

    // Envelope: شروع سریع، پایان نرم
    gainNode.gain.setValueAtTime(0, ctx.currentTime);
    gainNode.gain.linearRampToValueAtTime(volume, ctx.currentTime + 0.01);
    gainNode.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + duration);

    oscillator.connect(gainNode);
    gainNode.connect(ctx.destination);

    oscillator.start(ctx.currentTime);
    oscillator.stop(ctx.currentTime + duration);
}

/// 🎵 صدای موفقیت — دو تا نت بالا
export function playSuccess(): void {
    playTone(523.25, 0.15, 'sine', 0.15);  // C5
    setTimeout(() => playTone(783.99, 0.25, 'sine', 0.15), 100);  // G5
}

/// ❌ صدای خطا — یه نت پایین
export function playError(): void {
    playTone(220, 0.35, 'sawtooth', 0.12);  // A3
}

/// ℹ️ صدای اطلاع — یه نت ملایم
export function playInfo(): void {
    playTone(659.25, 0.15, 'sine', 0.1);  // E5
}

/// ⚠️ صدای هشدار — دو تا نت کوتاه
export function playWarning(): void {
    playTone(440, 0.12, 'triangle', 0.12);  // A4
    setTimeout(() => playTone(440, 0.12, 'triangle', 0.12), 150);
}

/// 📥 صدای دریافت فایل — نت‌های نزولی
export function playIncoming(): void {
    playTone(880, 0.1, 'sine', 0.12);  // A5
    setTimeout(() => playTone(659.25, 0.1, 'sine', 0.12), 80);  // E5
    setTimeout(() => playTone(523.25, 0.2, 'sine', 0.12), 160);  // C5
}

/// 📤 صدای ارسال فایل — نت‌های صعودی
export function playOutgoing(): void {
    playTone(523.25, 0.1, 'sine', 0.12);  // C5
    setTimeout(() => playTone(659.25, 0.1, 'sine', 0.12), 80);  // E5
    setTimeout(() => playTone(880, 0.2, 'sine', 0.12), 160);  // A5
}

/// 🎊 صدای جشن — آکورد
export function playCelebration(): void {
    playTone(523.25, 0.2, 'sine', 0.12);  // C5
    setTimeout(() => playTone(659.25, 0.2, 'sine', 0.12), 80);  // E5
    setTimeout(() => playTone(783.99, 0.2, 'sine', 0.12), 160);  // G5
    setTimeout(() => playTone(1046.50, 0.4, 'sine', 0.12), 240);  // C6
}