<script setup lang="ts">
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const avatarBasicAutoCode = `row (gap: "4") {
    avatar (alt: "@shadcn", src: "https://github.com/shadcn.png") {}
    avatar (fallback: "CN") {}
}
`
const avatarBasicVueCode = `<div class="flex gap-4">
  <Avatar>
    <AvatarImage src="https://github.com/shadcn.png" alt="@shadcn" />
    <AvatarFallback>CN</AvatarFallback>
  </Avatar>
  <Avatar>
    <AvatarFallback>JD</AvatarFallback>
  </Avatar>
</div>
`
const avatarSizesAutoCode = `row (gap: "4", class: "items-center") {
    avatar (size: "sm", src: "https://github.com/shadcn.png") {}
    avatar (src: "https://github.com/shadcn.png") {}
    avatar (size: "lg", src: "https://github.com/shadcn.png") {}
}
`
const avatarSizesVueCode = `<div class="flex items-center gap-4">
  <Avatar class="h-8 w-8">
    <AvatarImage src="https://github.com/shadcn.png" />
    <AvatarFallback>SM</AvatarFallback>
  </Avatar>
  <Avatar class="h-10 w-10">
    <AvatarImage src="https://github.com/shadcn.png" />
    <AvatarFallback>MD</AvatarFallback>
  </Avatar>
  <Avatar class="h-14 w-14">
    <AvatarImage src="https://github.com/shadcn.png" />
    <AvatarFallback>LG</AvatarFallback>
  </Avatar>
</div>
`
const avatarStackedAutoCode = `row (class: "-space-x-2") {
    avatar (src: "https://github.com/shadcn.png") {}
    avatar (src: "https://github.com/leerob.png") {}
    avatar (src: "https://github.com/evilrabbit.png") {}
}
`
const avatarStackedVueCode = `<div class="flex -space-x-2">
  <Avatar class="border-2 border-background">
    <AvatarImage src="https://github.com/shadcn.png" />
    <AvatarFallback>CN</AvatarFallback>
  </Avatar>
  <Avatar class="border-2 border-background">
    <AvatarImage src="https://github.com/leerob.png" />
    <AvatarFallback>LR</AvatarFallback>
  </Avatar>
  <Avatar class="border-2 border-background">
    <AvatarImage src="https://github.com/evilrabbit.png" />
    <AvatarFallback>ER</AvatarFallback>
  </Avatar>
</div>
`
</script>

<template>
  <ComponentDocPage title="Avatar" description="An image element with a fallback for representing the user." installCommand="npx shadcn-vue@latest add avatar">
    <DemoSection title="Simple" id="avatar-basic" :autoCode="avatarBasicAutoCode" :vueCode="avatarBasicVueCode">
      <template #preview>
        <div class="flex gap-4">
          <Avatar>
            <AvatarImage src="https://github.com/shadcn.png" alt="@shadcn" />
            <AvatarFallback>CN</AvatarFallback>
          </Avatar>
          <Avatar>
            <AvatarFallback>JD</AvatarFallback>
          </Avatar>
        </div>
      </template>
    </DemoSection>
    <DemoSection title="Sizes" id="avatar-sizes" :autoCode="avatarSizesAutoCode" :vueCode="avatarSizesVueCode">
      <template #preview>
        <div class="flex items-center gap-4">
          <Avatar class="h-8 w-8">
            <AvatarImage src="https://github.com/shadcn.png" />
            <AvatarFallback>SM</AvatarFallback>
          </Avatar>
          <Avatar class="h-10 w-10">
            <AvatarImage src="https://github.com/shadcn.png" />
            <AvatarFallback>MD</AvatarFallback>
          </Avatar>
          <Avatar class="h-14 w-14">
            <AvatarImage src="https://github.com/shadcn.png" />
            <AvatarFallback>LG</AvatarFallback>
          </Avatar>
        </div>
      </template>
    </DemoSection>
    <DemoSection title="Stacked" id="avatar-stacked" :autoCode="avatarStackedAutoCode" :vueCode="avatarStackedVueCode">
      <template #preview>
        <div class="flex -space-x-2">
          <Avatar class="border-2 border-background">
            <AvatarImage src="https://github.com/shadcn.png" />
            <AvatarFallback>CN</AvatarFallback>
          </Avatar>
          <Avatar class="border-2 border-background">
            <AvatarImage src="https://github.com/leerob.png" />
            <AvatarFallback>LR</AvatarFallback>
          </Avatar>
          <Avatar class="border-2 border-background">
            <AvatarImage src="https://github.com/evilrabbit.png" />
            <AvatarFallback>ER</AvatarFallback>
          </Avatar>
        </div>
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
            <TableCell>src</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Image source URL</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>alt</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Alt text for accessibility</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>fallback</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Fallback text when image fails</TableCell>
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
