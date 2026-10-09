// QuickDrop - Drag & Drop (با API رسمی Tauri)

import { getCurrentWebview } from "@tauri-apps/api/webview";

export interface DragDropOptions {
    onFiles: (paths: string[]) => Promise<void> | void;
}

/// فعال کردن Drag & Drop با API رسمی Tauri
export async function enableDragDrop(options: DragDropOptions): Promise<() => void> {
    const webview = getCurrentWebview();

    const unlisten = await webview.onDragDropEvent(async (event) => {
        const payload = event.payload;

        if (payload.type === "enter") {
            document.querySelector('.app')?.classList.add('drag-over');
            document.querySelectorAll('.panel').forEach(p => p.classList.add('drag-over'));
        }

        if (payload.type === "over") {
            // موس بالای پنجره
        }

        if (payload.type === "leave") {
            document.querySelector('.app')?.classList.remove('drag-over');
            document.querySelectorAll('.panel').forEach(p => p.classList.remove('drag-over'));
        }

        if (payload.type === "drop") {
            document.querySelector('.app')?.classList.remove('drag-over');
            document.querySelectorAll('.panel').forEach(p => p.classList.remove('drag-over'));

            const paths: string[] = (payload as any).paths || [];
            if (paths.length > 0) {
                await options.onFiles(paths);
            }
        }
    });

    return unlisten;
}