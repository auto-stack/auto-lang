<script setup lang="ts">
import type { TabsListProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { computed, inject } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { TabsList } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<TabsListProps & { class?: HTMLAttributes["class"] }>()

// PLAN-641：形态由 Tabs 根 provide（缺省 default 按钮托盘）。
const variant = inject("autoTabsVariant", computed(() => "default"))

const delegatedProps = reactiveOmit(props, "class")
</script>

<template>
  <TabsList
    v-bind="delegatedProps"
    :class="cn(
      variant === 'enclosed'
        ? 'inline-flex h-9 w-full items-stretch justify-start bg-muted text-muted-foreground'
        : 'inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground',
      props.class,
    )"
  >
    <slot />
  </TabsList>
</template>
