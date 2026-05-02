<!-- CardPage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { Avatar } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Form } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select } from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')

const copiedCode = ref<string>('')
const showCardBasicCode = ref<boolean>(true)
const activeCardBasicTab = ref<string>('auto')
const showCardLoginCode = ref<boolean>(true)
const activeCardLoginTab = ref<string>('auto')
const showCardProjectCode = ref<boolean>(true)
const activeCardProjectTab = ref<string>('auto')
const showCardNotificationsCode = ref<boolean>(true)
const activeCardNotificationsTab = ref<string>('auto')

// Copy to clipboard function
async function copyCode(code: string, id: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(code)
    copiedCode.value = id
    setTimeout(() => {
      copiedCode.value = ''
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

const cardBasicAutoCode = `card {
    card-header {
        card-title (text: "Card Title") {}
        card-description (text: "Card Description") {}
    }
    card-content {
        "Card Content"
    }
    card-footer {
        "Card Footer"
    }
}
`
const cardBasicVueCode = `<Card>
  <div class="flex flex-col space-y-1.5 p-6">
    <div class="text-lg font-semibold leading-none tracking-tight">Card Title</div>
    <div class="text-sm text-muted-foreground">Card Description</div>
  </div>
  <div>
    Card Content
  </div>
  <div class="flex items-center p-6 pt-0">
    Card Footer
  </div>
</Card>
`
const cardLoginAutoCode = `card (class: "w-full max-w-sm") {
    card-header {
        card-title (text: "Login to your account") {}
        card-description (text: "Enter your email below to login to your account.") {}
        card-action {
            button (text: "Sign Up", variant: "link") {}
        }
    }
    card-content {
        form {
            col (gap: "6") {
                col (gap: "2") {
                    label (text: "Email", for: "email") {}
                    input (id: "email", type: "email", placeholder: "m@example.com") {}
                }
                col (gap: "2") {
                    row (class: "flex items-center") {
                        label (for: "password", text: "Password") {}
                        link (text: "Forgot your password?", href: "#") {}
                    }
                    input (type: "password", id: "password") {}
                }
            }
        }
    }
    card-footer (class: "flex-col gap-2") {
        button (class: "w-full", text: "Login") {}
        button (text: "Login with Google", class: "w-full", variant: "outline") {}
    }
}
`
const cardLoginVueCode = `<Card>
  <div class="flex flex-col space-y-1.5 p-6">
    <div class="text-lg font-semibold leading-none tracking-tight">Login to your account</div>
    <div class="text-sm text-muted-foreground">Enter your email below to login to your account.</div>
    <div>
      <Button variant="link">Sign Up</Button>
    </div>
  </div>
  <div>
    <Form>
      <div class="flex flex-col gap-6">
        <div class="flex flex-col gap-2">
          <Label for="email">Email</Label>
          <Input type="email" placeholder="m@example.com" />
        </div>
        <div class="flex flex-col gap-2">
          <div class="flex flex-row gap-4 flex items-center">
            <Label for="password">Password</Label>
            <a href="#">Forgot your password?</a>
          </div>
          <Input type="password" />
        </div>
      </div>
    </Form>
  </div>
  <div class="flex items-center p-6 pt-0 flex-col gap-2">
    <Button>Login</Button>
    <Button variant="outline">Login with Google</Button>
  </div>
</Card>
`
const cardProjectAutoCode = `card {
    card-header {
        card-title (text: "Create project") {}
        card-description (text: "Deploy your new project in one-click.") {}
    }
    card-content {
        col (gap: "4") {
            col (gap: "2") {
                label (text: "Name") {}
                input (placeholder: "Name of your project") {}
            }
            col (gap: "2") {
                label (text: "Framework") {}
                select {
                    select-trigger {
                        select-value (placeholder: "Select") {}
                    }
                    select-content {
                        select-item (value: "next", text: "Next.js") {}
                        select-item (value: "sveltekit", text: "SvelteKit") {}
                        select-item (value: "astro", text: "Astro") {}
                        select-item (text: "Nuxt", value: "nuxt") {}
                    }
                }
            }
        }
    }
    card-footer (class: "flex justify-between") {
        button (variant: "outline", text: "Cancel") {}
        button (text: "Deploy") {}
    }
}
`
const cardProjectVueCode = `<Card>
  <div class="flex flex-col space-y-1.5 p-6">
    <div class="text-lg font-semibold leading-none tracking-tight">Create project</div>
    <div class="text-sm text-muted-foreground">Deploy your new project in one-click.</div>
  </div>
  <div>
    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-2">
        <Label>Name</Label>
        <Input placeholder="Name of your project" />
      </div>
      <div class="flex flex-col gap-2">
        <Label>Framework</Label>
        <Select>
          <div>
            <div placeholder="Select" />
          </div>
          <div>
            <div value="next">Next.js</div>
            <div value="sveltekit">SvelteKit</div>
            <div value="astro">Astro</div>
            <div value="nuxt">Nuxt</div>
          </div>
        </Select>
      </div>
    </div>
  </div>
  <div class="flex items-center p-6 pt-0 flex justify-between">
    <Button variant="outline">Cancel</Button>
    <Button>Deploy</Button>
  </div>
</Card>
`
const cardNotificationsAutoCode = `card {
    card-header {
        card-title (text: "Notifications") {}
        card-description (text: "You have 3 unread messages.") {}
    }
    card-content (class: "grid gap-4") {
        col (gap: "4") {
            row (class: "flex items-center justify-between") {
                col (gap: "1") {
                    text (text: "Push Notifications", class: "text-sm font-medium") {}
                    text (text: "Send notifications to device.", class: "text-sm text-muted-foreground") {}
                }
                switch {}
            }
            separator {}
            col (gap: "4") {
                row (class: "flex items-start gap-4") {
                    avatar (size: "sm") {
                        avatar-image (src: "/avatars/01.png", alt: "Avatar") {}
                        avatar-fallback (text: "OM") {}
                    }
                    col (gap: "1", class: "grid gap-1") {
                        text (text: "Your call has been confirmed.", class: "text-sm") {}
                        text (text: "1 hour ago", class: "text-sm text-muted-foreground") {}
                    }
                }
                row (class: "flex items-start gap-4") {
                    avatar (size: "sm") {
                        avatar-image (src: "/avatars/02.png", alt: "Avatar") {}
                        avatar-fallback (text: "JD") {}
                    }
                    col (class: "grid gap-1", gap: "1") {
                        text (class: "text-sm", text: "You have a new message!") {}
                        text (text: "1 hour ago", class: "text-sm text-muted-foreground") {}
                    }
                }
                row (class: "flex items-start gap-4") {
                    avatar (size: "sm") {
                        avatar-image (alt: "Avatar", src: "/avatars/03.png") {}
                        avatar-fallback (text: "WW") {}
                    }
                    col (gap: "1", class: "grid gap-1") {
                        text (class: "text-sm", text: "Your subscription is expiring soon!") {}
                        text (text: "2 hours ago", class: "text-sm text-muted-foreground") {}
                    }
                }
            }
        }
    }
    card-footer {
        button (class: "w-full", variant: "outline", text: "Mark all as read") {}
    }
}
`
const cardNotificationsVueCode = `<Card>
  <div class="flex flex-col space-y-1.5 p-6">
    <div class="text-lg font-semibold leading-none tracking-tight">Notifications</div>
    <div class="text-sm text-muted-foreground">You have 3 unread messages.</div>
  </div>
  <div class="grid gap-4">
    <div class="flex flex-col gap-4">
      <div class="flex flex-row gap-4 flex items-center justify-between">
        <div class="flex flex-col gap-1">
          <span class="text-muted-foreground leading-7 text-sm font-medium">Push Notifications</span>
          <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Send notifications to device.</span>
        </div>
        <div />
      </div>
      <Separator />
      <div class="flex flex-col gap-4">
        <div class="flex flex-row gap-4 flex items-start gap-4">
          <Avatar>
            <div src="/avatars/01.png" alt="Avatar" />
            <div>OM</div>
          </Avatar>
          <div class="flex flex-col gap-1 grid gap-1">
            <span class="text-muted-foreground leading-7 text-sm">Your call has been confirmed.</span>
            <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 hour ago</span>
          </div>
        </div>
        <div class="flex flex-row gap-4 flex items-start gap-4">
          <Avatar>
            <div src="/avatars/02.png" alt="Avatar" />
            <div>JD</div>
          </Avatar>
          <div class="flex flex-col gap-1 grid gap-1">
            <span class="text-muted-foreground leading-7 text-sm">You have a new message!</span>
            <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 hour ago</span>
          </div>
        </div>
        <div class="flex flex-row gap-4 flex items-start gap-4">
          <Avatar>
            <div alt="Avatar" src="/avatars/03.png" />
            <div>WW</div>
          </Avatar>
          <div class="flex flex-col gap-1 grid gap-1">
            <span class="text-muted-foreground leading-7 text-sm">Your subscription is expiring soon!</span>
            <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">2 hours ago</span>
          </div>
        </div>
      </div>
    </div>
  </div>
  <div class="flex items-center p-6 pt-0">
    <Button variant="outline">Mark all as read</Button>
  </div>
</Card>
`
const codeblock1Code = `npx shadcn-vue@latest add card`
watch(activeCardBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeCardLoginTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeCardProjectTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeCardNotificationsTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">Card</h1>
      Displays a card with header, content, and footer.
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Installation</h2>
      <div class="relative rounded-lg border overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 bg-muted border-b">
          <span class="text-xs text-muted-foreground font-medium">bash</span>
          <button
            @click="copyCode(codeblock1Code, 'codeblock1')"
            class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
          >
            <svg v-if="copiedCode !== 'codeblock1'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
            {{ copiedCode === 'codeblock1' ? 'Copied!' : 'Copy' }}
          </button>
        </div>
        <pre class="p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))] overflow-x-auto"><code class="block font-mono !p-0 language-bash">{{ codeblock1Code }}</code></pre>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Basic</h2>
      <!-- Merged CardBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <Card>
              <div class="flex flex-col space-y-1.5 p-6">
                <div class="text-lg font-semibold leading-none tracking-tight">Card Title</div>
                <div class="text-sm text-muted-foreground">Card Description</div>
              </div>
              <div>
                Card Content
              </div>
              <div class="flex items-center p-6 pt-0">
                Card Footer
              </div>
            </Card>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showCardBasicCode = !showCardBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showCardBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showCardBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeCardBasicTab = 'auto'"
                  :class="activeCardBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeCardBasicTab = 'vue'"
                  :class="activeCardBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeCardBasicTab === 'auto' ? cardBasicAutoCode : cardBasicVueCode, 'card-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'card-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'card-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeCardBasicTab === 'auto' ? 'auto' : 'html')">{{ activeCardBasicTab === 'auto' ? cardBasicAutoCode : cardBasicVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Login Form</h2>
      <!-- Merged CardLogin Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <Card>
              <div class="flex flex-col space-y-1.5 p-6">
                <div class="text-lg font-semibold leading-none tracking-tight">Login to your account</div>
                <div class="text-sm text-muted-foreground">Enter your email below to login to your account.</div>
                <div>
                  <Button variant="link">Sign Up</Button>
                </div>
              </div>
              <div>
                <Form>
                  <div class="flex flex-col gap-6">
                    <div class="flex flex-col gap-2">
                      <Label for="email">Email</Label>
                      <Input type="email" placeholder="m@example.com" />
                    </div>
                    <div class="flex flex-col gap-2">
                      <div class="flex flex-row gap-4 flex items-center">
                        <Label for="password">Password</Label>
                        <a href="#">Forgot your password?</a>
                      </div>
                      <Input type="password" />
                    </div>
                  </div>
                </Form>
              </div>
              <div class="flex items-center p-6 pt-0 flex-col gap-2">
                <Button>Login</Button>
                <Button variant="outline">Login with Google</Button>
              </div>
            </Card>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showCardLoginCode = !showCardLoginCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showCardLoginCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showCardLoginCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeCardLoginTab = 'auto'"
                  :class="activeCardLoginTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeCardLoginTab = 'vue'"
                  :class="activeCardLoginTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeCardLoginTab === 'auto' ? cardLoginAutoCode : cardLoginVueCode, 'card-login')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'card-login'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'card-login' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeCardLoginTab === 'auto' ? 'auto' : 'html')">{{ activeCardLoginTab === 'auto' ? cardLoginAutoCode : cardLoginVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Create Project</h2>
      <!-- Merged CardProject Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <Card>
              <div class="flex flex-col space-y-1.5 p-6">
                <div class="text-lg font-semibold leading-none tracking-tight">Create project</div>
                <div class="text-sm text-muted-foreground">Deploy your new project in one-click.</div>
              </div>
              <div>
                <div class="flex flex-col gap-4">
                  <div class="flex flex-col gap-2">
                    <Label>Name</Label>
                    <Input placeholder="Name of your project" />
                  </div>
                  <div class="flex flex-col gap-2">
                    <Label>Framework</Label>
                    <Select>
                      <div>
                        <div placeholder="Select" />
                      </div>
                      <div>
                        <div value="next">Next.js</div>
                        <div value="sveltekit">SvelteKit</div>
                        <div value="astro">Astro</div>
                        <div value="nuxt">Nuxt</div>
                      </div>
                    </Select>
                  </div>
                </div>
              </div>
              <div class="flex items-center p-6 pt-0 flex justify-between">
                <Button variant="outline">Cancel</Button>
                <Button>Deploy</Button>
              </div>
            </Card>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showCardProjectCode = !showCardProjectCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showCardProjectCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showCardProjectCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeCardProjectTab = 'auto'"
                  :class="activeCardProjectTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeCardProjectTab = 'vue'"
                  :class="activeCardProjectTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeCardProjectTab === 'auto' ? cardProjectAutoCode : cardProjectVueCode, 'card-project')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'card-project'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'card-project' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeCardProjectTab === 'auto' ? 'auto' : 'html')">{{ activeCardProjectTab === 'auto' ? cardProjectAutoCode : cardProjectVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Notifications</h2>
      <!-- Merged CardNotifications Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <Card>
              <div class="flex flex-col space-y-1.5 p-6">
                <div class="text-lg font-semibold leading-none tracking-tight">Notifications</div>
                <div class="text-sm text-muted-foreground">You have 3 unread messages.</div>
              </div>
              <div class="grid gap-4">
                <div class="flex flex-col gap-4">
                  <div class="flex flex-row gap-4 flex items-center justify-between">
                    <div class="flex flex-col gap-1">
                      <span class="text-muted-foreground leading-7 text-sm font-medium">Push Notifications</span>
                      <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Send notifications to device.</span>
                    </div>
                    <div />
                  </div>
                  <Separator />
                  <div class="flex flex-col gap-4">
                    <div class="flex flex-row gap-4 flex items-start gap-4">
                      <Avatar>
                        <div src="/avatars/01.png" alt="Avatar" />
                        <div>OM</div>
                      </Avatar>
                      <div class="flex flex-col gap-1 grid gap-1">
                        <span class="text-muted-foreground leading-7 text-sm">Your call has been confirmed.</span>
                        <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 hour ago</span>
                      </div>
                    </div>
                    <div class="flex flex-row gap-4 flex items-start gap-4">
                      <Avatar>
                        <div src="/avatars/02.png" alt="Avatar" />
                        <div>JD</div>
                      </Avatar>
                      <div class="flex flex-col gap-1 grid gap-1">
                        <span class="text-muted-foreground leading-7 text-sm">You have a new message!</span>
                        <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 hour ago</span>
                      </div>
                    </div>
                    <div class="flex flex-row gap-4 flex items-start gap-4">
                      <Avatar>
                        <div alt="Avatar" src="/avatars/03.png" />
                        <div>WW</div>
                      </Avatar>
                      <div class="flex flex-col gap-1 grid gap-1">
                        <span class="text-muted-foreground leading-7 text-sm">Your subscription is expiring soon!</span>
                        <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">2 hours ago</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <div class="flex items-center p-6 pt-0">
                <Button variant="outline">Mark all as read</Button>
              </div>
            </Card>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showCardNotificationsCode = !showCardNotificationsCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showCardNotificationsCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showCardNotificationsCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeCardNotificationsTab = 'auto'"
                  :class="activeCardNotificationsTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeCardNotificationsTab = 'vue'"
                  :class="activeCardNotificationsTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeCardNotificationsTab === 'auto' ? cardNotificationsAutoCode : cardNotificationsVueCode, 'card-notifications')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'card-notifications'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'card-notifications' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeCardNotificationsTab === 'auto' ? 'auto' : 'html')">{{ activeCardNotificationsTab === 'auto' ? cardNotificationsAutoCode : cardNotificationsVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Components</h2>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Component</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>Card</TableCell>
            <TableCell>Root container component</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>CardHeader</TableCell>
            <TableCell>Header section with title, description, and action</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>CardTitle</TableCell>
            <TableCell>Card title text</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>CardDescription</TableCell>
            <TableCell>Card description text</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>CardAction</TableCell>
            <TableCell>Action area in header, like Sign Up button</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>CardContent</TableCell>
            <TableCell>Main content area</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>CardFooter</TableCell>
            <TableCell>Footer section for actions</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  </div>

</template>

<style scoped>
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>


