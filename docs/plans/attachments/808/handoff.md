# PLAN-808 T-07 work handoff（终态送审收据）

2026-10-18。本文件记录 work 阶段的净迁移、owner 交接、证据入口与
merge-owned pending 门。代码/测试均在本计划唯一实施树：

- worktree: `D:/autostack/.wt/lang-808/auto-lang`
- branch: `codex/plan-808-dev`
- base: `2046a025b9a445d35a02da2c72a1613259a0cb60`（master）
- auto-down sibling: `D:/autostack/.wt/lang-808/auto-down` @ `c39ac7f`
  （detached，只读消费，不改源码）
- real DLL: `D:/autostack/auto-term/target/debug/autoterm_core.dll`
  SHA256 `c17ecca15a84f86cb26ad1f391490afe2e8b7420a369047c4bac3f6b8f0e8f58`
  （auto-term `main` @ `7a0d0e8b718fb260641f0d09ada95a7346771cd3`，clean）
- code commits: `3c02068ca`（首轮实现+测试）；`c346f1077`（777 首轮基线
  重绑）；`69961e046`（R1 findings 修复）；`69337f541`（R2 findings 修复）；
  `9f02a18dc`（R3 findings 修复）；
  **`b561796b5`（R4 findings 修复，当前冻结代码）**
- root `Cargo.lock`（gitignored）SHA256：
  `9a431eb9e6163c154798db92543028db38a29af70ddac008014cc48ebd6a8fe1`
  （新 member `auto-terminal-core` 已解析；仅 libloading 既有锁条目）

## 1. 净迁移（文件/行数）

| 面 | before | after | 说明 |
|---|---|---|---|
| `vm/ffi/term_engine.rs` | 1394 行 | 355 行 | 仅 34 shim + 2 处窗口门面委托；DLL/UI/采样业务清零 |
| `vm/ffi/term_engine.rs` 的 `crate::ui::` | 49 | 2 | 两处均为 `ui::terminal_provider` 门面委托 |
| 新 `crates/auto-terminal-core` | 0 | 1182 行（4 文件） | types/engine/provider/lib；闭包仅 libloading |
| 新 `ui/terminal_provider.rs` | 0 | 540 行 | UI view/input/palette/window adapter |
| `ui/terminal/mod.rs` | 1993 行 | 1951 行 | `TermColor/TermCell/几何/palette slots` 改 core 单份 re-export |
| 新 `tests/plan808_terminal_domain.rs` | 0 | 1168 行 | V-01..V-06 矩阵 |
| 独立 fixture `terminal-core-consumer` | 0 | Cargo + ~120 行 + verify-deps | 无 auto-lang/UI/man/auto-term 闭包 |
| 777 `core_to_ui` 文本候选 | 905 | 860 | observe 收据 `t07-777-observe.json`（-45） |

其他接线：`vm/host_capabilities.rs`（terminal 进程默认四级 API）、
`vm/native.rs`（NI 槽/Clone/new/accessor）、`vm/engine.rs`（构造快照/实例
set-get）、`lib.rs`（两处 if-absent 安装）、`ui/mod.rs`、`tests.rs`、
根/`auto-lang` Cargo（member + path dep）、`.autoos/module-boundaries.toml`
（`auto-terminal-core = runtime`）、`auto-man/rust_ui.rs`
（`DELIVERY_CRATES` 12→13，792 受影响闭包）。

## 2. 34 项 native 映射

ID/canonical/弹序/返回类型**全部不变**；逐项旧臂与新 provider 方法见
[contract-baseline.md](contract-baseline.md) §1。计数核对：catalog 34 条 =
shim 34 个 = provider 34 方法；plan808 `catalog_and_shims_match_34` 永久
钉扎。

## 3. 剩余 edge owner 表（不虚报整域完成）

| 残边 | 实际路径 | owner / 退出 |
|---|---|---|
| media asset/image_pipeline | `vm/ffi/stdlib.rs`（30 处） | R03 媒体完整子域（后续独立合同） |
| i18n | `vm/ffi/stdlib.rs::shim_i18n_t` → `ui::i18n_lookup` | R03 i18n 后续合同 |
| ~~diff 9915..9917/9921~~ | `vm/native.rs`/`ui_gen` | **已由 PLAN-807 迁出**（canonical 2026-10-19 已沉淀"文本 core 直调+快照端口"）；不列残边，merge 只与当前文本合流（R4-04） |
| a2r direct frame arms | `a2r_std.rs:1231/1237` | ui/overview 既有未迁边界 |
| terminal UI 注册表/输入/选中/菜单/滚动投影 | `ui/terminal/mod.rs`、`iced/widget.rs`、`ui/dynamic.rs:2587` | UI 视图/输入 owner（本计划只改消费路径） |
| a2c/a2r 原生 engine C-ABI 语料 | `test/a2c/18_c_interop/004_engine_face`、`test/a2r/27_c_abi/*` | 不在 34 native 支持面；不改 |
| `kill`/`set_palette`/`shell_pid` | DLL 导出但无 auto-lang native | 原支持面之外；不新增绑定 |

## 4. 验证证据（work-owned，全部在本树当前代码）

| 门 | 命令 | 结果 |
|---|---|---|
| core check | `cargo check -p auto-terminal-core` | exit 0 |
| core tests | `cargo test -p auto-terminal-core` | **7/7 pass**（R3：retired store + lease/free/隔离） |
| core+lang check | `cargo check -p auto-terminal-core -p auto-lang` | exit 0 |
| Ui terminal 定向 | `cargo test -p auto-lang --lib ui::terminal -- --test-threads=1` | 41/41 pass |
| plan808（真实 DLL） | `AUTOTERM_ENGINE_DLL=... cargo test -p auto-lang --lib plan808 -- --test-threads=1` | **24/24 pass**（R4：22 原例 + 复活路由 `revived_provider_never_leaves_owner_list_during_prune` + 无界 retired `retired_values_survive_unbounded_later_retirements`） |
| 独立 consumer | `cargo test --manifest-path test/fixtures/modularity/terminal-core-consumer/Cargo.toml`（DLL env） | 4/4 pass |
| consumer 闭包 | `bash test/fixtures/modularity/terminal-core-consumer/verify-deps.sh` | PASS（仅 libloading/windows-link） |
| no-UI 构建 | `cargo check -p auto-lang --no-default-features` | exit 0（34 shim + core 服务无 UI 依赖；config/window 1024×768 缺席臂保留） |
| ui-only 组合 | `cargo check -p auto-lang --no-default-features --features ui` | 预存红（`ui_gen/vue.rs` iced、`sched_diag`/`vm_bridge` session/dynamic 未门控——F-785-P01 既有缺陷，非本计划文件；默认含 ui-iced 档绿） |
| 真实 DLL 有界探针 | `python docs/plans/attachments/808/t01-dll-probe.py` | 34 checks pass |
| 777 strict | `python scripts/modularity-audit.py --mode strict --no-cargo-check` | **pass，0 violations / 2 grandfathered**（`r1-777-strict.json.txt`；core_to_ui 868） |
| 792 delivery driver | `python test/fixtures/modularity/generated-ui-consumer/driver.py` | 见 `r1-792-driver.log.txt` / `r1-792-driver-receipt.json.txt`（R1 代码重跑：**25/25**） |
| 复审原反例重放 | `powershell -File attachments/808/review-r1/replay.ps1` | **全部期望值命中**（`r1-replay.log.txt`）：ha=1/hb=2、A_live=0/B_live=1、in-flight free=0、并发 free=1 次、drop free=1 次、VmBridge 装 adapter=true、泵 1/0、真实 child free/drop 均 false（已回收） |
| R2 重入反例重放 | `powershell -File attachments/808/review-r2/run-callback-probe.ps1` | **8/8 case completed_before_3s=true exit=0**（`r2-callback-replay.log.txt`；owns/count/drop/slot-drop × direct-control/非 control） |
| R2 缓存反例重放 | `powershell -File attachments/808/review-r2/build-probes.ps1` | **四行全部期望值**（`r2-cache-replay.log.txt`）：free 前后/替换后均 80×24，switch→free 均 90×30 |
| R3 组合反例重放 | `powershell -File attachments/808/review-r3/run-combinations.ps1` | **全部期望值**（`r3-combination-replay.log.txt`）：剪枝暂停期 owner=true/view=(80,24)/A_live=0；跨 VM 全 91×31；直接 free 全 92×32 |
| R4 修复路径重放 | `powershell -File attachments/808/r4-repair-replay.ps1` | **全部期望值**（`r4-repair-replay.log.txt`）：stale 零判定+重装/真实 spawn 交错 → after_verdict owner=true/(83,27)/(83,27)、真实 free 后 A_live=0、冻结 (83,27)；4096 条无关退休后 global=Some(94×34)/late=(94,34)/early=(94,34)。R4 代码上 R3 combination（`r4-replay-r3-combination.log.txt`）与 R2 callback/cache（`r4-replay-r2-callback.log.txt`/`r4-replay-r2-cache.log.txt`）重放仍全期望 |
| fmt | `rustfmt --check` 触面文件 | 0 diff |
| 生成 Rust terminal 金样 | `cargo test -p auto-lang --lib terminal_`（3 金样名） | 3/3 pass（该过滤面 25/26；唯一红 `plan705_client_total_deadline_terminal_error` 为并发负载下时序 flake，solo 复跑 1/1 绿、不属 terminal 面） |

真实 DLL 测试在缺 `AUTOTERM_ENGINE_DLL` 时打印显式 `PENDING(plan808)`，
不计为通过；work 证据以 env 开启运行的结果为准。

### 4.1 R1 复审 finding 闭合表（commit `69961e046`）

| Finding | 不变量 | 修复 | 永久正负控制 |
|---|---|---|---|
| F808-R1-01 P1 旧句柄误释放新服务 | 已暴露句柄在 provider 替换后仍绑定原 owner | 进程全局 handle 命名空间；NI 保留 provider 历史；带 handle 的 shim 用 `owns_handle` 反查原 owner 再回落当前/共享 | plan808 `provider_replacement_keeps_old_handle_bound_to_owner`：A/B 各有活会话；切换后旧柄读/写/释放只动 A，重复 free 不动 B；第二 VM 与共享回落；core 全局唯一 handle |
| F808-R1-02 P1 VmBridge 漏装 adapter | 生产组件构造入口先装 adapter 再建 VM | `new_with_children`/`new_from_decls` 加 `terminal_provider::install_process_default()` | plan808 `both_bridge_constructors_install_terminal_adapter`：清空默认槽后真实构造两个 bridge，槽 Some、bridge VM 快照 Some、bridge 自己的 VM 泵排空输入；显式默认优先 |
| F808-R1-03 P1 并发 free 双进 DLL | 每会话恰好一次 free，不与在途操作重叠 | `SessionTable`：表锁不跨门；lease/retire；free 先摘除、等门、置退役、调 DLL 一次 | core `session_leases_serialize_free_and_never_double_free`：在途 lease 阻塞 free；free 后 lease absent；两线程并发 free 恰一次动作；`different_sessions_do_not_block_each_other` |
| F808-R1-04 P1 service drop 不释放会话 | 末引用释放/替换剪枝时释放会话，子进程退出 | `Drop for EngineService` 在 Library 析构前逐会话 free；owner 剪枝释放旧 service | plan808 `service_drop_frees_live_session_and_child` + `provider_replacement_last_reference_releases_old_service`：PowerShell 子进程自报 PID；drop/末引用释放后 OS 进程消失（非仅表归零） |
| F808-R1-05 P3 两条新警告 | 本计划新增文件零警告 | 删除未用 `TerminalHostProvider` 导入与多余 `mut` | `rustfmt --check` + 定向编译无本计划文件 warning |

### 4.2 R2 复审 finding 闭合表（commit `69337f541`）

| Finding | 不变量 | 修复 | 永久正负控制 |
|---|---|---|---|
| F808-R2-01 P1 NI 锁内执行 provider 回调/析构（4 路重入死锁） | 任何 NI 锁内不得执行 provider 方法或析构 | `terminal_owner_for`/`terminal_cache_owner_for` 锁内只快照 Arc 列表、锁外查询，generation 变化重试；`prune_terminal_owners` 锁内摘下列表、锁外 `live_handle_count`/析构，锁外求值后把并发新增合并回；`set_terminal_provider` 锁内只换槽、旧 provider 锁外析构；`remember` 重复项锁外释放 | plan808 `plan808_reentrancy`：owns/count/drop/slot-drop 四路重入均 3s 内完成；四路 direct 正控；并发 set/query/prune 保活 owner。复审原 `callback-probe` 重放 8/8 完成 |
| F808-R2-02 P2 free 后视口缓存经 provider 切换变 0×0 | retired 句柄的 cursor/viewport 冻结值在切换/剪枝/销毁后仍可读 | shim free 在 owner 剪枝/销毁前把 cache 复制进 per-VM retired 表；缓存 shim 解析序 = retired 表 → 活 owner → retired owner（懒复制）→ 当前 provider；只存纯值，不持旧 service、不触 DLL 指针 | plan808 `freed_handle_cache_survives_provider_switch`：free→switch 80×24、switch→free 90×30 + 非零 cursor、重复 free、未知柄 0×0。复审原 `cache-probe` 四行全部期望值 |
| F808-R2-03 P3 修复轮新增两条 `unused_mut` | 新增文件零未处理 warning | `reaper_a`/`reaper_b` 去掉 `mut` | 定向编译无本计划文件 warning |

### 4.3 第二次失败诊断（review 要求）

| 维度 | 内容 |
|---|---|
| 剩余不变量 | ① NI 锁外执行 provider 方法/析构（查询/剪枝/槽替换）；② live/retired 句柄的 owner 语义分离：live 走 `owns_handle`，retired 的 cursor/viewport 值必须仍可按冻结臂读取；③ 强 owner 保留（句柄存活）与资源释放（free/剪枝/Drop）互不矛盾 |
| 之前为何漏检 | R1 只验证“活句柄 owner 路由”和“live_handle_count=0 剪枝”，oracle 以 live-only `owns_handle` 为唯一入口；没有 retired/unknown 句柄断言，也没有会重入的 provider。绿色 13/13 无法证明“锁外回调”与“free 后缓存”两类规则 |
| 受影响路径/状态 | `terminal_owner_for`（查询）、`prune_terminal_owners`（live_count/移除析构）、`set_terminal_provider`（旧槽析构）、`remember_terminal_owner`（重复项释放）；handle 状态 = live / retired / unknown；替换/free 两种顺序 |
| 设计/oracle 调整 | 查询=快照+generation 重试；剪枝=摘除→锁外求值/析构→合并并发新增；槽替换=换值→锁外析构。缓存：新增 `TerminalRetiredCache` 值拷贝 + per-VM retired 表，缓存 shim 四段解析；oracle 从 live-only 扩为 live/retired/unknown 三态 + 两种顺序 + 重入 provider/并发 set-query-prune |
| 当前结果 | 复审原 `callback-probe` 8/8 完成、`cache-probe` 四行期望值；plan808 20/20（真实 DLL）；core 6/6；UI 41/41；no-default 0；consumer 4/4；777 strict pass；792 25/25（R2 代码重跑） |

### 4.4 R3 复审 finding 闭合表与诊断（commit `9f02a18dc`）

| Finding | 不变量 | 修复 | 永久正负控制 |
|---|---|---|---|
| F808-R3-01 P1 剪枝清空列表导致窗口内真实 free 丢失 | 锁外求值期间可见 owner 列表必须保持可解析；每个真实操作在窗口内都成功 | `prune_terminal_owners` 快照**不摘除**可见列表 → 锁外 `live_handle_count` → 短写锁只移除求值死亡项（锁内零 provider 调用）→ 锁外对移除项复检，复活者重加；generation 仅在实际变更时前进 | plan808 `prune_window_keeps_owner_routable_and_free_releases`：真实 DLL + `PausedCount` 在 `live_handle_count` 暂停；暂停期 owner=true、viewport=80×24、`shim_free` 后 A_live=0；并发 set/query/prune 测试改为**每次查询都必须命中**。复审 `combination-probe`：during_prune true/(80,24)/0 |
| F808-R3-02 P2 跨 VM/直接 free 的消费者未预读则丢旧缓存 | 冻结 cursor/viewport 值对任何消费者在 owner 剪枝/销毁后仍可读，且不持旧 service | core `free` 在退休前把纯值写入**进程级 retired 缓存**（有界 FIFO，`retired.rs`；**R4-02 裁定该上限未获授权、R4 轮已改无界**，见 §4.5）；shim free 另存 per-VM 表；缓存 shim 顺序 = per-VM → 活 owner → retired owner → **进程级** → 当前 provider | plan808 `retired_cache_is_shared_across_consumers_and_direct_free`：VM1 free 共享 91×31 / 直接 core free 92×32；另一 VM 未预读、未知柄 free 触发剪枝后仍读 91×31/92×32 + 非零 cursor；含预读正控、重复/未知 free；复审 `combination-probe` 全期望值 |

**第三次失败诊断**：

| 维度 | 内容 |
|---|---|
| 漏检根因 1 | R2 并发测试只断言“500 次查询中至少成功一次 + 最终列表恢复”，没有把**窗口内的每次真实读/write/free**作为 oracle。剪枝把列表清空后，窗口内所有操作都误路由，但最终列表恢复使测试通过 |
| 漏检根因 2 | retired 缓存只写入“执行 free 的 VM”并由“首次查询”懒复制；其他 VM 和 direct core free 从未写入，剪枝删掉 owner 历史后无值可读。测试只覆盖同 VM 的 free→switch 顺序 |
| 设计/oracle 调整 | 剪枝：快照不摘除 + 锁外求值 + 短锁移除（零 provider 调用）+ 复活重加；缓存：core free 发布进程级纯值，shim 多一段全局回落。oracle 改为“窗口内真实操作逐个成功”与“跨 VM/直接 free/预读正控/重复/未知 free 全矩阵” |
| 当前结果 | 复审 `combination-probe` 全期望值；plan808 22/22（真实 DLL）；core 7/7；R2 callback/cache 重放仍全过 |

### 4.5 R4 复审 finding 闭合表与第四次诊断（commit `b561796b5`）

| Finding | 不变量 | 修复 | 永久正负控制 |
|---|---|---|---|
| F808-R4-01 P1 合法零值判定后 provider 复活，移除→重加窗口内真实调用不可路由/free 丢失 | 死项判定采样后发生的**重装与经 spawn shim 的会话发布**必须与移除协调：复活者在整个窗口连续可路由；不得以“事后重加/重试/sleep”替代 | owners 列表条目化（`TerminalOwnerEntry` = provider Arc + revival 计数）：`set_terminal_provider` 重装与 spawn shim 成功发布都 bump 所在条目计数；`prune_terminal_owners` 对每条目先读 stamp 再锁外 `live_handle_count`，短写锁内**仅 stamp 未变才移除**（锁内零 provider 调用，只读条目原子；bump 与检查同写锁互斥）。复活者从未离开列表；有效移除后又复活者由发布路径重新入列（shim 返回句柄前 remember）；不再有“移除→复检→重加”窗口 | plan808 `revived_provider_never_leaves_owner_list_during_prune`（真实 DLL）：`PausedCount::new_stale_zero` 采样时 A 真空、暂停期重装+真实 `shim_term_spawn_ex` 83×27+切回 B → 判定返回陈旧 0 → prune 完成：owner=true、VM(槽=B)/另一 VM 视口均 (83,27)、真实 free 后 A_live=0、冻结值保持。复审 `r4-repair-replay`：after_verdict owner=true/(83,27)/(83,27)、after_free A_live=0 |
| F808-R4-02 P2 进程级 retired 缓存 4096 条 FIFO 淘汰仍可查询的冻结值（未授权语义缩减） | 仍可按句柄查询的冻结 cursor/viewport 值不得因**无关退休**被淘汰；旧 `CURSORS`/`VIEWPORTS` 静态表从不淘汰，任何上限都需授权修订 | `retired.rs` 撤销 FIFO 上限：无界 HashMap（句柄进程全局唯一不别名；条目为小纯值；容量=进程内实际退休会话数）；未经授权不得自加容量策略 | core `retired_store_keeps_all_values_without_eviction`（5000 条后首条仍在，全量断言）；plan808 `retired_values_survive_unbounded_later_retirements`（真实 DLL）：真实 94×34 会话、未预读 late VM、owner 剪枝后 4097 条无关退休（公表面 `set_viewport`+`free`，无额外 PTY）→ late 仍读 (94,34)。复审 `r4-repair-replay`：第 4096 条后 global=Some(94×34)/late=(94,34)/early=(94,34) |
| F808-R4-03 P3 新测试三条 `unused_mut` | 本计划新增文件零未处理 warning | `:1702/:1754/:1798` 三处 `mut reaper`/`mut reaper2` 去掉 `mut`（`disarm` 按值收参，绑定不需 mut） | test-profile 定向编译 plan808 文件零 warning |
| F808-R4-04 P2 SD-03/残边表仍把 807 已迁 diff 族列为未迁（并发 canonical 输入更新） | 拟沉淀文本与当前 canonical（PLAN-807 已合入 @`6955e3178`）不得冲突；merge 不得盲写旧残边覆盖当前知识 | spec-delta SD-01/SD-02/SD-03 与「不在本域」去除 diff 残边并注明 807 已迁；SD-03 增**合流指令**（以当前 canonical 为基底、`term/media` 收窄为 `media`、保留 807 注记与 PLAN-807 节）；handoff 残边表 diff 行标已迁；contract-baseline §3.1.6/4.2/8 同步 R4 协议 | 本轮不私改 canonical/ledger；merge 时按合流指令应用 |

**第四次失败诊断**：

| 维度 | 内容 |
|---|---|
| 漏检根因 1 | R3 修复的 oracle 只覆盖“锁外求值期列表完整”（求值期暂停）与“求值后重加”，没有覆盖**零值判定合法返回后与重装/新 spawn 的交错**——判定线程被抢占后返回陈旧 0 是合法时序，移除只按 Arc 身份、不校验判定有效性；“移除→复检→重加”的复检期 provider 不在可见列表，窗口内 owner=false/free 误路由丢单 |
| 漏检根因 2 | R3 引入进程级 retired 缓存时**未经语义对照自加了 4096 FIFO 上限**：旧静态 `CURSORS`/`VIEWPORTS` 从不淘汰，原合同“free 后保留旧值”没有容量边界；边界测试只插 1 条，未断言容量/淘汰行为 |
| 设计/oracle 调整 | 剪枝协议从“身份移除+事后重加”改为“revival stamp 条件移除”（判定可失效化，复活者从不离列）；retired 缓存恢复无界语义。oracle 补：零→活状态转换（stale 零判定 + 重装/真实 spawn 与剪枝交错）、移除路径全窗口读/free、**无关退休任意条数后冻结值仍在**（含 4096 边界）|
| 当前结果 | `r4-repair-replay` 全期望（owner=true/(83,27)/(83,27)/A_live=0；4096 条后 Some(94×34)/late=(94,34)）；R3 combination/R2 callback+cache 重放仍全过；plan808 真实 DLL **24/24**；core **7/7**（新 no-eviction）；UI 41/41 |

## 5. 计划/规范交接

- SD 草案：[spec-delta.md](spec-delta.md)（SD-01 add auto-terminal-core/
  project.md；SD-02 vm overview；SD-03 ui overview；SD-04 terminal-mux-model）。
- 777 基线：首轮 manifest/config 已在干净提交 `3c02068ca` 上重绑
  （`c346f1077`）。R1 修复未改 manifest/config，在 `69961e046` 上 strict
  **pass**（0 violations / 2 grandfathered，`r1-777-strict.json.txt`）；
  merge 在最终 master 组合上再重绑。
- 792：`DELIVERY_CRATES` 已加 `auto-terminal-core`（13 项，生成
  crates=14）；R3 代码上 driver **25/25**（fresh 175s / relocated locked /
  负控 / dev 路由；`r3-792-driver-receipt.json.txt`）。merge 合流其它新
  成员/闭包变化时按 792 driver 重放受影响 fresh/locked。
- 收据持久化：原先被 `.gitignore` 的 `*.json` 已改为 `.json.txt`
  （`t07-777-observe.json.txt`、`t07-792-driver-receipt.json.txt`、
  `r1-777-strict.json.txt`、`r1-792-driver-receipt.json.txt`）随簿记入库。
- ledger/索引/归档：未改（merge-owned）。

## 6. merge-owned pending 门（不得在本阶段宣称交付）

- 冻结提交上裸 `cargo t` + `cargo tv`；实际触面含 `ui_gen`? 本计划未改
  `ui_gen` 发射算法（仅消费侧不变），按计划不机械触发 `cargo tu`；若
  merge diff 实际触及 `ui_gen/**` 再补。
- 777 strict 在最终组合树上 `--refresh-baseline` 重绑后 pass。
- 792 driver 在最终组合树上重放受影响 fresh/locked（若闭包变化）。
- Spec/ledger/index/归档 + wt-guard clean 后清理。

## 7. 未决/风险

- T-01 已冻结的旧臂残留：free 后 CURSORS/VIEWPORTS 保留旧值（旧可观察
  值；单调句柄不复用）。plan808 real-DLL 测试断言 free 后 rows 空/
  is_exited=true，不依赖 cursor/viewport 清零。
- 真实 DLL/PTY 生命周期依赖机器环境；无 `AUTOTERM_ENGINE_DLL` 的运行只
  产生 PENDING 证据，需在 review/merge 环境复跑同一命令。
- a2r 原生 engine C-ABI 测试（`a2c`/`a2r` 语料）不属 34 native 面，未被
  本计划修改；若合并窗口其它计划触碰 engine ABI，需复核。
- 媒体/i18n 等 R03 残边未迁；本计划完成不等于 R03 整包完成。
