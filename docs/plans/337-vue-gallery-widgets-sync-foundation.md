# Plan 337: vue-gallery ↔ @auto-ui/widgets 薄同步层

> **类型**:完整计划(设计 + 实施)
> **状态**:设计待确认,实施未开始
> **日期**:2026-06-26
> **前身**:[331](331-autoui-vue-widgets-npm-library-design.md)(@auto-ui/widgets)、[336](336-vue-gallery-autoui-widgets-showcase.md)(vue-gallery)
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

# 第一部分:设计

## 1. 背景与动机

现状有三处**独立维护**的 widget 清单,随 widget 增多必然漂移:

| 清单 | 位置 | 形态 |
|---|---|---|
| AURA widget registry | `crates/.../ui_gen/widget/registry.rs`(`WidgetRegistry::with_defaults()`) | ~60 个 widget(AURA 语言的真值源) |
| 库的 library templates | `crates/.../ui_gen/vue.rs` 的 `library_template` match + `LIBRARY_WIDGETS` const | 12 个(手写,**两份**分开维护) |
| vue-gallery 展示 | `examples/vue-gallery/src/widgets.ts`(目录)+ `src/pages/*.vue` | 12 页(手写) |

v1(12 widget)还能靠人肉对齐;目标 ~25-30 时,每加一个 widget 要同步改 **3-5 处**,漂移不可避免。

### 1.1 本计划的目标(薄同步)

把三处清单的**同步关系**做成显式、可机器检查、可半自动生成,而不是一上来做"从 AURA `WidgetSpec` 全自动派生 Vue SFC"那种深重构(那是后续 Plan 的事,现在做是投机)。

具体三件事:
1. **库内单一真值源**:`LIBRARY_WIDGETS` 与 `library_template` 自洽(一处定义、一处派生、测试兜底)。
2. **AURA ↔ 库 漂移守卫**:测试 + `auto ui backlog` 命令,把"哪些 AURA widget 还没进库"从**静默漂移**变成**显式 backlog**。
3. **库 ↔ vue-gallery 半自动**:`auto ui build` 能为每个库 widget 生成/校验 vue-gallery 页面骨架 + 目录,加 widget = 1 处 `library_template` 条目 + 重新生成。

### 1.2 非目标(留给后续 Plan)

- 从 AURA `WidgetSpec` 的 `BackendMapping` **自动派生**库的 Vue SFC(深 codegen 统一)。
- 把库做成 AURA registry 的 1:1 镜像(本次目标是有选型 ~25-30,非全集)。
- per-widget 的高保真示例与文档(那是 336 之后的 polish Plan)。

---

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **库清单真值源** | `LIBRARY_WIDGETS` 常量为唯一显式清单;`library_template` 必须覆盖它每一项 | match 的键无法运行时枚举,故由显式清单做源;测试反向断言"每项都有模板" |
| **复合 widget 覆盖判定** | AURA 标签 `t` 被"覆盖" ⟺ 存在库 widget `w` 使 `t == w` 或 `t.starts_with("{w}-")` | 一个库 `card` 自然覆盖 AURA 的 `card-content`/`card-header`/…;`alert-dialog` 覆盖 `alert-dialog-action` 等 |
| **backlog 形态** | `auto ui backlog` 命令打印未覆盖 AURA widget(按 category 分组);**不**让 CI 因 backlog 非空而失败 | 覆盖是**有选型**的(非目标=全集),backlog 是受管待办,非回归 |
| **漂移守卫测试(失败型)** | 断言"每个 `LIBRARY_WIDGETS` 都存在于 AURA registry" | 反向:库里出现 AURA 没有的名字 = 拼错/陈旧,该立刻失败 |
| **vue-gallery 页面生成** | `auto ui build --target gallery-stubs --out examples/vue-gallery` 为缺失的库 widget 生成**最小页骨架**;已存在的页**不覆盖** | 保护手写内容;只补缺。blurb/分组仍人肉维护(生成占位) |
| **vue-gallery 目录(widgets.ts)** | 拆为 `widgets.generated.ts`(名字+路由,生成)+ `widgets.meta.ts`(blurb+分组,人肉) | 机械部分自动、有意义的描述部分人肉;生成文件头部带"DO NOT EDIT" |

---

## 3. 漂移模型与检查矩阵

```
AURA WidgetRegistry  ──(backlog 命令:列出未覆盖)──▶  @auto-ui/widgets (LIBRARY_WIDGETS)
        │                                                    │
        │              (守卫测试:库项必须在 AURA)            │
        │                                                    │
        │                                                    ▼
        │              (auto ui build --target gallery-stubs)
        │                                         vue-gallery pages + widgets.generated.ts
        │
        └─ 复合覆盖规则:t == w  or  t.starts_with("{w}-")
```

| 检查 | 类型 | 失败含义 |
|---|---|---|
| `LIBRARY_WIDGETS` 每项 `generate_widget_sfc` 成功 | 单测 | 清单有条目但没模板(漏写 match arm) |
| `LIBRARY_WIDGETS` 每项 ∈ AURA registry 标签集 | 单测 | 库里出现 AURA 不认识的名字(拼错/陈旧) |
| `auto ui backlog` | 命令(人/CI 跑,非 fail) | 列出 AURA 有但库还没收的 widget(受管待办) |
| vue-gallery 每个 `widgets.generated.ts` 项有对应 page 文件 | 集成检查 | 加了库 widget 但忘了加展示页 |

---

# 第二部分:实施计划

> **Repo rules (CLAUDE.md):** 在专用 worktree 开发;改 codegen/CLI 后跑 `cargo build -p auto`;改 codegen 后跑 `cargo test -p auto-lang --lib -- test_library`。
>
> **Goal:** 三处 widget 清单的关系从"人肉对齐"升级为"一处真值 + 测试守卫 + 半自动生成",为扩到 ~25-30 widget 铺路。
>
> **Architecture:** `LIBRARY_WIDGETS`(唯一显式清单)→ 漂移守卫测试对照 AURA `WidgetRegistry::iter()` → `auto ui build --target gallery-stubs` 生成 vue-gallery 骨架。
>
> **Tech Stack:** Rust(测试 + CLI codegen)、TypeScript(vue-gallery 生成文件)。

## Pre-flight: Worktree

```bash
git worktree add -b plan-337/widgets-sync ../auto-lang-337
cd ../auto-lang-337
```

---

## Phase 1 — 库内单一真值源 + 自洽测试

**Files:** `crates/auto-lang/src/ui_gen/vue.rs`(测试,~现有 library 测试旁)

### Task 1.1: 自洽测试

**Step 1: 失败测试**(追加到 library 测试块)

```rust
#[test]
fn test_library_widgets_list_is_self_consistent() {
    // Every advertised widget must have a renderable template.
    for name in VueGenerator::LIBRARY_WIDGETS {
        let mut gen = VueGenerator::new_library();
        gen.generate_widget_sfc(name)
            .unwrap_or_else(|e| panic!("LIBRARY_WIDGETS lists '{name}' but template is missing: {e}"));
    }
    // Sorted + deduped (keeps the list tidy as it grows).
    let mut sorted = VueGenerator::LIBRARY_WIDGETS.to_vec();
    sorted.sort();
    assert_eq!(VueGenerator::LIBRARY_WIDGETS.to_vec(), sorted, "LIBRARY_WIDGETS must be sorted");
}
```

**Step 2:** 当前应**直接通过**(12 项都有模板且已有序)——本任务主要是把不变量钉住。验证 `cargo test -p auto-lang --lib -- test_library_widgets_list_is_self_consistent` 绿。

**Step 3:** Commit `test(ui_gen): pin LIBRARY_WIDGETS self-consistency invariant`。

---

## Phase 2 — AURA ↔ 库 漂移守卫 + backlog 命令

**Files:** `crates/auto-lang/src/ui_gen/vue.rs`(守卫测试)、`crates/auto/src/cmd_ui.rs` + `main.rs`(backlog 命令)

### Task 2.1: 覆盖判定 + 守卫测试(反向,失败型)

**Files:** vue.rs 新增 `pub fn covers_aura_tag(tag: &str) -> bool` 与测试。

**Step 1: 失败测试**

```rust
#[test]
fn test_library_widgets_exist_in_aura_registry() {
    let reg = WidgetRegistry::with_defaults();
    let aura_tags: std::collections::HashSet<&str> = reg.all_widgets().keys().map(|s| s.as_str()).collect();
    for w in VueGenerator::LIBRARY_WIDGETS {
        // library widget must be a known AURA tag (exact or prefix-grouped)
        let known = aura_tags.contains(*w) || aura_tags.iter().any(|t| t.starts_with(&format!("{w}-"));
        assert!(known, "LIBRARY_WIDGETS has '{w}' but AURA registry has no such widget");
    }
}
```

**Step 2:** 验证(当前 12 项应都在 AURA)→ 绿。

**Step 3:** 加 `pub fn covers_aura_tag(tag: &str) -> bool`(同上逻辑,供 CLI 复用)。

**Step 4:** Commit `test(ui_gen): drift guard — library widgets must exist in AURA registry`。

### Task 2.2: `auto ui backlog` 命令

**Files:** `cmd_ui.rs`(加 `Backlog` arm)、`main.rs`(`UiAction` 加变体)、`vue.rs` 暴露 `LIBRARY_WIDGETS`(已有)。

**Step 1:** `UiAction::Backlog { noisy: bool }`(默认打印)。

**Step 2:** `cmd_ui::backlog`:
- `WidgetRegistry::with_defaults()`,遍历 `all_widgets()` keys。
- 对每个 AURA tag,用 `VueGenerator::covers_aura_tag` 判定是否已被库覆盖。
- 未覆盖的按 `WidgetCategory`(从 `WidgetSpec.category`)分组打印。
- 末尾打印汇总:`X covered / Y total (Z uncovered)`。

**Step 3:** 手测:`auto ui backlog` → 打印分组列表(button/card/… 不出现;accordion/alert/calendar/… 出现)。

**Step 4:** Commit `feat(cli): 'auto ui backlog' lists AURA widgets not yet in @auto-ui/widgets`。

---

## Phase 3 — vue-gallery 半自动生成

**Files:** `crates/auto/src/cmd_ui.rs`(新 target)、`examples/vue-gallery/src/widgets.ts`(拆分)

### Task 3.1: 拆 widgets.ts 为 generated + meta

**Files:**
- Create `examples/vue-gallery/src/widgets.generated.ts`(`auto ui build` 生成,头部 `// DO NOT EDIT — generated by auto ui build`)。内容:名字 + 路由。
- Rename/trim 现有 `widgets.ts` → `widgets.meta.ts`:保留 blurb + group(人肉),keyed by widget name。
- 改 `App.vue`/`Home.vue`/`router.ts` 改为消费 generated(名字+路由)与 meta(blurb+group)的**合并**结果。

**Step 1:** 手写 `widgets.generated.ts` 的初始内容(与现有 12 项一致),让 build 绿,作为生成的目标格式样本。

**Step 2:** `widgets.meta.ts`:把现有 `widgets.ts` 的 group+blurb 改写成 `{ [name]: { group, blurb } }`。

**Step 3:** `widgets.ts`(新,薄封装):合并 generated + meta,导出 `widgetGroups`(App/Home 继续用),保持现有 API 不变。`pnpm build` 绿。

**Step 4:** Commit `refactor(vue-gallery): split widgets catalog into generated + meta`。

### Task 3.2: `auto ui build --target gallery-stubs`

**Files:** `cmd_ui.rs`(新 target 分支)、vue.rs(可能加 SFC 页骨架模板 helper)。

**Goal:** `auto ui build --target gallery-stubs --out examples/vue-gallery/src`:
- 读 `LIBRARY_WIDGETS`,重写 `widgets.generated.ts`(名字+路由,排序)。
- 对每个**尚无** `pages/<widget>.vue` 的 widget,生成最小骨架页(导入该 widget、一个 `<DemoBlock>` 占位、`<PropTable>` 空)。**已存在的页不覆盖**(保护手写内容)。

**Step 1:** 在 vue.rs 加 `pub fn gallery_page_stub(name: &str) -> String`,返回最小 `.vue` 源(参考现有页结构,只放一个 default DemoBlock + 占位 slot,代码注释 `// TODO: variants/states`)。

**Step 2:** `cmd_ui::build` 增加 `target == "gallery-stubs"` 分支:写 `widgets.generated.ts` + 逐 widget 写页(跳过已存在)。

**Step 3:** 手测:`auto ui build --target gallery-stubs --out examples/vue-gallery/src` → 报告 `regenerated widgets.generated.ts (12); pages existing: 12, created: 0`。删一页重跑 → `created: 1` 且内容是最小骨架。

**Step 4:** `pnpm build` 绿。

**Step 5:** Commit `feat(cli): 'auto ui build --target gallery-stubs' scaffolds vue-gallery pages`。

---

## Phase 4 — per-widget 安装提示 + 文档

**Files:** `examples/vue-gallery/src/components/DemoBlock.vue`(或新小组件)、README

### Task 4.1: 每页顶部"安装"小条

**Goal:** 每个 widget 页(或 DemoBlock)显示一行:`npx @auto-ui/widgets add <widget>`,带复制按钮(复用已有的 copy 机制)。

**Step 1:** 新组件 `InstallHint.vue`(props: `widget`),显示 `npx @auto-ui/widgets add {widget}` + Copy。在每页 `<h2>` 下渲染 `<InstallHint :widget="'button'" />`。(可由 Task 3.2 的骨架自动包含。)

**Step 2:** 各现有页补上 `<InstallHint>`(12 页,机械)。

**Step 3:** `pnpm build` 绿。

**Step 4:** Commit `feat(vue-gallery): per-widget install hint`。

### Task 4.2: README 记录同步工作流

**Files:** `examples/vue-gallery/README.md` + `packages/widgets/README.md`

**内容:** "加一个新 widget"端到端流程:
1. `library_template` 加条目 + `LIBRARY_WIDGETS` 加名(Phase 1 测试保自洽)。
2. `auto ui build --target vue --out packages/widgets/registry` 生成包产物。
3. `auto ui build --target gallery-stubs --out examples/vue-gallery/src` 生成展示页骨架。
4. 手写页内容(variants/states/code)+ `widgets.meta.ts` 的 blurb/group。
5. `auto ui backlog` 看还有哪些 AURA widget 待收。

**Commit:** `docs: end-to-end 'add a widget' workflow with sync tooling`。

---

## Definition of Done

- [ ] `LIBRARY_WIDGETS` 自洽测试绿;清单是 `library_template` 的真子集。
- [ ] 漂移守卫测试绿:每个库 widget 都在 AURA registry(精确或前缀复合覆盖)。
- [ ] `auto ui backlog` 命令可用,正确分组打印未覆盖 AURA widget。
- [ ] `widgets.ts` 拆为 generated(机器)+ meta(人肉);`App`/`Home`/`router` 消费合并结果;`pnpm build` 绿。
- [ ] `auto ui build --target gallery-stubs` 能重写 generated 目录 + 为缺页生成最小骨架(不覆盖已存在)。
- [ ] 每个 widget 页有 install hint;README 记录端到端"加 widget"流程。
- [ ] 验证:临时删一个页文件 → gallery-stubs 重生成;`auto ui backlog` 列表合理。
- [ ] worktree 分支在 build + 测试绿后合并回 `master`。
