import {call} from './_invoke'

export const startFrpc = () => call<number>('start_frpc')
export const stopFrpc = () => call<void>('stop_frpc')
export const frpcStatus = () => call<boolean>('frpc_status')