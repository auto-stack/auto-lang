<script setup lang="ts">
import type { Component } from "vue"
import { computed } from "vue"
import { RouterLink } from "vue-router"

/**
 * Plan 482: AutoUI NavItem — the web side of the nav-item class-token
 * contract. The class strings below are mirrored verbatim from
 * crates/auto-lang/src/ui/nav_contract.rs (single source of truth; a Rust
 * unit test asserts the two ends cannot drift).
 */
const ITEM_BASE_MD =
  "nav-item flex w-full items-center gap-2 rounded-md px-3 h-9 text-sm text-left text-foreground select-none cursor-pointer transition-colors"
const ITEM_BASE_LG =
  "nav-item flex w-full items-start gap-3 rounded-md px-3 py-[10px] text-sm text-left text-foreground select-none cursor-pointer transition-colors"
const ITEM_BASE_SM =
  "nav-item flex w-full items-center gap-2 rounded-md px-2 h-7 text-xs text-left text-foreground select-none cursor-pointer transition-colors"
const ITEM_HOVER = "hover:bg-accent hover:text-accent-foreground"
const ITEM_ACTIVE = "bg-primary/10 text-primary font-medium"
const ITEM_DISABLED = "opacity-60 cursor-default"
const BADGE_PILL =
  "ml-auto inline-flex items-center justify-center rounded-full bg-primary/15 text-primary px-2 py-[2px] text-xs font-medium shrink-0"
const ICON_MD = "h-4 w-4 shrink-0"
const ICON_LG = "h-5 w-5 shrink-0"
const TEXTS_FILL = "flex-1 min-w-0"
const TEXT_DESC = "text-xs text-muted-foreground"

const props = withDefaults(
  defineProps<{
    /** Route address — renders a RouterLink (hash URL updates). */
    to?: string
    /** Selected state (explicit; router mode can also auto-detect). */
    active?: boolean
    /** Grayed out, not clickable. */
    disabled?: boolean
    /** Left icon as literal text/emoji (when no resolved lucide component). */
    icon?: string
    /** Resolved lucide component (codegen imports literal icon names). */
    iconComp?: Component
    /** Primary text. */
    label?: string
    /** Secondary line (two-line layout). */
    desc?: string
    /** Right-side badge pill. */
    badge?: string
    /** md = h-9 single line (default), lg = two-line, sm = h-7 compact. */
    size?: "sm" | "md" | "lg"
  }>(),
  { label: "", desc: "", badge: "", size: "md" },
)

defineEmits<{ click: [event: MouseEvent] }>()

const base = computed(
  () => (props.size === "sm" ? ITEM_BASE_SM : props.size === "lg" ? ITEM_BASE_LG : ITEM_BASE_MD),
)
// Active items never carry hover classes (build-time either/or) so hover can
// never override the selected background — mirrors the VM builder exactly.
const state = computed(() =>
  props.disabled ? ITEM_DISABLED : props.active ? ITEM_ACTIVE : ITEM_HOVER,
)
const classes = computed(() => `${base.value} ${state.value}`)
const iconClasses = props.size === "lg" ? ICON_LG : ICON_MD
const textsClasses = computed(() =>
  props.badge ? `flex flex-col min-w-0 ${TEXTS_FILL}` : "flex flex-col min-w-0",
)
const root = computed(() => (props.to && !props.disabled ? RouterLink : "button"))
</script>

<template>
  <component
    :is="root"
    :to="root === RouterLink ? to : undefined"
    :disabled="root === 'button' ? disabled || undefined : undefined"
    type="button"
    :class="classes"
    :aria-current="active ? 'page' : undefined"
    :data-active="active ? 'true' : undefined"
  >
    <component :is="iconComp" v-if="iconComp" :class="iconClasses" />
    <span v-else-if="icon" :class="'inline-flex items-center justify-center ' + iconClasses">{{ icon }}</span>
    <slot>
      <span :class="textsClasses">
        <span class="truncate">{{ label }}</span>
        <span v-if="desc" :class="TEXT_DESC + ' truncate'">{{ desc }}</span>
      </span>
    </slot>
    <span v-if="badge" :class="BADGE_PILL">{{ badge }}</span>
  </component>
</template>
