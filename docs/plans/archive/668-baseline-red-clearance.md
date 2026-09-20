---
plan_id: PLAN-668
status: archived               # 终态（archived plans do not go back）
feature_name: baseline-red-clearance
author: [zcode]
created_at: 2026-09-20
updated_at: 2026-09-20
plan_revision: 1
current_step: 9
total_steps: 10

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: ["docs/specs/auto-man/project.md"]
touched_goals: []             # 修复批无 GOAL-NNN 映射，见 §5 规范增量说明

affects: [auto-man, auto-lang/ui]
---

# [PLAN-668] baseline-red-clearance（存量红基线清零批）

## 0. 变更摘要

把 `KNOWN-DEBT-AND-RISKS.md` 在册的 master 预存测试红（tv/daily 两红、iced 档四红、
564-Q6 老家族、test-trans golden 五件、aavm 金样两件、零散集成红六件）与 examples
构建链红（脚手架类型面双错、038 vue-tsc、017/025 api_gen 后端转译）集成清偿；逐条
复核分类（修复 / 核销-复核不红 / 环境红豁免 / 域外转介）并回写台账。产出：各档门禁
从近六个计划被迫沿用的"相对基线零新增红"口径回到**绝对全绿**（环境红豁免清单除外）。

## 1. 目标

### 1.1 目标

1. **G-A 测试档红清零**：R-01..R-20（§5 清单）中经 T-01 复核确认仍红的条目全部修复
   或按分类处置，终验矩阵对应滤串全绿。
2. **G-B examples 构建链红清零**：R-21..R-24——抽样示例集 `auto build` 的 vue-tsc 面
   绿、017/025 rust 后端转译产物 cargo check 绿。
3. **G-C 台账治理**：本批全部条目在 KNOWN-DEBT-AND-RISKS 落处置（✅修复+提交号 /
   核销-复核不红 / 环境红豁免注记 / 域外转介指针），后续计划复审不再重复对账本批签名。

### 1.2 非目标（Non-goals）

- **musk p053 家族修复**（R-25）：musk 域在案（P645-D2/P028-D4/P648-D2），本批仅做
  master 签名复测与呈报，不修改 musk 域代码。
- **P660-D4 rust/a2r 臂四类系统性缺口**（use fn 不发射/中文 mojibake/button 子文案/
  `[]str` Value）：真 codegen 能力缺口非陈旧产物，归 vm-component-parity / a2r 后续
  专项（014-weather 已以语料规避）。
- **P581-D1 website vitepress build 红**：台账标"待用户裁定"，不在本批。
- **RC/内存族**（P506-1 038 UAF、P667-D1、P659-D1、585 等）：独立专项候选。
- **物理机/实机清单族、dep-rust V2 族（P591/P592/P596）、主题族**：各有既定触发条件。
- **L0 顺手修批**（P665-D6 spec-index.py 根治、P663-D5 ui/component.rs:97 cfg 门控、
  P632-D1、P665-D5/D7）：另行 fix worktree，不混入本批。
- **环境红 stabilize 工程**：R-11（ui::layout 几何族）等环境依赖红默认只登记豁免口径
  （576 债注记：数量随会话 6↔9↔14 浮动），不做稳定化改造。
- 不动 auto-os / auto-down / musk 域文件；不引入新功能。

### 1.3 成功样貌

`cargo tv` / `cargo tt`（目标滤串）/ `cargo t iced`（目标滤串）/ `cargo taa
test_aavm2_goldens_check` 全绿；抽样示例构建链绿；KNOWN-DEBT 中本批 27 条全部闭合；
下一个计划的复审门禁可以直说"全绿"，无需再附基线对照表。

## 2. 架构方案

无新架构，纯清偿批（先例：Plan 515 桌面 DEBT 批处理一/二期、Plan 524、Plan 513
整合清理批）。工作形态：

- **单 worktree**：`D:/autostack/.wt/lang-668/auto-lang`（plan-668-dev），T-01..T-09
  全部在 worktree 内完成；计划簿记（本文件）留主检出。
- **先勘定后动手**：T-01 在 worktree（同 master HEAD）跑全档电池实采红签名——台账
  归因是历史快照，部分条目可能已被后续计划顺修（如 R-08 c2_param_msg 曾由 492 M6
  恢复过）；复核不红的直接核销，避免修已不存在的红。
- **三组横切**：G-A（测试断言/golden 再生/表同步，auto-lang 侧）、G-B（脚手架与
  生成器，auto-man/ui_gen 侧）、G-C（复核呈报+台账回写）。
- **风险与合流**：在途并行会话（plan-036 等）脏文件含 renderer.rs——本批 R-05 修复
  面（aura_view_builder.rs convert_popover）与其可能相邻，合并期如遇重叠按多会话
  礼仪对账；T-01 开工前复核 master 状态。

## 3. 技术栈

- Rust 测试档：`cargo t / tf / tv / tt`（nextest 别名）、`cargo taa
  test_aavm2_goldens_check`（aavm 金样档）、`--features ui-iced` 特性档、insta 快照
  （`cargo insta accept`）。
- golden 再生：a2r golden 逐例 bless + 人工核验；aavm2 期望文件再生。
- 构建链验证：`auto build`（vue-tsc && vite build）——**P075-D1 沙箱纪律**：凡
  `auto build`/`Pac::resolve` 类验证一律在仓库外临时目录跑（复制工程→构建→先 rmdir
  摘 deps 链接再整树删）；pnpm/vite/cargo 仓库内允许（node_modules 实勘无 reparse
  point）。
- 诊断辅助：stash 双跑对照、detached 基点复现（既有方法论，台账各条已含复现命令）。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户于 2026-09-20 会话批准："按照你推荐的候选，起草计划文档"——**授权范围=起草
  本计划**；执行（进入 executing + 建 worktree）须按 AGENTS L1 步 2 再确认
  （见 §10 待澄清 1）。
- 仓界：auto-lang 主仓（含 crates/auto-man）。预算/自动延续上限：未指定。

### 4.2 背景调查（台账来源，均已含归因与复现命令）

- 近六个计划重复交税的实证：P667-D2（合并门禁改"相对基线零新增红"口径）、
  P661-D7（661 基线对跑实录 tv 2 红 + iced 4 红 + ui 滤串 ~50-52 红）、P662-D2、
  P645-D2、P028-D4、P648-D2（musk p053 家族三次独立发现）、FIX-REORG-D1。
- 564-Q6 老家族：2026-09-05 基点实证 15+ 失败（plan370 d8 / plan492 c2_param_msg /
  ui::layout grid 与 master_stack 全族 / ui::iced lucide manifest / aura strip_html），
  "需维护者排查归位"——至今无人认领。
- test-trans golden：018-T05（`14_modules/007_shared_var`+`27_c_abi`+实编门 3
  unexpected）与 P667-D2 同集；P566 记 aavm2 金样 b13/b32 为 577 再生遗漏。
- examples 构建链：P660-D1+P657-D2（两错打红全部示例 auto build：tsconfig 无
  vite/client 类型；auto-sources 仅 `auto run` 写入而 build 阶段缺位）；P666-D2
  （038 vue-tsc store 名）；P658-C1+P666-D1（017/025 api_gen 后端转译红，两侧对称
  预存=CLI 后端 codegen 演化漂移）。
- 规约状态：本批不触及 docs/specs 现行规则；仅 SD-01/SD-02 为 auto-man 脚手架契约
  增补（§5）。

### 4.3 环境与约束

- **多会话并行**：term-024/lang-025/lang-036/musk-080 等活跃 worktree 在途（均
  2026-09-20 有提交）；master 共享，他方 UU/MERGE_HEAD 默认勿解（受托代解先撤他方
  暂存）。
- **worktree 红线**：内禁 junction/symlink；移除前 `bash D:/autostack/wt-guard.sh`。
- **tf 顺序性红基线**（工具链雷区在案）：终验以 tf 全量跑一次为口径，顺序性差异按
  T-01 分类处理。
- **prismjs 钉 1.29.0**（1.30.0 全域阻断 vue dev）：构建链验证若涉 pnpm install，
  遇浮动解析按钉版处置（656 F-R1 先例：探针脚本内置钉版自愈）。
- **测试副作用**：auto-man 测试会写脏 `examples/rust-workspace/**` tracked 产物
  （P580-D3/P633-D3 在案）——基线跑若产生脏 diff，记录后还原，不误当回归。

## 5. 详细设计

### 5.1 条目清单（R-01..R-27）与初判修法

**G-A 测试档红（auto-lang 侧）**

| # | 条目（测试/现象） | 台账来源 | 初判修法 |
|---|---|---|---|
| R-01 | `ui_gen::rust::tests::mouse_area_emits_events_and_logical_extent` | P667-D2 / P661-D7 | 疑 660 系 rust/a2r 合入漂移；对照 660 §5.6 对拍表定位后修发射或断言 |
| R-02 | `ui_gen::rust::tests::test_autodown_panel_heading_codegen` | 同上 | 同上 |
| R-03 | `cargo t iced` 滤串：`lucide_icon_coverage` | P661-D7② | 逐条定位（manifest/清单再生或断言） |
| R-04 | `conditional_style_hover` | P661-D7② | 逐条定位 |
| R-05 | `p010_popover_ondismiss_extracted_from_events` | P661-D7② + 027 在册 | 027 已归因：解释臂拖拽幽灵 popover 被判 widget 形态——修 `aura_view_builder.rs convert_popover` 坐标锚判定 |
| R-06 | `external_config_poll` | P661-D7② + P667-D2 | P667-D2 标"并行/环境闪测（单跑绿）"→ T-01 先分类，环境红则豁免登记 |
| R-07 | plan370 `d8_toggle_dark_mode` | 564-Q6 / P524-1 | 015 默认翻 true 断言未跟——断言跟语义 |
| R-08 | plan492 `c2_param_msg_declaration_both_tracks_alive` | 564-Q6 | 492 M6 曾恢复过——复核现行态后修 |
| R-09 | plan055 `strip_html`（双空格） | P502-1 / P524-1 | 断言跟语义（strip 后双空格行为裁定） |
| R-10 | `ui::iced lucide manifest` | 564-Q6 | manifest 再生或断言对齐 |
| R-11 | `ui::layout` grid/master_stack 族 | 564-Q6 + 576 注记 | **环境红候选**（本机显示几何，数量随会话 6↔9↔14 浮动）→ 分类定案豁免口径，不盲修 |
| R-12 | test-trans golden 五件：`14_modules/007_shared_var`、`27_c_abi/003`、`27_c_abi/004`、`a2r_rustc_real_compile_gate`（含 024_nested_async_await 解析错） | P667-D2 / 018-T05 | 逐例 bless 再生（人工核验）或便宜根因修；台账原话"逐例 bless 或根因修+归因清账" |
| R-13 | aavm2 金样 `b13_is_enum` / `b32_is_break_continue` | P566 | 577 再生遗漏的两件 expected.rs 再生（bless） |
| R-14 | `covered_elements_within_target_set` | P566 / P033-D4 | imagesurface 登记（element_coverage.rs:43）与 `Coverage::target_set()` kinds 表脱钩——target_set 增 imagesurface 臂或登记降级（P033-D4 注"归 PLAN-656 表同步收口"，本批承接） |
| R-15 | `test_display_family_codegen_arm_fixture`（icon size 14→20px） | P649 / 564-Q6 | 特性配置债：无 ui-iced 必红/有则绿——`ui_gen/rust.rs:3227` 一带 icon 臂默认档路径修正，双特性态同绿 |
| R-16 | plan606 `test_029_photo_gallery_thumbnails...`（data-URL 断言） | P642-D5 / D11 | 断言过时（语料已演进为路径引用+渲染端内嵌契约）→ 更新断言到现契约 |
| R-17 | `plan488_dnd_bridge_app_handlers` | FIX-REORG-D1 | dnd-bridge app.at 解析失败（caption_text UndefinedVariable + 20 处 RBrace）→ 修 app.at 源或定位解析器回归 |
| R-18 | `ui_snapshots__editor.snap` / `__sidebar.snap` 过期 | P451 复审 | `cargo insta accept`（015-notes SFC 字节漂移） |
| R-19 | docs_gen `kitchen_sink_page_in_sync` + `schema_drift_fence` | P528-D6 | baseline 裁剪（`SCHEMA_DRIFT_UPDATE_BASELINE=1`）+ kitchen_sink 同步 |
| R-20 | spa-routes e2e 5 红（/ui/* title 漂移） | P582-D1 | 资产重生成或断言修 |

**G-B examples 构建链红（auto-man / ui_gen 侧）**

| # | 条目 | 台账来源 | 初判修法 |
|---|---|---|---|
| R-21 | 全部示例 `auto build` vue-tsc 红：`main.ts TS2339 import.meta.env` | P660-D1 / P657-D2② | 脚手架 tsconfig types 补 `"vite/client"`（auto-man gen 模板）——一处修全域绿 |
| R-22 | `overlay.ts TS2307 '../auto-sources'`（013/046/047） | P657-D2① | 阶段序缺口：build 阶段也写 auto-sources，或 overlay 改惰性类型（T-06 内定，SD-02） |
| R-23 | 038-minesweeper vue-tsc：`useMinesweeperStore.ts:282/287` store 名未解析 | P666-D2 | 生成器演化滞后定位（codegen 演化 vs 陈旧 gen 产物对照） |
| R-24 | 017-chat / 025-sys-monitor rust 后端转译红（017 db.rs `"AutoBot"` E0308 八处；025 29 错） | P658-C1 / P666-D1 | api_gen/a2r 演化漂移——种子字面量 &str 落 String 槽等；两侧对称预存确认后逐族修 |

**G-C 复核/呈报/核销**

| # | 条目 | 台账来源 | 处置 |
|---|---|---|---|
| R-25 | musk_vm_track p053 家族 4 红（p053_1×2 / p053_4 / p053_6） | P645-D2 / P028-D4 / P648-D2 | musk 域——本批仅 master 签名复测+呈报，不修 |
| R-26 | P551-D2 autodown feature path 依赖失效 | P551-D2 | 复核是否已被随行修（近期各计划 cargo 正常，疑条件性失效）——不红则核销 |
| R-27 | 台账核销回写（R-01..R-26 全部处置落册） | 本批 | T-08 执行 |

### 5.2 任务分组策略

按"独立可验证结果"分组：发射/断言族（T-02）、564-Q6 语义族（T-03）、golden/快照
再生批（T-04）、断言契约与表同步（T-05）、脚手架/生成器（T-06）、api_gen 后端
（T-07）、呈报与核销（T-08）、终验（T-09）、合并收尾（T-10）。每任务收口即跑该组
滤串，不留"最后一起验"。

### 5.3 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-man/project.md | before：脚手架 tsconfig 无 vite/client 类型约定（main.ts `import.meta.env` 依赖隐式全局，vue-tsc TS2339）。after：gen 模板 tsconfig `types` 显式含 `"vite/client"` | P660-D1 根修落入生成器模板，属生成器行为契约 | AC-07 |
| SD-02 | add | docs/specs/auto-man/project.md | before：`auto-sources.ts` 仅 `auto run` 驱动写入（auto-man vue.rs write_auto_sources_ts），build 阶段缺位。after：`auto build` 阶段同样产出 auto-sources（或 overlay 改惰性类型——T-06 勘定后复审终稿回填） | P657-D2① 阶段序缺口的契约化 | AC-07 |

**无其余规范面变更的说明**：本批其余条目均为测试断言修正、golden/快照再生、登记表
同步与语料小修——行为契约不变，不改 docs/specs 现行规则。

## 6. 测试设计

- **基线电池（T-01，worktree 内，同 master HEAD）**：`cargo t`、`cargo tv`、
  `cargo tt`、`cargo nextest run -p auto-lang --features ui-iced`（ui-iced 档）、
  `cargo taa test_aavm2_goldens_check`；ambiguous 项（环境 vs 代码）在主检出做签名
  对照复测。产出 `docs/plans/evidence/p668/baseline.md`。
- **分组验证（各 T 收口）**：见 §8 各任务验证命令——滤串级，快。
- **终验矩阵（T-09）**：`cargo tv` + `cargo tt` + `cargo t` + ui-iced 特性档 +
  `cargo taa test_aavm2_goldens_check` + `cargo tf`（全量一次，顺序性红按 T-01 分类）
  + 抽样示例构建链复跑（沙箱）。判据：相对 T-01 基线，目标条目全绿；残余全部落入
  环境红豁免清单（随 AC-01 呈报）。
- **构建链验证沙箱纪律**：抽样集 {013-todo, 014-weather, 017-chat, 025-sys-monitor,
  038-minesweeper} + 046/047 复测（P657-D2 原发现面）——`auto build` 在仓库外临时
  目录执行（P075-D1），vue-tsc 面为判据。

## 7. 验收标准

- **AC-01 基线勘定产物**：`docs/plans/evidence/p668/baseline.md` 存在，含 (a) 各档
  实跑红签名全集；(b) §5 清单 R-01..R-26 逐一分类：修复（并入 T-xx）/ 核销-已不红 /
  环境红豁免（口径+复测方法）/ 域外转介——无"未分类"项。验证：文件审阅 + 分类表
  覆盖率核对（26/26）。
- **AC-02 tv/daily 两红清零**：T-09 终验中 `cargo tv` 与 `cargo t` 对应滤串
  （mouse_area_emits_events_and_logical_extent / test_autodown_panel_heading_codegen）
  绿（R-01/R-02；若 T-01 判核销-已不红则以复测记录替代）。
- **AC-03 iced/ui 档红清零**：`cargo t iced` 目标滤串（lucide_icon_coverage /
  conditional_style_hover / p010_popover_ondismiss）绿（R-03..R-05）；R-06、R-11 按
  T-01 分类定案（环境红豁免登记或修复）；`covered_elements_within_target_set` 绿
  （R-14）；`test_display_family_codegen_arm_fixture` 在有/无 ui-iced 两特性组合下
  同绿（R-15，P649 双态复现口径）。
- **AC-04 test-trans golden 清零**：`cargo tt` 目标滤串（14_modules_007_shared_var /
  27_c_abi / a2r_rustc_real_compile_gate）绿；每件 bless 附人工核验记录（evidence，
  R-12）。
- **AC-05 aavm 金样清零**：`cargo taa test_aavm2_goldens_check` 全绿（R-13）。
- **AC-06 零散集成红清零**：plan606 029 断言、plan488 dnd-bridge、ui_snapshots 双
  快照、docs_gen kitchen_sink/schema_drift、spa-routes e2e 各自滤串绿或按分类处置
  落册（R-16..R-20）。
- **AC-07 examples 构建链清零**：抽样集 + 046/047 的 `auto build` vue-tsc 面绿
  （R-21/R-22/R-23）；017/025 rust 后端转译产物 cargo check 绿（R-24，按 T-07 诊断
  收敛口径）；SD-01/SD-02 契约行落笔。
- **AC-08 台账核销回写**：KNOWN-DEBT-AND-RISKS 中 R-01..R-27 全部落处置（✅修复+
  提交号 / 核销-复核不红 / 环境红豁免注记 / 域外转介指针），564-Q6 总条目随家族
  清偿状态更新；无遗留开口（R-27）。
- **AC-09 终验与收尾**：§6 终验矩阵通过（残余仅环境红豁免清单内项）；wt-guard
  clean → 合并 master → 收据回填本计划。

## 8. 执行步骤

> 全部代码工作在 `D:/autostack/.wt/lang-668/auto-lang`（T-00 建）；每步完成后追加
> `[✅ 已完成]` 证据行。

- **T-00 建组**：主检出 commit 簿记（本文件+.next-id）→ `git worktree add
  D:/autostack/.wt/lang-668/auto-lang -b plan-668-dev`；flip `status: executing`。
  验证：worktree HEAD==master。
  [✅ 已完成] 2026-09-20：flip 提交 27c196c04；worktree 建于
  D:/autostack/.wt/lang-668/auto-lang（plan-668-dev），HEAD 27c196c04==master。
- **T-01 基线勘定与分类**（→AC-01）：在 worktree 跑 §6 基线电池；对 §5 清单 26 项
  逐条复测签名（台账复现命令为准），分类四态；ambiguous 项主检出对照。产出
  `docs/plans/evidence/p668/baseline.md`。验证：分类表 26/26 覆盖。
  [✅ 已完成] 2026-09-20：证据=docs/plans/evidence/p668/baseline.md（26/26 四态 +
  计划外 A-01..A-08 入册）。要点：①nextest 0.9.138 默认 fail-fast，采集必须
  --no-fail-fast；②R-06"P667 单跑绿"失效——实为 OS 主题依赖（light 机器必红），
  转修复 T-05；③R-07/R-10/R-15/R-19/R-26 五件核销-已不红；④R-12/13 同根=PLAN-018
  括号化后金样未再生（bless 面）+use.c 清单回退/await 解析两真回归；⑤R-17 根因=
  测试装载器绕过 stylekit 预注册（load_inline 裸 parse），plan339 016 同根（A-01）。
- **T-02 发射/断言族修复**（→AC-02/AC-03）：R-01/R-02（对照 660 对拍表定位）、
  R-03/R-04、R-15（rust.rs:3227 icon 臂双特性态）。验证：各滤串隔离绿。
- **T-03 564-Q6 语义断言族**（→AC-03）：R-07/R-08/R-09/R-10 + R-11 分类定案
  （豁免口径成文：CI 为准/本机浮动记录）。验证：`cargo t`（ui-iced 档）目标滤串绿。
  [✅ 已完成] 2026-09-20：R-07 核销（基线已绿）/R-10 并入 R-03/R-11 环境红豁免
  （15 件 layout.rs:371 同断言，576 在案）；R-08 断言跟 clientX-rect 现行发射
  （旧 offsetX 形态过期）/R-09 strip_html 语义裁定（tag→空格替换不折叠，职责=
  去标签+实体解码）。提交 a8e44e3f9，滤串 2/2 绿。
- **T-04 golden/快照再生批**（→AC-04/AC-05）：R-12（逐例 bless+人工核验）、R-13
  （aavm expected.rs 再生）、R-18（insta accept）。验证：`cargo tt` / `cargo taa
  test_aavm2_goldens_check` / 快照滤串绿。
  [✅ 已完成] 2026-09-20：R-12 双根因修——①003/004 use.c manifest 从未入库
  （`.gitignore` 全局 `*.json` 吞掉——补 `!27_c_abi/**/*.json` 白名单+重建两
  清单，金样级精确复原）；②parser dot 链 `.await/.go` 后缀委托 Pratt（024
  嵌套位解析错清零）；③trans async 块尾 Expr 不补分号（块值复活，实编门
  E0277 清零）。007/005/020/006 金样随 PLAN-018 括号族+块尾语义 bless（diff
  人工核验全同族）；R-13 A2R_BLESS 四金样（b13/b32/g25/b42 同括号族）；R-18
  三快照 accept（app/editor/sidebar——SFC 字节漂移，结构断言不变）。提交
  acf22be45（含 .gitignore 修正）。`cargo tt` 4038/4038 全绿、`cargo taa
  test_aavm2_goldens_check` 绿、实编门 153 compiled/0 unexpected。
- **T-05 断言契约与表同步**（→AC-03/AC-06）：R-05（convert_popover 坐标锚判定源
  修）、R-14（target_set/登记同步）、R-16（029 断言更新）、R-17（dnd-bridge 源修或
  解析器定位）、R-19（schema_drift baseline 裁剪+kitchen_sink）、R-20（spa-routes）。
  验证：各滤串绿。

  [✅ 已完成] 2026-09-20（提交 f1f239918）：R-05 坐标锚判定改声明面/R-06 防回环
  段 manual 源（OS 主题依赖根修）/R-14 表同步三件/R-16 路径引用契约/R-17+A-01
  测试装载器 stylekit 预注册/A-02 逐模块预注册三站点（015 EditorPanel VM 面板
  复活，真生产缺陷）/A-03 夹具补 pac.at/A-04 根形态回调剥离/A-05 跟 625 占位卡
  契约/A-06 别名快照镜像/R-20 断言跟 auto-os 资产名。T-05 后日常档全跑：残余仅
  musk 6 + ui::layout 15 = 豁免+域外集（ffi dep_parity_018 环境挂起另记）。- **T-06 脚手架类型面与生成器**（→AC-07）：R-21（tsconfig types 补 vite/client，
  SD-01）、R-22（auto-sources 阶段序，SD-02——两案勘定择一）、R-23（038 store 名）。
  验证：抽样示例沙箱 `auto build` vue-tsc 绿；`cargo check -p auto-man`。

  [✅ 已完成] 2026-09-20（提交 a377d23ea+70a30ea84）：R-21 tsconfig types:
  [vite/client]（SD-01）/R-22 build 产 auto-sources（SD-02）/R-23 038 三修+013
  随检两修（store 字面别名裸调/List.pop 空值合并/负字面量初始化器/回调契约
  emit 载荷元数取调用位）。抽样集 {013,014,017,025,038}+046/047 沙箱
  （P075-D1 纪律）vue-tsc 面全绿。- **T-07 api_gen 后端转译**（→AC-07）：R-24——017 db.rs 字面量类型族 + 025 全量 29
  错归因分族；修 api_gen/a2r 发射。验证：两例后端产物 cargo check 绿（沙箱）。

  [✅ 已完成] 2026-09-20（提交 70a30ea84）：017=str-let if 臂字面量 .to_string()
  强制；025 六修（sys 直通+边界强转+局部 []T→Vec+a2r-std 依赖模板+qualify 保护
  +内联尾 return 包 JsonResponse+sys 记录字段 i64）。两例后端 cargo check 绿
  （沙箱钉 worktree 绝对路径验产物码）；cargo tt 4038/4038 无回归。- **T-08 呈报与台账核销回写**（→AC-08）：R-25 musk 呈报、R-26 P551-D2 复核、R-27
  全量回写 KNOWN-DEBT-AND-RISKS。验证：台账 diff 审阅，无开口残留。

  [✅ 已完成] 2026-09-20：台账新增 P668 清偿批登记节（R-01..R-26+A-01..A-08
  全数落处置）+22 条在册条目划线回写；564-Q6 家族注销；musk p053×4 呈报+
  p054×2 新观测转介。- **T-09 全档终验**（→AC-09 前半）：§6 终验矩阵一次跑全。验证：判据达 AC-02..AC-07
  口径。
  [✅ 已完成] 2026-09-21：矩阵六档（worktree 计划分支）——tv 3816/3816 ✓/
  tt 4038/4038 ✓/t 5300/5320（残余=musk 6 域外+ui::layout 14 环境豁免，
  clipboard 本轮绿）/desktop_behavior 11/11 ✓/ui_snapshots 3/3 ✓/taa 金样 ✓/
  **tf 3669/3669 全绿**（R-19 tf 档口径确认）；构建链抽样集 7 例沙箱绿
  （{013,014,017,025,038}+046/047 vue-tsc 面；017/025 后端 cargo check）。
  **并流**：master 期间被 PLAN-036/038/669 推进——merge master 入
  plan-668-dev 零冲突（a41a47fab），合并树复核 t 5307/5327 同残余集、tt
  4039/4039 ✓、taa ✓、038 构建链复验绿（合并后 CLI）。AC-02..AC-07 全达。
- **T-10 复审与合并收尾**（→AC-09 后半）：/auto-plan:review → merge master（Conventional
  Commit `fix(test): ... (Plan 668)` 族）→ wt-guard clean → worktree/分支/组目录
  三清 → 收据回填。

## 9. 复审记录

- **2026-09-21 merge 收据（/auto-plan:merge，PLAN-668:r1 五 checkpoint）**：
  stage: merge | outcome: pass（delivered，五 checkpoint 闭环） | delivery_commit: master@b29132f2b（归档收据 ecb03d601） |
  ①prepared：账本三件套 worktree 内提交（specs.json 外科插入 P668-1/2 616→618、
  auto-man plans.md 668 表行、spec-index.py 再生 INDEX——038 vue 行随 master 现行
  同步，SD 节不触模块表零漂）；reviewed 基线 08cd793a3（=review 收据 51e4a98a4 所钉）
  →rebase master@51e4a98a4（669 已落地世代）零冲突，8 提交线性重放，映射：
  93e5aeca8/238bc4dfb/7980c5b91/233a28407/4bf7591a9/4f72a077b/fb9db2d75/3d2dd9482。
  ②landed：`git merge --ff-only plan-668-dev` 主检出 tip==b29132f2b（无合并提交，
  range-diff 等价在 rebase 验证）；落前门禁（rebase 树+账本提交）：tf 3684/3684 ✓/
  tt 4053/4053 ✓/t 5321/5341（残余=musk 6+layout 14 豁免域外集）/038 构建链绿
  （delivery 二进制）；插曲：target incremental 24G 撑盘（os error 112 假编译错）
  ——清 incremental+陈旧沙箱后复验绿。③ledger_refreshed：specs.json P668-1/2 +
  plans.md 表行 + INDEX（见 ①，随 delivery 提交入库）。④archived：本步 git mv +
  status: archived + completion_kind: delivered。⑤cleaned：wt-guard 双查 clean（auto-lang+auto-down）→ worktree remove×2 → branch -d plan-668-dev(was b29132f2b) → 组目录（含 logs 余料）删除 → prune+worktree list+磁盘三查零残留。
- **2026-09-21 review（/auto-plan:review，执行会话内复核——独立性声明：结论自工件重建）**：
  stage: review | PLAN-668 | plan_revision 1 | outcome: **pass**（含一轮 needs_fix 闭环） |
  reviewed_commit: plan-668-dev@08cd793a3（=a41a47fab merge + F-668-R1） | base_commit:
  27c196c04（并流 master 至 002d74e29 世代：PLAN-036/038/669 簿记） | dependency_revisions:
  组内 auto-down detached@fba6563 | spec_inputs: docs/specs/auto-man/project.md（SD 行
  0198f814c 落笔+合并树复核） | acceptance_results: AC-01 pass（baseline.md R-01..R-26
  机械核对零缺+四态零未分类；A-01..08 六行显式两行折叠全覆盖）/AC-02 pass（tv
  3817/3817 新鲜复跑）/AC-03 pass（38 合并滤串新鲜全绿含 display_family 双态：
  tv 无 ui-iced+daily 有 ui-iced 皆绿）/AC-04 pass（tt 4039/4039+实编门 0 unexpected，
  bless 附人工核验记录）/AC-05 pass（taa 金样绿）/AC-06 pass（AC-02/03/06 滤串 38
  测全绿+tf 3670/3670 含 docs_gen 面）/AC-07 pass（沙箱 7 例 vue-tsc 面+017/025 后端
  cargo check 绿+SD 行在档）/AC-08 pass（台账 P668 节+21 处清偿标记+564-Q6 注销，
  机械核对）/AC-09 pass（终验矩阵在案；merge 收尾即本记录后执行） | findings:
  **F-668-R1**（medium→已闭环）：master 并流（PLAN-038 脚手架扩展功能色）致
  ui_snapshots__app 字节漂移 19837→20509B（结构断言不变）——needs_fix 回 work
  一轮 accept（08cd793a3，R-18 同款），ui_snapshots 3/3 复绿+tf 3670/3670 复验。
  非阻塞观察：R-20 playwright 实跑未执行（环境无 browser，静态等价验证四资产
  全符，AC-06"或按分类处置落册"口径内）；R-25/A-08 musk p054×2 新观测已呈报
  转介 | evidence: docs/plans/evidence/p668/baseline.md（在库持久）+本记录内嵌
  摘要（临时日志 .wt/lang-668/logs 随组清理，数字以本记录与 baseline.md 为准） |
  next: merge（/auto-plan:merge——Conventional Commit 族+账本三件套+归档+wt-guard
  三清，组内含 auto-down 依赖位）。
- **2026-09-21 work handoff（/auto-plan:work）**：stage: work | PLAN-668 |
  plan_revision 1（+T-05..T-09 证据增补） | outcome: pass | code_commit:
  plan-668-dev @ merge a41a47fab（T-02=9ad405771/T-03=a8e44e3f9/T-04=acf22be45/
  T-05=f1f239918/T-06=a377d23ea/T-06+07=70a30ea84/SD=0198f814c，基于 27c196c04，
  并流 master 至 a41a47fab） | task_ids: T-00..T-09 全勾（T-10=review/merge 归
  下技能） | evidence: docs/plans/evidence/p668/baseline.md + 本文件各任务
  [✅] 行 + .wt/lang-668/logs/{tv,tt,t,tf,taa,db,snaps}_final.log | blockers: 无
  | next: /auto-plan:review（revision-bound 证据如上）→ merge（Conventional
  Commit `fix(test)/fix(vm)/fix(gen)/docs(specs): ... (Plan 668)` 族）→
  wt-guard clean → worktree/分支/组目录三清（组内含 auto-down 依赖位）。

- **2026-09-20 draft handoff（/auto-plan:new）**：stage: new；PLAN-668 / plan_revision
  1；outcome: pass——合同完备（27 条清单全数映射 AC/T，规范增量 SD-01/SD-02 就位，
  无未归属目标）；next: 待澄清 1（执行确认）裁决后 /auto-plan:work 自 T-00 起。

## 10. 待澄清事项

1. **执行确认**（阻塞 executing）：本稿为起草授权产物；按 AGENTS L1 步 2，执行须
   用户确认后 flip `status: executing` 并建 worktree。
2. **musk p053 家族（R-25）口径**：默认仅复测签名+呈报（musk 域）；若 T-01 诊断
   显示为 auto-lang master 侧便宜根因，是否顺手修？默认否（避免跨域扩战）。
3. **ui::layout 环境红（R-11）豁免口径**：默认登记豁免（CI 为准、本机浮动记录），
   不做 stabilize；如要求本机可复现绿需另立工程，工期另计。
4. **构建链抽样集**：默认 {013, 014, 017, 025, 038} + 046/047 复测；是否需要全量
   33 示例 `auto build`（预计数小时，默认否）。
