---
plan_id: PLAN-708
status: archived               # 终态（r3 收窄交付，2026-09-30 归档；M/L 档承接=PLAN-711）
# r3 收窄（2026-09-30 用户裁定）：S 档（T-00/01/02）+正确性观测与文档收口即关计划；
# M/L 档（T-03/04/05/11/12 + T-06 余量/T-09 全矩阵）移出单独立项 PLAN-711。
# r2 范围确认：用户 2026-09-30 明确调用 /auto-plan:work 实施 PLAN-708（Q-06 的 r2 契约确认）。
feature_name: vm-render-responsiveness
author: [agent]
created_at: 2026-09-29
updated_at: 2026-09-30
plan_revision: 3

# r3 定案（已交付行为收口）。
supersedes_spec_components:
  - docs/specs/auto-lang/ui/architecture.md
new_spec_components:
  - docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md
touched_goals: [GOAL-007]

affects: [auto-lang/ui]
current_step: 7
total_steps: 13
---

# [PLAN-708] vm-render-responsiveness

## 0. 变更摘要

本计划解决 widgets-gallery VM 加载/切页时 UI 响应延迟。r1 独立审查发现：gallery 已开启两类 memo，部分重页没有 Init；无预算的 deferred Init 仍堵 UI，“update 返回前 drain”也不等于帧已呈现。r2 依据 [R708-01..08](reports/708-design-review.md) 修订，保留任务/验收 ID 和全部复审历史。

架构依据：[VM 加载响应性专题设计](../design/autoui/vm-loading-responsiveness.md)（proposed，未实施）；复用 [帧预算设计](../design/autoui/vm-frame-budget.md)、[Design 34](../design/34-vm-store-ownership-and-handler-discipline.md) 的 CPU 段切讨论，保持现有共享堆与单 VM 执行者，不在本计划裁定 actor/store 全局所有权。

| 档 | r2 内容 | 本计划交付 |
|---|---|---|
| S | 先分段归因；修正 memo 优先级/失效，减少实测重复构建与代码生成税 | 实施，已有开关只作基线核查 |
| M | view 登记 Init、挂载代际状态机、真实显示路径调度、可续跑 CPU 执行片与有界恢复泵 | 实施；覆盖首次派发和 resume |
| L | worker 独占 VM/准备展示快照、UI 不同步等 VM 的正式设计 | 仅设计，后续独立 Plan |

本轮完成的是 replan；所有实现与性能证据仍待 work。未经探针验证，不宣称特定代码是 10–30s 的唯一根因。

## 1. 目标

### 目标

- G1（S）：outlet 页 memo 缺省开启，显式关闭/诊断开关规则完整；扩大默认面前证明缓存开关的可观察行为一致。
- G2（S）：保留 gallery 已有侧栏/outlet memo，验证有效命中与失效；按实际耗时减少重复构建，不能把既有旗标列为性能收益。
- G3（M）：显示路径不执行 child/page Init；view 发现需求后，按可验证的骨架帧交付顺序启动有预算任务；真正完成后刷新完整内容。
- G4（M）：加载时继续处理滚动、窗口操作、取消和再导航；首帧与完整就绪分别验收，长 CPU Init/恢复段不能占满一次 update。
- G5（L）：交付正式 worker 设计，明确线程所有权、快照/队列、取消/退出与 parked 迁移；不将其写为已交付 ADR。

### 非目标与边界

- 不做 JIT、字节码 ISA 重设计、handler 类型限制或全局 actor/store 所有权迁移。
- 不改 Vue/Web 生成行为；Vue 作为共享语料的终态参考。
- 不把 AutoVM 整体搬线程，不实施通用列表虚拟化或整个 iced 渲染器替换。
- 不声称任意阻塞 FFI、任意大布局和冷编译都有硬实时上界。gallery 及受测夹具中的真实超限路径必须处理或返回 needs_replan，不能记债后通过。
- 保留共享根态/当前实例支持面；不借 Init 队列重做全部多实例隔离。取消不回滚已发生副作用。
- 本计划的导航指标从已装载 App 会话的输入开始；编译/装载/开窗前耗时单列，若发生在 desktop UI 泵内也必须计入连续占用，不能从总卡顿中删掉。

### 影响模块

| 仓/文件 | 改动 |
|---|---|
| auto-lang `crates/auto-lang/src/ui/aura_view_builder.rs` | memo 三态/局部失效；Init demand；骨架与热点构建准备 |
| auto-lang `crates/auto-lang/src/ui/dynamic.rs` | 挂载/代际状态；CPU/Init 就绪观察；dirty 传播与 prepared 结果 |
| auto-lang `crates/auto-lang/src/ui/vm_bridge.rs` | 带身份的执行结果、CPU continuation、有界泵、取消与完成通知 |
| auto-lang `crates/auto-lang/src/vm/{engine,task}.rs` | UI 专用 CPU 片/累计安全预算；保留非 UI 同步契约 |
| auto-lang `crates/auto-lang/src/ui/{session.rs,iced/renderer.rs,memo_deps.rs}` | 真入口/挂载与 frame 接线、局部版本、展示/MCP缓存 |
| auto-lang `docs/design/autoui/vm-loading-responsiveness.md` | L2 设计先行；worker 只作提议 |
| auto-lang `docs/specs/auto-lang/{ui,vm}/...` | 已实现行为的拟议沉淀，review/merge 后写回 |
| auto-os widgets-gallery | 默认只读固定语料做性能/终态核查；若须改具体热点语料，在同组 auto-os worktree，记录原因，不改展示需求 |

### r1 → r2 契约变化

1. 新增 T-00/T-11/T-12 和 AC-09..12；所有原 T/AC ID 保留。
2. T-02/AC-03 从“打开已有旗标”改为“核查有效缓存及减少实测重复成本”。
3. M 从无界两拍改为 CPU 可续跑与完整挂载调度；这是为达成原 responsiveness 目标的执行契约修订，须纳入 r2 的实施授权。
4. G4 原“首帧 ≤16ms 级”保留为 60Hz 优化目标；r2 提议下述可复现硬门禁并增加加载期交互指标。这是阈值明确化/变更，不继承为已获批准，也不由 T-00 自行放宽。
5. AC-08 保留“交付 L 架构设计”的目的，位置由拟议 current-state ADR-27 改为 proposed 设计文档。

## 2. 架构方案

### 当前事实

```text
UI 输入 → run_session/update_inner → handler 首段/恢复段（同步 CPU）
                     ↓
            dynamic_view_impl
              ├ MCP 同步构建（受门控）
              └ 显示构建 → child Init / computed → Element → layout/draw/present
```

702/705 将 I/O 等待 parked，不自动分片纯 CPU。gallery 侧栏与 outlet 已有 `memo:true`；FileTree 的 flatten_tree 在 computed，Row/DataTable 页本身无 Init。MCP 旁路和 preview-card 示例代码生成也是待测候选。

### r2 实施顺序

```text
T-00：版本/性能基线 + 帧顺序/CPU 安全探针
   │
   ├ S：开关与失效正确性 → 定向减少重复成本
   │
   └ M：CPU continuation + 总 pump 预算
          → Init 挂载状态机
          → window/AppId/代际的骨架帧通知
          → 有预算的首次/恢复执行
          → completed/failed/cancelled 真实终结
          → display/MCP 同代际刷新
```

在 T-00 未通过时不得直接进入 M；报告真实失败并修订，不以普通 tick、全部 Init drain 或关闭 MCP 作替代。长期 worker 另立 Plan，本次文档先行。

## 3. 技术栈

- Rust + iced 0.14（以 Cargo.lock 锁定版本为准），AutoVM/AutoTask 原执行栈。
- 702 parked/重入/清栈纪律；705 的 HTTP 等待；707 正在扩充 stream readiness/取消，按实际落地 commit 配对。
- 045/046/047 的 memo、dep/path version、computed signal；局部 UI epoch 与 mount generation 为拟新增面。
- 调度在真实 `run_session` / DesktopSession 内，不只改 `ComponentIced::update`。
- 验证复用 autoui-verifier 的 `test_vm_mcp.py`、`test_vue_playwright.mjs`；性能事件采集可在既有脚本增能力，不另建一套临时驱动。

## 4. 需求分析与背景调查

### 用户授权与实施边界

- r1 起草记录声称：2026-09-29 用户已授权短期+中期实施、长期仅规划；本会话不伪造对该历史的独立确认。
- 本会话用户要求审查实施方案，得到 needs_replan；随后明确“OK，继续下一步”，授权修订到 r2 并检查方案。
- 本轮产物为计划/设计；未运行实现门禁，未将本轮修订授权扩写为合入/发布授权。
- 开始行为实施前需确认 r2 中新增 CPU 段契约、串行纪律及 §7 的拟议门禁；确认后不重复询问相同范围。T-00 的未决技术项由有界探针解决，禁止以降低指标消除失败。
- 未指定预算、自动续跑上限。main 仅写计划/设计簿记，所有实现、构建与运行在 708 worktree。

### 版本/规格基线

| 输入 | 版本或内容 SHA256 |
|---|---|
| auto-lang current / diff base | `66c9cac193ad1a674be521e61440fcf5c65145e4` |
| 708 原分支 | `ec5adb7af02450f7bf2a8f0ea1543859b385ecbc`；Windows Git 不能解析其 /mnt/d gitdir，T-00 在创建环境验证并重同步 |
| gallery / auto-os | `93050a6a19231238cd5d4478909e9d6efd30be95` |
| ui/architecture.md | `2539D60836531F07EB146EBCC19B6655527659A300D9AE39A682EA7080EF1866` |
| vm/architecture.md | `A793830B492DDCB2C8C7C34C443FE931994C2E4AA2AEA961827B621830E0241A` |
| design/autoui/vm-frame-budget.md | `EA7A455F8AC2F17BA67C95A007D92E65425EC0C90A493BD7E9D286A6E4E1A693` |
| Design 34 | `9F349C88228F0A84F7BC00FEC8313983E7501962209C4DA838C9D14F6F2DE51A`；待裁定提议，不把其所有权选型当成已生效 |
| 707 契约 r1 | SHA256 `2B8DB9C713FE79D4B96C0B7E8E379D17A64F6F3153C23D5576004FE8F4700729`；执行分支观察 `2da544537994d0f1dcff53e00b7dac9f60b6e007` |
| r1 审查后正文 | SHA256 `9EEF963C9A30A928F1A9204F2EB193B36922FE8C4112DEE1E528B99DE511E01B`；审查记录在 §9 |

### 静态调查与证据限制

- `app.at:183/696/701` 已启用 sidebar/outlet memo。
- `aura_view_builder::render_outlet_inner/render_outlet_page_memo/fire_child_init_if_any` 是开关/Init 真实入口；`child_init_should_fire` 判定即写身份。
- `engine::drive_handler_segment` 10M 步预算不是短时间片，现有耗尽路径也不是可续跑的正确 Completed 语义。
- `VmBridge::call_vm_fn/call_computed_fn` 是同步 Value 契约；computed signal 冷 miss 仍可能同步重算。
- `DynamicComponent::view_with_debug_gated`、`dynamic_view_impl` 的 MCP/显示构建、`VueGenerator::gen_previewcard_code` 必须独立测量。
- `run_session/update_inner` 有多个提前 return；component dirty 必须传播到 AppState.view_dirty。
- 本地 iced_winit 0.14 的 frames 广播位于 compositor.present 之前。frames 本身不是成功上屏通知，T-00 必须验证实际屏障。
- 上轮走查的 10–30s 为历史观察，没有本轮采样数据；本轮不把静态调查写成性能验证通过。

## 5. 详细设计

### S-01 开关及失效

读取 env 三态与 prop Option<bool>，优先级冻结如下：

| AUTO_OUTLET_MEMO | prop 未设 | prop=false | prop=true |
|---|---|---|---|
| 0 | off | off | off |
| 1 | on | on | on |
| 未设/其他值 | on（缺省） | off | on |

env=0 是诊断强制关，覆盖显式 true；env=1 保留显式强开兼容。046 的并集契约调整须明确记录，测试矩阵不能只测 prop=false。

产物失效至少覆盖 preview show/tab/copied、nav 局部态、route/params、props、theme/popover、模板/热重载、mount generation、probe 配置。局部 UI epoch 必须进入 outlet/memo 适用键或保守失效；静态扫描无法证明的继续 Degrade。pending/错误/不完整产物不得当正常页永久缓存；命中帧重放实际 child mount/timer/事件路由簿记。

### S-02 gallery 定向减负

T-00 给出热点排序、占比和开关 A/B：冷首访/复访/稳定帧、MCP 开/关分别测。保留已有旗标，不能以“已打开”宣布收益。

只实施报告中超预算的重建/生成热点：示例代码按模板/生成器/选项缓存；MCP 与显示尽可能共享已提交构建结果，维持 probe/bounds/状态采集语义。Element 不可 Clone，不把 store-then-take 当跨帧复用。通用虚拟化或 renderer 重做需要重新规划。

### M-01 demand 与生命周期

以 AppId、当前 widget/实例路径、key、mount generation 定位需求。view 经共享 interior-mutable sink 登记，不直接派发；首次发现预留身份，MCP/显示双 build 不重复排队。

状态为 queued/running/runnable/waiting/completed/failed/cancelled。父/页准备完依赖 props 后启动 child；无 Init 页内的 child 仍被发现。根/module/store 的实际调用路径在 T-00 记录，须维持既有顺序；若 UI 加载期存在同步重 Init，纳入同一预算路径，不能漏掉宿主入口。

handler observer 明确区分“已接受段”与真正完成：仅 Completed 才解除 Init loading，Missing 可选 handler 静默，异常失败保留错误面。key/route/reload/close 取消旧代际；未执行项直接丢弃，已开始项清栈/等待凭据/忙态一次清理。取消不回滚前缀副作用，禁止旧段在新代际写入。

### M-02 CPU 预算与恢复泵（T-11）

UI 专用首次调用/续跑返回 Completed、I/O Waiting、CPU Runnable 或 Cancelled。保留完整执行栈和所有权，不重进函数/重复请求、不提前清栈、不将预算耗尽伪装为成功。非 UI 同步接口不被迫消费未完成 Value。

提议每片 4ms/4096 指令，至多每 64 指令查时钟；会话宿主每轮 CPU pump 总预算 8ms。首次/恢复、多个 App、突发输入共用轮次预算；ready 集有限，FIFO/round-robin 保公平，本轮让出的任务不得本轮立即反复续跑。native/FFI 单调用超过预算仍是失败热点，必须处理或 replan。

10M 指令/增长保护跨 CPU 片累计，I/O 等待不算忙耗时。CPU continuation 存活时，同 App VM 写事件串行有界排队（提议 128），覆盖 input 代写、timer、props、MCP fixture、reload；宿主滚动/resize/close 可继续。I/O park 时沿 702 的交错边界。展示沿已提交快照/骨架，避免 view 再写正在计算的 props；必须测试闭包、异常、多帧、返回槽/RC、非交换写序与取消。

### M-03 帧交付与骨架

真实接线：`run_session/update_inner`、DesktopSession update、`dynamic_view_impl` 及 session 挂载入口。流程为 demand → 带版本骨架 → draw/present 交付 → 按 window/AppId/generation 消费通知 → 有预算 Init。

T-00 冻结能证明该顺序的公开 API/宿主适配，验证 frames+draw 标记是否足够；未证明时不得以 update 末尾/sleep/普通 tick 替代。需要额外 renderer 回执时先写 decision 并重审影响范围，不能暗加未规划的 iced fork。

通知通道预先建立；view 登记需求仅唤醒一次，不依赖从未刷新的条件订阅。无任务时不运行周期计算泵。最小化/不可见窗用明确策略推进，恢复可见时显示最新代际。

loading 文本默认“Loading…”，保留已有侧栏和窗口交互；嵌套需求按依赖阶段发现，占位能继续准备子件而非永久早退。完成/失败/cancel 更新 component dirty、AppState.view_dirty、memo/已提交展示/MCP 版本，所有提前 return 臂都覆盖。

### M-04 computed / 冷态构建残面（T-12）

预算不覆盖 `call_computed_fn/call_vm_fn` 的同步 Value 契约。T-00 测 FileTree.flatten_tree、预览代码生成、冷态树构建与 layout/draw；不把它们归为 Init。

如果超预算，先复用已存在的 computed signal/依赖版本缓存、分段准备具名热点，让 view 只读 Ready 值；缺值维持 loading，不能用 Nil/空成功占位而污染缓存。异步准备的依赖/props 版本与失效要闭合，结果槽 RC 接管与 062/047 同纪律。副作用或依赖不可证明时不得自动搬所有 computed；无法在 S/M 边界内达成 gallery 门禁则 needs_replan，不能把必需热点推给 L 后仍宣称完成。

### L 设计与 707 协调

专题设计 §7 给出独占 worker 边界、快照/事件队列、取消/退出与资源句柄；不要求 RefCell 一律改 Mutex，UI 不同步等待 worker。

707 r1 正在修改 engine、wait readiness 和清理。T-00 记录实际分支/接口，实施前重同步已落地主线；708 修改必须兼容最终 wait 集合，不抹掉 707 新分支。交付前有串联验证：stream/async wait → CPU-heavy resume → cancel/close，各类资源终结一次。不等待 707 完成来进行基线/缓存工作，但联合接口验收不能跳过。

### 规范增量

以下均为拟议，实施后 review 冻结、merge 写回；本次不改 canonical Specs/ledger。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs | r3 处置 |
|---|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/architecture.md | memo prop/env 并集与未定义缺省 → 三态优先级、局部 epoch/组件局部态失效及 Degrade 诊断 | 默认面扩展须正确 | AC-01/02/03/12 | **沉淀**（已交付） |
| SD-02 | modify | docs/specs/auto-lang/ui/architecture.md | ADR-19 陈旧重放文字、ADR-24 I/O parked → 当前身份契约 + deferred Init 真完成/取消、显示通知与 dirty 接线 | 不把接受段当完成 | AC-04/05/06/07/11 | **移出→711** |
| SD-03 | add | docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md（新） | 无现行加载契约 → 已验证 memo/失效/热点观测边界（S 档），链接 proposed 调度设计（711） | current-state 与目标态分离 | AC-01/02/03/09/12 | **沉淀**（按已交付面收窄） |
| SD-04 | modify | docs/specs/auto-lang/vm/architecture.md | ADR-23 仅 I/O 段化 → UI CPU Runnable、累计安全预算、帧/栈/RC及 legacy 边界 | 长 CPU 可续跑而非静默完成 | AC-10/11 | **移出→711** |

## 6. 测试设计

### 基线与探针（有界决策）

T-00 最多一轮完整调查加一轮纠正复测，产出 `reports/708-baseline.md` 与 `reports/708-decision.md`。固定 release 二进制 SHA、源码/语料 commit、Windows/CPU/GPU/分辨率/DPI/刷新率、MCP/F12/热重载选项，先记录 warm-up，再采样。帧屏障、单 native 成本、执行栈/写序/707 配对无法确认时记录阻塞，并停止依赖的 M 实施，基线/文档仍可完成。

不以 MCP 请求耗时等同 UI 响应；UI 侧时间戳记录：输入接收、handler/Init 每片起止、computed/build/codegen/convert/layout/draw/present、首次骨架与终态、MCP sync、state/view 版本。统计最大连续占用与不同 App 公平性。

### 运行用例

| 用例 | 覆盖/预期 |
|---|---|
| Row/DataTable 无 Init 页 | 冷构建和复访归因；不得通过延后 Init 虚报收益 |
| AreaChart 计算型 child | 骨架显示→片间交互→完整图形；首发与复访一致 |
| FileTree computed | 冷 miss 与展开后失效；不得缓存旧行或忽略计算耗时 |
| 长 CPU 首段及 async 后 CPU 恢复段 | 都切成 Runnable，计算结果/副作用次数等价同步参考 |
| nested call/closure/try-catch | 预算横跨栈帧，最终返回/异常/RC纪律不变 |
| 重复 view/MCP + 无 Init 父页 | Init 一代际一次；发现真实 child，不永久 loading |
| A→B→A/key、失败/reload/unmount/close | 旧任务不续跑；等待资源、busy/队列/栈一次清理 |
| 两 App + 128 项队列边界 | 重计算 App 不阻断另一 App，队满可观察，无饥饿/无限排队 |
| env/prop 完整矩阵 + preview/nav/theme/popover | 缓存开/关结果一致，显式 false 与缺省区分，pending 不永久命中 |
| 首帧无输入、resize/surface retry、最小化恢复 | 主动唤醒；代际/窗口对应正确；真实屏幕与 MCP 更新 |
| 707 wait→CPU resume→取消 | 对最终接入的所有 wait 种类逐种验证，不遗漏 stream 清理 |

### 验证门禁

- drafting/replan：内容、链接、ID/映射检查；无 Rust 改动，不运行 Cargo/docs_gen。
- work：`cargo check -p auto-lang`，以及 `cargo t plan708` / 相关 ui、vm_bridge 模块；单测命名纳入计划族，可检查有意义的生命周期和执行结果。
- 最终 review/fold：因 T-11 修改 VM 执行契约，运行一次 `cargo tf`，它已包含 corpus；可选 tv 仅为定向快捷，不要求重复 tf+tv。
- 不触发 aavm 代码时不跑 taa；不改 schema/文档生成器/语法参考时不另跑 docs_gen。
- gallery VM 实机复用 autoui-verifier；涉及共享 .at 语料的改动才追加 Vue 相关终态核查。MCP 开/关是测量对照，正式验收包含正常 MCP 开启。
- 运行期/独立复审前实现提交需 clean/绑定 commit；原计划编写者的自检不能代替独立实现验收。

## 7. 验收标准

### r2 拟议性能参数

这些是待确认的契约参数，**不是当前实测**，T-00 不得自行降低：

- 60Hz 的 16.7ms 单帧预算保留为优化目标；切页输入到首次骨架真实呈现：p95≤100ms、max≤250ms。
- 正常加载期间连续 UI 线程占用 max≤50ms；窗口/侧栏输入到可见反馈：p95≤100ms、max≤250ms。区分接受反馈与 VM 业务动作完成。
- 每页首访和复访各≥20次：Row、DataTable、AreaChart、FileTree、Home；冷启动≥5次，单独报告装载/开窗前时间。每轮加载中注入滚动/切页/resize，不能等就绪后才测。
- 长 CPU/async-resume 夹具至少运行≥2s（同步参考），有多个任务的持续压力≥30s；至少两个 App 同时运行。
- 报告 p50/p95/max、完整就绪时间、memo hit/miss/degrade、native 最大耗时、资源/队列峰值。优化后完整就绪 p95 不得劣于同配置基线 25%以上，避免用无限延迟换 UI 指标。
- 超限若因系统调度/GPU等外部噪声，必须留独立证据并复测；不删坏样本或静默放宽。不满足为 fail/needs_replan。

| ID | 可观察标准 | 任务 / 验证 | r3 处置 |
|---|---|---|---|
| AC-01 | prop 缺省下 outlet memo 生效，重复稳定页不全量重建；开关行为等价 | T-01/02/08，hit/full 计数及状态/画面 | **pass**（plan708_bare_default_on + 实机 hit 计数） |
| AC-02 | env=0 覆盖 prop=true 强制关；完整三态矩阵符合 §5 | T-01/08，12 组合含其他值 | **pass**（9 组合矩阵 + env 四形态解析测试） |
| AC-03 | gallery 既有 sidebar/outlet 开关保持有效，路由切换未变侧栏组复用；收益由热点报告量化 | T-02/06/09 | **pass**（bench A/B bare7=0ms vs cards7=9.4s；实机复测矩阵 baseline §3） |
| AC-04 | 显示及 MCP build 均不派发 child/page Init，重复 build 同代际只登记一次 | T-03/08，派发计数/栈追踪 | **移出→711** |
| AC-05 | 通知证明骨架交付后启动 Init；一次完成正确传播至真实 display/MCP dirty，首次无输入也推进 | T-04/06/08/09 | **移出→711** |
| AC-06 | 重页首帧与加载期间窗口/侧栏交互满足上述 r2 提议门禁 | T-00/05/09，UI 侧时间线及在屏证据 | **移出→711**（S 档实测：冷 27-365ms，门禁全达标路径待 711 复核） |
| AC-07 | Init/准备任务真正终结后内容完整；async、嵌套、失败/取消不无限 loading，不写回旧代际 | T-03/05/08/09/12 | **移出→711** |
| AC-08 | L 正式设计已登记，线程/状态/快照/队列/取消/退出/parked 边界完整，proposed 与现状分离 | T-07/10，设计审查 | **移出→711**（设计文档已存在，711 承接） |
| AC-09 | 分段基线可复现，Row/DataTable/Init/computed/构建/渲染/MCP成本区分，根因由数据支撑 | T-00/06/09 | **pass**（baseline §3.1-3.2 判别实验+复测矩阵） |
| AC-10 | 长 CPU 首段/resume 可续跑；闭包/异常/参数/副作用/RC与参考一致，累计安全护栏不绕过 | T-11/08/09 | **移出→711** |
| AC-11 | 总泵有界、公平；取消/队满/关窗/重载与 707 wait 一次清理，无旧任务恢复 | T-03/04/08/09/11 | **移出→711** |
| AC-12 | preview/nav/props/params/theme/probe/reload失效正确；长 computed 或冷构建热点不逃过 UI门禁 | T-01/02/08/09/12 | **拆分**：preview/nav/epoch 失效面 **pass**（epoch 五臂+测试）；computed 热点部分 **移出→711** |

## 8. 执行步骤

Worktree 复用：`D:/autostack/.wt/lang-708/auto-lang` / `plan-708-dev`，先在创建它的 WSL 环境验证 .git 路径、status 与 commit；不得因 Windows prunable 判定而删除。确认 free 且本地改动已记录后，重同步主线和 707 依赖。若需要修复登记，使用可恢复手段，不 prune/delete 未知工作；所有 remove 前必须 wt-guard clean。gallery 若需改语料则使用同组 auto-os worktree，禁止 junction/symlink。

所有任务未实施，T-00 + T-01..12 共13步。阶段内保留稳定 ID；不为让勾选通过删除失败项。

| ID | 依赖 | 文件/符号及产物 | 预期 / AC |
|---|---|---|---|
| T-00（新） | r2 实施范围确认 | 既有 worktree/二进制；renderer/engine/builder 静态与运行探针；reports/708-baseline.md、708-decision.md（新） | 冻结归因、帧屏障、budget/写序/native/707 配对；AC-06/09/10/11/12 |
| T-01 | T-00 | render_outlet_inner、memo_deps、宿主局部 epoch；plan046 测试 | 缺省与三态、失效/Degrade正确；AC-01/02/12 |
| T-02 | T-00/01 | gallery app.at（核查）、gen_previewcard_code、dynamic_view_impl | 既有开关实效与已证明热点减负；AC-01/03/12 |
| T-03 | T-00，T-11 结果接口 | fire_child_init_if_any/outlet、DynamicComponent、VmBridge、session Init 入口 | demand/代际/observer/取消；根-child 顺序；AC-04/07/11 |
| T-04 | T-03/11，帧屏障探针 pass | run_session/update_inner/DesktopSession/通知通道 | 帧后有限 pump、首次唤醒/AppId、公平与dirty；AC-05/11 |
| T-05 | T-03/04 | outlet/child placeholder、真实显示缓存 | 依赖骨架/就绪/error状态，不永久早退；AC-06/07 |
| T-06 | T-00，随 T-01..05/11/12 实施 | AUTO_MEMO_DIAG/P631_PROFILE；renderer/VM片/在屏采集 | 全阶段时间戳、ready代际/占用/资源计数；AC-03/05/09 |
| T-07 | T-00 decision 与已实施阶段 | 本专题设计；拟议 Spec delta（准备草案） | 设计补齐，SD-01..04只描述已验证面；AC-08及所有规范映射 |
| T-08 | T-01..05/11/12 | 原 plan045/046/047/702 同族 + plan708 测试族 | check + scoped；调度/栈/RC/失效/取消红测变绿；AC-01..05/07/10..12 |
| T-09 | T-06/08，707 最终接口 | 标准验证脚本 + reports/708-runtime.md（新） | 样本与加载中交互/多App/终态；AC-03/05..07/09..12；最终一次tf |
| T-10 | T-07/09 | 报告、计划矩阵、Spec冻结包 | 独立实现 review 的 commit/AC/SD/风险清单齐；不是自行reviewed/merge |
| T-11（新） | T-00 | engine::drive_handler_segment、AutoTask、VmBridge段接口/pump | CPU Runnable、首发/resume预算、累计护栏/写序；AC-10/11 |
| T-12（新） | T-00/02/11 | call_computed_fn/call_vm_fn、computed signals、builder热点 | 依赖安全缓存/具名prepare，Ready值/RC/失效；AC-07/12；不能一律同步漏测 |

- [x] T-00 基线/安全探针/decision（reports/708-baseline.md §3/§4 + 708-decision.md D-1..D-6：帧屏障裁定 listen_raw 序障；归因闭环 preview-card 臂 VueGenerator 每实例 ~1.1-1.3s、DataTable 静默 Degrade、FileTree 恒 MISS；env=0 无强制关实证；无 T-00 级阻塞）
- [x] T-01 outlet 三态与失效（worktree 7ac8f51c2：三态门+Outlet Option<bool> 四层贯通+memo:false 新形态+缺省 on；宿主 UI epoch+五处 bump 堵组件局部态 memo 陈旧缺口；Degrade 诊断面；plan708 5 测试全绿+既有族零回归）
- [x] T-02 gallery 既有 memo 核查及热点减负（worktree 956d86a90：归因实证=preview-card 每卡每帧 VueGenerator::new ~1.27s；WidgetRegistry::with_defaults 进程级 OnceLock+boot 后台预热；实测冷构建 row 8.1s→365ms/datatable 9.6s→27ms/area-chart 1.2s→1ms/filetree 1.4s→2ms，warm 全 0-1ms；DEGRADE 诊断行实证生效；a2vue 预存红+plan358 并行抖动定责零回归）
- [x] T-06（**部分交付**，余量→PLAN-711）：DEGRADE 诊断面 + AUTO_MEMO_DIAG 全链实测可用（本计划范围）；分段时间戳/在屏采集/资源计数随 M 档泵移 711
- [x] T-07（**收口于已交付面**）：设计文档已登记（docs/design/autoui/vm-loading-responsiveness.md）；拟议 Spec delta 按已交付行为收窄（SD-01/SD-03 沉淀；SD-02/SD-04 移 711）
- [x] T-08（**已交付面**）：scoped 回归=plan708 6 + 045 13 + 046 38 + memo 91 + outlet 12 + parser outlet；阶段门禁 tf no-fail-fast 5891 执行零回归（12 预存红+4 并行抖动+1 环境中止逐例定责，见 §9 阶段复审）
- [x] T-10 证据与复审交接：复审记录+落地收据在册（§9），报告 708-baseline/708-decision 固化
- [ ] T-03 Init demand/代际生命周期（**r3 移出→PLAN-711**）
- [ ] T-04 真实入口帧通知与有界泵（**r3 移出→PLAN-711**）
- [ ] T-05 骨架/完成/失败显示（**r3 移出→PLAN-711**）
- [ ] T-09 VM 实机性能/终态 §7 全矩阵（**r3 移出→PLAN-711**；S 档实测数据已固化 baseline §3）
- [ ] T-11 CPU 可续跑执行片（**r3 移出→PLAN-711**；engine 假成功缺陷一并移交）
- [ ] T-12 computed/冷构建预算残面（**r3 移出→PLAN-711**；DataTable memo_block Degrade 根因与 FileTree 恒 FILL 随迁）

## 9. 复审记录

- `stage: new`, PLAN-708 r1, `outcome: pass`, `next: **wait for external review**`
- 执行门禁：外部 AI 审过本契约（尤其 §2 三档边界、§5 M-01/M-02 Init 两拍、
  §10 Q-01/02）并给出 pass / needs_fix 后，才进入 `/auto-plan:work`。
- 当前**零实现提交**；勿在 review 前动 crates/。

### 2026-09-29 独立实施方案审查（r1）

- `stage: review`（实施前设计检查，非实现验收）| `plan_id: PLAN-708` | `plan_revision: 1` | `outcome: needs_replan`。
- `reviewed_commit/base_commit: 66c9cac193ad1a674be521e61440fcf5c65145e4`（现行代码依据）；708 分支 `ec5adb7af02450f7bf2a8f0ea1543859b385ecbc`；gallery 依赖 `93050a6a19231238cd5d4478909e9d6efd30be95`。
- `spec_inputs`、原拟议规范增量快照、AC→任务→证据及发现 R708-01..08 见 [完整审查报告](reports/708-design-review.md)。未运行 Cargo 或实机性能实验，无实现验收 pass。
- 关键问题：gallery 已开启两类 memo；无时间片 Init drain 仍堵 UI；update 末尾不保证帧已上屏；缺少 Init pending/parked/取消状态及缓存失效约束；env=0 公式无法覆盖显式 prop=true；缺少加载期交互延迟门禁。
- `next: /auto-plan:new 修订为 r2 后重审`。保留 `drafting`、既有任务与 AC ID，当前审查不授予按 r1 开工许可。

### 2026-09-29 replan（r2）

- `stage: new` | `plan_id: PLAN-708` | `plan_revision: 2` | `outcome: blocked`（修订已完成；新增 VM 执行契约与 r2 门禁需在行为实施前确认，独立方案检查待做；不代表实现缺陷或实现验收）。
- changed：T-01..10、AC-01..08、SD-01..03；added：T-00/11/12、AC-09..12、SD-04；历史 r1 needs_replan 保留，不覆盖未来版本。
- 新增架构设计及索引：`docs/design/autoui/vm-loading-responsiveness.md`；不发布 canonical Spec/ledger。
- R708-01..08 的契约处置与自检映射见 [r2修订检查](reports/708-r2-contract-check.md)。本轮由修订者自检，明确不满足最终独立实现 review。
- `next: 确认 r2 新增执行契约/门禁并进行独立方案检查 → work T-00`；技术探针失败时返回 needs_replan。保持 drafting、current_step=0；不沿用 r1 pass 开始旧方案。

### 2026-09-30 work 开工（T-00 前置）

- `stage: work` | `plan_id: PLAN-708` | `plan_revision: 2` | `outcome: in_progress` | `code_commit: 无实现（worktree 同步至 e1bab972eb7ac3fcda8cc7bd8793f2791f0a04c6）`
- 用户明确调用 `/auto-plan:work` 实施本计划 → 作为 r2 新增 CPU 段契约/串行纪律/§7 门禁的开工确认（Q-06），status drafting→executing。
- worktree 修复收据（可恢复、零删除）：`.git/worktrees/auto-lang2/gitdir` 与 worktree `.git` 两文件由 `/mnt/d/...` 改写为 `D:/...`；修复后 `git worktree list` 不再 prunable，branch=plan-708-dev、status clean（零未提交改动）、`master..HEAD` 空 → `git merge --ff-only master` 同步至 e1bab972e（含 707 全部落地接口）。
- 主检出预检：仅 docs/簿记与 `.tmp-vm-*` 走查残留，无 crates/test 代码 WIP，符合 master 零 WIP 代码规则。
- `next: T-00 静态/运行探针 → reports/708-baseline.md + 708-decision.md`；探针失败项按 §6 有界处置。

### 2026-09-30 work T-01/T-02（S 档完成）

- `stage: work` | `plan_id: PLAN-708` | `plan_revision: 2` | `outcome: in_progress` | `code_commit: worktree 7ac8f51c2（T-01）+ 956d86a90（T-02）`
- T-01：三态表/缺省 on/UI epoch 五臂/Degrade 诊断面落地；scoped 门禁 plan708(5)+plan045(13)+plan046(38)+memo(91)+outlet(14)+parser outlet 全绿。
- T-02：判别实验四组锁定 preview-card 每卡每帧 `VueGenerator::new()`（跳过代码生成仍 9.0s）；`WidgetRegistry::with_defaults` 进程级缓存 + run_session 后台预热；实机复测冷构建 row 8.1s→365ms / datatable 9.6s→27ms / area-chart 1.2s→1ms / filetree 1.4s→2ms，warm 0-1ms；T-01 的 DEGRADE 诊断行实机可见（DataTable reason=memo_block）。
- 预存红定责（零回归）：`musk_vm_track_p053_6_widget_content`（base e1bab972e 同败，707 台账 musk flaky 族）、`test_a2vue_desktop_surface_asset`（base 同败，707 台账 a2vue）、`plan358_d1_for_style_if_msg_on_stress`（并行抖动，带改动单跑 2/2 过）。
- T-02 残面（移交后续任务）：Row 冷 365ms 仍超 50ms 占用门禁（真实渲染工作，M 档骨架/泵与 T-12 继续压）；DataTable memo_block Degrade 根因（T-12）；FileTree 恒 FILL（T-12 computed 缓存）。
- `next: T-11 CPU 可续跑执行片 → T-03 Init demand/代际 → T-04 帧通知泵 → T-05 骨架显示 → T-06/07/08/12 → T-09 实机矩阵 → T-10 交接`。

### 2026-09-30 阶段复审（S 档：T-00/01/02 增量落地前）

- `stage: review`（**阶段复审**——整体计划保持 executing，不授予终审 reviewed）| `plan_id: PLAN-708` | `plan_revision: 2` | `outcome: pass`（阶段范围）。
- `reviewed_commit: worktree plan-708-dev @ 956d86a90`（T-01=7ac8f51c2 + T-02=956d86a90）| `base_commit: e1bab972e` | dependency: gallery `93050a6a`（auto-os，干净）/ auto-down `3373a5cc6e`（detached 兄弟）。
- 复审者=实现会话本身（独立性受限声明）：裁定从工件重建——重读两提交全量 diff（8 文件 +454/−33，唯一新增 eprintln 为 AUTO_MEMO_DIAG 门控诊断行）、失败逐例 base 检出复跑定责。
- 阶段门禁：`cargo tf --no-fail-fast`（排除 `default_headers_reach_wire_on_plain_get`——707 台账在案预存环境红，本机实测悬挂 >78min）= **5891 执行，PLAN-708 改动零回归**。预存红 12（base 同败或 707 台账在案）：musk p053/p054 ×6、a2vue ×1、projector ×1、plan606_gallery ×1、docs_gen ×2（core_reference/kitchen_sink，base 同败）、default_headers ×1。并行抖动 4（带改动单跑全过）：p508_g2_outproc_arm、clipboard_files_and_image、plan358_stress、plan707_wait_generator。环境中止 1：plan705_e2e_deadline（单跑 2.88s 过；tf 中止系复审中重复实例事故的僵尸进程占用 18511/18512 端口，已清理）。
- AC 映射（阶段范围内）：AC-01/02/03/09 → **pass**（plan708 5 测试 + 045/046/memo/outlet 既有族 + 实机 A/B 数据）；AC-12 → **partial**（preview/nav/epoch 失效面已闭，"冷构建热点不逃过 UI 门禁"未达——Row 残余 365ms，映射 M 档 T-03..05/T-12，非未授权延期）；AC-04..08/10/11 → M 档任务未实施，不评。
- 发现：F-1（低）非法 outlet 头参 parse-error 路径无直接测试——修正路由 T-08 测试族。fmt 漂移为仓库存量（base 同检确认），零新增 debug 残留。
- Spec delta：SD-01..04 维持 proposed，不随阶段落地发布；canonical Specs 与 ledger 本轮未动。
- 证据持久化：门禁全量日志 `tmp/708/tf_gate.log`（worktree 内易失）——结论摘录已固化于本记录；实机数据固化于 708-baseline.md §3 与 dfb2bf223 簿记。
- `next: 阶段落地 plan-708-dev → master（ff-only），计划保持 executing，M 档（T-11→T-03..T-10/T-12）续作`。


### 2026-09-30 阶段落地（S 档 landed）

- `stage: merge`（**阶段落地**——checkpoint `landed`；整体计划保持 executing，终审合并（Spec 沉淀/ledger/归档/清场）未触发：10/13 任务未完成、SD-01..04 维持 proposed）| `plan_id: PLAN-708:r2` | `outcome: pass`（阶段范围）。
- rebase 收据：plan-708-dev 于 master 并行推进（706 收据 + **709 全量实现与归档**，另一会话在 708 门禁窗口期落地）后重放；旧→新映射 `7ac8f51c2→1c26e70b7`、`956d86a90→f169ad42f`；`git range-diff` 两行全 `=`（补丁等价，safe-rewrite proof）。
- 合并态定向刷新（708/709 同触 renderer.rs）：cargo check 零错误；plan708 6/6、plan045 13/13、plan046 38/38、outlet 12/12 全绿。
- landed：`git merge --ff-only` master tip == plan-708-dev tip == `f169ad42fc6bc63f213797b72759dcfa1477e2e9`（零合并提交）；树与已验证 worktree tip 逐字节一致（crates/ 面同一棵树），scope 验证即该树上的实跑结果。
- 未做（按入口门禁本就不适用）：canonical Spec 沉淀、ledger 刷新、归档、wt-guard/清场；worktree `D:/autostack/.wt/lang-708/auto-lang`（auto-down 兄弟在组）保留供 M 档续作，分支与 master 同点。
- `next: M 档 T-11 → T-03..T-10/T-12 续作于同一 worktree；完成后终审 review → 终审 merge`。

### 2026-09-30 r3 修订（收窄，用户裁定）

- `stage: new` | `plan_id: PLAN-708` | `plan_revision: 3` | `outcome: pass`（修订自检；scope 变更已获用户授权）。
- **授权**：用户 2026-09-30 明示"计划 708 现在 review+merge，M 档单独立项"——r2 的 M/L 档契约移出，AC-04..08/10/11 随任务转 PLAN-711（新立项 vm-loading-scheduling，drafting，承接设计文档与 T-03/04/05/09/11/12 + T-06 余量 + SD-02/04）。此为 scope 缩减而非完成声明；不存在未授权延期。
- 收窄依据（实测）：T-02 后冷构建 27-365ms/复访 0-1ms，原始 10-30s 病灶消除；骨架/延迟 Init 的设计动机被移除。
- 修订影响面：任务清单（T-03/04/05/09/11/12 标记移出）、AC 表（新增 r3 处置列）、规范增量（SD-02/04 移出；SD-01/03 按已交付面收窄）、frontmatter（supersedes 收窄至 ui/architecture.md，touched_goals=[GOAL-007]）。

### 2026-09-30 r3 复审（终审，收窄后契约）

- `stage: review` | `plan_id: PLAN-708` | `plan_revision: 3` | `outcome: pass` | `reviewed_commit: f169ad42fc6bc63f213797b72759dcfa1477e2e9`（=master，S 档已落地态）| `base_commit: e1bab972e`。
- 复审独立性受限声明（实现会话自审）：裁定从工件重建——r3 契约逐条对证：AC-01/02/03/09 + AC-12 失效面全部绑定已固化证据（plan708 6 测试 + 阶段门禁 tf 5891 执行零回归 + baseline 实测矩阵 + 落地收据 range-diff 等价证明）；移出项核对 PLAN-711 骨架承接完整性（T-03/04/05/09/11/12 + T-06 余量 + SD-02/04 + engine 假成功缺陷登记）。
- 代码自 f169ad42f 未变更（仅 docs/簿记后继提交）；阶段复审证据（2026-09-30）对本基线持续有效，复用理由=同一代码树+同一测试配置。
- 发现：F-1（低）沿用阶段复审登记，随 711 的 T-08 收口；无新增发现。
- Spec delta 冻结：SD-01（modify ui/architecture.md：memo 三态/env 强制关/宿主 epoch/组件局部态失效/Degrade 诊断）+ SD-03（add ui/design/vm-loading-responsiveness.md：已验证 memo/失效/热点观测边界 + 链接 711 proposed 调度设计）；SD-02/04 不在本次沉淀范围。
- `next: merge（沉淀 SD-01/03 + ledger + 归档 + 清场）`。

### 2026-09-30 merge 收据（PLAN-708:r3）

- `stage: merge` | `outcome: pass` | `completion_kind: delivered`（r3 收窄面）| delivery `a39ef6827`（ff-only，rebase 映射 1df3c0f30→a39ef6827 range-diff 全等；docs-only descendant of reviewed f169ad42f，delta 核对实现/依赖零变更）。
- checkpoint 链：`prepared`（SD-01/SD-03 冻结于复审记录；711 骨架承接核对于 r3 记录）→ `landed`（a39ef6827，master==分支）→ `ledger_refreshed`（P708-1 designs/P708-2 reviews 外科插入；回读断言新条目在位且 P707/709 既有零扰动；INDEX 再生 26 projects）→ `archived`（本文件，git mv + status archived）→ `cleaned`（worktree/分支/组目录 wt-guard 后移除，收据随下一步提交补记）。
- canonical：`docs/specs/auto-lang/ui/architecture.md` ADR-26 + `docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md`（新）+ `ui/plans.md` 708 行；SD-02/04 未沉淀（711 范围）。
- 生产面观测（landing ≠ deployment）：主检出 release 二进制未随本次重建（worktree 探针用二进制为同源代码构建）；下次消费方构建自然刷新——显式登记，不声明已部署。
- 并行会话窗口记录：本次 merge 期间 master 三度被并行推进（709 归档/706 复审），均以 rebase+range-diff 全等等价证明处理；无冲突。
- `next: 无（终态）；PLAN-711（vm-loading-scheduling，drafting）承接 M/L 档`。

## 10. 待澄清事项

| ID | 项目 | r2 处置 / owner |
|---|---|---|
| Q-01 | 骨架样式 | 默认轻量 Loading…；保留侧栏与窗口交互，终态基线不变；T-05 |
| Q-02 | drain 时间片 | 已改为有界首发+resume；初值4ms片/8ms轮次/4096指令/64步查时钟，T-00验证，超限不能整段fallback |
| Q-03 | L 是否单独实施 | 是；本计划只交付正式proposed设计，后续另立Plan |
| Q-04（新） | 首帧成功呈现屏障 | T-00 bounded spike 冻结；frames/sleep/tick都不自行视为证明；无可行屏障则needs_replan |
| Q-05（新） | CPU片与共享堆/事件写序 | T-00核查并冻结同App有界串行纪律/部分状态观察矩阵，T-11验证；不裁定Design34全局store模型 |
| Q-06（新） | r2性能阈值及新增VM契约 | 提议值在§7；需进入行为实施前确认此r2，不能冒称r1已批准，T-00不得降低 |
| Q-07（新） | 707等待/取消接口 | T-00记录当前revision；最终review按已落地commit联合检查，不抹掉并行工作 |
| Q-08（新） | computed/native/编译/布局超限 | T-00实测、T-02/12处理；若越过S/M边界则needs_replan，不能延期后判通过 |
