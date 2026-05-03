<script setup lang="ts">
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
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
  <CardHeader>
    <CardTitle>Card Title</CardTitle>
    <CardDescription>Card Description</CardDescription>
  </CardHeader>
  <CardContent>
    <p>Card Content</p>
  </CardContent>
  <CardFooter>
    <p>Card Footer</p>
  </CardFooter>
</Card>
`
const cardLoginAutoCode = `card (class: "w-full max-w-sm") {
    card-header {
        card-title (text: "Login to your account") {}
        card-description (text: "Enter your email below to login to your account.") {}
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
const cardLoginVueCode = `<Card class="w-full max-w-sm">
  <CardHeader>
    <CardTitle>Login to your account</CardTitle>
    <CardDescription>Enter your email below to login to your account.</CardDescription>
  </CardHeader>
  <CardContent>
    <form>
      <div class="grid w-full items-center gap-4">
        <div class="flex flex-col space-y-1.5">
          <Label for="email">Email</Label>
          <Input id="email" type="email" placeholder="m@example.com" />
        </div>
        <div class="flex flex-col space-y-1.5">
          <div class="flex items-center">
            <Label for="password">Password</Label>
            <a href="#" class="ml-auto inline-block text-sm underline">
              Forgot your password?
            </a>
          </div>
          <Input id="password" type="password" />
        </div>
      </div>
    </form>
  </CardContent>
  <CardFooter class="flex flex-col gap-2">
    <Button class="w-full">Login</Button>
    <Button variant="outline" class="w-full">Login with Google</Button>
  </CardFooter>
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
  <CardHeader>
    <CardTitle>Create project</CardTitle>
    <CardDescription>Deploy your new project in one-click.</CardDescription>
  </CardHeader>
  <CardContent>
    <form class="flex flex-col gap-4">
      <div class="flex flex-col gap-2">
        <Label for="name">Name</Label>
        <Input id="name" placeholder="Name of your project" />
      </div>
      <div class="flex flex-col gap-2">
        <Label for="framework">Framework</Label>
        <Select>
          <SelectTrigger>
            <SelectValue placeholder="Select" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="next">Next.js</SelectItem>
            <SelectItem value="sveltekit">SvelteKit</SelectItem>
            <SelectItem value="astro">Astro</SelectItem>
            <SelectItem value="nuxt">Nuxt</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </form>
  </CardContent>
  <CardFooter class="flex justify-between">
    <Button variant="outline">Cancel</Button>
    <Button>Deploy</Button>
  </CardFooter>
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
const cardNotificationsVueCode = `<Card class="w-full max-w-sm">
  <CardHeader>
    <CardTitle>Notifications</CardTitle>
    <CardDescription>You have 3 unread messages.</CardDescription>
  </CardHeader>
  <CardContent class="grid gap-4">
    <div class="flex items-center space-x-4 rounded-md border p-4">
      <div class="flex-1 space-y-1">
        <p class="text-sm font-medium leading-none">Push Notifications</p>
        <p class="text-sm text-muted-foreground">Send notifications to device.</p>
      </div>
      <Switch />
    </div>
    <div>
      <div class="mb-4 grid grid-cols-[25px_1fr] items-start pb-4 last:mb-0 last:pb-0">
        <span class="flex h-2 w-2 translate-y-1 rounded-full bg-sky-500" />
        <div class="space-y-1">
          <p class="text-sm font-medium leading-none">Your call has been confirmed.</p>
          <p class="text-sm text-muted-foreground">1 hour ago</p>
        </div>
      </div>
      <div class="mb-4 grid grid-cols-[25px_1fr] items-start pb-4 last:mb-0 last:pb-0">
        <span class="flex h-2 w-2 translate-y-1 rounded-full bg-sky-500" />
        <div class="space-y-1">
          <p class="text-sm font-medium leading-none">You have a new message!</p>
          <p class="text-sm text-muted-foreground">1 hour ago</p>
        </div>
      </div>
      <div class="mb-4 grid grid-cols-[25px_1fr] items-start pb-4 last:mb-0 last:pb-0">
        <span class="flex h-2 w-2 translate-y-1 rounded-full bg-sky-500" />
        <div class="space-y-1">
          <p class="text-sm font-medium leading-none">Your subscription is expiring soon!</p>
          <p class="text-sm text-muted-foreground">2 hours ago</p>
        </div>
      </div>
    </div>
  </CardContent>
  <CardFooter>
    <Button variant="outline" class="w-full">
      <CheckIcon /> Mark all as read
    </Button>
  </CardFooter>
</Card>
`
</script>

<template>
  <ComponentDocPage title="Card" description="Displays a card with header, content, and footer." installCommand="npx shadcn-vue@latest add card">
    <DemoSection title="Basic" id="card-basic" :autoCode="cardBasicAutoCode" :vueCode="cardBasicVueCode">
      <template #preview>
        <Card class="w-full max-w-sm">
          <CardHeader>
            <CardTitle>Card Title</CardTitle>
            <CardDescription>Card Description</CardDescription>
          </CardHeader>
          <CardContent>
            <p>Card Content</p>
          </CardContent>
          <CardFooter>
            <p>Card Footer</p>
          </CardFooter>
        </Card>
      </template>
    </DemoSection>
    <DemoSection title="Login Form" id="card-login" :autoCode="cardLoginAutoCode" :vueCode="cardLoginVueCode">
      <template #preview>
        <Card class="w-full max-w-sm">
          <CardHeader>
            <CardTitle>Login to your account</CardTitle>
            <CardDescription>Enter your email below to login to your account.</CardDescription>
          </CardHeader>
          <CardContent>
            <form>
              <div class="grid w-full items-center gap-4">
                <div class="flex flex-col space-y-1.5">
                  <Label for="email">Email</Label>
                  <Input id="email" type="email" placeholder="m@example.com" />
                </div>
                <div class="flex flex-col space-y-1.5">
                  <div class="flex items-center">
                    <Label for="password">Password</Label>
                    <a href="#" class="ml-auto inline-block text-sm underline">Forgot your password?</a>
                  </div>
                  <Input id="password" type="password" />
                </div>
              </div>
            </form>
          </CardContent>
          <CardFooter class="flex flex-col gap-2">
            <Button class="w-full">Login</Button>
            <Button variant="outline" class="w-full">Login with Google</Button>
          </CardFooter>
        </Card>
      </template>
    </DemoSection>
    <DemoSection title="Create Project" id="card-project" :autoCode="cardProjectAutoCode" :vueCode="cardProjectVueCode">
      <template #preview>
        <Card class="w-full max-w-sm">
          <CardHeader>
            <CardTitle>Create project</CardTitle>
            <CardDescription>Deploy your new project in one-click.</CardDescription>
          </CardHeader>
          <CardContent>
            <form class="flex flex-col gap-4">
              <div class="flex flex-col gap-2">
                <Label for="name">Name</Label>
                <Input id="name" placeholder="Name of your project" />
              </div>
              <div class="flex flex-col gap-2">
                <Label for="framework">Framework</Label>
                <Select>
                  <SelectTrigger>
                    <SelectValue placeholder="Select" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="next">Next.js</SelectItem>
                    <SelectItem value="sveltekit">SvelteKit</SelectItem>
                    <SelectItem value="astro">Astro</SelectItem>
                    <SelectItem value="nuxt">Nuxt</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </form>
          </CardContent>
          <CardFooter class="flex justify-between">
            <Button variant="outline">Cancel</Button>
            <Button>Deploy</Button>
          </CardFooter>
        </Card>
      </template>
    </DemoSection>
    <DemoSection title="Notifications" id="card-notifications" :autoCode="cardNotificationsAutoCode" :vueCode="cardNotificationsVueCode">
      <template #preview>
        <Card class="w-full max-w-sm">
          <CardHeader>
            <CardTitle>Notifications</CardTitle>
            <CardDescription>You have 3 unread messages.</CardDescription>
          </CardHeader>
          <CardContent class="grid gap-4">
            <div class="flex items-center space-x-4 rounded-md border p-4">
              <div class="flex-1 space-y-1">
                <p class="text-sm font-medium leading-none">Push Notifications</p>
                <p class="text-sm text-muted-foreground">Send notifications to device.</p>
              </div>
              <Switch />
            </div>
            <div class="space-y-3">
              <div class="flex items-start gap-4">
                <Avatar class="h-9 w-9">
                  <AvatarImage src="https://github.com/shadcn.png" alt="Avatar" />
                  <AvatarFallback>OM</AvatarFallback>
                </Avatar>
                <div class="space-y-1">
                  <p class="text-sm font-medium leading-none">Your call has been confirmed.</p>
                  <p class="text-sm text-muted-foreground">1 hour ago</p>
                </div>
              </div>
              <Separator />
              <div class="flex items-start gap-4">
                <Avatar class="h-9 w-9">
                  <AvatarImage src="https://github.com/evilrabbit.png" alt="Avatar" />
                  <AvatarFallback>JD</AvatarFallback>
                </Avatar>
                <div class="space-y-1">
                  <p class="text-sm font-medium leading-none">You have a new message!</p>
                  <p class="text-sm text-muted-foreground">1 hour ago</p>
                </div>
              </div>
              <Separator />
              <div class="flex items-start gap-4">
                <Avatar class="h-9 w-9">
                  <AvatarImage src="https://github.com/leerob.png" alt="Avatar" />
                  <AvatarFallback>WW</AvatarFallback>
                </Avatar>
                <div class="space-y-1">
                  <p class="text-sm font-medium leading-none">Your subscription is expiring soon!</p>
                  <p class="text-sm text-muted-foreground">2 hours ago</p>
                </div>
              </div>
            </div>
          </CardContent>
          <CardFooter>
            <Button variant="outline" class="w-full">Mark all as read</Button>
          </CardFooter>
        </Card>
      </template>
    </DemoSection>
  </ComponentDocPage>
</template>

<style scoped>
pre[class*="language-"] {
  margin: 0;
}
</style>
