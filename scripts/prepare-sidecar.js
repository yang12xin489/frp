import {existsSync, copyFileSync, mkdirSync} from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const triple = process.env.TAURI_ENV_TARGET_TRIPLE; // e.g. x86_64-pc-windows-msvc
const isDebug = process.env.TAURI_ENV_DEBUG === 'true';
const profile = isDebug ? 'debug' : 'release';
const ext = process.platform === 'win32' ? '.exe' : '';

if (!triple) {
    console.error('TAURI_ENV_TARGET_TRIPLE not set.');
    process.exit(1);
}

const srcDir = path.join('src-tauri', 'target', triple, profile);
const src = path.join(srcDir, `frp-client-watchdog-${triple}${ext}`);

if (!existsSync(src)) {
    console.error('Sidecar not found:', src);
    process.exit(1);
}

const dstDir = path.join('src-tauri', "target");
mkdirSync(dstDir, {recursive: true});
const dst = path.join(dstDir, `frp-client-watchdog-${triple}${ext}`);
copyFileSync(src, dst);