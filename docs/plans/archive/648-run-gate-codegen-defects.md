---
plan_id: PLAN-648
status: archived
feature_name: 跑法/门禁/codegen 缺陷簇整备——dev 跑法 VM api 委托断供 + tf 门 E0433 谱系 + codegen 遗留面清理
author: [zhaopuming/zcode-session]
created_at: 2026-09-18T00:00:00Z
updated_at: 2026-09-19T00:00:00Z
plan_revision: 1
current_step: 6
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
  [✅ 2026-09-19] 决策工件 `docs/plans/evidence/648/t00-decision.md`:
  主断点=生成面裸名-only api 改写(两合成入口同病);限定调用回落
  back 链进程内执行+宿主引擎 DLL 缺席降级→全空;"第三断点"不存在
  (021 "3 连接=轮询"系误读);AUTO_FFI_TRACE 敏感性归 auto-term 侧
  仪器(与断供无因果)。次生发现:VM handler 再入缺陷(P648-D1,
  三次实证)。仪器:AUTO_LANG_HTTP_TRACE(simple_http_json 汇聚点)。
  复现/对照日志同目录 5 件。
- **T-01 [D2] T-10 根修**。前置 T-00。关联 AC-02。
  [✅ 2026-09-19] worktree 9ada0a21e:api_funcs 增限定名键(扁平化
  全名+别名末段.fn)+codegen 门卫放行 Dot(Ident,fn);db.* 不受
  影响;merged 路径零改动。plan340 12/12(含新增限定名回归测试);
  实机 `auto run -r vm` 70s 零交互 1063 次 api 轮询全 200,
  tab-title="shell 1"/pane-lines 真实 Windows banner,Tab "● shell 1"
  +提示符渲染(截图 evidence/648/t01-window-rendered.png)。
  注意:交互事件(restore/键入)触发既有再入缺陷 P648-D1——
  无交互判据已达成,交互稳定性独立立项。
- **T-02 [D3] tf 门禁修复**(plan024 门控;镜像既有 gated 惯例)。
  前置:无(与 T-00 并行)。关联 AC-03。
  [✅ 外部落地 2026-09-18 12:10] 并行会话 64b65529b ⑤已补
  plan024 dashboard 测试 ui feature 门("解锁 tv/tf",提交信息含
  tf 3611/3613 双红对照)。本计划不重复副作用;E0433 消除的编译
  实证并入 T-04 tf 全量门(见下)。
- **T-03 [D4] codegen 遗留面收口**(CRUD 原型退役评估/清理)。
  前置:无(与 T-00/T-02 并行)。关联 AC-05。
  [✅ 2026-09-19] 裁定=明示保留(非退役):API_DATA CRUD=demo 脚手架
  兜底,在库消费方 020-music-player/031-image-viewer(带 db.at 应用
  全走吸收臂)。worktree 6422eb267:契约注释+双锁定测试(吸收优先/
  CRUD 兜底形态)2/2 绿。vue.rs 环境性失败复核归 P028-D4/P645-D2
  预存红家族(master 同败实证),显式豁免不并入。
- **T-04 全量门归档**(tf/cargo t 红集清单定界落档)。前置 T-02。
  关联 AC-04。
  [✅ 2026-09-19] `evidence/648/t04-redset.md`:master 全量 tf 3 红
  (ffi_dual_019/mouse_area/display_family,并行 flaky 家族,单测隔离
  均过)⊇ worktree 红集(1-2 红),零新增红;E0433 消除实证=tf 全
  特性组合编译通过且 3641 全装载执行(AC-03 同时闭合);日常档
  musk 3 红=master 同败(P648-D2/P028-D4 家族)。
- **T-05 app 实机验证 + 收口**。前置 T-01+T-02+T-03。关联
  AC-02/03/04/05。
  [✅ 2026-09-19] 实机判据于 T-01 轮达成(截图+1063×200 轮询+真实
  内容流,AC-02);收口=作用域门禁全绿(plan340 12/12 含新增回归、
  merged_api_client 2/2、tf 全量零新增红)+ 债务登记(P648-D1/D2)
  + spec 契约复核(限定名改写与 545 bare=命名空间语义同向,零
  delta 必要)。

## 9. 复审记录

- 2026-09-18 draft handoff:`stage: new`,PLAN-648 rev1。`outcome:
  pass`(起草授权 = 021 T-10 移交 + 排障发现承接;T-00/T-02/T-03
  为首批可执行项)。`next: work`。注:本仓 643/645-647 有并行会话
  活动,执行时与 INDEX/.next-id 复核冲突面。
- 2026-09-19 work stage:work | PLAN-648 | rev1 | outcome:pass |
  code_commit:plan-648-dev 9ada0a21e(T-01 根修)+6422eb267(T-03
  锁定;worktree D:/autostack/.wt/lang-648/auto-lang,base=master
  4817b51e1;组内依赖 worktree .wt/lang-648/auto-down@b1c88de
  detached) | task_ids:T-00..T-05 全六项 | evidence:evidence/648/
  七件(t00-decision.md 决策工件/t00-repro1 修复前/verify4-5+final
  修复后与再入触发/window-rendered.png 渲染实证/t04-redset.md 红集
  定界);plan340 12/12(含限定名回归新增)、merged_api_client 2/2、
  tf 全量零新增红(master 3 闪红 ⊇ worktree 红集)、实机 1063×200
  轮询+Tab"● shell 1"+提示符渲染 | blockers:无 | next:review。
  执行期要点:①T-02 修复已由并行会话 64b65529b(09-18 12:10)落地,
  本计划核验而非重复(技能"未复检前不重复副作用"条);②次生发现
  VM handler 再入缺陷(交互触发失控,P648-D1 三次实证)按路由记债
  独立立项,不并入本计划范围——无交互判据(AC-02)已达成,交互
  稳定性为独立缺陷面。
- 2026-09-19 review stage:review | PLAN-648 | rev1 | outcome:**pass** |
  reviewed_commit:6422eb2677382da7873ad91452699a8a8946f453 |
  base_commit:4817b51e1e5268ae28fd9651f1b8aa1342d6ba86 | deps:
  .wt/lang-648/auto-down@b1c88def9(detached 只读 path 依赖) |
  spec_inputs:docs/specs/auto-lang/frontend/design/module-resolution.md
  (545 bare=命名空间语义;限定调用=规范形态) |
  acceptance_results:AC-01 pass(决策工件在库四节+佐证 7 件);
  AC-02 pass(独立复跑 40s/4083 响应全 200,tab-title="shell 1",
  pane-lines 真实 banner,循环零爬升;在库截图亲验 Tab "● shell 1"
  +提示符);AC-03 pass(tf 全特性编译通过+3641 全装载,E0433 消除;
  复用 6422eb267 同点工作期证据——代码/依赖/配置此后未变,本轮
  plan340/vm_file/merged 定向复跑均绿佐证);AC-04 pass(t04-redset
  在库,master 3 闪红⊇worktree,零新增红);AC-05 pass(契约注释+
  双锁定测试复跑 2/2) | findings:F-R1..R4 全 info 无阻塞(见下)
  | evidence:evidence/648/ 七件+本轮复跑(review-rerun 4083×200
  摘录于本记录)+门禁复跑(tv fail-fast 2727 绿仅 display_family
  闪红=已知家族;vm_file 47/47;plan340 12/12;merged 2/2;tt/tb
  不适用零 trans/book 改动;aavm 触发条件零命中) | next:merge。
  独立性声明:执行会话内复审——裁定自工件重建(diff 逐行审读/
  测试断言核验/实机独立复跑/截图亲验),未采信执行者摘要。
  F-R1[info] P648-D1 再入缺陷移交合规(KNOWN-DEBT+§10.4),AC-02
  字面判据独立复现,非静默延后;F-R2[info] host-bridge/no-op 臂仍
  裸名-only(ash-runner 形态限定调用不走 auto.host.call,G1 范围外
  对称性观察,候选后续 parity);F-R3[info] alias_form 键与同名局部
  变量方法调用的理论碰撞在模块别名遮蔽语义下不可达(1063+4083
  请求全 200 行为反证);F-R4[info] 复审期 no-fail-fast tv 全量因
  重档组限流+并行会话 cargo 竞争 45min 未收,以 fail-fast tv+
  vm_file 定向+tf 全量(更宽集完整跑通)组合替代,证据充分。
  规范增量冻结复述:无 add/modify/retire(§5 书面理由:缺陷整备
  不改已发布契约面;T-01 使 codegen 追齐 545 限定调用规范形态,
  与现行 spec 同向零冲突);frontmatter supersedes/new/touched=[]
  与之一致。

## 10. 待澄清事项

1. ~~T-10 的 AUTO_FFI_TRACE 敏感性与"轮询但数据不渲染"的第三断点
   归因(D1 输出)~~ 已闭(t00-decision.md §二:第三断点不存在,
   021 "3 连接=轮询"系误读;AUTO_FFI_TRACE=auto-term 侧仪器行为,
   与断供无因果,如仍复现归 auto-term 侧);
2. ~~vue.rs 环境性失败是否随本计划环境整备一并处理~~ 已闭
   (P028-D4/P645-D2 预存红家族,master 同败实证,显式豁免);
3. ~~与并行会话(643/645-647 谱系)的文件冲突面协调~~ 已闭
   (T-02 由 64b65529b 落地,本计划核验不重复;改动面 4+1 文件
   与并行计划零交集);
4. 新增移交:VM 事件 handler 再入缺陷(P648-D1,交互阻断级,
   三次实证)——dev 跑法 VM 轨实点交互前需先修,独立立项。

- 2026-09-19 merge stage(PLAN-648:r1 五 checkpoint 收据):
  **prepared**=规范增量冻结=无(复审 pass 书面理由在案);沉淀面=
  docs/specs/auto-lang/vm/plans.md+docs/specs/auto-man/plans.md 各增
  648 一句话沉淀行;账本 .autoos/specs.json upsert P648-1(reviews,
  复审与合并收据)/P648-2(reports,限定名改写变更摘要)回读验证
  (124/101 条);spec-index.py 再生 INDEX 字节同一(26 projects 零漂移)。
  事故与矫正:首枚账本提交误 `git add -A` 裹挟并行会话三未提交件
  (656 计划/015-notes main.rs/rust-workspace Cargo.toml)——软重置
  拆分重提,外来件已还原工作区待归属会话认领。
  **landed**=master 3d0d63345(--no-ff fold plan-648-dev;reviewed
  6422eb267 祖先校验过;main smoke plan340 12/12 含新增回归)。
  **ledger_refreshed**=.autoos/specs.json P648-1/P648-2(readback OK)。
  **archived**=git mv docs/plans/648-run-gate-codegen-defects.md →
  archive/ + status:archived;completion_kind:**delivered**。

## 11. spec-sync 回写记录

- vm/plans.md +648 行(限定名改写根修一句话沉淀);
- auto-man/plans.md +648 行(CRUD 契约明示保留+锁定测试);
- .autoos/specs.json:P648-1(reviews 收据)/P648-2(reports 变更摘要);
- 规范增量=无(canonical specs 零改动,理由见 §5 与 review 冻结复述);
- KNOWN-DEBT-AND-RISKS.md:P648-D1(handler 再入缺陷,交互阻断级,
  三次实证)/P648-D2(musk 预存红复核)——移交后续计划。
