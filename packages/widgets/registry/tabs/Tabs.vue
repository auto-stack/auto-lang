<!-- AutoUI widget registry (Plan 331). Visual layer derived from shadcn-vue (MIT). See NOTICES.
     Maintained in-repo (PLAN-641 Q1: no upstream generator). -->
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed, provide } from 'vue'
import {
  TabsRoot,
  type TabsRootEmits,
  type TabsRootProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<TabsRootProps & { class?: HTMLAttributes['class'], variant?: 'default' | 'enclosed' }>()
const emits = defineEmits<TabsRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, variant: __, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)

// PLAN-641：形态向 List/Trigger/Content 子件下发（provide/inject——跨 slot
// prop drilling 不可行）。enclosed = 激活 tab 与内容面板连通，非激活 tab 扁平。
const variant = computed(() => props.variant ?? 'default')
provide('autoTabsVariant', variant)
</script>

<template>
  <TabsRoot v-bind="forwarded" :class="cn('relative', props.class)">
    <slot />
  </TabsRoot>
</template>
