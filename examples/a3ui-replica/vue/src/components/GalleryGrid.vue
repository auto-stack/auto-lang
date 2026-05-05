<template>
  <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
    <div
      v-for="widget in galleryWidgets"
      :key="widget.id"
      class="bg-white rounded-xl border border-slate-200 p-4 cursor-pointer hover:shadow-md transition-shadow"
      @click="openWidget(widget)"
    >
      <div class="font-medium text-slate-800 mb-1">{{ widget.name }}</div>
      <div class="text-sm text-slate-500 mb-3">{{ widget.description }}</div>
      <div class="h-48 bg-slate-50 rounded-lg border border-slate-100 overflow-hidden">
        <A2UIRenderer :components="widget.components" :data-model="widget.dataModel" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import A2UIRenderer from './A2UIRenderer.vue'

const router = useRouter()

const galleryWidgets = [
  {
    id: 'flight-status',
    name: 'Flight Status',
    description: 'Airline flight card with departure/arrival info',
    components: [
      { id: 'root', component: 'Card', child: 'content' },
      { id: 'content', component: 'Column', children: ['header', 'route', 'times'], gap: 12 },
      { id: 'header', component: 'Row', children: ['flight-icon', 'flight-num', 'date'], justify: 'spaceBetween', align: 'center' },
      { id: 'flight-icon', component: 'Icon', name: 'send', size: 16 },
      { id: 'flight-num', component: 'Text', value: 'OS 87', variant: 'body' },
      { id: 'date', component: 'Text', value: 'Mon, Dec 15', variant: 'caption', color: 'text-secondary' },
      { id: 'route', component: 'Row', children: ['origin', 'arrow', 'dest'], justify: 'center', align: 'center', gap: 8 },
      { id: 'origin', component: 'Text', value: 'Vienna', variant: 'h4' },
      { id: 'arrow', component: 'Icon', name: 'arrow_forward', size: 20 },
      { id: 'dest', component: 'Text', value: 'New York', variant: 'h4' },
      { id: 'times', component: 'Row', children: ['departs', 'status', 'arrives'], justify: 'spaceBetween', align: 'center' },
      { id: 'departs', component: 'Column', children: ['dep-label', 'dep-time'], align: 'center', gap: 4 },
      { id: 'dep-label', component: 'Text', value: 'Departs', variant: 'caption', color: 'text-secondary' },
      { id: 'dep-time', component: 'Text', value: '6:15 PM', variant: 'body' },
      { id: 'status', component: 'Column', children: ['st-label', 'st-val'], align: 'center', gap: 4 },
      { id: 'st-label', component: 'Text', value: 'Status', variant: 'caption', color: 'text-secondary' },
      { id: 'st-val', component: 'Text', value: 'On Time', variant: 'body', color: 'accent-green' },
      { id: 'arrives', component: 'Column', children: ['arr-label', 'arr-time'], align: 'center', gap: 4 },
      { id: 'arr-label', component: 'Text', value: 'Arrives', variant: 'caption', color: 'text-secondary' },
      { id: 'arr-time', component: 'Text', value: '10:30 PM', variant: 'body' },
    ],
    dataModel: {},
  },
  {
    id: 'email-compose',
    name: 'Email Compose',
    description: 'Email composition form',
    components: [
      { id: 'root', component: 'Card', child: 'form' },
      { id: 'form', component: 'Column', children: ['from-row', 'to-row', 'subject-row', 'body', 'actions'], gap: 12 },
      { id: 'from-row', component: 'Row', children: ['from-label', 'from-val'], gap: 8, align: 'center' },
      { id: 'from-label', component: 'Text', value: 'FROM', variant: 'caption', color: 'text-secondary' },
      { id: 'from-val', component: 'Text', value: 'alex@acme.com', variant: 'body' },
      { id: 'to-row', component: 'Row', children: ['to-label', 'to-val'], gap: 8, align: 'center' },
      { id: 'to-label', component: 'Text', value: 'TO', variant: 'caption', color: 'text-secondary' },
      { id: 'to-val', component: 'Text', value: 'jordan@acme.com', variant: 'body' },
      { id: 'subject-row', component: 'Row', children: ['sub-label', 'sub-val'], gap: 8, align: 'center' },
      { id: 'sub-label', component: 'Text', value: 'SUBJECT', variant: 'caption', color: 'text-secondary' },
      { id: 'sub-val', component: 'Text', value: 'Q4 Revenue Forecast', variant: 'body' },
      { id: 'body', component: 'Text', value: 'Hi Jordan,\n\nFollowing up on our call...\n\nBest,\nAlex', variant: 'body' },
      { id: 'actions', component: 'Row', children: ['send-btn', 'discard-btn'], gap: 8 },
      { id: 'send-btn', component: 'Button', label: 'Send email', variant: 'primary' },
      { id: 'discard-btn', component: 'Button', label: 'Discard', variant: 'secondary' },
    ],
    dataModel: {},
  },
  {
    id: 'calendar-day',
    name: 'Calendar Day',
    description: 'Day view calendar',
    components: [
      { id: 'root', component: 'Card', child: 'content' },
      { id: 'content', component: 'Column', children: ['day-header', 'events'], gap: 12 },
      { id: 'day-header', component: 'Column', children: ['day-name', 'day-num'], align: 'center' },
      { id: 'day-name', component: 'Text', value: 'Sunday', variant: 'caption', color: 'text-secondary' },
      { id: 'day-num', component: 'Text', value: '28', variant: 'h2' },
      { id: 'events', component: 'List', children: ['evt1', 'evt2', 'evt3'], gap: 8 },
      { id: 'evt1', component: 'Row', children: ['evt1-title', 'evt1-time'], justify: 'spaceBetween', align: 'center' },
      { id: 'evt1-title', component: 'Text', value: 'Lunch', variant: 'body' },
      { id: 'evt1-time', component: 'Text', value: '12:00 - 12:45 PM', variant: 'caption', color: 'text-secondary' },
      { id: 'evt2', component: 'Row', children: ['evt2-title', 'evt2-time'], justify: 'spaceBetween', align: 'center' },
      { id: 'evt2-title', component: 'Text', value: 'Q1 roadmap review', variant: 'body' },
      { id: 'evt2-time', component: 'Text', value: '1:00 - 2:00 PM', variant: 'caption', color: 'text-secondary' },
      { id: 'evt3', component: 'Row', children: ['evt3-title', 'evt3-time'], justify: 'spaceBetween', align: 'center' },
      { id: 'evt3-title', component: 'Text', value: 'Team standup', variant: 'body' },
      { id: 'evt3-time', component: 'Text', value: '3:30 - 4:00 PM', variant: 'caption', color: 'text-secondary' },
    ],
    dataModel: {},
  },
  {
    id: 'weather',
    name: 'Weather',
    description: 'Current weather display',
    components: [
      { id: 'root', component: 'Card', child: 'content' },
      { id: 'content', component: 'Column', children: ['header', 'temps', 'location', 'desc'], align: 'center', gap: 8 },
      { id: 'header', component: 'Text', value: 'Current', variant: 'caption', color: 'text-secondary' },
      { id: 'temps', component: 'Row', children: ['high', 'low'], gap: 16, align: 'center', justify: 'center' },
      { id: 'high', component: 'Text', value: '72°', variant: 'h2' },
      { id: 'low', component: 'Text', value: '58°', variant: 'h4', color: 'text-secondary' },
      { id: 'location', component: 'Text', value: 'Austin, TX', variant: 'body' },
      { id: 'desc', component: 'Text', value: 'Clear skies with light breeze', variant: 'caption', color: 'text-secondary' },
    ],
    dataModel: {},
  },
  {
    id: 'user-profile',
    name: 'User Profile',
    description: 'Social profile card',
    components: [
      { id: 'root', component: 'Card', child: 'content' },
      { id: 'content', component: 'Column', children: ['avatar', 'name', 'handle', 'bio', 'stats', 'follow-btn'], align: 'center', gap: 8 },
      { id: 'avatar', component: 'Image', src: 'https://i.pravatar.cc/150?img=5', width: 64, height: 64, borderRadius: 'full' },
      { id: 'name', component: 'Text', value: 'Sarah Chen', variant: 'h4' },
      { id: 'handle', component: 'Text', value: '@sarahchen', variant: 'caption', color: 'text-secondary' },
      { id: 'bio', component: 'Text', value: 'Product Designer at Tech Co.', variant: 'body', align: 'center' },
      { id: 'stats', component: 'Row', children: ['followers', 'following', 'posts'], gap: 24, justify: 'center' },
      { id: 'followers', component: 'Column', children: ['f-num', 'f-label'], align: 'center', gap: 2 },
      { id: 'f-num', component: 'Text', value: '12,400', variant: 'h4' },
      { id: 'f-label', component: 'Text', value: 'Followers', variant: 'caption', color: 'text-secondary' },
      { id: 'following', component: 'Column', children: ['fg-num', 'fg-label'], align: 'center', gap: 2 },
      { id: 'fg-num', component: 'Text', value: '892', variant: 'h4' },
      { id: 'fg-label', component: 'Text', value: 'Following', variant: 'caption', color: 'text-secondary' },
      { id: 'posts', component: 'Column', children: ['p-num', 'p-label'], align: 'center', gap: 2 },
      { id: 'p-num', component: 'Text', value: '347', variant: 'h4' },
      { id: 'p-label', component: 'Text', value: 'Posts', variant: 'caption', color: 'text-secondary' },
      { id: 'follow-btn', component: 'Button', label: 'Follow', variant: 'primary' },
    ],
    dataModel: {},
  },
  {
    id: 'login-form',
    name: 'Login Form',
    description: 'Authentication form',
    components: [
      { id: 'root', component: 'Card', child: 'content' },
      { id: 'content', component: 'Column', children: ['title', 'subtitle', 'email-field', 'pw-field', 'signin-btn', 'signup-row'], align: 'center', gap: 12 },
      { id: 'title', component: 'Text', value: 'Welcome back', variant: 'h4' },
      { id: 'subtitle', component: 'Text', value: 'Sign in to your account', variant: 'caption', color: 'text-secondary' },
      { id: 'email-field', component: 'Column', children: ['email-label', 'email-input'], gap: 4 },
      { id: 'email-label', component: 'Text', value: 'Email', variant: 'caption', color: 'text-secondary' },
      { id: 'email-input', component: 'TextField', placeholder: 'Email', value: '' },
      { id: 'pw-field', component: 'Column', children: ['pw-label', 'pw-input'], gap: 4 },
      { id: 'pw-label', component: 'Text', value: 'Password', variant: 'caption', color: 'text-secondary' },
      { id: 'pw-input', component: 'TextField', placeholder: 'Password', type: 'password', value: '' },
      { id: 'signin-btn', component: 'Button', label: 'Sign in', variant: 'primary' },
      { id: 'signup-row', component: 'Row', children: ['no-account', 'signup-link'], gap: 4, justify: 'center' },
      { id: 'no-account', component: 'Text', value: "Don't have an account?", variant: 'caption', color: 'text-secondary' },
      { id: 'signup-link', component: 'Text', value: 'Sign up', variant: 'caption', color: 'accent-purple' },
    ],
    dataModel: {},
  },
]

function openWidget(widget: any) {
  // Navigate to widget editor with the widget data
  router.push({
    path: `/widget/${widget.id}`,
    query: { data: JSON.stringify(widget) }
  })
}
</script>
