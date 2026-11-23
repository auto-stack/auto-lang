# ace-migration: m2-host — C/Rust FFI、自动binding与通用宿主资源

> Status: PLAN-967 全部 work 任务（T-F0、T-01..15）与两轮独立复审（final needs_fix →
> scoped **pass** @517920443，2026-11-15）已交付；本文件记录**已验现值接口**与边界。
> 来源：[PLAN-967](../../plans/967-ace-m2-host.md)（§5 冻结合同、§6 T-04 台账、§8/§9 收据）。

## F0 已验接口（AV 真调 Rust 的通用切片）

单一导入输入 = 带版本化注解的 Rust provider 源文件本身；任何 per-function 手写 manifest
均不存在。

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 导入器 | `auto_bindgen::rust_import::import/generate_shim/to_manual_manifest`（格式 `auto.bindgen.rust-provider/1`；注解 `auto-ffi provider/fn`；域外具名拒绝 `rust-import.*`） | `crates/auto-bindgen/src/rust_import.rs` |
| 冻结 provider | `f0_text`：`text_byte_len`/`text_is_ascii`/`text_first_byte`(fallible `empty-input:-1`)/`text_panic_probe` | `experimental/ace/crates/abi/rustside/f0_text.rs` |
| 构建单元 | provider+生成 shim 平铺一个 rustc `cdylib` 闭包（记录 rustc 身份）；`LoadLibraryW/GetProcAddress` 装载并解析导出 | `experimental/ace/crates/abi/src/rust_provider.rs` |
| 物理帧（版本化） | `extern "C" (data: *const u8, byte_len: u64) -> i32|u8`；fallible `+ out: *mut i32 -> i32`，`None`→最小命名负 status；bool=0/1 | 生成 shim（rust_import） |
| panic 通道 | shim 单元内 `catch_unwind` + 导出 `f0_panic_flag()`；宿主每调用后观察并清零→独立 host 失败（`foreign.panic`）；**任何 unwind 都不跨 C 边界**（跨 DLL unwind 宿主不可 catch，实测冻结） | shim 生成 + `av-core/foreign_exec.rs` |
| AV stub 表 | `ForeignStubTable`：拒绝序=身份(unknown-symbol/signature，含 form/ABI 形匹配)→generation→UTF-8→call-bytes(65536)→run-bytes(1048576，每 run 计费输入字节)→reentry，全部在 callee 前；`callee_count` 只计真实调用；`bump_generation` 全表失效 | `experimental/ace/crates/av-core/src/foreign_exec.rs` |
| AV 执行 | `av::execute_foreign`（每 run `authorize_module`+run_bytes 复位）；`abc_verify::verify_abc_foreign`（无表默认拒绝，938 语义不变）；fallible status≠0 → `Trap{ForeignStatus}`（命名表在 manifest error model） | `av-core/src/{av,abc_verify,contract}.rs` |
| 源前门 | `use.rs <provider>`（CST `use-rs-item`，仅 text-record profile；`source.use-rs.syntax/domain/duplicate/provider`；legacy oracle 一致） | `grammars/auto-source.at` + `core/src/source/{cst/project,parser,resolve}.rs` |
| 永久验证 | `ace-abi/tests/cases/rust_f0.rs`（5 E2E：真 cdylib、真 AV 调用、oracle、拒绝、预算、身份/重绑定）+ av-core foreign_exec 单测 + auto-bindgen 单测 | 冻结组 `cargo t` 受影响并集 |

边界（现值开放项，owner 见 KNOWN-DEBT-AND-RISKS D-967-T05）：GrantSet **owner/grant 授权轴**
（F0 已交 identity+generation+form 维；全维账本待后续）；owned String/Vec/Result lane 与
async importer 面（域内为**具名拒绝**，非已交付能力）；F4 日常道 lane-split（f0 编译即被测
行为 vs 显式构建道的固定组结构，治理裁定开放）；运行依赖准入（OS/CRT/ISA 运行轴）归 978
（B-06 正名交接）；1043 `#in` 平台属性在 FFI 腿的选择未行使（已交付后缀装配 Rs 消费）；
ProgramSnapshot/session 链接面（969/E0）。938/939 的 host 注入裸名调用道保持（F0 前门为
"存在即校验"的可用性前门，非强制门）。

## T-08：异步完成端口（中立合同 + 宿主注册表 + 受控真线程 provider）

合同 `foreign-contract/port.rs`（`autoabi.contract.completion-port/1`）：PortValue/PortEvent +
唯一 `completion_disposition` 规则表（publish/drop-late/unknown；cancel 墓碑把迟到完成归为
DropLate 而非 UnknownPort）。宿主 `av-core CompletionPorts`：register/complete/cancel/teardown
绑会话 generation——迟到完成可观察丢弃、绝不发布、绝不触已闭会话；每次状态变更是因果
PortEvent（968 await 消费此，967 不反依 968）。端口绑 generation：teardown/rebind 杀 in-flight
端口（实测）。受控 provider `rustside/f0_async.rs`（真 cdylib、真 OS 线程、真 C-ABI 完成回调）；
`f0_async` 2 E2E（发布/cancel 迟到丢/teardown 迟到丢）。C 侧行（export/callback pin/过期
generation）=939 永久 fixtures（ce-*/callback-*/handle-* decks）。

## T-09/T-10：工程入口与真实预处理 C importer

- **工程入口**：`open_project_session(project_root, provider, generation)`——`use.rs <provider>`
  解析到工程文件 `<root>/<provider>.rs`；binding/shim/构建单元/manifest 全部 open 时从该文件
  派生（工程内不存在 manifest 文件 = V-09"删手写manifest"构造性满足）；具名拒绝
  `rust-import.provider-name/provider-missing/provider-read` + importer 域拒绝。永久正反例
  `f0_project_import_runs_rejects_and_recovers`。
- **C importer**（`auto_bindgen::c_import` + `abi/ctool`）：注解头 = 元数据
  （`// auto-ffi-c provider <name>/<ver> target=...`；`fn` 条目 `form=pure ret=|fallible errors=`）；
  **真 cl /EP 预处理**后解析声明（record/enum/callback/view/out/opaque/static-inline/常量；
  varargs/union/bitfield/packing 逐行具名拒绝）；空标量 spine 遵 939。`open_c_provider`：
  工程 .h+.c → 派生 manifest + cl 编译 provider obj（include 闭包真观察）+ 伴生 .h/.c 暂存；
  `build_c_dll`/`build_c_dll_with`（cl /LD；额外 TU+SDK include 闭包=真实库腿）。target 轴
  冻结校验（`c-import.target`）。冻结 fixtures `cside/f0_c.{h,c}`；`c_import` 4 案。
  F-967-COFF-CB：callback 资源 manifest 在整模 split 道具名 fail-closed（`foreign.callback-lane`
  →939 逐函数 session 道）。

## T-11：Rust importer 扩面（混合帧 + 形状审计 + 双 rustc）

参数域扩至 view+i32/bool 混合（≤4）；T-11 形状审计对 泛型/owned String/Vec/Result/async/
trait对象/&mut 逐项**具名拒绝**（边界行闭合，非静默缩窄）。混合帧 AV 臂 `dispatch_view_scalars`
+ `ViewI32*` callables。双 rustc C-ABI 交叉（stable+nightly，`locate_second_toolchain` 真发现
rustup toolchains 目录）：身份异、接口 facts 同、AV 宿主跑另一工具链 provider；staticlib 同
闭包产物（B-02）。冻结 fixture `rustside/f0_rich.rs`；`f0_rich` 3 案。

## T-12：四端/六路径消费（同一 binding/shim/provider）

| 路径 | 交付 | 永久验证 |
|---|---|---|
| AV/Rust | F0/项目/装配入口 → AV stub 表真调 | rust_f0/f0_int1026/f0_load1043 |
| AV/native | native lowering→caller COFF→rust-lld+import lib→真机器码 | `f0_native_machine_code_consumes_the_same_auto_binding`（exit 9/重绑定 10） |
| AV/C | `build_c_dll` 真 cl DLL 驱动 AV 道 | `f0_av_c` |
| AC | 938 manual + T-03 生产 TestCase 道（`abi/rust-f0-certify-v1` worker 配方，av+native legs） | 固定组 `case-ace-m2-f0-rust-967`（6 案） |
| A2R | `a2x-rust foreign_bridge`（生成 Rust 经同一 shim 导出真编译真运行；域外 `a2r.foreign-unsupported` 保持） | `f0_axr` 2 案 |
| A2PY | `a2x-python foreign_bridge`（ctypes CDLL 装载同一 cdylib、typed `A2xForeignStatus` 异常通道） | `f0_a2py` 2 案 |

V-COVERAGE 六路径全实跑（T-01/T-12 收据）。1043 来源后缀装配消费：见下 AC-LOAD 节。

## T-13：端口×generation 与资源纪律

future 端口绑会话 generation（teardown/rebind 杀 in-flight、迟到丢弃实测）；重入/destroy 一次/
stale/cancel-late = 939 callback/handle/txn decks（native+c 腿）+ `f0_t13` 3 案（映射+fail-closed+
CE export decks 身份绑定）。v1/v2 双 provider 并存：见"ABI 兼容轴"节。

## T-14：精确构建缓存与分发清单

`ProviderBuildCache`（S967B 层，进现有 783 ManagedStore）：BuildIdentity 全轴真读
（provider 字节/shim 字节/rustc `--version`/生成器身份/target/crate-types）。精确失效实测：
provider 编辑翻身份、shim 轴独立翻、工具链轴独立翻、AUTO body 编辑不触键、删工件=miss
（不 stale-green）、双 rustc 两身份一合同。分发清单 BuildRecord JSON（cdylib+staticlib+import
lib+全轴+runtime note）交 978。运行兼容轴与构建缓存**分账**（consumer rustc 变化不强制重建
不变 provider）。`f0_t14` 2 案。

## T-15：钉版本真实库（serde_json + SQLite）

- **Rust**：`// auto-ffi dep <name> <spec>` 注解 = 干净工程唯一依赖配置；
  `build_cargo_provider`（真 cargo 单元 `cargo build --offline` dev profile，cdylib+staticlib
  同闭包；Cargo.lock 实证钉版；解析失败具名 `f0.cargo-build` 带 cargo 自身诊断）。
  fixture `rustside/f0_json.rs`（`=1.0.151`）。
- **C**：SQLite **内容钉版**（`locate_pinned_sqlite` 真读 sqlite3.h `SQLITE_VERSION`；错版
  具名 `f0.sqlite-pin`）；amalgamation 编译一次共享对象（`compile_c_unit`），provider DLL 与
  独立 oracle exe 链接同一 obj。fixtures `cside/p967_sqlite.{h,c}`（open/exec/step/close 线，
  连接为 provider 私有）。
- 验证 `f0_t15` 3 案：干净工程→AV 真调（oracle=进程内 serde_json / 独立 cl oracle exe）；
  磁盘 .db 持久化读写；负控矩阵=错版本/缺库(C1083 具名)/坏 SQL/未开连接/双 close/内嵌 NUL
  路径（`nul-path` 真库边界）/超长（有界回读）/取消(Cancelled+恢复)/资源退出（卸载恰一次+
  重载）/去缓存重建/dist 隔离消费（V-ABI-PACK）/A2PY ctypes 腿。

## IF-CALL：p967.frame/1 受检 tag 参数帧（AC-INT-1026）

Python 适配器（1026）消费的通用调用帧——receiver+positionals+kwargs 走**既有 ViewText 物理道**
（ptr+len），AV 准入级校验（坏帧具名 `foreign.frame` 拒绝且 dispatch=0），生成 shim 内置
类型+游标解码（防御二次门）。**冻结线规**（AV 校验器与 shim 解码器双实现，顺序依赖：
整帧 `foreign.utf8` 门先于帧校验）：

```
p967.frame/1 LF
'r' SP tag SP len LF + <len> payload LF        (receiver)
'p' SP count LF                                  (count 0..=8)
[tag SP len LF + payload LF] x count
'k' SP count LF                                  (count 0..=8)
[namelen LF + name bytes + tag SP len LF + payload LF] x count   (namelen 1..=64)
无尾随字节（精确耗尽）
tags: n(len 0) | i(i32 十进制) | b(0|1) | t(任意字节 ≤4096) | o(u64 十进制)
```

导入域：fn 注解 `frame=p967.frame/1` + 恰一参数 `frame: &F0Frame`（轴/类型/版本不匹配=
`rust-import.frame` 具名拒绝）。shim 生成 `F0Val/F0Kwarg/F0Frame` pub 类型（同 crate 平铺，
provider 零依赖）。opaque `o` 引用 = provider 私有 id 空间（INT-1026 资源模型：单一账本归
provider；跨 provider 递交=具名 bad-handle 拒绝）。独立通用 provider `rustside/f0_tframe.rs`
（标量/text 含 NUL/opaque 注册表/kwargs/panic 探针，绝非 Torch）；`f0_int1026` 2 案
（8 类坏帧 dispatch=0、过期代次拒 admission、panic=host 失败）。

**版本化超集 `p967.frame/2`（PLAN-1129）**。布局不变，在 `/1` 的五种标签之外新增带类型的
浮点值：`d`（f64）与 `f`（f32），payload 为 `<w2> <w1> <w0>`——三个无符号十进制字，把 64 位
位型按 21/21/22 位自高位拆开，每字有界，故一个值只有一种编码。`f` 的 payload 另要求高 32 位
为零（`w2 == 0` 且 `w1 < 1024`），超出即具名拒绝而不是截断成 f32。畸形 payload 是具名
`foreign.frame` 拒绝，消息带帧版本（`p967.frame/<版本> rejected: <原因>`），派发前生效。
`/1` 帧永不携带 `d`/`f`。fn 注解可写
`frame=p967.frame/2`，该声明同时接受两个线路版本（旧模式只由帧头选择）。生成的 shim 类型随之
增加 `F0Val::F64(u64)`、`F0Val::F32(u32)` 与 `F0Frame::version`（1 或 2，解码器按帧头填写）。
注解是 provider manifest 内容，因而帧版本进入接口身份：`RuntimeCompatibility` 的逐符号签名
哈希对带帧声明显式纳入版本，`/2` 需求遇 `/1` 供给是具名 `m2build.compat.symbol-signature`。
AV 校验器与 shim 解码器是同一字节集的双实现，仍须同步修改（`frame_wire_1129_tests`、
`f0_int1026`、`py_float_1129`）。shim 解码器的源文本单独放在
`crates/auto-bindgen/src/frame_shim_prelude.rs`：
生成器 `include_str!` 原样拼入 provider，单元测试 `include!` 编译同一份文本，解码规则因此在
不用生成 provider 的情况下也可直接受测。未声明帧的符号签名哈希不变。

## AC-LOAD：1043 来源装配的 Rs 消费（V-LOAD）

`open_assembled_rs_provider_session(root, entry, ctx, generation)`：原样消费 1043 交付 API
（`discover_provider_sources` → `assemble_package_inputs` → **Rs 实现层**源 → 同一
importer/shim/build 链；无 av.rs loader）。公共 `.at` = Auto 适配器（各后端同一份），`.rs.at`
= Rust provider 实现；FFI 腿点名 `ProviderChoice::Kind(Rs)`（Auto 默认 at 便携体，无则具名
拒——1043 语义原样）。装配自身具名错误以 `load.*` 前缀传播；选中输入编辑→provider digest
翻→同入口重建；未选兄弟（如 `.py.at`）→digest 不变。`f0_load1043` 2 案。

## ABI 兼容轴（Windows 交付面；AC-ABI-COMPAT）

- **工件×manifest 交叉**：cdylib PE machine 字段（e_lfanew→PE 签→machine=0x8664）对照
  manifest target——真读工件头，非信任标签；staticlib 为 `!<arch>` 归档形态（原生重链接
  分发形，B-02）。错 target 注解=投影冻结拒绝。
- **接口轴**：ghost 符号/form 翻转=importer 具名拒；**可选增量**（v2=v1+新 fn，required 超集）
  旧 caller 经同一 AV 管线重准入；required 破坏→前门 `source.undefined`。
- **双代并存**：v1/v2（body 编辑）双 DLL 同时装载为独立单元；v1 opaque 交 v2=provider 注册表
  具名 bad-handle（跨代不接管）；v1 退出 v2 存活；重载可用。
- 跨 rustc 交叉与缓存身份见 T-11/T-14 节。运行依赖准入（CRT/OS/ISA）归 978（B-06）。
  `f0_abi_axes` 3 案。

## T-02：既有应用桥逐项映射与旧数字 ID 兼容裁定（2026-11-14）

| 既有桥 | 旧实现（producer→consumer） | 新 ACE 归属 |
|---|---|---|
| C 桥 | `use.c`/`use c` 源语法（`use_scanner.rs`/`parser/statements.rs` UseKind::C）→ `auto-bindgen` 预置 5 头 manifest（extractor）→ `auto-lang-core/vm/ffi/c_ffi.rs` libloading → 旧 AutoVM | 公共工程入口与真实预处理 importer：T-09/T-10；`ace.m2.p967.source01`（strlen）按原支持域映射 |
| Rust 桥 | `use.rs`（UseKind::Rust）→ `trans/rust`（a2r 源到源）+ `project_fixups.rs` manifest 静态/动态 wrapper → rustc | F0 auto importer（本文件上表）已交借用 UTF-8 域；完整公开元数据/泛型/owned：T-11；A2R 同源腿：T-12 AXR0 |
| Python 桥 | `use.py`（UseKind::Python）→ `trans::python`/`py_known` | A2PY 目标（1042 已交 readiness）；INT-1026 通用 tag 帧消费：T-09/11/12/13/14 分维度，T-12 接 AV |
| 旧 native 数字 ID | `auto-lang-core/vm/host_bridge.rs` 数字宿主 ID 直通 | **只作兼容映射**：新权威是 provider/symbol/content_version/form+generation 身份（F0 已交 identity+generation 维；owner/grant 维 T-13）。任何新代码不得以数字 ID 授权 |

会话组合层（T-02 交付）：`ForeignSession`（av-core `foreign_exec`，open/bind/execute/teardown/
rebind/adopt）+ `F0Session`/`open_f0_session`（ace-abi `rust_provider`，build→admit→load→bind
一体保活，Drop 恰好释放库一次）。永久生命周期正反例：`f0_session_lifecycle_success_teardown_
unauthorized_rebind`（V-02 合法身份真成功；V-01 teardown 后 admission 与 execution 双拒且
dispatch 计数冻结；V-06 空绑定 unauthorized 且 callee_count=0；provider 重绑定同入口恢复）。
V-03 cancel/late 与回调形态 V-04 全量仍在 939 native 道（callback/txn fixtures）直至 T-08
异步完成端口；V-05 重入 = foreign_exec 单测 + console ProviderScope 同律。

## T-FRAME：受检可写输出借用帧（PLAN-1064 SD-FRAME，2026-11-17）

- **合同与自动绑定（无手工 manifest）**：`ace-foreign-contract` 新增 `ResKind::OutBytes`；`crates/auto-bindgen/src/{rust,c}_import.rs` 从 Rust `&mut [u8]` 与 C `(uint8_t* dst, uint64_t cap)` 自动导出输出帧声明（保留 `&mut i32` 等非字节切片的 `rust-import` 具名拒绝，不放宽任意 `&mut`）。
- **物理执行与守卫（`av-core::foreign_exec`）**：新增 `OutBufferFrame` 与 `dispatch_view_out_bytes`，在调用前校验 `owner`/`origin`/session `generation`/可写权限/`0 <= off <= cap` 与 `run_bytes` 预算，分配带前导/尾随 guard bytes 的隔离暂存区并在返回后校验 `0 <= n <= cap` 与 guard bytes 完整性（错写即主机失败、`0` 字节泄露，过期/取消/跨代帧调用前拒、`callee_count = 0`）。
- **永久证据与 B 分账**：`ace-abi::out_frame_1064`（3 E2E：`v_frame_rust_provider_read_into_and_guards` / `v_frame_c_provider_read_into` / `v_frame_security_lifecycle_and_import_negatives`）+ `auto-bindgen` 10/10；`AC` native 机器代码侧 `OutBytes` 调用与帧守卫校验已依 `SD-AC-STAGED` 移交 `PLAN-1068 T-03`。

## 普通 AV Rust 工程（IF-ACE-AV-RS-PROJECT/1）

Windows/MSVC 的 `ac-build` 从工程声明读取 Rust provider 源与可选锁文件。普通 Auto
的 `use.rs` 引用该源声明的 provider 身份。967 生成的 C-compatible shim、受检声明和
`F0Session` 由 1093 普通编译入口消费，产出带 owner 的 ABC。执行时先按当前会话代次
重新准入 ABC，再逐调用声明检查当前授权。SDK 每次打开新的会话，provider DLL 从 SDK 与
系统目录解析运行依赖。

已验证的工程调用包含借用 UTF-8、空串、provider 换版以及带锁的 Rust 依赖。在同一已打开
会话重放工程 ABC，撤权、预取消和关闭后旧代次均为零新增派发。shim 隔离 Rust panic，
同一会话随后可以执行合法调用。

Rust 注解的 `owned_open`、`owned_borrow`、`owned_close` 共享 `owned=Type`，分别核
`Box<Type>` 返回、`&Type` 只读借用和消费 `Box<Type>`。编译到中立资源声明，Auto 按
939 既有整型句柄表面传值。会话把 guest token 绑定到真实 provider、资源类型和代次，并
对原 DLL 持有代码租约。显式关闭在调用原方前撤销条目，错误或会话关闭由原方处理剩余条目。
工程和搬移后的 SDK 均验证恰一次析构，双关闭具名拒绝。

Rust 的 `form=pure ret=i32` 可以声明 `(&F0Callback, i32, i32) -> i32`。Auto 参数
使用同签名的函数值，可捕获 Auto 局部变量。Rust 通过 `cb.call(ctx, item)` 在原 AV 执行
链内同步调用。回调借用限于本次调用和原线程，共用原执行的指令、栈帧和墙钟预算。回调
期间禁止再次进入 host、foreign 或调度器，闭包的错误保留原 AV 结果。

`form=complete_sync` 和 `form=complete_async` 接收一个 `i32`，完成值也是 `i32`。
同步 provider 在调用线程执行，异步 provider 在持有 DLL 租约的后台线程执行，调用方在该
指令等候端口的值。取消或墙钟耗尽时，调用方先离开，后台线程保持租约直至返回。每次调用
的状态独立传回，后台 panic 不会污染前台调用。端口绑定到 `ForeignSession` 的代次并
至多发布一次。取消或关闭只产生一次 release，后续到达记录为 late drop。

工程和 SDK 共用 Windows x64 MSVC C ABI。导入清单为 `auto.bindgen.rust-provider/1`，
普通闭包产物使用 `auto.abc.av-closures/1`。SDK 使用 `ace-m2-build/sdk-recipe/1`，
绑定描述为 `ace-m2-build/av-rust-program/1`。输入与交付可从工程标识、Rust 源和 shim
摘要、锁文件摘要、目标、ABC 与 DLL 摘要及运行授权核对。包内 `targets/av/program.abc`、
`targets/av/rust-program.json` 与 `providers/rust/provider.dll` 是 AV Rust 运行闭包的
主要工件。后续工程通过同一个 `ac-build build|sdk|run` 入口消费接口。

## A2R 工程运行时面（PLAN-1116）

- **调度/异步**：任务/actor 沿用 968 共享脱糖与 sched_rt.txt 运行时；异步块与
  `~T` future 函数为惰性 future（首用物化，正数=ask 活 port），`await` 按符号
  分派。生成器状态机走语言通道；显式 next/close 的所有权合同=非 Copy owned
  Assign/消费杀源 + 状态字段写回（AV 句柄别名 vs A2R 逐读克隆的桥）。
- **外库**：worker `run-language` 的 `foreign` 段（声明形，同 AV）经全身份轴复验
  （descriptor/digest 重导/stamp/DLL sha）入 `compile_language_source_with_foreign`
  公共前门；降低时校验 967 冻结 bridge 域；链接=cdylib import lib，DLL 为进程依赖；
  `AutoTrap::Foreign(i32)` 仅语言通道 trap 集。详见
  [project-runtime-and-distribution](../auto-a2r/project-runtime-and-distribution.md)。
