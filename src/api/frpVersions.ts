import {call} from './_invoke'
import {ActiveFrp, FrpVersion} from '@/domain/frpVersion'

export const getVersions = () => call<FrpVersion[]>('get_versions')
export const downloadVersion = (name: string, url: string) => call<void>('download_version', {name, url})
export const deleteVersion = (name: string) => call<void>('delete_version', {name})
export const activateVersion = (name: string) => call<void>('activate_version', {name})
export const deactivateVersion = (name: string) => call<void>('deactivate_version', {name})
export const getActiveVersion = () => call<ActiveFrp>('get_active_version')