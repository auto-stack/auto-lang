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
const avatarBasicVueCode = `<div class="flex flex-row gap-4">
  <Avatar>
    <AvatarImage src="https://github.com/shadcn.png" alt="@shadcn" />
  </Avatar>
  <Avatar>
    <AvatarFallback>CN</AvatarFallback>  </Avatar>
</div>
`
</script>

<template>
  <ComponentDocPage title="Avatar" description="An image element with a fallback for representing the user." installCommand="npx shadcn-vue@latest add avatar">
    <DemoSection title="Simple" id="avatar-basic" :autoCode="avatarBasicAutoCode" :vueCode="avatarBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <Avatar>
                <AvatarImage src="https://github.com/shadcn.png" alt="@shadcn" />
              </Avatar>
              <Avatar>
                <AvatarFallback>CN</AvatarFallback>              </Avatar>
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
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>
