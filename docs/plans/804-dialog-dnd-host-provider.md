---
plan_id: PLAN-804
status: execution_done
feature_name: 对话框与拖出宿主能力迁移
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 6
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
| [x] | T-05 | work | T04 | 【R2 重勾：三 finding 全闭合——F01：v04_2 补第四腿真 save 取消（含默认名种子断言）、v04_3 owner 断言改 driver 存活期采集 live_owner==0 且经保留的 ui::dialog::open 门面调用（与 v04_1 的 live_owner==共享窗 成对正反控，错误父窗策略会被双向拒绝）；F02：renderer 测试模块新增 v05_dnd_finished_reaches_initiating_app_over_real_chain——真 DesktopSession 双 App（同源均带 on_dnd_finished）+焦点在干扰方对照；发起=发起方 App handler 内 bridge.call_handler→VM shim→真 provider→真 DoDragDrop（SendInput ESC 驱动）；锚定=dispatch 环代际比对（镜像 run_session DM::App 臂原函数 dnd_drag_generation/maybe_anchor_dnd_initiator）；消费=真完成通道 take_finished_effect（session.rs DndFinishedRecipe 订阅轮询源）+ DndFinished 臂原两步 dnd_finished_target/inject_native_event；断言仅发起方 finish=got:none、焦点所在的干扰方 (none-yet)、锚定消费即取走 → 1/1（0.97s）；F03：更正输入分类（整个 auto-lang 源码为随包编译输入，签名不变≠闭包未变）并于本树实跑 python test/fixtures/modularity/generated-ui-consumer/driver.py → 25/25（V01 fresh 解析+真构建 113s、V02 重定位 locked 113s+metadata 单源+资源字节对拍、V03a..j 全负控判红+恢复绿、V04 dev 模式保持+模板同源；收据 attachments/804/work-r2/792-driver-receipt.json，producer=本树 HEAD）。R2 终验（修复提交 12e893628 后）：dialog 4/4（7.71s，含 save 取消腿）+ 五会话 1/1（3.95s）+ App 链 1/1（0.97s）+ fixture no-ui 7/7 与 ui-dialog-only 1/1 + plan804 default 10/10 + native-dnd 11/11 + 兄弟锚 779:4+785:6+790:7+794:6 全绿。原 work『792 输入零变化』判定有误，以本行实跑为准；原五会话/实机证据保留为 OS 层证据】 V04真模态+V05真拖出，V06合流验证——**全部实机证据齐**：①`AUTO_LANG_DIALOG_E2E=1 cargo test -p auto-lang --lib plan804_os_e2e::dialog_e2e -- --test-threads=1` → **4/4**（VM 链+owner=我方窗实锚/open 选真 Unicode 文件/save 默认名种子+选目标/取消空串/模态后 owner 可用/a2r unparented owner=0/run_with_capture 高层入口）；②ui-dialog-only 形态 `AUTO_LANG_DIALOG_E2E=1 cargo test --manifest-path …/dialog-dnd-provider-consumer/Cargo.toml --features ui-dialog-host v04_ui_dialog_only -- --test-threads=1` → **1/1**（兼容安装真入槽+真模态选文件）；③`AUTO_LANG_DND_E2E=1 cargo test -p auto-lang --features native-dnd --lib plan804_os_e2e::dnd_e2e -- --test-threads=1` → **1/1 五会话**（text 落下内容一致+Drop/Enter 线程=调用线程+effect copy/现有文件 CF_HDROP 路径一致/虚拟文件描述符+FileContents 落地 bytes 双通道一致/ESC 取消 true+无 Drop+effect none/取消后再落=DRAG_ACTIVE 复原+代际≥5）。V06：FileDialog::new/pick_file/save_file/add_filter 全仓唯一于 host_adapters::dialog（renderer pick_folder=PLAN-019 族外）；dialog_parent 单份；native.rs crate::ui 52→50（dnd 两处清零）；兄弟锚 779:4+785:6+790:7+794:6 全绿（无回退）；777 observe 零新增边、strict fail=基面预存（stash 复证，803 合并已刷新 master 基线，本树重绑归 merge）；792 输入零变化（facade 签名不变→fresh/locked/重放无需重跑）；802 仍未合入（R3 修复轮在途）→共享写面并集责任留 T-08 | 03/04 |
| [x] | T-06 | work | T05 | 【R2 重勾：重新冻结提交 **12e893628**（plan-804-dev，7 文件 +492/-163）——F04 卫生全清：计划触面文件新增警告零（default+native-dnd 双档复验）、skip_children fmt 全净（renderer 本计划区域零差异，基线 9 处预存不变）、ui/native_dnd.rs 模块头与 start_drag 文档陈旧线程叙述（『专用 STA 线程』『受理即返/线程拉起失败』）源码级纠正为输入线程内联阻塞真实现（SD-03 落源码非仅素材）；e2e 加固：全屏置顶接收窗（活桌面置顶窗抢 Drop 实录）+输入清洁门（并发用户输入 interleaved 实录）+会话旗标看门狗（注入态不残留系统/不误杀后续会话）。运行条件注记：OS 拖出档需安静桌面（实录：用户按住 LBUTTON/RBUTTON 时清洁门超时后会话被真实输入 interleaved 而红，键态清后同代码即绿）——env 门控实机档的运行条件，非代码缺陷】 来源/删边/去重/剩余族/SD收据，冻结提交 **315ed5b5f**（branch plan-804-dev，18 文件）——终验：`cargo check -p auto-lang` 0 错 ✅；`plan804` default 10/10 ✅、native-dnd 11/11 ✅；fixture no-ui 7/7 ✅；`--no-default-features --features ui-dialog` check exit 0 ✅；三套实机档复跑全绿（6.44s/0.73s/4.79s）；计划文件 rustfmt 全净（命中差异均为预存子模件）；renderer.rs PLAN-019 注释旧位置引用随迁更正。SD 素材见 §9 work 收据 | 04 |
| [ ] | T-07 | review | T06 | 同树独立AC/SD和OS证据审查，merge门pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、沉淀/归档/wt-guard清理 | 06 |

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
- **F804-R1-03** → invariant：交付闭包的编译输入含整个 auto-lang 源码——任何源码变更（含新增模块路径）都必须按真实入口重放 792 受影响控制 → 修复：更正 work 侧错误分类；于本树实跑 `python test/fixtures/modularity/generated-ui-consumer/driver.py`（正式 `auto gen --delivery-root` 入口、producer=本树 HEAD）→ 结果：**25/25**（V01 fresh 解析+真构建 113s、V02 重定位 locked 113s + metadata iced_widget 单源@B 内 + 资源字节=producer、V03a..j 负控全红+恢复绿、V04 dev 模式路由不变+main.rs 逐字节同源）；收据 [792-driver-receipt.json](attachments/804/work-r2/792-driver-receipt.json)。与 802/新 master 的最终合流重核仍归 T-08。
- **F804-R1-04** → invariant：本计划新增代码零警告零格式差异；计划点名的陈旧线程说明须落到源码 → 修复：lib.rs / plan804_dialog_dnd_provider.rs 表达式位 `///` 改 `//`；os_e2e 死代码清理（Rect 删除、shared_own_window 复用 OwnWindow 消重、FindWindowExW 删除）、`IsWindow_`→`is_window_alive`、双 unsafe、ShowWindow must_use、未用导入清零；skip_children fmt 全净（renderer 本计划区域零差异，基线 9 处预存不动）；native_dnd.rs 模块头（"专用 STA 线程"→输入线程内联+拖入 STA 代理分述）与 start_drag 文档（"受理即返/线程拉起失败"删除，补 true=执行过会话/效果另报语义）源码级纠正 → 结果：default+native-dnd 双档 plan804 触面零新增警告；fmt check 0 差异。

**R2 返修附加加固（活桌面实录驱动，非 finding 但影响 oracle 稳定性）**：全屏置顶接收窗（他置顶窗抢 Drop 实录：DragEnter 命中而 Drop 丢失）、输入清洁门（用户按住 LBUTTON/RBUTTON 时真实输入 interleaved 会话实录）、驱动必发 LEFTUP + 会话旗标绑定看门狗（注入态不残留系统、不误杀后续会话——无界 ESC 突发曾误杀会话 3 的实录修正）。运行条件注记：OS 拖出档需安静桌面（键态清后同代码即绿，复验 3.95s）——env 门控实机档的运行条件而非代码缺陷。

**R2 终验组合（12e893628）**：`cargo check -p auto-lang` 0 错；plan804 default 10/10；native-dnd 11/11；fixture no-ui 7/7 + ui-dialog-only 1/1（双腿）；dialog 4/4；五会话 1/1；App 链 1/1；兄弟锚 779:4+785:6+790:7+794:6；792 driver 25/25；fmt/警告零。777 strict 基线重绑、master/802 并集、裸 cargo t/tv 仍为 merge-owned（T-08）。

## 10. 待澄清事项

无外部决策阻塞。T01核实真实OS证据取得路径；若环境不能完成必验，记录条件/缺口，不重复802的组件证据替代实机问题。此次只建计划，不运行OS对话框/拖拽，也不操作真实用户文件。
