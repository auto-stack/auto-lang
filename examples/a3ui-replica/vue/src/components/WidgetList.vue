<template>
  <div class="mt-2 mx-2">
    <div v-if="widgets.length === 0" class="px-3 text-[13px] text-slate-400 italic">
      No widgets yet
    </div>
    <router-link
      v-for="widget in widgets"
      :key="widget.id"
      :to="`/widget/${widget.id}`"
      exact-active-class="router-link-exact-active"
      active-class=""
      :class="[
        'flex items-center mx-2 px-3 py-2 rounded-md text-[13px] transition-all duration-150',
        isActive(widget.id)
          ? 'bg-violet-50 text-violet-600'
          : 'text-slate-600 hover:bg-violet-50 hover:text-violet-600'
      ]"
    >
      {{ widget.name }}
    </router-link>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const widgets = ref<any[]>([])

function loadWidgets() {
  widgets.value = JSON.parse(localStorage.getItem('a2ui-widgets') || '[]')
}

function isActive(id: string) {
  return route.path === `/widget/${id}`
}

onMounted(() => {
  loadWidgets()
  window.addEventListener('a2ui-widgets-changed', loadWidgets)
})

onUnmounted(() => {
  window.removeEventListener('a2ui-widgets-changed', loadWidgets)
})
</script>
