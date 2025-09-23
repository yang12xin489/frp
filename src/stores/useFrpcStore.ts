// src/stores/useFrpcStore.ts
import {defineStore} from 'pinia'
import {listen, type UnlistenFn} from '@tauri-apps/api/event'
import {frpcStatus} from '@/api/frpc'

export type LogLevel = 'stdout' | 'stderr' | 'system'

export interface LogEntry {
    id: number;
    ts: number;
    level: LogLevel;
    text: string
}

const MAX_LINES = 5000

export const useFrpcStore = defineStore('frpc', {
    state: () => ({
        running: false,
        entries: [] as LogEntry[],
        _nextId: 1,
        _listening: false,
        _un: [] as UnlistenFn[],
    }),
    getters: {
        asText: (s) => s.entries
            .map(e => `[${new Date(e.ts).toLocaleTimeString()}] ${e.level.toUpperCase()} ${e.text}`)
            .join('\n'),
    },
    actions: {
        pushRaw(line: string, level: LogLevel) {
            const parts = (line ?? '').split(/\r?\n/)
            for (const p of parts) {
                if (!p) continue
                this.entries.push({id: this._nextId++, ts: Date.now(), level, text: p})
                if (this.entries.length > MAX_LINES) this.entries.splice(0, this.entries.length - MAX_LINES)
            }
        },
        clear() {
            this.entries = []
        },
        async attachListeners() {
            if (this._listening) return
            const un1 = await listen<string>('frpc://stdout', e => this.pushRaw(e.payload, 'stdout'))
            const un2 = await listen<string>('frpc://stderr', e => this.pushRaw(e.payload, 'stderr'))
            const un3 = await listen<{ code: number | null }>('frpc://close', e => {
                this.pushRaw(`\n[frpc] 退出 code=${e.payload.code ?? 'null'} signal=null`, 'system')
                this.running = false
            })
            this._un = [un1, un2, un3]
            this._listening = true
        },
        detachListeners() {
            this._un.forEach(u => {
                try {
                    u()
                } catch {
                }
            })
            this._un = []
            this._listening = false
        },
        async hydrate() {
            this.running = await frpcStatus()
            await this.attachListeners()
        },
    },
})