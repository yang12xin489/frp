<template>
  <xin-grid label="代理设置">
    <template #header>
      <n-button type="success" @click="open">添加</n-button>
    </template>
    <template #default>
      <n-card hoverable embedded v-for="proxy in store.proxies" :key="proxy.id">
        <template #header>
          <n-flex justify="space-between" align="center">
            <n-tag :bordered="false" type="success">{{ proxy.type }}</n-tag>
            <n-switch :value="proxy.enable" @update:value="enableSwitch(proxy)"/>
          </n-flex>
        </template>
        <n-flex justify="space-between" :vertical="true">
          <span style="font-size: 15px;font-weight: bold">{{ proxy.name }}</span>
          <span style="font-size: 12px;">{{ proxy.localIP }}:{{ proxy.localPort }}</span>
        </n-flex>
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
  <http-proxy-modal v-model:visible="show" :initial="preset" @callback="callback"/>
</template>
<script setup lang="ts">
import {nextTick, onMounted, ref} from 'vue'
import {useProxiesStore} from '@/stores/useProxiesStore'
import type {Proxy} from '@/domain/types'
import XinGrid from "@/components/common/XinGrid.vue";
import HttpProxyModal from "@/components/proxies/HttpProxyModal.vue";

const store = useProxiesStore()
const show = ref(false)
const preset = ref<Partial<Proxy> | null>(null)

function open() {
  preset.value = null;
  nextTick(() => {
    show.value = true;
  });
}

function onEdit(p: Proxy) {
  preset.value = {...p};
  nextTick(() => {
    show.value = true;
  });
}

async function onDelete(p: Proxy) {
  await store.remove(p.id);
  await refresh()
}

async function refresh() {
  await store.fetch()
}

async function callback() {
  show.value = false
  await refresh()
}

onMounted(() => {
  refresh()
})

function enableSwitch(v: Proxy) {
  store.addOrUpdate({
    ...v,
    enable: !v.enable
  })
}
</script>