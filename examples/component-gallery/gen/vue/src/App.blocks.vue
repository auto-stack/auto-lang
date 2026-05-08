<!-- App component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
const router = useRouter()

import { Button } from '@/components/ui/button'
import { Drawer } from '@/components/ui/drawer'
import { ScrollArea } from '@/components/ui/scroll-area'

import { BarChart, Bell, Command, Home, Image, Layers, LayoutGrid, LayoutTemplate, Lock, Menu, MousePointerClick, Navigation, Palette, PanelLeft, Search, SquareStack, Table, Type } from 'lucide-vue-next'

import ThemeToggle from '@/components/ThemeToggle.vue'


const sidebarOpen = ref<boolean>(false)
const searchOpen = ref<boolean>(false)
const searchQuery = ref<string>('')

function closeSidebar(): void {
  sidebarOpen.value = false;

}

function closeSearch(): void {
  searchOpen.value = false;

}

function openSearch(): void {
  searchOpen.value = true;

}

function openSidebar(): void {
  sidebarOpen.value = true;

}


</script>

<template>
  <div class="flex flex-col h-screen">
    <div class="flex flex-col h-screen overflow-hidden">
      <header class="sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur">
        <div class="flex flex-row h-14 items-center px-4 md:px-6 w-full gap-3">
          <Button class="md:hidden -ml-2" @click="openSidebar">
            <Menu class="h-4 w-4" />          </Button>
          <div class="flex flex-row items-center gap-2 mr-4 shrink-0">
            <Layers class="w-5 h-5 h-5 w-5 text-primary" />
            <span class="font-bold text-lg hidden sm:inline">Auto UI</span>
          </div>
          <div class="flex flex-row hidden md:flex items-center gap-1 text-sm">
            <a href="/" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">Home</span></a>
            <a href="/ui/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">UI</span></a>
            <a href="/ui/gallery/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">Gallery</span></a>
            <a href="/ui/blocks/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">Blocks</span></a>
            <a href="/ui/charts/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">Charts</span></a>
            <a href="/ui/a2ui/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">A2UI</span></a>
          </div>
          <div class="flex flex-row ml-auto items-center gap-1">
            <Button variant="ghost" class="hidden md:flex h-9 w-9" @click="openSearch">
              <Search class="h-4 w-4" />            </Button>
            <ThemeToggle />
          </div>
        </div>
      </header>
      <Drawer v-model:open="searchOpen" @Close="closeSearch" />
      <div class="flex flex-row flex-1 overflow-hidden">
        <aside class="hidden md:flex w-64 lg:w-72 border-r bg-background flex-col shrink-0">
          <ScrollArea class="flex-1 px-3 py-4">
            <span class="mb-1.5 px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">Overview</span>
            <router-link to="/" class="group block" active-class="" exact-active-class="router-link-exact-active">
              <div class="flex flex-row items-center gap-2 rounded-md px-2 py-1.5 text-sm">
                <Home class="w-5 h-5 h-4 w-4" />
                <span>Home</span>
              </div>
            </router-link>

            <span class="mt-4 mb-1.5 px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">Blocks</span>
            <router-link to="/blocks">
              <div class="flex flex-row items-center gap-2 rounded-md px-2 py-1.5 text-sm"><LayoutTemplate class="h-4 w-4 shrink-0" /><span>All Blocks</span></div>            </router-link>
            <router-link to="/blocks/login-01">
              <div class="flex flex-row items-center gap-2 rounded-md px-2 py-1.5 text-sm"><Lock class="h-4 w-4 shrink-0" /><span>Login</span></div>            </router-link>
            <router-link to="/blocks/dashboard-01">
              <div class="flex flex-row items-center gap-2 rounded-md px-2 py-1.5 text-sm"><BarChart class="h-4 w-4 shrink-0" /><span>Dashboard</span></div>            </router-link>
            <router-link to="/blocks/products-01">
              <div class="flex flex-row items-center gap-2 rounded-md px-2 py-1.5 text-sm"><Table class="h-4 w-4 shrink-0" /><span>Products</span></div>            </router-link>
            <router-link to="/blocks/sidebar-demo">
              <div class="flex flex-row items-center gap-2 rounded-md px-2 py-1.5 text-sm"><PanelLeft class="h-4 w-4 shrink-0" /><span>Sidebar</span></div>            </router-link>
          </ScrollArea>
        </aside>
        <Drawer v-model:open="sidebarOpen" @Close="closeSidebar" />
        <main class="flex-1 overflow-hidden">
          <ScrollArea class="h-full">
            <div class="p-4 md:p-8 lg:p-10 max-w-5xl mx-auto pb-24 md:pb-10">
              <router-view />
            </div>
          </ScrollArea>
        </main>
      </div>
      <div class="flex flex-row md:hidden fixed bottom-0 left-0 right-0 h-16 bg-background/95 backdrop-blur border-t z-40 items-center justify-around">
        <router-link to="/" class="group block" active-class="" exact-active-class="router-link-exact-active">
          <div class="flex flex-col items-center gap-0.5 py-2 px-3">
            <Home class="w-5 h-5 h-5 w-5" />
            <span class="text-[10px]">Home</span>
          </div>
        </router-link>
        <Button @click="openSidebar">
          <div class="flex flex-col items-center gap-0.5 py-2 px-3">
            <LayoutGrid class="w-5 h-5 h-5 w-5" />
            <span class="text-[10px]">Components</span>
          </div>
        </Button>
        <Button @click="openSearch">
          <div class="flex flex-col items-center gap-0.5 py-2 px-3">
            <Search class="w-5 h-5 h-5 w-5" />
            <span class="text-[10px]">Search</span>
          </div>
        </Button>
        <ThemeToggle>
          <div class="flex flex-col items-center gap-0.5 py-2 px-3">
            <Palette class="w-5 h-5 h-5 w-5" />
            <span class="text-[10px]">Theme</span>
          </div>
        </ThemeToggle>
      </div>
    </div>
  </div>

</template>

<style scoped>
/* Component styles */

</style>


