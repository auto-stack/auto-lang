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

const codeComposed = `<script setup lang="ts">
import {
  Card, CardHeader, CardTitle,
  CardDescription, CardContent, CardFooter,
} from '@auto-ui/widgets/registry/card'
import { Button } from '@auto-ui/widgets/registry/button'
<\/script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Upgrade plan</CardTitle>
      <CardDescription>Get more from your workspace.</CardDescription>
    </CardHeader>
    <CardContent>
      Unlimited projects, priority support, and more.
    </CardContent>
    <CardFooter>
      <Button>Upgrade</Button>
      <Button variant="ghost">Later</Button>
    </CardFooter>
  </Card>
</template>`

const rows = [
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes (Card + all sub-parts).' },
]
</script>

<template>
  <h2 class="page-title">Card</h2>

  <DemoBlock title="Composed" description="Card with header / title / description / content / footer." :code="codeComposed">
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

  <PropTable :rows="rows" />
</template>
