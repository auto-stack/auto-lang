<script setup lang="ts">
// Plan 405 vue-ref App shell: top nav (brand + Home/Sign in/Settings/New Post)
// + router-view outlet. Mirrors the planned app.at routes{...} + outlet.
import { auth, logout } from './stores/auth'
import { useRouter } from 'vue-router'
const router = useRouter()
function doLogout() { logout(); router.push('/') }
</script>

<template>
  <div class="min-h-screen">
    <nav class="border-b border-gray-200">
      <div class="max-w-3xl mx-auto px-4 py-3 flex items-center gap-4">
        <router-link to="/" class="text-brand font-bold text-xl no-underline">conduit</router-link>
        <div class="ml-auto flex items-center gap-4 text-sm">
          <router-link to="/" class="text-gray-600 hover:underline">Home</router-link>
          <template v-if="auth.isLoggedIn">
            <router-link to="/settings" class="text-gray-600 hover:underline">Settings</router-link>
            <a href="#" class="text-gray-600 hover:underline" @click.prevent="doLogout">{{ auth.currentUser?.username }}</a>
          </template>
          <template v-else>
            <router-link to="/login" class="text-gray-600 hover:underline">Sign in</router-link>
            <router-link to="/register" class="text-gray-600 hover:underline">Sign up</router-link>
          </template>
        </div>
      </div>
    </nav>
    <router-view />
  </div>
</template>
