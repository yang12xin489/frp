export enum ProxyType {
    TCP = 'tcp',
    UDP = 'udp',
    HTTP = 'http',
    HTTPS = 'https',
    STCP = 'stcp',
    SUDP = 'sudp',
    XTCP = 'xtcp',
}


export enum AuthType { TOKEN = 'token', OIDC = 'oidc' }

export enum DomainType { SUB = 'sub', CUSTOM = 'custom' }

export interface Proxy {
    id: string;
    name: string;
    type: ProxyType
    enable: boolean
    localIP: string
    localPort: number
}

export interface HttpSwitch {
    domain: DomainType;
    auth: boolean
}

export interface HttpProxy extends Proxy {
    subdomain: string
    customDomains: string[]
    locations: string[]
    httpUser: string
    httpPassword: string
    switch: HttpSwitch
}