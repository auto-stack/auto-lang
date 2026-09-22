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
      cn('flex touch-none select-none transition-colors items-start justify-center',
         orientation === 'vertical'
           && 'h-full w-[var(--sb-size,8px)]',
         orientation === 'horizontal'
           && 'w-full h-[var(--sb-size,8px)] flex-col',
         props.class)"
  >
    <!-- PLAN-692 unified thumb (rev4, user rulings 2026-09-22/23):
         - visible tint mixes in the theme accent (primary/40) — plain gray read too pale;
         - hovering the thumb deepens it to primary/60 (adaptive: darker over light bg,
           lighter over dark bg); NO width change on hover (rev4 ruling — thumb keeps
           its constant width, set by `size`); no press/active highlight (rev2);
         - transform is EXCLUDED from the transition: reka mounts the thumb at its
           layout origin and then repositions it via `transform: translate3d` —
           animating `all` made it visibly slide from the track top to its real
           position on every hover-mount (flash). Only background-color eases.
         Thickness var layering: reka drives thumb length through inline
         `width/height: var(--reka-scroll-area-thumb-{width,height})`; the thickness
         side is undefined by reka, so it is layered here per orientation.
         Track alignment MUST stay items-start: reka positions the thumb via
         transform from its static layout origin — any cross-axis centering
         (items-center) shifts the origin and breaks drag mapping. -->
    <ScrollAreaThumb
      class="relative rounded-full bg-[hsl(var(--primary)/0.4)] transition-colors duration-150 cursor-pointer hover:bg-[hsl(var(--primary)/0.6)]"
      :class="orientation === 'vertical'
        ? '[--reka-scroll-area-thumb-width:var(--sb-size,8px)]'
        : '[--reka-scroll-area-thumb-height:var(--sb-size,8px)]'"
    />
  </ScrollAreaScrollbar>
</template>
