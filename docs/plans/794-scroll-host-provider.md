---
plan_id: PLAN-794
status: execution_done
feature_name: scroll controller 与子锚点宿主能力迁移
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 6
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/widgets/scroll-pane.md, docs/specs/auto-lang/ui/design/scroll-child-anchor.md]
new_spec_components: []
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-lang/vm, auto-lang/ui, widgets]
depends_on: [PLAN-781, PLAN-790]
---

# [PLAN-794] scroll controller 与子锚点宿主能力迁移

## 0. 变更摘要

将9900–9905 controller/offset族及781新增9922/9923子锚点族的VM→UI直引迁为owned宿主provider；保留renderer队列、测量和锚点状态机单一来源。八个native全部纳入，不能只迁旧offset六函数。

## 1. 目标

无UI宿主能注入真实scroll能力并经源码/VM观察可区分状态；UI生产通过adapter保持现有滚动与子锚点效果，core shim无ui::scroll实现依赖。非目标：code_editor滚动9911..9913、term engine、managed content新锚点支持、方法语法糖或滚动算法重写。

## 2. 架构方案

core保留参数/返回编解码；provider只暴露owned句柄、轴/intent、测量/锚点状态DTO，不含UI Axis/ScrollIntent类型、VM heap id或renderer引用。UI adapter委托现有controller registry与状态机。沿790优先级：显式per-VM/默认快照保持，兼容installer仅补空槽；不在core暗建UI实例。

## 3. 技术栈

既有host_capabilities/NativeInterface/VM session、StakeGuard、ui/scroll/controller.rs、scroll-pane与781锚点collector；autoui-verifier双端定向验证。

## 4. 需求分析与背景调查

取材 auto-lang@`9c773601abfdb25b80c08d06922d0ad50edc653a`；[本批输入指纹](attachments/parallel-extraction-20261016/inputs.json) 记录现行 Specs 与实际源码 SHA256。overview仅作导航，以模块Spec和真实代码为准。789/790/791均归档8/8；792为独立构建发行线，当前无交付假设。
用户要求“最好能有几个并行的计划”，本轮按该授权创建可执行合同和编排文档；未授权在本轮实施、创建agent/工作树、发布或修改旧仓refs。确认合同后各自work；本计划不依赖792完成，正式搬仓仍须满足整条路线退出条件。
787/788在途实验线不设为全局前置；实施者必须按实际diff重核类型/manifest共享面。全部实现/测试在本计划唯一组树，主检出只写计划簿记。
native_catalog明确9900–9905和9922/9923；native.rs现仍直接调用ui::scroll、按pending_native_arg_count解码可选轴。controller.rs含全局队列、最近测量和绑定generation；781合同规定queued不是完成、managed=unsupported、换绑/失锚等invalid reason与一次性消费。必须完整继承，不因provider有返回值就认为已滚动。
主写面native scroll族、host_capabilities/engine/相关宿主安装、UI adapter与定向scroll测试；不改793 error实现、不改795 lock/target、不改792 rust_ui及依赖升级。lib.rs/manifest等共享片段按写入窗口串行；不要求等792整项结束。

## 5. 详细设计

T01冻结八函数ID/arity/栈消费/返回形态及各feature缺席分支。保持controller为任意非空opaque string，不能新强制@scrollctl前缀。轴默认/未知轴、相对/绝对参数解码与state record类型保持；Core DTO在shim转换，UI adapter转换为原UI类型。
保留renderer的binding/measured state唯一真源；不复制第二个core registry。to_child入队与anchor_state查询仍分离；pending/applied/invalid/unknown、unbound首绑与换绑、key查找、managed unsupported和重复请求覆盖均不变。列表/字符串/记录所有权在成功/失败/缺席/提前返回配平，锁外调用provider。
per-VM隔离针对provider选择及快照，不声称同一UI adapter的既有全局registry自动变成多租户隔离；UI句柄分配和双pane寻址保持。不要顺手修改全局registry算法。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md | 八native provider/DTO/缺席与ownership | core去反向边 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | UI adapter与生产装配、未迁族清单收窄 | 接回真宿主 | AC-02/04 |
| SD-03 | modify | docs/specs/widgets/scroll-pane.md | 增宿主装配说明，原测量/intent合同保持 | 责任清楚 | AC-04 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/scroll-child-anchor.md | 调用边界迁provider，不改状态机/完成oracle | 781完整继承 | AC-02/04 |

### SD 冻结素材稿（T-06，供 T-07 复审与 T-08 沉淀）

**SD-01（vm/overview 增补节）**：`scroll_controller/to_start/to_end/by/to/state`（9900..9905）与 `to_child/anchor_state`（9922/9923，canonical `auto.scroll.*`）供值经 per-VM `ScrollHostProvider`（trait 五方法：`next_controller_handle()->String`、`enqueue_intent(&str, ScrollIntentDto)`、`controller_snapshot(&str)->ScrollSnapshotDto`、`enqueue_anchor_request(&str,&str)->Result<(),&'static str>`、`anchor_request_state(&str,&str)->Option<ScrollAnchorStateDto>`；DTO=core 自有 `ScrollAxisDto{X,Y}`/`ScrollIntentDto{ToStart/ToEnd/ScrollBy/ScrollTo}`（无 source——natives 恒 Programmatic）/`ScrollSnapshotDto` 8×f64（progress 由 provider 侧几何单源投影）/`ScrollAnchorStateDto{Pending/Applied/Invalid(String)}`；零 UI 类型/VM 堆 id/字符串池索引/renderer 引用）。装配=779/785/790 同构三级（实例注入 > 构造期进程默认快照 > 兼容 if-absent 补空槽）；注入不 bump binding_revision/不撤 proof；Clone/merge 保 provider。缺席语义=T-01 冻结降级臂逐字面值（controller=Int 0、intent 族=Bool false、state=Float 0.0、to_child="error:unsupported"、anchor_state="unknown"——值语义非 raise，任何 provider 不得伪装成功）。栈解码契约：COUNTED arity 辨识可选轴；弹序与 codegen 左→右压序对齐（可选轴在顶先弹——T-01 勘定修复 656 期 handle 先弹的可选轴误路由，探针实证、库内零破坏面）；`scroll_to(handle,a,b)` a 字符串=轴形单 intent、否则坐标形 X 先 Y 后双 intent；空串句柄跳过入队仍按在位性返 true/false；九轴词只认 "x" 其余归 Y。句柄=任意非空 opaque string（不强制前缀）。
**SD-02（ui/overview 增补节+未迁清单收窄）**：`ScrollControllerProvider`（无状态）委托 `ui::scroll::controller` 既有注册表/锚点状态机（renderer binding/measured 单一真源、drain 解析不迁、不复制第二 registry），挂 `ui` 单门控（队列层纯 Rust 无 OS 面，无 790 双门控）；progress 经 `ui::scroll::geometry::progress` 单源现算（与迁移前 shim 内联同源同值）。宿主接线四点与 779/785/790 同集：`install_ui_host_capabilities`、`run_file_dynamic_ui_inner`、VmBridge 两构造（全 if-absent）。未迁族清单更新：core→ui 直引剩余=code_editor（2910..2915/2919..2924/9906..9913 含其滚动三函数）、dialog（2927/2928）、dnd_start（2938）、term/media 等按 790 版清单减去 scroll controller 族（9900..9905/9922/9923）。
**SD-03（widgets/scroll-pane 增补节）**：宿主装配说明——VM 侧八 native 经 `ScrollHostProvider` 装配（UI 生产经 `ScrollControllerProvider` 兼容默认恢复原接线；no-ui 宿主注入自定义实现得真滚动观察面）；原测量/intent 合同（f64 逻辑空间、drain 顺序折叠、未预热晚一拍 F-4 纪律、双轴独立叠加）与 widget 行为零改动。
**SD-04（scroll-child-anchor 调用边界节更新）**：`to_child/anchor_state` 调用边界迁 provider（core shim 编组 `queued`/`error:<reason>`/`pending`/`applied`/`invalid:<reason>`/`unknown` 字符串）；781 状态机（queued≠完成、managed unsupported 入队即拒、换绑代际、gen=0 采当前绑定、未预热/无采集 pending 晚一拍、not_found/not_bound 一次性、后写覆盖重新 pending、applied 不驻留）与完成 oracle（真实 offset 变化+目标正面积交集）原样留在 controller.rs/renderer——零语义迁移。

## 6. 测试设计

实施者未另指派，独立final review与merge分别交接，不新建或指定额外模型。下表V用例由work实现并执行，不是本轮通过证据。开发及needs_fix只做定向验收与必要cargo check；最终代码提交、final review通过后，由merge在冻结提交跑裸cargo t及实际触面档。tf仅主检出批量单实例，不随各计划并发；无Schema/生成器改动不跑docs_gen。

| Case / owner | AC | 入口/输入 | observable oracle / 边界 |
|---|---|---|---|
| V01 / work | 01/02 | 新test/fixtures/modularity/scroll-provider-consumer，no-ui源码→编译→八native | fake记录axis/offset/key并返回特异快照/状态，真实VM输出匹配；独立manifest无UI类型provider |
| V02 / work | 02/03 | 各arity/轴/Unicode句柄与key、缺席降级、两VM/Clone/显式默认优先、重复调用清栈reap | 参数顺序/返回形态与旧臂一致；不串provider、不泄漏、不覆写；无假默认值掩盖没接线 |
| V03 / work | 02/04 | 真实UI普通pane/双pane/锚点；首绑、换绑、失锚、未测量、managed、重复请求 | 原pending/applied/invalid与one-shot成立；真实offset变化+目标正面积交集，queued/progress不是完成证明 |
| V04 / work | 04 | autoui-verifier对现有scroll/781 fixture Vue+VM定向交互 | 双端原行为一致，不修改golden或移除失败case；不要求全画廊 |

拟树D:/autostack/.wt/lang-794/auto-lang，branch codex/plan-794-dev。定向验证必须在final review前完成；merge-owned裸t+tv，若改ui_gen加tu；其他重档仍主检出单实例。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 八native无UI实现直引，no-ui独立源码VM注入V01通过 |
| AC-02 | work | ID/arity/缺席/状态与781语义保持，V02/03通过 |
| AC-03 | work | 成功/失败/缺席ownership、实例选择/优先级负控通过 |
| AC-04 | work | UI真实adapter/renderer测量链不漂移，V03/04通过，剩余宿主族清单明确 |
| AC-05 | review | 独立源码VM+真实UI完成oracle及SD复审通过 |
| AC-06 | merge | 冻结门禁、规范/ledger/归档与guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 781/790归档、合同确认 | 冻结native/feature/arity/record与781状态旧臂，列宿主构造清单 | 02/03/04 |

T-01 冻结矩阵（实勘 native.rs@8b97d3c9b + native_catalog 9900..9905/9922/9923 + controller.rs）：

| ID | canonical | 实参（压序→弹序） | 返回·provider 在位 | 返回·缺席（=旧 no-ui 臂逐字面值） |
|---|---|---|---|---|
| 9900 | auto.scroll.controller | 无实参 | Str（宿主分配不透明非空句柄；UI 宿主="@scrollctl:N"） | Int 0（非空串，旧臂原形） |
| 9901 | auto.scroll.to_start | handle[,axis]；COUNTED arity≥2 才弹 axis | Bool true | Bool false |
| 9902 | auto.scroll.to_end | 同上 | Bool true | Bool false |
| 9903 | auto.scroll.by | handle[,axis],delta；弹 delta→axis(arity≥3)→handle（勘定后弹序；旧臂 delta→handle→axis 见下勘定注记——R1 材料同步） | Bool true | Bool false |
| 9904 | auto.scroll.to | handle,a,b；弹 b→a→handle；a 字符串=轴形(1 intent)，否则坐标形(2 intents，X 先 Y 后) | Bool true | Bool false |
| 9905 | auto.scroll.state | handle | "ScrollState" 8 具名字段 record（GenericInstanceData 堆路径，全 Float：offset_x/y、viewport_w/h、content_w/h、progress_x/y） | Float 0.0（非 record） |
| 9922 | auto.scroll.to_child | handle,key；弹 key→handle | Str "queued"/"error:bad_handle"/"error:empty_key"/"error:unsupported" | Str "error:unsupported" |
| 9923 | auto.scroll.anchor_state | 同上 | Str "pending"/"applied"/"invalid:<reason>"/"unknown" | Str "unknown" |

轴解码：轴词只认 "x"，其余（含缺省/非字符串）归 Y。句柄=任意非空 opaque string（不新强制 @scrollctl 前缀）；9901..9904 空串句柄=跳过入队仍返 true/false（按 provider 在位性）。
栈纪律统一裁定：八 shim 统一体恒弹声明 arity（旧 no-ui 臂纪律）；旧 UI 臂在「空句柄+可选轴」组合少弹可选轴（潜伏失衡，正常程序不可达）——按 §5 配平纪律修正，返回值不变（记为合同内对齐，非语义变更）。
**T-01 勘定（work 期发现，探针实证）**：9901/9902/9903 的旧 UI 臂「先弹 handle 再弹可选轴」与 codegen 左→右压栈（末参在顶）冲突——可选轴形实参被误路由（记录型 provider 探针：`scroll_to_end(h,"x")` 落 handle="x"、axis=Y；`scroll_by(h,"x",d)` 同病）；9904 的 b,a,handle 弹序自洽正确。库内全部 .at 用法（p656/p781 capability tests）均为单轴简写形/无轴形，零破坏面。修复=三 shim 弹序改为「可选轴在顶先弹、handle 后弹」（与 9904 同型），行为按**签名合同**（`to_end(handle, axis?)` 轴路由到轴）对齐；V02 arity 矩阵以此为新回归锚。
DTO（按现有 state/anchor 投影冻结）：`ScrollAxisDto{X,Y}`；`ScrollIntentDto{ToStart/ToEnd/ScrollBy/ScrollTo}`（无 source——natives 恒 Programmatic，UI adapter 落 source=Programmatic）；`ScrollSnapshotDto` 8×f64（progress 由 provider 侧几何单源投影，core 不复制滚动算法）；`ScrollAnchorStateDto{Pending/Applied/Invalid(String)}`（reason 透传）；trait `ScrollHostProvider` 五方法：next_controller_handle/enqueue_intent/controller_snapshot/enqueue_anchor_request/anchor_request_state。
781 状态机完整继承：queued≠完成；managed=unsupported（入队即拒）；换绑代际不符→invalid:not_bound（gen=0 请求采当前绑定）；未预热/无采集帧→pending 晚一拍；key 不在当帧→invalid:not_found（一次性）；同 (handle,key) 后写覆盖重新 pending；applied 不驻留拉回；无记录→unknown。
宿主构造清单（T-04 接线点，全 if-absent 补缺）：`install_ui_host_capabilities`（lib.rs 统一 installer，cfg ui）、`run_file_dynamic_ui_inner`（cfg ui-iced）、VmBridge 两构造（cfg ui-iced）。adapter 门控=feature "ui" 单门控（ui::scroll 队列层纯 Rust 无 OS 面，无 windows/native-clipboard 双门控）。
| [x] | T-02 | work | T01 | owned provider/DTO、VM及默认快照/优先关系，注册定向控制 | 01/03 |
| [x] | T-03 | work | T02 | 八shim去UI引用、栈解码和返回编组，保留降级并配平引用 | 01/02/03 |
| [x] | T-04 | work | T03 | UI新adapter及统一宿主安装，委托原registry/collector，不复制状态机 | 02/04 |
| [x] | T-05 | work | T04 | 外部consumer、V01..04及真实UI证据，777检查八族删边；R1重开→重闭：F-794-R1-02 探针oracle修复（新鲜查询+失败传播+永久负控+pending重放），修复后探针 60/60 | 01–04 |
| [x] | T-06 | work | T05 | 触面/未迁族、双端/ownership/并行收据与SD素材；提交冻结代码；R1重开→重闭：F-794-R1-01 基线重采样@7a566c89b+strict pass、F-794-R1-03 unused_mut清零、T-01旧行9903弹序材料同步 | 01–04 |
| [ ] | T-07 | review | T06 | 独立八native及UI实际完成条件、负控、SD冻结 | 05 |
| [ ] | T-08 | merge | final pass | 裸t+tv及实际tu；沉淀/归档/guard清理，后合入者同步重验 | 06 |

**R1 findings 闭环行（finding → invariant → affected → fix → current results → remaining gap）**：
- F-794-R1-01（P2 基线过期）→ 不变式=Cargo.toml 变更触及 777 STALENESS_PATHSPECS 后冻结基线必须同步重锚且 strict 复跑 → affected=本批 consumer exclude（合法新增，非坏边）→ fix=干净提交 7a566c89b 上 `--refresh-baseline` 显式重采样（declared_edges=43 无新增禁向边）+ strict 复跑 pass（0 violations，2 条既有窄 grandfather 保持）+ 基线入库 d027e5f61 → 剩余缺口=无。
- F-794-R1-02（P2 VM 探针 oracle）→ 不变式=状态查询 oracle 每轮必须重新执行真实查询动作且点击失败必须判负（不得读缓存标签）→ affected=p781 vm_probe `expect_reply`（781 继承面，仅 oracle 层；生产状态机经复审独立源码补控确认无回归）→ fix=每轮重按只读 Q* 按钮重跑 `scroll_anchor_state` + press 失败立即判负 + 样本序列入证据 + 桩式永久负控（`oracle_failed_press_stale_label_rejected`：press=False+旧标签 applied 必拒，确定性无 UI 依赖）+ `requeue_pending_to_terminal_replay`（GoC3 重入队经 781 后写覆盖确定回 pending→新鲜查询到 applied）→ 复现对照=复审原跑 56/57（`v03_old_key_not_found_on_ch2` 粘 pending）→ 修复后 60/60（57 原 case 全保留零删除，3 新控制入列）→ 剩余缺口=无（注：重放样本序列中 pending 是否被采样取决于帧时序，状态机经重入队确定走了 pending 分支，序列入 detail 如实记录；Vue 探针 `clickAndWait` 同类单读模式未被复审列为 finding 且双跑 59/59，记为 781 fixture 观察项不扩面）。
- F-794-R1-03（P3 unused_mut）→ 不变式=本计划新增文件零新增警告 → affected=plan794 测试 FakeScroll::enqueue_anchor_request → fix=去无用 mut → 复核=`cargo t scroll_host_provider` 9/9 且输出零 plan794 文件警告 → 剩余缺口=无。
- wt-guard 链接红项（复审收尾注记，非编号 finding）→ fix=按 guard 指定方式逐链接 `cmd /c rmdir`（356 个 junction，目标均树内 .pnpm）后清除 p656/p781 生成 gen 树与 rust-workspace/dist（均 git 忽略生成物；根因=Vue 探针运行的 pnpm install 在树内建 junction——后续在 worktree 内跑 Vue 探针须自检清理）+ 杀探针遗留进程（vite node/esbuild/PID19128 backend）→ wt-guard 重跑 **clean** exit0 → 剩余缺口=无。

T-02 证据：host_capabilities.rs 增 `ScrollHostProvider` 五方法 trait + 四 DTO（Axis/Intent/Snapshot/AnchorState，全 owned 标准库值域）+ 进程默认槽四 API（set/if_absent/get/clear，同锁原子语义与 779/785/790 逐条相同）；engine.rs `new_inner` 构造期快照 + `AutoVM::set/scroll_host_provider`；native.rs NativeInterface 槽（per-VM RwLock + Clone 保留 + set/get，注入不动 binding_revision）。`cargo check -p auto-lang` 零错误。
T-03 证据：八 shim 统一 feature 中性体（旧 `#[cfg(feature="ui")]` UI 臂与 `#[cfg(not)]` 降级臂全删合并）；缺席臂=T-01 冻结逐字面值；空句柄/轴解码/双 intent 次序/StakeGuard 配平保持；T-01 勘定=9901/9902/9903 弹序改「可选轴在顶先弹」修复 656 期可选轴误路由（记录型 provider 探针实证，见 §4 勘定注记；库内全部用法为简写形零破坏）。`cargo t scroll` 81/81、`cargo t provider` 36/36。
T-04 证据：`ui/scroll_host_provider.rs` 新 adapter（DTO↔UI 类型转换 + `install_process_default` if-absent 兼容默认 + 优先关系/同源读/直通三单测）；挂 `ui` 单门控；接线四点=lib.rs 统一 installer、`run_file_dynamic_ui_inner`、VmBridge 两构造。
T-05 证据：in-crate `tests/plan794_scroll_host_provider.rs` 6/6（剪断锚节区段圈定避 code_editor 误伤/ID 钉扎/缺席降级全矩阵/fake 委托精确参数含坐标形 X 先 Y 后/Clone-merge-两 VM 隔离-实例覆盖不污染默认/真实 adapter→注册表接线含 managed 拒绝+失锚 not_found 驻留+后写覆盖重新 pending）；外部 fixture `scroll-provider-consumer` 5/5（V01 no-ui 真管线 stdout + V02 arity/轴/Unicode/缺席/两 VM/Clone/merge/显式默认优先/32 轮清栈+reap_all 后 heap 活计数回基线）；V03/V04 实机=p781 VM 探针 57/57 + p781 Vue 59/59（首跑 58/59 唯一红=overlay.ts 动态导入竞态与 scroll 无关，二跑全绿，报告存档）+ p656 VM 全过（六旧函数全链 ordinary/managed/on-scroll）；p656 Vue 腿环境性跳过（fixture 无 gen/backend 生成路径预存缺口，非本批触面；冗余覆盖由 781 Vue v06 承担）；777 core_to_ui code 类 570→549（八族直引删净，新增仅 4 处 composition-root install 接线）。收据=attachments/794/work/evidence-summary.md。
T-06 证据：rustfmt 触面文件（fmt 递归波及的 43 个非触面文件已全部回滚——790 同款坑规避）；`cargo check -p auto-lang -p auto` 零错误零新增警告；SD 冻结素材稿见 §5；merge-owned 门禁（裸 `cargo t`+`tv`；ui_gen 零触面故 tu 不触发；tf 批量档主检出单实例）**pending** 于冻结提交。

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（合同preflight） | next=合同确认且native热点空闲后work | 0/8；无需等792，不以fake代替UI完成证明。

2026-10-16 stage=work | PLAN-794 | plan_revision=1 | outcome=pass（work 完成，6/8；T-07 review / T-08 merge 待接） | worktree=D:/autostack/.wt/lang-794/auto-lang | branch=codex/plan-794-dev | base=8b97d3c9b | code=91974d8f8（树 clean） | deps=auto-down sibling worktree detached@c39ac7f9（组内解析只读；792/793/795 并行线热点不相交本轮未推进） | tasks=T-01..T-06 | evidence=AC-01（八shim零 ui::scroll 直引——剪断锚节区段圈定 + 777 core_to_ui code 类 570→549；no-ui 独立 manifest fixture 真管线 V01 5/5）；AC-02（ID/arity/缺席/781 语义——in-crate 6/6 + fixture 降级/参数矩阵 + p781 VM 57/57 全状态机 + p656 VM 六旧函数全链）；AC-03（ownership 配平=缺席/空句柄/成功路径恒弹声明 arity + 32 轮清栈 + reap_all 后 heap 活计数回基线；实例选择/优先=构造快照/两 VM 不串/实例覆盖不污染默认/Clone-merge 保 provider/注入不 bump revision）；AC-04（真实 UI adapter→原注册表接线不漂移 + p781 双端 57/57+59/59 + p656 VM；未迁族清单入 SD-02）。T-01 勘定（合同内对齐，非语义变更）：9901/9902/9903 可选轴形弹序修复（656 期潜伏误路由，探针实证+零库内破坏面，V02 新回归锚）。merge-owned pending：裸 `cargo t` + `cargo tv` 于冻结提交 91974d8f8（ui_gen 零触面 tu 不触发；tf 批量档主检出单实例）。验证集：cargo check -p auto-lang/-p auto 零错误零新增警告；cargo t plan794 6/6、scroll 81/81、provider 36/36；fixture 5/5；实机探针见 attachments/794/work/evidence-summary.md | blockers=无合同内未闭项；环境注记=sccache 端口保留段绕开（SCCACHE_SERVER_PORT=42917）、p781 vue 后端冷编译超 CLI 120s 需预热一次、p656 Vue 腿预存 gen 缺失环境性跳过（冗余覆盖在案） | next_responsible_role=review（T-07：独立重放八 native 负控/实机完成 oracle/弹序勘定复核/SD 冻结校验，绑定 91974d8f8）

2026-10-16 R1：stage=review | review_scope=final | reviewer_identity=当前独立Codex审查上下文（未参与实施；未另建agent/树） | plan_id=PLAN-794 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-794/auto-lang | branch=codex/plan-794-dev | reviewed_commit=91974d8f8d826a9cdb3ea6a50038a518bc02c4e9 | base_commit=8b97d3c9bc9c86cb4d8ebde4a5d7856554e83b9c | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（clean）；本机锁/配置/探针/CLI指纹见附件 | spec_inputs=四目标Spec与原Plan SHA256见附件spec-inputs.json.txt；冻结SD见spec-delta.md及hash | acceptance_results=AC01/03 pass，AC02/04 partial，AC05 fail，AC06 pending | findings=F-794-R1-01(P2：777 strict exit2，Cargo.toml使冻结基线过期)、F-794-R1-02(P2：指定VM查询探针可接受pressed=False+旧applied，且只读缓存pending；原实机56/57)、F-794-R1-03(P3：新增测试126行unused_mut) | evidence=[完整报告与复现](attachments/794/review-r1/review.md)；定向9/9（含计划6例）、外部consumer5/5、独立源码八native及32轮heap/pool配平补控2/2、真实VM重新查询控制组57/57、隔离Vue+VM API后端59/59、触面fmt/diff-check通过；默认Rust后端复跑因原有进程占用生成exe失败，未冒称通过 | pending_closeout_gates=裸cargo t+tv、规范/ledger/归档、guard链接清理；wt-guard实际exit1（生成node_modules含junction，禁止当前移除）；ui_gen未触及不触发tu，tf仍批量单实例 | next=executing 4/8，保留T01..04、重开T05/06，原树修复提交后复审；未修实施代码/合入。上方work通过收据保留为历史，不覆盖本轮开放findings。

2026-10-16 stage=work（R1 修复轮） | PLAN-794 | plan_revision=1 | outcome=pass（T-05/T-06 重闭，6/8；T-07 R2 复审待接） | worktree=D:/autostack/.wt/lang-794/auto-lang | branch=codex/plan-794-dev | base=8b97d3c9b | code=**d027e5f61**（91974d8f8 实施 + 7a566c89b R1 修复 + d027e5f61 基线重锚，树 clean + wt-guard clean） | deps=auto-down detached@c39ac7f 不变 | tasks=T-05/T-06 重闭（闭环行见 §8 R1 findings 闭环行） | findings 闭环：F-794-R1-01 → 干净提交 7a566c89b 上重采样基线（declared_edges=43 无新增禁向边）+ strict 复跑 pass（0 violations）+ 入库 d027e5f61 + 再复跑 pass → 无剩余；F-794-R1-02 → expect_reply 改每轮重按只读 Q* 按钮重跑 native + press 失败立即判负 + 样本序列入证据 + 桩式永久负控 + GoC3 重入队 pending→终态重放 → 修复后探针 **60/60**（原 57 case 全保留；复审原跑 56/57 红项 `v03_old_key_not_found_on_ch2` 现绿）→ 无剩余（Vue 探针同类单读模式为 781 fixture 观察项，未扩面）；F-794-R1-03 → 去无用 mut，`cargo t scroll_host_provider` 9/9 零 plan794 警告 → 无剩余。wt-guard 红项 → 逐链接 rmdir（356 junction 树内目标）+ 清生成树（p656/p781 gen、rust-workspace、dist）+ 杀探针遗留进程（vite node/esbuild/PID19128）→ guard **clean exit0** → 无剩余。材料同步=T-01 旧表 9903 弹序行改 delta→axis→handle（勘定史保留）。受影响重放：cargo t scroll_host_provider 9/9 零警告；VM 探针 60/60×2 跑（样本序列一致）；777 strict pass×2（基线入库前后）。merge-owned 门禁维持 pending（裸 t+tv 于新冻结提交 d027e5f61） | blockers=无 | next_responsible_role=review（T-07 R2：重放三 finding 修复+受影响验收+SD 冻结复核+wt-guard 复核，绑定 d027e5f61）

## 10. 待澄清事项

DTO具体字段在T01按现有state/anchor投影冻结；若必须改变UI状态机、feature缺席值或加入新跨线程调度，先出具体冲突修订，不能顺手发明新的scroll架构。
