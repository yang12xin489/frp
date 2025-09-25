<template>
  <div ref="chartEl" style="width: 600px; height: 400px;"></div>
</template>

<script lang="ts" setup>
import { onMounted, onUnmounted, ref } from 'vue';
import * as echarts from 'echarts';
import type { ECharts } from 'echarts';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

const chartEl = ref<HTMLDivElement | null>(null);
let chart: ECharts | null = null;
let unlisten: UnlistenFn | null = null;
let rafId: number | null = null;

const MAX_POINTS = 60;
const INTERVAL_MS = 1000; // 上报周期（1s）
const SPAN = (MAX_POINTS - 1) * INTERVAL_MS;

type Item = { id: number; value: [number, number] };
const data: Item[] = [];

// 平滑滚动的锚点：最近一次采样时，窗口左端应在的位置
let anchorStart = 0;        // 窗口左端的时间戳（ms）
let anchorPerf = 0;         // 对应的 performance.now()（ms）

function fmt(v: number) {
  if (!isFinite(v)) return '-';
  const u = ['B/s', 'KB/s', 'MB/s', 'GB/s', 'TB/s'];
  let i = 0;
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
  return `${v.toFixed(i ? 1 : 0)} ${u[i]}`;
}

onMounted(async () => {
  chart = echarts.init(chartEl.value!);

  chart.setOption({
    // 我们用 RAF 做平移，不需要内置更新动画
    animation: false,
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'line' },
      formatter: (params: any) => {
        const p = params[0];
        const t = new Date(p.data.value[0]);
        return `${t.toLocaleTimeString()}<br/>下行：${fmt(p.data.value[1])}`;
      },
    },
    xAxis: {
      type: 'time',
      boundaryGap: false,
      axisLabel: { show: false },
      axisTick: { show: false },
      axisLine: { show: false },
      splitLine: { show: false },
    },
    yAxis: {
      type: 'value',
      min: 0,
      axisLabel: { formatter: (v: number) => fmt(v as number) },
      axisTick: { show: false },
      minorTick: { show: false },
      splitLine: { show: true },
    },
    series: [{
      type: 'line',
      showSymbol: false,
      smooth: true,
      areaStyle: { opacity: 0.12 },
      data,
    }],
  });

  prefill();          // 先铺满 0 值，防止首次跳变
  startRafPan();      // 开始按帧平滑左移

  // 来一批数据：只追加新点，不改旧点
  unlisten = await listen('frp:traffic', (e) => {
    const payload = e.payload as Array<{ proxy: string; up_bps: number; down_bps: number }>;
    const totalDown = payload.reduce((s, it) => s + it.down_bps, 0);

    const ts = Date.now();

    // 推进锚点（窗口左端应对齐到最新时刻）
    anchorStart = ts - SPAN;
    anchorPerf = performance.now();

    // 丢掉窗口外老数据（多留一两点冗余也行）
    while (data.length && data[0].value[0] < anchorStart - INTERVAL_MS) {
      data.shift();
    }

    // 右侧追加新点（id=时间戳，确保对象身份稳定）
    data.push({ id: ts, value: [ts, totalDown] });
    // 控制长度：最多保留窗口长度 + 少量冗余
    if (data.length > MAX_POINTS + 2) data.splice(0, data.length - (MAX_POINTS + 2));

    // 只更新数据本身（不动坐标，坐标由 RAF 连续平移）
    chart!.setOption({ series: [{ data }] });
  });

  // 自适应
  const onResize = () => chart?.resize();
  window.addEventListener('resize', onResize);
  onUnmounted(() => window.removeEventListener('resize', onResize));
});

function prefill() {
  const now = Date.now();
  anchorStart = now - SPAN;
  anchorPerf = performance.now();

  data.length = 0;
  for (let i = 0; i < MAX_POINTS; i++) {
    const t = anchorStart + i * INTERVAL_MS;
    data.push({ id: t, value: [t, 0] });
  }
  chart!.setOption({
    series: [{ data }],
    xAxis: { min: anchorStart, max: anchorStart + SPAN },
  });
}

function startRafPan() {
  const step = () => {
    // 根据“当前时间相对锚点的流逝”推算窗口左端，形成连续左移
    const elapsed = performance.now() - anchorPerf; // ms
    const displayStart = anchorStart + elapsed;
    chart!.setOption({ xAxis: { min: displayStart, max: displayStart + SPAN } });
    rafId = requestAnimationFrame(step);
  };
  rafId = requestAnimationFrame(step);
}

onUnmounted(() => {
  if (unlisten) { unlisten(); unlisten = null; }
  if (rafId != null) { cancelAnimationFrame(rafId); rafId = null; }
  chart?.dispose();
  chart = null;
});
</script>


<style scoped>
</style>