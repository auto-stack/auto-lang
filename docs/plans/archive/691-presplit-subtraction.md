---
plan_id: PLAN-691
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: presplit-subtraction（拆仓前减面瘦身批）
author: [zcode]
created_at: 2026-09-22
updated_at: 2026-09-22
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/design/32-autoui-repo-extraction.md §0 体量口径勘误, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/ui/architecture.md, docs/specs/auto-lang/project.md, docs/specs/INDEX.md]
touched_goals: []             # 域内卫生批，无 GOAL-NNN 语义变更（GOAL-007 视觉/parity 面零触及）

affects: [auto-lang/ui, auto-lang/desktop-protocol, auto-lang/tests]
current_step: 5
total_steps: 6
---

# [PLAN-691] presplit-subtraction（拆仓前减面瘦身批）

## 0. 变更摘要

Design 32（AutoUI 拆仓）执行前的**减面四刀**——全部是 subtraction（删/移/订正）而非
refactoring，目的：缩小拆分迁移面、提高"哪些文件跟 auto-ui 走"的可判定性、纠正已传染
的错误体量口径：

1. **注释订正**：两处"UI 已占本 crate ~90%"错误说法（Design 32 §0、Cargo.toml:13）→
   实测 ~52%（2026-09-22 复核，见 §4）。
2. **gpui 死后端移除**：`ui-gpui` feature 全仓零启用，`src/ui/gpui/`（4 文件 ~3.3k 行）
   + optional deps（gpui-lib/gpui-component/anyhow）+ 8 个 examples 的 cfg 阶梯臂。
3. **pixels 臂退役**：在册债 P-RQ-PIX 执行——量化门已达标（034 memory-matrix 结论行），
   条件②（widget 覆盖收口）现场勘定后决定删/顺延。前置=PLAN-690 合并。
4. **91 个顶层散测试归位**：`src/*_tests.rs`+`test_*.rs`（91 文件 30,241 行）纯 move 到
   `src/tests/`，lib.rs 对应 mod 声明迁入 tests.rs（cfg 门原样）。

## 1. 目标

- 拆分前迁移面缩小：少搬 ~4.2k 行死码（gpui）+ ~2k 行待退役机件（pixels 臂）。
- 文件可判定性：src/ 根目录只留生产模块，散测试进 tests/——拆仓时"跟 auto-ui 走"的
  判定从"逐个猜"变为"按目录"。
- 决策依据纠偏：90% → ~52%，直接影响拆仓后 auto-lang 的体量预期与资源规划。

**非目标**（拆分后 auto-ui 内政，本计划不做）：
- renderer.rs（3.5 万行）/vue.rs（3.0 万行）内部结构化；
- 生成器适配器共享层推广（jet/ark 的 shared 模式 → vue/rust）；
- 编辑器/终端/mpv 运行时归属裁定（随 auto-ui 走还是另立 auto-apps）；
- 任何行为变更、任何 API 面变更。

## 2. 架构方案

减面不改架构，四刀相互独立、可单独回滚：

- 刀①是纯文档/注释订正（master 允许的簿记类，但与刀②同 worktree 提交更省一轮门禁）。
- 刀②删一个 feature 阶梯的整层：feature 定义 → deps → 模块 → examples cfg 臂 →
  （若 specs 活文档提及）spec 注记。零启用者 ⇒ 默认构建面完全不变。
- 刀③按债册 P-RQ-PIX 的删除面清单执行：`FrameMode::Pixels` 档、`pixels.rs`、
  `dual_mode.rs`、renderer.rs `NativePixelsHost` 族、dual_mode 测试族；queue 臂
  （stage3/native_projector）不动。执行前先勘退役门条件②现状，不达标则降级为
  "勘定报告+顺延裁定"，不删。
- 刀④纯 move：文件 git mv + mod 声明从 lib.rs 迁 tests.rs，内容零变更（hash 对拍证明）。

## 3. 技术栈

纯 Rust crate 内删除/移动 + 文档订正；无新依赖、无 schema 改动、无 aavm 路径触及
（零 `cargo taa` 触发条件，见 §6）。

## 4. 需求分析与背景调查

**授权**：用户 2026-09-22 会话提出"这几个工作我们现在就可以做吧？是不是可以立一个
新的计划来做？"——范围即上述四刀（本轮会话结论清单原文）。无额外预算/自动续跑授权。

**体量口径复核（2026-09-22，本会话）**：
- `crates/auto-lang/src` git 跟踪 cat 口径 **579,210 行**（`find … | xargs wc -l` 的
  567k 口径含 xargs 分批截断风险，cat 口径为准）；
- UI 集群 = ui/ 208,017 + ui_gen/ 77,468 + aura/ 9,561 + design_tokens/ 2,559 +
  a2ui/ 1,972 ≈ **30.0 万 ≈ 52%**（剔测试 ~57%）；
- 回溯 09-20 当天（git archive @ 296adb54f 复核）：src 实为 **557,098 行**，UI ≈ 27.7 万
  ≈ 50% ⇒ 原调研"27.2 万/30.3 万=90%"**分子基本对、分母量错**（疑似
  `xargs wc -l | tail -1` 分批截断，本会话同坑复现后改 cat 口径）；
- 传染点两处：`docs/design/32-autoui-repo-extraction.md:18`（§0）、
  `crates/auto-lang/Cargo.toml:13-14` 注释。

**gpui 勘定（2026-09-22）**：
- `src/ui/gpui/{mod,renderer,auto_render,vnode_entity}.rs` 4 文件 ~3.3k 行；
- `ui-gpui` feature（Cargo.toml:56）全仓 toml/rs **零启用者**（default =
  ["with-file-history","ui-iced"]，无任何 crate/example/tests 引用该 feature）；
- deps：gpui-lib(=gpui 0.2.2)/gpui-component(0.5.0) 均 optional 且仅 ui-gpui 引；
  anyhow optional（Cargo.toml:202）仅 ui-gpui 引（src 内 "anyhow" 字样其余为字符串表）；
- examples 8 个（ui_counter/ui_container/ui_accordion/ui_list/ui_gallery/ui_layout/
  ui_navigation_rail/…）内 cfg 阶梯三臂：`ui-iced` → `ui-gpui` → 双无编译错误提示
  （ui_counter.rs:48-66 实证）——删 gpui = 收阶梯为两臂 + 清注释；
- docs/design、docs/plans/archive 的历史提及**保留不改**（历史文档原则）。

**pixels 臂勘定（2026-09-22）**：债册 P-RQ-PIX（KNOWN-DEBT-AND-RISKS.md）四条件：
①native auto 翻转 ✅（PLAN-032）；③tag10/11 位图过线 ✅（PLAN-034 §1.15）；
④像素原生五 kind 裁定 ✅；**②widget 覆盖收口至像素原生族外全量（"M7-c 后"）待现场核**
（PLAN-674 已收 codeeditor，terminal 曾记"M7-c 撞面"）。量化门（用户 2026-09-19 定标）
已达标：`docs/plans/reports/assets/034/memory-matrix.txt` 结论行——release×default×
1 窗（tiny-skia）rqhost private=11260KB ≤ 102400KB **达标**；app 6460KB ≤ 10MB 过。
删除面 = desktop_protocol/pixels.rs（749 行）/dual_mode.rs/FrameMode::Pixels 档/
renderer.rs NativePixelsHost 族（ui/ 内 29 处引用）/dual_mode 测试族。
**前置依赖：PLAN-690 合并**（690 双 commit 改 desktop_protocol+renderer.rs，先删必冲突；
690 现状=execution_done 待复审，分支 plan-690-dev、worktree lang-690 在途）。

**散测试勘定（2026-09-22）**：`src/*_tests.rs` + `test_*.rs` 共 **91 文件 30,241 行**
（含 test_runner.rs/test_double_lexer.rs/test_float_full.rs/test_parser_arrow.rs；
test_util/ 目录 269 行另行归位）。mod 声明全在 **lib.rs**（实证 plan_088_tests:183、
plan340_tests:7444、musk_vm_track_tests:7826，声明带 Plan 编号溯源注释）；
`src/tests.rs`（106 行）已有 67 个 mod 先例 + `#[path = "tests/plan377_bench.rs"]` 先例。
文件名含 UI 语义的仅 ~3 个（style/pointer/layout 类），绝大多数核心侧——**归位本身
即为拆仓判定服务**（UI 侧测试已在 ui/ 目录内，随模块走）。文件内容形态注意：
部分文件内含同名内层 mod（plan340_tests.rs:10 `mod plan340_tests {`），不影响 move。

## 5. 详细设计

### 刀① 注释订正（T-01）
- `docs/design/32-autoui-repo-extraction.md:18`：§0 句子改
  "UI 相关代码约占 `crates/auto-lang/src` 的 ~52%（约 30 万/58 万行，2026-09-22 cat
  口径复核；原稿 90%/30.3 万系分母测量误差）"；
- `crates/auto-lang/Cargo.toml:13-14` 注释：同口径改写（保留 Plan 330→2026-09-22 裁定
  叙事，只纠数字）。
- 记忆档已在会话中先行修正（autoui-repo-extraction-research.md），不在本计划范围。

### 刀② gpui 移除（T-02）
1. `git rm -r crates/auto-lang/src/ui/gpui/`；删 `ui/mod.rs:205-206` 的
   `#[cfg(feature = "ui-gpui")] pub mod gpui;`；
2. Cargo.toml：删 `ui-gpui` feature（:56）、gpui-lib/gpui-component（:200-201）、
   anyhow optional（:202，若全仓再无 optional 消费者——勘定已证仅 ui-gpui 引）；
   Cargo.lock 随构建再收缩；
3. examples 8 个：cfg 阶梯收两臂（iced → 双无编译错误提示），删 gpui doc 注释行；
4. specs 活文档勘定：若 docs/specs/ui/* 现述"三后端（iced/gpui/headless）"则同步
   （执行时 grep 定位；历史 design/plans 豁免）。
- 回滚=单 commit revert。

### 刀③ pixels 退役（T-03，条件式）
1. **门勘定**（产物=勘定记录入 §9）：核条件②——以 PLAN-674/683/690 后的 RQ widget
   覆盖集现状比对"像素原生族外全量"；②达标 → 执行删除；不达标 → 产出报告，本刀
   降级为顺延（AC-03 记 deferred，等 M7-c）；
2. 删除（达标时）：desktop_protocol/{pixels.rs,dual_mode.rs}；mod.rs/client_runtime.rs/
   stage3.rs 中 FrameMode::Pixels 档与 dual_mode 挂载；renderer.rs NativePixelsHost/
   PixelsChild/run_independent_* 族；dual_mode 测试族；
3. 债册 P-RQ-PIX 销账行（保留四条件+量化门证据链接）；
4. 实机验收：`cd examples/ui/001-helloworld && auto run -r vm -q` 单窗渲染正确
   （031 或 003 亦可，对照 034 走查口径）。

### 刀④ 散测试归位（T-04）
1. 生成清单：`ls crates/auto-lang/src/*_tests.rs test_*.rs` 全列 + 每文件 cfg 门
   （从 lib.rs 声明行摘）→ 计划内附清单（执行时落 §8 证据）；
2. `git mv` 逐文件 → `src/tests/`（**平铺**，不建子目录——域分类留给拆仓阶段 2 资产
   迁移，见 Q-3）；test_util/ 目录同批归位；
3. lib.rs 对应 `mod x;` 声明（含 cfg 门与溯源注释）剪切 → tests.rs 追加区；
   tests.rs 子模块解析自动落 `src/tests/x.rs`，无需 #[path]；
4. 跨引用修复：grep `crate::<moved_mod>::` 逐个改 `crate::tests::<moved_mod>::`
   （预期仅 test_runner/test_util 类支撑件有跨引）；
5. 纯 move 证明：`git diff --stat` 全部为 rename similarity 100%（或逐文件 hash 对拍）。

### 规范增量

| delta_id | add/modify/retire | target | before → after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/design/32-autoui-repo-extraction.md §0 | "约占 90%——27.2 万/30.3 万行" → "~52%——约 30 万/58 万行（2026-09-22 cat 口径复核，原分母系测量误差）" | 拆仓决策依据纠偏 | AC-01 |
| SD-02 | retire（条件） | docs/plans/KNOWN-DEBT-AND-RISKS.md P-RQ-PIX | 四条件退役门在册 → 门②勘定达标即销账（附证据链）；未达标 → 门状态行更新 | 在册债执行 | AC-03 |
| SD-03 | modify | docs/specs/ui/*（若 gpui 活提及） | 后端清单含 gpui → 移除注记 | 代码-规范同步 | AC-02 |

## 6. 测试设计（门禁映射，按 AGENTS.md 分级）

| 任务 | 改动类 | 门禁 |
|---|---|---|
| T-01 注释订正 | C（文档/注释） | 零 cargo（Category A 禁令） |
| T-02 gpui 移除 | B（Rust 删码） | `cargo check -p auto-lang` + `cargo check -p auto-lang --examples` + `cargo t` |
| T-03 pixels 退役 | B（Rust 删码） | `cargo t desktop_protocol` + 实机 `-q` 走查；零 aavm 触碰 → 无 taa |
| T-04 散测试 move | B'（纯 move 但触门控 mod 编译） | `cargo t` + `cargo tv` + `cargo tt` + `cargo tb`（四档覆盖 default/vm-files/trans/book 门控面） |
| T-05 合入前全量 | — | `cargo tf`（对照预存红在册口径，零新增） |

健康检查：零新编译警告；被改文件局部 fmt（全仓 fmt 分叉禁令不变）；无残留 debug print。

## 7. 验收标准

- **AC-01**（T-01）：两处注释口径为 ~52%/58 万行并注明复核口径与日期；diff 仅注释/
  文档行，零行为面。验证：`git diff` 逐行人工审。
- **AC-02**（T-02；r2 口径勘正）：`cargo check -p auto-lang`（default）零错；
  `--examples --features build-examples` 零错（裸 `--examples` 因 p023_probe 补
  required-features 门后跳过该例→零错）。**渲染后端面零活引用**（src/ui/gpui、
  style/gpui_adapter、examples、HostBackend 全清）；**数据面豁免在案**：config.rs
  `UiBackend::Gpui`/session 默认串/aura `BackendMatrix.gpui`（aura.at 目录唯一源契约，
  零派发勘定）保留，历史成因注记（vnode.rs 等）保留。
- **AC-03**（T-03，条件式）：门②勘定记录在案。达标路径：pixels.rs/dual_mode.rs 删、
  FrameMode 单 Queue 档、renderer.rs NativePixelsHost 族清、债册销账、`-q` 单窗实机
  渲染正确。不达标路径：勘定报告 + 顺延裁定落 §9，标 deferred（不算失败，算门机制生效）。
- **AC-04**（T-04）：清单 91+test_util 全部落 `src/tests/`；`git diff --stat` rename
  100%（内容零变更）；lib.rs 根区测试 mod 清空（生产 mod 不动）；cargo t/tv/tt/tb 四档
  对照预存红基线零新增。
- **AC-05**（T-05/T-06）：`cargo tf` 零新增红；复审记录 + 账本三件套（ui/plans.md 行 +
  specs.json 条目 + `python scripts/spec-index.py` 再生）落盘。

## 8. 执行步骤

> 开工序：master 上 commit `.next-id`+本计划 → `git worktree add
> D:/autostack/.wt/lang-691/auto-lang -b plan-691-dev`（Plan 529 分组平铺）→ 全部代码
> 改动在 worktree；簿记回 master。**T-03 前置=PLAN-690 已合并**——若开工时 690 未合，
> 先执行 T-01/02/04，T-03 排 690 merge 后（同 worktree 续做或返回复工）。

- **T-01** 两处注释订正（AC-01；§5 刀①）。验证：git diff 人工审。
  [✅ 已完成] worktree commit `f876e2661`（2 文件 +5/-2，纯注释）。
- **T-02** gpui 死后端移除（AC-02；§5 刀②）。验证：§6 T-02 行三命令全绿 + grep 零命中。
  [✅ 已完成] commit `a5c5d0d49`（39 文件 +57/-4030）。勘正四枚：①移除面比立项
  勘定多 `style/gpui_adapter.rs`（554 行，style/mod.rs 两处门控挂载）；②examples 面
  19 个非 8 个；③裸 `--examples` 的 p023_probe 14×E0433 为 master 预存（主检出同位
  对拍在案），rider 补 `required-features = ["iced-layout-tests"]` 门收口；④AC-02
  grep 口径勘正为"渲染后端面零活引用+数据面豁免"（r2）。门禁：check 0 错；
  examples(build-examples) 0 错；cargo t 9 红全预存零新增（musk×6 在册族 +
  desktop_protocol×2[690 档"2 红全预存"] + a2vue×1[682 转告]；主检出 28 红含
  blueprints 盘损环境红 19 对照）。
- **T-03** pixels 臂退役门勘定 + 条件执行（AC-03；§5 刀③）。验证：§6 T-03 行 +
  勘定记录入 §9。**[✅ 勘定收口=顺延路径]**（2026-09-22，Q-2 默认裁定执行）：
  量化门达标在案（034 memory-matrix 结论行：release×tiny-skia×1窗 11260KB ≤
  102400KB；app 6460KB ≤10MB）；**条件②未达**——现行设计档（desktop-protocol-v1.md，
  post-690 master）:816 labels not-yet（软栅格无文本面）+:824/:827 terminal/imagesurface
  M7-c 撞面未立项——像素原生族外 widget 覆盖未收口。**裁定：顺延整刀，不删**；等
  M7-c 批+labels 收口后另立。勘定附带：P-RQ-PIX 债册行已在 684-690 期间账面变动中
  不在册（实质退役门=design/autoui/desktop-protocol-v1.md §1.15 门判定行）——
  SD-02 目标随勘正为该设计档行（不再指向债册行）。
- **T-04** 散测试归位（AC-04；§5 刀④）。验证：§6 T-04 行四档 + rename 100% 对拍。
  [✅ 已完成] commit `24ddad334`：90 文件+test_util/ git mv（90×R100 + 2×R09x
  =plan492 互引修）；lib.rs -91 声明块迁 tests.rs（cfg 门/溯源注释原样）。
  勘正三枚：①**test_runner 例外留根**——生产依赖勘定（vm/ffi/stdlib.rs:10921
  `auto.test.*` natives 调 discover/run，Plan 263），91→90；②plan077_integration_tests
  为死文件（声明早已注释，1.3k 行从未编译）随迁保留注释态（删除候选另记）；
  ③外引修 2+2 处（plan492_m4/m5 + conformance_tests 两行 test_util use）。
  **事故与恢复在案**：追加脚本首次运行在 join 处抛 TypeError，但 `open('w')` 已先
  截断 tests.rs——第二脚读到空串写入→既有 67 声明丢失（~427 测试静默消失，零编译
  错）。nextest list 双侧全名对拍发现；恢复=git 原内容+追加段重拼（157 声明验证）。
  **四档门禁全过**：t=5451/9 红（与搬移前基线逐位全等，9 红全预存同名）；tv/tt/tb
  同 9 红家族零新增（5599/5821/5506 总数）。
- **T-05** 全量门禁 `cargo tf`（AC-05 前半）。
  [✅ 已完成] tf=5452/10 红：9 预存家族（musk×6+desktop_protocol×2+a2vue×1）+
  ffi_dual_019（tf 满配并行调度敏感 flaky，689 在册 ffi flaky 家族；双侧 solo
  复跑全绿 worktree 3.2s/main 2.7s，非移动致伤）。零新增红。
- **T-06** 复审（/auto-plan:review）+ merge 账本三件套 + 归档（AC-05 后半）。

## 9. 复审记录

- 2026-09-22 draft handoff（auto-plan:new）：stage=new，outcome=pass（待用户确认 Q-1..
  Q-3 后即可 /auto-plan:work），next=work。
- 2026-09-22 work start（auto-plan:work）：用户裁定 Q-1=gpui 删除（未来需要再独立加，
  属拆仓后）；Q-2/Q-3 采默认。开工序：master 提交骨架 → worktree lang-691/plan-691-dev。
  T-03 前置 PLAN-690 未合（plan-690-dev 在途），本批执行 T-01/02/04/05。
- 2026-09-22 work handoff（auto-plan:work）：`stage: work | PLAN-691 | r2 |
  outcome: pass | code: plan-691-dev @24ddad334（T-01 f876e2661→T-02 a5c5d0d49→
  T-04 24ddad334，三 commit）| tasks: T-01..T-05（T-03=勘定顺延路径收口）|
  evidence: 四档 t/tv/tt/tb+tf 全过零新增红（红集=预存 9+在册 ffi flaky 1）|
  blockers: 无 | next: review`。**复审注意事项**：①worktree 内 4 个 docs/specs
  文件为他方会话 SD-03 位写（未暂存，merge 时裁定归属）；②分支基点 90d578c08，
  master 已前移（690 合入+684/690 归档+692 起草）——merge 阶段须 rebase，
  冲突预估面=Cargo.toml（690 亦改）；③master 主检出 blueprints/ 盘损（他方
  会话作业，68 文件未暂存删除+空嵌套目录）未代处置，需其归属会话或用户路由。
- 2026-09-22 review（auto-plan:review，**独立性声明：本会话=执行会话，裁定全部从
  工件重推**——git diff/HEAD 树 grep/增量重跑，不采信执行期自述）：
  `stage: review | PLAN-691 | r2 | outcome: **pass** | reviewed_commit:
  24ddad334372acd7681b3a4bb355cec69ed77724 | base: 90d578c08 | deps:
  auto-down@3373a5c（组内 detached 依赖位）| spec_inputs: design/32 §0,
  design/autoui/desktop-protocol-v1.md §1.15, specs/auto-lang/{project,ui/*} |
  验收: AC-01 pass（f876e2661 双 hunk 逐字核，纯注释）；AC-02 pass（HEAD 树渲染面
  grep 零命中[余=Cargo.toml 两行墓碑注记]；依赖四件全无；**裸 --examples 复审增量
  重跑=0 错**[p023 门生效直接证据]；examples(build-examples) 0 错@a5c5d0d49 复用
  理由=T-04 未触 examples）；AC-03 pass=顺延路径（门②证据=master 设计档 :816 labels
  not-yet/:824/:827 terminal+imagesurface M7-c；Q-2 默认裁定）；AC-04 pass（rename
  90×R100+2×R09x+test_util/；根残留仅 test_runner；lib.rs 余量=先在 #[path] 定向
  声明族[见 F-1]；tests.rs=157 声明；t=5451/9 与基线逐位全等+tv/tt/tb/tf 四档回执；
  告警 351→344 纯减 7 零新增[复审增量对拍]）；AC-05 gates 面 pass（tf 10 红=预存 9+
  ffi_dual_019 flaky——689 债册"ffi flaky 留档"+684 晚间 tf 基线 10 红双互证+双侧
  solo 绿；账本三件套=merge 阶段 T-06 设计内）| findings: F-1(info) lib.rs 存先在
  `#[path="tests/…"]` 定向声明族 ~10 处（plan510/plan608/plan394/plan577×2/
  gallery_pages/plan632/plan664/plan633/back_proxy…基点即有，非 T-04 缺陷）——与
  归位后形态不同构，拆仓阶段 2 前可一次归并进 tests.rs（候选，非债）；F-2(info)
  plan077_integration_tests 死文件（1.3k 行，声明注释态随迁）删除候选待裁定 |
  route: R-1 worktree 内 4 docs/specs 外部位写（他会话 SD-03，±6 行）内容与实现
  一致——**裁定 merge 收编并注明来源**（对方会话已按用户裁定退出，退回无主）|
  next: merge`

- 2026-09-22 merge（auto-plan:merge）consolidation receipt `PLAN-691:r2`：
  - `prepared`：reviewed@24ddad334 → SD-03 specs 收编 commit（rebased ffe800947，来源注
    并行会话）+ 账本三件套 commit 0236239ae（plans.md 行 691/孤儿标记清除+specs.json
    P691-1(tests)/P691-2(reviews) 660 条+INDEX 再生幂等零 diff）；编码事故一次（heredoc
    中文按错码页解码污染 plans.md 行+提交信息）即撤即修（Write 工具脚本+msg 文件重提，
    定损=早期 12 文件清扫与 specs.json 未殃及）。
  - `landed`：rebase master 干净 4/4 **range-diff 全等**（f876e2661→c60cf8905/
    a5c5d0d49→3ac279750/24ddad334→a72257e6f/0cac22a25→ffe800947）；rebased 态复验=
    cargo check 0 错+cargo t 5453/9 与 master 基线同集（含 p690 两测全过）；
    `git merge --ff-only` → master tip=0236239ae=delivery commit（零合并提交）；主检出
    烟测 check 0 错。
  - `ledger_refreshed`：.autoos/specs.json P691-1/P691-2 外科插入（json.loads 全文验证+
    节位 tests/reviews 各一）；docs/specs/auto-lang/ui/plans.md 表行；INDEX 再生幂等。
  - `archived`：git mv → docs/plans/archive/691-presplit-subtraction.md，status:
    archived，completion_kind: delivered。
  - `cleaned`：✅ wt-guard 双 clean（reparse 零）→ auto-down 依赖位归属仓移除 →
    lang-691/auto-lang 移除 → plan-691-dev 删（was 0236239ae=delivery tip）→ 组目录清。
    **五 checkpoint 闭环，completion_kind: delivered。**

## 10. 待澄清事项
- **Q-1（✅ 已裁定 2026-09-22 用户）**：**gpui 删除**——未来有需要再独立加（且属 auto-ui
  拆分出去之后的事）。git 历史即封存。
- **Q-2（T-03 内裁定）**：pixels 门②widget 覆盖现状若勘定不达标（terminal M7-c 撞面
  未收口等）——顺延整刀（默认）还是放宽门？默认顺延。
- **Q-3（建议即采）**：散测试归位形态平铺 `src/tests/`（默认，机械优先、rename 100%
  可证）vs 域子目录（ui/core/vm——分类价值留给拆仓阶段 2，那时随资产迁移一次到位）。
  默认平铺。
