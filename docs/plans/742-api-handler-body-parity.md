---
plan_id: 742
title: "api.rs handler body transpile parity（NOTES-001 T-04 跨仓）"
status: executing
author: [Codex]
created_at: 2026-10-04T21:20:00+08:00
plan_revision: 1
current_step: 0
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

- [ ] T-1: 失败测试钉三缺陷（api_gen tests，extract_api_lenient + generate_api_rs 路径）
- [ ] T-2: 修复 D1/D2（+D3 复现后）使测试绿
- [ ] T-3: 端到端——用本 worktree CLI 重建 015-notes 后端，T-04 adapter 编译通过 + v1 电池重跑

## 4. 验证

- cargo t api_gen（本 plan 测试）
- cargo tt（trans 档）
- 015-notes：auto run + v1 电池全绿 + legacy 冒烟（T-04 联动）

## 5. 已知延后

- D-742-1: `use api: Note` 于 db.at/repository.at 报 'Note' undefined
  （api.at 本地定义+构造正常）——跨模块导入类型构造解析缺陷，单独计划处理；
  本计划以"映射留在 api.at"绕开。

## 6. 复审记录

- 未执行。
