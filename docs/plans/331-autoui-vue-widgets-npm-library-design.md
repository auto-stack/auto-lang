# Plan 331: @auto-ui/widgets — AutoUI 生成的 Vue 组件库(npm 发布)

> **类型**:设计文档(brainstorming 产出,待 writing-plans 转实施计划)
> **状态**:设计已确认,待出实施计划
> **日期**:2026-06-23
> **前身**:[324-autoui-widget-library-strategy.md](324-autoui-widget-library-strategy.md)(战略建议,识别了「缺少可发布的通用组件库」这一空白)
> **关联**:319(unify-vm-rust-rendering)、327(015-notes-vm-render)

---

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
  "bin": { "auto-ui": "./cli/index.js" },
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
| 反馈 | `badge` | |
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

## 11. 风险与待定

| 项 | 风险 | 缓解 |
|---|---|---|
| `ui_gen/vue.rs` 渲染后端改造工作量 | 每个 widget 一段 Rust 模板,12 个是实活 | v1 严守 12 个最小集,先打通 1-2 个验证模板可复用 |
| Tailwind 预编译 CSS 与用户 Tailwind 冲突 | 重复 class 打架 | CLI 明确二选一提示 |
| reka-ui 版本漂移 | 用户项目版本与包期望不符 | 声明 peer 范围,CLI 装匹配版本 |
| gallery 改名牵动引用 | 其他示例/文档引用旧路径 | 改名时全局搜替换 `component-gallery` |

---

## 下一步

转入 **writing-plans** skill,把本设计拆成可执行的实施计划(分阶段:① 改造 `ui_gen/vue.rs` 渲染后端 → ② `auto ui build` 子命令 → ③ 搭 `packages/widgets/` 骨架 + CLI → ④ 12 个 widget → ⑤ License/归属 → ⑥ gallery dogfood → ⑦ 发布)。
