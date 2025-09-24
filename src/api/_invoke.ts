import {invoke} from '@tauri-apps/api/core'

export async function call<T>(cmd: string, payload?: any): Promise<T> {
    try {
        return await invoke<T>(cmd, payload);
    } catch (e: any) {
        const msg = typeof e === 'string' ? e : e?.message ?? String(e);
        throw new Error(msg);
    }
}