<!-- Dashboard01Page component -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import { AreaChart } from '@/components/ui/chart-area'
import { Avatar } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { TrendingUp, TrendingDown } from 'lucide-vue-next'

const timeRange = ref<string>('30d')

const statsByRange: Record<string, any[]> = {
  '7d': [
    { label: 'Total Revenue', value: '$12,231.89', change: '+5.1%', trend: 'up' },
    { label: 'Subscriptions', value: '+421', change: '+12.3%', trend: 'up' },
    { label: 'Bounce Rate', value: '11.02%', change: '-1.2%', trend: 'down' },
    { label: 'Active Now', value: '+189', change: '+54', trend: 'up' },
  ],
  '30d': [
    { label: 'Total Revenue', value: '$45,231.89', change: '+20.1%', trend: 'up' },
    { label: 'Subscriptions', value: '+2350', change: '+180.1%', trend: 'up' },
    { label: 'Bounce Rate', value: '12.23%', change: '+4.1%', trend: 'up' },
    { label: 'Active Now', value: '+573', change: '+201', trend: 'up' },
  ],
  '90d': [
    { label: 'Total Revenue', value: '$124,502.00', change: '+32.5%', trend: 'up' },
    { label: 'Subscriptions', value: '+6,830', change: '+210.2%', trend: 'up' },
    { label: 'Bounce Rate', value: '10.89%', change: '-2.3%', trend: 'down' },
    { label: 'Active Now', value: '+1,203', change: '+412', trend: 'up' },
  ],
}

const revenueByRange: Record<string, any[]> = {
  '7d': [
    { desktop: 40, month: 'Mon', mobile: 15 }, { desktop: 55, month: 'Tue', mobile: 30 },
    { desktop: 35, month: 'Wed', mobile: 25 }, { desktop: 60, month: 'Thu', mobile: 20 },
    { desktop: 48, month: 'Fri', mobile: 35 }, { desktop: 70, month: 'Sat', mobile: 45 },
    { desktop: 52, month: 'Sun', mobile: 28 },
  ],
  '30d': [
    { desktop: 186, month: 'Jan', mobile: 80 }, { month: 'Feb', mobile: 200, desktop: 305 },
    { desktop: 237, mobile: 120, month: 'Mar' }, { desktop: 73, mobile: 190, month: 'Apr' },
    { month: 'May', desktop: 209, mobile: 130 }, { mobile: 140, desktop: 214, month: 'Jun' },
  ],
  '90d': [
    { desktop: 320, month: 'Jan', mobile: 180 }, { desktop: 410, month: 'Feb', mobile: 250 },
    { desktop: 380, month: 'Mar', mobile: 200 }, { desktop: 290, month: 'Apr', mobile: 310 },
    { desktop: 450, month: 'May', mobile: 280 }, { desktop: 520, month: 'Jun', mobile: 340 },
    { desktop: 490, month: 'Jul', mobile: 360 }, { desktop: 580, month: 'Aug', mobile: 410 },
    { desktop: 610, month: 'Sep', mobile: 390 },
  ],
}

const stats = computed(() => statsByRange[timeRange.value])
const revenueData = computed(() => revenueByRange[timeRange.value])

const users = ref<any[]>([
  { name: 'User 1', amount: '+$42.00', initial: 'U1', email: 'user1@example.com' },
  { amount: '+$84.00', initial: 'U2', email: 'user2@example.com', name: 'User 2' },
  { email: 'user3@example.com', amount: '+$126.00', initial: 'U3', name: 'User 3' },
  { amount: '+$168.00', initial: 'U4', name: 'User 4', email: 'user4@example.com' },
  { email: 'user5@example.com', name: 'User 5', initial: 'U5', amount: '+$210.00' },
])
</script>

<template>
 <div class="flex flex-col pb-8">
 <h1 class="text-4xl font-bold tracking-tight">Dashboard</h1>
 <span class="text-muted-foreground">Overview of your business metrics.</span>
 <div class="flex flex-col w-full p-6 gap-6 mt-6">
 <div class="flex flex-row items-center justify-between">
 <h2 class="text-lg font-semibold">Overview</h2>
 <div class="flex flex-row gap-1">
 <Button v-for="range in ['7d', '30d', '90d']" :key="range" :variant="timeRange === range ? 'default' : 'outline'" size="sm" @click="timeRange = range">{{ range }}</Button>
 </div>
 </div>
 <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
 <Card v-for="stat in stats" :key="stat.label">
 <div class="flex flex-col p-6 gap-2">
 <span class="text-sm font-medium text-muted-foreground">{{ stat.label }}</span>
 <div class="flex flex-row items-center gap-2">
 <span class="text-2xl font-bold">{{ stat.value }}</span>
 </div>
 <span class="flex items-center gap-1 text-xs" :class="stat.trend === 'up' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
 <TrendingUp v-if="stat.trend === 'up'" class="h-3 w-3" />
 <TrendingDown v-else class="h-3 w-3" />
 {{ stat.change }}
 </span>
 </div>
 </Card>
 </div>
 <div class="flex flex-row gap-4 flex-col lg:flex-row">
 <Card class="flex-1">
 <div class="flex flex-col p-6 gap-4">
 <span class="text-base font-semibold">Revenue Overview</span>
 <AreaChart :data="revenueData" :categories="['desktop', 'mobile']" index="month" class="h-[300px] w-full" />
 </div>
 </Card>
 <Card class="flex-1">
 <div class="flex flex-col p-6 gap-4">
 <span class="text-base font-semibold">Recent Sales</span>
 <span class="text-sm text-muted-foreground">You made 265 sales this month.</span>
 <div class="flex flex-col gap-3">
 <div class="flex flex-row items-center justify-between" v-for="user in users" :key="user.name">
 <div class="flex flex-row items-center gap-3">
 <Avatar><span>{{ user.initial }}</span></Avatar>
 <div class="flex flex-col gap-0.5">
 <span class="text-sm font-medium">{{ user.name }}</span>
 <span class="text-xs text-muted-foreground">{{ user.email }}</span>
 </div>
 </div>
 <span class="text-sm font-medium">{{ user.amount }}</span>
 </div>
 </div>
 </div>
 </Card>
 </div>
 </div>
 </div>

</template>

<style scoped>
/* Component styles */

</style>
