# SPEC — 029-photo-gallery（Plan 537）

> Purpose: 图库——macOS 相册风图片浏览器。侧边栏相册导航 + 搜索/排序/
> 网格密度工具栏 + 响应式缩略图网格 + 大图查看器（上一张/下一张/收藏）。
> **Frontend-only，无后端；图片源为 picsum.photos 固定 seed 网络 URL。**
> 暗色对齐：shadcn 语义 token + AutoOS Dark/indigo 默认（root 声明
> `dark_mode` bool / `accent_color` str 主题契约变量）。
>
> （骨架——数据形状/过滤排序规则/双端差异注记于 T5 定稿）

## 形态

单文件单组件（025/027/028 形态）：全部状态内聚 `src/front/app.at` 的 App
根 widget，无 routes/store 子组件/模块级 fn。网格 ↔ 查看器两视图由
`var mode str` 全页条件切换（028 overlay 门控同款），不走路由。

## 图片源

- 缩略图 `https://picsum.photos/seed/gal-{NN}/400/300`
- 大图 `https://picsum.photos/seed/gal-{NN}/1600/1200`
- 同 seed 同图、确定性、无 API key。VM 侧 reqwest 阻塞下载 + 缓存；
  离线时 VM 首字母色块兜底、Vue 显示 alt 文本（功能断言不依赖图片
  字节加载成功）。

## 运行

- Vue 独立：`auto run`（front_port 4029）
- VM 独立：`auto run -r vm`
- 桌面宿主：`cargo run -p auto-lang --features ui-iced --example
  ui_desktop -- --fullscreen --apps-dir examples/ui`（零登记收录）
