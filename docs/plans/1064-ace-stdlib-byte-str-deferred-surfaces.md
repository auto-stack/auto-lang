---
plan_id: PLAN-1064
status: drafting
feature_name: ACE标准库byte→Str车道解锁与延后面交付
author: [ZCode]
created_at: 2026-11-16
updated_at: 2026-11-16
plan_revision: 1
current_step: 0
total_steps: 7
depends_on: [PLAN-973, PLAN-972]
supersedes_spec_components: []
new_spec_components: [docs/specs/ace-migration/m2-stdlib-io.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace]
---

# [PLAN-1064] ACE标准库byte→Str车道解锁与延后面交付

## 0. 变更摘要

执行用户2026-11-16裁定（[PLAN-973](973-ace-m2-stdlib-io.md) §10.1 选项 c）：973 卡在**字节→Str 构造车道缺口**（972 已知边界）的库面**具名延期到本计划**，973 按已交付面收口；本计划承接延后面并在车道解锁后交付。本计划当前 drafting、0/7，仅规划承接合同，未实施、未建树、未运行测试。

车道解锁落点（972 域修订 vs 本包内建）是 T-01 有界决定，须用户确认后才能进 executing；本计划不是 973 的反向依赖，973 不等待本计划。Linux 相关验证按 2026-11-14 裁定继续移交 [1063](1063-ace-stdlib-linux-deferred-validation.md)，本计划只在 Windows 关闭当前门。

## 1. 目标

1. 解锁 packages-aggregates 面的**字节→Str 构造**车道（972 归档对账表的具名结构边界），落点由 T-01 冻结。
2. 交付 973 移交的延后面（见 §5 表），保旧公开 API sentinel 合同逐面不变；无旧同形的新面（KV storage）按 Design55/56 冻结独立合同。
3. 全部新面按 973 已冻结的六路径要求（AV/native、AV×C、AV×Rust、AC、A2R、A2PY＋原 A2C 适用域）真实验证。

非目标：Linux 实跑（归 1063）、972 已交付面的重做、重建平行装配/车道门（复用 973 已交付底座）、本计划之外的语义扩面。

## 2. 架构方案

复用 973 已交付底座：50 冻结平台操作×4 表、stdlib/ace/sys 装配集与 1043 来源装配、车道门（resolve/verify/lowering/abc/host/mir_verify）。byte→Str 构造只新增一条共享 Auto 算法车道（预期落 std.syscommon 或 972 域 std 面，T-01 定），各延后面在其上组合，不改平台操作冻结形状（i32→i32、lo+hi、UTF-16/bytes 双协议）。数据流不变：公共 .at → 装配 → 绑定/HIR/MIR → 六路径 → 真实 OS 可观察输出。

## 3. 技术栈

现有 ACE workspace、Auto 公共源、TestCase/固定组。唯一拟实施树 `D:/autostack/.wt/lang-1064/auto-lang`（分支 `plan-1064-dev`，基面=973 合入后的 master），尚未创建；材料 scope `D:/autostack/plan-artifacts/plan-1064/`，开工登记。不创建 junction/symlink。

## 4. 需求分析与背景调查

- 来源裁定：973 r11 §10.1（2026-11-16 用户选项 c）；972 归档 §5 对账表把 hex/base64 **解码**列为同车道 (b) 项——车道解锁后可同批交付，是否并入由 T-01 决定，不静默吸收。
- 973 代码基线 `a9eceef38` 的模块头具名注记（延后面与代码锚点，写入 §5 表；开工重核 SHA）。
- 旧 zip 公开 API：[stdlib/auto/zip.at](../../stdlib/auto/zip.at)——`list_json(path) str` / `read_text(path,name,max_bytes) str` / `extract_entry(path,name,dest,max_bytes) int`（读面返回 Str，故与 read_text 同车道）。
- KV storage 无旧同形公开 API（stdlib/auto/storage.at 是集合存储后端，非 KV）——按 973 T-01 冻结记录"纯 Auto 组合"，合同由本计划 T-01 冻结并标"无旧同形"。
- 旧 read_text/env.get sentinel：read_text 出错 `""`、env.get 未设 `""`（973 T-01 冻结的逐面 sentinel 约定）。

## 5. 详细设计

### 973 延后面移交表（owner=1064；代码锚点为 973 基线 a9eceef38 快照）

| # | 面 | 代码锚点（stdlib/ace/sys/） | 旧公开合同 | 973 状态 |
|---|---|---|---|---|
| D-01 | `file.read_text` | file.at:162-168（注记，未实现） | 出错 `""`（旧 sentinel） | 摘除 |
| D-02 | 目录名枚举（dir_names） | 无代码（path.at 仅纯字符串算法） | 由 T-01 冻结旧 stdlib 对应面 | 摘除 |
| D-03 | `env.get/get_or/home_dir/local_data_dir` | env.at:18-21 | 未设 `""` | 摘除（set/remove 已交付） |
| D-04 | `path.canonical` 真实符号链接解析 | 无代码（Windows 侧；Linux 语法归一 v1 缺口归 1063 L-02） | 由 T-01 冻结 | 摘除 |
| D-05 | `process.args()/current_dir()` | process.at:40-44 | 旧 args/cwd Str 面 | 摘除 |
| D-06 | `spawn_with_output` 捕获文本（stdout/stderr Str） | process.at:139-141 | JSON envelope `{"exit_code","stdout","stderr"}` | 计数/上限已交付，文本延后 |
| D-07 | KV storage 持久面（get/set/remove 行重建） | storage.at:1-8（escape/unescape 已导出待用） | 无旧同形；Design55/56 独立合同 T-01 冻结 | 未实现 |
| D-08 | zip/inflate（list_json/read_text/extract_entry） | 无代码 | stdlib/auto/zip.at 三元组 | 未实现 |
| D-09 | `file.read_bytes` Option\<List\> 形状 | file.at:130-133 | 现 List\<int\>+`read_bytes_len==-1` 替代 | A2R collections lowering 具名缺口，随车道一并裁定 |

### 车道解锁落点（T-01 有界决定，须用户确认）

选项：a) 972 域 owner 修订（std 面加 byte→Str 构造 builtin/库函数，语义归 972 域）；b) 本包在 std.syscommon 内实现共享构造算法（语义归本包，972 (b) 解码项可引用）。两选项都保旧 sentinel 与长度字符串 ABI 边界；决定记入本 Plan 后进 executing。

### 规范增量（待实际交付后沉淀）

| delta_id | add/modify | target | before/after | AC |
|---|---|---|---|---|
| SD-BSTR-01 | modify | docs/specs/ace-migration/m2-stdlib-io.md（973 merge 先建） | 延后面=具名缺口→已验证车道+逐面交付与独立 oracle | AC-02..05 |
| SD-BSTR-02 | add | 同上（车道节） | 无 byte→Str 构造→已验车道语义/边界/预算 | AC-02 |

## 6. 测试设计

| Case | AC | 输入与独立判据 | 入口 |
|---|---|---|---|
| V-01 | AC-02 | 车道构造：A中😀\0=9B 字节流→Str 往返逐码位相等；非法 UTF-8 序列拒绝分态；1 MiB 预算边界 | sys/973 既有 TestCase 车道，新例 ≤10s |
| V-02 | AC-03 | D-01..D-06 逐面旧 sentinel/独立 expected（read_text 出错 ""、env 未设 ""、捕获文本=CRLF 计数基线、cwd 往返） | 同上 |
| V-03 | AC-04 | zip 三元组对独立 oracle（Python zipfile 参照）；KV storage 行重建/转义往返、损坏行分态 | 同上 |
| V-04 | AC-05 | 六路径+原 A2C 适用域并集实跑（沿 973 §6 固定职责组选取），零选择/缺工具不充 PASS | `cargo t` 固定组 |

Linux 腿全部 NOT_RUN、owner=1063（沿 973 §10 Linux 移交表扩展行）；新例逐条计时 ≤10s。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 车道落点经用户确认；延后面逐条旧合同/来源/锚点/六路径映射冻结，无 unknown | T-01 |
| AC-02 | work | byte→Str 构造车道真跑绿，语义/边界/预算有独立 oracle 与负例 | T-02/V-01 |
| AC-03 | work | D-01..D-06 落面并保旧 sentinel，独立 expected 全绿 | T-03/V-02 |
| AC-04 | work | zip 三元组+KV storage 交付，独立 oracle 全绿 | T-04/V-03 |
| AC-05 | work | 冻结并集实跑，六路径覆盖，阻塞具名 | T-05/V-04 |
| AC-06 | review | 独立复审通过，绑定 revision/SHA | T-06 |
| AC-07 | merge | 规范增量落档、归档、清树 | T-07 |

## 8. 执行步骤

- [ ] **T-01｜work｜AC-01**：车道落点裁定冻结（§5 选项，须用户确认）＋延后面旧合同/锚点/映射冻结＋972 (b) 解码项是否并入决定。
- [ ] **T-02｜work｜AC-02**：实现 byte→Str 构造车道（含 D-09 Option\<List\> 形状裁定）＋V-01。
- [ ] **T-03｜work｜AC-03**：落 D-01..D-06 面＋V-02。
- [ ] **T-04｜work｜AC-04**：zip/inflate 纯 Auto 实现＋KV storage 持久面＋V-03。
- [ ] **T-05｜work｜AC-05**：冻结并集实跑＋V-04。
- [ ] **T-06｜review｜AC-06**：独立复审。
- [ ] **T-07｜merge｜AC-07**：沉淀、归档、清树。

## 9. 复审记录

2026-11-16 stage: new | 本计划由 PLAN-973 §10.1 用户裁定（选项 c）创建为延后面承接 owner；取号 1064（scripts/new-plan.sh，master 主检出）。仅规划承接合同：延后面移交表 9 项（D-01..D-09，代码锚点=973 基线 a9eceef38 快照）、车道落点二选项、V/AC/任务 7 项。未实施、未建 worktree、未运行任何测试、未改 canonical Specs；973 不反向依赖本计划。

## 10. 待澄清事项

1. **车道解锁落点**（阻塞进入 executing）：§5 选项 a（972 域 owner 修订）vs b（本包 std.syscommon 内建）。须用户确认；两选项均不改旧 sentinel 与 ABI 边界。
2. 972 归档对账表同车道 (b) 项（hex/base64 解码）是否随本包车道同批交付——T-01 一并裁定。
