<script setup lang="ts">
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const accordionBasicAutoCode = `accordion (class: "w-full", collapsible: "true", type: "single") {
    accordion-item (value: "item-1") {
        accordion-trigger (text: "Is it accessible?") {}
        accordion-content {
            "Yes. It adheres to the WAI-ARIA design pattern."
        }
    }
    accordion-item (value: "item-2") {
        accordion-trigger (text: "Is it styled?") {}
        accordion-content {
            "Yes. It comes with default styles that match the other components."
        }
    }
    accordion-item (value: "item-3") {
        accordion-trigger (text: "Is it animated?") {}
        accordion-content {
            "Yes. It's animated by default, but you can disable it if you prefer."
        }
    }
}
`
const accordionBasicVueCode = `<Accordion type="single" :collapsible="true">
  <AccordionItem value="item-1">
    <AccordionTrigger>Is it accessible?</AccordionTrigger>
    <AccordionContent>
      Yes. It adheres to the WAI-ARIA design pattern.
    </AccordionContent>
  </AccordionItem>
  <AccordionItem value="item-2">
    <AccordionTrigger>Is it styled?</AccordionTrigger>
    <AccordionContent>
      Yes. It comes with default styles that match the other components.
    </AccordionContent>
  </AccordionItem>
  <AccordionItem value="item-3">
    <AccordionTrigger>Is it animated?</AccordionTrigger>
    <AccordionContent>
      Yes. It's animated by default, but you can disable it if you prefer.
    </AccordionContent>
  </AccordionItem>
</Accordion>
`
const accordionMultipleAutoCode = `accordion (type: "multiple", class: "w-full") {
    accordion-item (value: "item-1") {
        accordion-trigger (text: "Product Information") {}
        accordion-content {
            "Our flagship product combines cutting-edge technology with sleek design."
        }
    }
    accordion-item (value: "item-2") {
        accordion-trigger (text: "Shipping Details") {}
        accordion-content {
            "We offer worldwide shipping through trusted courier partners."
        }
    }
    accordion-item (value: "item-3") {
        accordion-trigger (text: "Return Policy") {}
        accordion-content {
            "We stand behind our products with a comprehensive 30-day return policy."
        }
    }
}
`
const accordionMultipleVueCode = `<Accordion type="multiple">
  <AccordionItem value="item-1">
    <AccordionTrigger>Product Information</AccordionTrigger>
    <AccordionContent>
      Our flagship product combines cutting-edge technology with sleek design.
    </AccordionContent>
  </AccordionItem>
  <AccordionItem value="item-2">
    <AccordionTrigger>Shipping Details</AccordionTrigger>
    <AccordionContent>
      We offer worldwide shipping through trusted courier partners.
    </AccordionContent>
  </AccordionItem>
  <AccordionItem value="item-3">
    <AccordionTrigger>Return Policy</AccordionTrigger>
    <AccordionContent>
      We stand behind our products with a comprehensive 30-day return policy.
    </AccordionContent>
  </AccordionItem>
</Accordion>
`
</script>

<template>
  <ComponentDocPage title="Accordion" description="A vertically stacked set of interactive headings that each reveal a section of content." installCommand="npx shadcn-vue@latest add accordion">
    <DemoSection title="Single" id="accordion-single" :autoCode="accordionBasicAutoCode" :vueCode="accordionBasicVueCode">
      <template #preview>
        <Accordion type="single" :collapsible="true">
          <AccordionItem value="item-1">
            <AccordionTrigger>Is it accessible?</AccordionTrigger>
            <AccordionContent>
              Yes. It adheres to the WAI-ARIA design pattern.
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="item-2">
            <AccordionTrigger>Is it styled?</AccordionTrigger>
            <AccordionContent>
              Yes. It comes with default styles that match the other components.
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="item-3">
            <AccordionTrigger>Is it animated?</AccordionTrigger>
            <AccordionContent>
              Yes. It's animated by default, but you can disable it if you prefer.
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </template>
    </DemoSection>
    <DemoSection title="Multiple" id="accordion-multiple" :autoCode="accordionMultipleAutoCode" :vueCode="accordionMultipleVueCode">
      <template #preview>
        <Accordion type="multiple">
          <AccordionItem value="item-1">
            <AccordionTrigger>Product Information</AccordionTrigger>
            <AccordionContent>
              Our flagship product combines cutting-edge technology with sleek design.
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="item-2">
            <AccordionTrigger>Shipping Details</AccordionTrigger>
            <AccordionContent>
              We offer worldwide shipping through trusted courier partners.
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="item-3">
            <AccordionTrigger>Return Policy</AccordionTrigger>
            <AccordionContent>
              We stand behind our products with a comprehensive 30-day return policy.
            </AccordionContent>
          </AccordionItem>
        </Accordion>
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
            <TableCell>type</TableCell>
            <TableCell>string</TableCell>
            <TableCell>single</TableCell>
            <TableCell>single, multiple</TableCell>
            <TableCell>Single or multiple items open</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>collapsible</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Allow all items to be collapsed</TableCell>
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
