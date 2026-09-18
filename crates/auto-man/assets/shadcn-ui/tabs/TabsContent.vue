<script setup lang="ts">
import type { TabsContentProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { computed, inject } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { TabsContent } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<TabsContentProps & { class?: HTMLAttributes["class"] }>()

// PLAN-641：enclosed 内容面板与激活 tab 同背景（顶部无间距/圆角断点）。
const variant = inject("autoTabsVariant", computed(() => "default"))

const delegatedProps = reactiveOmit(props, "class")
</script>

<template>
  <TabsContent
    v-bind="delegatedProps"
    :class="cn(
      variant === 'enclosed'
        ? 'bg-background ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2'
        : 'mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
      props.class,
    )"
  >
    <slot />
  </TabsContent>
</template>
