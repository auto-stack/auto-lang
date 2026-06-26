<script setup lang="ts">
import { Avatar, AvatarImage, AvatarFallback } from '@auto-ui/widgets/registry/avatar'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const codeImage = `<script setup lang="ts">
import { Avatar, AvatarImage, AvatarFallback } from '@auto-ui/widgets/registry/avatar'
<\/script>

<template>
  <Avatar>
    <AvatarImage src="https://avatars.githubusercontent.com/u/9919?v=4" alt="GitHub" />
    <AvatarFallback>GH</AvatarFallback>
  </Avatar>
</template>`

const codeFallback = `<script setup lang="ts">
import { Avatar, AvatarImage, AvatarFallback } from '@auto-ui/widgets/registry/avatar'
<\/script>

<template>
  <Avatar>
    <AvatarImage src="/broken.png" alt="missing" />
    <AvatarFallback>AB</AvatarFallback>
  </Avatar>
</template>`

const rows = [
  { prop: '(Avatar)', type: 'AvatarRootProps', default: '—', desc: 'reka-ui Avatar root; sizes via class.' },
  { prop: '(AvatarImage) src', type: 'string', default: '—', desc: 'Image URL (reka-ui AvatarImageProps).' },
  { prop: '(AvatarImage) alt', type: 'string', default: '—', desc: 'Alt text.' },
  { prop: '(AvatarFallback)', type: 'AvatarFallbackProps', default: '—', desc: 'Shown when image fails/loads.' },
]
</script>

<template>
  <h2 class="page-title">Avatar</h2>

  <DemoBlock title="With image" :code="codeImage">
    <Avatar>
      <AvatarImage src="https://avatars.githubusercontent.com/u/9919?v=4" alt="GitHub" />
      <AvatarFallback>GH</AvatarFallback>
    </Avatar>
  </DemoBlock>

  <DemoBlock title="Fallback" description="Broken src -> fallback shown." :code="codeFallback">
    <Avatar>
      <AvatarImage src="/does-not-exist.png" alt="missing" />
      <AvatarFallback>AB</AvatarFallback>
    </Avatar>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
