<script setup lang="ts">
import { Badge } from '@auto-ui/widgets/registry/badge'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const variants = ['default', 'secondary', 'destructive', 'outline'] as const

const rows = [
  { prop: 'variant', type: "'default' | 'secondary' | 'destructive' | 'outline'", default: "'default'", desc: 'Visual style.' },
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="page-title">Badge</h2>

  <DemoBlock title="Variants">
    <Badge v-for="v in variants" :key="v" :variant="v">{{ v }}</Badge>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
