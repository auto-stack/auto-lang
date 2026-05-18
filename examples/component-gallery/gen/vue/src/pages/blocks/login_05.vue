<!-- Login05Page component -->
<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { CheckCircle, Loader2, Mail } from 'lucide-vue-next'

const email = ref<string>('')
const loading = ref<boolean>(false)
const error = ref<string>('')
const successOpen = ref<boolean>(false)
const successEmail = ref<string>('')

async function handleSendLink(): Promise<void> {
  error.value = ''
  if (!email.value) { error.value = 'Please enter your email'; return }
  if (!email.value.includes('@')) { error.value = 'Please enter a valid email'; return }
  loading.value = true
  await new Promise(r => setTimeout(r, 1500))
  loading.value = false
  successEmail.value = email.value
  successOpen.value = true
}

function closeSuccess() {
  successOpen.value = false
  email.value = ''
  error.value = ''
}
</script>

<template>
 <div class="flex flex-col pb-8">
 <h1 class="text-4xl font-bold tracking-tight">Login</h1>
 <span class="text-muted-foreground">Email-only login with magic link.</span>
 <div class="flex flex-col w-full items-center justify-center p-6 mt-6">
 <Card>
 <div class="flex flex-col gap-6 p-6 max-w-sm w-full">
 <div class="flex flex-col gap-2 text-center">
 <div class="flex justify-center">
 <div class="rounded-full bg-muted p-3"><Mail class="h-6 w-6" /></div>
 </div>
 <h2 class="text-2xl font-bold tracking-tight">Sign in with email</h2>
 <span class="text-sm text-muted-foreground">Enter your email and we'll send you a sign-in link.</span>
 </div>
 <div v-if="error" class="rounded-md bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-600 dark:text-red-400">{{ error }}</div>
 <div class="flex flex-col gap-4">
 <div class="flex flex-col gap-2">
 <label class="text-sm font-medium leading-none" for="email">Email</label>
 <Input v-model="email" type="email" placeholder="m@example.com" />
 </div>
 <Button class="w-full" :disabled="loading" @click="handleSendLink">
 <Loader2 v-if="loading" class="mr-2 h-4 w-4 animate-spin" />
 {{ loading ? 'Sending...' : 'Send Magic Link' }}
 </Button>
 </div>
 </div>
 </Card>
 </div>

 <!-- Success Dialog -->
 <Dialog v-model:open="successOpen">
 <DialogContent class="sm:max-w-[400px]">
 <DialogHeader>
 <div class="flex flex-col items-center gap-4 py-4">
 <CheckCircle class="h-12 w-12 text-green-600 dark:text-green-400" />
 <DialogTitle class="text-center">Magic Link Sent</DialogTitle>
 <DialogDescription class="text-center">A sign-in link has been sent to <strong>{{ successEmail }}</strong>. Please check your inbox.</DialogDescription>
 </div>
 </DialogHeader>
 <DialogFooter>
 <Button class="w-full" @click="closeSuccess">OK</Button>
 </DialogFooter>
 </DialogContent>
 </Dialog>
 </div>

</template>

<style scoped>
/* Component styles */

</style>
