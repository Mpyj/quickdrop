// QuickDrop - Confetti Animation
// 
// یه انیمیشن جشن ساده با Canvas API
// بدون dependency خارجی

interface Particle {
    x: number;
    y: number;
    vx: number;
    vy: number;
    size: number;
    rotation: number;
    rotationSpeed: number;
    color: string;
    shape: 'rect' | 'circle';
    opacity: number;
}

const COLORS = [
    '#667eea',  // primary
    '#764ba2',  // secondary
    '#10b981',  // success
    '#f59e0b',  // warning
    '#ef4444',  // error
    '#a5b4fc',  // light primary
    '#ffffff',
];

let canvas: HTMLCanvasElement | null = null;
let ctx: CanvasRenderingContext2D | null = null;
let particles: Particle[] = [];
let animationFrame: number | null = null;
let isRunning = false;

/// راه‌اندازی canvas
function setupCanvas(): boolean {
    if (canvas && ctx) return true;

    canvas = document.getElementById('confetti-canvas') as HTMLCanvasElement;
    if (!canvas) {
        console.warn('confetti-canvas not found');
        return false;
    }

    ctx = canvas.getContext('2d');
    if (!ctx) {
        console.warn('Canvas 2D not supported');
        return false;
    }

    // اندازه‌ی canvas = اندازه‌ی پنجره
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    return true;
}

function resizeCanvas(): void {
    if (!canvas) return;
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
}

/// ساخت یه ذره
function createParticle(x: number, y: number): Particle {
    const angle = Math.random() * Math.PI * 2;
    const speed = 2 + Math.random() * 6;

    return {
        x,
        y,
        vx: Math.cos(angle) * speed,
        vy: Math.sin(angle) * speed - 8,  // شروع به بالا
        size: 6 + Math.random() * 8,
        rotation: Math.random() * Math.PI * 2,
        rotationSpeed: (Math.random() - 0.5) * 0.2,
        color: COLORS[Math.floor(Math.random() * COLORS.length)],
        shape: Math.random() > 0.5 ? 'rect' : 'circle',
        opacity: 1,
    };
}

/// رسم یه ذره
function drawParticle(p: Particle): void {
    if (!ctx) return;

    ctx.save();
    ctx.globalAlpha = p.opacity;
    ctx.translate(p.x, p.y);
    ctx.rotate(p.rotation);
    ctx.fillStyle = p.color;

    if (p.shape === 'rect') {
        ctx.fillRect(-p.size / 2, -p.size / 4, p.size, p.size / 2);
    } else {
        ctx.beginPath();
        ctx.arc(0, 0, p.size / 2, 0, Math.PI * 2);
        ctx.fill();
    }

    ctx.restore();
}

/// آپدیت یه ذره (فیزیک)
function updateParticle(p: Particle): void {
    p.vy += 0.25;  // جاذبه
    p.vx *= 0.99;  // اصطکاک هوا
    p.x += p.vx;
    p.y += p.vy;
    p.rotation += p.rotationSpeed;

    // محو شدن تدریجی
    if (p.y > window.innerHeight * 0.6) {
        p.opacity -= 0.01;
    }
}

/// حلقه‌ی انیمیشن
function animate(): void {
    if (!ctx || !canvas) return;

    // پاک کردن canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // آپدیت + رسم
    particles = particles.filter((p) => {
        updateParticle(p);
        if (p.opacity <= 0 || p.y > window.innerHeight + 50) {
            return false;
        }
        drawParticle(p);
        return true;
    });

    // ادامه یا توقف
    if (particles.length > 0) {
        animationFrame = requestAnimationFrame(animate);
    } else {
        isRunning = false;
        if (canvas) {
            ctx.clearRect(0, 0, canvas.width, canvas.height);
        }
    }
}

/// شلیک Confetti از یه نقطه
/// 
/// @param originX - مختصات افقی (پیش‌فرض: وسط)
/// @param originY - مختصات عمودی (پیش‌فرض: وسط)
/// @param count - تعداد ذرات (پیش‌فرض: ۸۰)
export function fireConfetti(
    originX?: number,
    originY?: number,
    count: number = 80,
): void {
    if (!setupCanvas()) return;

    const x = originX ?? window.innerWidth / 2;
    const y = originY ?? window.innerHeight / 2;

    // اضافه کردن ذرات
    for (let i = 0; i < count; i++) {
        particles.push(createParticle(x, y));
    }

    // شروع انیمیشن اگه در حال اجرا نیست
    if (!isRunning) {
        isRunning = true;
        animationFrame = requestAnimationFrame(animate);
    }
}

/// انیمیشن جشن از دو طرف پایین (مثل جشن)
export function celebrate(): void {
    if (!setupCanvas()) return;

    const w = window.innerWidth;
    const h = window.innerHeight;

    // از پایین چپ
    fireConfetti(w * 0.15, h * 0.9, 60);

    // از پایین راست (با تاخیر کوچیک)
    setTimeout(() => {
        fireConfetti(w * 0.85, h * 0.9, 60);
    }, 150);

    // از وسط بالا (با تاخیر بیشتر)
    setTimeout(() => {
        fireConfetti(w * 0.5, h * 0.3, 80);
    }, 300);
}

/// توقف فوری
export function stopConfetti(): void {
    particles = [];
    if (animationFrame !== null) {
        cancelAnimationFrame(animationFrame);
        animationFrame = null;
    }
    isRunning = false;
    if (ctx && canvas) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
    }
}