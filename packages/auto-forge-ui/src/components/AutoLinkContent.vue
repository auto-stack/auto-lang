<template>
  <span class="auto-link-content">
    <template v-for="(part, i) in parts" :key="i">
      <a
        v-if="part.type === 'link' && part.id"
        class="spec-link"
        :href="`#${part.id}`"
        @click.prevent="$emit('jump', part.id)"
      >
        {{ part.text }}
      </a>
      <span v-else>{{ part.text }}</span>
    </template>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  text: string
}

const props = defineProps<Props>()

defineEmits<{
  (e: 'jump', id: string): void
}>()

const SPEC_ID_REGEX = /\b([GRAPTVXIS]\d+(?:\.\d+)?)\b/g

interface TextPart {
  type: 'text' | 'link'
  text: string
  id?: string
}

const parts = computed(() => {
  const result: TextPart[] = []
  let lastIndex = 0
  let match: RegExpExecArray | null

  const regex = new RegExp(SPEC_ID_REGEX.source, 'g')
  const text = props.text

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      result.push({ type: 'text', text: text.slice(lastIndex, match.index) })
    }
    result.push({ type: 'link', text: match[0], id: match[1] })
    lastIndex = regex.lastIndex
  }

  if (lastIndex < text.length) {
    result.push({ type: 'text', text: text.slice(lastIndex) })
  }

  return result
})
</script>

<style scoped>
.auto-link-content {
  white-space: pre-wrap;
  word-break: break-word;
}

.spec-link {
  color: var(--af-primary);
  text-decoration: none;
  font-weight: 500;
  cursor: pointer;
  border-radius: 3px;
  padding: 0 1px;
  transition: background 0.1s;
}

.spec-link:hover {
  background: hsl(var(--primary) / 0.1);
  text-decoration: underline;
}
</style>
