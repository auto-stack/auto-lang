<!-- AutoUI widget registry (Plan 331). Visual layer derived from shadcn-vue (MIT). See NOTICES.
     Maintained in-repo (PLAN-641 Q1: no upstream generator). -->
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { TabsContent, type TabsContentProps, useForwardProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<TabsContentProps & { class?: HTMLAttributes['class'] }>()

// PLAN-641：形态由 Tabs 根 provide（缺省 default 按钮托盘）。
const variant = inject<import('vue').ComputedRef<'default' | 'enclosed'>>(
  'autoTabsVariant',
  computed(() => 'default'),
)

const delegatedProps = computed(() => { const { class: _, ...d } = props; return d })
const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <TabsContent
    v-bind="forwarded"
    :class="
      cn(
        variant === 'enclosed'
          ? 'border border-t-0 border-border bg-background ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2'
          : 'mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        props.class,
      )
    "
  >
    <slot />
  </TabsContent>
</template>
