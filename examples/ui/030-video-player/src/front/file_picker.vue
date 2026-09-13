<!--
  file_picker.vue — 本地视频文件选择（PLAN-617 T-09）。

  这是 `use.web component` 的手写本地组件（k4-ports-forwarding 先例）：
  Web 端它是 File API 的薄包装 —— 隐藏的 <input type="file"> + 一个
  `trigger` 计数prop 驱动 + `ready`/`pick` 两个 emit。选中文件后用
  URL.createObjectURL 生成可播放地址回传，payload 由 Vue 原生地按位传给
  DSL handler（onpick: .LocalPicked → store.LocalPicked(url, name)）。

  两个如实的边界：
  - VM/iced 端不渲染本组件（use.web component 在 aura 侧退化为 0×0 的
    lucide 占位 —— "file-picker" 不在 lucide 表内，渲染为空），因此 store
    的 local_pick_ready 永远不会置真，点按钮得到的是诚实降级文案。
  - 取消选择不产生任何事件（input change 只在选中时触发），界面维持原状，
    不假装发生了什么。
-->
<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'

const props = defineProps<{ trigger: number }>()
const emit = defineEmits<{ ready: []; pick: [string, string] }>()

const input = ref<HTMLInputElement | null>(null)

onMounted(() => emit('ready'))

// trigger 是递增计数而非布尔：连续两次点按钮不会因值不变而丢失第二次触发。
watch(
  () => props.trigger,
  (seq) => {
    if (seq > 0) input.value?.click()
  },
)

function onChange(e: Event) {
  const el = e.target as HTMLInputElement
  const f = el.files?.[0]
  if (f) emit('pick', URL.createObjectURL(f), f.name)
  // 清空 value：同一文件被再次选中时 change 仍会触发。
  el.value = ''
}
</script>

<template>
  <input
    ref="input"
    type="file"
    accept="video/*,.mp4,.m4v,.webm,.mkv,.mov,.avi"
    style="display: none"
    @change="onChange"
  />
</template>
