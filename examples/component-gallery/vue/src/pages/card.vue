<script setup lang="ts">
import { Avatar } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Form } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select } from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

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
</script>

<template>
  <ComponentDocPage title="Card" description="Displays a card with header, content, and footer." installCommand="npx shadcn-vue@latest add card">
    <DemoSection title="Basic" id="card-basic" :autoCode="cardBasicAutoCode" :vueCode="cardBasicVueCode">
      <template #preview>
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
      </template>
    </DemoSection>
    <DemoSection title="Login Form" id="card-login" :autoCode="cardLoginAutoCode" :vueCode="cardLoginVueCode">
      <template #preview>
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
      </template>
    </DemoSection>
    <DemoSection title="Create Project" id="card-project" :autoCode="cardProjectAutoCode" :vueCode="cardProjectVueCode">
      <template #preview>
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
      </template>
    </DemoSection>
    <DemoSection title="Notifications" id="card-notifications" :autoCode="cardNotificationsAutoCode" :vueCode="cardNotificationsVueCode">
      <template #preview>
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
      </template>
    </DemoSection>

  </ComponentDocPage>
</template>

<style scoped>
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>
