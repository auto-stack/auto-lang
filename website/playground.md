# Playground

<script setup>
import AutoPlayground from './.vitepress/theme/components/AutoPlayground.vue'
</script>

<AutoPlayground />

::: tip Need a backend?
The playground requires a running Auto playground server. You can start one locally:

```bash
cargo run -p auto-playground
```

Or set the `apiUrl` prop to point to a remote instance.
:::
