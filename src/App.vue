<template>
  <n-message-provider>
    <n-layout vertical>
      <n-layout-header style="height:32px" data-tauri-drag-region></n-layout-header>
      <n-layout has-sider style="height: calc(100vh - 32px)">
        <n-layout-sider width="150">
          <n-menu :options="menuOptions" :value="activeKey" @update:value="go"/>
        </n-layout-sider>
        <n-layout-content style="margin: 0 4px 12px 0;">
          <router-view/>
        </n-layout-content>
      </n-layout>
    </n-layout>
  </n-message-provider>
</template>
<script setup lang="ts">
import {computed, h, ref, watch} from 'vue'
import {RouterLink, useRoute, useRouter} from 'vue-router'
import type {MenuOption} from 'naive-ui'
import {NIcon} from 'naive-ui'
import {Icon} from '@iconify/vue'

const route = useRoute();
const router = useRouter()
const activeKey = ref(route.path);
watch(() => route.path, (p) => (activeKey.value = p))

function makeKey(to: any) {
  const r = router.resolve(to);
  return r.href.replace(/^#/, '')
}

function renderIconByName(name?: string) {
  if (!name) return undefined;
  return () => h(NIcon, null, {default: () => h(Icon, {icon: name})})
}

const menuOptions = computed<MenuOption[]>(() => router.getRoutes().filter((r) => r.meta?.title).map<MenuOption>((r) => {
  const to = {path: r.path}
  return {key: makeKey(to), icon: renderIconByName(r.meta!.icon as string | undefined), label: () => h(RouterLink, {to}, {default: () => (r.meta!.title as string)})}
}))

function go(val: string) {
  router.push(val)
}
</script>