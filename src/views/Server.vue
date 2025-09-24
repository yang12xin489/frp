<template>
  <n-card title="FRPS 服务器设置" class="card-keep-header">
    <template #header-extra>
      <n-space size="small">
        <n-button type="info" size="small" :disabled="loading" @click="onReset">重置</n-button>
        <n-button type="success" size="small" :loading="saving" @click="onSave">保存</n-button>
      </n-space>
    </template>
    <n-scrollbar style="padding-right: 16px">
      <n-form ref="formRef" :model="form" label-placement="left" size="small" label-width="80px">
        <n-divider title-placement="left">服务器配置</n-divider>
        <n-form-item label="地址">
          <n-input v-model:value="form.serverAddr" placeholder="127.0.0.1" clearable/>
        </n-form-item>
        <n-form-item label="端口">
          <n-input-number v-model:value="form.serverPort" placeholder="7000" :min="1" :max="65535" clearable/>
        </n-form-item>
        <n-form-item label="认证">
          <n-switch v-model:value="form.switch.auth">
            <template #checked>开启</template>
            <template #unchecked>关闭</template>
          </n-switch>
        </n-form-item>
        <n-form-item label="token" v-if="form.switch.auth">
          <n-input v-model:value="form.auth.token" type="password" show-password-on="click" placeholder="token" clearable/>
        </n-form-item>
        <n-divider title-placement="left" class="margin0">Web 配置</n-divider>
        <n-form-item label="地址">
          <n-input v-model:value="form.webServer.addr" placeholder="127.0.0.1" clearable/>
        </n-form-item>
        <n-form-item label="端口">
          <n-input-number v-model:value="form.webServer.port" placeholder="7400" :min="1" :max="65535" clearable/>
        </n-form-item>
        <n-form-item label="认证">
          <n-switch v-model:value="form.switch.webServer">
            <template #checked>开启</template>
            <template #unchecked>关闭</template>
          </n-switch>
        </n-form-item>
        <n-form-item label="用户名" v-if="form.switch.webServer">
          <n-input v-model:value="form.webServer.user" clearable/>
        </n-form-item>
        <n-form-item label="密码" v-if="form.switch.webServer">
          <n-input v-model:value="form.webServer.password" type="password" show-password-on="click" clearable/>
        </n-form-item>
      </n-form>
    </n-scrollbar>
  </n-card>
</template>
<script setup lang="ts">
import {reactive, ref, onMounted} from 'vue'
import type {FormInst} from 'naive-ui'
import {useMessage} from 'naive-ui'
import {useConfigStore} from '@/stores/useConfigStore'
import {defaultConfig, type FrpcConfig} from '@/domain/frpc'

const message = useMessage();
const store = useConfigStore()
const formRef = ref<FormInst | null>(null)
const form = reactive<FrpcConfig>({...defaultConfig})
const loading = ref(false);
const saving = ref(false)

onMounted(async () => {
  loading.value = true;
  try {
    await store.fetch();
    Object.assign(form, store.cfg || defaultConfig)
  } catch (e) {
    console.error(e);
    message.error('加载配置失败')
  } finally {
    loading.value = false
  }
})

async function onSave() {
  if (!formRef.value) return
  try {
    saving.value = true;
    await formRef.value.validate();
    await store.save({...form});
    message.success('已保存')
  } catch (e) {
    console.error(e);
    message.error('保存失败')
  } finally {
    saving.value = false
  }
}

async function onReset() {
  try {
    loading.value = true;
    await store.fetch();
    Object.assign(form, store.cfg || defaultConfig)
  } catch (e) {
    console.error(e);
    message.error('重置失败')
  } finally {
    loading.value = false
  }
}
</script>
<style scoped>
.card-keep-header {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.card-keep-header :deep(.n-card__content) {
  flex: 1 1 0;
  overflow: hidden;
}
</style>