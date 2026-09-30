<script setup lang="ts">
import EvidenceImage from './EvidenceImage.vue'
withDefaults(defineProps<{ lang?: 'zh' | 'en' }>(), { lang: 'en' })
</script>

<template>
  <div class="autoshell-preview">
    <EvidenceImage
      src="/apps/autoshell/ash-01.png"
      :alt="lang === 'zh' ? 'ash 原生终端：彩色 ls 命令与文件表格' : 'Native ash terminal: colored ls command and file table'"
      :caption="lang === 'zh' ? 'ls · 彩色文件表格。取自 ash-01 原生截图，点击查看完整会话。' : 'ls · A colored file table. Detail from the original ash-01 capture; enlarge to see the full session.'"
      :zoom-label="lang === 'zh' ? '查看完整截图' : 'View full capture'"
      :close-label="lang === 'zh' ? '关闭' : 'Close'"
      :original-label="lang === 'zh' ? '打开原图' : 'Open original'"
      table-preview
    />
  </div>
</template>

<style scoped>
.autoshell-preview { min-width: 0; width: 100%; }
</style>
