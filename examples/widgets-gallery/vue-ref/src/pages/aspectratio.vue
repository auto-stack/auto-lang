<script setup lang="ts">
import { AspectRatio } from '@/components/ui/aspect-ratio'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const aspectratioBasicAutoCode = `div (class: "w-[450px] bg-muted rounded-md overflow-hidden border") {
    aspect-ratio (ratio: 1.777) {
        img (class: "object-cover w-full h-full", src: "https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80", alt: "Photo by Drew Beamer") {}
    }
}
`
const aspectratioBasicVueCode = `<div class="w-[450px] bg-muted rounded-md overflow-hidden border">
  <AspectRatio>
    <img class="object-cover w-full h-full" src="https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80" alt="Photo by Drew Beamer" />
  </AspectRatio>
</div>
`
const aspectratioRatiosAutoCode = `row (gap: "4", class: "flex-wrap") {
    col (class: "w-[150px]", gap: "2") {
        div (class: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 1) {
                div (class: "flex items-center justify-center w-full h-full bg-primary/10") {
                    "1:1"
                }
            }
        }
        text (text: "Square (1:1)", class: "text-sm text-muted-foreground") {}
    }
    col (class: "w-[150px]", gap: "2") {
        div (class: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 1.777) {
                div (class: "flex items-center justify-center w-full h-full bg-primary/10") {
                    "16:9"
                }
            }
        }
        text (class: "text-sm text-muted-foreground", text: "Widescreen (16:9)") {}
    }
    col (gap: "2", class: "w-[150px]") {
        div (class: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 0.5625) {
                div (class: "flex items-center justify-center w-full h-full bg-primary/10") {
                    "9:16"
                }
            }
        }
        text (class: "text-sm text-muted-foreground", text: "Portrait (9:16)") {}
    }
    col (gap: "2", class: "w-[150px]") {
        div (class: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 1.333) {
                div (class: "flex items-center justify-center w-full h-full bg-primary/10") {
                    "4:3"
                }
            }
        }
        text (text: "Standard (4:3)", class: "text-sm text-muted-foreground") {}
    }
}
`
const aspectratioRatiosVueCode = `<div class="flex flex-row gap-4 flex-wrap">
  <div class="flex flex-col gap-2 w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio>
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          1:1
        </div>
      </AspectRatio>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Square (1:1)</span>
  </div>
  <div class="flex flex-col gap-2 w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio>
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          16:9
        </div>
      </AspectRatio>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Widescreen (16:9)</span>
  </div>
  <div class="flex flex-col gap-2 w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio>
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          9:16
        </div>
      </AspectRatio>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Portrait (9:16)</span>
  </div>
  <div class="flex flex-col gap-2 w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio>
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          4:3
        </div>
      </AspectRatio>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Standard (4:3)</span>
  </div>
</div>
`
</script>

<template>
  <ComponentDocPage title="AspectRatio" description="Displays content within a desired ratio." installCommand="npx shadcn-vue@latest add aspect-ratio">
    <DemoSection title="Example" id="aspectratio-basic" :autoCode="aspectratioBasicAutoCode" :vueCode="aspectratioBasicVueCode">
      <template #preview>
        <div class="w-[450px] bg-muted rounded-md overflow-hidden border">
              <AspectRatio>
                <img class="object-cover w-full h-full" src="https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80" alt="Photo by Drew Beamer" />
              </AspectRatio>
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Common Ratios" id="aspectratio-ratios" :autoCode="aspectratioRatiosAutoCode" :vueCode="aspectratioRatiosVueCode">
      <template #preview>
        <div class="flex flex-row gap-4 flex-wrap">
              <div class="flex flex-col gap-2 w-[150px]">
                <div class="bg-muted rounded-md overflow-hidden">
                  <AspectRatio>
                    <div class="flex items-center justify-center w-full h-full bg-primary/10">
                      1:1
                    </div>
                  </AspectRatio>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Square (1:1)</span>
              </div>
              <div class="flex flex-col gap-2 w-[150px]">
                <div class="bg-muted rounded-md overflow-hidden">
                  <AspectRatio>
                    <div class="flex items-center justify-center w-full h-full bg-primary/10">
                      16:9
                    </div>
                  </AspectRatio>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Widescreen (16:9)</span>
              </div>
              <div class="flex flex-col gap-2 w-[150px]">
                <div class="bg-muted rounded-md overflow-hidden">
                  <AspectRatio>
                    <div class="flex items-center justify-center w-full h-full bg-primary/10">
                      9:16
                    </div>
                  </AspectRatio>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Portrait (9:16)</span>
              </div>
              <div class="flex flex-col gap-2 w-[150px]">
                <div class="bg-muted rounded-md overflow-hidden">
                  <AspectRatio>
                    <div class="flex items-center justify-center w-full h-full bg-primary/10">
                      4:3
                    </div>
                  </AspectRatio>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Standard (4:3)</span>
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
            <TableCell>ratio</TableCell>
            <TableCell>number</TableCell>
            <TableCell>1</TableCell>
            <TableCell>-</TableCell>
            <TableCell>The aspect ratio (width/height)</TableCell>
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
