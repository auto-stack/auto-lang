<template>
  <div class="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-3">
    <a
      v-for="icon in icons"
      :key="icon.name"
      :href="`https://fonts.google.com/icons?selected=Material+Symbols+Outlined:${icon.name}:FILL@0`"
      target="_blank"
      rel="noopener"
      class="flex flex-col items-center gap-2 p-3 rounded-xl border border-slate-200 hover:border-violet-300 hover:shadow-sm cursor-pointer transition-all bg-white"
    >
      <span class="material-symbols-outlined text-2xl text-slate-600">{{ icon.name }}</span>
      <span class="text-[10px] text-slate-500 text-center leading-tight">{{ icon.label }}</span>
    </a>
  </div>
</template>

<script setup lang="ts">
const icons = [
  { name: 'home', label: 'home' },
  { name: 'menu', label: 'menu' },
  { name: 'close', label: 'close' },
  { name: 'arrow_back', label: 'arrow_back' },
  { name: 'arrow_forward', label: 'arrow_for...' },
  { name: 'chevron_left', label: 'chevron_l...' },
  { name: 'chevron_right', label: 'chevron_r...' },
  { name: 'expand_more', label: 'expand_m...' },
  { name: 'expand_less', label: 'expand_less' },
  { name: 'more_vert', label: 'more_vert' },
  { name: 'more_horiz', label: 'more_horiz' },
  { name: 'refresh', label: 'refresh' },
  { name: 'search', label: 'search' },
  { name: 'settings', label: 'settings' },
  { name: 'add', label: 'add' },
  { name: 'remove', label: 'remove' },
  { name: 'edit', label: 'edit' },
  { name: 'delete', label: 'delete' },
  { name: 'save', label: 'save' },
  { name: 'done', label: 'done' },
  { name: 'check', label: 'check' },
  { name: 'check_circle', label: 'check_cir...' },
  { name: 'cancel', label: 'cancel' },
  { name: 'send', label: 'send' },
  { name: 'share', label: 'share' },
  { name: 'download', label: 'download' },
  { name: 'upload', label: 'upload' },
  { name: 'print', label: 'print' },
  { name: 'content_copy', label: 'copy' },
  { name: 'content_paste', label: 'content_p...' },
  { name: 'mail', label: 'mail' },
  { name: 'email', label: 'email' },
  { name: 'message', label: 'message' },
  { name: 'chat', label: 'chat' },
  { name: 'phone', label: 'phone' },
  { name: 'call', label: 'call' },
  { name: 'notifications', label: 'notificat...' },
  { name: 'notification_add', label: 'notificat...' },
  { name: 'play_arrow', label: 'play_arro...' },
  { name: 'pause', label: 'pause' },
  { name: 'stop', label: 'stop' },
  { name: 'skip_next', label: 'skip_next' },
  { name: 'skip_previous', label: 'skip_prev...' },
  { name: 'volume_up', label: 'volume_up' },
  { name: 'volume_off', label: 'volume_off' },
  { name: 'mic', label: 'mic' },
  { name: 'videocam', label: 'videocam' },
  { name: 'photo_camera', label: 'photo_ca...' },
  { name: 'image', label: 'image' },
  { name: 'music_note', label: 'music_not...' },
  { name: 'person', label: 'person' },
  { name: 'people', label: 'people' },
  { name: 'group', label: 'group' },
  { name: 'account_circle', label: 'account_c...' },
  { name: 'face', label: 'face' },
  { name: 'sentiment_satisfied', label: 'sentiment...' },
  { name: 'info', label: 'info' },
  { name: 'help', label: 'help' },
  { name: 'warning', label: 'warning' },
  { name: 'error', label: 'error' },
  { name: 'add_circle', label: 'add_circl...' },
  { name: 'add_box', label: 'add_box' },
  { name: 'verified', label: 'verified' },
  { name: 'star', label: 'star' },
  { name: 'star_border', label: 'star_bord...' },
  { name: 'favorite', label: 'favorite' },
  { name: 'favorite_border', label: 'favorite_...' },
  { name: 'thumb_up', label: 'thumb_up' },
  { name: 'thumb_down', label: 'thumb_down' },
  { name: 'folder', label: 'folder' },
]
</script>
