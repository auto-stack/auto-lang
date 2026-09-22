<script setup lang="ts">
import type { ScrollAreaScrollbarProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { ScrollAreaScrollbar, ScrollAreaThumb } from "reka-ui"
import { cn } from "@/lib/utils"

const props = withDefaults(defineProps<ScrollAreaScrollbarProps & { class?: HTMLAttributes["class"] }>(), {
  orientation: "vertical",
})

const delegatedProps = reactiveOmit(props, "class")
</script>

<template>
  <ScrollAreaScrollbar
    v-bind="delegatedProps"
    :class="
      cn('flex touch-none select-none transition-colors items-center justify-center',
         orientation === 'vertical'
           && 'h-full w-[calc(var(--sb-size,8px)_+_6px)] p-px',
         orientation === 'horizontal'
           && 'w-full h-[calc(var(--sb-size,8px)_+_6px)] flex-col p-px',
         props.class)"
  >
    <!-- PLAN-692 unified thumb: thickness = --sb-size (8px default), hover widens
         +2px/side, press highlights primary. Reka drives thumb length through inline
         `width/height: var(--reka-scroll-area-thumb-{width,height})`; the thickness
         side is undefined by reka, so the thickness var is layered here per orientation
         (vertical consumes width, horizontal height) without touching the length var. -->
    <ScrollAreaThumb
      class="relative rounded-full bg-border transition-all duration-150 cursor-pointer active:bg-primary"
      :class="orientation === 'vertical'
        ? '[--reka-scroll-area-thumb-width:var(--sb-size,8px)] hover:[--reka-scroll-area-thumb-width:calc(var(--sb-size,8px)_+_4px)]'
        : '[--reka-scroll-area-thumb-height:var(--sb-size,8px)] hover:[--reka-scroll-area-thumb-height:calc(var(--sb-size,8px)_+_4px)]'"
    />
  </ScrollAreaScrollbar>
</template>
