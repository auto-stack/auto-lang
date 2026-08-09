<script setup lang="ts">
// Plan 405 vue-ref Settings: edit profile + logout.
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { auth, updateProfile, logout } from '../stores/auth'
const router = useRouter()
const image = ref(auth.currentUser?.image ?? '')
const username = ref(auth.currentUser?.username ?? '')
const bio = ref(auth.currentUser?.bio ?? '')
const email = ref(auth.currentUser?.email ?? '')

function save() {
  updateProfile({ image: image.value, username: username.value, bio: bio.value, email: email.value })
  router.push('/')
}
function doLogout() { logout(); router.push('/') }
</script>

<template>
  <div class="max-w-md mx-auto px-4 py-8 text-center">
    <h1 class="text-3xl font-semibold mb-4">Your Settings</h1>
    <form @submit.prevent="save" class="text-left space-y-3">
      <input v-model="image" placeholder="URL of profile picture" class="input-field" />
      <input v-model="username" placeholder="Username" class="input-field" />
      <textarea v-model="bio" rows="4" placeholder="Short bio about you" class="input-field"></textarea>
      <input v-model="email" type="email" placeholder="Email" class="input-field" />
      <input type="password" placeholder="New Password (leave blank to keep)" class="input-field" />
      <div class="flex justify-between">
        <button type="button" class="text-red-500 border border-red-500 px-3 py-1 text-sm rounded hover:bg-red-50" @click="doLogout">Or click here to logout.</button>
        <button type="submit" class="btn-brand">Update Settings</button>
      </div>
    </form>
  </div>
</template>
