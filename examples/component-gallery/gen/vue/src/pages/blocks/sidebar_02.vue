<!-- Sidebar02Page component - Auto-generated from Auto language -->
<script setup lang="ts">
import { Collapsible } from '@/components/ui/collapsible'
import { Sidebar, SidebarContent, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem } from '@/components/ui/sidebar'

import { Command, Database, FileText, Folder, Play } from 'lucide-vue-next'



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
 <Collapsible>
 <div class="flex flex-row items-center px-2 py-1 text-xs font-medium text-muted-foreground">
 <span>Platform</span>
 </div>
 <SidebarMenu>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <Play class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Playground</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <Database class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Models</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <FileText class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Documentation</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 </SidebarMenu>
 </Collapsible>
 <Collapsible>
 <div class="flex flex-row items-center px-2 py-1 text-xs font-medium text-muted-foreground">
 <span>Projects</span>
 </div>
 <SidebarMenu>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <Folder class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Design Engineering</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 <SidebarMenuItem>
 <SidebarMenuButton>
 <Folder class="w-5 h-5 h-4 w-4 mr-2" />
 <span>Sales & Marketing</span>
 </SidebarMenuButton>
 </SidebarMenuItem>
 </SidebarMenu>
 </Collapsible>
 </div>
 </SidebarContent>
 </Sidebar>
 <div class="flex flex-col flex-1 p-8">
 <h1 class="text-4xl font-bold tracking-tight text-2xl font-bold tracking-tight">
 <span>Sidebar 02</span>
 </h1>
 <span class="text-sm text-muted-foreground mt-2">
 <span>Sidebar with collapsible groups.</span>
 </span>
 </div>
 </div>
 </div>

</template>

<style scoped>
/* Component styles */

</style>


