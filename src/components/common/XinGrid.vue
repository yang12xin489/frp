<template>
  <div>
    <n-flex justify="space-between" align="center" style="margin-right: 8px">
      <h2>{{ label }}</h2>
      <slot name="header"/>
    </n-flex>
    <n-scrollbar style="height: 81vh;right: -4px">
      <div class="ver-grid" style="margin-right: 12px">
        <slot name="default"/>
      </div>
    </n-scrollbar>
  </div>
</template>
<script setup lang="ts">
defineProps<{ label: string; }>()
</script>
<style scoped>
.ver-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 16px;
}
</style>