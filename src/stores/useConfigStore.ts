import {defineStore} from 'pinia'
import {loadConfig, saveServer} from '@/api/config'
import type {FrpcConfig} from '@/domain/frpc'

export const useConfigStore = defineStore('config', {
    state: () => ({cfg: null as FrpcConfig | null, loading: false}),
    actions: {
        async fetch() {
            this.loading = true;
            try {
                this.cfg = await loadConfig()
            } finally {
                this.loading = false
            }
        },
        async save(cfg: FrpcConfig) {
            await saveServer(cfg);
            this.cfg = cfg
        },
    },
})