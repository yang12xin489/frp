<template>
  <xin-grid label="代理设置">
    <template #header>
      <n-button type="success" @click="open">添加</n-button>
    </template>
    <template #default>
      <n-card v-for="proxy in store.proxies" :key="proxy.name" :title="proxy.name" size="small" hoverable>
        <template #footer>
          <n-space justify="end">
            <n-button size="small" @click="onEdit(proxy)">编辑</n-button>
            <n-popconfirm :show-icon="false" positive-text="删除" negative-text="取消" @click="onDelete(proxy)">
              确定删除该代理？
              <template #trigger>
                <n-button type="error" size="small">删除</n-button>
              </template>
            </n-popconfirm>
          </n-space>
        </template>
      </n-card>
    </template>
  </xin-grid>
  <proxy-editor :visible="show" :initial="preset" @callback="handleSaved"/>
</template>
<script setup lang="ts">
import {onMounted, ref} from 'vue'
import {useProxiesStore} from '@/stores/useProxiesStore'
import type {Proxy} from '@/domain/types'
import XinGrid from "@/components/versions/XinGrid.vue";
import ProxyEditor from "@/components/proxies/ProxyEditor.vue";

const store = useProxiesStore()
const show = ref(false)
const preset = ref<Partial<Proxy> | null>(null)

function open() {
  preset.value = null;
  show.value = true
}

function onEdit(p: Proxy) {
  preset.value = {...p};
  show.value = true
}

async function onDelete(p: Proxy) {
  await store.remove(p.name, p.type as any);
  await refresh()
}

async function refresh() {
  await store.fetch()
}

function handleSaved(proxy: Proxy) {
  show.value = false
}

onMounted(() => {
  refresh()
})
</script>