<script setup lang="ts">
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const tabsBasicAutoCode = `tabs (class: "w-64", default: "account") {
    tabs-list {
        tabs-trigger (text: "Account", value: "account") {}
        tabs-trigger (value: "password", text: "Password") {}
    }
    tabs-content (value: "account") {
        "Make changes to your account here."
    }
    tabs-content (value: "password") {
        "Change your password here."
    }
}
`
const tabsBasicVueCode = `<Tabs default-value="account" class="w-64">
  <TabsList>
    <TabsTrigger value="account">Account</TabsTrigger>
    <TabsTrigger value="password">Password</TabsTrigger>
  </TabsList>
  <TabsContent value="account">
    Make changes to your account here.
  </TabsContent>
  <TabsContent value="password">
    Change your password here.
  </TabsContent>
</Tabs>
`
const tabsThreeAutoCode = `tabs (default: "music", class: "w-96") {
    tabs-list {
        tabs-trigger (value: "music", text: "Music") {}
        tabs-trigger (value: "podcasts", text: "Podcasts") {}
        tabs-trigger (value: "live", text: "Live") {}
    }
    tabs-content (value: "music") {
        "Explore your music library."
    }
    tabs-content (value: "podcasts") {
        "Discover new podcasts."
    }
    tabs-content (value: "live") {
        "Watch live streams."
    }
}
`
const tabsThreeVueCode = `<Tabs default-value="music" class="w-96">
  <TabsList>
    <TabsTrigger value="music">Music</TabsTrigger>
    <TabsTrigger value="podcasts">Podcasts</TabsTrigger>
    <TabsTrigger value="live">Live</TabsTrigger>
  </TabsList>
  <TabsContent value="music">
    Explore your music library.
  </TabsContent>
  <TabsContent value="podcasts">
    Discover new podcasts.
  </TabsContent>
  <TabsContent value="live">
    Watch live streams.
  </TabsContent>
</Tabs>
`
</script>

<template>
  <ComponentDocPage title="Tabs" description="A set of layered sections of content, known as tab panels, that display one panel of content at a time." installCommand="npx shadcn-vue@latest add tabs">
    <DemoSection title="Simple" id="tabs-basic" :autoCode="tabsBasicAutoCode" :vueCode="tabsBasicVueCode">
      <template #preview>
        <Tabs default-value="account" class="w-64">
          <TabsList>
            <TabsTrigger value="account">Account</TabsTrigger>
            <TabsTrigger value="password">Password</TabsTrigger>
          </TabsList>
          <TabsContent value="account">
            Make changes to your account here.
          </TabsContent>
          <TabsContent value="password">
            Change your password here.
          </TabsContent>
        </Tabs>
      </template>
    </DemoSection>
    <DemoSection title="Three Tabs" id="tabs-three" :autoCode="tabsThreeAutoCode" :vueCode="tabsThreeVueCode">
      <template #preview>
        <Tabs default-value="music" class="w-96">
          <TabsList>
            <TabsTrigger value="music">Music</TabsTrigger>
            <TabsTrigger value="podcasts">Podcasts</TabsTrigger>
            <TabsTrigger value="live">Live</TabsTrigger>
          </TabsList>
          <TabsContent value="music">
            Explore your music library.
          </TabsContent>
          <TabsContent value="podcasts">
            Discover new podcasts.
          </TabsContent>
          <TabsContent value="live">
            Watch live streams.
          </TabsContent>
        </Tabs>
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
            <TableCell>default</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Default active tab value</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>value</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Tab trigger/content value</TableCell>
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
