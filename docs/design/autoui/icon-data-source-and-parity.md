# AutoUI 图标数据源与双端一致性口径

> 需求级设计注记（PLAN-620 T-06，2026-09-14 立档）。
> 姊妹篇：[icon-state-and-library-policy](icon-state-and-library-policy.md)（PLAN-621，
> state 契约与图库选型裁定）。本文回答：VM/iced 端的 lucide 字形从哪来、怎么再生、
> 与 Web 端如何由构造成立地一致、以及防漂移门禁。

## 1. 数据源与生成器

```
lucide 官方数据（本地已安装的 lucide-vue-next，dist/esm/icons/*.js）
        │  scripts/gen-lucide-table.mjs（离线、幂等，不联网）
        ▼
crates/auto-lang/src/ui/iced/lucide_generated.rs   ← 生成文件，勿手改
   pub(super) static LUCIDE_ICONS: &[(&str, &str)]  (按名升序，24×24 内部 markup)
   pub(super) const LUCIDE_SOURCE_VERSION: &str     (生成时源版本，漂移门禁对拍用)
   pub(super) fn lookup(name) -> Option<&'static str>  (二分查找)
        │
        ▼
renderer.rs::lucide_svg → lucide_fragment(name) → lucide_svg_doc_with(name, stroke_width)
   （单层包装 `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
     stroke-width=… stroke-linecap="round" stroke-linejoin="round">`）
```

- **为什么是「生成」而不是「引 crate」**：引 Rust 侧 lucide crate 会引入新依赖与
  版本漂移；从本地已安装的 lucide-vue-next 抽取则**零新依赖**、可离线重跑，且
  数据与 Web 端同源（Vue 端运行时 import 的就是这个包）。
- **为什么是「全量」**：表里只有「记得去补」的几十个名字时，缺名在 VM 端静默
  渲染成空盒（债务 P537-D1 的根因形态）。全量表让「同名 ⇒ 同字形」由构造保证。
- **重跑方式**：`node scripts/gen-lucide-table.mjs`（自动在 `examples/**`、
  `packages/**` 里发现已安装的包，pnpm `.pnpm` 布局也覆盖，取版本号最高者）；
  或显式 `--src <lucide-vue-next/dist/esm/icons>`（会向上探测 package.json 取版本）。
- **遗留别名**：`sidebar`（上游已更名 `panel-left`）、`file-icon`（已并入 `file`）
  ——这两个名字在 lucide-vue-next 里已不存在，保留别名只为不把既有语料打成空盒。

## 2. 双端一致性口径（为什么能做到像素级）

- **几何**：双端消费同一份 path 数据（Web=浏览器 SVG 引擎渲染 lucide-vue-next
  组件；VM=iced `svg` widget 经 resvg/tiny-skia 光栅化同一 markup）——几何由
  构造成立一致；抗锯齿亚像素差异不可归零，验收用阈值化 diff（同 619 ink 锚、
  `tools/parity_shot_diff.py` 预算），不承诺逐字节像素相等。
- **尺寸**：显式 `w-*`/`h-*`/`size-*` 类 > `size:` prop > 默认 20px（PLAN-619/617
  双端口径，见 `with_icon_size` 与 `vue.rs` icon 臂的镜像实现）。
- **颜色**：`stroke="currentColor"` ↔ `svg::Style.color` 单色 tint 通道同构
  （引擎级 currentColor，画时着色，悬停变体可用）。
- **描边宽度**：基档 2.0、≥48px 独立图标 1.5（Plan 518 G4②，PLAN-619 ink 回归锚
  `plan619_lucide_doc_renders_geometric_ink` 守护）；PLAN-621 起 state=on 在基档上
  +0.5。

## 3. 防漂移门禁（T-07）

生成产物内嵌 `LUCIDE_SOURCE_VERSION` 常量与对拍测试
`source_version_matches_installed_package`：

- 本地安装包版本（walkdir 扫 `examples/**`，可用 `LUCIDE_VUE_NEXT_ROOT` 覆盖探测根）
  ≠ 生成时版本 → 红，错误信息指路再生命令；
- 本地找不到安装包（CI/纯净 checkout）或版本未知 → 打印 skip 理由后通过；
- 升级 lucide-vue-next 后的标准动作：`pnpm install`（任意示例）→ 重跑生成器 →
  门禁转绿（Web 端 import 的是同一个包，无第二处版本要管）。

## 4. 已知边界

- 生成器只识别 `createLucideIcon("…", [[tag, attrs], …])` 形态（当前全部 1401 个
  均为此形态）；上游若引入新数据形态，生成器解析失败会**打印并跳过**该图标
  （表变小 → `table_is_sorted_and_nonempty` 的 >1000 断言兜底告警）。
- markup 含 `"#` 序列的图标无法用 `r#"…"#` 包裹——生成器直接退出（当前不存在，
  属防御性断言）。
- 254 KiB 静态表长期留在仓库是「两端一致」的对价（620 待澄清③裁定：保留全量；
  按需裁剪会重新打开「记得去补」的口子）。
