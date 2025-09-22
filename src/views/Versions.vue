<template>
  <xin-grid label="FRP 版本管理">
    <template #header>
      <n-popover trigger="hover" placement="left">
        <template #trigger>
          <n-button type="primary">
            <template #icon>
              <NIcon size="18">
                <Icon icon="tabler:refresh"/>
              </NIcon>
            </template>
          </n-button>
        </template>
        <span>获取Frp最新版本</span>
      </n-popover>
    </template>
    <template #default>
      <n-card hoverable embedded v-for="version in store.frpVersions.values()" :key="version.id" style="width: 100%;">
        <template #cover>
          <n-progress
              v-if="store.isDownloading(version.name)"
              :percentage="store.getProgress(version.name)"
              :show-indicator="false"
              :border-radius="0"
              :gap-degree="0"
              status="success" x
              processing/>
        </template>

        <template #header>
          <n-flex justify="space-between" align="center">
            <n-tag>{{ version.version }}</n-tag>
            <n-switch
                :value="version.active"
                @update:value="activeSwitch(version)"
                :disabled="!version.exist || store.activating"
                :loading="store.activating && store.activatingName === version.name"
            />
          </n-flex>
        </template>
        <n-tag :bordered="false">{{ version.size }}</n-tag>
        <template #footer>
          <n-space justify="end" align="center">
            <template v-if="version.exist">
              <n-popconfirm positive-text="删除" negative-text="取消" @positive-click="onDelete(version)">
                确定删除该版本？
                <template #trigger>
                  <n-button size="small" type="error">删除</n-button>
                </template>
              </n-popconfirm>
            </template>
            <template v-else>
              <n-button size="small" :disabled="version.exist || store.isDownloading(version.name)" @click="onDownload(version)">下载</n-button>
            </template>
          </n-space>
        </template>
      </n-card>
    </template>
  </xin-grid>
</template>
<script setup lang="ts">
import {useFrpVersions} from '@/stores/useFrpVersions'
import {onBeforeUnmount, onMounted} from 'vue'
import {useMessage} from 'naive-ui'
import {Icon} from '@iconify/vue'
import XinGrid from "@/components/versions/XinGrid.vue";

const store = useFrpVersions()
const message = useMessage()

onMounted(async () => {
  await store.fetchVersions();
  await store.bindAll();
})

onBeforeUnmount(() => store.unbindAll())

function onDownload(v: any) {
  store.setProgress(v.name, 0)
  store.download(v);
}

async function onDelete(v: any) {
  try {
    await store.delete(v);
    message.success('已删除')
  } catch (e) {
    console.error(e);
    message.error('删除失败')
  }
}

function activeSwitch(v: any) {
  if (v.active) {
    store.deactivate(v);
  } else {
    store.activate(v);
  }
}
</script>