<template>
  <n-result status="success" title="FRP 桌面客户端" description="请选择左侧菜单开始配置或管理版本"/>
</template>
<script setup lang="ts">
import {listen} from '@tauri-apps/api/event';

function fmtBps(bps: number) {
  if (bps < 1024) return `${bps.toFixed(0)} B/s`;
  const kb = bps / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB/s`;
  const mb = kb / 1024;
  if (mb < 1024) return `${mb.toFixed(2)} MB/s`;
  return `${(mb / 1024).toFixed(2)} GB/s`;
}

onMounted(async () => {
  await listen('frp:traffic', (e) => {
    const data = e.payload as any;
    const list = Array.isArray(data) ? data : [data];

    // 遍历并输出
    for (const {proxy, up_bps, down_bps, up_total, down_total} of list) {
      console.log(
          `[${proxy}]  ↑ ${fmtBps(up_bps)}  ↓ ${fmtBps(down_bps)}  ` +
          `(tot ↑ ${up_total} B, tot ↓ ${down_total} B)`
      );
    }
  });
});
</script>