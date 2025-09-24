import {call} from './_invoke'
import type {FrpcConfig} from '@/domain/frpc'
import type {Proxy} from '@/domain/types'

export const loadConfig = () => call<FrpcConfig>('load_config')
export const saveServer = (cfg: FrpcConfig) => call<void>('save_server', {partial: cfg})
export const loadProxies = () => call<Proxy[]>('load_proxies')
export const saveProxy = (proxy: Proxy) => call<void>('save_proxy', {proxy})
export const removeProxy = (name: string) => call<boolean>('remove_proxy', {name})
export const setSetting = (key: string, value: unknown) => call<boolean>('set_setting', {key, value})
export const getSetting = <T = unknown>(key: string) => call<T | null>('get_setting', {key})