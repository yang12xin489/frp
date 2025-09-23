<template>
  <n-card size="small" :title="title" class="log-card">
    <n-input v-model:value="q" size="small" placeholder="搜索（支持正则）" clearable style="width: 100%;margin-bottom: 8px"/>
    <n-space justify="space-between" align="center" class="toolbar">
      <n-space :size="8">
        <n-checkbox v-model:checked="regex">正则</n-checkbox>
        <n-checkbox v-model:checked="showStdout">stdout</n-checkbox>
        <n-checkbox v-model:checked="showStderr">stderr</n-checkbox>
        <n-checkbox v-model:checked="showSystem">system</n-checkbox>
        <n-checkbox v-model:checked="wrap">换行</n-checkbox>
        <n-checkbox v-model:checked="showTs">时间</n-checkbox>
        <n-checkbox v-model:checked="follow">跟随</n-checkbox>
        <n-checkbox v-model:checked="lineNumbers">行号</n-checkbox>
        <n-checkbox v-model:checked="useAnsi">颜色</n-checkbox>
      </n-space>
      <n-space :size="8">
        <n-button size="small" @click="onCopy">复制</n-button>
        <n-button size="small" @click="$emit('clear')">清空</n-button>
        <n-button size="small" @click="onExport">导出</n-button>
      </n-space>
    </n-space>

    <n-virtual-list
        ref="vref"
        class="v-list"
        :items="filtered"
        :item-size="itemSize"
        :item-resizable="wrap"
        :item-key="itemKey"
        :default-scroll-velocity="1"
        @scroll="onScroll"
    >
      <template #default="{ item, index }">
        <div :class="['row', item.level]">
          <div v-if="lineNumbers" class="gutter">{{ lineNo(item, index) }}</div>
          <div class="line">
            <span v-if="showTs" class="ts">[{{ formatTs(item.ts) }}]</span>
            <span class="lvl">{{ item.level.toUpperCase() }}</span>
            <span class="txt" v-html="renderHtml(item.text, item.level)"></span>
          </div>
        </div>
      </template>
    </n-virtual-list>
  </n-card>
</template>

<script setup lang="ts">
import {computed, nextTick, ref, watch} from 'vue'
import {NVirtualList, useMessage, VirtualListInst} from 'naive-ui'
import {type LogEntry, useFrpcStore} from '@/stores/useFrpcStore'
import {AnsiUp} from 'ansi_up'

defineProps<{ title?: string }>()
defineEmits<{ (e: 'clear'): void }>()

const store = useFrpcStore()
const message = useMessage()

// 控件状态
const q = ref('')
const regex = ref(false)
const showStdout = ref(true)
const showStderr = ref(true)
const showSystem = ref(true)
const wrap = ref(false)
const showTs = ref(true)
const follow = ref(true)
const lineNumbers = ref(true)
const useAnsi = ref(true)

// 固定行高（不开换行时）：性能最佳
const BASE_LINE_HEIGHT = 20
const WRAP_ESTIMATED = 24
const itemSize = computed<number>(() => (wrap.value ? WRAP_ESTIMATED : BASE_LINE_HEIGHT))

const itemKey = (e: LogEntry) => e.id

function lineNo(item: LogEntry, _index: number) {
  return item.id
}

// 级别过滤
const levelOk = (lv: string) =>
    (lv === 'stdout' && showStdout.value) ||
    (lv === 'stderr' && showStderr.value) ||
    (lv === 'system' && showSystem.value)

// 过滤（含正则）
const filtered = computed<LogEntry[]>(() => {
  const src = store.entries
  if (!q.value) return src.filter(e => levelOk(e.level))
  try {
    const flags = 'gi' + (supportsIndices() ? 'd' : '')
    const re = regex.value
        ? new RegExp(q.value, flags)
        : new RegExp(q.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), flags)
    return src.filter(e => levelOk(e.level) && re.test(e.text))
  } catch {
    const needle = q.value.toLowerCase()
    return src.filter(e => levelOk(e.level) && e.text.toLowerCase().includes(needle))
  }
})

// 自动跟随到底部
const vref = ref<VirtualListInst | null>(null)
const atBottom = ref(true)

watch(() => filtered.value.length, async () => {
  await nextTick()
  if (follow.value && atBottom.value) {
    vref.value?.scrollTo({position: 'bottom'})
  }
})

function onScroll(e: Event) {
  const el = e.target as HTMLElement | null
  if (!el) return
  const delta = el.scrollHeight - el.scrollTop - el.clientHeight
  atBottom.value = delta < 4
}

// ANSI + 搜索高亮
const ansi = new AnsiUp()
ansi.use_classes = false

const HL_COLORS = ['#ffe58f', '#ffd591', '#ffccc7', '#d9f7be', '#bae7ff', '#efdbff']

function supportsIndices() {
  try {
    new RegExp('a', 'd');
    return true
  } catch {
    return false
  }
}

function escapeHtml(s: string) {
  return s.replace(/[&<>"']/g, c => ({'&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'}[c]!))
}

function ansiHtml(s: string) {
  return useAnsi.value ? ansi.ansi_to_html(s) : escapeHtml(s)
}

function renderHtml(text: string, level: 'stdout' | 'stderr' | 'system') {
  if (!q.value) {
    const html = ansiHtml(text)
    if (level === 'stderr' && html === escapeHtml(text)) return `<span style="color:#c0342b;font-weight:600">${html}</span>`
    if (level === 'system' && html === escapeHtml(text)) return `<span style="color:#888">${html}</span>`
    return html
  }

  // 正则/字面量
  let re: RegExp
  try {
    const flags = 'gi' + (supportsIndices() ? 'd' : '')
    re = regex.value
        ? new RegExp(q.value, flags)
        : new RegExp(q.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), flags)
  } catch {
    re = new RegExp(q.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi')
  }

  const withIdx = (re as any).hasIndices === true || /d/.test(re.flags)
  let out = ''
  let pos = 0

  for (const m of text.matchAll(re as any)) {
    const idx = (m as any).index ?? 0
    const full = m[0] ?? ''
    if (full === '') continue

    out += ansiHtml(text.slice(pos, idx))

    if (withIdx && (m as any).indices) {
      const inds: Array<[number, number] | undefined> = (m as any).indices
      const [ms, me] = inds[0]!
      let p = ms
      for (let g = 1; g < inds.length; g++) {
        const r = inds[g];
        if (!r || r[0] < 0) continue
        const [gs, ge] = r
        if (gs > p) out += ansiHtml(text.slice(p, gs))
        const color = HL_COLORS[(g - 1) % HL_COLORS.length]
        out += `<span class="hl" style="background:${color}">${ansiHtml(text.slice(gs, ge))}</span>`
        p = ge
      }
      if (p < me) out += ansiHtml(text.slice(p, me))
      pos = me
    } else {
      out += `<span class="hl">${ansiHtml(full)}</span>`
      pos = idx + full.length
    }

    if ((re as any).lastIndex !== undefined && (re as any).lastIndex === idx) {
      ;(re as any).lastIndex++ // 避免零宽卡住
    }
  }

  out += ansiHtml(text.slice(pos))

  if (level === 'stderr' && !useAnsi.value) out = `<span style="color:#c0342b;font-weight:600">${out}</span>`
  if (level === 'system' && !useAnsi.value) out = `<span style="color:#888">${out}</span>`
  return out
}

// 复制/导出
async function onCopy() {
  try {
    await navigator.clipboard?.writeText(store.asText);
    message.success('已复制')
  } catch {
    message.error('复制失败')
  }
}

async function onExport() {
  // try {
  //   const filePath = await save({
  //     title: '保存日志',
  //     defaultPath: `frpc-${new Date().toISOString().replace(/[:.]/g, '-')}.log`,
  //     filters: [{name: 'Log', extensions: ['log', 'txt']}]
  //   })
  //   if (!filePath) return
  //   await writeTextFile(filePath, store.asText)
  //   message.success('已保存')
  // } catch (e) {
  //   console.error(e);
  //   message.error('保存失败')
  // }
}

function formatTs(ts: number) {
  const d = new Date(ts)
  return d.toLocaleTimeString()
}
</script>

<style scoped>
.log-card {
  height: 70vh;
  display: flex;
  flex-direction: column;
  overflow: hidden; /* 防止外层出现滚动条 */
}

/* （关键）内容区本身成为 flex 容器，并允许收拢高度 */
.log-card :deep(.n-card__content) {
  display: flex;
  flex-direction: column;
  flex: 1 1 0;
  min-height: 0;
  overflow: hidden;
}

/* NVirtualList 根元素：自己滚动，而不是页面滚动 */
.v-list {
  flex: 1 1 0;
  min-height: 0;
  height: 100%;
  overflow: auto;      /* 纵向/横向滚动条都以内滚为准 */
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace;
  font-size: 12px;
  line-height: 1.5;
}

.toolbar {
  margin-bottom: 8px;
}

/* 行：行号 + 内容 */
.row {
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: start;
}

/* 行号栏 */
.gutter {
  user-select: none;
  color: #888;
  text-align: right;
  padding: 0 8px;
  min-width: 3em;
}

/* 文本样式与换行策略 */
.line {
  display: inline;
}

.nowrap .txt {
  white-space: pre;
}

/* 不换行：出现横向滚动，但在 .vlist 内 */
.wrap .txt {
  white-space: pre-wrap;
  word-break: break-word;
}

.ts {
  color: #888;
  margin-right: 6px;
}

.lvl {
  color: var(--n-text-color);
  margin-right: 6px;
}

.stderr .lvl {
  color: #c0342b;
  font-weight: 600;
}

.system .lvl {
  color: #888;
}

.hl {
  background: #ffe58f;
  border-radius: 2px;
  padding: 0 1px;
}
</style>