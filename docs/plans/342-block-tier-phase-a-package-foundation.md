# Plan 342: Block 层 Phase A — block 包格式 + BlockRegistry + blocks-gallery 骨架

> **类型**:完整计划(实施)
> **状态**:设计待确认,实施未开始
> **日期**:2026-06-26
> **战略文档**:[docs/design/17-blocks-first-class.md](../design/17-blocks-first-class.md)(Skill 模型:spec + 参考实现集 + gotchas)
> **关联**:widget 层(Plan 331/336/337,提供 block 编排的调色板)、Plan 338(M1 的 note-list 页 = 一个 Data-display block 实例)
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

## 1. 目标

把 Design 17 的 **block 包**形态落地为可被工具消费的实物,并产出第一批真实 block 包作为模板。具体:

1. **定义 block 包格式**:`spec.md`(frontmatter + NL)+ `reference/<variant>.at`(每变体一份参考实现)+ `gotchas.md`(反例)。
2. **`BlockRegistry`**:扫描 block 包目录、按 `kind/name` 索引、供 CLI 与 gallery 消费(与 `WidgetRegistry` 同构)。
3. **`examples/blocks-gallery` 骨架**:像 vue-gallery 展示 widget 那样,展示每个 block 的 spec + 参考实现的**实时渲染**(经 a2vue)+ 变体切换 + gotchas。
4. **首批两个 block 包**(覆盖两类,且一个直接喂给 Plan 338 M1):
   - `form/login`(variants: `minimal`、`with_sso`)+ gotchas。
   - `data-display/note-list`(variant: `default`)—— M1 的 notes 列表页即其实例。

完成后:block 层有了与 widget 层对等的"载体"(registry + gallery + 包格式),为 Phase B(AI 生成器 + `auto block add`)铺路。

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **block 包位置** | 仓库根 `blocks/<kind>/<name>/` | 与 `examples/`、`packages/` 并列;block 是 Auto 源码目录(非 npm 包),`packages/` 暗示发布,不合 |
| **registry 实现** | `crates/auto-lang/src/ui_gen/block/{mod,registry,spec}.rs` | 与 `ui_gen/widget/` 同构;`BlockRegistry::with_defaults()` 扫 `blocks/` |
| **spec 格式** | Markdown + YAML frontmatter | 人/AI 都易读写;frontmatter 机器可检,NL 正文给 AI 组装指引 |
| **reference 渲染** | blocks-gallery 经 a2vue 编译 reference `.at` → Vue 实时预览 | 复用现有 a2vue;reference 本就是合法 Auto UI 源 |
| **变体组织** | 每变体一个 `reference/<variant>.at` | 变体 = 物化的柔性范围样本(Design 17 §2.4) |
| **gotchas 形态** | `gotchas.md`:每条 `### <反例标题>` + `错误`/`为什么`/`正确`(可带代码块) | 结构化反例,便于 AI 读取与 gallery 展示 |
| **CI** | 每个 reference `.at` 能 `auto build` 绿 + frontmatter 过 schema 校验 | reference 是 spec 的回归 fixture(Design 17 §2.4),必须可编译 |

## 3. 非目标(留给后续 Plan)

- AI 生成器与 `auto block add --from` 命令 → **Plan 343(Phase B)**。
- 全量 block 目录覆盖(每类多个)→ Phase C(由 Design 16 M1-M3 基准驱动)。
- 命名 slot / 运行时 block 复用 → Phase D(依赖 AURA 扩展)。
- `dataSource` 的类型系统细节 → Design 16 Rung 2。

---

## 实施计划

> **Repo rules (CLAUDE.md):** 在专用 worktree 开发;改 codegen 后 `cargo build -p auto`、`cargo test -p auto-lang`。
>
> **Goal:** block 包格式 + `BlockRegistry` + `examples/blocks-gallery` 骨架 + 首批 2 个 block 包(form/login、data-display/note-list)。
>
> **Architecture:** `blocks/<kind>/<name>/{spec.md, reference/*.at, gotchas.md}` ← `BlockRegistry`(Rust,扫目录)→ `examples/blocks-gallery`(Vite+a2vue 渲染)+ spec/reference CI。

## Pre-flight: Worktree

```bash
git worktree add -b plan-342/block-phase-a ../auto-lang-342
cd ../auto-lang-342
```

---

## Phase 1 — block 包格式规范 + schema 校验

### Task 1.1: 写格式规范文档

**Files:** `docs/design/blocks/block-package-format.md`(或 design 17 附录)

**内容:**
- 目录结构 `blocks/<kind>/<name>/{spec.md, reference/<variant>.at, gotchas.md}`。
- `spec.md` frontmatter schema:`kind`、`name`、`palette: []str`(引用 WidgetRegistry 名)、`extension_points: []str`、`dataSource: {…}`(签名描述)、`variants: []str`、`acceptance: []str`。
- NL 正文段:`# Intent` / `# 本 block 吸收的变化` / `# 组装指引` / `# References` / `# Gotchas`。
- `reference/<variant>.at`:合法 Auto UI 源(`scene: ui`),每个对应 frontmatter 的一个 variant。
- `gotchas.md`:每条 `### 标题` + `**错误**`/`**为什么**`/`**正确**`(可带 ```` ```auto ```` 代码)。

**Commit:** `docs(blocks): block package format specification`。

### Task 1.2: frontmatter schema 校验

**Files:** `crates/auto-lang/src/ui_gen/block/spec.rs`(`BlockSpec` 结构 + 从 spec.md 解析 + 校验 palette 项都在 WidgetRegistry、variants 与 reference 文件一一对应)。

**Step 1:** 解析 spec.md frontmatter(YAML)+ 必填字段校验。
**Step 2:** 交叉校验:每个 `variant` 必须有对应 `reference/<variant>.at`;`palette` 每项必须在 `WidgetRegistry` 中存在(漂移守卫,同 337 思路)。
**Step 3:** 单测:合法包通过;缺 reference / palette 拼错 → 报错。
**Commit:** `feat(block): parse + validate block package spec (frontmatter + cross-checks)`。

---

## Phase 2 — BlockRegistry

### Task 2.1: registry 扫描与索引

**Files:** `crates/auto-lang/src/ui_gen/block/{mod.rs, registry.rs}`

**Step 1:** `BlockRegistry::with_defaults()` 扫 `blocks/` 下所有包,解析为 `BlockPackage { spec: BlockSpec, references: HashMap<variant, path>, gotchas: Path }`。
**Step 2:** API:`iter()`、`get(kind, name)`、`list_by_kind(kind)`、`all_packages()`(同构 `WidgetRegistry`)。
**Step 3:** 暴露到 `ui_gen` 公共 API;`cargo build -p auto-lang` 绿。
**Commit:** `feat(block): BlockRegistry scans blocks/ and indexes by kind/name`。

---

## Phase 3 — 首批 block 包

### Task 3.1: form/login 包

**Files:** `blocks/form/login/{spec.md, reference/minimal.at, reference/with_sso.at, gotchas.md}`

**Step 1:** `spec.md`:frontmatter(kind=form, palette=[Input,Button,Label,Checkbox,Separator,ErrorState], extension_points=[fields,submit,third_party,success,error_display], dataSource={attempt:(creds)=>Session}, variants=[minimal,with_sso], acceptance=[…])+ NL 正文。
**Step 2:** `reference/minimal.at`:email+password+submit 的 LoginBox widget,接 `dataSource.attempt`,带 loading/error,标 EDIT 区。
**Step 3:** `reference/with_sso.at`:在 minimal 基础上加 SSO provider 按钮(third_party 扩展点物化)。
**Step 4:** `gotchas.md`:至少 3 条(端点写死、漏 loading/error、label 未关联)。
**Step 5:** 每个 reference `auto build`(单包)绿。
**Commit:** `feat(blocks): form/login package (minimal + with_sso references, gotchas)`。

### Task 3.2: data-display/note-list 包

**Files:** `blocks/data-display/note-list/{spec.md, reference/default.at, gotchas.md}`

**Step 1:** spec(kind=data-display, palette=[Input,Button,…], extension_points=[items,empty,loading,error,toolbar], dataSource={list: ()=>[]Note}, variants=[default])。
**Step 2:** `reference/default.at`:列表 + 搜索 + empty/loading/error 槽,接 `dataSource.list`。**与 Plan 338 M1 的 notes 列表页对齐**(M1 可直接采用此 block)。
**Step 3:** gotchas:3 条(数据源写死、漏 empty/loading、key 缺失)。
**Commit:** `feat(blocks): data-display/note-list package (default reference, gotchas)`。

---

## Phase 4 — examples/blocks-gallery 骨架

### Task 4.1: gallery 脚手架

**Files:** `examples/blocks-gallery/{package.json, vite.config.ts, src/main.ts, App.vue, router.ts}`

**Step 1:** Vite+Vue+TS,结构与 vue-gallery 同构;消费 `@auto-ui/widgets` styles.css + chrome CSS(可复用 vue-gallery 的 app.css 风格)。
**Step 2:** 侧栏按 kind 分组(form/data-display),列出 block。
**Step 3:** 从 `BlockRegistry`(经 `auto block export-gallery-meta` 生成 `widgets.generated.ts` 同构的 catalog)驱动导航。
**Commit:** `feat(blocks-gallery): scaffold + sidebar by kind`。

### Task 4.2: block 详情页(spec + 渲染 reference + gotchas)

**Files:** `examples/blocks-gallery/src/pages/BlockPage.vue`

**Step 1:** 每页:渲染 spec(意图/组装指引)+ **变体切换** → 对应 reference 经 a2vue 实时渲染 + gotchas 列表。
**Step 2:** reference 渲染:把 `reference/<variant>.at` 经 `auto build --target vue`(或预生成)产出 Vue 组件挂载。
**Step 3:** `pnpm dev` 肉眼:login 两变体可切换渲染;note-list 显示列表+空态。
**Commit:** `feat(blocks-gallery): block detail page (spec + rendered references + gotchas)`。

---

## Phase 5 — CI:reference 可编译 + spec 校验

### Task 5.1: 守卫测试

**Files:** `crates/auto-lang/src/ui_gen/block/registry.rs`(测试)+ 可能 `.github/workflows`

**Step 1:** Rust 测试:每个 block 包的每个 reference `.at` `auto build`(单包)绿;frontmatter 过 schema;palette 项都在 WidgetRegistry。
**Step 2:** (可选)workflow:push 到 `blocks/**` 时跑上述校验。
**Commit:** `test(blocks): CI guard — references compile + spec validates`。

---

## Definition of Done

- [ ] block 包格式规范文档就绪;frontmatter schema 有解析+校验+单测。
- [ ] `BlockRegistry` 扫 `blocks/`、按 kind/name 索引、有 iter/get/list_by_kind API。
- [ ] 两个 block 包(form/login 带 minimal+with_sso,data-display/note-list)就绪,含 spec + 每 variant reference + gotchas。
- [ ] 每个 reference `.at` `auto build` 绿;palette 漂移守卫通过。
- [ ] `examples/blocks-gallery` 骨架:侧栏分组、block 详情页(spec + 变体切换实时渲染 + gotchas)、`pnpm dev`/`pnpm build` 绿。
- [ ] 守卫测试(CI)覆盖 reference 编译 + spec 校验。
- [ ] worktree 分支在 build + 测试绿后合并回 `master`。
