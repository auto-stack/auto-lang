---
plan_id: PLAN-648
status: drafting
feature_name: 跑法/门禁/codegen 缺陷簇整备——dev 跑法 VM api 委托断供 + tf 门 E0433 谱系 + codegen 遗留面清理
author: [zhaopuming/zcode-session]
created_at: 2026-09-18T00:00:00Z
updated_at: 2026-09-18T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 6
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-648 · 跑法/门禁/codegen 缺陷簇整备

## 0. 变更摘要

来源 = PLAN-021(auto-term,r3 归档件)T-10 移交 + 排障发现的
auto-lang 侧缺陷簇(证据:auto-term evidence/021/
vm-delegation-break.log 四轮记录):

1. **T-10 dev 跑法 VM api 委托断供**:`auto run -r vm`(app:api
   "rust")前台 tick 照跑、api.* 静默零到达——分体 HTTP 面前台轮询
   连接存在但数据不渲染;AUTO_FFI_TRACE 环境敏感性(trace 开 =
   前台连接归零)与"半屏"现象未归因。
2. **tf 门禁 E0433 谱系**:plan024_named_view_tests.rs:266
   `use crate::ui::style::Style;` 未门控(`ui` feature configured
   out)→ tf/nextest 全特性组合编译失败,全量门不可用。019 复审
   "AC-07 partial(tv 门禁外部 E0433 移交)"同谱系在案。
3. **codegen 遗留面**:merged 生成器 CRUD 原型(API_DATA 存储)与
   auto-term 声明式契约语义不符——PLAN-021 T-09 已令 rust 轨优先
   走 db.at 进程内吸收,CRUD 路径的存留价值/退役方式待清。

## 1. 目标

- G1 `auto run -r vm`(api:rust)api 委托链修复:前台 api.* 到达
  back 且数据渲染(app 实机内容流);AUTO_FFI_TRACE 敏感性归因。
- G2 tf 全量门可用:plan024 E0433 修复后 tf 全量跑通,红集对基线
  (cargo t 41 红/639 谱系 40 红)定界归档为已知基线。
- G3 codegen 遗留面收口:merged CRUD 原型路径退役或明示保留契约,
  生成器行为有测试锁定。

### 非目标

- 官方滚动条/虚拟滚动(auto-term PLAN-022 承接);
- auto-term 侧 app 功能变化;
- Blueprint/640/643 等并行计划面。

## 2. 架构方案

T-00 bounded investigation(决策工件):dev 跑法 VM api 派发链
仪器化定位(已知断面:api:rust 强制 AUTO_VM_MERGE=0 → split-HTTP;
前台轮询存在但数据不渲染;trace 环境敏感)→ 根修落点定界(auto
CLI runner / vm_bridge / 生成面)。tf 门禁 = 测试文件 feature 门控
规范修复(镜像既有 gated 测试文件惯例)。codegen = 收口性清理。

## 3. 技术栈

- auto-lang(本仓):crates/auto(CLI)、crates/auto-lang
  (vm_bridge/lib/api_gen/rust_ui)、crates/auto-man;
- 验证载体:auto-term app(rust-workspace 从零构建配方,PLAN-021
  T-11 target-p2 配方)。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户指令 2026-09-18(021 merge 后):"给后续计划立项"——本计划
  承接 021 T-10 移交与排障发现;预算/自动续跑:未指定。

### 4.2 已有证据(全部在库)

- auto-term evidence/021/vm-delegation-break.log:四轮排查记录
  (症状/A-B 双 exe/逐变量排除/env 敏感性/轮询连接观测);
- auto-term evidence/021/t-nff-*.log:折叠前后全量门红集(41 红)
  与 tf E0433 失败日志;
- 019 归档件:"tv 门禁外部 E0433 移交 019/020 lineage" 在案。

### 4.3 代码事实(读码 2026-09-18)

- auto/src/main.rs:985-1002:api:rust 强制 AUTO_VM_MERGE=0(设计:
  "Rust backend runs in a separate process, so VM+Rust is always
  split mode");
- rust_ui.rs generate_api_client:merged(吸收)/split(HTTP)双
  路径;PLAN-021 T-09 后 rust 轨前台优先吸收;
- plan024_named_view_tests.rs:266:`use crate::ui::style::Style;`
  无 feature 门控;lib.rs:7054 `#[cfg(feature = "ui")]`;
- auto-term 侧 app-back.exe 分体链路本身健康(curl 全通)。

## 5. 详细设计

| # | 改动 | 文件:符号(仓) | 说明 |
|---|---|---|---|
| D1 | T-10 定因调查 | auto CLI runner + vm_bridge(仪器化) | 决策工件:断点层级定界 |
| D2 | T-10 根修 | 视 D1 定界 | 委托链修复 |
| D3 | tf 门禁修复 | plan024_named_view_tests.rs(feature 门控) | tf 全量可编译 |
| D4 | codegen 遗留面收口 | auto-man merged/split 生成器 | CRUD 原型退役或明示保留 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| (无) | — | — | 本计划为缺陷整备,不改既有已发布契约面;门禁/跑法属仓库内行为 | 修复不引入新对外契约;如 D1/D2 定界发现需契约化再补 delta | — |

## 6. 测试设计

1. T-10:app 实机验证(rust-workspace 从零构建 + `auto run -r vm`
   内容流/Tab 标签渲染)+ 生成面单测锁定;
2. tf 门:修复后 tf 全量跑通,红集清单归档为已知基线(对 cargo t
   41 红定界);
3. auto-man 套件绿(PLAN-021 复审轮 297 过/2 环境性失败的 vue.rs
   项随本计划环境整备一并处理或显式豁免)。

## 7. 验收标准

- **AC-01** T-10 定因:决策工件在案(断点层级 + env 敏感性归因)。
  验证:决策工件。
- **AC-02** T-10 根修:`auto run -r vm` app 实机 Tab 标签渲染 +
  终端内容流(T-07 载体同判据)。验证:实机 + 日志。
- **AC-03** tf 门修复:plan024 门控修复后 tf 全量编译通过并可跑。
  验证:tf 全量日志。
- **AC-04** 红集归档:tf/cargo t 红集清单定界(基线 vs 新增)并
  落档 KNOWN-DEBT 或等效。验证:清单在库。
- **AC-05** codegen 遗留面收口:CRUD 原型退役或保留契约明示 +
  测试锁定。验证:单测/文档。

## 8. 执行步骤

- **T-00 [D1] T-10 定因调查**(bounded investigation,决策工件)。
  前置:无。关联 AC-01。
- **T-01 [D2] T-10 根修**。前置 T-00。关联 AC-02。
- **T-02 [D3] tf 门禁修复**(plan024 门控;镜像既有 gated 惯例)。
  前置:无(与 T-00 并行)。关联 AC-03。
- **T-03 [D4] codegen 遗留面收口**(CRUD 原型退役评估/清理)。
  前置:无(与 T-00/T-02 并行)。关联 AC-05。
- **T-04 全量门归档**(tf/cargo t 红集清单定界落档)。前置 T-02。
  关联 AC-04。
- **T-05 app 实机验证 + 收口**。前置 T-01+T-02+T-03。关联
  AC-02/03/04/05。

## 9. 复审记录

- 2026-09-18 draft handoff:`stage: new`,PLAN-648 rev1。`outcome:
  pass`(起草授权 = 021 T-10 移交 + 排障发现承接;T-00/T-02/T-03
  为首批可执行项)。`next: work`。注:本仓 643/645-647 有并行会话
  活动,执行时与 INDEX/.next-id 复核冲突面。

## 10. 待澄清事项

1. T-10 的 AUTO_FFI_TRACE 敏感性与"轮询但数据不渲染"的第三断点
   归因(D1 输出);
2. vue.rs 环境性失败(auto-os mirror 缺失)是否随本计划环境整备
   一并处理,或显式豁免(T-03 顺带评估);
3. 与并行会话(643/645-647 谱系)的文件冲突面协调。
