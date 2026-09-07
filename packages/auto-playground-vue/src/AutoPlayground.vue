<template>
  <PlaygroundCard :code="code" :api-base="apiBase" :height="height" />
</template>

<script setup lang="ts">
/**
 * @deprecated 向后兼容别名（Plan 581 起由 PlaygroundCard + SnippetRunner 分层实现，Playground 设计 §4）。
 * 新宿主请直接使用 `PlaygroundCard`（卡片层，工具栏可配置）或 `SnippetRunner`（拼图层，纯运行单元）；
 * 过渡期结束后本别名将移除。
 *
 * 行为变化仅一处（Playground 设计 §4.4 裁定）：工具栏不再默认显示 ExampleSelector，
 * 需要旧常驻行为时改用 `<PlaygroundCard example-selector>`。
 */
import PlaygroundCard from './components/PlaygroundCard.vue'

const props = withDefaults(defineProps<{
  code?: string
  apiUrl?: string
  height?: string
}>(), {
  code: `fn main() {
    let message = "Hello from Auto!"
    print(message)
}`,
  apiUrl: '',
  height: '500px',
})

// 旧 prop 名映射：apiUrl（后端根地址，''=同源）→ apiBase（''=同源 /api；显式时补 /api 后缀）。
const apiBase = props.apiUrl ? `${props.apiUrl}/api` : ''
</script>
