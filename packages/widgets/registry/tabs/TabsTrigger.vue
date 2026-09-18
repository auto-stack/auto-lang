<!-- AutoUI widget registry (Plan 331). Visual layer derived from shadcn-vue (MIT). See NOTICES.
     Maintained in-repo (PLAN-641 Q1: no upstream generator). -->
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { TabsTrigger, type TabsTriggerProps, useForwardProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<TabsTriggerProps & { class?: HTMLAttributes['class'] }>()

// PLAN-641：形态由 Tabs 根 provide（缺省 default 按钮托盘）。
const variant = inject<import('vue').ComputedRef<'default' | 'enclosed'>>(
  'autoTabsVariant',
  computed(() => 'default'),
)

const delegatedProps = computed(() => { const { class: _, ...d } = props; return d })
const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <TabsTrigger
    v-bind="forwarded"
    :class="
      cn(
        variant === 'enclosed'
          ? 'inline-flex items-center justify-center whitespace-nowrap px-4 text-sm font-medium transition-all border border-border focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 data-[state=active]:border-b-0 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=inactive]:bg-secondary/40 data-[state=inactive]:text-muted-foreground'
          : 'inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm',
        props.class,
      )
    "
  >
    <span class="truncate">
      <slot />
    </span>
  </TabsTrigger>
</template>
