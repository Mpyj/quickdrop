// QuickDrop - Context Menu (راست‌کلیک)

export interface MenuItem {
    id: string;
    icon: string;
    label: string;
    action: () => void | Promise<void>;
    danger?: boolean;
    separator?: boolean;
}

let currentMenu: HTMLElement | null = null;
let outsideClickHandler: ((e: MouseEvent) => void) | null = null;

/// نمایش context menu
export function showContextMenu(
    x: number,
    y: number,
    items: MenuItem[],
): void {
    hideContextMenu();

    const menu = document.getElementById('context-menu');
    if (!menu) {
        console.warn('context-menu not found in DOM');
        return;
    }

    // ساخت محتوا
    menu.innerHTML = items
        .map((item) => {
            if (item.separator) {
                return `<div class="context-menu-separator"></div>`;
            }
            return `
                <button class="context-menu-item ${item.danger ? 'danger' : ''}" data-id="${item.id}">
                    <span class="context-menu-icon">${item.icon}</span>
                    <span>${item.label}</span>
                </button>
            `;
        })
        .join('');

    // موقعیت
    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;
    menu.classList.add('active');

    // چک کن از صفحه بیرون نزنه
    requestAnimationFrame(() => {
        const rect = menu.getBoundingClientRect();
        if (rect.right > window.innerWidth) {
            menu.style.left = `${window.innerWidth - rect.width - 10}px`;
        }
        if (rect.bottom > window.innerHeight) {
            menu.style.top = `${window.innerHeight - rect.height - 10}px`;
        }
    });

    // اتصال eventها
    menu.querySelectorAll<HTMLElement>('.context-menu-item').forEach((btn) => {
        btn.addEventListener('click', async () => {
            const id = btn.dataset.id;
            const item = items.find((i) => i.id === id);
            if (item) {
                hideContextMenu();
                await item.action();
            }
        });
    });

    currentMenu = menu;

    // بستن با کلیک بیرون
    outsideClickHandler = (e: MouseEvent) => {
        if (!menu.contains(e.target as Node)) {
            hideContextMenu();
        }
    };
    setTimeout(() => {
        if (outsideClickHandler) {
            document.addEventListener('click', outsideClickHandler);
        }
    }, 0);

    // بستن با Escape
    document.addEventListener('keydown', handleEsc);
}

function handleEsc(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
        hideContextMenu();
    }
}

/// بستن context menu
export function hideContextMenu(): void {
    const menu = document.getElementById('context-menu');
    if (menu) {
        menu.classList.remove('active');
        menu.innerHTML = '';
    }
    if (outsideClickHandler) {
        document.removeEventListener('click', outsideClickHandler);
        outsideClickHandler = null;
    }
    document.removeEventListener('keydown', handleEsc);
    currentMenu = null;
}