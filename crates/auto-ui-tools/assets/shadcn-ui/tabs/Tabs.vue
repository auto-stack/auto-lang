<script setup lang="ts">
import type { TabsRootEmits, TabsRootProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { computed, provide } from "vue"
import { TabsRoot, useForwardPropsEmits } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<TabsRootProps & { class?: HTMLAttributes["class"], variant?: "default" | "enclosed" }>()
const emits = defineEmits<TabsRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, variant: __, ...delegated } = props
  return delegated
})

// PLAN-641：形态向 List/Trigger/Content 子件下发（provide/inject）。
const variant = computed(() => props.variant ?? "default")
provide("autoTabsVariant", variant)

const forwardedProps = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <TabsRoot v-bind="forwardedProps" :class="cn('relative', props.class)">
    <slot />
  </TabsRoot>
</template>
