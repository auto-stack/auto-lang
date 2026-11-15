# ace-migration: m2-facade — 产品执行配置与嵌入前门

> Status: PLAN-969 的 **E0 早期切片**（T-E0-I/V/R/D，2026-11-15 phase fold
> @1ad2d2c60）已交付；本文件记录**已验现值接口**与边界。完整 M2-A 前门义务
> （run/run_file/args/capture/deadline/input/config 兼容面、T-01..T-08）**未交付**，
> 见"边界"与 [PLAN-969](../../plans/969-ace-m2-facade.md) §5/§8。
> 来源：E0接口冻结记录（§5）、T-E0-R 独立审计（PASS，findings 全数修复）、
> §9 收据；消费方：978/B0、980/HELLO。

## E0 已验接口（公开显式ACE同步执行前门）

公开合同与执行桥**分离**：合同在 `auto-lang-core`（零 ACE 依赖、零 legacy
引用——模块内无 `run_autovm`/`parse` 符号，ACE 失败永不切换默认引擎）；可执行
引擎由宿主经 `install_ace_executor` 注入，参考实现是新桥 crate
`auto-lang-ace-host`（root workspace 成员但**非** auto-lang/auto-lang-core 依赖，
daily 组不构建 ACE）。

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 引擎选择 | `AceEngineChoice{AceAV}`；`parse_ace_engine("av"\|"ace-av")`；未知名与**全部旧引擎名**→`Denied{ace.engine/unknown}`（零dispatch，不自动切旧机器） | `crates/auto-lang-core/src/ace_entry.rs` |
| 请求 | `AceRequest{source, entry_def_id, capabilities, grants, provider, wall, cancel}`（全显式输入，每run重核；`simple()`便捷形） | 同上 |
| 结果分态 | `AceOutcome{Success{value,steps}, Diagnostic{stage,code,message,span}, Denied, ResourceDenied, Cancelled, HostFailure}`＋`AceRunReport{engine, diag_count, callee_count, rejected_count, wall_ms}`；前端诊断**原stage/span透传**（实测钉死：bind/source.argument、text/source.syntax、text/source.unsupported） | 同上 |
| 注册/执行 | `install_ace_executor(Arc<dyn AceExecutor>)`（单owner、同指针幂等）/`shutdown_ace_executor()`/`run_ace`/`run_ace_named`；未注册→`HostFailure{ace.host/executor-unregistered}`；锁poison→`ace.host/executor-lock-poisoned`；同步执行、不spawn线程（UI主线程合同） | 同上 |
| 执行桥 | `auto_lang_ace_host::install()`（进程单例幂等）；provider生命周期 `build_f0_text_provider(s)`/`revoke`/`readmit`/`teardown_provider` | `crates/auto-lang-ace-host/src/lib.rs`（crate根`#![cfg(windows)]`，ace-abi模式） |
| 每run纪律 | 查注册→撤销态预拒（`ace.provider/revoked`，编译/装载前）→foreign车道caps⊆冻结F0集{foreign.scalar,foreign.resource}（域外=`ace.capability/unsupported`预拒）→grants对已授予caps核（`ace.grant/unadmitted`；纯源非空grants=`ace.grant/no-provider`）→每run重admit（`f0_authority`+`manual.admit`）→独立`load()`（本run独占`ForeignStubTable`/计数）→`ForeignSession` adopt→execute→teardown（关闭后晚到调用拒绝） | 同上 |
| 纯源车道 | `compile_scalar_or_core_source_product`→`lower_checked`（会话暖启同seam）→`verify_mir`→ABC encode/decode→`verify_abc`→`execute`；入口缺失=`entry`阶段诊断（`ace.entry/missing`） | 桥消费 `ace-av`/`ace-lowering`/`ace-av-core` 既有公开接口 |
| foreign车道 | `SourceQuerySession.set_foreign`→`compile_text_record_product`→`lower_checked_owned`（F0 owned/text-record族）→…→`verify_abc_foreign`→`ForeignSession`（967冻结消费面） | 同上 |
| 永久验证 | `crates/auto-lang-ace-host/tests/e0_facade.rs`（V-E0-DISPATCH/CLOSE/SOURCE经真公开门：text_byte_len("A中😀\0")=9且callee=1、坏签名零dispatch原span、撤权/未知/旧引擎具名拒、双身份20轮计数不串、teardown后HostFailure、cancel→Cancelled、B01=14）＋core侧`ace_entry::tests`4纪律用例（daily可见，纯std） | 固定组 `cargo t ace-facade`（domain ace，入ace集） |

## 边界（现值开放项）

- E0=**同步一次执行**＋宿主owner创建/关闭；跨请求暖会话、编辑、async/daemon/
  热更新未交付（原 T-02/03 与 970 后续）。
- 兼容前门（run_abt/create_vm_from_abt、run_with_mode/auto mode、capture/
  input/deadline/config 全量、stdin/EOF/cwd/args）**未交付**——归 T-08/AC-06；
  E0 的同步功能不冒完整入口兼容。
- 撤权与在途 run 的竞态：E0 同步单线程合同下无影响（零 thread::spawn），
  async 切片须重定义（T-E0-R F-E0R-05 边界）。
- 桥 crate 整体 Windows-only（ace-abi 模式：非 Windows 编译为空 crate）；
  E0 不承诺跨平台。
- `AceValue` 公开值为标量面（Unit/Bool/I8..F64/Char/Opaque具名）；记录/文本
  句柄不穿透中立合同（后续任务扩展）。
- 消费方接线：978/B0 与 980/HELLO 经 `auto_lang_core::run_ace`＋
  `auto_lang_ace_host::install()`；UI 不自建第二解释器/私有 FFI runtime。
