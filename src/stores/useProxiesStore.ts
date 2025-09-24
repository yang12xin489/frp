import {defineStore} from 'pinia'
import {loadProxies, removeProxy, saveProxy} from '@/api/config'
import type {Proxy} from '@/domain/types'

export const useProxiesStore = defineStore('proxies', {
    state: () => ({
        proxies: [] as Proxy[]
    }),
    actions: {
        async fetch() {
            this.proxies = await loadProxies()
        },
        async addOrUpdate(p: Proxy) {
            await saveProxy(p);
            await this.fetch()
        },
        async remove(name: string) {
            await removeProxy(name);
            await this.fetch()
        },
    },
})