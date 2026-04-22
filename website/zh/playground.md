# Playground

<script setup>
import AutoPlayground from '../.vitepress/theme/components/AutoPlayground.vue'
</script>

<AutoPlayground />

::: tip 需要后端？
Playground 需要运行 Auto playground 服务器。你可以在本地启动：

```bash
cargo run -p auto-playground
```

或设置 `apiUrl` 属性指向远程实例。
:::
