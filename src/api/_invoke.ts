import {invoke} from '@tauri-apps/api/core'

export async function call<T>(cmd: string, payload?: any): Promise<T> {
    try {
        return await invoke<T>(cmd, payload);
    } catch (e: any) {
        const msg = typeof e === 'string' ? e : e?.message ?? String(e);
        throw new Error(msg);
    }
}

export const mapFrpVersion = (raw: any) => ({
    id: raw.id,
    name: raw.name,
    size: raw.size,
    version: raw.version,
    createdAt: raw.created_at,
    count: raw.count,
    url: raw.url,
    exist: raw.exist,
    active: raw.active
})