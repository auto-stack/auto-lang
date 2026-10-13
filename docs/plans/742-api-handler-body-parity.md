---
plan_id: 742
title: "api.rs handler body transpile parity（NOTES-001 T-04 跨仓）"
status: execution_done
author: [Codex]
created_at: 2026-10-04T21:20:00+08:00
plan_revision: 1
current_step: 3
total_steps: 3
base_branch: master
base_commit: a6c0f9691
depends_on: []
---

# 742：api.rs handler body transpile parity

## 0. 背景与授权

auto-os/apps/015-notes 的 NOTES-001 T-04（legacy CRUD → repository adapter）把
多语句业务体写进 api.at 端点后，api_gen 的 handler 体发射产出不可编译 Rust。
用户已批准治本（方案1）。来源实录见
auto-os/apps/015-notes/docs/plans/001-durable-inbox.md §9 work 记录 3/4。

## 1. 缺陷清单（三例均有 Notes 实际编译错误佐证）

- D1 List 构造原样发射：`var out List<Note> = List<Note>.new([])` →
  `let mut out: Vec<Note> = List<Note>.new(vec![]);`（类型转换了、值表达式没走
  构造降级；repository.rs 同构语句产出 `vec![]` 正常——api.rs 体路径状态缺失）。
- D2 体引用的 crate 模块缺 `use crate::repository;` 导入（api.rs 头固定
  axum/types/std 三行，不收集 api.at 的 use 语句）。
- D3 `==` 链式比较发射（实错行待复现后补录）。

## 2. 方案

try_transpile_body 已走 RustTrans::stmt 全量发射器；差异在初始化状态与
api_gen 的导入收集。修复点：

- T1（D1）：api.rs 体路径的 List 构造降级——按 repository.rs 模块路径的对齐
  方式补齐 RustTrans 初始化（register_type 字段类型/所需集合），或在该路径
  显式处理 `List<T>.new(args)`；以失败测试锁行为。
- T2（D2）：generate_api_rs 收集体/来源文本中的 crate 模块引用并发射
  `use crate::<mod>;`（api.rs 头部）。
- T3（D3）：复现后修对应发射（comparison 链）。

## 3. 任务

- [x] T-1: 失败测试钉三缺陷（api_gen tests，extract_api_lenient + generate_api_rs 路径）
  —— `test_handler_body_module_calls_and_list_construct`（crates/auto-man/src/api_gen.rs:5449，
  提交 152c832bc）：走完整 generate_api 流水线，断言 `use crate::repository;` 注入（D2）、
  List 构造降级（D1）、无链式比较（D3）；c5d54f125 扩展 NoteView 非主键返回走 a2r
  wrap 路径（D5 面）。
- [x] T-2: 修复 D1/D2（+D3 复现后）使测试绿——实际执行扩展至 D1~D9（提交账：
  D1/D2/D4+borrow 泛化=152c832bc；D5/D6=c5d54f125；companion 先序+crate 限定路径=
  bd1ae6ec9；D7 SqliteDb clone 事务可见性=d98517f0c；(fs,mkdir_all) 分派=005d148a0；
  D8 store dark_mode 种子=9f6481e8c；D9 fs.dir_names=14e642773+67d655f76 改名）。
  D3 由 D1 路径的发射正常化覆盖（测试断言"无链式比较"残留）。
- [x] T-3: 端到端——015-notes 经 NOTES-001 r3/r4 迭代交付（D8/D9 即 r3/r4 实测中
  发现修复）；auto-os c4c848d 记录 "NOTES-001 r4 delivered/archived; battery S0-S14
  + Vue 16/16 + VM 19/19"，子模块 gitlink 现为 1391bd7。

## 4. 验证

- cargo t api_gen（本 plan 测试）
- cargo tt（trans 档）
- 015-notes：auto run + v1 电池全绿 + legacy 冒烟（T-04 联动）

执行时点验证（2026-10-04~06，提交内记录）：c5d54f125 记 tt 29 红对基线 33（净 -4，
新增红隔离复跑均绿=并行 flake）；015-notes r4 电池全绿交付。

追认复核（2026-10-13，主检出 f799be601）：`cargo test -p auto-man --lib api_gen`
45/45 绿（含钉行为测试）；`cargo tt` 5649/5649 绿（1 slow）；批量回归收据
（.last-batch-regression.json，覆盖至 PLAN-766，2026-10-12）tt 腿 5629/5629 绿、
new_reds 空——742 代码合入后至今无回归归因于本计划。

## 5. 已知延后

- D-742-1: `use api: Note` 于 db.at/repository.at 报 'Note' undefined
  （api.at 本地定义+构造正常）——跨模块导入类型构造解析缺陷，单独计划处理；
  本计划以"映射留在 api.at"绕开。

## 6. 合并血缘与追认说明

实现于 2026-10-04~06 在 lang-742 worktree（已退役）`plan-742-dev` 分支完成，
a6e108f60 合入 v0.6-dev，再经 PLAN-775 收敛（774951052 integrate v0.6-dev on v0.5
main）进入 master；知识沉淀 e35f9e99b（auto-man api-generation-integrity）。
PLAN-745 交接注记（0bc1bea01）确认 742 合并面对其 27 处共现预存红零贡献。
计划簿记当时未随合并回写，2026-10-13 本记录为对账追认（reconcile）：代码/测试/
端到端证据已在 master 逐项核验（见 §3/§4），非重新执行。

## 7. 复审记录

- 2026-10-13 work 追认对账（stage: work | 742 | rev 1 | pass | worktree 已退役
  （lang-742，历史 plan-742-dev）| base a6c0f9691 | code 至 67d655f76，现随
  v0.6-dev 收敛在 master f799be601 | dep: a2r-std 同分支面 | T-1/T-2/T-3 全勾
  | 证据：§3/§4（钉行为测试 api_gen.rs:5449 绿、tt 5649/5649、auto-os c4c848d
  r4 电池交付记录、批量回回收据覆盖）| blockers: 无 | next: review（/auto-plan:review
  按 §3 任务清点 + §4 验证面独立复核；已知延后仅 D-742-1）
