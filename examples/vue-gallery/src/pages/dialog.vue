<script setup lang="ts">
import { Dialog, DialogContent } from '@auto-ui/widgets/registry/dialog'
import { Button } from '@auto-ui/widgets/registry/button'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight mb-6">Dialog</h2>

  <DemoBlock title="Basic" description="Trigger slot opens the styled DialogContent overlay.">
    <Dialog>
      <template #trigger><Button>Open dialog</Button></template>
      <DialogContent>
        <template #title>Are you sure?</template>
        <template #description>This action cannot be undone.</template>
        <div class="flex justify-end gap-2">
          <Button variant="outline">Cancel</Button>
          <Button variant="destructive">Delete</Button>
        </div>
      </DialogContent>
    </Dialog>
  </DemoBlock>

  <PropTable :rows="[
    { prop: '(Dialog)', type: 'DialogRootProps', default: '—', desc: 'reka-ui Dialog root; open state via v-model.' },
    { prop: '(DialogContent)', type: 'DialogContentProps', default: '—', desc: 'Styled overlay + panel; slots: default, title, description.' },
    { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes on the panel.' },
  ]" />
</template>
