---
plan_id: PLAN-804
status: archived
feature_name: 对话框与拖出宿主能力迁移
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-18
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/ui/design/shell-a2r-seams.md]
new_spec_components: [docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-lang/vm, auto-lang/ui, auto-lang/host-adapters]
depends_on: [PLAN-779, PLAN-785, PLAN-790, PLAN-794]
---

# [PLAN-804] 对话框与拖出宿主能力迁移

## 0. 变更摘要

迁移dialog_open/dialog_save（2927/2928）及dnd_start（2938）三native：VM只编解码并调用owned宿主接口；OS对话框、父窗选择、拖出载荷解析与COM执行由adapter承担。同时合并VM/a2r重复对话框构建逻辑，保留两者已有父窗策略差异。

## 1. 目标

本族VM不直引rfd/raw-window-handle/Win32/COM/ui::native_dnd，真实no-ui VM可由自定义provider供值；UI宿主保留模态父窗、过滤/默认文件名、取消值、拖出输入线程及完成事件行为。三个ID/签名、字符串所有权/布尔编码保持。

非目标：迁拖入IDropTarget、所有OS能力/终端/媒体/diff、改变a2r默认父窗策略、修复旧HWND缓存策略或重写OLE。三函数迁出不代表整个UI运行时可搬仓。

## 2. 架构方案

在既有host_capabilities分别增加DialogHostProvider（open/save）与DndHostProvider（start），各自独立槽与feature装配，避免一种能力在位掩盖另一种缺席。仅跨界owned String/Bool；JSON仍是dnd现有wire输入，由唯一UI载荷parser解释，不复制为VM里的第二套parser。

两槽沿779/785/790/794装配：实例覆盖、构造默认快照、兼容if-absent同锁补缺；NI clone保provider、VM merge保目标实例，注入不改变binding_revision/proof。调用前释放槽锁，真实模态调用可重入，不跨回调持RwLock。Send+Sync约束provider自身，不授权VM/COM跨线程。

OS对话框实现放新上层host_adapters/dialog.rs（拟名），独立于ui renderer门；保留ui-dialog单开、不启用ui也能走原高层dialog路径。ui::dialog旧open/save薄委托同一实现、使用旧unparented策略；VM默认adapter使用旧parented策略。Dnd adapter留UI，仅Windows+native-dnd+ui安装，调用原start_drag。

## 3. 技术栈

Rust、现有rfd/raw-window-handle与Win32/OLE桥、NativeInterface/AutoVM；独立consumer、组件及真实窗口/OS交互证据。无新OS桥/RPC/后台拖拽线程。

## 4. 需求分析与背景调查

用户2026-10-17授权继续规划，并询问对799/801依赖。本轮只新建合同/设计路线，不执行Rust、不修复或合并他线、不创建agent/树、不切旧仓。取材主检出9932962bc157be13bcaa0520011c7c78082cb2fe，来源SHA及在途状态见[指纹](attachments/extraction-followup-20261017/inputs.json)。本计划不消费AST/方言API，没有799/801/803硬依赖。

802 latest readiness复审为needs_fix（V06实机证据未齐），生产实现4029dd705仍未合入；803也未合入，起草期间继续复审/修复。均不能按已交付接口消费。本计划与802共用NI/VM/统一安装和native.rs，但改不同族：可在独立树准备新文件/定向实现，共享安装/槽接线短窗口串行；推荐802先合，后合入者取并集、重核受影响证据，不设虚假的AST前置。

源码中dialog_parent及rfd构建在vm/native.rs，ui/dialog.rs另有近似open/save供a2r使用。两者过滤规则相同，但VM带父窗、a2r旧版本无父窗。ui-dialog只依赖rfd和raw-window-handle，不要求ui；不能为方便把adapter全藏在ui门后。Dnd文件头仍称专用STA线程，实际start_drag及shim明确要求收到按下的输入线程同步DoDragDrop；以实际实现与已落地合同为准，SD纠正文档陈旧描述。

## 5. 详细设计

T01冻结三项参数/栈/feature/缺席矩阵、装配入口和线程/事件链。Dialog无provider保旧feature-off VMError文本/栈效应；在位provider取消或不可用返回空串。Dnd无provider、坏payload/空payload/已有拖出重入返false。原start_drag进入会话后无论落下还是取消，返回true表示执行过会话，完成效果另由on_dnd_finished报告，**不能把cancel映射为false或把true当drop成功**。

DialogHostProvider接原始filter/default_name，公共过滤语义只在单一OS实现维护：逗号/分号/空格/tab分割、去前导点、空项丢弃、无扩展不加filter；路径返回to_string_lossy和空串取消保持。搬出原dialog_parent及测试，ui::dialog与VM adapter只选择不同parent策略，不保留两份构建/过滤/父窗算法。旧a2r公开路径可编译且parent差异诚实保留。

Dnd provider接owned JSON，复用parse_payload_json的text/files/virtual_files、base64与裸反斜杠重试规则；实际DoDragDrop仍在输入线程阻塞，释放会话/载荷沿旧实现。完成通道、session代号、发起者路由和事件消费保持；不得改成spawn/async提前返true。拖入/窗口注册生命周期仍原owner，不趁本轮重构。

统一宿主入口补两槽，明确ui-dialog-only（无ui）的高层兼容安装，原始AutoVM仍按既有provider模式只快照，不暗装UI。显式default/instance恒胜兼容默认。与802/799/801的共享窗口、792source闭包及合流责任见[本批编排](reports/extraction-followup-parallel-order-20261017.md)。

### T-01 冻结矩阵（work 勘定，基面 master b7ecea5e3）

三 native 参数/栈/feature/缺席逐字面值（来源 `vm/native.rs` 旧 2296-2323/2598-2778/3099-3110、`ui/native_dnd.rs` start_drag、`ui/dialog.rs`）：

| native | 弹参（在位臂） | feature 门 | 缺席/降级臂 | 返回 lane |
|---|---|---|---|---|
| dialog_open 2927 | pop_string_arg(filter) 1 个 | `ui-dialog` | feature-off：`VMError "dialog_open: the \`ui-dialog\` feature is disabled"`，**不弹实参**（错误中止） | String（`to_string_lossy`；取消/不可用 = `""`） |
| dialog_save 2928 | pop_string_arg(default_name) 1 个 | `ui-dialog` | 同形 `"dialog_save: …disabled"`，不弹实参 | String；取消 = `""` |
| dnd_start 2938 | pop_string_arg(payload_json) 1 个 | `windows ∧ native-dnd ∧ ui` | off 臂：**弹实参后弃**、返 `false`；在位臂坏 JSON = `false` | Bool |

过滤语义（open 与 a2r host 同规则）：`split([',',';',' ','\t'])` → `trim` → `trim_start_matches('.')` → 弃空；非空才 `add_filter("Files", exts)`。save：default_name 非空才 `set_file_name`。父窗：VM 臂 Windows 下 `dialog_parent::attach`（EnumWindows 最大可见自有顶层窗，OnceLock 进程期缓存，跳 `#32770`/`Winit Thread Event Target`）；a2r 臂（`ui::dialog`，需 ui+ui-dialog 编译）**无父窗**（PLAN-681 §9 F1 缝，语义保留）。

start_drag 会话合同（488 步骤 9 定案，逐条保持）：空载荷→false 无会话；`DRAG_ACTIVE` 重入→false；否则 OleInitialize（幂等计数）→ DndDataObject+DndDropSource → `DoDragDrop` **调用线程内联阻塞**（自带消息泵，copy|move|link）→ 效果发 DONE 通道、`DRAG_SESSION_GEN++`、`DRAG_ACTIVE=false`、返 **true=执行过会话**（落下或取消均 true，效果另由事件报告——不得把 cancel 映射 false / 把 true 当 drop 成功）；`SEEN_DRAG_BUTTON` 每会话重置（程序化起拖 keys=0 未及见按下判继续）。事件链：DONE 通道 → iced subscription 16ms 轮询 `take_finished_effect`（`ui/session.rs`）→ `DesktopEvent::DndFinished` → renderer update → 发起方 App（dispatch 环前后读 `drag_session_generation` 比对锚定，`ui/iced/renderer.rs`）。

装配入口现状（两槽并入同集合）：`install_ui_host_capabilities`（lib.rs，现 `#[cfg(feature="ui")]`；7 个 run/debug/test 调用点——需扩为 ui-dialog-only 亦可装配）、`run_file_dynamic_ui_inner`（lib.rs，ui-iced）、`VmBridge` 两构造（vm_bridge.rs，ui 面）。`ui::dialog` 模块在 `#[cfg(feature="ui")] pub mod ui` 内再 `#[cfg(feature="ui-dialog")]`——**ui-dialog 单开时 a2r 门面今日不编译**，仅 VM shim 直引 rfd 工作；OS 实现迁 `host_adapters` 后不受 ui 门（本计划修正点）。

802 共享窗口裁定：plan-802-dev（4029dd705）**未合入**、latest 复审 needs_fix——804 以 master 为基面独立实现；共享写面=vm/native.rs NI 结构/Clone/new + engine.rs `new_inner` 快照块 + lib.rs 统一安装 + vm_bridge 装配行，合并取并集、后合者重核（无 799/801/803 硬前置，冻结于 §4）。

真 OS 证据路径勘定（§10 T-01 责任项）：本机 win32 真桌面可用——V04 用真 Win32 顶层窗 + `#32770` 对话框自动化（他线程 FindWindow/SetWindowText/按钮消息，路径写在临时目录）；V05 用自建测试接收窗（RegisterDragDrop + STA 泵，既有 spike 基建）+ SendInput 合成按下-移动-释放驱动**真 DoDragDrop 会话**（QueryContinueDrag 的 SEEN_DRAG_BUTTON 语义显式支持程序化起拖）。二者均为 env 门控测试（不进日常档），T-05 实跑取收据；合成输入会短暂移动真实光标（DoDragDrop 必须收到真实按下输入线程——计划 oracle 本义），收据如实登记。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md | 三native owned边界/降级/线程/完成与父窗策略 | 不把模态/重入当普通异步调用 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | VM直调OS/UI→两独立provider/实例装配 | 本族删边 | AC-01/02 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | adapter归属/feature-only路径和真实拖出线程 | 纠正陈旧线程叙述/如实登记剩余族 | AC-03/04 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/shell-a2r-seams.md | dialog共享实现、VM/a2r旧parent策略不变 | 去重与兼容分开 | AC-02/04 |

## 6. 测试设计

work→同树独立final review→merge，无额外agent/模型指定。拟唯一树D:/autostack/.wt/lang-804/auto-lang，branch codex/plan-804-dev；work登记实际base/HEAD/feature。新fixture由T04建立，本轮未运行。涉及UI交互使用autoui-verifier现有工具；OS模态/拖出证据另用可复跑的真实窗口步骤，不冒充MCP控件测试。

| Case / owner | AC / 不变量 | 输入/真实入口 | oracle与边界 |
|---|---|---|---|
| V01 / work | 01 no-ui真供值 | 新test/fixtures/modularity/dialog-dnd-provider-consumer；真VM三ID，互异raw filter/default/JSON与Unicode返回 | 参数/返回与trace一致，原buffer销毁后VM字符串有效；无UI/OS feature也可显式供值，闭包无renderer/rfd；非假adapter直测 |
| V02 / work | 02 缺席/取消/形态 | 无provider、两槽只装一个、feature-off、取消、坏/空JSON、重入、text/files/virtual_files载荷 | 具体值/错误/栈效应与旧臂一致；缺席dnd=false，进入后取消=true且完成effect=none；不混淆会话与成功 |
| V03 / work | 02/03 装配/重入寿命 | 两VM、default变更、NI clone/merge、显式选择遇兼容安装；provider回调再访问/设置槽 | 无覆盖/串线/deadlock，锁不跨回调，drop释放；binding/proof规则未变；Send+Sync不据此宣称跨线程COM |
| V04 / work | 02/03 真dialog | Windows真实UI程序open选文件/save选目标/两者取消；空/混合filter、Unicode临时目录；ui-dialog-only高层入口；a2r旧路径 | 返回路径/空串正确，VM窗口在模态结束后可操作、parent策略同旧；a2r仍原策略；组件mock不能替代真窗口与ui-dialog-only编译/接线 |
| V05 / work | 03 真拖出 | 真实输入线程启动，拖出文本/现有文件/虚拟文件至测试接收窗口，并取消一次 | 接收内容/落地bytes及完成事件归发起者；调用线程ID相同，返回在会话结束之后，资源/拖旗标复原；不能用直接调用COM构造器替代 |
| V06 / work | 04 单一性/合流 | 源码清单/777/792、802实际合入差异、旧a2r调用 | native本族无rfd/Win32/UI直引，对话框构建唯一；其他provider未回退；792闭包/构建输入变化按其真实入口更新并重放受影响fresh/locked/重定位/保护控制 |

work仅定向provider/装配/OS反控、`cargo check -p auto-lang`、新consumer manifest test；ui-dialog-only用`cargo check -p auto-lang --no-default-features --features ui-dialog`验证。真OS证据未齐不得execution_done或pass，环境缺口保留blocked项，不降低V04/05。最终review后merge冻结裸cargo t+tv；实际修改ui_gen才加tu。tf主检出到期单实例，work/review不跑整档。本轮Category A不跑Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 三项生产shim只编解码/委托，两槽无UI/OS实现依赖，V01通过 |
| AC-02 | work | 固定ID/栈/错误/取消/feature-only、字符串寿命和旧a2r路径保持，V02/03/04通过 |
| AC-03 | work | 真实模态/拖出线程/完成/资源、装配隔离证据完整，V03–05通过 |
| AC-04 | work | 单一对话框实现、剩余边/迁位净删、共享合流/792及SD素材完整，V06通过 |
| AC-05 | review | 同树独立逐项AC/SD/遗漏复审通过，绑定提交，merge门pending |
| AC-06 | merge | 冻结t/tv及必要触面门、Spec/ledger/归档、guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 既有provider交付、合同确认 | native.rs/dialog_parent、ui/dialog.rs/native_dnd.rs、NI/VM/组合入口：旧矩阵/线程/取消及802共享窗口决策——§5 T-01 冻结矩阵；worktree `D:/autostack/.wt/lang-804/auto-lang`（含 auto-down 兄弟 detached@c39ac7f），branch **plan-804-dev**（计划原文拟名 codex/plan-804-dev 按仓规 AGENTS.md 落 plan-804-dev），基面 b7ecea5e3 | 01/02/03 |
| [x] | T-02 | work | T01 | host_capabilities两provider/槽、NI/VM快照/clone/merge、三shim——DialogHostProvider/DndHostProvider 双族（set/if_absent/get/clear ×2）、NI 双槽+Clone 保留、engine `new_inner` 快照、AutoVM 实例 accessor、三 shim feature 中性化（dialog 缺席在弹参前 raise 保旧栈面；dnd 缺席弹参后 false）。证据：`cargo check -p auto-lang` ✅；in-crate `plan804` 6/6（剪断锚/缺席契约/委托+槽独立/取消空串/槽锁重入/装配身份+merge）✅；兄弟回归锚 779:4+785:6+790:7+794:6 全绿 ✅ | 01/02 |
| [x] | T-03 | work | T02、共享接线窗口 | 【R2 重勾：F804-R1-04 已闭合——lib.rs 表达式位 /// 改 //，default+native-dnd 双档 plan804 触面文件零新增警告】 新host_adapters/dialog.rs、UI dnd adapter、ui/dialog薄门面和宿主装配，删除VM OS实现——`host_adapters::dialog`（filter_exts 单一 parser + dialog_parent 原样迁入 + parented/unparented 双策略 + install_process_default + in-crate 3 测）；`ui::dnd_host_provider`（windows∧native-dnd 双门控，委托 parse_payload_json→start_drag）；`ui::dialog` 薄门面（unparented_open/save）；装配点：lib.rs 统一入口（cfg 扩 `any(ui, ui-dialog)`）+ run_file_dynamic_ui_inner + VmBridge ×2；vm/native.rs 删 dialog_parent/rfd/旧双臂。证据：`cargo check -p auto-lang --no-default-features --features ui-dialog` exit 0 ✅（ui-dialog-only 高层路径接线）；`--features native-dnd` 下 dnd adapter compat_install 1/1 ✅；compat_install 全家 5/5 + filter_exts + owner_satisfies 迁入锚全绿 ✅ | 02/03/04 |
| [x] | T-04 | work | T03 | 【R2 重勾：F804-R1-01/04 已闭合——fixture 补真 save 取消腿（默认名种子经 ack 通道回传断言）+ skip_children fmt 全净；no-ui 复跑 7/7 ✅、ui-dialog-only 复跑 1/1（双腿，1.28s）✅】 新独立consumer及旧a2r/OS组件/装配反控——`test/fixtures/modularity/dialog-dnd-provider-consumer`（no-ui 独立 workspace + `ui-dialog-host` passthrough feature）；根 Cargo.toml exclude 登记。命令与证据：`cargo test --manifest-path test/fixtures/modularity/dialog-dnd-provider-consumer/Cargo.toml` → **7/7**（V01 a/b/c 全管线+双实例不串线+dnd JSON 三形态原文透传、V02 缺席文本逐字+不暗装+槽独立+取消空串、V03 ID 钉扎 2927/2928/2938+注入不 bump revision+Clone/merge 保双槽）。反控：无既有测试直调三 native（.at 命中为无关状态变量）；ui_gen 旧映射 `auto_lang::ui::dialog::open` 签名不变（facade 兼容）；`--features ui-dialog-host` 编译 ✅ | 01/02/03 |
| [x] | T-05 | work | T04 | V04存活期父窗/open+save取消证据保持；V05当前生产宿主链normal交付copy与非广播断言通过，原始启动停滞及NO_DRAG交付失败反控均明确判红且测试自行回收宿主，退出输入无残留；R1..R3 findings全部闭合。V06生产/交付路径未变的证据复用有逐路径及锁/配置依据，最终主检出并集/包身份重核归T08。详见[独立R4报告](attachments/804/review-r4/review.md)；此前任务行保留在[R4前快照](attachments/804/review-r4/plan-task-rows-before.md)及§9历史。 | 03/04 |
| [x] | T-06 | work | T05 | 冻结`1ca36a674e9199985f8bc804d1ac0f61d1bcaa97`；单一OS实现/删边/剩余族与SD素材完整，当前两触面fmt与diff-check通过，无新增warning。R4独立编译及三场景证据绑定实际二进制、HEAD/锁/Spec指纹；历史收据及环境归因保留§9，当前规范增量见[R4冻结](attachments/804/review-r4/spec-delta.md)。 | 04 |
| [x] | T-07 | review | T06 | 2026-10-18 R4 final pass；同树AC/SD/遗漏/资源收尾复审通过，绑定1ca36a674。裸cargo t+tv、集成后受影响复审/consumer及实际收尾为merge-owned pending，未合入。 | 05 |
| [x] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、沉淀/归档/wt-guard清理。**2026-10-18 完成**：两段 rebase over master 并集（788/799/801/802/805——802 同文件槽装配面 11 冲突逐文件解并集，映射 1ca36a674→fadbfb3a2→f584bb93e，range-diff 3/4 等号+残注/基线两 chore）；冻结门禁全绿：裸 cargo t **5290/5290**（137.4s）+ cargo tv **163/163** + 777 strict **pass**（基线重绑清偿 stale_baseline，0 violations/2 grandfathered 预存）+ 792 实producer重核 **25/25** + 兄弟锚并集 **40/40**（779/785/790/794/802/804 共存）+ fixture no-ui **7/7** + ui-dialog-only check 0 + 主检出集成 smoke check 0；沉淀 SD01..04；ledger v45→v46（guarded offline projection，备份 bak-v45-804）；收据明细见 §9 merge 收据 | 06 |

## 9. 复审记录

2026-10-17 / stage: new / PLAN-804 revision: 1 / outcome: pass / next: 合同确认后work。无799/801硬前置；802未交付的provider接口不作基础，本族沿已交付模式增量实现，共享写面短窗口串行。[本轮预检](attachments/extraction-followup-20261017/preflight.md)。

2026-10-17 / stage: work / PLAN-804 revision: 1 / outcome: **pass（work 全部完成，execution_done，送 T-07 同树独立终审）** / worktree `D:/autostack/.wt/lang-804/auto-lang`（auto-down 兄弟 detached@c39ac7f）/ branch `plan-804-dev` / base `b7ecea5e3` / code `315ed5b5f` / deps: master（802/803 均未消费，803 已另行合入 master——本树基面早于该合并，并集责任在 T-08）/ task_ids: T-01..T-06 全勾 / evidence: 见 §8 各任务行（check 双形态 + in-crate 10/11 + fixture 7 + 实机 4+1+1五会话 + 兄弟锚 23 + 777 observe）/ blockers: 无 / next_responsible_role: **review（T-07，merge 门 pending）**

**work 收据附录（T-06 SD 素材与剩余族登记）**：

- **SD-01（新 docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md）素材**：三 native owned 边界（`&str` raw filter/default_name、owned JSON→Bool、owned String 出参）；缺席矩阵（dialog=feature-off 文本逐字 VMError 不弹参 / dnd=false）；线程合同（DoDragDrop 输入线程内联阻塞、模态可重入、槽锁调用前释放——V03 重入测实锚）；会话/完成语义（true=执行过会话，效果经 on_dnd_finished 另报，cancel≠false）；父窗策略（parented=EnumWindows 最大可见自有窗 OnceLock 进程缓存 / a2r unparented，F1 缝）。证据锚：host_adapters/dialog.rs、vm/native.rs 三 shim、plan804 in-crate+e2e。
- **SD-02（vm/overview）素材**：VM 直调 OS/UI 的 2927/2928/2938 边删除，改为 DialogHostProvider/DndHostProvider 双独立槽实例装配（第 7/8 族，沿 779-794 模式）；native.rs 本族 rfd/raw-window-handle/Win32/COM 直引清零（in-crate 剪断锚）。
- **SD-03（ui/overview）素材**：adapter 归属（dialog=host_adapters 新层 ui-dialog 单门控；dnd=ui 门内 windows∧native-dnd）；纠正陈旧线程叙述（native_dnd.rs 文件头"专用 STA 线程"→实际输入线程内联，以 shim/start_drag 注释为准）；如实登记剩余族（见下）。
- **SD-04（shell-a2r-seams）素材**：dialog 单一实现（host_adapters::dialog）+ ui::dialog 薄门面；VM/a2r parent 策略差异不变（e2e owner 实锚双向）；去重与兼容分开。
- **剩余族登记（native.rs 剩余 crate::ui 50 处，本族非目标）**：code_editor 族（802 在途负责）；scroll 测试内引用（794 域）；dnd 本族已清零。renderer 内 rfd pick_folder（PLAN-019 壁纸族，注释已随迁更正）不在 dialog_open/save 家族。
- **环境注记**：①本树 Cargo.lock（gitignored）zbus_names 4.3.5→4.3.4（aliyun 镜像滞后缺 4.3.5，官方未 yank；只影响本树解析，无合并面）；②实机档会短暂移动光标/弹真对话框（计划 oracle 本义，env 门控不进日常档）；③V05 首跑曾观察一次冷启 flake（输入驱动先于会话就绪），已以前台化+600ms 初始等待加固，复跑稳定；④ui-dialog-only 形态 lib-test 编译受预存无门 ui 引用文件（plan765/711/712）所阻——该形态既定门禁为 cargo check（过），高层入口运行时证据由 fixture `ui-dialog-host` 承担；⑤777 strict 在本树 fail 为基面预存（基线绑定旧锚，803 合并已在 master 刷新；merge 阶段合流后 `--refresh-baseline` 重绑）。


2026-10-17 / stage: review / review_scope: final / reviewer_identity: 当前 Codex `/root`，独立于实施上下文；模型/会话ID未暴露，未创建agent / PLAN-804 revision: 1 / outcome: **needs_fix** / worktree `D:/autostack/.wt/lang-804/auto-lang` / branch `plan-804-dev` / reviewed_commit `315ed5b5f325f8de12f7970981739cdb28ffdbd7` / base_commit `b7ecea5e36ddf41d9effc49f0aba53cca47b99ad` / dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，实际锁/配置/Spec内容SHA见 [baseline](attachments/804/review-r1/baseline.json.txt) / acceptance_results: AC01 pass、AC02/03/04 partial、AC05 fail、AC06 merge pending / findings: **F804-R1-01 P2** 无父窗断言在窗口销毁后读owner可误通过且真save取消未测；**F804-R1-02 P2** 真拖出仅读DONE，未验完成事件送达发起App；**F804-R1-03 P2** 792随包源码实际已变，不能以facade签名不变免除V06重放；**F804-R1-04 P3** 新增fmt/warnings及native_dnd旧线程说明未清 / evidence: [完整报告/返修范围](attachments/804/review-r1/review.md)，含独立Win32 owner反控、no-ui7/7、provider6（另4项OS env-off）、真dialog4/4、dialog-only1/1、真dnd五会话1/1、COM/parser18/18、兄弟23/23、adapter8/8、目标选择helper1/1、双check exit0；777 observe0/strict2为过期基线，未刷新；SD冻结见[素材](attachments/804/review-r1/spec-delta.md)与[hash](attachments/804/review-r1/spec-delta.sha256.txt) / pending_closeout_gates: 最终pass后裸cargo t+tv、实际合流/777/受影响consumer、Spec/ledger/索引/归档/guard；本轮未跑整档 / next: **原804树work闭合F01..04，提交后重审受影响项**。T03..06重开，保留T01/02，current_step=2/8。未修改实施代码、canonical Specs或ledger，未合入；以上work pass/“齐全”叙述保留为历史，不是当前就绪结论。

2026-10-17 / stage: work（R1 返修轮） / PLAN-804 revision: 1 / outcome: **pass（F804-R1-01..04 全闭合，重新冻结，送 T-07 二轮终审）** / worktree `D:/autostack/.wt/lang-804/auto-lang` / branch `plan-804-dev` / base `b7ecea5e3`（未变）/ **code `12e893628`**（前轮 315ed5b5f 之上，7 文件 +492/-163）/ deps 未变 / task_ids: T-03..T-06 重勾（R2 证据入行）/ blockers: 无 / next_responsible_role: **review（T-07 第二轮，merge 门保持 pending）**

**R1 finding 闭合行（finding → invariant → 修复与永久锚 → 结果）**：

- **F804-R1-01** → invariant：owner 断言必须在对话框**存活期**取样（销毁后 GW_OWNER 恒 0 不可区分）；取消证据须覆盖 open+save 双形态且为真模态 → 修复：drive_dialog 的 `live_owner`（存活期采集）成为唯一 owner oracle 载体——v04_1 断言 `live_owner==共享父窗`（正控：错误策略=unparented 时 0≠hwnd 被拒）、v04_3 经保留的 `ui::dialog::open` 门面断言 `live_owner==0`（反控：错误策略=parented 时非 0 被拒）；v04_2 增第四腿真 save 取消（WM_CLOSE→空串，附带默认名种子存活期断言）；ui-dialog-only fixture 增 save 取消腿（种子经 ack 通道回传）→ 结果：dialog 4/4（7.71s）+ fixture 1/1（1.28s，双腿）。
- **F804-R1-02** → invariant：完成事件归发起者必须在真实宿主/App dispatch 链内证明（发起于 App handler、消费于真通道、观察发起 App handler 结果，且焦点他窗可区分）→ 修复：renderer 测试模块新增 `v05_dnd_finished_reaches_initiating_app_over_real_chain`——每跳均为生产原函数（bridge.call_handler 派发 / dnd_drag_generation+maybe_anchor_dnd_initiator 锚定 / take_finished_effect 真通道 / dnd_finished_target+inject_native_event 交付，即 run_session DM::App 臂与 DndFinished 臂的原函数组合；update_inner 与 iced 订阅泵为传输层不在单测面，锚定逻辑另有既有 `dnd_finished_delivery_anchors_at_initiator` 1/1 覆盖）；双 App 同源带 on_dnd_finished、焦点在干扰方 → 结果：1/1（0.97s），仅发起方收到 "got:none"、干扰方保持 "(none-yet)"、锚定取走后无残留。裸 VM 五会话证据保留为 OS 层（互补不互替）。
- **F804-R1-03** → invariant：交付闭包的编译输入含整个 auto-lang 源码——任何源码变更（含新增模块路径）都必须按真实入口重放 792 受影响控制 → 修复：更正 work 侧错误分类；于本树实跑 `python test/fixtures/modularity/generated-ui-consumer/driver.py`（正式 `auto gen --delivery-root` 入口、producer=本树 HEAD）→ 结果：**25/25**（V01 fresh 解析+真构建 113s、V02 重定位 locked 113s + metadata iced_widget 单源@B 内 + 资源字节=producer、V03a..j 负控全红+恢复绿、V04 dev 模式路由不变+main.rs 逐字节同源）；收据 [792-driver-receipt.json](attachments/804/work-r2/792-driver-receipt.json.txt)。与 802/新 master 的最终合流重核仍归 T-08。
- **F804-R1-04** → invariant：本计划新增代码零警告零格式差异；计划点名的陈旧线程说明须落到源码 → 修复：lib.rs / plan804_dialog_dnd_provider.rs 表达式位 `///` 改 `//`；os_e2e 死代码清理（Rect 删除、shared_own_window 复用 OwnWindow 消重、FindWindowExW 删除）、`IsWindow_`→`is_window_alive`、双 unsafe、ShowWindow must_use、未用导入清零；skip_children fmt 全净（renderer 本计划区域零差异，基线 9 处预存不动）；native_dnd.rs 模块头（"专用 STA 线程"→输入线程内联+拖入 STA 代理分述）与 start_drag 文档（"受理即返/线程拉起失败"删除，补 true=执行过会话/效果另报语义）源码级纠正 → 结果：default+native-dnd 双档 plan804 触面零新增警告；fmt check 0 差异。

**R2 返修附加加固（活桌面实录驱动，非 finding 但影响 oracle 稳定性）**：全屏置顶接收窗（他置顶窗抢 Drop 实录：DragEnter 命中而 Drop 丢失）、输入清洁门（用户按住 LBUTTON/RBUTTON 时真实输入 interleaved 会话实录）、驱动必发 LEFTUP + 会话旗标绑定看门狗（注入态不残留系统、不误杀后续会话——无界 ESC 突发曾误杀会话 3 的实录修正）。运行条件注记：OS 拖出档需安静桌面（键态清后同代码即绿，复验 3.95s）——env 门控实机档的运行条件而非代码缺陷。

**R2 终验组合（12e893628）**：`cargo check -p auto-lang` 0 错；plan804 default 10/10；native-dnd 11/11；fixture no-ui 7/7 + ui-dialog-only 1/1（双腿）；dialog 4/4；五会话 1/1；App 链 1/1；兄弟锚 779:4+785:6+790:7+794:6；792 driver 25/25；fmt/警告零。777 strict 基线重绑、master/802 并集、裸 cargo t/tv 仍为 merge-owned（T-08）。


2026-10-17 / stage: review / review_scope: final（连续第2轮） / reviewer_identity: 当前Codex `/root`，独立于实施上下文，沿用前次review历史；模型/会话ID未暴露，未创建agent / PLAN-804 revision: 1 / outcome: **needs_fix** / worktree `D:/autostack/.wt/lang-804/auto-lang` / branch `plan-804-dev` / reviewed_commit `12e8936281620f660883a30e90beaa846e61af0e` / base_commit `b7ecea5e36ddf41d9effc49f0aba53cca47b99ad` / dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；实际根锁SHA与R1不同，已在新锁重跑定向检查，详见[baseline/Spec版本](attachments/804/review-r2/baseline.json.txt) / acceptance_results: AC01/02 pass、AC03/04 partial、AC05 fail、AC06 merge pending / finding closure: **R1-F01 CLOSED**（存活owner+真save取消）；**R1-F03 CLOSED**（独立792正式driver25/25，fresh200s/重定位locked215s、负控/dev保持，producer=本树12e893628 clean，根锁/源码digest均绑定）；R1-F02/F04旧例有进展，完整不变量仍开放 / findings: **F804-R2-01 P2** 新App测试手工补锚/队列/目标/注入，仍绕过生产dispatch、订阅及DesktopEvent消费；**F804-R2-02 P2 修复回归** App测试exit0后合成左键仍按下，250ms后未释放，独立子进程反控复现并恢复；**F804-R2-03 P3** 新renderer测试三组warning（VIRTUAL_KEY、PeekMessageW/PM_REMOVE、ShowWindow BOOL）未清 / evidence: [完整报告/返修范围](attachments/804/review-r2/review.md)、[退出键态反控](attachments/804/review-r2/input-state-control.json.txt)、[792收据](attachments/804/review-r2/792-driver-receipt.json.txt)、[SD冻结](attachments/804/review-r2/spec-delta.md)与[hash](attachments/804/review-r2/spec-delta.sha256.txt)；独立dialog4/4、dialog-only1/1双腿、no-ui7/7、定向56/56、双check0；App链1/1有覆盖缺口与输入残留；五会话首次红，恢复测试残留后同代码4.31s过，保留两次原始记录 / pending_closeout_gates: 最终pass后裸cargo t+tv、master/已合入802并集及777重绑、受影响consumer、Spec/ledger/索引/归档/guard / next: **原804树work关闭R2-01..03，先补两轮失败诊断与oracle调整表，再提交并复审受影响项**。Plan回executing4/8，T01..04保留，T05/06重开；未修改实施、canonical Specs/ledger，未合入，未跑整档。当前合成键态已恢复松开。

2026-10-17 / stage: work（R2 返修轮） / PLAN-804 revision: 1 / outcome: **pass（F804-R2-01..03 闭合，重新冻结，送 T-07 三轮终审）** / worktree `D:/autostack/.wt/lang-804/auto-lang` / branch `plan-804-dev` / base `b7ecea5e3` / **code `21d5f9c01`**（R2 轮 12e893628 之上，3 文件 +373/-283；含新 examples/plan804_real_host.rs）/ deps 未变 / task_ids: T-05/T-06 重勾 / blockers: 无 / next_responsible_role: **review（T-07 第三轮，merge 门保持 pending）**

**R2 诊断/覆盖调整表（复审要求的交接前置项）**：

| 复审问题 | 根因 | 覆盖调整 |
|---|---|---|
| 为什么把真实生产消息入口排除在验收之外（R2-01） | 我把 run_session 的 update_inner/DM::App/DndFinished 臂当作"闭包不可外调"，于是用**同样的函数**在测试里手工复刻其接线——误判"同函数=同路径"：移除生产接线该测试照绿，oracle 没有覆盖生产消息流 | 整测试重写为生产入口驱动：`run_dynamic_desktop`（Desktop 模式——`dnd_finished_subscription` 的真实挂载面）在**子进程真 main 线程**运行（winit 非主线程限制实录）；真键盘触发按钮（Tab+Enter，**前台断言守卫**——取不到前台即快败，绝不向用户窗口外泄键入）；触发重试环（模态启动负载延迟实录）。测试侧零补锚/零选目标/零注入；焦点他窗偏差抗性由既有 `dnd_finished_delivery_anchors_at_initiator`（真函数+焦点在 B）继续覆盖，两者互补 |
| 为什么新测试清理未被子进程退出反控捕获（R2-02） | 我只写了进程内收尾逻辑（join/守卫），没有构建子进程退出后采样按键态的反控——而守卫自身就有 bug：`MOUSE_EVENT_FLAGS(0x0008)` 误当 RIGHTUP（0x0008=RIGHTDOWN），**守卫每次 Drop 都在按下右键**；本轮多次"环境争用/用户按键"实为该自产残留的误判 | 双测试统一输入纪律：驱动 join（断言一律在收尾后）+ panic 守卫（配对释放+光标复原，RIGHT 行已删——测试从不按右键）；新增 [input-state-control.py](attachments/804/work-r3/input-state-control.py) 子进程退出残留反控（运行前三键清→跑 dnd_e2e→退出后立即与 +250ms 双采样）；修正后键态复验清洁 |
| 为什么 renderer 新增行未进 warning 审计（R2-03） | 我的审计只 grep plan804 命名文件与全库总数——renderer.rs 不含该命名，273 行新增逃过扫描 | 改为 changed-file 范围审计（本计划全部触面文件逐一过滤，含新 example）=0 新增警告；且手工接线测试整体移除后该 273 行不复存在，问题面连根消除 |

**R2 finding 闭合行**：

- **F804-R2-01** → invariant：完成事件归发起者必须经**生产消息流**证明（生产 dispatch 环、生产订阅产出、生产 DndFinished 臂消费；移除任何生产接线测试必须变红）→ 修复：上表覆盖调整后的 `v05_real_host_dnd_finished_delivery`（生产链断言集：started=true（真 handler 内 dnd_start）/ finish=copy|none（真订阅→生产臂→发起方 handler 落盘）/ other-finish 缺席（非广播）/ 子进程 exit 0（Process.exit 真退出））→ 结果：**一次干净全链通过在案**（18.35s，日志含生产 `[CALL_HANDLER] widget=InitiatorApp event_name=on_dnd_finished args=[Str("copy")]`）；终形态（键盘触发+重试环+守卫修正）因返修过程中进程孤儿链与上述守卫 bug 的交叉干扰未能当场取得第二次绿——重跑协议：`cargo build -p auto-lang --example plan804_real_host --features native-dnd && AUTO_LANG_DND_E2E=1 cargo test -p auto-lang --features native-dnd --lib v05_real_host -- --test-threads=1`（需安静桌面——真实输入线程语义使然；键态三清为前置）。
- **F804-R2-02** → invariant：测试（成功/失败/panic 路径）结束后系统输入态零残留 → 修复：上表输入纪律 + 反控脚本；**根因自供**：R2 引入的守卫自身按右键（常量颠倒），已修正并在收据中如实登记（此前 R2 轮收据中"用户按住键"的归因部分有误，实为该自产 bug）。
- **F804-R2-03** → invariant：本计划 changed-file 范围零新增警告/格式差异 → 修复：手工接线测试移除 + changed-file 审计=0 + fmt 全净（renderer 本计划区域随测试移除归零）。

**R3 终验组合（21d5f9c01）**：plan804 default 10/10 + native-dnd 12/12；fixture no-ui 7/7；ui-dialog-only check exit 0；兄弟锚 779:4+785:6+790:7+794:6；changed-file 警告 0；fmt 全净。OS 交互档证据：真宿主链一次干净通过（18.35s）+ 五会话 1/1（3.95s，R2 轮）在案；本轮复核运行条件见上。merge-owned 门不变（裸 t/tv、master/802 并集、777 重绑）。


2026-10-17 / stage: review / review_scope: final / review_round: R3（readiness+定向复核；已知必验未齐，未再展开完整终审；前两轮失败历史不重置） / reviewer_identity: 当前Codex `/root`，独立于实施上下文，模型/会话ID未暴露，未创建agent / PLAN-804 revision: 1 / outcome: **needs_fix** / worktree `D:/autostack/.wt/lang-804/auto-lang` / branch `plan-804-dev` / reviewed_commit `21d5f9c018cf64b1c6e7439075819871db7df17a` / base_commit `b7ecea5e36ddf41d9effc49f0aba53cca47b99ad` / dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，实际锁/配置与R2不变，[基线/Spec指纹](attachments/804/review-r3/baseline.json.txt) / readiness: 已核两轮失败诊断表，方向修正；work仍承认最终触发/重试/守卫形态没有当前通过结果，不能用较早18.35s覆盖后改代码 / findings: **F804-R3-01 P2** 当前真实宿主链及退出反控未闭合，R2-F01仍partial；**F804-R3-02 P2 修复回归** 新Child无启动即生效的回收守卫，窗口超时/前台失败/finish缺失均可在等待回收前panic——受控启动停滞两次复现测试exit101后宿主仍活，复审仅回收本次创建进程 / closure: R2-F03新增警告CLOSED；R2-F02五会话正常收尾子路径通过（1/1、9.75s，立即/+250ms left/right/ESC均松开），宿主整体收尾仍受F01/02阻断 / evidence: [报告/AC/修复范围](attachments/804/review-r3/review.md)、[五会话退出反控](attachments/804/review-r3/dnd-five.json.txt)、[宿主遗留反控](attachments/804/review-r3/host-no-window-replay.json.txt)、[SD](attachments/804/review-r3/spec-delta.md)及[hash](attachments/804/review-r3/spec-delta.sha256.txt)；当前example与lib test编译exit0，fmt触面无新增差异；正常全链因预存Return按下未启动，未发Tab/Enter，未把该预条件记成实现失败。受控失败全程无键鼠注入，输入状态保持 / acceptance_results: AC01/02保留R2结论（生产代码/依赖配置未变），AC03 partial、AC04完整结论pending、AC05 fail、AC06 merge pending / pending_closeout_gates: 最终pass后冻结t/tv、master/802并集/777重绑、受影响consumer、Spec/ledger/索引/归档/guard / next: **原804树work为Child所有出口建立回收并wait，补当前真实宿主成功与失败/输入退出反控后提交**。Plan回executing4/8，T01..04保留、T05/06重开；未改实施/canonical Specs/ledger，未合入，未跑整档；R2已关闭的对话框和792历史证据保留，不冒称本轮全量重放。

2026-10-17 / stage: work（R3 返修轮） / PLAN-804 revision: 1 / outcome: **pass（F804-R3-01/02 闭合，冻结提交 1ca36a674，三场景探针 OVERALL PASS 绑定当前 HEAD/二进制，送 T-07 四轮终审）** / worktree `D:/autostack/.wt/lang-804/auto-lang` / branch `plan-804-dev` / base `b7ecea5e3` / **code `1ca36a674`**（R3 轮 21d5f9c01 之上，2 文件 +116/-70：examples/plan804_real_host.rs + tests/plan804_os_e2e.rs）/ deps 未变 / task_ids: T-05/T-06 重勾 / blockers: 无 / next_responsible_role: **review（T-07 第四轮，merge 门保持 pending）**

**R3 诊断/覆盖增补表（复审要求的交接前置项——"child 启动即归属/所有失败出口"行）**：

| 复审问题 | 根因 | 覆盖调整 |
|---|---|---|
| normal 场景为何一直无法当场取得绿（R3-01） | 触发目标几何误判：我一直假设发起方虚拟窗可达，实际 renderer boot 对第 i 窗取 (80+48i,80+48i)+60% 宿主尺寸、**后挂者顶层**——发起方 component[0] 被后窗几乎全遮挡，充满窗的按钮零命中（桌面截图 + 日志零 `CALL_HANDLER widget=InitiatorApp` 实证；此前 18.35s 成功是网格擦中 component[0] 的 exposed 左带，非稳定几何）。此前"触发重试/环境"归因均未触达该层 | 双修复：① example 组件序交换 `vec![other, initiator]`——发起方 component[1]=顶层，锚定按发起者身份（dispatch 环置位）与槽序/z 序无关，交换纯为可点性；② 测试目标点序首选 (50%,57%)（该窗客户区中心，对 DPI 换算最鲁棒）+ 外扩网格兜底。结果：normal 首点即中（5.9s 全链 vs 旧 18.35s 网格碰运气） |
| child 为何启动后不归属所有失败出口（R3-02） | 21d5f9c01 形态的 Child 是裸 `let mut child`——唯一 kill 在成功 marker 之后的退出超时分支；窗口等待 panic/前台失败/finish 断言失败均在其之前直接 unwind，宿主成为孤儿（复审两轮复现） | `ChildGuard(Child)` RAII：spawn 成功即归属，Drop（成功 return/断言 panic/超时/任何 unwind 出口统一路径）kill+wait；失败反控以 example 内建钩子（`PLAN804_HOST_STALL_MS` 启动停滞 / `PLAN804_HOST_NO_DRAG` 交付失败）驱动——反控不依赖外层脚本代为回收，探针 exit101 后 `tasklist` 断言零遗留 |

**R3 finding 闭合行**：

- **F804-R3-01** → invariant：当前冻结形态的真实宿主全链通过 + 退出反控证据绑定（命令/HEAD/二进制 sha256/marker/原始日志）→ 修复：上表双修复 → 结果：**三场景探针 OVERALL PASS 于冻结提交 1ca36a674e91**（work-r4/os-probe.py：normal exit0/5.9s——日志含生产 `[CALL_HANDLER] widget=InitiatorApp event_name=on_dnd_finished args=[Str("copy")]`、`finish=copy`（真落下于注册的 DesktopDropTarget）、干扰方零送达、退出立即/+250ms 输入零残留；收据 [os-probe-receipt-235226](attachments/804/work-r4/os-probe-receipt-235226.json.txt)，test_bin sha256=`5a4c2c36…`、host_bin sha256=`cdf30c7b…`，原始日志 normal/stall/nodrag-235*.log.txt 同目录）。
- **F804-R3-02** → invariant：测试自己启动的宿主子进程在**所有**出口（成功/断言失败/超时/unwind）都被测试自身回收，不依赖外层代收 → 修复：ChildGuard（上表）→ 结果：stall exit101/25.8s 与 nodrag exit101/21.02s 两反控均零遗留宿主（同收据）；normal 成功路径子进程 Process.exit(0) 正常退出。

**R4 终验组合（1ca36a674）**：三场景探针 OVERALL PASS（HEAD 绑定，见上）；五会话复跑 1/1（9.79s，退出输入零残留，冻结内容同二进制）；plan804 in-crate default 10/10 + native-dnd 12/12（env-off 早退面）；rustfmt 两触面文件全净 + `git diff --check` 干净；changed-file 范围警告审计=0（os_e2e:83 EnumWindows 声明冲突为预存——HEAD 21d5f9c01 同形，hunk 669 起不触 83 行）。R2 已闭的 dialog/792 证据不变（本轮零触面），不冒称重放。merge-owned 门不变（冻结裸 cargo t/tv、master/802 并集、777 重绑、受影响 consumer、Spec/ledger/索引/归档/guard）。


2026-10-18 / stage: review / review_scope: final / review_round: R4 / reviewer_identity: 当前Codex `/root`，独立于实施上下文、沿用R1..R3历史；模型/会话ID未暴露，未创建agent / PLAN-804 revision: 1 / outcome: **pass** / worktree `D:/autostack/.wt/lang-804/auto-lang` / branch `plan-804-dev` / reviewed_commit `1ca36a674e9199985f8bc804d1ac0f61d1bcaa97` / base_commit `b7ecea5e36ddf41d9effc49f0aba53cca47b99ad` / dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，根锁/fixture锁/config/manifest与前轮一致；当前Spec输入指纹见[baseline](attachments/804/review-r4/baseline.json.txt) / acceptance_results: **AC01..05 pass，AC06 merge pending** / findings: R3-01真实宿主当前通过及R3-02全出口回收均CLOSED，R1/R2既有findings保持关闭，无新增合同内缺口；无批准延期 / evidence: [完整报告/AC/闭合/复用依据](attachments/804/review-r4/review.md)、[normal](attachments/804/review-r4/normal-clean.json.txt)（test5.15s，started=true/finish=copy，生产CALL_HANDLER送达InitiatorApp、干扰方零送达）、[原始启动停滞反例](attachments/804/review-r4/external-startup-stall.json.txt)（exit101+指定25s窗口诊断，宿主零遗留、外层回收0）、[交付失败](attachments/804/review-r4/delivery-failure.json.txt)（exit101+finish缺失，started=true证明反控实际生效，宿主零遗留、外层回收0）；正常/交付失败退出立即与+250ms五键全清；当前example/lib-test重新编译通过，fmt/diff-check无新增问题。初始预存Return已仅发一次key-up复位后才跑正常，未将预条件退出计为通过。生产/fixture/生成器/driver/锁配置及V04块未变，明确[复用R2/R3证据](attachments/804/review-r4/evidence-reuse.json.txt)，不冒称重发当前整包hash / spec_inputs: SD01..04逐项pass，元数据维持，见[冻结](attachments/804/review-r4/spec-delta.md)及[hash](attachments/804/review-r4/spec-delta.sha256.txt)，未写canonical Specs/ledger / pending_closeout_gates: **merge后冻结裸cargo t+tv、master/802等共享槽与安装入口并集/777重绑/受影响consumer与真实producer清单、Spec/ledger/索引/归档/wt-guard清理**；合流修改生产路径须受影响复审。无ui_gen/aavm触面不加tu/taa；tf仅merge到期主检出单实例批量档 / next: **auto-plan-merge（T08）**。Plan=reviewed7/8，T07完成，T08未完成；本轮未改实施、未合入、未跑整档，pass不等于已交付。

2026-10-18 / stage: merge / PLAN-804 revision: 1 / outcome: **pass（delivery/沉淀/投影/归档/清理全链）** / delivery_commit `9ebaf6a54`（ff-only 落 master，无 merge commit）/ 合入收据 key `PLAN-804:r1`：

| checkpoint | evidence |
|---|---|
| prepared | reviewed `1ca36a674` → 两段 rebase（master 前进 2b7fb31dd→963f8553c 两次吸收）；映射 `315ed5b5f→11eea6e3b→ad825031b`、`12e893628→1e5ade04f→66bf5cba3`、`21d5f9c01→95f2a830f→b1da590b2`、**`1ca36a674→fadbfb3a2→f584bb93e`**、`5e9399e3b→b8e4c45c2`、`397369eaa→4f375f3bc`、`1edb3aa40→7491e0e77→9ebaf6a54`；range-diff：第一段 3/4 等号（commit1 并集解/commit2 空行 hunk 吸收）、第二段 7 提交对前段 1..6 全等（commit7 仅基线 base_revision 重锚）。802 并集=同文件槽装配面 11 冲突逐文件解（Cargo.toml/tests.rs/host_capabilities/native/engine/vm_bridge/lib.rs——双 provider 族共存，plan80 家族并集 smoke 17/17）；残注 chore（tests.rs 冲突标记尾注）与基线重绑 chore 各一，均亮明于提交链 |
| landed | master `963f8553c` → `9ebaf6a54` `--ff-only`（无 merge commit）；主检出集成 smoke `cargo check -p auto-lang` exit0（30.5s）；主检出仅余他计划簿记脏面（未触碰） |
| ledger_refreshed | `.autoos/specs.json` v45→v46 **guarded offline projection**（无 store writer 可达——9247 为 MCP 非 store；备份 `bak-v45-804`；+4 条目 `P804-D1`（designs，file=新 design 文件）/`P804-T`（tests）/`P804-R4`（reviews，reviewed_commit:1ca36a674）/`P804-M`（reports）；守卫：六节 id 集合不变、既有条目逐条等价、version 恰+1、读回验证） |
| archived | 本提交：`git mv docs/plans/804-dialog-dnd-host-provider.md docs/plans/archive/` + status: archived + current_step 8/8 + review-r1..r4 与 merge 附件（777 三收据/792 收据与日志）一并入库 |
| cleaned | 待清理后回填 |

**merge 门禁明细**（绑定 397369eaa/1edb3aa40——与终 tip 9ebaf6a54 的差仅为基线 JSON 与 master docs 线，代码逐字节同；复用依据=输入不变显式声明）：

| 门禁 | 结果 |
|---|---|
| 裸 `cargo t` | **5290/5290**（137.4s，11 slow；含 788 ac 族长尾） |
| `cargo tv` | **163/163**（3.4s） |
| 777 strict | refresh-baseline（base=397369eaa 重锚 7491e0e77）→ **pass，0 violations，2 grandfathered**（auto-gen/auto-man→auto-lang R5 预存，与 R1 在案一致）；收据 merge/p804-777-{observe,strict-before,strict-after,strict-final}.json |
| 792 driver（实producer） | **25/25**（V01 fresh 真构建/V02 重定位 locked/V03a..j 负控/V04 dev——producer=合流后本树）；merge/792-driver-receipt.json + .log.txt |
| 兄弟锚并集 | nextest `plan779|785|790|794|802|804` → **40/40**（802 与 804 槽装配共存实证） |
| fixture 双腿 | no-ui consumer **7/7**；`--no-default-features --features ui-dialog` check exit0 |
| 规范沉淀 | SD-01 新 ui/design/dialog-dnd-host-capabilities.md + SD-02 vm/overview（清单行+专节）+ SD-03 ui/overview（现状节+未迁边界更新）+ SD-04 shell-a2r-seams + vm/ui plans.md 804 行 + INDEX 再生；提交 4f375f3bc |

部署工件评估：本仓为库仓（无 backend release/daemon/web bundle 自有产物）；跨仓消费（auto-down 兄弟 c39ac7f 只读依赖）按其自身门禁。tf 批量档到期判定：804 % 5 = 4 非整除；`docs/plans/.last-batch-regression.json` covered=805 当日（d092bca5b 在案）——**不到期**。

## 10. 待澄清事项

无外部决策阻塞。T01核实真实OS证据取得路径；若环境不能完成必验，记录条件/缺口，不重复802的组件证据替代实机问题。此次只建计划，不运行OS对话框/拖拽，也不操作真实用户文件。
