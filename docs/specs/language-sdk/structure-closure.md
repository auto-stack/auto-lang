# 结构收口合同：运行桥接、stdlib 装配与 Rust 转译职责（PLAN-884）

> **Status**: active（PLAN-884 交付；2026-10-29）
> 路径：`crates/auto-lang-core/src/{entry,vm/ffi,trans/rust}` | 上游合同：[project.md](project.md)
> 结构总则：[architecture.md](architecture.md) | 消费者与发布边界：[project.md](project.md) §真实消费者

本页记录 PLAN-884 实际完成的**有限结构收口**：在保留 crate 粒度、默认特性、依赖与 ABI 的前提下，
把四类职责收敛到唯一 owner。它是**结构事实**，不主张性能收益，也不把未完成的减法记作已达标。

## 1. 范围与守恒原则

- 保持 `auto-lang-core` 单包单语言实现与 `auto-lang` 组合/再导出分层；**不拆 member、不改
  default/features/manifest 依赖、不改 ABI/CALL 约定**（拆分决策仍由
  [build-cost-baseline](../repository-boundaries/build-cost-baseline.md) 的 THR-D1 结论约束）。
- 只合并**已证明同机制**的实现；语义分叉、策略不同、通道不同的路径保持分离。旧实现不留
  第二份永久 fallback 副本；经审计确认不可达的块按"不能静默丢失"原则逐字保留并登记（§8）。
- 守恒对象：公开 `pub` API 面与类型身份、run 结果/stdout/元数据、typed trap 分类、错误类别与消息、
  872 绑定表（名字/ID/生效实现/覆盖序）、a2r 生成字节与资源清单、源码指纹输入闭包。
- 规模数字（§6）为同口径物理行**记录**，不是验收阈值；843 自设行数目标不再追认。

## 2. 运行桥接单源（`entry/run_bridge.rs`）

`entry.rs` 仍是全部公开 run 入口的家（签名/返回/文档不变），只把**同构机制**下沉到
`entry/run_bridge.rs`（`pub(super)`，不进 crate 公开面）；入口侧只做受审计的参数绑定与元组投影。

| 组 | 入口族 | 桥接 |
|---|---|---|
| G1/G2/G3 同构 | `run_with_capture_and_(path_)bytecode[_with_meta/_with_deadline/_with_input]`、`run_autovm`、`run_autovm_capture` | `run_on_dedicated_thread(code, capture, path, deadline, input)` 单源；capture/path/deadline/input 是显式参数，不在此硬编码 |
| G4 选择器 | 上层 `run`/`run_with_capture`（`ExecutionEngine` 选择 + 弃用告警副作用） | 保留独立 facade，不并入 G1/G3 |
| G5 staged | `run_with_capture_and_path_staged` | `run_staged_on_dedicated_thread` 独立：返回 `StagedRunFailure`（非 `join_execution`），**不对称保持** |

桥接合同（改一处即须复核全部入口）：

- owned clone（`code`/`path`）后 spawn 专用线程，栈=`vm_thread_stack_size()`（`RUST_MIN_STACK>=4MiB`
  时沿用，否则 16MiB）；线程内 `block_on_autovm_local` 驱动 LocalSet，不放出 `!Send` VM。
- spawn 失败保持 `.expect("Failed to spawn execution thread")`（真 panic，不转 `Err`）；
  join panic 保持 `Err(Msg("execution thread panicked"))`；两种基础设施失败都不是 typed trap。
- staged 通道的 `LAST_RUN_TRAP_CODE` **必须在 worker 闭包内、join 之前** take（线程亡于 join），
  `StagedRunFailure.run_trap_code` 只承载原 typed VM trap；不把解析错误文本升级成 trap。
- 不吞错、不把 deadline 变成功、不新增即时取消语义；请求本地 root/输入在异常与正常退出都恢复
  （`RequestFileRootGuard` 仍由 engine 侧按 `input.is_some()` 装配，entry 不直触）。
- 非 run 职责（parse facade、module 解析、bootstrap、`eval_config_with_vm`、文件测试引擎、
  bytecode 池重映射、FFI 装配）留在 `entry.rs`，未因同文件而改写；其调用桥若实质变化须纳入
  受影响消费者复核。

## 3. stdlib 域装配（`vm/ffi/stdlib*`）

- **入口只装配**：`vm/ffi/stdlib.rs` 壳（约 2278 行）保留 Native ID 常量带、状态声明、轻包装、
  受审计的组合调用与跨域测试模块；算法/数据/真实 shim 归 `vm/ffi/stdlib/<domain>.rs`。
- **域文件（13）**：`file_io`、`http_client`、`crash_trace`、`strings_json`、`math`、`image`、
  `sqlite`、`sys_env`（843 期）+ `request_root`、`storage`、`exit_audit`、`http_async`、
  `rust_stdlib_dispatch`（884 期新增，迁自壳内基础设施段）。
- **注册按域拆分但保持语句序**：`register_stdlib_ffi` 只剩 19 个域 helper 的有序调用，调用序=原
  连续语句序（plan872 有效绑定序载荷）；musk extern 注册尾段仍就地（跨模块 extern 族，见 884-N3）。
  `production()` 六步序（catalog → 手工 → interop → inventory 覆写 → request I/O → 具名 override/
  契约）不变，允许"同 ID 后写覆盖"的既有裁定不变。
- **状态唯一 owner**：`REQUEST_FILE_ROOT` TLS、`STORAGE_MAP`/DIRTY、`JSON_DOC_ORIGINALS`、
  `NET_HANDLE_COUNTER`、`HTTP_ROUTES`、`TCP_LISTENERS`/`STREAMS`、`HTTP_RESPONSES`、
  `ASYNC_STREAMS`、`SESSIONS`、`MIDDLEWARE_CHAIN`、`TEMPLATES`、`EVENT_BUS`、
  `IMAGE_TICKET_HANDLES`、`TASK_*`、`SQLITE_HANDLE_COUNTER`、`WARNED_API_NOOP`、
  `LAST_HTTP_STATUS` 各一份，不复制第二份表。
- **源码指纹闭包**：`stdlib_assembly` 的 `loader::stdlib_assembly_fingerprint` 链式哈希与
  `manifest::provider_inputs(Vm)` 输入清单必须同时覆盖全部域文件（含 884 新增 5 个）。
  新增迁移源漏列=内容静默不入指纹；`provider_inputs(Vm)` 的域目录覆盖由常驻守卫测试
  `stdlib_assembly::manifest` 的 `plan884_provider_input_coverage` 钉住（文件集合 ⊆ 输入清单），
  漏列即带名红。指纹**值**变化是合法的（plan738 只钉确定性与敏感性）。

## 4. `call_inner` 形状族派发（`trans/rust/call_emit*`）

- `call_inner` 由**顺序 if-let 链**改为同序的**族派发**：22 个顶层阶段按调用形状拆入
  `trans/rust/call_emit/{call_bare_builtins,call_value_ctors,call_macro_shapes,call_task,
  call_tag_ctor,call_bina_methods,call_dot_methods,call_module_methods,call_generic_fallback}.rs`；
  `call_emit.rs` 只保留审计过的派发序、`call` 外层与 `emit_cabi_kit` 模板。
- **三态显式**：族 helper 返回 `CallShape::{Miss, Handled}`（`Miss`=形状不属本族、落穿下一族；
  `Handled`=已发射、立即返回）；发射/校验失败走 `Err`。末族 `emit_call_generic_fallback` 无条件
  终止（无总 miss）。**不得**用 `Ok(())` 同时表示"未命中"与"已发射"，也不得在已发射后继续调用
  通用 fallback。
- **首匹配序不可重排**：重叠载荷对（B2 先于 Value ctor、B6 先于 `.send`、env.get 字节差异、
  `str.from_uint`/`fs.metadata`、`set`/`contains`/`len` 四路族、B9 先于 B21、B4/B5 先于
  Value ctor、json len 的 Bina 无 cast vs Dot 有 cast、http post 四形）为前提；换序即行为漂移，
  a2r golden 与形状反例是唯一判据。
- **`call` 外层事务序保持**：临时 `Vec` 收集 → UTF-8 校验 → `serialize_positional_args`（二次
  全量实参发射，幂等副作用）→ `collect_core_reference`（发射完成后仍可 `Err` 的诚实拒绝门）→
  单次 `out.write_all`。部分写入/已产生副作用的位置不得改变。
- 审计确认不可达的三处块（Bina 方法翻译实效死、Dot 形 Pattern B 死、module.Type 构造死）
  逐字保留（884-N4）；若派发语义改变须重做不可达性审计，不按行数删除。

## 5. Rust 主管线主题模块（`trans/rust*`）

- `trans/rust.rs`（约 4293 行）只保留**编排**：`RustTrans` 类型与构造、`Trans::trans` 阶段序
  （header→prescan→类型推断→Phase2 拆分+全局注册→Phase3 decl 发射→Phase4 main 合成→prelude/
  导入注入）、单文件链、project/merged 驱动与查表。
- 主题模块（9）：`expr_emit`、`stmt_emit`、`decl_emit`、`ty`、`call_emit`、`project_fixups`
  （843 期）+ `post_fixes`（27 个后处理 fix 体 + derive mask）、`ast_scans`（mutates-self/await/
  Result/依赖 crate 扫描族 + 共用收集器）、`merged_fixes`（merged 后处理 + regex 包 + split_args）
  （884 期新增）。同一算法只有一份，原大入口不保留旧实现作永久 fallback。
- **收集器去重**：`collect_fn_spec_params`（逐字节同）与 `collect_fn_str_params`（仅注释差）统一为
  `ast_scans.rs` 单份，project 与 merged 两管线共用；`collect_fn_param_types` **保持双份**——
  merged 版额外收集 merge-mut 标志（载荷语义分叉，参数化合并待单独签核，见 884-N1）。
- 守恒：生成源码/导入/资源/引用证明、类型与借用上下文、`RustTrans` 标志写入时序、source map、
  失败阶段与诊断不变；不为缩总行数外移测试或拆分 HTTP 服务。

## 6. 规模事实（同口径物理行，2026-10-29 冻结）

| 位置 | 行数 | 角色 |
|---|---|---|
| `entry.rs` | 3205 | 公开 run 入口 + 非 run 职责（未强拆） |
| `entry/run_bridge.rs` | 82 | 运行桥接单源（G1/G2/G3 + G5） |
| `vm/ffi/stdlib.rs` | 2278 | 装配壳（ID 带/状态声明/组合/测试） |
| `vm/ffi/stdlib/*`（13 域） | 约 12.0k | 域算法/shim/状态 |
| `vm/native.rs` | 3687 | 注册表与 catalog/inventory 装配（未改） |
| `trans/rust.rs` | 4293 | 主管线编排 |
| `trans/rust/*`（8 主题，不含 call_emit 族） | 约 19.7k | 发射/扫描/后处理算法 |
| `trans/rust/call_emit.rs` | 210 | `call` 外层 + 22 族派发 + cabi kit |
| `trans/rust/call_emit/*`（9 文件） | 约 7.9k | 22 个形状族 helper |

## 7. 测试与验收锚（长期保留）

| 锚 | 类型 | 覆盖 |
|---|---|---|
| `crates/auto-lang/tests/plan884_language_structure_contract.rs`（11 例） | 独立 integration | V-02 入口结果/stdout/path 相对导入/bytecode+meta/input 三态；V-03 deadline 已到、staged 编译错误≠typed trap、运行期 trap 分类 |
| `stdlib_assembly::manifest::plan884_provider_input_coverage` | 单元守卫 | V-09 迁移源 ⊆ `provider_inputs(Vm)`（漏列带名红） |
| `plan872_builtin_semantics_tests` + `plan872_binding_table_json_matches_regeneration`（13 例） | 绑定/语义 | 名字/ID/生效实现/覆盖序与六族语义零漂移 |
| a2r golden（405 例）+ `plan716`/`plan608`/`plan746`/`plan755`/`plan781`/`plan738` | 金样/契约 | 生成字节、CALL_SPEC 路由、HTTP 字节契约、装配清单与指纹 |
| `test/fixtures/modularity/language-sdk-consumer`（5/5）、`three-repository-consumer`（3/3）、`verify-deps.sh` | 真实消费者 | SDK 独立消费、跨仓协议、无反依赖（auto-ui/iced） |

档位：结构改动触面=裸 `cargo t` + `cargo tv` + `cargo tt`；本 Plan 无 book/UI 生成/AAVM 写面，
故 `tb`/`tu`/`taa` 不触发。

## 8. 保留限制与在册债务

| ID | 级别 | 内容 |
|---|---|---|
| 884-N1 | MEDIUM | `collect_fn_param_types` project/merged 双份语义分叉（merged 多 merge-mut 收集）；合并需参数化并重做两侧 oracle |
| 884-N2 | LOW | `vm/native.rs:1700-1706` 注释描述 catalog 覆盖为前史；现行 catalog 与 inventory 对 872 族已 fn 身份收敛，序互换实测中性（非回归） |
| 884-N3 | LOW | `stdlib.rs` 壳仍驻留跨域 `mod tests`（约 600 行）与 musk extern 注册尾段（约 90 行）；有界决策保留 |
| 884-N4 | INFO | `call_emit/` 三处审计定死码逐字保留（不可达/尚无 oracle 块不静默丢失） |

另：`entry.rs` 本不在源码指纹闭包内（与基面一致，非本 Plan 引入）；HTTP 服务壳（`http_server*`）
经审计确认 `server_*` 单源已成立（无仓内调用方引用旧内联路由/派发），**只核销不迁文件**。

## 9. 写面与 ACE 边界

本 Plan 的结构写集=R 域（`entry`、`vm/ffi/stdlib*`、`trans/rust*`、`stdlib_assembly`）。ACE 相关
S/N 域（`experimental/ac-*`、native/MIR lowering）、H 域（`test_cases/**`、auto-test-case 协议、
`binding.tsv`、`ace_m1_tests.rs`）均只读：本收口不复制其 WIP 类型/lowering，也不改 ACE 期望值与
协议；`core/lib.rs` 与上层 `auto-lang/src/lib.rs` 仅保留再导出/注册的最小适配（本 Plan 实际零改动）。
跨域若需真实适配，先按 Plan 884 §5.1 的 X884 字段登记 owner/符号/消费者再改。

## 10. Provenance

- 归档 Plan：[PLAN-884](../../plans/archive/884-language-structure-closure-without-crate-split.md)
- 上游：PLAN-843（结构半部）、PLAN-869（热点/减法）、PLAN-872（内建双轨裁定）、PLAN-873（不拆 crate 结论）
- 基线审计：`docs/plans/attachments/884/t01-{baseline-freeze,audit-entry,audit-stdlib,audit-call,
  audit-rust-http-api}.md`（本地证据，命令/退出码/选择数摘要见归档 Plan §9）
