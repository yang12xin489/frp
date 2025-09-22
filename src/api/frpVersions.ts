import {call, mapFrpVersion} from './_invoke'
import type {FrpVersion, ActiveFrp} from '@/domain/frpVersion'

export async function getVersions(): Promise<FrpVersion[]> {
    return (await call<any[]>('get_versions')).map(mapFrpVersion)
}

export const downloadVersion = (name: string, url: string) => call<void>('download_version', {name, url})
export const deleteVersion = (name: string) => call<void>('delete_version', {name})
export const activateVersion = (name: string) => call<void>('activate_version', {name})
export const deactivateVersion = (name: string) => call<void>('deactivate_version', {name})
export const getActiveVersion = () => call<ActiveFrp>('get_active_version')