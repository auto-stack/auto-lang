<!-- Login02Page component -->
<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
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

async function handleLogin(): Promise<void> {
  error.value = ''
  if (!email.value) { error.value = 'Please enter your email'; return }
  if (!email.value.includes('@')) { error.value = 'Please enter a valid email'; return }
  if (!password.value) { error.value = 'Please enter your password'; return }
  loading.value = true
  await new Promise(r => setTimeout(r, 1500))
  loading.value = false
  successEmail.value = email.value
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
 <span class="text-muted-foreground">A two-column login form with cover image.</span>
 <div class="flex flex-row w-full mt-6">
 <div class="hidden lg:flex lg:w-1/2 bg-card relative items-center justify-center overflow-hidden rounded-l-lg">
 <div class="absolute inset-0 opacity-40 bg-gradient-to-br from-slate-700 via-slate-900 to-black dark:from-slate-800 dark:via-slate-950 dark:to-black" />
 <div class="flex flex-col relative z-10 gap-4 text-center p-12">
 <span class="text-3xl font-bold text-white">Welcome back</span>
 <span class="text-lg text-white/70 max-w-sm">Sign in to access your dashboard and manage your projects.</span>
 </div>
 </div>
 <div class="flex flex-1 items-center justify-center p-6 bg-background border rounded-r-lg lg:rounded-l-none">
 <div class="flex flex-col w-full max-w-sm gap-6">
 <div class="flex flex-col gap-2 text-center">
 <h2 class="text-2xl font-bold tracking-tight">Login to your account</h2>
 <span class="text-sm text-muted-foreground">Enter your email below to login</span>
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
 <a href="#" class="text-sm text-primary hover:underline">Forgot password?</a>
 </div>
 <div class="relative">
 <Input v-model="password" :type="showPassword ? 'text' : 'password'" class="pr-10" />
 <button @click="showPassword = !showPassword" class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
 <Eye v-if="!showPassword" class="h-4 w-4" />
 <EyeOff v-else class="h-4 w-4" />
 </button>
 </div>
 </div>
 <Button class="w-full" :disabled="loading" @click="handleLogin">
 <Loader2 v-if="loading" class="mr-2 h-4 w-4 animate-spin" />
 {{ loading ? 'Logging in...' : 'Login' }}
 </Button>
 </div>
 <div class="flex flex-col gap-2 text-center">
 <span class="text-sm text-muted-foreground">Don't have an account?</span>
 <a href="#" class="text-sm text-primary hover:underline">Sign up</a>
 </div>
 </div>
 </div>
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
