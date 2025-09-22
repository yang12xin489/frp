<template>
  <n-modal v-model:show="showInner" :mask-closable="!loading" transform-origin="center"
           style="width: 90%; height: 80vh">
    <n-card :title="title" class="card-keep-header" :bordered="false" size="huge">
      <n-scrollbar>
        <n-form ref="formRef" :model="model" :rules="rules" :label-placement="labelPlacement" :label-width="labelWidth"
                size="small">
          <slot :model="model" :form-ref="formRef"></slot>
        </n-form>
      </n-scrollbar>

      <template #footer>
        <slot name="footer" :onSave="onSave" :loading="loading">
          <n-space justify="end">
            <!-- 改这里：直接改本地 showInner，立刻关闭；watch/set 会自动 emit 给父组件 -->
            <n-button @click="onCancel" :disabled="loading">{{ cancelText }}</n-button>
            <n-button type="primary" :loading="loading" @click="onSave">{{ okText }}</n-button>
          </n-space>
        </slot>
      </template>
    </n-card>
  </n-modal>
</template>

<script setup lang="ts">
import {reactive, ref, toRaw, unref, watch, computed} from 'vue'
import type {FormInst, FormRules} from 'naive-ui'

type LabelPlacement = 'left' | 'top'

const props = withDefaults(defineProps<{
  show: boolean;
  title: string;
  defaultModel: Record<string, any>;
  initial?: Record<string, any> | null;
  rules?: FormRules;
  labelPlacement?: LabelPlacement;
  labelWidth?: number | string;
  okText?: string;
  cancelText?: string
}>(), {
  initial: null, rules: undefined, labelPlacement: 'left', labelWidth: 80, okText: '保存', cancelText: '取消',
})

const emit = defineEmits<{ (e: 'update:show', v: boolean): void; (e: 'callback', model: any): void }>()

/** ① 用 computed 做 v-model“代理”，避免双向 watch 的绕路 */
const showInner = computed({
  get: () => props.show,
  set: (v: boolean) => emit('update:show', v),
})

const formRef = ref<FormInst | null>(null)
const loading = ref(false)

function clonePlain<T>(val: T): T {
  const v = toRaw(unref(val)) as any
  try {
    return structuredClone(v)
  } catch {
    return JSON.parse(JSON.stringify(v))
  }
}

const model = reactive<Record<string, any>>(clonePlain(props.defaultModel || {}))

function deepMergeSkipUndef(target: any, source: any) {
  if (!source || typeof source !== 'object') return
  for (const [k, v] of Object.entries(source)) {
    if (v === undefined) continue
    if (v && typeof v === 'object' && !Array.isArray(v)) {
      if (!target[k] || typeof target[k] !== 'object') target[k] = {}
      deepMergeSkipUndef(target[k], v)
    } else {
      target[k] = v
    }
  }
}

/** 打开时重置表单模型（用 showInner 来触发） */
watch(showInner, (v) => {
  if (!v) return
  Object.keys(model).forEach((k) => delete (model as any)[k])
  deepMergeSkipUndef(model, clonePlain(props.defaultModel || {}))
  if (props.initial) deepMergeSkipUndef(model, clonePlain(props.initial))
}, {flush: 'sync'})

async function onSave() {
  if (!formRef.value) return
  loading.value = true
  try {
    await formRef.value.validate?.()
    emit('callback', clonePlain(model))
    showInner.value = false // 保存后关闭
  } finally {
    loading.value = false
  }
}

/** ② 取消时直接本地关闭（会通过 computed 的 set 自动 emit 给父组件） */
function onCancel() {
  if (loading.value) return
  showInner.value = false
}
</script>

<style scoped>
.card-keep-header {
  display: flex;
  flex-direction: column;
  max-height: 80vh;
}

.card-keep-header :deep(.n-card__content) {
  flex: 1 1 0;
  overflow: hidden;
}
</style>
