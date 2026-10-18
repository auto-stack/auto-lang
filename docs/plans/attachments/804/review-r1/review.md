# PLAN-804 final review R1 — needs_fix

2026-10-17（Asia/Shanghai）。本轮为独立于实施上下文的验收复审；reviewer 为当前 Codex `/root`，未取得可验证的模型/会话标识，不虚构额外 reviewer。未创建 agent/新工作树，未修改实施代码，未合入。

## 冻结基线

- Plan revision 1；review_scope=final；受审工作树 `D:/autostack/.wt/lang-804/auto-lang`，branch `plan-804-dev`。
- reviewed_commit=`315ed5b5f325f8de12f7970981739cdb28ffdbd7`；base_commit=`b7ecea5e36ddf41d9effc49f0aba53cca47b99ad`；进入/结束均无工作树脏文件。
- auto-down 兄弟只读依赖=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。实际锁、测试配置、Spec 输入及原 Plan 哈希见 [baseline](baseline.json.txt)。根锁被忽略，以实际 SHA-256 绑定，未把锁的跟踪状态当成证据。
- 主检出在本轮期间从 `3732bc2c9f5582a141f4aa16beefe4c30a93c0f1` 前进到 `fcdcc99cc7c30d3f4f00f123fc7205a1114dd74c`；804 的代码 HEAD 未变。本轮读取主检出当前 Plan/Specs，没有向804树同步其他线代码。
- 技能来源：`C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md`；遵循仓库 2026-10-14 测试时序，仅执行定向验收，整档回归留 merge。

## Findings（均为原合同内缺口，未批准延期）

### F804-R1-01 / P2 — V04 的无父窗断言可误通过，真实 save 取消证据缺失

位置：`crates/auto-lang/src/tests/plan804_os_e2e.rs:624–633`，`v04_3_unparented_a2r_path_has_no_owner`。`drive_dialog` 已经在窗口存活时采集 `live_owner`，但测试将该值丢弃，等待模态函数返回、驱动 join 后才 `GetWindow(dlg, GW_OWNER)`。此时窗口已经销毁，返回 0 不能区分 parented/unparented。

独立反控 [owner-oracle-control.py](owner-oracle-control.py) 使用真实 Win32 owned popup：存活时 owner 与父窗相等；销毁后 `IsWindow=false`、`GetWindow(...,GW_OWNER)=0`，现有“无父窗”断言仍会通过。结果见 [JSON](owner-oracle-control.json.txt)。这是 oracle 缺陷，未声称当前 unparented 实现本身错误。

此外，`v04_2_parented_open_save_cancel_roundtrip` 实际只执行 open 接受、save 接受、open 取消；其余三个测试和 ui-dialog-only fixture 都只执行 open。合同 V04 明定“两者取消”，fake 返回空串不能替代真实 save 取消。修复应断言存活期 owner（建议通过保留的 `ui::dialog` 门面调用），补真实 save 取消；保留 parented 正控，证明错误的父窗策略会被该断言拒绝。

影响：AC-02/03，V04，T-04/T-05。

### F804-R1-02 / P2 — V05 完成通道读数没有覆盖完成事件送达发起 App

位置：`crates/auto-lang/src/tests/plan804_os_e2e.rs:1056–1216`。真实五会话确实经 shim→provider→DoDragDrop，验证了内容、线程、取消=true、effect 与取消后再次起拖；但测试只创建裸 VM 和接收窗，通过 `take_finished_effect()` 直接取走 DONE 队列，完全没有发起 App、另一焦点 App、subscription、`DesktopEvent::DndFinished` 或 `on_dnd_finished` handler。

因此 `Some("copy")` 旁“完成效果经事件通道归发起侧”的注释和 work 收据超出了实际断言。既有 `dnd_finished_delivery_anchors_at_initiator` 已独立复跑通过，但它人为传 `gen+1` 给辅助函数，未把真实会话接到 App 事件消费。合同 V05/AC-03 要求的“完成事件归发起者”仍未被真实链路证明。

修复：在实际宿主/App dispatch 链内发起会话，消费真实完成事件并观察发起 App 的 handler 结果；采用焦点在另一 App 的可区分控制，证明事件由发起者接收且不会被裸队列消费代替。保留本轮五会话证据作为 OS 层证据，不把它重命名为 App 层验收。

影响：AC-03，V05，T-05/T-06。

### F804-R1-03 / P2 — V06 将签名未变误判为792构建输入未变，遗漏必验重放

work 在 T-05 写“792 输入零变化（facade 签名不变→fresh/locked/重放无需重跑）”。实际交付实现 `crates/auto-man/src/rust_ui.rs:3997–4013,5170–5180` 明列整个 `auto-lang`，递归复制受管 crate 的所有源码。804 新增 `host_adapters/**` 并修改 `lib.rs`、`ui/dialog.rs`、VM、UI 装配文件；这些均是随包源码和真实编译输入。`ui-dialog` 形态还新增了 `lib.rs→host_adapters` 路径，签名不变不等于内容或编译闭包未变。

合同 V06 明确要求按真实入口更新并重放受影响的 fresh/locked/重定位/保护控制。计划和附件没有本提交的该项实跑证据，本轮没有用旧792 build结果替代。修复应更正输入分类，在804树用正式 `generated-ui-consumer/driver.py` 绑定当前 producer HEAD/锁/源 digest 跑受影响验收；如选择复用某些未受影响的保护控制，逐项说明代码、依赖、配置、输入未变的依据。与802及新master的最终合流仍由 merge 重核，不把这项 work-owned 遗漏后移到 T-08。

影响：AC-04，V06，T-05/T-06。

### F804-R1-04 / P3 — 新增格式/警告未清，线程描述纠错未落到源码

- `rustfmt --check --edition 2021 --config skip_children=true`：新增 `vm/host_capabilities.rs:488` 多余空行、新 fixture `tests/dialog_dnd_vm.rs:226` 起多处差异，均 exit 1，见 [格式结果](fmt-results.json.txt) 及各文件日志。renderer 的差异已经用 base 文件单独检查确认为预存，不列入本项返修。
- 本次新增 warning：`lib.rs:481` 和 `tests/plan804_dialog_dnd_provider.rs:243` 的 `///` 落到表达式/语句；e2e 的 `Rect`/`OwnWindow`/`pump_for`/`FindWindowExW` 等未使用，`IsWindow_` 非 snake case、两处多余 unsafe。见 [provider-default.log.txt](provider-default.log.txt)，不能以全库已有警告抵销新增警告。native-dnd 档额外有未用 `STGMEDIUM`。
- 计划点名要纠正的 `ui/native_dnd.rs:5` 仍写“专用 STA 线程”，`:559` 仍写“受理即返/线程拉起失败”，与同处实际输入线程同步实现冲突；此次 diff 未修改该文件。SD-03 的素材不能被当成源码说明已经纠正。

修复仅清理本计划新增项并纠正本族线程说明，不要求顺手格式化/清理全库预存差异。

影响：AC-04/05，T-03/T-04/T-06，SD-03。

## 验收映射

| AC | tasks / artifacts | 结果与覆盖边界 |
|---|---|---|
| AC-01 | T02/03；三 shim、双接口、外部 no-ui consumer | pass：无UI消费方7/7；shim只编解码/委托；无rfd/raw-window-handle/Win32/native_dnd调用；ID未变 |
| AC-02 | T02–05；缺席/栈/owned结果/feature-only/对话框 | partial：no-ui和真实open/save选取通过；V04 oracle与save取消缺口见F01 |
| AC-03 | T03–05；装配/重入/模态/拖出/事件 | partial：双槽、快照、clone/merge、兼容安装及OS五会话通过；App完成链未验见F02，V04见F01 |
| AC-04 | T03–06；去重/删边/合流/792/SD | partial：单一OS实现、旧门面及feature装配确认；792遗漏F03、卫生/线程说明F04；802/新master集成仍pending |
| AC-05 | T07 | fail：本轮 needs_fix；未授 reviewed |
| AC-06 | T08 | pending：merge-owned，未执行/未声称交付 |

补充覆盖限制：V03 重入测试在 dialog 回调中读 dialog 槽、写另一 dnd 槽；源代码检查确认 accessor 的 Arc clone 后读锁已释放，但该测试自身不能区分“持 dialog 读锁调用”的错误实现。同槽写反控、Drop 观察有助于补强寿命证据；本轮不单独将它们升级为新的交付范围。NI/VM 的目标槽保留与身份不变已逐代码核对。

## 本轮复跑结果（不把env-off测试计为真实OS证据）

| 入口 / 日志 | 实际结果 |
|---|---|
| `cargo test --manifest-path test/fixtures/modularity/dialog-dnd-provider-consumer/Cargo.toml` / [consumer-no-ui](consumer-no-ui.log.txt) | 7/7 |
| `cargo test -p auto-lang --lib plan804 -- --test-threads=1` / [provider-default](provider-default.log.txt) | 10/10，其中6项provider、4项OS测试env-off早退 |
| `AUTO_LANG_DIALOG_E2E=1 cargo test -p auto-lang --lib plan804_os_e2e::dialog_e2e -- --test-threads=1` / [dialog-real](dialog-real.log.txt) | 4/4，6.50s；F01说明断言范围 |
| `AUTO_LANG_DIALOG_E2E=1 cargo test --manifest-path test/fixtures/modularity/dialog-dnd-provider-consumer/Cargo.toml --features ui-dialog-host v04_ui_dialog_only -- --test-threads=1` / [dialog-only-real](dialog-only-real.log.txt) | 1/1，0.69s，真实无ui高层入口 |
| `AUTO_LANG_DND_E2E=1 cargo test -p auto-lang --features native-dnd --lib plan804_os_e2e::dnd_e2e -- --test-threads=1` / [dnd-real](dnd-real.log.txt) | 1/1，五会话4.71s；F02说明事件边界 |
| `cargo test -p auto-lang --features native-dnd --lib native_dnd:: -- --test-threads=1` / [dnd-components](dnd-components.log.txt) | 18/18；坏JSON/裸反斜杠/base64/空载荷/COM消费等 |
| `cargo test -p auto-lang --features native-dnd --lib dnd_finished_delivery_anchors_at_initiator -- --test-threads=1` / [helper](dnd-target-helper.log.txt) | 1/1，辅助函数级，不冒充实际App事件 |
| `cargo nextest run -p auto-lang --lib -E 'test(plan779) \| test(plan785) \| test(plan790) \| test(plan794)'` / [siblings](sibling-providers-no-wrapper.log.txt) | 23/23；首跑受sccache端口10013阻断，去RUSTC_WRAPPER后同测全绿 |
| `cargo nextest run -p auto-lang --lib --features native-dnd -E 'test(compat_install) \| test(filter_exts_frozen_semantics) \| test(owner_satisfies_rfd_parent_bound)'` / [adapters](adapter-controls.log.txt) | 8/8，两新族与旧族兼容安装、过滤、父窗类型 |
| `cargo check -p auto-lang` / [default](check-default.log.txt)；`--no-default-features --features ui-dialog` / [dialog-only](check-dialog-only.log.txt) | 均exit0，有预存和F04新增warning |
| `git diff --check b7ecea5e3 HEAD` / [log](diff-check.log.txt) | exit0；rustfmt结果另列，不能互相替代 |
| `python scripts/modularity-audit.py --mode observe/strict --output <review artifact>` | observe exit0；strict exit2：config_drift_since_baseline/stale_baseline，0新增禁止声明边、2 grandfather；主检出基线重绑属merge集成责任 |

## 规范增量审查与路由

冻结素材见 [spec-delta](spec-delta.md)，SHA见 [spec-delta.sha256.txt](spec-delta.sha256.txt)。SD01/02/04与代码一致：双owned能力槽、缺席矩阵、单一OS实现及VM/a2r父窗差异；SD03需要F04源码纠错。主检出已有799的交付闭包事实须保留，804不能回滚当前模块知识。

`supersedes_spec_components` / `new_spec_components` / `touched_goals` 的现有路径和 GOAL-003/007 合理，维持。未发布canonical Specs、未写ledger。

outcome=needs_fix；Plan回 `executing`，T03/T04/T05/T06重开，保留T01/T02，current_step=2/8。原work声称和测试历史均保留。next=在同一804树关闭F01..04，提交修复后复审受影响内容。

pending_closeout_gates：最终pass后在新冻结提交跑裸 `cargo t` + `cargo tv`；无ui_gen实现改动，不触发 `tu`，无aavm触面不触发 `taa`。另需master/802并集、777实际基线重核、受影响consumer证据、Spec/ledger/索引/归档和wt-guard清理。`tf`仅merge到期判定后主检出单实例批量运行。本轮不运行任何这些整档，也不授合入许可。
