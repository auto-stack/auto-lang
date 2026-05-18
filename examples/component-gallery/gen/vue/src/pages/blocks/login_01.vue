<!-- Login01Page component -->
<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { CheckCircle, Eye, EyeOff, Loader2 } from 'lucide-vue-next'

const email = ref<string>('')
const password = ref<string>('')
const loading = ref<boolean>(false)
const error = ref<string>('')
const showPassword = ref<boolean>(false)
const successOpen = ref<boolean>(false)
const successEmail = ref<string>('')
const socialLoading = ref<string>('')

async function handleLogin(): Promise<void> {
  error.value = ''
  if (!email.value) { error.value = 'Please enter your email'; return }
  if (!email.value.includes('@')) { error.value = 'Please enter a valid email'; return }
  if (!password.value) { error.value = 'Please enter your password'; return }
  if (password.value.length < 4) { error.value = 'Password must be at least 4 characters'; return }
  loading.value = true
  await new Promise(r => setTimeout(r, 1500))
  loading.value = false
  successEmail.value = email.value
  successOpen.value = true
}

async function handleSocial(provider: string): Promise<void> {
  socialLoading.value = provider
  await new Promise(r => setTimeout(r, 1500))
  socialLoading.value = ''
  successEmail.value = email.value || provider
  successOpen.value = true
}

function closeSuccess() {
  successOpen.value = false
  email.value = ''
  password.value = ''
  error.value = ''
}
</script>

<template>
 <div class="flex flex-col pb-8">
 <h1 class="text-4xl font-bold tracking-tight">Login</h1>
 <span class="text-muted-foreground">A clean login form with email and password.</span>
 <div class="flex flex-col w-full items-center justify-center p-6 mt-6">
 <Card>
 <div class="flex flex-col gap-6 p-6">
 <div class="flex flex-col gap-2 text-center">
 <h2 class="text-2xl font-bold tracking-tight">Login to your account</h2>
 <span class="text-sm text-muted-foreground">Enter your email below to login to your account</span>
 </div>
 <div v-if="error" class="rounded-md bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-600 dark:text-red-400">{{ error }}</div>
 <div class="flex flex-col gap-4">
 <div class="flex flex-col gap-2">
 <label class="text-sm font-medium leading-none" for="email">Email</label>
 <Input v-model="email" type="email" placeholder="m@example.com" />
 </div>
 <div class="flex flex-col gap-2">
 <div class="flex flex-row items-center justify-between">
 <label class="text-sm font-medium leading-none" for="password">Password</label>
 <a href="#" class="text-sm text-primary hover:underline">Forgot your password?</a>
 </div>
 <div class="relative">
 <Input v-model="password" :type="showPassword ? 'text' : 'password'" class="pr-10" />
 <button @click="showPassword = !showPassword" class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
 <Eye v-if="!showPassword" class="h-4 w-4" />
 <EyeOff v-else class="h-4 w-4" />
 </button>
 </div>
 </div>
 <Button variant="default" class="w-full" :disabled="loading" @click="handleLogin">
 <Loader2 v-if="loading" class="mr-2 h-4 w-4 animate-spin" />
 {{ loading ? 'Logging in...' : 'Login' }}
 </Button>
 </div>
 <div class="flex flex-col gap-4">
 <div class="relative">
 <div class="absolute inset-0 flex items-center">
 <span class="w-full border-t border-border" />
 </div>
 <div class="relative flex justify-center text-xs uppercase">
 <span class="bg-background px-2 text-muted-foreground">Or continue with</span>
 </div>
 </div>
 <Button variant="outline" class="w-full" :disabled="!!socialLoading" @click="handleSocial('github')">
 <Loader2 v-if="socialLoading === 'github'" class="mr-2 h-4 w-4 animate-spin" />
 {{ socialLoading === 'github' ? 'Connecting...' : 'GitHub' }}
 </Button>
 </div>
 <span class="text-center text-sm text-muted-foreground">
 Don't have an account? <a href="#" class="text-primary hover:underline">Sign up</a>
 </span>
 </div>
 </Card>
 </div>

 <!-- Success Dialog -->
 <Dialog v-model:open="successOpen">
 <DialogContent class="sm:max-w-[400px]">
 <DialogHeader>
 <div class="flex flex-col items-center gap-4 py-4">
 <CheckCircle class="h-12 w-12 text-green-600 dark:text-green-400" />
 <DialogTitle class="text-center">Login Successful</DialogTitle>
 <DialogDescription class="text-center">Welcome back! You have signed in as <strong>{{ successEmail }}</strong></DialogDescription>
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
