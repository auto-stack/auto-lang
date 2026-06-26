<script setup lang="ts">
import { Dialog, DialogContent } from '@auto-ui/widgets/registry/dialog'
import { Button } from '@auto-ui/widgets/registry/button'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const codeBasic = `<script setup lang="ts">
import { Dialog, DialogContent } from '@auto-ui/widgets/registry/dialog'
import { Button } from '@auto-ui/widgets/registry/button'
<\/script>

<template>
  <Dialog>
    <template #trigger><Button>Open dialog</Button></template>
    <DialogContent>
      <template #title>Are you sure?</template>
      <template #description>This action cannot be undone.</template>
      <div style="display:flex;justify-content:flex-end;gap:0.5rem">
        <Button variant="outline">Cancel</Button>
        <Button variant="destructive">Delete</Button>
      </div>
    </DialogContent>
  </Dialog>
</template>`

const rows = [
  { prop: '(Dialog)', type: 'DialogRootProps', default: '—', desc: 'reka-ui Dialog root; open state via v-model.' },
  { prop: '(DialogContent)', type: 'DialogContentProps', default: '—', desc: 'Styled overlay + panel; slots: default, title, description.' },
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes on the panel.' },
]
</script>

<template>
  <h2 class="page-title">Dialog</h2>

  <DemoBlock title="Basic" description="Trigger slot opens the styled DialogContent overlay." :code="codeBasic">
    <Dialog>
      <template #trigger><Button>Open dialog</Button></template>
      <DialogContent>
        <template #title>Are you sure?</template>
        <template #description>This action cannot be undone.</template>
        <div class="row-end">
          <Button variant="outline">Cancel</Button>
          <Button variant="destructive">Delete</Button>
        </div>
      </DialogContent>
    </Dialog>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
