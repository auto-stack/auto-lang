# Plan 336: examples/vue-gallery — @auto-ui/widgets 包展示页(dogfood 消费者)

> **类型**:完整计划(设计 + 实施)
> **状态**:设计待确认,实施未开始
> **日期**:2026-06-26
> **前身**:[331-autoui-vue-widgets-npm-library-design.md](331-autoui-vue-widgets-npm-library-design.md)(发布 `@auto-ui/widgets` npm 库)
> **关联**:324(widget-library-strategy)、319(unify-vm-rust-rendering)
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

# 第一部分:设计

## 1. 定位与目标

### 1.1 一句话定位

`examples/vue-gallery/` 是一个 **Vite + Vue 3 + TS** 应用,**直接 import `@auto-ui/widgets` 包**(registry 子路径 + 预编译 `styles.css`),把包里每个 widget 用其全部 variant / size / 状态渲染出来。它是 `@auto-ui/widgets` 的:

1. **视觉验证 harness** —— 每个 codegen 生成的组件能正确渲染(肉眼回归)
2. **最忠实的 dogfood 消费者** —— 证明包能被真实 Vite 项目消费、`styles.css` 生效
3. **活文档** —— 给包用户看每个 widget 长啥样、有哪些 variant、怎么用

### 1.2 v1 目标

1. 跑通:`pnpm dev` 起来,12 个 v1 widget 每页渲染正确
2. **零 Tailwind 配置**:`import '@auto-ui/widgets/styles.css'` 单独承担样式(顺带验证 Plan 331 §6 的预编译 CSS 路径)
3. `pnpm build` 绿(可挂 CI 回归)
4. README 讲清定位 + 与 `examples/gallery` 的区别 + 如何加新 widget 页

### 1.3 非目标(v1)

- 不展示 Auto 源码 / 不做 Auto↔Vue 对照(那是 `examples/gallery` 的职责)
- 不做交互式 props playground(改 props 实时预览)——v1 只静态展示 variant 矩阵
- 不发布到 npm / 不部署到 GitHub Pages(v1 仅本地 + CI build)
- 不替代 `examples/gallery`(后者继续做 Auto 语言展示)

---

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **位置** | `examples/vue-gallery/`(与 `examples/gallery` 并列) | 符合 repo `examples/` 约定;两个 gallery 目的不同,并列清晰 |
| **消费方式** | 直接 `import { X } from '@auto-ui/widgets/registry/<widget>'` + `import styles.css` | 展示页要**永远反映包内当前状态**;`auto-ui add` 拷贝会产生漂移(registry 重生成后拷贝过时) |
| **模式 A 边界澄清** | 逐 `.vue` 经 `exports: "./registry/*"` 子路径在 Vite dev 下直接编译,**可用** | Plan 331 里"模式 A v1 不实现"指**编译成单 bundle**(`dist/index.js`);逐文件源码导入不在该限制内 |
| **样式路径** | 走 `dist/styles.css`(零配置),**不**自带 Tailwind | 顺带验证 Plan 331 §8 的 Tailwind 二选一"路径 A";保持展示页零配置 |
| **依赖来源** | `"@auto-ui/widgets": "file:../../packages/widgets"`(file dep) | monorepo 内本地联调,改包后 vite HMR 即时反映 |
| **路由** | `vue-router`,每 widget 一页(`/<widget>`) | 标准、可扩展;未来加 widget 只加一条路由 |
| **文档表来源** | v1 手维护每页的 props/variant 表 | registry 的 variants.ts 无结构化元数据,v1 不引入元数据抽取 |

---

## 3. 与 examples/gallery 的边界(避免混淆)

| | `examples/gallery`(原 component-gallery) | `examples/vue-gallery`(本计划) |
|---|---|---|
| 展示对象 | **Auto 源码** → 生成的 Vue(对照) | `@auto-ui/widgets` 包里的 **Vue 组件** |
| 受众 | 学 Auto 语言 / 看转译器 | 用这个 npm 包的开发者 |
| 示例代码形态 | Auto(`button "txt"`) | 直接 `<Button variant="...">` |
| 是否对照 Auto↔Vue | 是 | 否 |
| 样式 | 生成的 Tailwind 项目 | 包的 `styles.css`(零配置) |

**结论**:两者并存,互不重构。`examples/gallery` 是 Auto 语言展示;`examples/vue-gallery` 是 npm 包展示。

---

## 4. 目录结构

```
examples/vue-gallery/
├── package.json            # @auto-ui/widgets(file dep) + vue + vue-router + reka-ui;devDeps: vite, @vitejs/plugin-vue, vue-tsc, typescript
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
├── index.html
├── README.md               # 定位 + 与 gallery 区别 + 加 widget 页流程
└── src/
    ├── main.ts             # createApp + router + import '@auto-ui/widgets/styles.css'
    ├── App.vue             # 布局:侧栏(widget 列表) + 主区(<RouterView>)
    ├── router.ts           # /, /button, /input, ... 每 widget 一条
    ├── components/
    │   ├── DemoBlock.vue   # 标题 + 描述 + 预览 + 代码片段的展示壳
    │   └── PropTable.vue   # props/variant 表(手维护数据)
    └── pages/
        ├── Home.vue        # widget 总览(分组卡片)
        ├── button.vue, input.vue, textarea.vue, checkbox.vue, switch.vue,
        ├── label.vue, card.vue, separator.vue, badge.vue, avatar.vue,
        └── dialog.vue, tabs.vue
```

### 4.1 package.json 关键字段

```jsonc
{
  "name": "vue-gallery",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@auto-ui/widgets": "file:../../packages/widgets",
    "reka-ui": "^2.0.0",
    "vue": "^3.4.0",
    "vue-router": "^4.3.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "typescript": "^5.4.0",
    "vite": "^5.2.0",
    "vue-tsc": "^2.0.0"
  }
}
```

> 不写 Tailwind 依赖(走 `styles.css`)。reka-ui 是 `@auto-ui/widgets` 的 optional peer,展示页实际装上。

---

## 5. 每页展示模式

每个 widget 页统一结构:

```
<DemoBlock title="Variants">
  <Button variant="default">Default</Button>
  <Button variant="destructive">Destructive</Button>
  ...每个 variant 一例
</DemoBlock>

<DemoBlock title="Sizes">  <!-- 仅对有 size 的 widget -->
  ...
</DemoBlock>

<DemoBlock title="States">
  <Button disabled>Disabled</Button>
  ...
</DemoBlock>

<PropTable :rows="[...]" />  <!-- 手维护的 props 表 -->
```

分组导航(侧栏):
- 表单:button / input / textarea / checkbox / switch / label
- 布局:card / separator
- 反馈:badge / avatar
- 覆盖/导航:dialog / tabs

---

## 6. CI / 回归

- **最小**:`pnpm build` 绿(`vue-tsc --noEmit` + `vite build`)——保证类型 + 可构建。
- **挂载点**:可加一个独立 workflow `.github/workflows/build-vue-gallery.yml`(push 到 `examples/vue-gallery/**` 或 `packages/widgets/registry/**` 时触发),或并入现有 gallery deploy。v1 倾向独立 workflow、只 build 不 deploy。
- **不包含**:pixel/visual regression(超出 v1)。

---

## 7. 风险

| 项 | 风险 | 缓解 |
|---|---|---|
| registry 子路径在 vite 下解析 | `exports: "./registry/*"` 是否被 vite 正确解析 | Phase 1 先打通单个 Button,确认 resolve 再铺开 |
| `styles.css` 与 vite 的 CSS 注入顺序 | 预编译 CSS 可能被 vite 进一步处理 | Phase 1 验证;必要时在 main.ts 顶部 import |
| 包改了之后展示页漂移 | 包 API 变化导致展示页编译失败 | 这正是 dogfood 价值——CI build 立刻暴露;file dep + HMR 让本地即时反映 |
| 手维护 props 表过时 | registry 无结构化元数据 | v1 接受;未来 Plan 可加 widget spec 元数据生成 |

---

# 第二部分:实施计划

> **Repo rules (CLAUDE.md):** 在专用 worktree 开发;前端改动用 `pnpm build` 验证。
>
> **Goal:** `examples/vue-gallery/` 跑起来——Vite + Vue 3 + TS 应用,直接消费 `@auto-ui/widgets`(registry 子路径 + styles.css),渲染全部 12 个 v1 widget 的 variant/状态矩阵,`pnpm build` 绿。
>
> **Architecture:** Vite 应用,`@auto-ui/widgets` 经 `file:` dep 引入;`main.ts` import `styles.css` 走零配置样式;`vue-router` 每 widget 一页;统一 `DemoBlock`/`PropTable` 展示壳。
>
> **Tech Stack:** Vite + Vue 3 + TypeScript + vue-router(reka-ui 经包传递)。

## Pre-flight: Worktree

```bash
git worktree add -b plan-336/vue-gallery ../auto-lang-336
cd ../auto-lang-336
```

后续任务均在此执行。整计划 `pnpm build` 绿后才合并回 `master`。

---

## Phase 1 — 脚手架 + 单 widget 打通

**目标**:Vite + Vue + TS 起来,`import { Button } from '@auto-ui/widgets/registry/button'` + `styles.css` 能渲染一个 Button。先验证 registry 子路径 + 预编译 CSS 在 vite 下工作(风险 §7 前两项)。

### Task 1.1: 项目骨架

**Files:** Create `examples/vue-gallery/{package.json, vite.config.ts, tsconfig.json, tsconfig.node.json, index.html, src/main.ts, src/App.vue}`

**Step 1:** 按 §4.1 写 `package.json`。`vite.config.ts`:
```ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
export default defineConfig({ plugins: [vue()] })
```
`tsconfig.json` 继承 `@vue/tsconfig/tsconfig.dom.json`,`include: ["src/**/*.ts","src/**/*.vue"]`。

**Step 2:** `src/main.ts`:
```ts
import { createApp } from 'vue'
import '@auto-ui/widgets/styles.css'
import App from './App.vue'
createApp(App).mount('#app')
```

**Step 3:** `src/App.vue`(最小):
```vue
<script setup lang="ts">
import { Button } from '@auto-ui/widgets/registry/button'
</script>
<template>
  <div style="padding:2rem">
    <Button variant="default">It works</Button>
    <Button variant="destructive">Destructive</Button>
  </div>
</template>
```

**Step 4:** `cd examples/vue-gallery && pnpm install`(或 npm)→ 装依赖。
**Step 5:** `pnpm dev` → 浏览器看到两个 button,**样式正确**(primary 实心、destructive 红底)。验证:改 `packages/widgets/registry/button/variants.ts` 的某个 class → HMR 反映。
**Step 6:** Commit `feat(vue-gallery): scaffold Vite app consuming @auto-ui/widgets (button)`。

---

### Task 1.2: 构建绿

**Step 1:** `pnpm build` → `vue-tsc --noEmit && vite build` 全过,`dist/` 产出。
**Step 2:** 若 `vue-tsc` 报 registry 组件类型错 → 记录并修(可能暴露 Plan 331 遗漏的类型问题)。
**Step 3:** Commit `build(vue-gallery): type-check + vite build green`。

---

## Phase 2 — 路由 + 展示壳

### Task 2.1: 路由与布局

**Files:** `src/router.ts`、`src/App.vue`(改)、`src/pages/Home.vue`、`src/components/DemoBlock.vue`、`src/components/PropTable.vue`

**Step 1:** `router.ts` —— `createRouter` history mode,路由表:`/` → Home,`/<widget>` → 动态或逐条 import 各 page。
**Step 2:** `App.vue` —— 左侧栏(分组 widget 列表,`<RouterLink>`) + 主区 `<RouterView>`;顶部加一行说明"本页直接消费 `@auto-ui/widgets`"。
**Step 3:** `DemoBlock.vue` —— props: `title`、`description?`;slot 为预览区;下方可选 `<pre>` 代码片段。
**Step 4:** `PropTable.vue` —— props: `rows: {prop, type, default, desc}[]`,渲染 `<table>`。
**Step 5:** `Home.vue` —— 分组卡片(表单/布局/反馈/覆盖),每卡片列 widget 链接。
**Step 6:** `pnpm dev` → 侧栏可导航,Home 显示分组。`pnpm build` 绿。
**Step 7:** Commit `feat(vue-gallery): router + DemoBlock/PropTable + Home`。

---

## Phase 3 — 12 个 widget 展示页

每页遵循 §5 模式。按分组提交(4 个 commit,便于 review):

### Task 3.1: 表单类(button / input / textarea / checkbox / switch / label)

**Files:** `src/pages/{button,input,textarea,checkbox,switch,label}.vue` + `router.ts` 注册。

**每页内容**(逐 widget):
- `button`:全部 variant(default/destructive/outline/secondary/ghost/link)× 全部 size(default/sm/lg/icon)+ disabled
- `input`:default、带 placeholder、disabled、`model-value` 受控
- `textarea`:default、disabled
- `checkbox`:checked/unchecked、disabled
- `switch`:checked/unchecked、disabled
- `label`:基础、`for` 关联 input

**验证:** `pnpm dev` 逐页肉眼;`pnpm build` 绿。
**Commit:** `feat(vue-gallery): form widget pages (button/input/textarea/checkbox/switch/label)`。

### Task 3.2: 布局类(card / separator)

- `card`:Card + CardHeader/CardTitle/CardDescription/CardContent/CardFooter 组合示例
- `separator`:horizontal / vertical

**Commit:** `feat(vue-gallery): layout widget pages (card/separator)`。

### Task 3.3: 反馈类(badge / avatar)

- `badge`:全部 variant(default/secondary/destructive/outline)
- `avatar`:Avatar + AvatarImage + AvatarFallback(fallback 演示用故意坏 src)

**Commit:** `feat(vue-gallery): feedback widget pages (badge/avatar)`。

### Task 3.4: 覆盖/导航类(dialog / tabs)

- `dialog`:trigger + DialogContent(title/description/footer slots)、open/close
- `tabs`:Tabs + TabsList/TabsTrigger/TabsContent,多 tab 切换

**Commit:** `feat(vue-gallery): overlay/nav widget pages (dialog/tabs)`。

---

## Phase 4 — README

**Files:** `examples/vue-gallery/README.md`

**内容:**
- 定位(一句话)+ 与 `examples/gallery` 的区别表(搬 §3)
- 快速开始:`pnpm install && pnpm dev`
- 消费方式说明(为什么用 registry 子路径而非 `auto-ui add`)
- 样式说明(走 `styles.css`,不带 Tailwind)
- **加新 widget 页流程**(给未来贡献者):
  1. 确认 `packages/widgets/registry/<widget>/` 存在(否则先在 Plan 331 codegen 里加)
  2. `src/pages/<widget>.vue` 按 §5 模式写
  3. `router.ts` 加路由 + 侧栏分组注册
  4. `pnpm dev` 肉眼 + `pnpm build` 绿

**Commit:** `docs(vue-gallery): README with positioning + add-widget workflow`。

---

## Phase 5 — CI 回归(可选,推荐)

**Files:** `.github/workflows/build-vue-gallery.yml`

**Step 1:** workflow:on push 到 `examples/vue-gallery/**` 或 `packages/widgets/registry/**`;ubuntu + node 20 + `pnpm install` + `pnpm build`。
**Step 2:** 主分支 push 触发一次,确认绿。
**Step 3:** Commit `ci(vue-gallery): build regression on package/example changes`。

---

## Definition of Done (v1)

- [ ] `examples/vue-gallery/` 脚手架;`pnpm dev` 起得来。
- [ ] 单个 Button 经 `@auto-ui/widgets/registry/button` + `styles.css` 渲染正确(Phase 1 验证风险项)。
- [ ] `pnpm build` 绿(`vue-tsc --noEmit && vite build`)。
- [ ] 12 个 v1 widget 各一页,variant/状态矩阵齐全(肉眼)。
- [ ] 侧栏分组导航 + Home 总览。
- [ ] README 讲清定位、与 `examples/gallery` 区别、加 widget 页流程。
- [ ] (推荐)CI workflow `pnpm build` 绿。
- [ ] worktree 分支在 `pnpm build` 绿后合并回 `master`。
