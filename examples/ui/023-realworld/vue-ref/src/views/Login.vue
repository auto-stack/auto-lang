<script setup lang="ts">
// Plan 405 vue-ref Login form (RealWorld style: centered, field-level errors).
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { login } from '../stores/auth'
const router = useRouter()
const email = ref('sarah@vercel.com')
const password = ref('password')
const errors = ref<string[]>([])

function submit() {
  errors.value = []
  const r = login(email.value, password.value)
  if (r.ok) router.push('/')
  else errors.value = [r.error || 'invalid']
}
</script>

<template>
  <div class="max-w-md mx-auto px-4 py-8 text-center">
    <h1 class="text-4xl font-semibold mb-2">Sign in</h1>
    <router-link to="/register" class="text-brand text-sm">Need an account?</router-link>
    <ul v-if="errors.length" class="text-red-500 text-sm my-3 text-left bg-red-50 rounded p-2">
      <li v-for="e in errors" :key="e">{{ e }}</li>
    </ul>
    <form @submit.prevent="submit" class="text-left mt-4 space-y-3">
      <input v-model="email" type="email" placeholder="Email" class="input-field" />
      <input v-model="password" type="password" placeholder="Password" class="input-field" />
      <div class="text-right"><button type="submit" class="btn-brand">Sign in</button></div>
    </form>
  </div>
</template>
