<template>
  <FormModal v-model:show="visible" title="添加 http 代理" :default-model="defaultModel" :initial="initial || undefined"
             @callback="onSubmit">
    <template #default="{ model }">
      <n-divider title-placement="left" class="margin0">基础配置</n-divider>
      <n-form-item label="代理名称">
        <n-input v-model:value="model.name" placeholder="名称" clearable/>
      </n-form-item>
      <n-form-item label="代理地址">
        <n-input v-model:value="model.localIP" placeholder="127.0.0.1" clearable/>
      </n-form-item>
      <n-form-item label="代理端口">
        <n-input-number v-model:value="model.localPort" placeholder="7000" :min="1" :max="65535"/>
      </n-form-item>

      <n-form-item label="认证">
        <n-switch v-model:value="model.switch.auth">
          <template #checked>开启</template>
          <template #unchecked>关闭</template>
        </n-switch>
      </n-form-item>
      <n-form-item label="用户名" v-if="model.switch.auth">
        <n-input v-model:value="model.httpUser" clearable/>
      </n-form-item>
      <n-form-item label="密码" v-if="model.switch.auth">
        <n-input v-model:value="model.httpPassword" type="password" show-password-on="click" clearable/>
      </n-form-item>

      <n-divider title-placement="left" class="margin0">域名配置</n-divider>
      <n-form-item label="域名类型">
        <n-select v-model:value="model.switch.domain" :options="options"/>
      </n-form-item>
      <n-form-item label="子域名" v-if="model.switch.domain === 'sub'">
        <n-input v-model:value="model.subdomain" clearable/>
      </n-form-item>
      <n-form-item label="自定义域名" v-if="model.switch.domain === 'custom'">
        <n-select v-model:value="model.customDomains" filterable multiple tag :show-arrow="false" :show="false"
                  placeholder="输入完整域名（www.xx.com），按回车确认"/>
      </n-form-item>
    </template>
  </FormModal>
</template>
<script setup lang="ts">
import FormModal from '@/components/common/FormModal.vue'
import {computed} from 'vue'
import {DomainType, type HttpProxy, ProxyType} from '@/domain/types'
import {saveProxy} from '@/api/config'

const defaultModel: HttpProxy = {
  name: '',
  type: ProxyType.HTTP,
  enable: false,
  localIP: '127.0.0.1',
  localPort: 8080,
  subdomain: '',
  customDomains: [],
  locations: [],
  httpUser: '',
  httpPassword: '',
  switch: {
    domain: DomainType.SUB,
    auth: false
  },
}

const props = defineProps<{ visible: boolean; initial?: Partial<HttpProxy> | null }>()
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void; (e: 'callback', v: HttpProxy): void }>()
const visible = computed({get: () => props.visible, set: (v) => emit('update:visible', v)})
const options = (Object.values(DomainType) as DomainType[]).map((v) => ({label: v, value: v}))

async function onSubmit(v: HttpProxy) {
  await saveProxy(v);
  emit('callback', v)
  emit("update:visible", false)
}
</script>
<style scoped>
</style>