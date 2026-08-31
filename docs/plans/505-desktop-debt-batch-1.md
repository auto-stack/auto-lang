---
plan_id: PLAN-505
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-debt-batch-1
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 9
---

# [PLAN-505] 桌面 DEBT 批处理一期——交互时序 + shell 面瑕疵 + 验收受阻族统一解

## 变更摘要

桌面特性线收官后的**债务集中清偿**（用户裁定的"桌面线完成后统一清"策略
兑现）。从 KNOWN-DEBT 及各归档计划登记中**裁剪**出四族可执行项，一次计划
批量清偿；环境依赖项（物理机复验）、决策项（真洞翻默认）、语言/VM 线缺陷
明确**排除在外**。

| 族 | 内容 | 债务号 | 价值 |
|---|---|---|---|
| **A 交互时序** | ① 事件泵 drain 修复（16ms 单发轮询 ≈62 事件/s → 每拍排空 + START/END 优先级出队，拖拽落位延迟秒级→帧级）② 快甩拖入首轮滞留 | 486 表格行（性能观感 🟡）/ P488-D3 | **最高**——直接影响 dock/拖放手感 |
| **B shell 面瑕疵** | ① shell.at 任务栏 top/bottom 双分支去重 ② pager 网格 ≤4 截断（计划定而未实现）③ window_thumbnail props 不透传 DOM ④ os-config daemon 发现序 PATH 级补全 ⑤ broker 停机旗标接生产调用点 | P487-3 / P497-1 / P497-2 / P501-1 / P480-R1 | 小而确定 |
| **C 实机验收受阻族** | "前台竞争/CUA 像素身份守卫阻断实机照"**一族统一解**：专用验收会话形态（守卫放行清单 / 无人值守窗口 / 驱动脚本统一入口），一次性解锁 P487-1、P496-1、P501-2 及 472/478/479 先例族的复验留痕 | P487-1 / P496-1 / P501-2（家族） | 方法论——终结"每计划一条受阻债"的滚雪球 |
| **D 增强小件** | ① on_dnd_finished 交付取完成时焦点 App ② 壁纸热切换（若证得"投影重注入+指纹天然支持"则顺路） | P488-D4 / 496 待澄清 | 低成本补全 |

**排除项**（登记但不入本期）：P494-2/-3 真洞物理机复验（环境依赖，等物理机
窗口）；真洞默认值翻转（决策项，待使用反馈）；P488-D1/-D2、P495-1/-2
（语言/VM 线缺陷与 tv 档卫生，归语言线专项）；P485-1（已有 TestLock 缓解，
维持观察）；P480-R2（口径文档项，随 500 复审顺带核对）；457 长线。

## 目标

- **G1 A 族**：拖拽松手到 dock 落位延迟从秒级降到帧级（drain-while-empty +
  优先级出队）；快甩拖入不再滞留首轮。
- **G2 B 族**：五项瑕疵逐项清偿，各自原有验证口径复跑绿。
- **G3 C 族**：形成可复用的实机验收通道（一份操作文档 + 一个放行机制），
  并用它把 P487-1/P496-1/P501-2 三条受阻债的实机照补齐留痕。
- **G4 D 族**：P488-D4 落地；壁纸热切换定案（做或明确不做+理由）。
- **G5 记账**：所有清偿项回写 KNOWN-DEBT 已清偿标记；排除项维持登记不丢失。

## 架构方案

无新架构——四族均为既有面上的修复/补全/流程件。A 族改
`ui/session.rs:1220` 段事件泵（drain + 分级出队）；C 族是**流程资产**（验收
通道文档 + 守卫放行配置），落在 `docs/` 与测试驱动脚本目录，复用
`.agents/skills/autoui-verifier/scripts/` 既有自动化入口。

## 技术栈

既有栈。零新依赖。

## 需求分析与背景调查

（KNOWN-DEBT 全量对账 2026-08-31：P47x–P50x 登记项逐条过筛）

- **A 族证据**：486 表格行（session.rs:1220 段 16ms 单发 try_recv；系统级
  LOCATIONCHANGE 噪声下 MOVESIZESTART/END 排队，实机松手→落位延迟数秒）——
  修复方向已在债务行写明（drain-while-empty + 可选分级）；P488-D3 合成拖拽
  实证的毫秒级快甩滞留同源。
- **B 族证据**：P487-3（shell.at v1 起双分支重复）、P497-1（计划文本定了
  ≤4 截断、实现漏了）、P497-2（a2vue 金样 SFC props 不透传）、P501-1
  （发现序 PATH 级留扩展位）、P480-R1（enable_broker 无生产调用点）。
- **C 族证据**：P487-1/P496-1/P501-2 三条同因（"OS 注入受阻变体/前台竞争/
  CUA 像素身份守卫"），472/478/479 各计划均有同族先例——每案单独绕的边际
  成本已高于统一解。
- **排程**：500（execution_done）待复审、503（视觉刷新）drafting 待领——
  本期与其零交叠（A 族碰 session 事件泵段、B 族碰 shell 资产与 registry
  小点，均不在 503 视觉面/500 协议面）。504 在途不涉。

## 详细设计

### 1. A 族：事件泵与拖入时序

- `native_dock_event_subscription`（session.rs:1220 段）改为**每拍
  drain-while-empty**；通道分级：MoveSizeStart/End 走高优先级通道（或同通道
  出队时优先级拾取），LOCATIONCHANGE 噪声降级；
- 指针采样节流保持 30Hz 不变（只改排空策略，不改采样率）；
- P488-D3：快甩（START→END 毫秒间隔）时 DragWatch 终态直接按 END 时指针
  判定（跳过中间 Over 态的滞留）。

### 2. B 族五小件

各 0.5 天内：双分支合并为参数化单分支（P487-3）；pager 行渲染加 ≤4+"+N"
（P497-1）；a2vue 生成器 props 透传补（P497-2）；发现序第三级 PATH 探测
（P501-1）；`enable_broker` 接 desktop 关停路径一处调用（P480-R1）。

### 3. C 族：实机验收通道（流程资产）

- 产出 ADR 短文：专用验收会话操作规程（什么状态可注入、守卫放行怎么配、
  失败回退）；
- 落一个放行配置/开关 + 用该通道补拍三条债的实机照（P487-1 齿轮开面板 +
  dock 热切换/Esc、P496-1 壁纸/图标交互、P501-2 齿轮→os-config 全链）；
- 驱动脚本归 `.agents/skills/autoui-verifier/scripts/`（复用入口不另起）。

### 4. D 族

- P488-D4：DoDragDrop 完成回注时取**发起时锚定 AppId**（不查完成时焦点，
  避开 VM 无焦点查询的缺口）——语义更稳；
- 壁纸热切换：探针验证投影字段重注入是否天然热刷新（指纹门控）；是→
  顺路做并实机照；否→一行"不做+理由"回写 496 债务行。

## 测试设计

1. **T1 A 族**：事件泵单测（注入 100 条噪声 + 2 条 START/END → 一拍排空、
   优先级序断言）；快甩用例（START/END 同拍到达 → 终态正确）。
2. **T2 B 族**：各小件既有验证口径复跑（shell 装载测/pager 用例/a2vue 金样/
   daemon 发现序单测/broker 单测）。
3. **T3 C 族**：通道规程演练一次全绿 + 三债实机照归档。
4. **T4 实机**：拖拽落位延迟体感复核（A 族）；其余随 C 族通道。

## 验收标准

1. A 族：注入式单测绿 + 实机拖拽落位延迟体感达标（秒级→即时）。
2. B 族五项各回写已清偿；T2 全绿。
3. C 族：规程文档 + 放行机制入库，三债实机照留痕回写。
4. D 族：P488-D4 单测+清偿；壁纸热切换定案成文。
5. `cargo t ui`、`cargo t session` 不回归；零警告；KNOWN-DEBT 记账零丢失
   （排除项仍在登记）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **A-事件泵**：`crates/auto-lang/src/ui/session.rs:1220` 段 drain-while-empty
   + START/END 优先出队 + T1 单测。
   验证：`cargo t session && cargo t native_dock`。
2. **A-快甩**：`ui/native_dock/mod.rs` DragWatch 终态快路（END 即判）+ T1。
   验证：`cargo t native_dock`。
3. **B1 任务栏去重**：`crates/auto-lang/assets/shell.at` 双分支合并 + 装载测。
   验证：`cargo t desktop_mcp`。
4. **B2 pager 截断**：shell.at pager 行 ≤4+"+N" + 用例。
   验证：`cargo t desktop_mcp`。
5. **B3-B5 小件**：a2vue props 透传（`ui_gen/vue.rs` 生成段）；daemon PATH 级
   （`ui/osconfig_daemon.rs` 发现序）；`enable_broker` 生产调用点（session
   关停路径）。
   验证：`cargo t vue && cargo t osconfig_daemon && cargo t session`。
6. **C-验收通道**：ADR 短文（`docs/design/autoui/` 或 plans/reports）+ 放行
   开关 + autoui-verifier 脚本入口。
   验证：通道演练一轮绿。
7. **C-补拍三债**：P487-1/P496-1/P501-2 实机照归档回写。
   验证：三债行已清偿注记。
8. **D 族**：P488-D4（发起时锚定 AppId）+ 壁纸热切换探针定案。
   验证：`cargo t native_dnd && cargo t ui`。
9. **记账收尾**：全部清偿项回写 KNOWN-DEBT；排除项核对仍在；健康检查；
   状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- C 族放行机制形态（守卫白名单 env / 专用测试窗标记 / 时段窗口）在 T6 时
  按实机环境定稿——原则：不改生产守卫默认行为。
- 壁纸热切换若探针结论为"需额外管道"则明确不做（本期不扩范围）。
- P488-D4 的"发起时锚定"若与 dnd_finished 既有消费者语义冲突，回退为债务
  维持（记录冲突证据）。
- A 族优先级出队若通道结构不便分级，允许退化为"纯 drain 不分级"（已消
  90% 延迟），分级转增强——以实测数据定。
