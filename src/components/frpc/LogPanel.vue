<template>
  <n-card size="small" title="frpc 日志" style="height: 280px">
    <n-scrollbar style="height: 220px" ref="scrollRef">
      <pre class="log">{{ logs.join('') }}</pre>
    </n-scrollbar>
  </n-card>
</template>
<script setup lang="ts">
import {nextTick, ref, watch} from 'vue'

const props = defineProps<{ logs: string[] }>()
const scrollRef = ref<any>(null)
watch(() => props.logs.length, async () => {
  await nextTick();
  const el = scrollRef.value?.$el?.querySelector?.('.n-scrollbar-content');
  if (el) el.scrollTop = el.scrollHeight
})
</script>
<style scoped>
.log {
  white-space: pre-wrap;
  word-break: break-word;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  font-size: 12px;
  margin: 0;
}
</style>