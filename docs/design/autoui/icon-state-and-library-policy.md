# AutoUI 图标：state 契约与图库策略

> 需求级设计注记（PLAN-621 T-06，2026-09-14 立档）。
> 上游背景：PLAN-620 完成图标双端一致性（全量字形表/尺寸口径/progress onseek）后，
> 用户提出图标五需求（双端像素级一致/可设色/满空双态/动态绘制/备用图库）。
> 本文沉淀 2026-09-14 调研数据、选型裁定与 `icon state` 契约设计。

## 1. 裁定摘要（Decision Record）

| # | 裁定 | 理由 |
|---|---|---|
| D1 | **lucide 保持主力图库，不整体迁移** | Remix 与 lucide 同名交集为 0（后缀结构 + 语义改名），全量迁移 = 永久别名映射表 + 全金样重基线，只为服务约 5% 场景（真填充需求 + 品牌图标）；且 Remix 反向缺名（lucide `bomb` 在 Remix 无对应） |
| D2 | **满/空双态用 `state` 契约（颜色 + 描边加重）模拟，不引 fill 变体** | lucide 官方明确不做 fill（纯描边哲学，无 roadmap）；颜色+描边加重覆盖「开关/启用」类语义，成本约为换库的 5% |
| D3 | **on/off 用亮度型区分（显著色 vs dim 灰），不用色相型** | 亮度差对色盲用户天然可辨（WCAG 禁仅靠色相）；`primary` ↔ `muted-foreground` 双盘 token 深浅主题自动正确 |
| D4 | **「关」态用 `muted-foreground` token，不用字面近背景色** | 贴背景太近导致不可发现；token 化 dim 可见性有保底且双端同源 |
| D5 | **补位策略 = IconifyJSON 统一管线，按需引入（playbook 见 §5，不在本计划实施）** | 一套解析器吃 200+ 图集；品牌图标（simple-icons）、真填充（phosphor-fill/remix-fill）按名取用 |
| D6 | **明确不走的路线**：图标字体（iced 端字体渲染与浏览器文本整形差异大，像素一致不可达）；iced Canvas + lyon 自绘（需自实现 stroke-to-path 对齐浏览器，成本高收益低） | — |

## 2. `icon state` 契约（已落地，PLAN-621）

### 2.1 作者面

```
icon (name: "bell", state: "on")        ← 字面量
icon (name: "heart", state: .liked)     ← bool 绑定（响应式）
```

### 2.2 展开面（双端同规则）

| state | 颜色 | 描边 |
|---|---|---|
| `on` | `text-primary`（**运行时 accent 预设主色**） | 基档 + 0.5（默认盒 2.0→2.5；≥48px 细线档 1.5→2.0） |
| `off` | `text-muted-foreground`（dim 灰，双盘） | 不加重 |
| 未声明 | 不注入任何东西（输出零 diff） | 不变 |

优先级：**显式 `text-*` 类 > state > 继承**（镜像 PLAN-620 尺寸口径的
「作者显式意图不被平台机制覆盖」）。

### 2.3 关键事实锚点（防翻案）

- **`primary` 的色相由 accent 预设运行时驱动**：`theme::resolve_semantic_rgb` 对
  `Color::Primary` 有专门臂，查 `design_tokens/registry.rs::accent_hsl`
  （indigo/coral/ocean/sage/amber 五预设；dark 下 L+10，PLAN-601 D2 双端统一）。
  即用户换 accent 预设时，`state:on` 图标自动跟随。
- **token `accent` ≠ accent 预设**：`Color::Accent`（shadcn `--accent`）是交互
  高亮 surface（hover 浅底；scaffold light = hsl(210 40% 96.1%) 近白），不是
  强调色族——不可用作 state:on 色。
- **实现锚点**：VM = `aura_view_builder.rs::with_state_tint` +
  `StyleClass::StrokeWidth`（renderer lucide 路径只读消费）；Web =
  `ui_gen/vue.rs` icon 臂（字面量 → 静态类 + attr；绑定 → `:class`/
  `:stroke-width` 三元，false → undefined 使 Vue 移除 attr）。
- **覆盖范围**：仅独立 `icon` 元素；`button`/`nav-item` 内嵌 icon 不适用
  （其 active 语义已独立存在）。

### 2.4 已知边界（诚实清单）

- 「收藏/评分」类语义（heart/star）用户心理预期是**实心填充**，颜色+描边模拟
  显得「不够满」——此类场景走 §5 补位（`ph:heart-fill` 等），第一版接受。
- Web 侧 ≥48px 细线基档（1.5）随本契约首次落入 Web（PLAN-619 G4② 当时只落了
  VM）；无 state 的既有图标不受影响。
- `state` 与显式 `color` 语义冲突时显式类赢（测试钉死）。

## 3. 2026-09-14 图库调研数据（选型依据）

| 图库 | 图标数 | 双态 | 网格 | 许可证 | 备注 |
|---|---|---|---|---|---|
| Lucide（现役） | 1401 | ❌ 官方不做 fill | 24×24 | ISC | [官方立场](https://lucide.dev/guide/lucide/advanced/filled-icons)；[Discussion #458](https://github.com/lucide-icons/lucide/discussions/458) |
| Remix Icon | 3244 = 1541 line + 1541 fill（**完美配对**） | ✅ | 24×24 | Apache-2.0 | 与 lucide 同网格；但同名交集 0（`clock`→`time-line`、`copy`→`file-copy-line`、`undo-2`→`arrow-go-back-line`、`power`→`shut-down-line`、`zap`→`flashlight-line` 等语义改名）；反例：lucide `bomb` 在 Remix 无对应 |
| Phosphor | ~1248 × 6 字重 = 9072 | ✅ fill + duotone | 16px 栅格（viewBox 256） | MIT | 字重梯度最全；duotone 突破单色 tint 模型 |
| Tabler | 6184 | ⚠️ fill 覆盖不全（~1053） | 24×24 | MIT | v3 起补 fill 中 |
| Material Symbols | 2500+ | ✅ fill 连续变量轴 | 24dp | Apache-2.0 | 优势在可变字体形态；字体路线与 iced 像素一致不可达（D6），静态 SVG 形态优势不再 |
| Heroicons | 316 × 4 | ✅ outline/solid | 24/20/16 | MIT | 基数小 |
| Fluent System Icons | ~2900 | ✅ regular/filled | 16/20/24 | MIT | — |
| Simple Icons | ~2200 品牌单色 | — | 24×24 | CC0（商标属各自所有者） | 品牌补位标准答案 |
| Iconify（聚合） | 200+ 图集 | 取决于图集 | 混合 | 按图集 | 统一 IconifyJSON schema、离线 bundle |

业界参照：shadcn/ui 绑死 lucide（单一描边库接受无双态）；MUI 自有包 + SvgIcon
逃生舱；Ant Design/Ionic/Fluent 自养多变体库（专职图标团队成本）；Flutter Material
走图标字体（本仓该避开的路线）。**没有主流框架以「多图库混排」为主方案**——
都是「一支主力集 + 明确逃生舱」。

像素级一致性口径：同源矢量数据 + 双端各自光栅化（web 浏览器 SVG 引擎 ↔ iced
resvg/tiny-skia）⇒ 几何由构造一致；抗锯齿亚像素差异不可归零，验收用阈值化
diff（如 95% 像素差 < 2/255），不承诺逐字节相等。颜色 = `currentColor` ↔
`svg::Style.color` 单色 tint 通道，双端已同构。

## 4. 波及面基线（2026-09-14 实测）

全仓 `.at` 字面量图标名 **29 个**（examples 23 + corpus/schema 7，`table` 重叠）：
bomb, book-open, calculator, clipboard, clock, copy, eye, file-plus, file-text,
film, folder, folder-open, image, list-checks, notebook, pencil, redo-2, save,
scissors, table, terminal, undo-2, zap + app-window, bell, layout-grid, power,
settings, square-stack。若未来换库，这是映射表的最小起步规模（动态绑定图标名
走同一张表）。

## 5. 补位 playbook（未来需要时按此执行，当前不实施）

1. **数据源**：devDependencies 锁版本引入 `@iconify-json/lucide`（替换现行
   lucide-vue-next ESM 解析）+ 按需 `@iconify-json/{ri,ph,simple-icons}`——
   彻底离线可复现，顺带解决 PLAN-620 待澄清#4（CI 固定版本）。
2. **命名**：图标名升级 `set:name` 双段（缺省 set = lucide，存量语料零迁移）；
   `gen-lucide-table.mjs` 泛化为 `gen-icon-table.mjs`，VM 表加图集前缀维度。
3. **优先级**：同名时 lucide 主力优先；补位名仅显式 `set:` 前缀可达。
4. **品牌图标边界**：Simple Icons 图形 CC0，但 logo 是各公司商标——用于「连接
   到该服务」属常规用法，不得用作自家品牌元素。
5. **验收**：沿用双端金样 + 阈值 diff 口径（§3）。

## 6. 关联

- PLAN-620（图标双端一致性基建）、PLAN-619（ink 几何回归锚）、PLAN-601（accent
  独立投影）、Plan 518 G4②（≥48px 细线档）、Plan 527 T8（dark: 门控）。
- 实施证据：PLAN-621 worktree 提交 `60b7c838c`/`609d514f2`/`df4bddcb2`/`074e60f77`。
