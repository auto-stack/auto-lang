<!-- Login04Page component -->
<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { CheckCircle, Eye, EyeOff, Loader2, ShieldCheck } from 'lucide-vue-next'

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
 <span class="text-muted-foreground">A split-screen login with secure badge.</span>
 <div class="flex flex-col w-full items-center justify-center bg-gradient-to-br from-background to-muted p-6 mt-6">
 <Card>
 <div class="flex flex-col md:flex-row">
 <div class="flex flex-col flex-1 gap-6 p-6 md:p-8">
 <div class="flex flex-col gap-1">
 <h2 class="text-2xl font-bold tracking-tight">Login</h2>
 <span class="text-sm text-muted-foreground">Enter your credentials to access your account</span>
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
 <a href="#" class="text-sm text-primary hover:underline">Forgot?</a>
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
 <span class="text-center text-sm text-muted-foreground">
 Don't have an account? <a href="#" class="text-primary hover:underline">Create an account</a>
 </span>
 </div>
 <div class="hidden md:flex flex-col items-center justify-center bg-muted p-8 gap-4 min-w-[200px]">
 <ShieldCheck class="h-12 w-12 text-foreground mx-auto" />
 <span class="text-foreground font-semibold text-center">Secure Login</span>
 <span class="text-sm text-muted-foreground text-center">Your data is encrypted and protected</span>
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
