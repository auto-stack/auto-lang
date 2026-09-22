<script setup lang="ts">
import type { ScrollAreaRootProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import {
  ScrollAreaCorner,
  ScrollAreaRoot,
  ScrollAreaViewport,
} from "reka-ui"
import { cn } from "@/lib/utils"
import ScrollBar from "./ScrollBar.vue"

// PLAN-692: `size` sets the scrollbar thumb width in px via --sb-size.
// The rail reserves +6px so the thumb can widen +2px/side on hover.
const props = withDefaults(defineProps<ScrollAreaRootProps & { class?: HTMLAttributes["class"], size?: number }>(), {
  size: 8,
})

const delegatedProps = reactiveOmit(props, "class", "size")
</script>

<template>
  <ScrollAreaRoot
    v-bind="delegatedProps"
    :style="{ '--sb-size': `${size}px` }"
    :class="cn('relative overflow-hidden', props.class)"
  >
    <ScrollAreaViewport class="h-full w-full rounded-[inherit]">
      <slot />
    </ScrollAreaViewport>
    <ScrollBar />
    <ScrollAreaCorner />
  </ScrollAreaRoot>
</template>
