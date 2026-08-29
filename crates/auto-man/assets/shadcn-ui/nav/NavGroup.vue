<script setup lang="ts">
import { computed, ref, watch } from "vue"
import { ChevronDown, ChevronRight } from "lucide-vue-next"

/**
 * Plan 482: AutoUI NavGroup — labeled (optionally collapsible) group of
 * nav items. Class strings mirrored verbatim from
 * crates/auto-lang/src/ui/nav_contract.rs.
 */
const GROUP_LABEL = "nav-group-label px-3 pt-2 pb-1 text-xs font-medium text-muted-foreground"
const GROUP_TOGGLE =
  "nav-group-toggle flex w-full items-center gap-2 px-3 py-2 rounded-md text-sm font-medium text-foreground cursor-pointer select-none"
const GROUP_TOGGLE_HOVER = "hover:bg-accent"
const GROUP_CONTENT = "nav-group-content flex flex-col gap-1"
const GROUP_CONTENT_INDENT = "pl-3"

const props = withDefaults(
  defineProps<{
    label?: string
    collapsible?: boolean
    /** Bound fold state; leave unbound for built-in per-group state. */
    open?: boolean
    indent?: boolean
  }>(),
  { label: "", collapsible: false, indent: false },
)

const emit = defineEmits<{ toggle: [] }>()

// Unbound (open === undefined) → self-managed, defaults open.
const isOpen = ref(props.open ?? true)
watch(
  () => props.open,
  (v) => {
    if (v !== undefined) isOpen.value = v
  },
)

function onToggle() {
  isOpen.value = !isOpen.value
  emit("toggle")
}

const contentClasses = computed(() =>
  props.indent ? `${GROUP_CONTENT} ${GROUP_CONTENT_INDENT}` : GROUP_CONTENT,
)
</script>

<template>
  <div class="nav-group flex flex-col">
    <div v-if="!collapsible" :class="GROUP_LABEL">{{ label }}</div>
    <button v-else type="button" :class="`${GROUP_TOGGLE} ${GROUP_TOGGLE_HOVER}`" @click="onToggle">
      <ChevronDown v-if="isOpen" class="h-4 w-4 shrink-0 text-muted-foreground" />
      <ChevronRight v-else class="h-4 w-4 shrink-0 text-muted-foreground" />
      <span class="truncate">{{ label }}</span>
    </button>
    <div v-show="isOpen" :class="contentClasses">
      <slot />
    </div>
  </div>
</template>
