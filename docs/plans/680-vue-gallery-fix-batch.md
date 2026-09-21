---
plan_id: PLAN-680
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: vue-gallery-fix-batch
author: [agent]
created_at: 2026-09-21
updated_at: 2026-09-21

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui-gen, auto-lang/vm-ffi, examples/ui/027-file-manager]
current_step: 5
total_steps: 5
---

# [PLAN-680] vue-gallery 修复滚动批（027 file-manager vue 轨首修）

## 变更摘要

滚动跟踪批：跟踪 ui-gallery vue 轨（`auto run` / gallery 内嵌）的走查问题，**先记账后修复**，
修复走 worktree。本批首个执行项 = **027-file-manager vue 轨启动报错**：

> 运行错误: [auto-gen] VM-only primitive "Env.get" has no Vue/JS build — this path only runs in VM mode

根因：PLAN-671 ① 将 `Env` 并入 `VM_ONLY_OBJECT_NATIVES`（ts_adapter.rs:19）后，vue 轨对
`Env.*` 一律发射运行时抛错的 `__vmOnly` 桩；而 027 的 `.Init` handler 无条件调用
`Env.get("USERPROFILE"/"HOME"/"AUTO_UI_IN_DESKTOP")`（app.at:719-723），挂载即抛 →
AppViewport 错误横幅 → 文件浏览内容（列表/快捷访问引导）全部不加载。

修复两件套：
1. **生成器**：vue 轨对 `Env.get("字面量键")` 做**生成期常量折叠**（gen 机 env 快照 →
   JS 字符串字面量）；非字面量键与其余 `Env.*`/`Process.*` 维持 `__vmOnly` 诚实抛错。
2. **轨别原语**：新增 `Env.track() -> str` native（VM shim 恒返 `"vm"`；vue 轨生成期折叠为
   `"vue"` 字面量），给 .at 语料一个无 Env 依赖的轨别信号。
3. **027 语料**：vue 轨降级守卫——bootstrap 与重 FS 路径以 `Env.track()` 门控，vue 轨渲染
   空态提示（"文件系统访问需 VM 轨"）而非抛错横幅；VM 轨全功能零漂移。

后续问题（用户走查追加）登记在 §需求分析与背景调查 的问题清单，按批内小项滚动执行。

## 目标

- **G1**：ui-gallery vue 轨内嵌的 027-file-manager 打开后**无错误横幅**，UI 骨架完整，
  列表区呈现友好空态提示（vue 轨 = 前端调试轨定位，027 SPEC 明示桌面事实轨 = VM）。
- **G2**：VM 轨（`auto run -r vm` / 桌面 in-process）027 行为**零漂移**：主目录解析、
  快捷访问引导、目录列表、缩略图全链不变。
- **G3**：`Env.get("字面量")` 折叠与 `Env.track()` 原语可被其它 demo 复用
  （041-auto-edit 有同款无条件 `Env.get` ×3，记账待修）。
- **G4**：滚动批记账结构就绪：问题清单 + 每项根因/修复口径/验证面，支持批内追加。

**非目标**：
- vue 轨 fs HTTP 桥（让 vue 轨真的列目录）：异步桥接是大设计（sync handler vs fetch await、
  dev server 端点面、安全边界），登记候选方向待用户裁定是否 L2 立项，不入本批。
- `Process.*` 的 vue 构建（同属 VM-only，无 demo 报修，维持 `__vmOnly`）。
- 041-auto-edit 的修复执行（记账，本批滚动项，验证炸点后小修）。

## 架构方案

改动全部落在 auto-lang 仓（生成器 + VM shim + 027 语料）；auto-os 的 ui-gallery 壳
（AppViewport.vue）零改动——横幅是诚实的，修复后不再触发。

1. **ts_adapter 折叠点**：`transpile_expr` 的 method-call 分支（ts_adapter.rs:1045 附近的
   `VM_ONLY_OBJECT_NATIVES` gate）**前置**一条特判：`recv == "Env" && method == "get"` 且
   首参为字符串字面量 → `std::env::var(KEY).unwrap_or_default()` 生成期求值，发射 JS
   字符串字面量（走既有字符串转义路径）；否则落入既有 `__vmOnly` 降级。附 warning 注记
   折叠语义（gen 机快照，非运行时读取）。
2. **`Env.track()` 原语**（VM 侧 + 生成器侧双形态，沿 localStorage 族
   "web-platform global 双构建"先例，native_catalog.rs:2771-2773）：
   - VM：`vm/ffi/stdlib.rs` 新 shim `shim_env_track() -> "vm"`，`#[auto_macros::rust_fn("Env.track")]`；
     `NATIVE_ENV_TRACK` 常量取**高段新 id**（memory 铁律：避开 29xx 动态分配带，选 9900+，
     与 native-catalog-id-collision 在案教训一致）；native_catalog.rs 两个注册表（静态表 +
     Plan 442 B-support 段落旁）各登一条。
   - vue：ts_adapter 对 `recv == "Env" && method == "track"`（零参）直接发射 `"vue"` 字面量，
     不经 `__vmOnly`。
3. **027 语料守卫**：`app.at` Init 的 `.in_desktop` 语义保留（桌面嵌入探测），新增
   `var is_vm str = Env.track()`（或 bool 形态，执行期按 027 现有风格定）；Tick bootstrap
   （fs.join/file.exists/file.is_dir 建快捷访问段）、列目录 read_dir 链、image.thumb 缩略图
   链以 `is_vm` 门控；vue 轨在列表区渲染既有语义 token 的空态提示行 + 状态栏文案复用。
   VM 轨分支包裹现有代码，零逻辑改动。

## 需求分析与背景调查

**授权记录**：2026-09-21 用户会话授权——"继续解决 vue 版 ui-gallery 的问题，新建计划跟踪
（用 worktree）"，首个问题 = 027 file-manager vue 轨报错（截图确证：AppViewport 运行错误
横幅 + 0 个项目空列表）。滚动批形态沿用 PLAN-672 先例。预算：批内小项，无特殊上限。

**已知问题清单（滚动记账，随走查追加）**：

| # | 问题 | 根因定位 | 修复口径 | 状态 |
|---|---|---|---|---|
| Q1 | 027 vue 轨启动抛 `Env.get` 无构建，内容不加载 | PLAN-671 ① 将 `Env` 并入 VM_ONLY 白名单（ts_adapter.rs:19、:1045-1073），027 `.Init` 无条件 `Env.get` ×3（app.at:719-723），vue 挂载即抛 | 本批 F-1/F-2/F-3（见 §详细设计） | **本批执行** |
| Q2 | 041-auto-edit vue 轨同款炸点（`Env.get("AUTO_PROJECT_DIR"/"AUTO_OPEN_PATH"/"AUTO_SAVE_PATH")` ×3，editor_store.at:109/:293/:320） | 同 Q1 机制；源码核对三键全为字面量，F-1 折叠即消除（T1 测覆盖机制） | 实机确认并入 auto-edit 冷重生成既有待办（PLAN-671 遗留） | 记账（判定消除，待彼实机） |
| Q3 | vue 轨 fs 桥缺失（fs.*/File.*/image.* 全为 `__vmOnly`，027 列目录在 vue 轨本质上不可用） | 架构现状：vue 轨无 fs 能力；api.* 的 await+fetch 机制（ts_adapter.rs:1359-1396）证明异步桥可行但需设计（sync handler 改造/dev server 端点/安全边界） | 候选 L2 设计，待用户裁定是否立项 | 记账候选 |
| Q4 | sidebar-in-row 布局塌陷：上游 SidebarProvider 默认 `w-full`（shrink-0）在 flex row 内与 flex-1 兄弟互斥→内容区 0 宽 | shadcn 上游默认为页面根用法设计；027 执行期 DOM 探针实测（x:1280/w:0），空态提示不可见的真因 | 027 已修（`w-auto`，018 先例）；015/017/022 等其它 sidebar-in-row demo 未排查 | 记账（批内候选走查项） |
| Q5 | `file.*`（小写）不在 VM_ONLY_OBJECT_NATIVES（只收了大写 `File`）——vue 轨裸发射未定义标识符 `file.exists(...)`，执行即 ReferenceError | PLAN-671 白名单录入口径；既往被 Init 先炸掩盖；027 现被轨别守卫规避 | 候选修法：白名单收编小写 `file` 或给 file 族 JS 构建另议；触发面前需全 demo 排查 | 记账（潜在雷，未触发不阻塞） |
| Q6+ | （待用户 vue 轨走查追加） | — | — | 空 |

**背景分析（证据）**：
- 报错横幅源：`examples/ui-gallery/gen/front/vue/src/gallery/AppViewport.vue:61,111`
  （生成物，模板源头在 auto-os 仓 `D:/autostack/auto-os/ui-gallery/src/gallery/AppViewport.vue`）；
  027 在 `demos-registry.ts:352` 注册。ui-gallery 的 `gen/` **全量 git-ignored**
  （`git ls-files examples/ui-gallery/gen` = 0 条，实测），生成期折叠不会把机器路径写入仓库。
- 抛错桩发射：`crates/auto-lang/src/ui_gen/ts_adapter.rs:1045-1073`（`VM_ONLY_OBJECT_NATIVES`
  gate，PLAN-444 建制、PLAN-023 增 image、PLAN-671 ① 增 Env/Process，注释明示
  "an honest runtime error beats silently illegal code"——本计划不推翻该裁定，只对
  **字面量键 Env.get** 与 **零参 Env.track** 两形态做生成期折叠）。
- 桩声明：`ui_gen/vue.rs:3863-3877`（`needs_vm_only_helper`）。
- VM 侧 env 实现：`vm/ffi/stdlib.rs:561`（`shim_env_get`）、`NATIVE_ENV_GET=1100`（:93）；
  web-platform global 双构建先例：`vm/native_catalog.rs:2771-2779`（localStorage/dom 族）。
- 027 架构定位：`examples/ui/027-file-manager/SPEC.md` §开篇——"桌面事实轨 = VM；
  vue 轨为前端调试轨"；§1 数据层明示 `Env.get("USERPROFILE") → 回落 HOME`。
- 027 现存 native 调用面（本批门控范围实测）：fs.join ×5 / fs.read_dir ×2 / fs.mtime /
  fs.canonical / fs.parent ×2 / fs.ext / fs.filename / fs.rename ×3 / fs.copy_recursive；
  file.exists ×6 / file.is_dir ×4 / file.size / file.write_text / file.create_dir /
  file.delete / file.remove_dir；image.thumb ×1；storage.get/set ×15（storage 在 vue 轨有
  localStorage 构建无需门控）。
- 027 在 vue 轨即使过了 Init 也会死在 Tick bootstrap（`app.at:735-760`：`fs.join`/
  `file.exists`/`file.is_dir` 建快捷访问）——所以 Q1 修复必须三件配套，只做折叠不解决横幅。

**约束**：
- VM-only 诚实抛错裁定（PLAN-671）不推翻：折叠仅限生成期可确证的两种形态。
- native id 避开 29xx（动态分配器覆写静态表，native-catalog-id-collision 在案），选 9900+。
- 不在 worktree 内创建 junction/symlink（三仓 .git 事故红线）；027 若 `auto gen` 产生
  pnpm junction，验证后按仓工具链雷区条目清理（cmd rmdir /s /q）再过 wt-guard。
- 本批改 VM 面（native_catalog/ffi）但不触 aavm 触发条件（auto/lib/*.at、aavm2 测试基建
  等零涉及）→ 不跑 `taa`。

## 详细设计

### F-1 生成期常量折叠：`Env.get("字面量")`（vue 轨）

- 位置：`crates/auto-lang/src/ui_gen/ts_adapter.rs`，method-call 分支 `VM_ONLY_OBJECT_NATIVES`
  gate **之前**。
- 规则：`Expr::Dot(Ident("Env"), "get")` 且 `call.args.args[0]` 为 `Expr::Lit` 字符串字面量 →
  读 `std::env::var(KEY).unwrap_or_default()`，经既有 JS 字符串转义发射为字面量；
  `ctx.note_warning("Env.get(\"KEY\") folded at generation time (host env snapshot)")`。
- 边界：非字面量键、`Env.get_or`/`Env.set`/`Env.var` 及其它 `Env.*`/`Process.*` →
  原有 `__vmOnly` 路径不动。VM 轨（rust/vm 发射）不经过 ts_adapter，零影响。
- 语义口径：折叠值 = **gen 机 env 快照**，重新 `auto gen`/`auto run` 才会刷新；与 VM 轨
  运行时读取在"同一台开发机"场景下值一致（dev 调试轨可接受），SPEC 增量记入 SD-01。

### F-2 `Env.track() -> str` 原语（双轨单源）

- VM：`crates/auto-lang/src/vm/ffi/stdlib.rs`
  - `pub const NATIVE_ENV_TRACK: u16 = 9901;`（9900+ 高段，避开动态分配带）
  - `#[auto_macros::rust_fn("Env.track")] pub fn shim_env_track() -> String { "vm".into() }`
  - `native_catalog.rs` 静态表 + 汇总表各登 `(9901, NATIVE_ENV_TRACK, shim_env_track, "auto.env.track")`
    （对齐 2771 族登记形态；执行期核对两处注册表的实际结构后照抄同款）。
- vue：ts_adapter 对 `Dot(Ident("Env"), "track")` 零参调用发射 `"vue"` 字面量（在 F-1 折叠点
  同一分支，先于 `__vmOnly` gate）。
- 其余轨（rust native 编译发射）：`Env.track` 若在 rust 发射面无注册则该轨 demo 编译期即报
  未知标识——执行期核查 rust 发射面（a2r/rust.rs）是否需要同步登记；如 027 不跑 rust 轨则
  登记 max(symmetry)但允许仅注释说明（避免超面扩张）。

### F-3 027 语料 vue 轨降级

- `examples/ui/027-file-manager/src/front/app.at`：
  - state 增 `var is_vm str = ""`；Init 首 行 `.is_vm = Env.track()`（折叠后 vue 轨恒 "vue"）。
  - Tick bootstrap 段（`if !.booted && .tick_count > 8` 整块）门控 `if .is_vm == "vm"`；
    vue 轨跳过引导（侧栏快捷访问为空分组骨架，可接受）。
  - 列目录/导航/重命名/删除/缩略图等 handler 的重 FS 段：在 handler 入口处
    `if .is_vm != "vm" { return }` 早退（保留纯视图状态变更如 view_mode/sort_col/show_hidden
    与 storage 持久化——这些 vue 轨可用，不门控）。
  - vue 轨空态提示：列表区在 `is_vm != "vm"` 时渲染一行 muted 文案（语义 token
    text-muted-foreground，如"vue 调试轨无文件系统访问 — 完整功能请使用 auto run -r vm"），
    复用既有 empty 视图位；不新增组件文件。
- `storage.get/set` 持久化（view_mode/sort/hidden）在 vue 轨**照常工作**
  （localStorage 构建），不门控——保住"前端调试轨"的样式调试价值。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/ui/vue-generation.md（vue 生成工程契约；执行期核对实际文件名，若无专节则在 ui 模块 overview 相应节） | `Env.get`：一律 `__vmOnly` → **字面量键折叠为 gen 机 env 快照字面量，其余仍 `__vmOnly`**；新增 `Env.track()`：VM 轨 "vm" / vue 轨 "vue" 双构建 | 诚实抛错裁定保留；调试轨需要无 Env 依赖的轨别信号与可折叠的字面量 env 读取 | AC-01/AC-02 |
| SD-02 | add | docs/specs/ui/（027 所属 demo 契约节或 examples/ui README 的轨别矩阵） | 027 vue 轨行为定版：shell+样式调试可用，FS 面空态提示，事实轨=VM | 027 SPEC 明示 vue=前端调试轨，降级口径入册防回归 | AC-04 |

（执行期若发现 specs 路径与实际目录名不符，按实际路径落并在此表回写。）

## 测试设计

- **生成器单测**（ts_adapter 内既有测试模块同款）：
  - T1: `Env.get("USERPROFILE")` → 输出含折叠字面量、不含 `__vmOnly`；gen 机无该变量时折叠为 `""`。
  - T2: `Env.get(some_var)`（非字面量）→ 仍发射 `__vmOnly('Env.get', ...)`。
  - T3: `Env.set(...)`/`Process.xyz(...)` → 仍 `__vmOnly`（回归保护）。
  - T4: `Env.track()` → `"vue"` 字面量。
- **VM shim 测试**：ffi_dual 或 stdlib 单测同款（`Env.track()` VM 轨返回 "vm"；
  参考 `ffi_dual_tests.rs:83` env_get_set 形态）。
- **双轨走查**（autoui-verifier 技能脚本）：
  - vue：`cd examples/ui/027-file-manager && auto run` → 页面无错误横幅、空态提示可见、
    view_mode 切换/搜索框/对话框可交互（截图取证）。
  - VM：`auto run -r vm` → 主目录/快捷访问/列目录/缩略图与批前一致（截图比对）；
    既有 `tests/desktop_mcp.py` / `plan023_check.py` 冒烟按需复跑。
  - gallery 内嵌：ui-gallery regen 后 AppViewport 挂 027 无横幅（若 gallery regen 成本高，
    以 027 独立运行 + 桩不触发为充分证据，gallery 复验降为抽查）。

## 验收标准

- **AC-01**：vue 轨 027 挂载后**无任何错误横幅**；DevTools console 无 `__vmOnly` 抛错。
  验证：`auto run` + Playwright 截图（.agents/skills/autoui-verifier/scripts/test_vue_playwright.mjs）。
- **AC-02**：列表区呈现空态提示文案（含 "auto run -r vm" 指引），样式走语义 token；
  view_mode/搜索/排序等纯视图交互不报错。验证：截图 + 脚本断言文案存在。
- **AC-03**：VM 轨 027 全功能零漂移：主目录解析、快捷访问五项引导、目录列表、缩略图链
  与批前一致。验证：`auto run -r vm` 走查截图比对 + `cargo tv` 绿。
- **AC-04**：T1-T4 生成器单测全绿；VM shim 测试绿。验证：`cargo t ts_adapter`（或所属
  测试模块名）+ `cargo t ffi`。
- **AC-05**：`cargo check -p auto-lang` 零 warning 新增；最终合入前 `cargo tf` 全绿
  （改了 VM 面，Category B 全档门禁）。
- **AC-06**：SD-01/SD-02 规范增量落库（merge 阶段），027 SPEC 增补 vue 轨降级口径一段。

## 执行步骤

（worktree：`D:/autostack/.wt/lang-680/auto-lang`，分支 `plan-680-dev`；簿记回 master。）

- **T-01 [F-1+F-2 生成器与原语]**（→ AC-01/AC-04）
  - `ts_adapter.rs`：折叠分支 + `Env.track` 特判 + 4 条单测（T1-T4）。
  - `vm/ffi/stdlib.rs` + `vm/native_catalog.rs`：`NATIVE_ENV_TRACK=9901` + shim + 两表登记。
  - 核查 rust 发射面是否需同步登记（详见 F-2 第三点，允许注释说明收口）。
  - 验证：`cargo check -p auto-lang` → `cargo t ts_adapter` → `cargo t ffi`。
  - [✅ 已完成] commit 8be1e48e1；check 零新告警（四文件 grep 空）；id 改 9907
    （计划原写 9901——9900-9906 已被 scroll/code_editor 占，执行期勘误）；
    `cargo t ts_adapter` 21/21（含新 5 测 T1/T1b/T2/T3/T4）；`cargo t ffi` 170/170
    （含 env_track_shim_returns_vm）。登记面勘误：Env 族走 inventory 机制
    （rust_fn 宏+NATIVE_ID_ENTRIES+ret-type 双表），不入 for_each_native。
- **T-02 [F-3 027 语料降级]**（→ AC-01/AC-02/AC-03）
  - `app.at`：is_vm state + Init 折叠读取 + Tick bootstrap 门控 + handler 早退 + 空态提示。
  - 验证：027 双轨走查（vue 截图 ×2：骨架+空态提示；VM 截图 ×2：快捷访问+列表）。
  - [✅ 已完成] commits 9602dab19+29d2b4ce4（截图入册 attachments/680/）。执行期
    新增修：sidebar_provider 补 `w-auto`——上游 SidebarProvider 默认 `w-full` 在
    row 内与 flex-1 兄弟互斥→内容区 0 宽（DOM 探针实测 x:1280/w:0），空态提示
    因此不可见；018-book-reader 同款先例。双轨走查：vue 初始无横幅、booted 后
    空态提示居中、网格切换+搜索输入无横幅；VM 主目录解析+快捷访问五项+盘符
    C/D/E/G+62 项列表全功能。产物审计：全部 file.*/__vmOnly 调用点均在守卫内
    或空列表不可达（CommitRename/CtxPaste/ExecuteDelete 需行级状态）。
- **T-03 [回归门禁]**（→ AC-03/AC-05）
  - `cargo tv`（VM 面改动触发）；041-auto-edit vue 轨实测 Q2 是否已被折叠消除
    （消除则在 §问题清单勾销，残余另记账）。
  - [✅ 已完成] `cargo tv` 3865/3865 全绿（VM 语料 golden 零漂移）。Q2 实测受限：
    041 `render: "vm"` 无 vue 产物（主检出同），gen 不产 vue——按源码核对三处
    `Env.get` 键全为字面量（editor_store.at:109/:293/:320），机制被 T1 测覆盖，
    判定"折叠落地即消除"；实机确认并入 auto-edit 冷重生成既有待办
    （PLAN-671 遗留，属彼闭环）。
- **T-04 [SPEC/SPEC.md 增量]**（→ AC-06）
  - 027 SPEC.md 补 vue 轨降级段；SD-01/SD-02 对应 specs 文件落点核对与草拟
    （正式落库在 merge 阶段执行）。
  - [✅ 已完成] commit a9b45b7d7。SD-01 落点确认为
    `docs/specs/auto-lang/ui/design/app-generation.md`（PLAN-677 契约列表增第 5 条
    "Env 轨别原语与字面量折叠"）；SD-02 落 027 SPEC.md §5.5（示例级文档，随
    worktree 提交即生效，无独立 specs 文件）。
- **T-05 [review+merge 簿记]**：`cargo tf` 全档 → /auto-plan:review → merge 收据 →
  归档清理（wt-guard 先行）。
  - [✅ 已完成] `cargo tf` 全档门禁于执行收尾运行（结果见复审记录）；review/merge
    两步为范式后续技能（/auto-plan:review → /auto-plan:merge）的固有阶段，work
    交付到此交接。

## 复审记录

- 2026-09-21 draft handoff：stage=new，PLAN-680 rev1。outcome=pass（Q1 根因三方确证：
  白名单机制/发射点/027 调用点；gen 产物 git-ignored 实测确证折叠不污染仓库）。
  next=work（T-01 起步）。待用户对计划确认后进 worktree 执行。

- 2026-09-21 work handoff：stage=work | plan_id=PLAN-680 | plan_revision=1 |
  outcome=pass | code_commit=worktree plan-680-dev 8be1e48e1/9602dab19/29d2b4ce4/
  a9b45b7d7（基线 master f2e1aa9a1；依赖位 auto-down detached fba6563） |
  task_ids=T-01..T-05 全勾 | evidence：
  - T-01 `cargo t ts_adapter` 21/21（新 5 测）+ `cargo t ffi` 170/170（新 shim 测），
    check 零新告警；
  - T-02 双轨走查截图六张入册（vue：无横幅+空态提示+网格/搜索交互；
    VM：主目录+快捷访问+盘符+62 项列表）；
  - T-03 `cargo tv` 3865/3865；T-05 `cargo tf` 3718/3718 全绿；
  - AC-01..06 全过（AC-06=SPEC/SPEC.md 增量已随 worktree 提交，merge 落库）。
  执行期勘误与发现：id 9901→9907（9900-9906 已占）；登记面=inventory 双表非
  for_each_native；新增滚动项 Q4（sidebar-in-row w-full 布局塌陷，027 已修 018 先例）
  与 Q5（`file.*` 小写不在 VM-only 白名单，裸标识符雷，027 已被守卫规避）。
  blockers=无 | next=review（/auto-plan:review），merge 时 WT-guard 注意 gen 目录
  pnpm junction 清理（仓工具链雷区在案）。

## 待澄清事项

- （无阻塞项）Q3（vue fs 桥）是否立项为 L2 设计，由用户在批内任意时点裁定，不阻塞 Q1 修复。
