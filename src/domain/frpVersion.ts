export interface FrpVersion {
    id: number
    name: string
    size: string
    version: string
    createdAt: string
    count: number
    url: string
    exist: boolean
    active: boolean
}

export interface ActiveFrp {
    name: string
    archivePath: string
    unpackDir: string
    exePath: string
    activatedAt: string
}