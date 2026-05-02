<script setup lang="ts">
import { Avatar } from '@/components/ui/avatar'
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
const hovercardBasicVueCode = `<div class="flex flex-row gap-4">
  <div>
    <div>
      <Avatar>
        <div src="https://github.com/shadcn.png" />
        <div>SC</div>
      </Avatar>
    </div>
    <div>
      <div class="flex flex-col gap-3 w-80">
        <div class="flex flex-row gap-4">
          <Avatar>
            <div src="https://github.com/shadcn.png" />
            <div>SC</div>
          </Avatar>
          <div class="flex flex-col gap-1">
            <h4>@shadcn</h4>
            Vue component library
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
`
</script>

<template>
  <ComponentDocPage title="HoverCard" description="A card that appears when hovering over an element, useful for displaying additional context." installCommand="npx shadcn-vue@latest add hover-card">
    <DemoSection title="Simple" id="hovercard-basic" :autoCode="hovercardBasicAutoCode" :vueCode="hovercardBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <div>
                <div>
                  <Avatar>
                    <div src="https://github.com/shadcn.png" />
                    <div>SC</div>
                  </Avatar>
                </div>
                <div>
                  <div class="flex flex-col gap-3 w-80">
                    <div class="flex flex-row gap-4">
                      <Avatar>
                        <div src="https://github.com/shadcn.png" />
                        <div>SC</div>
                      </Avatar>
                      <div class="flex flex-col gap-1">
                        <h4>@shadcn</h4>
                        Vue component library
                      </div>
                    </div>
                  </div>
                </div>
              </div>
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
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>
