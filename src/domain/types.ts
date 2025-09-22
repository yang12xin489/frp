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
    name: string;
    type: ProxyType
}

export interface HttpSwitch {
    domain: DomainType;
    auth: boolean
}

export interface HttpProxy extends Proxy {
    localIP: string
    localPort: number
    subdomain: string
    customDomains: string[]
    locations: string[]
    httpUser: string
    httpPassword: string
    switch: HttpSwitch
}