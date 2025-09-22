import type {AuthType} from './types'
import type {HttpProxy} from './types'

export interface FrpcConfig {
    serverAddr: string
    serverPort: number
    auth: { method: AuthType; token: string }
    webServer: { addr: string; port: number; user: string; password: string }
    proxies: HttpProxy[] | any[]
    switch: { auth: boolean; webServer: boolean }
}

export const defaultConfig: FrpcConfig = {
    serverAddr: '127.0.0.1',
    serverPort: 7000,
    auth: {method: 'token' as any, token: ''},
    webServer: {addr: '127.0.0.1', port: 7400, user: '', password: ''},
    proxies: [],
    switch: {auth: false, webServer: false},
}