<script setup lang="ts">
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { HoverCard, HoverCardContent, HoverCardTrigger } from '@/components/ui/hover-card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const hovercardBasicAutoCode = `row (gap: "4") {
    hovercard {
        hovercard-trigger {
            avatar {
                avatar-image (src: "https://github.com/shadcn.png") {}
                avatar-fallback (text: "SC") {}
            }
        }
        hovercard-content {
            col (gap: "3", class: "w-80") {
                row (gap: "4") {
                    avatar {
                        avatar-image (src: "https://github.com/shadcn.png") {}
                        avatar-fallback (text: "SC") {}
                    }
                    col (gap: "1") {
                        h4 (text: "@shadcn") {}
                        "Vue component library"
                    }
                }
            }
        }
    }
}
`
const hovercardBasicVueCode = `<HoverCard>
  <HoverCardTrigger as-child>
    <Avatar class="h-10 w-10 cursor-pointer">
      <AvatarImage src="https://github.com/shadcn.png" alt="@shadcn" />
      <AvatarFallback>SC</AvatarFallback>
    </Avatar>
  </HoverCardTrigger>
  <HoverCardContent class="w-80">
    <div class="flex justify-between space-x-4">
      <Avatar>
        <AvatarImage src="https://github.com/shadcn.png" />
        <AvatarFallback>SC</AvatarFallback>
      </Avatar>
      <div class="space-y-1">
        <h4 class="text-sm font-semibold">@shadcn</h4>
        <p class="text-sm">Vue component library</p>
      </div>
    </div>
  </HoverCardContent>
</HoverCard>
`

const hovercardButtonAutoCode = `hovercard {
    hovercard-trigger (as: "button") {
        button (variant: "outline", text: "@shadcn") {}
    }
    hovercard-content (class: "w-80") {
        col (gap: "2") {
            h4 (text: "@shadcn") {}
            "The Vue component library built on top of Radix UI."
        }
    }
}
`
const hovercardButtonVueCode = `<HoverCard>
  <HoverCardTrigger as-child>
    <Button variant="outline">@shadcn</Button>
  </HoverCardTrigger>
  <HoverCardContent class="w-80">
    <div class="flex flex-col gap-2">
      <h4 class="text-sm font-semibold">@shadcn</h4>
      <p class="text-sm">The Vue component library built on top of Radix UI.</p>
    </div>
  </HoverCardContent>
</HoverCard>
`

const hovercardDelayAutoCode = `hovercard (open-delay: "0") {
    hovercard-trigger (as: "button") {
        button (variant: "outline", text: "Instant") {}
    }
    hovercard-content (class: "w-60") {
        "This card appears immediately with no delay."
    }
}
`
const hovercardDelayVueCode = `<HoverCard :open-delay="0">
  <HoverCardTrigger as-child>
    <Button variant="outline">Instant</Button>
  </HoverCardTrigger>
  <HoverCardContent class="w-60">
    <p>This card appears immediately with no delay.</p>
  </HoverCardContent>
</HoverCard>
`
</script>

<template>
  <ComponentDocPage title="HoverCard" description="A card that appears when hovering over an element, useful for displaying additional context." installCommand="npx shadcn-vue@latest add hover-card">
    <DemoSection title="Simple" id="hovercard-basic" :autoCode="hovercardBasicAutoCode" :vueCode="hovercardBasicVueCode">
      <template #preview>
        <HoverCard>
          <HoverCardTrigger as-child>
            <Avatar class="h-10 w-10 cursor-pointer">
              <AvatarImage src="https://github.com/shadcn.png" alt="@shadcn" />
              <AvatarFallback>SC</AvatarFallback>
            </Avatar>
          </HoverCardTrigger>
          <HoverCardContent class="w-80">
            <div class="flex justify-between space-x-4">
              <Avatar>
                <AvatarImage src="https://github.com/shadcn.png" />
                <AvatarFallback>SC</AvatarFallback>
              </Avatar>
              <div class="space-y-1">
                <h4 class="text-sm font-semibold">@shadcn</h4>
                <p class="text-sm">Vue component library</p>
              </div>
            </div>
          </HoverCardContent>
        </HoverCard>
      </template>
    </DemoSection>

    <DemoSection title="With Button" id="hovercard-button" :autoCode="hovercardButtonAutoCode" :vueCode="hovercardButtonVueCode">
      <template #preview>
        <HoverCard>
          <HoverCardTrigger as-child>
            <Button variant="outline">@shadcn</Button>
          </HoverCardTrigger>
          <HoverCardContent class="w-80">
            <div class="flex flex-col gap-2">
              <h4 class="text-sm font-semibold">@shadcn</h4>
              <p class="text-sm">The Vue component library built on top of Radix UI.</p>
            </div>
          </HoverCardContent>
        </HoverCard>
      </template>
    </DemoSection>

    <DemoSection title="No Delay" id="hovercard-delay" :autoCode="hovercardDelayAutoCode" :vueCode="hovercardDelayVueCode">
      <template #preview>
        <HoverCard :open-delay="0">
          <HoverCardTrigger as-child>
            <Button variant="outline">Instant</Button>
          </HoverCardTrigger>
          <HoverCardContent class="w-60">
            <p>This card appears immediately with no delay.</p>
          </HoverCardContent>
        </HoverCard>
      </template>
    </DemoSection>

    <template #properties>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Property</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Default</TableHead>
            <TableHead>Values</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>open-delay</TableCell>
            <TableCell>number</TableCell>
            <TableCell>700</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Delay before opening (ms)</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>close-delay</TableCell>
            <TableCell>number</TableCell>
            <TableCell>300</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Delay before closing (ms)</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>open</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Controls the open state</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>
  </ComponentDocPage>
</template>

<style scoped>
pre[class*="language-"] {
  margin: 0;
}
</style>
