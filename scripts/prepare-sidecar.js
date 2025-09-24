import {existsSync, copyFileSync, mkdirSync} from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const watchdog = 'frpc-watchdog';
const triple = process.env.TAURI_ENV_TARGET_TRIPLE;
const isDebug = process.env.TAURI_ENV_DEBUG === 'true';
const profile = isDebug ? 'debug' : 'release';
const ext = process.platform === 'win32' ? '.exe' : '';

if (!triple) {
    console.error('TAURI_ENV_TARGET_TRIPLE not set.');
    process.exit(1);
}

const srcDir = path.join('target', profile);
const src = path.join(srcDir, `${watchdog}${ext}`);
console.log('frp-watchdog 路径', src)
if (!existsSync(src)) {
    console.error('Sidecar not found:', src);
    process.exit(1);
}
const dstDir = path.join("target");
mkdirSync(dstDir, {recursive: true});
const dst = path.join(dstDir, `${watchdog}-${triple}${ext}`);
console.log('拷贝至', dstDir)
console.log('拷贝为', dst)
copyFileSync(src, dst);