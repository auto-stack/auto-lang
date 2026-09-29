# PLAN-708 r1 实施方案审查

日期：2026-09-29。结论：**needs_replan（实施前契约修订）**。

用户提到的 PLAN-706 已归档，主题为 gallery Vue 展示修复。本次实际审查对象是 `docs/plans/708-vm-render-responsiveness.md`，其任务与用户描述一致。本次是独立于起草/执行上下文的设计审查，不是实现验收；计划保持 `drafting`，不授予开工或合入通过。

## 1. 审查基线与范围

| 项 | 基线 |
|---|---|
| Plan | PLAN-708，plan_revision=1，审查前正文 SHA256=`DA745A3EE5DCCEFCE330D1E1587DD169BCEE39D3CD0D8DF0D02DEC448F94DB26` |
| auto-lang 主检出 / 代码审查基线 | `66c9cac193ad1a674be521e61440fcf5c65145e4` |
| 708 分支 / 预建 worktree | `plan-708-dev` / `ec5adb7af02450f7bf2a8f0ea1543859b385ecbc`；登记路径 `/mnt/d/autostack/.wt/lang-708/auto-lang` |
| auto-os gallery | `93050a6a19231238cd5d4478909e9d6efd30be95`；`widgets-gallery/src/front/app.at` SHA256=`D817F20DB6B9801305F446DC4137C5AC9683ED0B6BE27BDCFC99BB5F0113A86E` |
| UI Spec 输入 | `docs/specs/auto-lang/ui/architecture.md` SHA256=`2539D60836531F07EB146EBCC19B6655527659A300D9AE39A682EA7080EF1866`，重点 ADR-19/24/25 |
| 调度器依据 | 本机 Cargo registry 中 iced_winit 0.14.0 `src/lib.rs`；本仓 iced 版本与锁定依赖一致 |

读取了计划、现行代码、相关 Spec、测试断言和 gallery 语料。未运行 Cargo 或实机性能实验：本次未改 Rust 源码，遵循 AGENTS.md Category A。计划的 10–30s 观察保留为起草者报告，不能由静态审查证明其根因或改善幅度。

`git worktree list --porcelain` 将 708 标为 prunable，Windows 下其 `.git` 指向 `/mnt/d/...`，`git -C D:/autostack/.wt/lang-708/auto-lang status` 失败。这可能是 WSL/Windows 路径混用；本次未修复、prune 或删除。实施前应在创建它的环境验证检出有效性、基线与本地改动。主检出已有 707 簿记和走查临时文件，未将其纳入本审查。

## 2. 必须修订的发现

### R708-01 [P1] 当前方案没有约束 UI 线程的连续占用，不能支撑 responsiveness 目标

关联：G3/G4、M-02、Q-02、T-04、AC-05/06。

`drain_pending_inits → call_handler_for` 仍在 UI 线程同步执行，且 Q-02 明确整段跑完。`vm/engine.rs:2331` 说明不 yield 的 handler 在调用内完成；`drive_handler_segment`（2513 起）使用 10,000,000 步保护预算，这不是短时间片。702 的 parked 路径释放的是 I/O 等待，不会自动分片纯 CPU 循环；就绪后的恢复段也可能再次很长。

因此，两拍最多改善加载反馈，之后仍可能冻结秒级；一次 drain 多个 Init 会累加占用。把它作为彻底消除无响应的中期措施，目标和机制不匹配。仅限制每次 drain 的任务数量或在两个 Init 之间检查耗时，也不能切断单个长 Init。

**修订建议**：先测连续占用，再明确本阶段交付边界。如果中期承诺加载时仍可交互，应加入可续跑的 CPU 时间片/指令片，或把已定位的重计算放到独占 worker；预算耗尽须保存调用栈后续跑，不能当成 handler 成功/失败结束。区分 CPU 让出与 I/O Pending，覆盖首次派发及 resume。若暂不实施，应将 M 明确命名为“首屏加载反馈优化”，删除消除冻结的承诺，并把实现非阻塞执行列为后续必做计划。VM 执行契约变更按 L2 补架构设计与独立验收。

### R708-02 [P1] 短期任务基于陈旧基线，缺少耗时归因

关联：§0/4、S-01/02、T-01/02/06/09、AC-01/03/06。

gallery 的 `app.at:183` 与 `app.at:696` 已有 `sidebar_provider (..., memo: true)`，`app.at:701` 已有 `outlet (memo: true)`。当前 `aura_view_builder.rs:5140` 中 prop=true 已开启 outlet memo。因此 S-02 是现状，S-01 的缺省修改也不会直接改变 gallery 这一路径，不能作为其性能改善的主要依据。

`pages/row.at` 没有 Init；`pages/datatable.at` 只有排序事件，没有 Init。`components/filetree.at:39` 将 `flatten_tree` 放在 computed，Init 仅设置展开态；`area_chart.at:86` 才有明显计算型 Init。计划将树拍平、大表处理一并归因于 Init，证据不足。

其他应测路径：renderer `21961` 的 MCP 同步构建与 `22209` 的真实显示构建可能同帧各执行一次；builder `4385` 起的 preview-card 在构建时还调用 VueGenerator 生成示例代码；layout/draw 和 computed 也不受 Init 延后直接覆盖。这些是调查候选，尚未实测定罪。

**修订建议**：新增实施首项 T-00，固定代码/二进制/语料版本，分别测冷启动、首访、复访和稳定帧；记录 event/update、各 Init/handler、computed、模板构建、代码示例生成、Element 转换、layout/draw/present、MCP 同步的独立耗时。复用 `AUTO_MEMO_DIAG`、`P631_PROFILE`，记录 memo hit/miss/degrade 的原因。对 MCP 开/关分别测量，不能把 MCP 请求时长视为 UI 冻结时长。S-02 改为既有能力核查；实测定位后再决定优化项。

### R708-03 [P1] “update 返回前 / AppTick”没有定义可靠的帧后执行屏障

关联：M-01/02/03、T-03/04/05、AC-04/05/06。

本机 iced_winit `src/lib.rs:1312–1339` 先处理 update，再重算 subscription；`AboutToWait` 分支随后 build_user_interfaces。draw/present 位于 RedrawRequested 路径。因此 update 末尾 drain 发生在后续 view 和上屏之前；若 pending 在 view 中才产生，本次 update 已经结束。“返回前”不是“帧交付后”。普通异步 Task 下一消息也不能单凭异步性证明已 present。

首个 view 产生 pending 时，若条件订阅此前已评估为空，需要明确谁主动唤醒并更新订阅，否则可能等到无关事件才运行。计划定位的 `renderer.rs:26180` 是通用 `ComponentIced::update`；真实 VM 应用走 `run_session` 内 `update_inner`（16597 起）与 DesktopSession 的 update closure（19387 起）。必须接到真实执行路径。

**修订建议**：画清 route commit → pending 入队 → placeholder 构建 → draw/present 确认 → Init 调度的顺序，选择并验证实际可用的帧完成信号，不能以 sleep 或普通 tick 代替证明。定义首次挂载无输入时的主动唤醒、多 AppId 隔离、所有提前 return 分支、暂停/最小化时的进展策略。Init/resume 写回后须同步 component dirty、AppState.view_dirty 及相关缓存；真实 update 的末尾传播逻辑（19146 起）会被提前 return 跳过，单测 component.dirty 不足以证明屏幕刷新。builder/view 是 `&self`，应明确共享队列的 interior mutability 和接入点，不能只声明普通 `Vec`。

### R708-04 [P1] deferred Init 缺少生命周期和异步完成状态

关联：M-01/03、T-03/05、AC-04/05/07。

`VmBridge::child_init_should_fire`（1447 起）在判定时就写入最后身份；这在“当场派发”下成立，延迟后还需要记录未执行、执行中和完成。702 中 `call_handler_for` 返回 Ok 也可能只代表 Parked 注册成功，不能据此移除加载态或把 Init 判成完成。

延迟期间可能重复 view（包括 MCP 与显示两次构建）、切换 key/route、A→B→A、热重载或卸载。当前 `widget + state_id + identity` 未定义旧任务失效规则；props 每帧重新播种，必须避免旧 Init 对新身份的状态执行。页自身无 Init、内部 child 有 Init 时，也不能只以“页 pending”决定占位。

**修订建议**：定义至少 queued/running/parked/completed/failed/cancelled 状态及 mount generation；明确根/store/页/嵌套 child 的顺序与 props 播种条件。保留当前 key 契约，校验 dispatch 时身份仍有效；未执行旧项应丢弃，已 parked 项要定义恢复与清理。占位到真正完成才结束，错误须有诊断，不能永久 Loading。pending 子树不得缓存成永久占位。测试覆盖无 Init 页中的计算型 child、二次构建去重、key 切换、离开后迟到完成、失败和卸载。这里是新方案必须补的语义，不是要求在本计划顺手重做整个多实例状态系统。

### R708-05 [P1] 缺省开启 memo 前必须验证宿主局部状态的失效

关联：S-01、T-01/08、AC-01/07。

outlet memo 的快速命中条件（`aura_view_builder.rs:5292` 起）主要检查 VM mutation seq / 全局 episode / recorded deps。preview-card 产物却读取 `preview_states`（4387 起）；renderer 的 `__preview_toggle` / `__preview_tab` / `__preview_copy` 分支（17214 起）只改宿主局部 map 和 `view_dirty`。`GlobalEpisode`（`memo_deps.rs:1015` 起）未包含这类局部状态。静态代码显示存在“UI 标脏但仍返回旧 memo 产物”的风险；本次没有运行交互复现，不能把它记成已实测回归。

**修订建议**：为 preview/nav 等宿主局部状态定义版本失效或保守 Degrade；验证 theme、popover、路由参数、props、热重载、MCP probe 开关及 mount/timer/事件簿记重放。核心验收是 memo 开/关的可观察行为一致，而不只是命中计数增加。该风险若已存在，也不能因其是预存而在扩大全局缺省面时免除验证。

### R708-06 [P2] 强制关闭公式与文字、AC-02 冲突

关联：S-01、T-01、AC-02。

计划 after=`prop_memo || env != "0"`。gallery prop=true 时，即使 env=0，结果依然为 true，不能“强制关”。现有 `plan046_outlet_prop_priority`（`aura_view_builder.rs:21990`）明确 prop/env 并集兼容规则。

**修订建议**：先冻结优先级。如果 env=0 是诊断强制关，必须先判断它，再决定缺省/显式 prop；明确 `memo:false` 是否保有关闭含义。补 env 未设/0/1/其他值 × prop 未设/false/true 的矩阵，并同步调整 046 的既有测试和规范。否则将文案限缩为“关闭环境缺省开门”，但那不能用于 gallery 的 A/B 诊断。

### R708-07 [P1] 验收只观察首帧和终态，没有证明加载期间可交互

关联：G4、§6/7、T-08/09、AC-06/07。

“首帧 ≤16ms 级”没有测量起止点/设备/冷暖状态；AC-06 又只要求实机看到壳。截图不能证明首帧延迟或期间没有冻结，Init 后内容齐也不能证明 Windows 消息泵可用。

**修订建议**：将验收拆为输入→首次实际 present 延迟、最大 UI 连续占用、心跳/输入响应间隙、完整内容就绪时间。T-00 后冻结可复现预算和样本数，报告分位数及最坏值；16.7ms 可作为 60Hz 帧预算目标，不能在无证据时直接作硬保证。加入加载期间滚动侧栏、再次导航、resize/关闭、长 CPU Init、async Init、连续切页，以及多会话互不阻塞测试。单测队列计数证明的是队列语义，不能替代 renderer 端到端验收。按最终触碰面运行 scoped gates；若改变 VM/核心执行契约，最终 review/fold 再一次 cargo tf；不无关触发 taa/docs_gen。

### R708-08 [P2] 长期方案应进入正式设计，不能写成当前已生效 ADR

关联：G5、T-07、SD-03、AC-08。

SD-03 拟将未实施的 worker 架构放入 current-state `architecture.md` ADR-27，与 auto-plan 规范“Spec 描述当前行为与持久决策”的约束冲突。短短三条也不足以成为可执行设计。`RefCell` 本身不等于不能 Send；独占 worker 内部的 RefCell 不必为跨线程而全部改成 Mutex，需要查真正跨边界的类型及所有权。

**修订建议**：在 `docs/design/autoui/` 新建设计并登记 `docs/design/00-intro.md`，用明确 planned/proposed 状态记录未实施部分，规范只链接路线及已生效边界。设计应回答 worker 独占哪些 VM/状态，UI 如何保留最后快照，输入如何排序/合并/取消，有界通道与背压、快照复制成本、错误退出、关窗与热重载、MCP/timers/parked 泵归属。避免 UI 持有同步 Mutex 等 worker 计算的方案。与 707 对 parked readiness/cleanup 的修改建立 revision 依赖；实施前重同步基线，最终复审联合接口变化。

## 3. 建议的修订结构

1. T-00：可复现性能基线与分段归因，先区分初次构建、重复构建、CPU Init、computed 和渲染布局。
2. S：针对实测热点减少重复构建和生成税；把 gallery 现有 memo 标志改为基线核查；修正强制关闭矩阵，补行为等价门禁。
3. M：实现明确的挂载状态机与帧完成调度；如果承诺加载时可交互，同时加入可中断/续跑的重计算机制，约束单次及总 pump 预算。
4. L：独占 worker 的正式设计、所有权和性能成本证明；不把未实现状态写成已生效 Spec。
5. 验收：同时测首帧、最大连续占用、交互间隙、终态及旧任务清理；证据随版本绑定。

优先处理 R708-01/02/03/04/07，随后收口 R708-05/06/08。这些修订改变目标或执行契约，需 `/auto-plan:new` 将 plan_revision 增至 r2，再审查后进入 work。保留 T/AC 稳定 ID，增加新项而不擦除本次记录。

## 4. AC → 任务 → 设计证据

以下全部为设计检查；**无实现验收 pass**。

| AC | 任务 | 设计检查结果 / 发现 |
|---|---|---|
| AC-01 | T-01/08 | partial：机制已有，但 gallery 原本已开；缺省扩大后的行为门禁缺失。R708-02/05 |
| AC-02 | T-01/08 | fail：prop=true 时 env=0 无法强制关。R708-06 |
| AC-03 | T-02/09 | 基线已满足开关项，缺少新增改善证据。R708-02 |
| AC-04 | T-03/08 | partial：方向合理，需补 builder 接线、去重与生命周期。R708-03/04 |
| AC-05 | T-04/08 | fail：帧后顺序未保证，派发一次不等价真实完成。R708-01/03/04 |
| AC-06 | T-05/09 | fail：首帧时序和延迟定义不足，期间交互无门禁。R708-01/03/07 |
| AC-07 | T-05/09 | partial：截图可验终态，缺少 async/失败/切页/cache 语义。R708-04/05/07 |
| AC-08 | T-07 | fail：拟议长期设计落 current-state ADR 且设计细节不足。R708-08 |

## 5. 审查的规范增量快照

审查对象 r1 的原拟议 delta（原样冻结，未发布）：

| delta_id | add/modify | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/architecture.md | 无 outlet memo 缺省规则 / 缺省开+显式关 | 降重复构建税 | AC-01 |
| SD-02 | modify | 同上 ADR-26 | 无 Init 两拍 / view 不派发 Init，帧后 drain | 绘制与计算分离第一步 | AC-04/05 |
| SD-03 | add | 同上 ADR-27（L 设计） | 无 / VM worker 边界设计 | 长期路线固化 | AC-08 |

修订要求：SD-01 加优先级与失效纪律并挂 AC-02；SD-02 调和 ADR-19 的陈旧逐帧重放文字、536 身份语义和 ADR-24 的 parked 真实完成契约；SD-03 转正式 planned design，不能声称 worker 已交付。完成实现审查后才填写最终 spec-impact metadata；本次不修改 canonical Specs 或 `.autoos/specs.json`。

遗漏/延后扫描结论：纯 CPU 的非阻塞执行被推迟到 L，但 G4 仍承诺改善无响应；首帧/生命周期/失效/量化门禁遗漏须在契约内补齐，不能以记债视作完成。宿主局部状态缓存失效与 computed/构建计算残面作为待核实风险登记至 KNOWN-DEBT-AND-RISKS；未确认其性能归因。健康检查限于计划和现有代码阅读，没有新 Rust diff，编译告警/格式/实机健康留实施验收。

## 6. 结论

缓存与让 view 避免同步 Init 的方向合理，但 r1 基线失准，调度与生命周期不闭合，无法证明用户要求的响应性改善。**先修订 r2；当前不建议按 r1 开工。**
