---
plan_id: PLAN-783
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 781-audit-fixes
author: [ZCode]
created_at: 2026-10-15
updated_at: 2026-10-15

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/ui/design/scroll-child-anchor.md
new_spec_components: []
touched_goals: [GOAL-007, GOAL-014]

affects: [auto-lang/ui, auto-lang/vm]
depends_on: []
current_step: 0
total_steps: 5
---

# [PLAN-783] 781 审计缺口修复（F-781-R01..R04 承接）

## 变更摘要

承接 PLAN-781 历史审计（[reports/781-historical-review-20261015.md](reports/781-historical-review-20261015.md)，
needs_fix，781 保持归档终态不回退）的四项 findings：两项 P1 代码缺陷（原生锚点
合成 id 的冒号歧义拆分、Vue 目标查找无最近 pane 归属过滤）与两项 P2 验收深度
缺口（10MiB 全章一致性 oracle、V-18 生命周期断言覆盖）。一个修复计划、一个
专用 worktree、一次独立复审；781 的已交付事实与归档状态不变，本计划只修缺口
并补齐证据面。

## 目标

| Finding | 级别 | 结果 | 任务 |
|---|---|---|---|
| F-781-R01 | P1 | 标准 `@scrollctl:N` 句柄与含分隔符/引号 key 在原生锚点寻址中正确往返（无歧义编码 + 真实标准 controller 双端验收） | T-01 |
| F-781-R02 | P1 | Vue 目标查找按最近 pane 归属过滤（含无 controller 嵌套 pane）；managed 判据同归属规则；消除 selector 注入面 | T-02 |
| F-781-R03 | P2 | 10MiB 章节一致性具备可执行且必选的全量/等价完整 oracle（或经显式合同修订+用户确认后记录边界），含损坏反例 | T-03 |
| F-781-R04 | P2 | V-18 生命周期断言覆盖实际取消语义：handler 进入握手、取消后零写入、deadline/断连/服务关闭终态与回收 | T-04 |

非目标：重开或追改 PLAN-781 任何记录；Reader 业务侧改动（reading.at 接入、
Init 韧性=READER-001）；段泵串行公平性（F-781-C01，独立新计划域）；aavm/
trans/book 面；锚点 DSL/公共 API 形状变更（9922/9923 签名与回执语义零变化）。

## 架构方案

R01 采用**不透明序号合成 id + build 期注册表**：wrapper 构建期分配单调序号
`scrollanchor:{seq}`，注册表 `seq → (pane, key)` 同帧构建同帧消费（collector
只回传 rect+seq，drain 侧查表解析）——彻底消除分隔符/引号/反斜杠往返与
Debug 字符串取值两类面，比长度前缀编码更稳。R02 采用 **`data-scroll-pane`
所有权标记 + closest 归属过滤 + 全量遍历属性比对**（不拼 selector，天然免疫
注入）。两 P1 修复均保持既有手写无冒号句柄与既有 fixture 行为不变（回归面：
`cargo t scroll/plan781` + 双端探针全量重跑）。R03/R04 为验收深度补齐，
不改产品代码（除非 T-03 探针证明 chapter API 有框架侧缓存缺陷）。

## 需求分析与背景调查

审计反例探针（`docs/plans/attachments/781/historical-review/probe.cjs`，
exit 0）已确认三源缺口在位；`scroll_anchor.rs:29-36`（冒号拼接/首冒号拆分）
与 `vue.rs:3938-3942`（子树 querySelectorAll 取 last + managed 子树判据）
本仓复核一致。canonical 规范 `scroll-child-anchor.md` 的"最近 ordinary
scroll-pane 归属、不同 pane 同名不串线"是冻结合同，修复是对齐而非改约。
R04 的生命周期实现（scope/许可回收）有 plan705/707/730 既有证据族，缺口在
**LegacyHttp 长忙路径的具体断言映射**——T-04 以进入握手+副作用观测桥接，不
重造生命周期机制。R03 的成本瓶颈初判为 Reader chapter_json 每调用全量重解析
（toc 实测 167s/次），T-03 先做有界成本探针再裁全量 vs 显式修订。

## 详细设计

### T-01（R01）：不透明序号 id + 注册表

- `iced/scroll_anchor.rs`：`anchor_widget_id(seq)`；模块级 `ANCHOR_SEQ_REG:
  Mutex<HashMap<u32,(String,String)>>`（build 期 wrapper 写入，drain 消费后
  清理或整帧重建）；collector 上报 `(seq, rect)`；drain 解析 seq→(pane,key)
  后走既有 bounds 表。`parse_anchor_widget_id` 退役为仅认 seq（旧冒号格式
  不再产生；无持久化面，无兼容负担）。
- 序号分配与 wrapper 构建同帧；换帧重建时旧 seq 失效按 not 解析（防御：注册
  表 miss → 丢弃该上报，不 panic）。
- 双端验收：fixture 增 `scroll_controller()` 标准 controller + key 含
  `:`/引号/反斜杠的行；vm_probe + vue_probe 增对应断言（queued→定位→applied）。

### T-02（R02）：最近 pane 归属过滤

- `ui_gen/vue.rs`：plain 路径与 shadcn scroll 臂对所有 scroll-pane 元素发射
  `data-scroll-pane`（与 controller 无关——所有权标记独立于寻址标记）。
- helper `scroll_to_child`：候选 = `e.querySelectorAll('[data-scroll-anchor]')`
  过滤 `el.closest('[data-scroll-pane]') === e && el.getAttribute(...) === key`
  （无 selector 拼接；文档序最后一个胜出维持）；managed 判据 = 存在
  `[data-scroll-managed]` 且其 `closest('[data-scroll-pane]') === e`。
- 嵌套 pane 无 controller 时不发 `:data-scroll-ctl` 但必发 `data-scroll-pane`
  ——内层目标归属内层；outer 请求 inner-only key → not_found；inner managed
  不再误伤 outer。
- 验收：vue_probe 增嵌套同名/仅内层有 key/内层 managed 三对照；vm_probe 增
  嵌套 pane 用例（若 iced 原生嵌套寻址同样受影响则一并修复——以 T-02 实施
  期探针为准，反例留档）。

### T-03（R03）：全章 oracle 或显式合同修订

- 探针：chapter_json 单调用成本（隔离实例计时）；若 ≤3s/章（缓存或轻路径）
  → 10MiB 全量扫描（1347+1346 章，绑定输入 hash 与冷重启后全章复扫）。
- 若成本不可行：**等价完整 oracle** = 专用全形状多章书（TXT+MD 各 ~30 章，
  覆盖章头/空行/开篇/无章头/尾截断形态）做**全章**逐字验收 + 冷重启全章复扫；
  10MiB 档保持原件 hash + TOC 计数 + 抽样正文——此为**合同修订**，须在
  本计划记录并**执行前报告用户裁定**（P2 finding 明文禁止静默降级）。
- 负例：损坏一章期望体（翻转哈希）→ oracle 必须红，证明其有效性。

### T-04（R04）：生命周期断言补齐

- 测试 .at：`/api/slowwrite` handler 进入即写 `entered` 标记，循环内每拍向
  副作用文件追加心跳，长忙（预算宽裕）；`/api/ping` 轻路由。
- 用例（th 档，真 TCP）：① deadline：短 `AUTO_HTTP_REQUEST_TIMEOUT_MS` →
  503 后心跳文件在观测窗内停止增长 + 无完成标记；② 断连：等待 `entered`
  出现后 drop socket → 断连后心跳停止 + 无完成标记 + 后续 ping 200；③ 服务
  关闭：shutdown 期间在途请求 → 单失败终态（沿用既有 shutdown 测试面，补
  LegacyHttp 长忙映射）；④ 资源回收：in-process 计数面（沿 plan705/707
  既有 scope/许可观测，若可及）断言无泄漏。
- 断言绑定 LegacyHttp 长忙路径（预算 1G 档驱动），非旧短路径证据引用。

### 规范增量（候选）

| delta_id | op | target | 内容 |
|---|---|---|---|
| SD-A | modify | docs/specs/auto-lang/ui/design/scroll-child-anchor.md | R01/R02 语义对齐：不透明序号 id（实现层）、最近 pane 归属含无 controller 嵌套 pane、managed 同归属判据、key 字符集全量支持（含分隔符/引号） |
| SD-B | modify | docs/specs/auto-lang/vm/design/handler-execution-budget.md | （若 T-03 走修订路线）10MiB 验收 oracle 形状的合同化边界；V-18 oracle 形状（进入握手/零写入观测） |

## 测试设计

| Case | Finding | oracle |
|---|---|---|
| W-01 | R01 | 标准 `scroll_controller()` 句柄 + `:`/引号/反斜杠 key 双端 queued→定位→applied；旧无冒号句柄回归 |
| W-02 | R02 | 嵌套同名（outer 请求滚 outer 的目标、零串线）；仅内层有 key → outer not_found；inner managed + outer ordinary → outer 不误 unsupported；引号 key 注入反例（属性比对无 selector） |
| W-03 | R03 | 全章/等价完整 oracle + 损坏章负例（oracle 必红）+ TXT/MD/冷重启覆盖 |
| W-04 | R04 | deadline/断连/关闭三路：entered 握手 + 断后零写入 + 单终态 + 回收观测 |
| W-05 | 回归 | `cargo t scroll/plan781` 全绿 + 双端探针全量重跑（41+43+新增）+ th 781 族 |

门禁：开发期 scoped（`cargo t scroll|plan781`、`cargo tu`、触 http 面
`cargo th`）；最终复审通过后 merge 在冻结提交跑裸 `cargo t` + 触面档
（2026-10-14 时序裁定）。全部实机驱动用例走专用 fixture/隔离实例纪律。

## 验收标准

- [ ] AC-01: W-01/W-02 双端全绿——标准句柄与任意合法 key 字符往返正确；
      嵌套 pane 归属不串线、inner-only not_found、managed 判据不误伤；
      既有 fixture 回归零破坏。
- [ ] AC-02: W-03 oracle 落地且负例红——或（修订路线）用户显式裁定记录
      在案 + 规范同步，无静默降级。
- [ ] AC-03: W-04 三路生命周期断言绿，绑定 LegacyHttp 长忙路径；引用旧
      短路径证据的地方全部替换为具体映射。
- [ ] AC-04: 交接记录含四 finding 逐项闭合表（反例→不变量→修复→永久
      用例→结果），781 归档面零改动；SD-A/B 候选冻结。

## 执行步骤

- [ ] T-01: owner_stage=work；R01 修复（序号 id + 注册表 + 单测 + 双端
  标准 controller 验收 + 回归）。AC-01。
- [ ] T-02: owner_stage=work；依赖T-01；R02 修复（data-scroll-pane 发射 +
  归属过滤 helper + 三对照 + 逃逸面消除 + 原生嵌套探针）。AC-01。
- [ ] T-03: owner_stage=work；依赖无；R03 成本探针 → 全量或修订裁定（修订
  须报用户）→ oracle + 负例 + TXT/MD/冷启动覆盖。AC-02。
- [ ] T-04: owner_stage=work；依赖无；R04 生命周期测试 .at + th 三路断言
  + 回收观测。AC-03。
- [ ] T-05: owner_stage=work；依赖T-01..04；W-05 全量回归 + 交接记录 +
  SD 候选冻结 → execution_done（复审与 merge 另属其角色）。AC-04。

## 9. 复审记录

（工作交接与复审记录追加于此。）

## 10. 待澄清事项

T-03 若走合同修订路线（10MiB 全章扫描成本不可接受），修订内容须用户显式
裁定后方可执行——P2 finding 明文"不能静默降级"。其余无用户前置决策。
