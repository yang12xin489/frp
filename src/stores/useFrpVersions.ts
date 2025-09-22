import {defineStore} from 'pinia'
import {listen, type UnlistenFn} from '@tauri-apps/api/event'
import {activateVersion, deactivateVersion, deleteVersion, downloadVersion, getVersions} from '@/api/frpVersions'
import {FrpVersion} from '@/domain/frpVersion'
import {createDiscreteApi} from 'naive-ui'

const {message} = createDiscreteApi(['message'])
const DOWNLOAD_PROGRESS = "frp_download_progress";
const ACTIVATING_STATUS = "frp_activating_status";

export const useFrpVersions = defineStore('frpVersions', {
    state: () => ({
        frpVersions: new Map<string, FrpVersion>,
        progresses: new Map<string, number>,
        activating: false,
        activatingName: '',
        _uns: [] as UnlistenFn[],
    }),
    getters: {
        getProgress: (state) => (name: string) => state.progresses.get(name) ?? 0,
        isDownloading: (state) => (name: string) => state.progresses.has(name),
    },
    actions: {
        async fetchVersions() {
            (await getVersions()).forEach((version: FrpVersion) => this.frpVersions.set(version.name, version))
        },

        setProgress(name: string, p: number) {
            this.progresses.set(name, p)
        },

        clearProgress(name: string) {
            this.progresses.delete(name)
        },

        async bindAll() {
            if (this._uns.length > 0) return
            const unbinders = await Promise.all([
                await listen<{ name: string; progress: number }>(DOWNLOAD_PROGRESS, (e) => {
                    this.setProgress(e.payload.name, e.payload.progress)
                    if (e.payload.progress === 100) {
                        setTimeout(() => {
                            this.clearProgress(e.payload.name)
                            const v = this.frpVersions.get(e.payload.name)!
                            this.frpVersions.set(e.payload.name, {
                                ...v,
                                exist: true,
                            })
                            message.success('下载完成')
                        }, 500)
                    }
                }),
                await listen<{ status: boolean }>(ACTIVATING_STATUS, (e) => {
                    console.log(this.activating, this.activatingName, e)
                    this.frpVersions.forEach((v, k) => {
                        this.frpVersions.set(k, {...v, active: v.name === this.activatingName})
                    })
                    this.activating = e.payload.status
                    this.activatingName = ''
                })
            ])
            this._uns.push(...unbinders)
        },

        unbindAll() {
            this._uns.splice(0).forEach((un) => {
                try {
                    un();
                } catch {
                }
            });
        },

        async download(v: FrpVersion) {
            await downloadVersion(v.name, v.url)
        },

        async activate(v: FrpVersion) {
            this.activating = true
            this.activatingName = v.name
            await activateVersion(v.name)
        },

        async deactivate(v: FrpVersion) {
            this.frpVersions.set(v.name, {
                ...v,
                active: false,
            })
            await deactivateVersion(v.name)
        },

        async delete(v: FrpVersion) {
            await deleteVersion(v.name)
            this.frpVersions.set(v.name, {
                ...v,
                exist: false,
                active: false,
            })
        },
    },
})