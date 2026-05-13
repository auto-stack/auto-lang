<template>
  <div ref="containerRef" class="markdown-content">
    <MarkdownRender :content="content" :final="true" />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { MarkdownRender } from 'markstream-vue'

const props = defineProps<{
  content: string
}>()

const emit = defineEmits<{
  linkClick: [id: string]
}>()

const containerRef = ref<HTMLElement | null>(null)

// ID reference regex: G1, G1.1, A1, D1, P1, S1.1, V1, X2026-05, I1
const ID_RE = /\b([GADPSVXI]\d+(?:\.\d+)?)\b/g

function processLinks() {
  const container = containerRef.value
  if (!container) return

  // Walk all text nodes inside the rendered markdown
  const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT, null)
  const textNodes: Text[] = []
  let node: Node | null
  while ((node = walker.nextNode())) {
    // Only process text nodes that are direct children of elements (not inside <a> or <code>)
    const parent = node.parentElement
    if (parent && (parent.tagName === 'A' || parent.tagName === 'CODE' || parent.tagName === 'PRE')) continue
    textNodes.push(node as Text)
  }

  for (const textNode of textNodes) {
    const text = textNode.textContent || ''
    if (!ID_RE.test(text)) continue
    ID_RE.lastIndex = 0

    const frag = document.createDocumentFragment()
    let lastIndex = 0
    let match: RegExpExecArray | null

    while ((match = ID_RE.exec(text))) {
      // Append text before match
      if (match.index > lastIndex) {
        frag.appendChild(document.createTextNode(text.slice(lastIndex, match.index)))
      }
      // Create clickable link
      const span = document.createElement('span')
      span.className = 'spec-link'
      span.textContent = match[1]
      span.addEventListener('click', (e) => {
        e.stopPropagation()
        emit('linkClick', match![1])
      })
      frag.appendChild(span)
      lastIndex = match.index + match[0].length
    }

    if (lastIndex < text.length) {
      frag.appendChild(document.createTextNode(text.slice(lastIndex)))
    }

    if (frag.childNodes.length > 0) {
      textNode.parentNode?.replaceChild(frag, textNode)
    }
  }
}

watch(() => props.content, () => {
  nextTick(processLinks)
}, { immediate: true })
</script>

<style scoped>
.markdown-content :deep(.spec-link) {
  display: inline;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.85em;
  font-weight: 600;
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 0.08);
  padding: 0.05em 0.35em;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.12s;
}
.markdown-content :deep(.spec-link:hover) {
  background: hsl(var(--primary) / 0.18);
}
</style>
