<script setup lang="ts">
// Plan 515 G3：桌面壁纸层（vue 桌面宿主 desktop-area 首子层）。
//
// 数据源 = 配置注入（生成期读 storage `shell.desktop.wallpaper`，与 VM
// 轨同键——vue 侧无 storage 桥，Plan 503-2 降级判定；运行期改壁纸需
// 下次生成生效）。三档语义与 VM 轨对齐（iced/renderer.rs
// desktop_wallpaper_element / desktop_wallpaper_scrim）：
//   - 图片路径：bg-cover 铺图 + bg-background scrim（light 10% /
//     dark 35%——`class="dark"` 由 index.html 携带，Plan 458）；
//   - "#hex" 纯色：直接铺色（VM 轨 = desktop.at 根 bg-[#hex] 同视觉）；
//   - 空：不渲染（desktop-area 底色透出 = VM 轨缺省档）。
// pointer-events-none：桌面空白点击语义不受壁纸层拦截。
import { computed } from 'vue'

const props = defineProps<{ value?: string }>()

const v = computed(() => (props.value ?? '').trim())
const isColor = computed(() => v.value.startsWith('#'))
const img = computed(() => (!isColor.value && v.value ? v.value : ''))
const color = computed(() => (isColor.value ? v.value : ''))
</script>

<template>
  <div
    v-if="img"
    class="absolute inset-0 bg-cover bg-center pointer-events-none"
    :style="{ backgroundImage: `url(${img})` }"
  ></div>
  <div v-if="img" class="absolute inset-0 bg-background/10 dark:bg-background/35 pointer-events-none"></div>
  <div v-else-if="color" class="absolute inset-0 pointer-events-none" :style="{ background: color }"></div>
</template>
