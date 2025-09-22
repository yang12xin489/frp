import {call} from './_invoke'

export const startFrpc = (exePath: string, cfgPath: string) => call<number>('start_frpc', {exePath, cfgPath})
export const stopFrpc = () => call<void>('stop_frpc')
export const frpcStatus = () => call<boolean>('frpc_status')
export const exportToml = () => call<string>('export_toml')
export const exportTomlToFile = () => call<string>('export_toml_to_file')