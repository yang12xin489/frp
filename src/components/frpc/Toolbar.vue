<template>
  <n-space :size="8">
    <n-button :type="running ? 'error' : 'success'" :loading="busy" @click="$emit('toggle')">{{ running ? '停止' : '启动' }}</n-button>
    <n-button tertiary @click="$emit('clear')">清空</n-button>
    <n-button tertiary @click="$emit('copy')">复制日志</n-button>
  </n-space>
</template>
<script setup lang="ts">
defineProps<{ running: boolean; busy?: boolean }>()
</script>