<!-- AutoUI widget registry (Plan 331). Visual layer derived from shadcn-vue (MIT). See NOTICES.
     Maintained in-repo (PLAN-641 Q1: no upstream generator). -->
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { TabsList, type TabsListProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<TabsListProps & { class?: HTMLAttributes['class'] }>()

// PLAN-641：形态由 Tabs 根 provide（缺省 default 按钮托盘）。
const variant = inject<import('vue').ComputedRef<'default' | 'enclosed'>>(
  'autoTabsVariant',
  computed(() => 'default'),
)

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
</script>

<template>
  <TabsList
    v-bind="delegatedProps"
    :class="
      cn(
        variant === 'enclosed'
          ? 'inline-flex h-9 w-full items-stretch justify-start bg-muted text-muted-foreground'
          : 'inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground',
        props.class,
      )
    "
  />
</template>
