# PLAN-724 T-08 验证报告：AC/SD 逐项证据、遗漏/延后/workaround 扫描

- 阶段：work（T-08，复审前置自查）
- 载具：worktree `D:/autostack/.wt/lang-724/auto-lang`，分支 `plan-724-dev`
- 提交链：`7e7c6b3f8`（T-02）→ `14e9cc389`（T-03/T-04 内核）→ T-05/T-06 →
  T-07 集成矩阵 → 报告批（见 `git log plan-724-dev`）

## 1. AC 逐项证据

| AC | 通过条件 | 证据（全部实测，命令在案） |
|---|---|---|
| AC-01 | 两 facade 共用网络内核；§5.1 必选映射完整；普通/认证 post 不冲突；header/body/status 正确 | 决策报告 §1 矩阵 21 面全登记并有证据列；全仓扫描 `reqwest::blocking\|spawn_blocking` 仅存文档注释（两 facade 零重复网络执行）；e2e sync/async/qualified 三腿 wire 断言（含元数分派探针钉 `plan724_probe_post_arity_dispatch_shapes`）；`Response.header_get` 大小写无关实测（内核 roundtrip + native integration） |
| AC-02 | 必选 async 面/块/流循环不阻塞 reactor；无 blocking client/spawn_blocking/同步 recv/join/每请求 runtime/thread；两种 Tokio runtime 健康事件先于 gate 放行；不支持调用有响亮诊断 | 内核 `plan724_kernel_current_thread_consumer_reactor_stays_live`（慢上游 ≥500ms 期间定时器 ≥10 tick）+ `…_stream_wait_yields`；integration `plan724_native_two_runtime_consumers_healthy_order`（current-thread + multi-thread）；源代码无 spawn_blocking/每请求线程（grep 在案）；`plan724_kernel_sync_bridge_panics_loudly_in_async_ctx` 钉响亮 panic；转译期上下文分派使生成代码不落该路径（002 golden） |
| AC-03 | 准入/体/队列/事件/carry 守预算；超限/超时/队满可观察终结；EOF 排空；控制串不作内核状态 | `plan724_kernel_queue_full_is_immediate_terminal`/`body_budget_observable_terminal`/`stream_item_budget_terminal`/`stream_open_timeout_terminal`/`stream_queue_full_fails_typed`/`backpressure_high_watermark_bounded`；EOF 排空+单次终结（raw/sse/integration utf8 三面）；内核状态机零控制串（typed `Option<u16>` status + `StreamItem::Data/Eof/Failed`）；资源报告算式在案 |
| AC-04 | 五阶段取消实际停止 producer；许可回基线；幂等/竞态不复活；错误可查；metadata 不串扰 | `cancel_dropped_future…`（非流式）/`stream_close_cancels…`（流，幂等双 close）/`native_cancel_storm…`（60 流，回收后可再准入）/`break_then_close_lifecycle`（break 不冒充关闭）；queued 阶段取消=abort 未轮询任务（storm 覆盖）；opening=`stream_open_timeout_terminal` 通路 + close-before-connect（单元 close 测试早于连接）；每请求 metadata 归属本请求（typed 字段，无线程局部关联——`last_status` 仅同步兼容信息，async 消费用返回值） |
| AC-05 | 必选同源 VM/a2r/原生 Rust 约定语义一致；两种 runtime 名称实编实跑；遗留同步认证保持；未承诺范围明确 | 对拍报告 §2/§3：同源 .at 产物实编实跑（cargo run 退出码 0）+ 原生 typed 交叉核验（integration）+ 限定名 `auto_lang::a2r_std::http` 真 TCP 腿；遗留同步认证 tuple 形状逐字节保留（qualified 腿 + T-02 e4 复用同一发射面）；差异单列（sse id 族/VM JSON headers 旧行为/last_status 语义） |
| AC-06 | P707-R1 根因确认后红转绿且有公开入口 wire 覆盖；未登记仍中止；712 保持；门禁无新增红；债务清偿 | T-02：修复前 30.0s 确定性红（复现日志在案）→ 修复后 0.46s 绿；登记→提交→消费配对（plain 臂验 `Structured{200}`）+ 未登记反例钉（句柄回收+无令牌无完成）；取消守卫零改动（`plan707_cancel` 5/5 复证）；plan712 2/2；`KNOWN-DEBT-AND-RISKS.md` P707-R1 已凭证据销号 |

## 2. 门禁结果（worktree，plan §6 档位）

| 档 | 命令 | 结果 |
|---|---|---|
| a2r-std 全量 | `cargo test -p a2r-std` | 36+7 lib/integration 全绿（43/43 nextest 复证 ×2） |
| tv（触 VM 源：stdlib.rs e4） | `cargo tv` | 162/162 绿 |
| tt | `cargo tt` | 1831/1834 绿；3 红=musk_vm_track p053 族——**预存基线红**（主检出 986e765ac 同红复现在案；KNOWN-DEBT 712 §10②「master 14 红族 p053×4」在案） |
| 裸 t（日档） | `cargo t` + `-E "not test(musk_vm_track)"` no-fail-fast | 红 set 与主检出基线**逐名一致**（p053/plan606/plan502/desktop_protocol/iced renderer/schema_drift×2/docs_gen——全部基线同红）；差异仅 e4：基线红（30s）→ worktree 绿（T-02 修复）；`plan707_wait_generator_park_drive_count_static` 基线批跑红、两侧隔离均绿=在案「并行负载序敏感」flake 族，非回归 |
| th | `cargo th` | 34/36 绿 + 2 红=back_proxy/relay 帧时序——主检出基线 `cargo th` 同红复现（逐名一致）；本计划新增 `http_e2e_plan724_*` 3 例全绿 |
| tu / auto-man | 未触 `ui_gen/**`、未改 auto-man | 按计划 §6 豁免，不跑 |

**确定性红归因纪律**：以上所有非绿项均给出主检出基线同红复现或在案债务
锚点，无「笼统归为环境原因」。

## 3. 健康：警告 / 格式 / 调试输出

- 新增警告：`cargo check -p a2r-std -p auto-lang` 中我触面文件零警告
  （现存警告全部位于未触文件，基线在案）。
- 调试输出：diff 扫描无非测试代码 `dbg!/println!/eprintln!` 新增（测试内
  eprintln 仅 storm 诊断注释行，属测试自身输出面）。
- 格式：触面文件与基线同风格（基线 `rustfmt --check` 本就不净：trans/rust.rs、
  stdlib.rs、a2r_std.rs、fs.rs、actor_parity.rs 基线版本同 diff——本仓不全局
  强制 rustfmt）；新增代码遵循周边风格，无新差异类别。

## 4. 遗漏 / 延后 / workaround 扫描

- **无未批准延后**：计划 §8 T-01..T-08 全部执行；§5.1 矩阵 21 面全部落地或
  显式登记为不支持（sse id 族——决策 §1 #21，属「冻结边界」非遗漏）。
- **无 workaround**：T-02 未削弱取消守卫（反例钉+守卫复证）；内核未用
  spawn_blocking/临时线程兜底；流 close 真取消非占位。
- **诚实边界**（决策 §5 + 对拍 §4）：同步 helper 被 async 路径调用=响亮
  panic；`stream_iter` 脱离 for-in 无 a2r 形态；VM shim JSON headers 旧行为
  未动；api_gen 的 `#[api] ~Stream` handler 生成另案；文件 helper 仍 ureq。
- **Spec delta 对齐**：SD-01..SD-06 提案维持计划表（目标文件与 before/after
  规则已按实际交付核对——`a2r-std/design/http-client-runtime.md`（新）、
  `trans/design/http-client-lowering.md`（新）、a2r-std/project、stdlib 三
  设计、trans/overview 的修改点与本报告证据一致）；canonical Spec 编辑待
  review 验证后由 merge 沉淀（本阶段未改 canonical Specs）。

## 5. 结论

全部 AC/SD 有实测证据；无未批准缺项；P707-R1 处置可核对（红→绿日志、
守卫复证、债务销号）。具备进入 `/auto-plan:review` 独立复审的条件。
