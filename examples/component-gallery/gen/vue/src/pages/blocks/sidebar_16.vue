<!-- Sidebar16Page component - Auto-generated from Auto language -->
<script setup lang="ts">
import { Sidebar, SidebarContent, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem } from '@/components/ui/sidebar'

import { Calendar, Command, Home, Inbox, Search, Settings } from 'lucide-vue-next'



</script>

<template>
 <div class="flex flex-col pb-8">
 <div>
 <Sidebar>
 <SidebarHeader>
 <div class="flex flex-row items-center gap-2 px-4 h-14">
 <Command class="w-5 h-5 h-6 w-6" />
 <span class="font-semibold">
 <span>Acme Inc</span>
 </span>
 </div>
 </SidebarHeader>
 <SidebarContent>
 <div class="flex flex-col gap-2 px-2">
 <SidebarMenu>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <Home class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Home</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <Inbox class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Inbox</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <Calendar class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Calendar</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <Search class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Search</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <Settings class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Settings</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 </SidebarMenu>
 </div>
 </SidebarContent>
 </Sidebar>
 <div class="flex flex-col flex-1 p-8">
 <h1 class="text-4xl font-bold tracking-tight text-2xl font-bold tracking-tight">
 <span>Sidebar 16</span>
 </h1>
 <span class="text-sm text-muted-foreground mt-2">
 <span>Sidebar layout variation 16.</span>
 </span>
 </div>
 </div>
 </div>

</template>

<style scoped>
/* Component styles */

</style>


