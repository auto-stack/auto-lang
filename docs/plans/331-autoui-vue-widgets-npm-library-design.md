# Plan 331: @auto-ui/widgets — AutoUI 生成的 Vue 组件库(npm 发布)

> **类型**:完整计划(设计 + 实施)
> **状态**:设计已确认,实施待执行
> **日期**:2026-06-23
> **前身**:[324-autoui-widget-library-strategy.md](324-autoui-widget-library-strategy.md)(战略建议,识别了「缺少可发布的通用组件库」这一空白)
> **关联**:319(unify-vm-rust-rendering)、327(015-notes-vm-render)
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行。

---

# 第一部分:设计

## 1. 定位与目标

### 1.1 一句话定位

AutoUI 用 Auto 语言定义跨平台 widget;Web 平台上,AutoLang 编译器把这些 widget **生成自包含的 Vue 3 SFC**,以 npm 包 + CLI 形式发布,供任意 Vue 项目消费。**组件由编译器生成,非手写。**

### 1.2 v1 目标

1. 跑通全链路:widget 定义 → `auto` 生成 `.vue` → 打包 → `npm publish` → 用户 `npx @auto-ui/widgets add` 拷贝
2. 首发 **12 个自包含原语**(覆盖表单/布局/反馈/覆盖层主干)
3. License 合规(MIT + reka-ui/shadcn-vue/Tailwind 归属)
4. `examples/component-gallery` 改名 `examples/gallery`,转为 `@auto-ui/widgets` 的 dogfood 消费者

### 1.3 非目标(v1)

- 依赖包模式(模式 A,直接 `import`)——仅留架构扩展点
- 复合组件(LoginForm/Sidebar 等组合 widget)
- AutoUI 自有视觉语言(v1 沿用 shadcn-vue 配方)
- React / 其他端产物

---

## 2. 关键决策(brainstorming 结论)

| 决策点 | 结论 | 理由 |
|---|---|---|
| **库的形态** | 编译产物 `.at → .vue`(非手写 wrapper) | 契合 AutoUI「`.at` 单一来源、多端产出」核心卖点 |
| **消费方式** | 模式 B:CLI 拷贝(shadcn 风格) | 与现有转译输出(shadcn-vue+Tailwind 假设)契合,绕开样式打包深坑,复用 `auto-man/vue.rs` |
| **CLI 产出方式** | 预生成 `.vue` 打进包,CLI 只拷贝 | 用户端零 Rust 依赖,最鲁棒 |
| **组件定位** | 自包含原语(共用 reka-ui 底座,脱离 shadcn-vue 成品) | 唯一与「AutoUI 跨平台生成 UI」目标一致的定位 |
| **视觉层** | v1 沿用 shadcn-vue 配方(参考实现,非逐字抄) | 快速、合规、好看;生成器架构保证未来可全局换配方 |
| **v1 组件数** | 12 个最小原语集 | 先验证全链路,再扩 |
| **仓库结构** | `packages/widgets/` + Node CLI | 职责清晰、发布洁净、gallery 顺势 dogfood、`packages/` 为多端留扩展 |
| **peer 依赖** | vue 必须 peer;reka-ui optional peer(CLI 默认自动装);Tailwind 不进 peer | 与 shadcn-vue 生态对齐 |
| **默认/覆盖** | reka-ui 自动装 + 可 `--reka-ui` 覆盖;Tailwind 提供预编译 `dist/styles.css`(零配置)与用户自带 Tailwind 二选一 | 满足「普通用户零配置」同时允许高级用户自定义 |
| **模式 A 扩展** | v1 不实现,留架构扩展点 | 未来不重写 |

---

## 3. 组件定位澄清

### 3.1 分层关系

| 层 | shadcn-vue | @auto-ui/widgets |
|---|---|---|
| 行为层(无障碍/键盘/焦点) | reka-ui | **reka-ui(共享)** |
| 样式层(Tailwind class 配方) | shadcn-vue recipe | AutoUI codegen 发射的 recipe(参考 shadcn-vue) |
| 组件 API/设计语言 | shadcn-vue prop 命名 | AutoUI 自己的 AURA widget API |

**结论**:脱离「shadcn-vue 成品」,但共用「reka-ui 底座」。AutoUI 是 reka-ui 之上的另一层样式,只是这层样式由编译器生成。

### 3.2 「生成而非手写」的含义

- AURA `.at` 是**使用** DSL(`button "txt" variant:"primary"` 调用原语)
- 原语的**渲染定义**活在 Rust 代码生成器后端(`ui_gen/vue.rs` 的 widget registry)
- 「从 `.at` 生成原语」= **codegen 的 widget registry 是事实来源**,`.at` 是规格/调用方
- 每个 widget 在 codegen 里有一段**渲染模板**(Rust 发射逻辑),参考 shadcn-vue 对应组件结构(reka-ui + Tailwind 最佳实践),输出**不 import `@/components/ui/...`**,而是自带 reka-ui import + Tailwind class → 自包含

### 3.3 License 处理

- 全链路宽松 MIT(reka-ui / shadcn-vue / Tailwind 均 MIT),无 copyleft 风险
- shadcn-vue 的设计模型本就是「拷代码进项目、归你所有」,当前组件「一对一复制」在 MIT 下合法,但**归属未做**——v1 补上
- 措施:`NOTICES` 文件 + 每个 `.vue` 顶部归属注释 + README Credits(详见 §7)

---

## 4. 生成流水线

```
widget 定义(Rust registry in crates/auto-lang/src/ui_gen/vue.rs)
        │  auto ui build --target vue --out packages/widgets/registry
        ▼
registry/*.vue  (自包含: import 'reka-ui' + Tailwind class)
        │  tailwindcss 扫 registry → packages/widgets/dist/styles.css
        ▼
packages/widgets/  (registry + dist/styles.css + cli + package.json + LICENSE/NOTICES)
        │  npm publish
        ▼
npm registry
        │  npx @auto-ui/widgets add <widget>
        ▼
用户项目  (CLI 拷 .vue + 自动装 reka-ui + 问 Tailwind 路径)
```

### 4.1 核心改动:`ui_gen/vue.rs` 渲染后端

**现状**:转译 AURA `button` 标签 → `import { Button } from '@/components/ui/button'`(调用现成 shadcn-vue 原语)。

**目标**:每个 widget 的渲染模板改为**发射自包含 `.vue`**——reka-ui import + Tailwind class + 由 AURA widget spec 驱动的 prop/variant。这是 v1 的核心工程量。

### 4.2 新增 `auto` 子命令

`auto ui build --target vue --out <path>`:遍历 widget registry,逐个生成 `.vue` 到目标目录。仅在我们的开发/CI 环境运行,用户端不跑。

---

## 5. 包结构

```
packages/widgets/
├── package.json        # name:@auto-ui/widgets  version  bin:cli  files:[registry,dist,cli,README,LICENSE,NOTICES]
├── LICENSE             # MIT (Copyright Soutek Co. Ltd.)
├── NOTICES             # reka-ui / shadcn-vue / Tailwind 归属
├── README.md
├── cli/                # Node/TS:add / list 命令
├── registry/           # ← auto 生成的 .vue(发布产物)
│   ├── button/{Button.vue, index.ts}
│   ├── input/{Input.vue, index.ts}
│   └── ...
└── dist/
    ├── styles.css      # 预编译 Tailwind(零配置用户 import)
    └── index.js        # (v2 模式 A 用,v1 留空/预留)
```

### 5.1 package.json 关键字段

```jsonc
{
  "name": "@auto-ui/widgets",
  "version": "0.1.0",
  "type": "module",
  "bin": { "auto-ui": "./cli/dist/index.js" },
  "files": ["registry", "dist", "cli", "README.md", "LICENSE", "NOTICES"],
  "peerDependencies": {
    "vue": "^3.4.0",
    "reka-ui": "^2.0.0"
  },
  "peerDependenciesMeta": {
    "reka-ui": { "optional": true }   // CLI 默认自动装,故 optional
  },
  "devDependencies": {
    "tailwindcss": "^3.4.0"            // 构建期,不进发布
  },
  "exports": {
    "./styles.css": "./dist/styles.css",
    "./registry/*": "./registry/*"     // 预留模式 A
  }
}
```

> Tailwind 不写进 peerDependencies(构建工具,非运行时依赖);README 与 CLI prominently 标注「要求项目已配置 Tailwind,或 import 预编译 styles.css」。

---

## 6. v1 组件清单(12 个)

| 类 | widget | 备注 |
|---|---|---|
| 表单 | `button` `input` `textarea` `checkbox` `switch` `label` | button 含 variant/size |
| 布局 | `card` `separator` | |
| 反馈 | `badge` `avatar` | |
| 覆盖/导航 | `dialog` `tabs` | |

各 widget 的 prop/variant 由 AURA widget spec 驱动;视觉配方参考 shadcn-vue 对应组件。

---

## 7. License / 归属

- **包 LICENSE**:MIT(与仓库一致,Copyright Soutek Co. Ltd.)
- **新增 `NOTICES` 文件**:列出
  - reka-ui — MIT — Copyright © Radix Vue / reka-ui contributors
  - shadcn-vue — MIT — Copyright © shadcn-vue contributors
  - Tailwind CSS — MIT
- **每个 registry `.vue` 顶部注释**:
  ```vue
  <!-- Generated by AutoUI from widgets/<name>.at.
       Visual layer derived from shadcn-vue (MIT). See NOTICES. -->
  ```
- **README「Credits」段**:based on reka-ui & shadcn-vue

---

## 8. CLI 行为

### 8.1 `npx @auto-ui/widgets add <widget>`

1. 读包内 `registry/<widget>/*.vue`,拷进用户 `src/components/ui/<widget>/`
2. 检测包管理器(npm/pnpm/bun);reka-ui 缺失则**自动安装**(已有则跳过)
3. 提示 Tailwind:
   - 用户有 Tailwind → 帮忙把组件路径加进 `tailwind.config` 的 `content`
   - 用户无 Tailwind → 引导 `import '@auto-ui/widgets/styles.css'`(零配置路径)
4. flags:
   - `--no-install`:不自动装 reka-ui
   - `--reka-ui <pkg>`:用自定义包,拷贝时**重写 import 路径**(`'reka-ui'` → `'@你的/fork'`)
   - `--no-styles`:不引入预编译 CSS

### 8.2 `npx @auto-ui/widgets list`

列出可用 widget。

### 8.3 Tailwind 二选一约束

预编译 `styles.css` 与用户自带 Tailwind **只能二选一**(两套 `.flex` 等会重复/冲突)。CLI 在 `add` 时明确询问,用户通过「是否 import styles.css」切换:
- **路径 A(零配置)**:import 我们的 `styles.css`,不跑自己的 Tailwind
- **路径 B(自定义)**:跑自己的 Tailwind,不 import 我们的 CSS

---

## 9. gallery 角色

- `examples/component-gallery/` → 改名 `examples/gallery/`
- 转为 `@auto-ui/widgets` 的 **dogfood 消费者**:用 `npx @auto-ui/widgets add` 引入组件,验证发布物
- 不再把 gallery 当作发布库(它是展示型应用,生命周期与发布库不同)

---

## 10. 未来模式 A 扩展点(v1 不实现,留架构)

- `dist/index.js` 预留:模式 A 时把 registry 编译为可 `import` 的 bundle(reka-ui 打包,`styles.css` 已就绪)
- `package.json` `exports` 预留 `'./styles.css'`、`'./registry/*'` 子路径
- gallery 消费方式:模式 B 下 `npx add`,模式 A 下改 `import` 即可切换——不破坏现有包

---

## 11. 风险

| 项 | 风险 | 缓解 |
|---|---|---|
| `ui_gen/vue.rs` 渲染后端改造工作量 | 每个 widget 一段 Rust 模板,12 个是实活 | v1 严守 12 个最小集,先打通 1-2 个验证模板可复用 |
| Tailwind 预编译 CSS 与用户 Tailwind 冲突 | 重复 class 打架 | CLI 明确二选一提示 |
| reka-ui 版本漂移 | 用户项目版本与包期望不符 | 声明 peer 范围,CLI 装匹配版本 |
| gallery 改名牵动引用 | 其他示例/文档引用旧路径 | 改名时全局搜替换 `component-gallery` |

---

# 第二部分:实施计划

> **Repo rules (CLAUDE.md):** 在专用 worktree 开发;改 codegen/CLI 后跑 `cargo build -p auto`;改 VM/codegen 后跑 `cargo test`。
>
> **Goal:** 发布 `@auto-ui/widgets` v0.1 到 npm——12 个 AutoUI 生成的自包含 Vue 3 SFC 原语,经 `npx @auto-ui/widgets add <widget>`(shadcn 式拷贝)安装。
>
> **Architecture:** 给现有 `VueGenerator` 加第三个 `VueMode::Library`,为每个 widget 发射一个独立 `.vue`(reka-ui import + Tailwind class,无 `@/components/ui/*`)。新子命令 `auto ui build --target vue` 遍历 `WidgetRegistry`,把 `.vue` 写入 `packages/widgets/registry/`。Node/TS CLI(`npx @auto-ui/widgets add`)把它们拷进用户项目并自动装 `reka-ui`。预编译 `dist/styles.css` 给零配置用户提供样式。
>
> **Tech Stack:** Rust(codegen + CLI)、Vue 3 + reka-ui + Tailwind(运行时)、Node/TypeScript(消费端 CLI)、npm(发布)。

## Pre-flight: Worktree

按 CLAUDE.md,所有计划工作在隔离 worktree 进行,不在 `master`。

```bash
git worktree add ../auto-lang-331 plan-331/autoui-vue-widgets
cd ../auto-lang-331
```

后续任务均在此执行。整计划 build + 测试通过后才合并回 `master`。

---

## Phase 1 — Codegen:自包含 `VueMode::Library`

**探索所得上下文:**
- `crates/auto-lang/src/ui_gen/vue.rs` — `VueGenerator`(struct ~L768)、`VueMode` 枚举(~L757:`Plain`/`Shadcn`)、`map_tag()`(~L2942)、`generate_sfc()`(~L1133)、`generate_shadcn_imports()`(~L7409)。测试在 ~L7760。
- `crates/auto-lang/src/ui_gen/widget/registry.rs` — `WidgetRegistry`;`spec.rs` — `WidgetSpec`/`BackendMapping`(component/import/props/events/extra_components)。

现有生成器为每个 `AuraWidget`(整个 app/page)发射一个 SFC,内部 import 共享 shadcn-vue 原语。库需要相反:每个**原语**一个 SFC。故新增一个入口,接收原语名 + 其 `BackendMapping`,发射独立组件,由新模式驱动。

### Task 1.1: 加 `VueMode::Library` 变体

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs`(VueMode 枚举 ~L757、`is_shadcn`/mode helpers)

**Step 1: 写失败测试**(追加到现有 mode 测试 ~L7866 附近)

```rust
#[test]
fn test_library_mode_constructor() {
    let gen = VueGenerator::new_library();
    assert!(gen.is_library());
    assert!(!gen.is_shadcn());
}
```

**Step 2: 运行验证失败**

`cargo test -p auto-lang -- vue::test_library_mode_constructor`
Expected: FAIL — `new_library` / `is_library` not found.

**Step 3: 实现**

- 给 `pub enum VueMode { Plain, Shadcn, Library }` 加 `Library`。
- 加 `pub fn new_library() -> Self { Self::with_mode(VueMode::Library) }`。
- 加 `pub fn is_library(&self) -> bool { matches!(self.mode, VueMode::Library) }`。

**Step 4: 运行验证通过**

`cargo test -p auto-lang -- vue::test_library_mode_constructor` → PASS。然后 `cargo build -p auto-lang` → 干净。

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui_gen/vue.rs
git commit -m "feat(ui_gen): add VueMode::Library for self-contained widget output"
```

---

### Task 1.2: 每个 widget 的独立 SFC 入口

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs`(在 `VueGenerator` 上加方法)
- Modify: `crates/auto-lang/src/ui_gen/mod.rs` / `lib.rs` — 必要时 re-export

**Goal:** 函数 `generate_widget_sfc(&mut self, name: &str) -> GenResult<String>`,为一个原语(如 `button`)产出完整独立 `.vue`,从 `reka-ui` import(绝不 `@/components/ui/*`)。

**Step 1: 写失败测试**

```rust
#[test]
fn test_library_button_sfc_is_self_contained() {
    let mut gen = VueGenerator::new_library();
    let sfc = gen.generate_widget_sfc("button").unwrap();
    assert!(sfc.contains("<template>"), "has template");
    assert!(sfc.contains("<script setup"), "has script setup");
    assert!(!sfc.contains("@/components/ui/"), "must NOT import shadcn-vue");
    assert!(sfc.contains("reka-ui"), "uses reka-ui as backend");
}
```

**Step 2: 运行验证失败**(`no method generate_widget_sfc`)。

**Step 3: 实现(最小)**

返回硬编码 button SFC 字符串使测试通过(1.4 再泛化)。button 模板:

```rust
pub fn generate_widget_sfc(&mut self, name: &str) -> GenResult<String> {
    // Phase 1.2: button only; generalized in 1.4 via registry lookup.
    debug_assert_eq!(name, "button"); // removed in 1.4
    Ok(r#"<script setup lang="ts">
import { computed } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '../utils'
import { buttonVariants } from './variants'
import type { ButtonVariants } from './variants'

const props = withDefaults(defineProps<{
  variant?: ButtonVariants['variant']
  size?: ButtonVariants['size']
  class?: string
  as?: string
  asChild?: boolean
}>(), { variant: 'default', size: 'default', as: 'button' })
</script>

<template>
  <Primitive :as="as" :as-child="asChild" :class="cn(buttonVariants({ variant, size }), props.class)">
    <slot />
  </Primitive>
</template>
"#.to_string())
}
```

**Step 4: 运行验证通过。** `cargo test -p auto-lang -- vue::test_library_button_sfc_is_self_contained` → PASS。

**Step 5: Commit** `feat(ui_gen): generate_widget_sfc entry point (button stub)`

---

### Task 1.3: 配套文件(variants.ts、utils.ts)

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` — 加 `generate_widget_support_files(&self, name: &str) -> Vec<(String, String)>`,返回 `(相对路径, 内容)` 对(如 `variants.ts`、`index.ts`)。

上面 button SFC import 了 `./variants` 和 `../utils`。生成器必须同时发射这些配套文件,使拷贝出的组件自包含。

**Step 1: 失败测试** — 断言 `generate_widget_support_files("button")` 返回的条目路径含 `variants.ts` 和 `index.ts`,且 `index.ts` re-export `Button`。

**Step 2: 验证失败。**

**Step 3: 实现** — 发射:
- `index.ts`: `export { default as Button } from './Button.vue'`
- `variants.ts`: `buttonVariants` cva recipe + `ButtonVariants` 类型(用 `class-variance-authority`)。Tailwind class 字符串取自 shadcn-vue 参考配方。

**Step 4: 验证通过。**

**Step 5: Commit** `feat(ui_gen): emit per-widget support files (variants/index)`

---

### Task 1.4: 经 registry 查找泛化

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs` + `spec.rs`
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` — `generate_widget_sfc` 现读取 `name` 的 `WidgetSpec`,派发到每-widget 模板 fn。

**Goal:** 去掉 `debug_assert_eq!(name,"button")` 桩,改为对新方法 `widget_template(&self, name: &str) -> WidgetTemplate` 的 match,返回每-widget 的 script/template/support 内容。1.4 实现 `button` + `input` + `label`(三个)以验证模式可泛化;其余 widget 落在 Phase 5。

**Step 1: 失败测试** — `generate_widget_sfc("input")` 和 `generate_widget_sfc("label")` 各自产出独立 SFC(无 `@/components/ui/`、用 reka-ui 或原生 input)。

**Step 2: 验证失败。**

**Step 3: 实现** — 加 `WidgetTemplate` 结构 `{ script: String, template: String, support_files: Vec<(String,String)> }` 与 `fn library_template(name) -> Option<WidgetTemplate>` 表。`generate_widget_sfc` 查它,缺则返回明确错误(`unknown widget: {name}`)。

**Step 4: 验证通过** — 三个 widget 测试全绿;`cargo test -p auto-lang -- vue`。

**Step 5: Commit** `feat(ui_gen): registry-driven library templates (button/input/label)`

---

## Phase 2 — `auto ui build` 子命令

**上下文:** CLI 是 clap-derived,在 `crates/auto/src/main.rs`(枚举 ~L339、match ~L1107)。参考模式:`crates/auto/src/cmd_vue.rs`。

### Task 2.1: clap 枚举加 `Ui` 命令

**Files:**
- Modify: `crates/auto/src/main.rs`(~L339 加变体;~L1107 加 match arm)

**Step 1:** `Commands` 枚举加:

```rust
#[command(about = "AutoUI widget library commands")]
Ui {
    #[command(subcommand)]
    action: UiAction,
},
```

及 `UiAction` 枚举:`Build { target: String, out: String, widgets: Vec<String> }`(target 默认 `vue`,out 默认 `packages/widgets/registry`)。

**Step 2:** 加 match arm 调 `cmd_ui::build(action)`。

**Step 3:** `cargo build -p auto` → 干净。`auto ui --help` 打印子命令。

**Step 4: Commit** `feat(cli): add 'auto ui' subcommand scaffold`

---

### Task 2.2: 创建 `crates/auto/src/cmd_ui.rs` build action

**Files:**
- Create: `crates/auto/src/cmd_ui.rs`
- Modify: `crates/auto/src/main.rs` — `mod cmd_ui;`

**Goal:** `build(action)` 实例化 `VueGenerator::new_library()`,遍历请求的 widgets(或全部已注册),调 `generate_widget_sfc` + `generate_widget_support_files`,写 `<out>/<widget>/Widget.vue`(+ 配套文件)。

**Step 1: 手动集成测试**(暂无 harness)— 建 `tmp/ui_build_test/` 并运行:

```bash
cargo build -p auto
./target/debug/auto ui build --target vue --out tmp/ui_build_test --widgets button,input,label
ls tmp/ui_build_test   # 期望:button/ input/ label/
cat tmp/ui_build_test/button/Button.vue   # 期望独立 SFC
```

**Step 2: 实现 `cmd_ui::build`** — 每个 widget:`fs::create_dir_all`、写 SFC + 配套文件。打印汇总(`wrote N widgets to <out>`)。错误带上下文传播。

**Step 3:** 重跑 Step 1 命令 → 全过;验证输出无 `@/components/ui/`。

**Step 4:** 加 Rust 集成测试 `crates/auto/tests/cmd_ui.rs`,用 `std::process::Command` spawn 二进制(`--out <tempdir>`),断言期望文件存在且自包含。已 dev-dep `assert_cmd` 则用之,否则用 `std::process`。

**Step 5: Commit** `feat(cli): 'auto ui build' writes self-contained .vue widgets`

---

## Phase 3 — 包脚手架(`packages/widgets/`)

### Task 3.1: 目录 + package.json

**Files:**
- Create: `packages/widgets/package.json`、`packages/widgets/LICENSE`、`packages/widgets/NOTICES`、`packages/widgets/README.md`、`packages/widgets/.gitignore`、`packages/widgets/cli/tsconfig.json`

**Step 1:** `packages/widgets/package.json`:

```jsonc
{
  "name": "@auto-ui/widgets",
  "version": "0.1.0",
  "description": "AutoUI-generated Vue 3 component primitives (reka-ui + Tailwind).",
  "type": "module",
  "license": "MIT",
  "bin": { "auto-ui": "./cli/dist/index.js" },
  "files": ["registry", "dist", "cli/dist", "README.md", "LICENSE", "NOTICES"],
  "exports": {
    "./styles.css": "./dist/styles.css",
    "./registry/*": "./registry/*"
  },
  "peerDependencies": { "vue": "^3.4.0", "reka-ui": "^2.0.0" },
  "peerDependenciesMeta": { "reka-ui": { "optional": true } },
  "devDependencies": {
    "tailwindcss": "^3.4.0",
    "typescript": "^5.3.0"
  }
}
```

**Step 2:** `LICENSE` = 复制仓库根 MIT(Soutek Co. Ltd.)。

**Step 3:** `NOTICES` — 列 reka-ui、shadcn-vue、Tailwind CSS,各带 copyright + "MIT"。加注:"Visual layer of generated components is derived from shadcn-vue (MIT)."

**Step 4:** `README.md` — 安装/用法(add 命令)、Tailwind 二选一说明(import styles.css 或自带)、Credits。

**Step 5:** `.gitignore` — `node_modules/`、`cli/dist/`。(注意:`registry/` 和 `dist/styles.css` **需提交**——它们是发布产物,虽生成但纳入版本,使发布可复现、消费端无需构建。)

**Step 6:** Commit `feat(packages): scaffold @auto-ui/widgets package (license/notices/readme)`

---

### Task 3.2: 生成 `.vue` 顶部归属头

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` — `generate_widget_sfc` 为每个发射的 SFC 预置归属注释。

**Step 1: 失败测试** — 断言 SFC 以 `<!-- Generated by AutoUI` 开头且含 `shadcn-vue (MIT)`。

**Step 2–4:** 实现、验证、commit `feat(ui_gen): attribution header on generated widgets`。

---

## Phase 4 — Node/TS CLI(`add` / `list`)

**Goal:** `npx @auto-ui/widgets add <widget>` 把 `registry/<widget>/*` 拷进用户 `src/components/ui/<widget>/`,自动装 `reka-ui`,提示 Tailwind 路径。`list` 打印可用 widget。

### Task 4.1: CLI 骨架 + `list`

**Files:**
- Create: `packages/widgets/cli/src/index.ts`、`packages/widgets/cli/src/list.ts`、`packages/widgets/cli/package-build.json`(scripts)

**Step 1:** `list.ts` — 读 `registry/` 目录名并打印。

**Step 2:** 极简 `index.ts` argv 解析(无依赖;或 devDep 加 `commander`)派发 `list`/`add`。

**Step 3:** 构建:`tsc` → `cli/dist/index.js`。验证 `node cli/dist/index.js list` 打印 widget 名(Phase 5 填充 registry 前;现打印空/占位)。

**Step 4: Commit** `feat(cli): @auto-ui/widgets list command`。

---

### Task 4.2: `add` — 拷贝组件文件

**Files:**
- Create: `packages/widgets/cli/src/add.ts`

**Step 1:** `add <widget>`:
1. 解析包内 `registry/<widget>/`(经 `import.meta.url`)。
2. 解析用户目标:`<cwd>/src/components/ui/<widget>/`(允许 `--out`)。
3. 拷贝全部文件。widget 未知则报错(建议 `list`)。

**Step 2:** `tmp/` 手测:`mkdir tmp/consumer && cd tmp/consumer && node ../../cli/dist/index.js add button` → `src/components/ui/button/Button.vue` 存在。

**Step 3: Commit** `feat(cli): add command copies widget into consumer project`。

---

### Task 4.3: `add` — 自动装 reka-ui

**Files:**
- Modify: `packages/widgets/cli/src/add.ts`

**Step 1:** 检测包管理器(复用镜像 `auto-man::pkg` 的逻辑:pnpm>bun>npm,或读 `packageManager` 字段)。若用户 deps 无 `reka-ui`,运行安装(`pnpm add reka-ui` 等)。尊重 `--no-install`。支持 `--reka-ui <pkg>` → 改写拷贝文件里的 `from 'reka-ui'` import 路径。

**Step 2:** 手测——全新 `tmp/consumer2`,跑 `add button`,确认其 package.json 加入 `reka-ui`。测 `--no-install` 跳过。测 `--reka-ui @my/fork` 改写 import。

**Step 3: Commit** `feat(cli): add auto-installs reka-ui (with --no-install / --reka-ui overrides)`。

---

### Task 4.4: `add` — Tailwind 提示 + `--no-styles`

**Files:**
- Modify: `packages/widgets/cli/src/add.ts`

**Step 1:** 检测用户 `tailwind.config.*`:
- 有 → 打印提醒把拷贝的 `.vue` 路径纳入 `content`(或尝试打补丁;v1 最小 = 仅打印提醒)。
- 无 → 打印指引:`import '@auto-ui/widgets/styles.css'`(零配置路径)。
- `--no-styles` → 完全跳过 styles 指引。

**Step 2:** 手测两条分支。

**Step 3: Commit** `feat(cli): add tailwind guidance + --no-styles flag`。

---

## Phase 5 — 其余 9 个 widget

Task 1.4 已发 button/input/label。v1 余下:`textarea`、`checkbox`、`switch`、`card`、`separator`、`badge`、`avatar`、`dialog`、`tabs`。

每个 widget 一个 task,遵循 **Widget 模板模式**:

### Widget 模板模式(每 widget 重复)

**Files:** Modify `crates/auto-lang/src/ui_gen/vue.rs` `library_template` 表。

**Step 1: 失败测试**

```rust
#[test]
fn test_library_<widget>_sfc() {
    let mut gen = VueGenerator::new_library();
    let sfc = gen.generate_widget_sfc("<widget>").unwrap();
    assert!(!sfc.contains("@/components/ui/"));
    assert!(sfc.contains("<template>") && sfc.contains("<script setup"));
    // widget 专属:如 dialog 用 reka-ui DialogRoot/DialogTrigger
}
```

**Step 2:** 验证失败(`unknown widget: <widget>`)。

**Step 3:** 加 `library_template` 条目——script(reka-ui import + props)、template(按 shadcn-vue 参考配方的 Tailwind class)、support 文件(variants/index)。归属头自动预置。

**Step 4:** 验证通过 + `cargo test -p auto-lang -- vue::test_library`。

**Step 5:** 重生成 + 视觉检查:`auto ui build --target vue --out tmp/ui_build_test --widgets <widget>`;打开 SFC 肉眼校对。

**Step 6:** Commit `feat(ui_gen): <widget> library template`。

**Widgets(各一笔提交):** `textarea`、`checkbox`、`switch`、`card`、`separator`、`badge`、`avatar`、`dialog`、`tabs`。

全部完成后:提交重新生成的 `packages/widgets/registry/` 内容。

---

## Phase 6 — 预编译 `dist/styles.css`

**Goal:** 零配置用户 `import '@auto-ui/widgets/styles.css'` 即得全部 widget 样式。

### Task 6.1: 对 registry 跑 Tailwind 构建

**Files:**
- Create: `packages/widgets/tailwind.config.cjs`、`packages/widgets/build-styles.js`(或 `.cjs`)

**Step 1:** `tailwind.config.cjs` — `content: ['./registry/**/*.vue', './registry/**/*.ts']`、默认 theme(shadcn slate base color CSS 变量)、`plugins: [require('tailwindcss-animate')]`。

**Step 2:** `build-styles.js` — shell `npx tailwindcss -i ./src/input.css -o ./dist/styles.css --minify`(input.css = `@tailwind` 指令 + shadcn CSS 变量 `:root`/`.dark` 块)。

**Step 3:** 运行 `node build-styles.js` → `dist/styles.css` 存在、非空、含 button class。

**Step 4:** 接入 package `scripts.build`:`"build:css": "node build-styles.js"`。

**Step 5:** 提交生成的 `dist/styles.css` + 配置。

---

## Phase 7 — Dogfood:gallery 改名 + 消费

### Task 7.1: 改名 `examples/component-gallery` → `examples/gallery`

**Files:** `git mv examples/component-gallery examples/gallery`;全局搜替换路径(`grep -r component-gallery`)。

**Step 1:** `git mv`,再 `grep -rn "component-gallery" --include=*.md --include=*.rs --include=*.json --include=*.ts .` 更新引用。

**Step 2:** 验证 gallery 仍能跑(`cd examples/gallery/vue && pnpm install && pnpm dev`)——构建绿。

**Step 3:** Commit `chore: rename component-gallery -> gallery`。

---

### Task 7.2: gallery 消费 `@auto-ui/widgets`

**Files:** Modify `examples/gallery/vue/package.json`(加 `"@auto-ui/widgets": "file:../../packages/widgets"`),再用 `npx` 把 v1 集合的 gallery 手维护 `src/components/ui/<widget>` 替换为包内拷贝。

**Step 1:** 加本地 file dep。`pnpm install`。

**Step 2:** 每个 v1 widget 跑 `pnpm exec auto-ui add <widget>`(或 `node ../../packages/widgets/cli/dist/index.js add <widget>`),覆盖 gallery 的 shadcn-vue 版本。确认页面仍渲染。

**Step 3:** 跑 gallery dev server,视觉验证每个 v1 widget 页面(button/input/.../dialog/tabs)显示正确。

**Step 4:** Commit `feat(gallery): consume @auto-ui/widgets (dogfood)`。这是包在真实 Vue 项目可用的端到端验证。

---

## Phase 8 — 发布就绪

### Task 8.1: Dry-run 发布

**Step 1:** 在 `packages/widgets/`:`npm pack` → 检视 tarball 内容(`tar tzf *.tgz`)。确认**仅** `registry/`、`dist/`、`cli/dist/`、README、LICENSE、NOTICES 被纳入——无源 `cli/src/`、无 `node_modules`。

**Step 2:** `npm publish --dry-run` → 确认无错、peer deps 正确、`bin` 可解析。

**Step 3:** 在 `packages/widgets/README.md` 记录发布流程("Maintainer release"):bump version → `pnpm build:css` → 重生成 registry(`auto ui build`)→ `npm publish`。

**Step 4:** Commit `docs(packages): publish procedure + verified npm pack contents`。

(未获用户明确批准,**不**执行真实 `npm publish`——不可逆的外向动作。)

---

## Definition of Done (v0.1)

- [ ] `auto ui build --target vue` 为全部 v1 widget 生成自包含 SFC(无 `@/components/ui/` import)。
- [ ] `packages/widgets/registry/` 填充;`dist/styles.css` 可构建。
- [ ] `npx @auto-ui/widgets list` / `add <widget>` 可用(拷贝 + reka-ui 自动装 + tailwind 指引)。
- [ ] `examples/gallery` 消费该包并正确渲染全部 v1 widget。
- [ ] `npm pack` / `npm publish --dry-run` 干净;`files` 白名单紧;LICENSE + NOTICES 齐;每 `.vue` 带归属头。
- [ ] 全部 `cargo test -p auto-lang -- vue` 绿;`cargo build -p auto` 绿。
- [ ] worktree 分支在 build + 测试绿后合并回 `master`。
