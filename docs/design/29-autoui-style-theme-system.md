# Design 29：AutoUI 样式抽象与主题系统（Style Recipes & Theme Registry）

> **状态**：Draft（2026-09-09 立档，待分解 plan）
> **来源**：2026-09-09 样式架构追问——「组件有 composable widget 抽象，样式只有
> Tailwind class 串，无组合无抽象；终极目标是任何 AutoUI app 简单切换整套主题
> （不只 accent）」——经代码实勘定案。
> **关联**：吸收 [theming-and-dark-mode](autoui/theming-and-dark-mode.md)（原
> Design 19，词汇表/正交维度沿用，值层由本设计重定义）；上游域章
> [Design 16](16-app-generation-and-ai-authoring.md)（AI authoring 面）、
> [Design 20](20-autoui-separation-architecture.md)（分离架构）；
> [base-styles-and-visual-parity](autoui/base-styles-and-visual-parity.md)（22，
> parity 规范）；债项 KNOWN-DEBT P518（per-app 主题架构缝）、P571-D1（`Color::Accent`
> 无解析臂）、charts-gallery 硬编码色 light 不翻色。

---

## 0. 问题与范围

本文回答一个问题族：**AutoUI 的样式值从哪里来、如何被引用、如何整套替换。**
具体包括：语义 token 词汇表已存在但值散落多处手抄，如何收敛为单一事实源；
「主题」如何从 dark/light 二值 + 5 个 accent 预设升级为可命名、可派生、可热切换的
整套色板；样式 class 串如何获得与 composable widget 对等的组合与抽象能力
（命名、复用、参数化）；以及这两层为什么是一个抽象而不是两个需求。

范围：Vue 臂（生成工程 + 运行时主题切换）与 VM/Iced 臂（`ui/style/` 解析 +
`resolve_semantic_rgb` 求值）的双端一致性；桌面 shell 的主题配置面
（os-config / settings / 持久化）；`.at` 语言层的 style 声明语法。不含：
Jet/Ark 后端（已死，生态线另行重启）、RenderQueue 像素级渲染、字体/图标资产主题化。

---

## 1. 现状实勘（2026-09-09）

### 1.1 已经有的（比直觉多）

- **语义 token 词汇表已在用**：`Color` 枚举语义变体（`ui/style/color.rs:7`——
  Primary/Secondary/Background/Surface/Muted/Error/Warning/Success/Info/On*）+
  class 层解析（`bg-primary`/`text-muted-foreground` 等 shadcn 风格 class 直达）。
  新示例（015-notes）已 token 化书写。
- **变体管道**：`hover:/focus:/active:/disabled:/dark:/断点:` 前缀门控
  （`ui/style/mod.rs`，Plan 527）；`dark:` 按 `theme::dark_mode()` 解析期过滤。
- **主题状态回路已通**：`theme.rs` thread-local（DARK_MODE/ACCENT_NAME/
  THEME_EPOCH/WINDOW_WIDTH）；`THEME_EPOCH` 供内容寻址视图缓存失效
  （StreamCache 消费）；Plan 518 `SetTheme(bool)` 热切换动词 +
  `shell.appearance.theme` 持久化 + boot 读回，证明「换值→失效→重建→构建期
  重解析语义色」全链可走。
- **accent 机制**：5 预设（indigo/coral/ocean/sage/amber，HSL 基准）；
  一等配置链 CLI `--theme/--accent` > os-config per-app `config.at` > pac.at >
  内置（Plan 458/504/506）。
- **Vue 端运行时换色先例**：脚手架生成的 `applyAccent()`（`root.style.setProperty
  ('--primary', …)`）证明运行时整套写 CSS 变量技术可行。
- **样式组合的雏形**：Plan 448-D style 数组形态 `style: ["基座", if …]`（类段
  显式 join）；button variant/size preset 表（`ui/style/variants.rs`，
  PLAN-571 单源 + 三表互锁）。

### 1.2 缺的（病灶定位）

**值的单一事实源不存在**——语义 token 的值至少四处置、靠注释互锁手抄：

| # | 位置 | 形态 | 互锁证据 |
|---|---|---|---|
| V1 | `ui_gen/vue.rs` `generate_base_css`（~15564） | 手写 `:root`/`.dark` CSS 变量模板串 | 注释「与 Rust 侧 theme.rs Color::Secondary 互锁（改任一须同步）」 |
| V2 | `ui/style/theme.rs` `resolve_semantic_rgb`（~188） | match 臂逐色硬编码 RGB，dark/light 双分支 | Plan 448/455/518/571 全是「对齐批」——每轮视觉校准开一个 plan 手工同步双端 |
| V3 | `variants.rs` ↔ vue 脚手架 `variants.ts` cva ↔ `ui_gen/rust` | class 配方三表互锁 | 文件头自述「三表改任一须同步」，靠 parity 测试钉住 |
| V4 | `ui/code_editor/theme.rs` | 编辑器高亮主题又一套 + `ACCENT_PALETTES` 在 vue.rs 与 theme.rs 各一份 | accent 名单双份维护 |

推论：**主题无法成为一等对象**——没有可替换的值载体，切换整套色板无从谈起。
这也是「对齐批」plan 反复出现的结构性根因。附带漂移风险已现形：vue 脚手架模板
`--background: 0 0% 100%`（纯白）与 theme.rs light 暖纸 `#f5f1e8`（Plan 518 stella
重校）分叉——是否属未同步（抑或 vue 桌面宿主另有 CSS 路径）列为 Phase 1 对账项。

**样式的组合抽象不存在**——class 串是唯一书写形态，重复以字面量拷贝蔓延：
015-notes 的「主色药丸按钮」串同文件原样重复两遍；013-todo 的 filter 按钮串重复
八遍（仅一词之差）。平台 preset 表（variants.rs）用户不可扩展、不可覆盖。

**主题表达力缺口**：dark/light 二值 + accent 单槽，无命名主题、无派生、无
app 级自定义色板；per-app 主题与 shell 全局主题共享 process-wide thread-local，
存在在案架构缝（KNOWN-DEBT P518：浅色桌面开深色 app 会把 shell chrome 一并翻深）。

---

## 2. 目标与非目标

**目标**

1. 语义 token 的值有单一事实源；改一个值只改一处，双端（vue 生成物/VM 求值）
   同源变化，「对齐批」plan 作业消失。
2. 主题 = 可命名的整套值表：内置若干套（zinc/stella-dark/stella-light/…）、
   支持派生（extends）与 app 级局部覆盖、运行时可热切换（复用 THEME_EPOCH 回路）。
3. `.at` 源码获得样式组合抽象：命名 recipe、引用与参数化、与既有
   字符串/数组/条件形态自由混写；编译期展开，零运行时成本，双端天然一致。
4. 任何 token 化书写的 app，切换主题时全部颜色跟随（含代码编辑器高亮层）。

**非目标（明确不做）**

- 不引入运行时样式对象 / 内联 style DSL 作为第二套样式表示（双端 parity 负担
  翻倍；class 字符串协议是既有资产，recipe 只是把重复字面量变为命名常量）。
- 不做 CSS 变量的计算表达力（`color-mix()`、嵌套 `calc()`）——token 值保持
  字面量 + 引用解析，VM 端是编译期解析 class，值层表达力无消费面。
- 第一刀 scope **只有 color token**：radius/shadow/duration/spacing 留表结构
  扩展位不开放——用户级主题切换需求 90% 是颜色（VSCode 主题几乎只有颜色），
  等真实需求出现再扩，避免重蹈 raw/design-token-system.md 七大类一步铺满的过度设计。
- 不强制迁移存量硬编码色 app（013-todo 的 `bg-blue-500` 类）；token 化覆盖率
  是主题切换**效果**的决定因素，作为验收口径而非强制改造项。

---

## 3. 核心裁定（设计原则）

**裁定 1：样式表示继续是 class 字符串协议。** recipe 展开为 class 串，token
解析为 RGB/CSS 变量——两端表示层零变更，所有既有 parser/审计/parity 基建
（Plan 527 清单契约）原样受益。

**裁定 2：token 是封闭受控词表，编译期校验未知 token。** 对 VSCode 自由字符串
键（扩展可贡献任意键）的修正：Auto 是强类型语言，词表以 shadcn 全集为基准
（background/foreground/card/card-foreground/popover/primary/primary-foreground/
secondary/secondary-foreground/muted/muted-foreground/accent/accent-foreground/
destructive/destructive-foreground/border/input/ring + 扩展 success/warning/info），
registry 外的颜色值只能走 Tailwind 调色板/字面量 class（现状语义不变），但
theme 声明里出现未知名 = 编译错误。VM 侧 `Color` 枚举是词表的子集投影
（`"card"|"surface"|"popover"` 收敛为 `Surface`，`color.rs:115`），投影映射成文
（§5.4），词表全集以 registry 为准。

**裁定 3：主题 = 覆盖链（partial override），不是全量表。** VSCode 主题 JSON
只写想覆盖的键、未覆盖回落产品默认的模型。合成顺序：

```
内置基座主题 → theme 声明 extends 链逐层覆盖 → accent 覆盖层（仅 primary 槽族）
→ os-config per-app 运行时覆盖
```

现有 5 个 accent 预设降维为「primary 槽位的预设覆盖」，`pac.at accent:` 键、
`AUTO_UI_ACCENT` env、优先级链全部保留兼容。

**裁定 4：recipe 是编译期字符串宏，desugar 到既有表达式机制。** recipe 体是
含参数插值的字符串，展开等价 f-string；引用在求值期内联为串。不新增任何运行时
语义——参数化 recipe 就是「返回 str 的函数」的语言面糖（§6.2）。

**裁定 5：dark/light 从布尔降维为「主题的 mode 属性」。** `dark:` 前缀门控从
`theme::dark_mode()` 改读 `active_theme().is_dark`；对外行为不变，`mode: auto`
跟随系统作为属性的自然扩展。

---

## 4. Layer 1：ThemeRegistry（值的抽象）

### 4.1 数据模型

```rust
/// ui/style/theme/registry.rs（新）—— 单一事实源
pub struct ThemeSpec {           // 声明态（可序列化、可派生）
    pub name: str,
    pub extends: Option<str>,    // 派生链（VSCode include 等价物）
    pub mode: Mode,              // Dark | Light | Auto
    pub colors: Map<TokenName, ColorLit>,   // partial——未写的键回落 extends
}
pub struct ResolvedTheme {       // 合成态（渲染消费）
    pub mode: Mode, pub is_dark: bool,
    pub colors: Map<TokenName, Rgb>,        // 链式 resolve 后的完整表
}
```

- `TokenName`：封闭枚举（§3 裁定 2 词表）。
- `ColorLit`：hex / `hsl(h s% l%)` 字面量。值可引用同主题内已定义 token
  （`primary-foreground: var(background)` 级别的引用在 resolve 期展开，仅一层
  别名，不做递归计算）。
- 内置主题表硬编码于 registry（zinc=现 vue 模板值、stella-dark/stella-light=
  Plan 518 校准值），**现状四处置的值全部迁入后删除原处**（V1–V4 改造，§4.4）。

### 4.2 主题声明格式（.at）

app 级声明落 pac.at（与既有 `theme:`/`accent:` 键同域扩展）或独立 `theme.at`：

```auto
theme {
    extends: "stella-dark"          // 派生基座；缺省 = "zinc"
    // mode 继承自基座，可不写；显式覆盖：
    // mode: "auto"
    colors: {
        primary: "#8b5cf6"          // 只写想覆盖的键（partial）
        card: "hsl(222 47% 9%)"     // 字面量或别名引用
    }
}
```

选 `.at` 语法而非 JSON 的理由：类型检查、与 pac.at 同构、AI 可直接生成
（呼应 Design 16 的 AI authoring 面——raw/design-token-system.md「AI 直接生成
token」的愿景在此兑现，但其多平台 Token Compiler 形态退役）。JSON 仅作为
registry 内部序列化格式（os-config 持久化），不作书写格式。

### 4.3 运行时切换

- VM 臂：`set_theme(name)` 合成 `ResolvedTheme` 存活动槽 + `THEME_EPOCH` 自增
  （既有失效回路：view 重建→class 重解析→`resolve_semantic_rgb` 查新表）。
  `set_dark_mode`/`set_accent_name` 保留为兼容写入口（内部改写活动主题的
  mode/primary 槽）。
- Vue 臂：生成期按活动主题渲染 `index.css` 变量块（V1 模板改生成）；运行时
  切换 = `applyAccent` 先例的泛化 `applyTheme(theme)`——逐键 `setProperty`
  写入整套变量 + 翻转 `.dark` class。
- Shell 配置面：`SetTheme(bool)` 泛化为 `SetTheme(name)`；os-config
  `appearance.theme` 值域从 `dark|light` 扩为主题名（读回兼容旧值映射
  stella-dark/stella-light）；settings 面板从二值开关变主题选择器。

### 4.4 四处置消费点改造（消灭手抄）

| 处 | 改造 | 验收 |
|---|---|---|
| V1 `generate_base_css` | 变量块从 `ResolvedTheme` 生成（含 dark/light 双块：基座主题 + 派生 light 基座或同主题双 mode 集） | 模板中零手写色值 |
| V2 `resolve_semantic_rgb` | match 臂全删，改查活动 `ResolvedTheme`（词表全集→`Color` 枚举投影映射，§5.4） | 函数体无字面 RGB |
| V3 variants 三表 | **不变**——那是 class 配方不是值；互锁继续靠 parity 测试，但配方内 token 引用的值全部来自 registry | 零改动 |
| V4 code_editor 桥 | hljs/editor 主题色从 `ResolvedTheme` 派生（背景/前景/选中/accent 槽注入） | 切主题后编辑器高亮同步翻转 |

### 4.5 per-app color context（解 P518 架构缝）

registry API 从第一天按 per-context 实例设计（`ResolvedTheme` 按值传递，
不是进程单例）；但落地分期：

- **Phase 2a**：进程内活动主题单值（现状 thread-local 形态换芯）——切主题影响
  全部 App，行为与现状一致，P518 架构缝维持登记。
- **Phase 2b**：session `allocate_app` 时按「shell 全局主题 × per-app os-config
  覆盖」合成 per-App 主题，渲染时按 App 路由各自 context（shell chrome 用
  shell 自己的主题）。与 KNOWN-DEBT P518 原建议一致，时机对齐 RenderQueue
  色彩上下文重构——若该重构排期远，2b 可独立先行（动态渲染路径
  `dynamic_view` 每帧回写时序是硬点）。

---

## 5. Layer 2：style recipe（组合的抽象）

### 5.1 语法（语言面）

与 widget/store 平级的声明块：

```auto
// 常量 recipe —— 命名的 class 串
style card-base = "bg-card rounded-xl shadow-sm border border-border"

// 参数化 recipe —— 插值参数带默认值
style pill(bg: str = "bg-primary", fg: str = "text-primary-foreground",
           pad: str = "px-4 py-2") =
    "{pad} {bg} {fg} rounded-full text-sm font-medium shadow-sm hover:{bg}/90 transition-colors"

// 引用组合
style pill-danger = pill(bg: "bg-destructive", fg: "text-destructive-foreground")
```

消费——与既有形态自由混写（448-D 数组、条件 if、f-string 全兼容）：

```auto
button "New"  { onclick: .Create, style: pill() }
button "删"   { onclick: .Del,    style: [pill(bg: "bg-destructive"), "ml-2"] }
col { style: card-base }        // 裸标识符 = 无参调用
text "标题" { style: "{card-base} mt-4" }   // 插值引用
```

### 5.2 语义（desugar，裁定 4）

- 常量 recipe → 字符串字面量。
- 参数化 recipe → 返回 `str` 的函数（既有 fn 机制）；调用点内联展开。
- 裸标识符消费 → 无参调用求值。引用整体在**视图构建期**求值（与条件
  style/f-string 同时刻），不引入新的求值时机。
- 展开点单一：AST→AuraNode 归一化处（三后端同源展开，天然 parity——Vue 转译
  拿到的已是展开后的串或可求值表达式）。
- recipe 体内建议（lint 级，非硬门）只引用语义 token class；裸调色板色
  （`bg-blue-500`）在 recipe 内触发告警——recipe 是 token 化书写的推广面。

### 5.3 与平台 preset 的关系

`variants.rs` 的 button variant/size 表是**平台层** recipe（内置、随版本演进），
app recipe 是**用户层**；两层命名空间分开（内置以 `button.primary` 等限定键
存在），v1 不开放 app recipe 覆盖内置 preset 键（开放与否见 §8 开放问题）。

### 5.4 词表投影映射（成文契约）

VM `Color` 枚举 ↔ registry 词表全集的投影关系：

| 词表 token | Color 投影 |
|---|---|
| background | Background |
| card / popover / surface | Surface（三键词表可分色，投影收敛——升级 Color 枚举为独立变体留作后续） |
| foreground / muted-foreground / card-foreground… | OnBackground / OnSurface（同理收敛） |
| primary / primary-foreground | Primary / OnPrimary |
| border / input / ring | Border（ring 单列消费点在 focus 描边，投影同值） |
| success / warning / info / destructive | Success / Warning / Info / Error |

映射表落 `ui/style/theme/registry.rs` 并以单测钉死（词表全集每个 token 必须
有投影；`Color::Accent` 解析臂随此补齐，清偿 P571-D1）。

---

## 6. VSCode 主题组织形式：借鉴与修正

| VSCode 机制 | 采纳形态 | 说明 |
|---|---|---|
| 主题 = partial override，未覆盖键回落产品默认 | ThemeSpec partial + extends 链 + 内置基座 | 「校准」变主题派生而非改源码 |
| `include` 派生链 | `extends:` | stella-* 对基座的局部重校即派生实例 |
| workbench 色 + token 色两层 | widget 语义色层 + code_editor 高亮层（V4 改造） | 切主题两层同源翻转 |
| 贡献点任意字符串键 | **修正**为封闭词表 + 编译期校验（裁定 2） | 强类型语言的收益点 |
| 运行时注册表 + 切换即重读 | ThemeRegistry + THEME_EPOCH 失效回路 | 回路已有，换芯即可 |
|（VSCode 无对应）| **新增** accent 覆盖层概念 | VSCode 无 accent 单槽先例；保留既有 5 预设兼容 |

---

## 7. 实施路线（三阶段 → plan 分解）

> 一起设计（本档）、分刀落地；Phase 1 独立可验收且即时还本。

**Phase 1：token 单源化（纯基建，不改语言，不改任何 app 源码）**
- ThemeRegistry 落地（内置 zinc/stella-dark/stella-light，值自 V1–V4 迁入）；
  V1/V2/V4 改造；词表投影映射 + `Color::Accent` 补臂。
- **验收 = 视觉零变化**：全部 examples 双端 golden/截图不动（registry 迁移是
  换存放处不是换值）；V1/V2 函数体零手写色值；对账 vue 模板 vs theme.rs 的
  light background 分叉（§1.2）定性并归一。
- 顺带还本：此后任何视觉校准 = 改 registry 一处（对齐批 plan 作业消失）。

**Phase 2：主题声明与切换（`.at` 声明面 + 运行时切换面）**
- `theme {}` 声明解析（pac.at/theme.at）+ extends 合成 + accent 降维兼容；
- `SetTheme(name)` 泛化 + os-config 主题名值域 + settings 选择器；
- Vue 臂 `applyTheme` 运行时写入；`dark:` 门控改读 `is_dark`（防回归专项）；
- per-context 分期按 §4.5（2a 随本阶段，2b 对齐 RenderQueue 或独立立项）。
- 验收：≥3 套内置主题热切换，token 化区域 100% 跟随（含编辑器高亮）；
  旧 `dark|light` 配置读回兼容；charts-gallery 硬编码色债在此背景下重估迁移。

**Phase 3：style recipe（语言层，✅ 已由 PLAN-607 交付）**
- parser/AST `style` 声明块 + desugar 展开（§5.2）+ 双端展开点接线；
- 与 448-D 数组/条件/f-string 混写回归；recipe lint（裸调色板告警）。
- 验收：013-todo filter 按钮串 8 处→1 处、015-notes 药丸串 2 处→1 处的
  收敛示范；golden 零变化（纯重构不改视觉）。
- 交付成果：`ast/ui.rs` `StyleRecipeDecl` AST、`design_tokens/recipe.rs` 注册/校验/展开引擎、Vue 与 VM 归一化单点 Desugar、硬编码调色板色 lint 守卫，以及 013-todo / 015-notes 示范重构。

每阶段一个独立 plan（L1 体量），Phase 1 可先行合入不受后两阶段约束。

---

## 8. 风险与开放问题

1. **Tailwind class attribute 内覆盖次序语义**：浏览器 CSS 特异性由样式表顺序
   决定，串内后写类不保证覆盖先写类（VM 侧是顺序应用 last-wins，双端本已
   有微差）。recipe 组合因此**只承诺拼接不承诺覆盖**——冲突片段靠 lint/约定
   排除（§5.2）；是否提供显式覆盖算子留开放问题。
2. **per-app context 渲染时序**（§4.5 2b）：`dynamic_view` 每帧回写全局态的
   现路径是主要改造面，需实测定案（继承 P518 架构缝的全部复杂性）。
3. **词表投影的收敛损耗**：card/popover/surface 三键在 VM 投影为同一
   `Surface`——主题若给三键不同值，VM 视觉仍收敛。升级 `Color` 枚举为独立
   变体是干净的解法但触碰全部 adapter match 臂，Phase 1 先成文、Phase 2 视
   内置主题是否需要三键分色再定。
4. **recipe 覆盖内置 preset 键**（`style button.primary = …`）：有真实诉求
   （app 品牌化按钮）但涉及与 variants 三表互锁的交互，v1 不开放，诉求出现
   时随 Phase 3 复审。
5. **Vue 生成期 vs 运行时双轨**：生成期固化（index.css 按活动主题渲染）与
   运行时 `applyTheme` 并存后，`mode: auto` 跟随系统的形态在 vue 臂需择
   `prefers-color-scheme` 媒体查询或宿主注入二一定案。
6. **迁移覆盖率口径**：主题切换效果 = token 化覆盖率，存量 app 硬编码色
   （013-todo/charts-gallery 在案）不强制迁移但验收报告需附覆盖率清单，
   防止「切了主题但一半没变」的隐性劣化。

---

## 9. 与既有设计的关系

| 既有文档/机制 | 处置 |
|---|---|
| [theming-and-dark-mode](autoui/theming-and-dark-mode.md)（原 Design 19，2026-07-17） | **吸收其词汇表（§3.1）与正交维度思想，取代其值层设计**（它定义了「查主题表」但未定义表的来源/格式/生命周期——本设计补的正是这层）；其 Jet/Ark 后端章节随后端退役作废；其 Phase 迁移计划由本设计 §7 取代。头部加取代注记 |
| [raw/design-token-system.md] | 历史素材。「AI 直接生成 token」愿景被 §4.2 吸收（theme.at 即 AI 书写面）；多平台 Token Compiler（Jet/Ark/C 目标）退役 |
| [base-styles-and-visual-parity](autoui/base-styles-and-visual-parity.md)（22） | 互补不变：它管「class 语义双端一致」，本设计管「token 值单源 + 主题替换」；variants 三表互锁归它，值来源归本设计 |
| Plan 458/504/506 优先级链 | 原样继承：CLI > os-config > pac.at > 内置，链尾接 registry 基座 |
| Plan 527 清单契约 | 原样受益（裁定 1：表示层不变）；词表封闭性未来可加「未注册语义 class」审计钩子 |
| KNOWN-DEBT P518 架构缝 / P571-D1 | 分别由 §4.5 与 §5.4 承接清偿 |
