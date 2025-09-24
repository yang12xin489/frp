<template>
  <n-space vertical>
    <n-space :size="8">
      <n-button :type="running ? 'error' : 'success'" :loading="busy" @click="startOrStop">
        {{ running ? '停止' : '启动' }}
      </n-button>
      <n-button tertiary @click="clearLogs">清空</n-button>
      <n-button tertiary @click="copyLogs">复制日志</n-button>
    </n-space>
    <LogPanel title="frpc 日志" @clear="clearLogs"/>
  </n-space>
</template>

<script setup lang="ts">
import LogPanel from '@/components/frpc/LogPanel.vue'
import {useFrpcStore} from '@/stores/useFrpcStore'
import {startFrpc, stopFrpc} from '@/api/frpc'
import {useMessage} from 'naive-ui'
import {computed, onMounted, ref} from 'vue'
import {getActiveVersion} from "@/api/frpVersions.ts";
import type {ActiveFrp} from "@/domain/frpVersion.ts";

const frpc = useFrpcStore()
const message = useMessage()

const running = computed(() => frpc.running)
const busy = ref(false)
const ver = ref<ActiveFrp>()

onMounted(async () => {
  await frpc.hydrate()
  ver.value = await getActiveVersion()
})

function clearLogs() {
  frpc.clear()
}

async function copyLogs() {
  try {
    await navigator.clipboard?.writeText(frpc.asText);
    message.success('已复制')
  } catch {
    message.error('复制失败')
  }
}

async function startOrStop() {
  if (busy.value) return
  busy.value = true
  try {
    if (!frpc.running) {
      frpc.clear()
      if (!ver.value?.exePath) {
        message.warning('请先在“版本管理”下载并激活一个 frp 版本');
        return
      }
      await startFrpc()
      frpc.running = true
      message.success('frpc 已启动')
    } else {
      await stopFrpc()
      frpc.running = false
      message.info('frpc 已停止')
    }
  } catch (e) {
    console.error(e)
    message.error('操作失败')
  } finally {
    busy.value = false
  }
}
</script>
