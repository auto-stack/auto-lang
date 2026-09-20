---
plan_id: PLAN-664
status: execution_done
feature_name: jade-consumer-upstream-package（jade/auto-edit 消费侧上游提名单：menubar 快照回归修复 + strict 域词位勘定补全 + JSON/storage 值域 + P-15/P-16 报错化 + DEBTS 651 回填）
author: [zhaopuming（auto-down 会话代拟，待 auto-lang 侧会话实勘修订）]
created_at: 2026-09-20
updated_at: 2026-09-20
plan_revision: 2
current_step: 6
total_steps: 6

supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/design/menubar-snapshot.md（SD-01：menubar 快照可见性与开合持久性契约——U-1 根修契约化；overview.md 现状节同步登记）
touched_goals: []

affects: [auto-lang（vm/codegen、ui 渲染快照面、DEBTS.md、docs/specs 注记面）]
---

# [PLAN-664] jade-consumer-upstream-package——消费侧上游提名单（五件合包）

> 来源：jade-garden L3 收口（PLAN-080 delivered，差异表后置面）+ auto-edit
> bootstrap（PLAN-001/002 delivered）两线消费侧的上游欠账合单。2026-09-20
> 用户裁定结构与分立（"比较多 → 独立计划"）。⚠ 本草案由 auto-down 侧会话
> 代拟——auto-lang 侧接手时须按惯例**起草前实勘**修订（T-00 即勘定任务，
> 部分缺口疑似已随近期提交部分落地，见 §4 表）。
>
> 消费方契约锚：jade 差异表（auto-down docs/plans/attachments/
> 079-l3-assembly-mode.md §8，D-05/D-11/D-12 等）+ auto-edit 041 矩阵
> 六失败实录（auto-down 会话四次 + 本包起草侧两次，39/5↔39/6 波动）。

## 0. 变更摘要

| # | 件 | 性质 | 消费方痛点 |
| --- | --- | --- | --- |
| U-1 | menubar 展开项快照回归修复 | **bug**（渲染器/vtree） | 041 矩阵六失败（MENUBAR_OPEN 条件分支不进 vm 快照遍历）；jade desktop menubar 同族 exe 可见=回归面特定于条件分支形态 |
| U-2 | strict 域词位勘定+补全（typeof 对象接收者 / Array.isArray 全域） | 勘定+补全（**部分疑似已落地**） | jade properties 编辑面差异 D-11（077 §6.1 在案） |
| U-3 | JSON/storage 值域勘定（json.encode/decode DSL 可用性 + storage.get str 域） | 勘定+收尾 | auto-edit graph settings 持久化差异 D-12；Plan 401 session KV str 域读回 |
| U-4 | P-15/P-16 静默死面报错化 | 健壮性 | P-15 跨项目 fn 导入静默死（079 E-8）；P-16 dep/导入撞名 link 期 Undefined symbol 无编译期告警（080 实录） |
| U-5 | DEBTS.md 651 矩阵 5 条回填 | **文档欠账**（五次提醒） | 651 交付时承诺的回填未落（DEBTS.md 现零 651 行，2026-09-20 实勘） |

## 1. 目标

1. 041 矩阵回 ≈48/2 口径（menubar 项入快照——U-1）。
2. jade 差异表 strict 域/值域两类升级前置解除或明确收口路径（U-2/U-3）。
3. 静默死面两类（P-15/P-16）至少编译期告警化（U-4）。
4. DEBTS 651 五条落册（U-5）。

**非目标**：fcose 力导布局、局部图谱 BFS（消费侧形态裁定，非上游）；
池版本锁定协议（方向一附款，另单）；651 矩阵本身的度量扩展。

## 2. 架构方案（待 auto-lang 实勘修订）

- U-1：渲染侧 vtree 快照遍历对条件分支（`if` 于 view 块内）展开内容的
  收录语义——修复面候选 = vm snapshot walk / menubar Popover lowering
  交互；**jade 对照**（menubar-menu DSL 元素族直建可见）供定位。
- U-2/U-3：词位现状勘定表驱动（见 §4）——已落地者补 e2e/文档锚，
  未落地者补全（跨 a2r/VM/vue 三轨一致性口径）。
- U-4：P-15 = fn 导入解析失败应报错；P-16 = dep/导入名与符号表撞名应
  编译期诊断（两处皆现为运行期/静默死）。
- U-5：从 651 归档计划/evidence 提取五条入 DEBTS.md。

## 3. 技术栈

auto-lang crates/auto-lang/src/vm/{codegen,native_catalog}.rs + ui 渲染
快照面 + DEBTS.md + docs/specs 相关注记；测试 = 041 矩阵复跑（auto-edit
侧，消费方验收）+ auto-lang 内部套件（cargo tf/tv 按面）。

## 4. 需求分析与背景调查（2026-09-20 勘定表定稿——auto-lang 侧实勘）

**授权记录**：2026-09-20 用户裁定上游包独立成计划（与 bps 方向一分立）；
同日用户向 auto-lang 会话下达执行指令（"计划664 实施它"）。

| 勘定点 | 勘定结论（auto-lang master @3df7b21a2 + worktree 实跑） | 定稿处置 |
| --- | --- | --- |
| menubar 快照回归（U-1） | **根因 ≠ 快照遍历**（Popover `[anchor, content]` 双子收录 Plan 422 既有）。真因 = renderer.rs "任意非 `__` 前缀消息关菜单"判据把 DSL TimeSource 泵事件（`Tick`/timer 处理器名，天然无前缀）误判为用户交互——041 状态栏时钟每秒 `Tick` 在消费方拍快照前关掉刚开的菜单（插桩实证：toggle `next=Some("view")` 后所有重建 `open=None`；`[UI_EVENT]` 日志只有 Tick）。39/5↔39/6 波动=竞态；jade exe 可见=其无周期非内部消息 | T-01 修复：`is_timesource_event` 判据排除（commit 3e6cfb886） |
| Array.isArray（U-2） | **已落地**（PLAN-057 T6：native 1919 全域 shim——ListData 真/对象/字符串/标量假；engine 注册 + codegen 白名单） | T-02 语料锚 |
| typeof 对象接收者（U-2） | 无 typeof DSL 关键字（web 侧走 TS ext 桥）；`.type` 属性 PLAN-066 语义=仅原始接收者（对象走字段访问，刻意设计）。**值域分派组合词位可用**：`Json.type_of(JSON.stringify(v))`（stringify 值域 PLAN-057 T11 + type_of 串域 Plan 446 D1 双态） | T-02 语料锚（覆盖 jade properties 编辑面需求） |
| json encode/decode（U-3） | 值域 encode=`JSON.stringify`（VM 感知）；decode=`Json.parse`（Plan 446 D1 物化 `__json_object`/ListData）。注意 `Json.encode` rust_fn 为串域占位（JSON-escape 一个字符串）——jade 应用 stringify 非 encode | T-02 语料锚 |
| storage 读回链（U-3） | **已落地可组合**：`storage.set(k, JSON.stringify(v))` → `storage.get(k)` → `Json.parse` → 字段直读全链绿（080 期"无 parse 词位"注记过时） | T-02 语料锚 |
| P-15（U-4） | 静默点 = lib.rs 根 use 环 + collect_module_imports 传递环的 `UseModuleResolution::None => continue/{}"`（两处零诊断） | T-03 告警化 |
| P-16（U-4） | 撞名时零编译期告警（080 实录 link 期死）；root/child model 字段在 use 装载环可及 | T-03 告警化 |
| DEBTS 651（U-5） | **详文已在 KNOWN-DEBT-AND-RISKS.md**（2026-09-19 收尾登记 P651-D1..D5+D6/F1）；DEBTS.md 零行（消费方口径"五条入 DEBTS.md"未满足） | T-04 镜像五行 |

**实勘修正**（相对草案）：U-1 根因改写（快照遍历猜测→Tick 泵自动关闭实证）；
U-2/U-3 全部收窄为锚（Q-2 预案生效）；U-5 收窄为镜像（详文权威在
KNOWN-DEBT，双账本互指）。vue 轨 U-2/U-3 = JS 透传同语义（jade web 侧
TS ext 已在产消费）；a2r 轨无此消费方 N/A。

## 5. 详细设计

### 规范增量

| delta_id | add/modify/retire | 目标文档 | before/after | rationale | acceptance |
| --- | --- | --- | --- | --- | --- |
| SD-01 | add（已落地） | docs/specs/auto-lang/ui/design/menubar-snapshot.md（+ overview.md 现状节登记） | 无 → menubar 快照可见性与开合持久性契约：Popover 子树全量进快照 / 开合=渲染器本地态 / 自动关闭判据排除 TimeSource 泵 / MCP press 带参可达 | U-1 契约化（实勘修正：非"条件分支快照收录"而是开合持久性） | AC-01 |

（U-2/U-3 词位若勘定需补 → 追加行；U-5 为 DEBTS.md 非规范增量。）

## 6. 测试设计

- U-1：auto-edit 041 矩阵复跑（消费方）≈48/2；auto-lang 侧最小复现件。
- U-2/U-3：三轨一致探针（jade gallery twin 形态可复用）。
- U-4：报错化负例（导入死/撞名 → 编译期诊断）。

## 7. 验收标准

| ID | 可观察行为 | 验证方法 | 结果 |
| --- | --- | --- | --- |
| AC-01 | 041 矩阵 menubar 六项全过（≈48/2 口径） | auto-edit 侧 desktop_mcp.py 复跑 | ✅ 49/1、47/1 两轮（超额过口径；残余单失败每轮不同=预存时序抖动） |
| AC-02 | strict 域/值域勘定表定稿：各缺口"已落地（锚）/补全（提交）"二态明确 | 勘定表 + 补全面套件绿 | ✅ §4 定稿（全部"已落地（锚）"）；语料 test_99_plan664 绿 |
| AC-03 | P-15/P-16 负例得编译期诊断（非静默/运行期死） | 负例探针 | ✅ plan664_p15/p16 双绿（[AUTO-USE-DIAG]） |
| AC-04 | DEBTS.md 含 651 五条 | 文件核对 | ✅ 五行在册（P651-D1..D5 镜像） |

## 8. 执行步骤（auto-lang 侧接手后按实勘修订）

- **T-00** [调查] 五件勘定（§4 表逐项实勘 + 勘定表定稿）。→ AC-02
  [✅ 已完成：§4 勘定表定稿（plan_revision 2）。关键实勘：U-1 根因改写
  （Tick 泵自动关闭，插桩实证链见 §4）；U-2/U-3 全已落地收窄为锚；
  U-5 收窄为 DEBTS 镜像（详文在 KNOWN-DEBT P651-D1..D5）]
- **T-01** [改] U-1 menubar 快照回归修复 + 最小复现件。依赖：T-00。
  [✅ 已完成：worktree commit 3e6cfb886。修复 =
  `DynamicComponent::is_timesource_event`（dynamic.rs，timesources 声明
  全集事件名匹配）+ renderer.rs 自动关闭判据排除。复现件三件：
  probe_menubar.py（机制级五段定位）+ 单测
  plan664_timesource_event_name_*（泵名/用户名/内部名三态）+ 041 矩阵
  e2e（39/6→49/1、47/1 两轮 menubar 六项稳定过；残余单失败每轮不同
  =预存 render 时序抖动族，T1 NOTE 在案类）]
- **T-02** [改] U-2/U-3 勘定后补全面。依赖：T-00。
  [✅ 已完成（收窄为锚，Q-2 预案）：worktree commit ed439e3ea。语料
  test/vm/99_plan664/001_value_lexemes（10 断言：isArray 四态/type_of
  五型分派/stringify 形态/storage 读回链含 remove 清理）一次通过——
  勘定结论"全部已落地可组合"得证；vm_file_tests 登记
  test_99_plan664_001_value_lexemes]
- **T-03** [改] U-4 报错化 + 负例。依赖：T-00。
  [✅ 已完成：worktree commit 51c58b7b3。lib.rs use 装载环三处
  `[AUTO-USE-DIAG]` 告警（P-15 根环+传递环 / P-16 root+child model
  字段撞名预检）+ `take_use_diags` thread_local 汇聚（单测断言面）；
  负例 plan664_p15/p16 双绿。硬错化留债（vue 双轨容忍性未勘，见 §9）；
  plan632 F2 复跑红=master 预存（主检出同红，独立 finding）]
- **T-04** [文] U-5 DEBTS 回填 + SD-01。依赖：T-00。
  [✅ 已完成：worktree commit 804ed4e94。DEBTS.md 651 五行镜像（指向
  KNOWN-DEBT P651-D1..D5 权威详文）；SD-01 =
  docs/specs/auto-lang/ui/design/menubar-snapshot.md + overview.md
  现状（2026-09-20）节]
- **T-05** [验证] 消费方联测（041 矩阵 + jade 侧门抽样）。依赖：T-01..04。
  [✅ 已完成：①041 矩阵（worktree 二进制）39/6→49/1、47/1 两轮；②jade
  desktop 真实消费方抽样 probe_jade_menubar.py（commit 896013e29）：文件/
  视图菜单打开后 `__menubar_item` 项入快照双 PASS（实测项含 重载文件
  列表 F5/命令面板 Ctrl+P/快速切换）；③全量门：cargo tv 全量
  3793/3795 + cargo tf 3631/3633，唯二失败
  mouse_area_emits_events_and_logical_extent /
  test_autodown_panel_heading_codegen 均 master 预存红（主检出击穿复现，
  与本计划改动面零关联）]

## 9. 复审记录

- 2026-09-20 draft handoff（auto-down 侧代拟）：`stage: new |
  plan_id: PLAN-664 | plan_revision: 1 | outcome: pass（草案就绪；
  auto-lang 侧实勘修订后生效） | next: auto-lang 会话接手实勘 + 用户
  授权执行`。
- 2026-09-20 work 完成交接：`stage: work | plan_id: PLAN-664 |
  plan_revision: 2 | outcome: pass（T-00..T-05 全任务完成，六件证据在
  §8） | code_commit: auto-lang worktree plan-664-dev =
  3e6cfb886（U-1 根修）+ ed439e3ea（U-2/U-3 语料锚）+ 51c58b7b3
  （U-4 告警化）+ 804ed4e94（U-5+SD-01）+ 896013e29（T-05 探针），
  base 3df7b21a2（master）| dependency_revisions: 组内 auto-down
  worktree @d1a83b6（master 同步依赖位，零改动） | task_ids:
  T-00,T-01,T-02,T-03,T-04,T-05 | evidence: 041 矩阵 39/6→49/1、47/1
  两轮（menubar 六项稳定）；jade desktop 抽样双 PASS；cargo tv 全量
  3793/3795 + tf 3631/3633（唯二失败 master 预存，主检出击穿复现）；
  p664 三组新测全绿（timesource 判定/词位语料/P-15P-16 负例） |
  findings: ①预存红三处 master 在案（f2_embedded_dep_component_
  expands_with_props[plan632]、mouse_area_emits_events_and_logical_
  extent、test_autodown_panel_heading_codegen[ui_gen codegen fixture
  族，与 KNOWN-DEBT 649 行同族]）——非本计划引入，独立 finding 待
  归属；②执行期事故一次：git stash pop 误弹他方共享 stash（plan-637
  026-final）入 worktree——即时外科复位（026-database 路径 reset+
  checkout），stash 条目因冲突保留零丢失，复盘教训=stash 共享栈
  禁用须用 patch 文件法（已在会话遵守） | blockers: 无 | next:
  /auto-plan:review（worktree 留存 D:/autostack/.wt/lang-664/）`。
  终验补记：全提交二进制（3e6cf..8960 重建）041 矩阵 50/0 满分——
  U-4 诊断对 041 编译路径零扰动（无 [AUTO-USE-DIAG] 噪声、无行为变化）。

## 10. 待澄清事项

| # | 事项 | owner | 处置 |
| --- | --- | --- | --- |
| Q-1 | 编号占位 664（.next-id 未见于仓根，663 在途）——接手时重扫两目录定号 | auto-lang 侧 | **✅ 已销号**：`.next-id`=664，663 为另一在途计划（vm-viewport-boundary），编号正确 |
| Q-2 | U-2/U-3 若勘定"已落地"，对应件收窄为文档锚——计划有界修订 | auto-lang 侧 | **✅ 已销号**：勘定全部已落地（§4 表），T-02 收窄为语料锚（plan_revision 2 有界修订） |
| Q-3 | （新增）P-15/P-16 硬错化升级——现落地为编译期告警（[AUTO-USE-DIAG]），硬错须先确认无合法未解析/撞名形态（vue 双轨应用容忍性未勘） | 后续计划/用户 | 告警化已满足 AC-03"编译期诊断"口径；升级硬错另议 |
