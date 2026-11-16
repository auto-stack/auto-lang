---
plan_id: PLAN-1065
status: drafting
feature_name: AC后端sys973平台操作真身交付
author: [ZCode]
created_at: 2026-11-16
updated_at: 2026-11-16
plan_revision: 1
current_step: 0
total_steps: 6
depends_on: [PLAN-973]
supersedes_spec_components: []
new_spec_components: [docs/specs/ace-migration/m2-stdlib-io.md]
touched_goals: [GOAL-002]
affects: [ace, stdlib]
---

# [PLAN-1065] AC后端sys973平台操作真身交付

## 0. 变更摘要

执行用户2026-11-16裁定（[PLAN-973](973-ace-m2-stdlib-io.md) T-05 复审发现 F1，选项 b）：AC 后端（直接编译，cranelift COFF/ELF）的 **50 个 sys973 平台操作真身**（现全部为 named-unsupported stub(-10) 诚实拒绝）**具名延期到本计划**交付；973 的 V-09/AC-07/AC-COVERAGE 行文修订为"AC stub=已裁定交付面"，AC 真实 OS 调用正例腿由本计划关闭。本计划 drafting 0/6，仅规划承接，未实施、未建树、未运行测试。

## 1. 目标

1. 为 `experimental/ace/crates/native` 发射器的 sys973 操作集（`SYS973_KINDS` 50 项：io_stdin×1/file×11/path·dir·canonical·replace·attrs×17/env×4/clock×4/entropy×1/proc×12）实现**真实发射体**替代 `emit_sys973_stub`，Windows（Win64 COFF，kernel32/bcrypt 平台 ABI 直调）先行。
2. 交付 V-09 的 AC 正例腿："AC真实调用OS API且不先产C/Rust业务源，同一公共系统程序四端运行"——AC 腿跑通 probe 级全链（真实磁盘/进程/时钟/熵可观察输出，独立 oracle）。
3. AC-COVERAGE 的 AC 路径从"stub 拒绝"升为真实运行；973 已交付的 stub 拒绝语义（-10、无静默 fallback）保留为负控。

非目标：AV/native、AV×C、AV×Rust、A2R、A2PY 路径（973 已交付）；Linux 真跑（归 [1063](1063-ace-stdlib-linux-deferred-validation.md) L-12 协同：本计划交 Windows 真身+Linux 代码臂，1063 认证）；byte→Str 面（归 [1064](1064-ace-stdlib-byte-str-deferred-surfaces.md)）。

## 2. 架构方案

两模式均有 1039/1041 存量先例，T-01 冻结逐操作选择：a) cranelift 内联体（如 mem_read_u8 的 bounds-check/load 形）；b) shim 符号链接（如 `IoWriteSome`→`shim_symbol_for`+链接期解析）。约束：AC 直接生成机器码经平台 ABI 调 OS API，不先产 C/Rust 业务源；不引入 libc/CRT 默认委托；操作冻结形状（i32→i32、lo+hi、UTF-16/bytes 双协议）不变。973 的四表对齐（source/av-core/native/a2x）与冻结装配集原样复用，本计划只替换 native 表的 stub 发射为真身。

## 3. 技术栈

现有 ACE workspace（experimental/ace/crates/native）、Win32 平台 ABI、TestCase/固定组（`ace-native`）。唯一拟实施树 `D:/autostack/.wt/lang-1065/auto-lang`（分支 `plan-1065-dev`，基面=973 合入后 master），尚未创建；材料 scope `D:/autostack/plan-artifacts/plan-1065/`，开工登记。

## 4. 需求分析与背景调查

- 来源裁定：973 §10（2026-11-16 F1 选项 b）+ T-05 复审记录（§9 第 6 轮）：`native/src/platform_io.rs:139-142`（"a REAL body lands with its AC increment"——本计划即该 increment 的具名 owner）、`:1476-1482`（Win64 全 stub 发射点）、`platform_io_linux.rs:1066-1070`（ELF 同）。
- 先例：`native/src/platform_io.rs` 内存操作真身（cranelift 内联，L1440 一带）、`IoWriteSome` shim 符号模式（L778-804）；1041 console 链接模式。
- 依赖事实：973 冻结操作表/装配集/绑定双点身份（973 合入后为 master 事实）；`is_sys973_kind`/`SYS973_KINDS` 即本计划替换集清单。

## 5. 详细设计

| # | 项 | 现状（973 基线） | 本计划交付 |
|---|---|---|---|
| A-01 | Win64 真身 50 操作 | stub(-10) | 逐操作真发射体（模式 a/b 由 T-01 冻结），保 -10 负控语义为显式测试 |
| A-02 | V-09 AC 正例腿 | 未交付 | probe 级公共系统程序 AC 编译+运行，真实 OS 可观察输出+独立 oracle |
| A-03 | AC-COVERAGE AC 路径 | stub 拒绝测试（sys973_native_stub_returns_unsupported） | 真实运行绿 + 拒绝负控保留 |
| A-04 | Linux ELF 真身 | stub 发射 | 代码臂（code-only），真跑归 1063 L-12 |

规范增量（实际交付后沉淀）：SD-ACBODY modify `docs/specs/ace-migration/m2-stdlib-io.md`——AC 路径"stub=已裁定交付面"→"Windows 真身已验/Linux 归 1063"。

## 6. 测试设计

| Case | AC | 判据 | 入口 |
|---|---|---|---|
| V-AC-01 | AC-02 | 50 操作逐族真实行为对独立 oracle（磁盘字节同一性/seek/EOF/权限/env/时钟带/熵/spawn 退出码），负控：不支持面仍 -10 拒绝 | ace-native conformance 新例，≤10s/条 |
| V-AC-02 | AC-03 | 同一 probe01 公共程序 AV vs AC 四端同值 | av/native conformance |
| V-AC-03 | AC-04 | 链接来源审计：无 libc/CRT 默认委托，kernel32/bcrypt 直调 | 代码审查+符号核对 |

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 逐操作模式冻结（内联 vs shim）+替换集清单核对无缺项 | T-01 |
| AC-02 | work | Win64 50 真身+V-AC-01 全绿 | T-02 |
| AC-03 | work | V-09 AC 正例腿+四端同值 | T-03 |
| AC-04 | work | 链接来源审计过+负控保留 | T-04 |
| AC-05 | review/merge | 复审+沉淀（SD-ACBODY）+归档 | T-05/T-06 |

## 8. 执行步骤

- [ ] **T-01｜work｜AC-01**：逐操作发射模式冻结+先例核对+替换集对账。
- [ ] **T-02｜work｜AC-02**：Win64 50 真身实现+V-AC-01。
- [ ] **T-03｜work｜AC-03**：V-09 AC 正例腿（probe 全链）+V-AC-02。
- [ ] **T-04｜work｜AC-04**：链接审计+负控+Linux 代码臂（A-04）。
- [ ] **T-05｜review｜AC-05**：独立复审。
- [ ] **T-06｜merge｜AC-05**：沉淀、归档、清树。

## 9. 复审记录

2026-11-16 stage: new | 本计划由 PLAN-973 T-05 复审 F1（用户裁定选项 b）创建为 AC 真身承接 owner；取号 1065（master 主检出）。仅规划：A-01..A-04 承接表+V/AC/任务 6 项；未实施、未建 worktree、未运行测试。973 不反向依赖本计划。

## 10. 待澄清事项

无（承接边界已由 973 §10 F1 裁定冻结；逐操作模式选择属 T-01 工程决定，无需用户裁定）。
