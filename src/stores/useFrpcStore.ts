import {defineStore} from 'pinia'
import {listen, type UnlistenFn} from '@tauri-apps/api/event'
import {frpcStatus} from '@/api/frpc'

export const useFrpcStore = defineStore('frpc', {
    state: () => ({
        running: false,
        logs: [] as string[],
        _listening: false,
        _un: [] as UnlistenFn[],
    }),
    actions: {
        push(line: string, isErr = false) {
            let txt = isErr ? `[ERR] ${line}` : line
            txt = txt.replace(/\x1B\[[0-9;]*m/g, '')
            this.logs.push(txt.endsWith('\n') ? txt : txt + '\n')
        },
        clear() {
            this.logs.length = 0
        },
        async attachListeners() {
            if (this._listening) return
            const un1 = await listen<string>('frpc://stdout', (e) => this.push(e.payload))
            const un2 = await listen<string>('frpc://stderr', (e) => this.push(e.payload, true))
            const un3 = await listen<{ code: number | null }>('frpc://close', (e) => {
                this.push(`\n[frpc] 退出 code=${e.payload.code} signal=null\n`)
                this.running = false
            })
            this._un = [un1, un2, un3]
            this._listening = true
        },
        detachListeners() {
            this._un.forEach((u) => {
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