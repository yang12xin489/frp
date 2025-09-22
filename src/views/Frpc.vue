<template>
  <n-space vertical>
    <Toolbar :running="running" :busy="busy" @toggle="startOrStop" @clear="clearLogs" @copy="copyLogs"/>
    <LogPanel :logs="logs"/>
  </n-space>
</template>
<script setup lang="ts">
import Toolbar from '@/components/frpc/Toolbar.vue'
import LogPanel from '@/components/frpc/LogPanel.vue'
import {useFrpcStore} from '@/stores/useFrpcStore'
import {exportTomlToFile, startFrpc, stopFrpc} from '@/api/frpc'
import {computed, onMounted, ref} from 'vue'
import {useMessage} from 'naive-ui'
import {getActiveVersion} from "@/api/frpVersions.ts";
import {ActiveFrp} from "@/domain/frpVersion.ts";

const frpc = useFrpcStore();
const message = useMessage()
const busy = ref(false)
const logs = computed(() => frpc.logs)
const running = computed({get: () => frpc.running, set: (v) => (frpc.running = v)})
const activeVersion = ref<ActiveFrp>()

onMounted(async () => {
  await frpc.hydrate();
  activeVersion.value = await getActiveVersion()
})

function clearLogs() {
  frpc.clear()
}

async function copyLogs() {
  try {
    await navigator.clipboard?.writeText(frpc.logs.join(''));
    message.success('已复制')
  } catch {
    message.error('复制失败')
  }
}

async function startOrStop() {
  if (busy.value) return
  busy.value = true
  try {
    if (!running.value) {
      frpc.clear()
      const cfgPath = await exportTomlToFile()
      if (!activeVersion?.value?.exePath) {
        message.warning('请先在“版本管理”下载并激活一个 frp 版本');
        return
      }
      await startFrpc(activeVersion?.value?.exePath, cfgPath)
      running.value = true
      message.success('frpc 已启动')
    } else {
      await stopFrpc();
      running.value = false;
      message.info('frpc 已停止')
    }
  } catch (e) {
    console.error(e);
    message.error('操作失败')
  } finally {
    busy.value = false
  }
}
</script>