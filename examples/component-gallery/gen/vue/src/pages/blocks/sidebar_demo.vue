<!-- SidebarDemoPage component -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import { Avatar } from '@/components/ui/avatar'
import { Card } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import { Bell, Command, FileText, Inbox, Send, User, Mail, FilePen, Star, PanelLeft } from 'lucide-vue-next'

const activeItem = ref<string>('inbox')
const sidebarOpen = ref(true)

const menuItems = [
  { id: 'inbox', label: 'Inbox', icon: Inbox, count: 12 },
  { id: 'drafts', label: 'Drafts', icon: FilePen, count: 3 },
  { id: 'sent', label: 'Sent', icon: Send, count: 0 },
  { id: 'profile', label: 'Profile', icon: User, count: 0 },
  { id: 'notifications', label: 'Notifications', icon: Bell, count: 5 },
]

const activeLabel = computed(() => menuItems.find(m => m.id === activeItem.value)?.label ?? 'Inbox')

const inboxItems = [
  { from: 'Alice Chen', subject: 'Project Update Q4', preview: 'Hi, here is the latest update on our Q4 project milestones...', time: '10:30 AM', starred: true },
  { from: 'Bob Zhang', subject: 'Design Review Meeting', preview: 'Please review the attached design mockups before our meeting tomorrow...', time: '9:15 AM', starred: false },
  { from: 'Carol Li', subject: 'API Documentation', preview: 'I have finished the API docs for the new endpoints. Please take a look...', time: 'Yesterday', starred: true },
  { from: 'David Wang', subject: 'Bug Report #142', preview: 'Found a critical bug in the payment module that needs immediate attention...', time: 'Yesterday', starred: false },
  { from: 'Eve Liu', subject: 'Team Building Event', preview: 'Reminder: Team building event this Friday at 3pm in the main hall...', time: 'Mon', starred: false },
]

const draftItems = [
  { to: 'Marketing Team', subject: 'Campaign Strategy', preview: 'Here are my thoughts on the upcoming campaign...', time: 'Draft' },
  { to: 'HR Department', subject: 'Leave Request', preview: 'I would like to request annual leave from...', time: 'Draft' },
  { to: 'Client', subject: 'Proposal Update', preview: 'Thank you for your patience. Here is the updated...', time: 'Draft' },
]

const sentItems = [
  { to: 'Alice Chen', subject: 'Re: Project Update Q4', preview: 'Thanks for the update. I have reviewed the milestones...', time: '11:00 AM' },
  { to: 'Team', subject: 'Weekly Standup Notes', preview: 'Here are the notes from today\'s standup meeting...', time: 'Yesterday' },
  { to: 'Bob Zhang', subject: 'Re: Design Review', preview: 'Great work on the mockups. I have a few suggestions...', time: 'Mon' },
]

const notifications = [
  { title: 'New deployment successful', description: 'Production build v2.4.1 deployed successfully', time: '5 min ago', type: 'success' },
  { title: 'Pull request merged', description: 'PR #234 "feat: add user dashboard" was merged into main', time: '1 hour ago', type: 'info' },
  { title: 'Build failed', description: 'CI build #567 failed on stage "test" — check logs', time: '3 hours ago', type: 'error' },
  { title: 'New team member', description: 'Frank Wu joined the Engineering team', time: 'Yesterday', type: 'info' },
  { title: 'Sprint review reminder', description: 'Sprint review meeting scheduled for tomorrow at 2pm', time: 'Yesterday', type: 'info' },
]
</script>

<template>
 <div class="flex flex-col pb-8">
 <h1 class="text-4xl font-bold tracking-tight">Sidebar</h1>
 <span class="text-muted-foreground">A fully interactive sidebar layout with navigation and content switching.</span>
 <div class="mt-6 mx-auto border rounded-lg overflow-hidden shadow-sm flex" style="width: 100%; max-width: 900px; height: 560px;">

 <!-- Sidebar -->
 <div class="shrink-0 border-r bg-muted/30 flex flex-col transition-all duration-200" :style="{ width: sidebarOpen ? '220px' : '0px', overflow: sidebarOpen ? 'visible' : 'hidden' }">
 <div class="flex flex-row items-center gap-2 px-4 h-14 border-b">
 <Command class="h-5 w-5" />
 <span class="font-semibold text-sm">Acme Inc</span>
 </div>
 <div class="flex flex-col gap-2 px-2 py-3 flex-1 overflow-auto">
 <span class="text-xs font-medium text-muted-foreground px-2 py-1">Platform</span>
 <button v-for="item in menuItems.slice(0, 3)" :key="item.id" class="flex flex-row items-center gap-2 px-2 py-1.5 rounded-md text-sm w-full text-left transition-colors" :class="activeItem === item.id ? 'bg-accent text-accent-foreground font-medium' : 'hover:bg-accent/50'" @click="activeItem = item.id">
 <component :is="item.icon" class="h-4 w-4 shrink-0" />
 <span class="flex-1">{{ item.label }}</span>
 <span v-if="item.count > 0" class="text-xs text-muted-foreground">{{ item.count }}</span>
 </button>
 <Separator class="my-1" />
 <span class="text-xs font-medium text-muted-foreground px-2 py-1">Settings</span>
 <button v-for="item in menuItems.slice(3)" :key="item.id" class="flex flex-row items-center gap-2 px-2 py-1.5 rounded-md text-sm w-full text-left transition-colors" :class="activeItem === item.id ? 'bg-accent text-accent-foreground font-medium' : 'hover:bg-accent/50'" @click="activeItem = item.id">
 <component :is="item.icon" class="h-4 w-4 shrink-0" />
 <span class="flex-1">{{ item.label }}</span>
 <span v-if="item.count > 0" class="text-xs text-muted-foreground">{{ item.count }}</span>
 </button>
 </div>
 <div class="flex flex-row items-center gap-3 px-4 py-3 border-t">
 <Avatar><span class="text-xs">JD</span></Avatar>
 <div class="flex flex-col gap-0.5 min-w-0">
 <span class="text-sm font-medium truncate">John Doe</span>
 <span class="text-xs text-muted-foreground truncate">john@example.com</span>
 </div>
 </div>
 </div>

 <!-- Main content -->
 <div class="flex-1 flex flex-col min-w-0">
 <header class="flex h-14 items-center gap-2 border-b px-4 shrink-0">
 <button @click="sidebarOpen = !sidebarOpen" class="inline-flex items-center justify-center rounded-md h-7 w-7 hover:bg-accent transition-colors">
 <PanelLeft class="h-4 w-4" />
 </button>
 <Separator orientation="vertical" class="h-4" />
 <span class="text-sm text-muted-foreground">Home</span>
 <span class="text-sm text-muted-foreground">/</span>
 <span class="text-sm font-medium">{{ activeLabel }}</span>
 </header>
 <div class="flex-1 overflow-auto p-4">
 <!-- Inbox -->
 <template v-if="activeItem === 'inbox'">
 <div class="flex flex-col gap-3">
 <div class="flex flex-row items-center justify-between">
 <h2 class="text-lg font-semibold">Inbox</h2>
 <span class="text-sm text-muted-foreground">{{ inboxItems.length }} messages</span>
 </div>
 <Card v-for="item in inboxItems" :key="item.subject" class="cursor-pointer hover:bg-muted/50 transition-colors">
 <div class="flex flex-row items-start gap-3 p-3">
 <Mail class="h-4 w-4 mt-0.5 text-muted-foreground shrink-0" />
 <div class="flex flex-col gap-0.5 flex-1 min-w-0">
 <div class="flex flex-row items-center gap-2">
 <span class="text-sm font-semibold">{{ item.from }}</span>
 <span class="text-xs text-muted-foreground ml-auto shrink-0">{{ item.time }}</span>
 </div>
 <span class="text-sm font-medium">{{ item.subject }}</span>
 <span class="text-xs text-muted-foreground truncate">{{ item.preview }}</span>
 </div>
 <Star v-if="item.starred" class="h-3 w-3 text-yellow-500 shrink-0 fill-yellow-500" />
 </div>
 </Card>
 </div>
 </template>
 <!-- Drafts -->
 <template v-else-if="activeItem === 'drafts'">
 <div class="flex flex-col gap-3">
 <h2 class="text-lg font-semibold">Drafts</h2>
 <Card v-for="item in draftItems" :key="item.subject" class="cursor-pointer hover:bg-muted/50 transition-colors">
 <div class="flex flex-row items-start gap-3 p-3">
 <FilePen class="h-4 w-4 mt-0.5 text-muted-foreground shrink-0" />
 <div class="flex flex-col gap-0.5 flex-1 min-w-0">
 <span class="text-sm font-medium">To: {{ item.to }}</span>
 <span class="text-sm font-semibold">{{ item.subject }}</span>
 <span class="text-xs text-muted-foreground truncate">{{ item.preview }}</span>
 </div>
 <span class="text-xs text-muted-foreground shrink-0">{{ item.time }}</span>
 </div>
 </Card>
 </div>
 </template>
 <!-- Sent -->
 <template v-else-if="activeItem === 'sent'">
 <div class="flex flex-col gap-3">
 <h2 class="text-lg font-semibold">Sent</h2>
 <Card v-for="item in sentItems" :key="item.subject" class="cursor-pointer hover:bg-muted/50 transition-colors">
 <div class="flex flex-row items-start gap-3 p-3">
 <Send class="h-4 w-4 mt-0.5 text-muted-foreground shrink-0" />
 <div class="flex flex-col gap-0.5 flex-1 min-w-0">
 <span class="text-sm font-medium">To: {{ item.to }}</span>
 <span class="text-sm font-semibold">{{ item.subject }}</span>
 <span class="text-xs text-muted-foreground truncate">{{ item.preview }}</span>
 </div>
 <span class="text-xs text-muted-foreground shrink-0">{{ item.time }}</span>
 </div>
 </Card>
 </div>
 </template>
 <!-- Profile -->
 <template v-else-if="activeItem === 'profile'">
 <div class="flex flex-col gap-4 max-w-sm">
 <h2 class="text-lg font-semibold">Profile</h2>
 <div class="flex flex-row items-center gap-4">
 <Avatar class="h-14 w-14"><span class="text-base">JD</span></Avatar>
 <div class="flex flex-col">
 <span class="text-base font-semibold">John Doe</span>
 <span class="text-sm text-muted-foreground">john@example.com</span>
 </div>
 </div>
 <Separator />
 <div class="flex flex-col gap-2">
 <div class="flex flex-row justify-between"><span class="text-sm text-muted-foreground">Role</span><span class="text-sm font-medium">Senior Engineer</span></div>
 <div class="flex flex-row justify-between"><span class="text-sm text-muted-foreground">Team</span><span class="text-sm font-medium">Platform</span></div>
 <div class="flex flex-row justify-between"><span class="text-sm text-muted-foreground">Location</span><span class="text-sm font-medium">San Francisco, CA</span></div>
 <div class="flex flex-row justify-between"><span class="text-sm text-muted-foreground">Joined</span><span class="text-sm font-medium">March 2023</span></div>
 </div>
 </div>
 </template>
 <!-- Notifications -->
 <template v-else-if="activeItem === 'notifications'">
 <div class="flex flex-col gap-3">
 <div class="flex flex-row items-center justify-between">
 <h2 class="text-lg font-semibold">Notifications</h2>
 <span class="text-sm text-muted-foreground">{{ notifications.length }} notifications</span>
 </div>
 <Card v-for="item in notifications" :key="item.title" class="cursor-pointer hover:bg-muted/50 transition-colors">
 <div class="flex flex-row items-start gap-3 p-3">
 <div class="mt-1.5 shrink-0 h-2 w-2 rounded-full" :class="item.type === 'success' ? 'bg-green-500' : item.type === 'error' ? 'bg-red-500' : 'bg-blue-500'" />
 <div class="flex flex-col gap-0.5 flex-1 min-w-0">
 <span class="text-sm font-semibold">{{ item.title }}</span>
 <span class="text-xs text-muted-foreground">{{ item.description }}</span>
 </div>
 <span class="text-xs text-muted-foreground shrink-0">{{ item.time }}</span>
 </div>
 </Card>
 </div>
 </template>
 </div>
 </div>

 </div>
 </div>

</template>

<style scoped>
/* Component styles */

</style>
