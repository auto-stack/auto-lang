<!-- TextFnCall component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, defineModel } from 'vue'
import { t } from 'vue-i18n'


const count = defineModel<number>("count", { default: 0 })


</script>

<template>
    <div class="flex flex-col gap-4">
      <span class="title">{{ t('nav.chat') }}</span>
      <span class="count">{{ t('chat.msgs', {count: count}) }}</span>
    </div>

</template>

<style>
/* Component styles */

</style>
