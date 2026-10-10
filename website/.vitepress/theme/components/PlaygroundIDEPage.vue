<template>
  <div class="ide-page">
    <!-- 探测完成前不渲染宿主：usePlaygroundFull 在 setup 期固化 apiBase，
         先探测后渲染保证 composable 拿到最终地址。 -->
    <PlaygroundIDEApp v-if="ready" :api-base="apiBase" />
    <p v-if="ready && backendDown" class="ide-down-hint">
      后端未连通——浏览与编辑可用；Run/Trans 需本地 <code>cargo run -p auto-playground</code>。
    </p>
  </div>
</template>

<script setup lang="ts">
// PLAN-754：website /playground 页与独立版同形（PlaygroundIDEApp）。
// apiBase 探测：同源 /api（部署形态）→ 127.0.0.1:3030（本地后端约定）。
// 注意 SPA 回退会对未知路径返 200+HTML——probe 必须验证真 JSON。
import { ref, onMounted, defineAsyncComponent } from 'vue';

const PlaygroundIDEApp = defineAsyncComponent(() =>
  import('auto-playground-vue').then((m) => m.PlaygroundIDEApp),
);

const apiBase = ref('/api');
const backendDown = ref(false);
const ready = ref(false);

async function probe(url: string): Promise<boolean> {
  try {
    const res = await fetch(`${url}/examples`, { signal: AbortSignal.timeout(2500) });
    if (!res.ok) return false;
    const ct = res.headers.get('content-type') ?? '';
    if (!ct.includes('json')) return false;
    const body = await res.json();
    return Array.isArray(body?.examples);
  } catch {
    return false;
  }
}

onMounted(async () => {
  if (!(await probe(apiBase.value))) {
    const local = 'http://127.0.0.1:3030/api';
    if (await probe(local)) {
      apiBase.value = local;
    } else {
      backendDown.value = true;
    }
  }
  ready.value = true;
});
</script>

<style scoped>
/* 页内高度受控（VitePress 有页头页脚，非全视口宿主）。 */
.ide-page {
  height: min(78vh, 900px);
  display: flex;
  flex-direction: column;
}
.ide-page :deep(.pgide-shell) {
  height: 100%;
  flex: 1 1 auto;
  min-height: 0;
}
.ide-down-hint {
  margin: 0.5rem 0 0;
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
}
.ide-down-hint code {
  font-size: 0.75rem;
}
</style>
