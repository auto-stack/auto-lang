import { reactive } from 'vue'
import type { User } from '../api/mock'
import { seedUsers } from '../api/mock'

// Plan 405 vue-ref auth store. In-memory + localStorage persistence
// (RealWorld standard: token in localStorage). Mock login/register validate
// against seedUsers; no real backend. This mirrors the planned AuthStore.at.
export const auth = reactive({
  currentUser: null as User | null,
  get isLoggedIn() { return this.currentUser !== null },
})

function persist() {
  if (auth.currentUser) localStorage.setItem('rw_user', JSON.stringify(auth.currentUser))
  else localStorage.removeItem('rw_user')
}

export function login(email: string, password: string): { ok: boolean; error?: string } {
  const u = seedUsers.find(x => x.email === email)
  if (!u) return { ok: false, error: 'email or password is invalid' }
  // mock: any password works for a known email
  auth.currentUser = { ...u }
  persist()
  return { ok: true }
}

export function register(username: string, email: string, password: string): { ok: boolean; error?: string } {
  if (seedUsers.some(x => x.email === email)) return { ok: false, error: 'email already exists' }
  const u: User = { id: Date.now(), email, username, bio: '', image: `https://i.pravatar.cc/100?u=${username}`, token: `mock-token-${username}` }
  auth.currentUser = u
  persist()
  return { ok: true }
}

export function logout() {
  auth.currentUser = null
  persist()
}

export function updateProfile(patch: Partial<User>) {
  if (!auth.currentUser) return
  auth.currentUser = { ...auth.currentUser, ...patch }
  persist()
}
