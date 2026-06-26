<script setup lang="ts">
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from '@auto-ui/widgets/registry/card'
import { Button } from '@auto-ui/widgets/registry/button'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'
</script>

<template>
  <h2 class="page-title">Card</h2>

  <DemoBlock title="Composed" description="Card with header / title / description / content / footer.">
    <Card class="half">
      <CardHeader>
        <CardTitle>Upgrade plan</CardTitle>
        <CardDescription>Get more from your workspace.</CardDescription>
      </CardHeader>
      <CardContent>
        <p class="text-sm">Unlimited projects, priority support, and more.</p>
      </CardContent>
      <CardFooter class="gap-2">
        <Button>Upgrade</Button>
        <Button variant="ghost">Later</Button>
      </CardFooter>
    </Card>
  </DemoBlock>

  <PropTable :rows="[
    { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes (Card + all sub-parts).' },
  ]" />
</template>
