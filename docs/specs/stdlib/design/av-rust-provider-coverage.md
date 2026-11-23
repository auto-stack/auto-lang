# AV/Rust 标准库来源覆盖矩阵（av-rust-provider-coverage）

> Status: PLAN-1114 交付（批 1–3、B-01..B-11）；**PLAN-1123 增量**（2026-11-23）：批 4 安全传输层、`dep` 注解内联特性键、预备闭包消费（见对应行与 B-04/B-12..B-14）。
> 记录 AV/RS 标准库（`IF-ACE-AV-RS-STDLIB/1`）的逐操作来源、平台与实际支持矩阵：
> 每行 = 一个公共操作 → 实现来源分类、owner、平台与验证入口。分类法沿
> [Design 60](../../../design/60-ace-rust-route.md) 与
> [backend-provider-coverage](backend-provider-coverage.md)。

## IF-ACE-AV-RS-STDLIB/1（接口冻结，1115 A2R 消费点）

| 面 | 冻结内容 |
|---|---|
| 源树 | `stdlib/ace/avrs/<module>.at`（契约面：`pub spec fn` 公共 API 记录）+ `<module>.rs.at`（`// auto-ffi` 注解 Rust 片段；`dep` 注解=显式外库钉版）|
| 选源 | `PackageAssemblyContext(Av, Kind(Rs), Windows)` 经 1043 `discover_provider_sources`/`assemble_package_inputs`（语义原样）|
| 组合/构建/会话 | `compose_avrs_provider` / `build_avrs_provider` / `open_assembled_rs_stdlib_session`（ace-abi）：全部选中 Rs 层 → 单一 `avrs/1` 单元 → 同一 importer/shim/build 链 → 一个 `F0Session`；构建根=组合源摘要键控 |
| 应用面 | 普通单模块源 `use.rs avrs` + provider 公共名（1093/967 前门）|
| 帧域 | view 首参 + ≤2 尾随 i32（`ViewI32ToI32` 本 Plan 补）/ 单 i32 完成口（M2Build 执行器）/ owned 三形 / OutBytes 帧 / 回调；边界 B-01..B-11 |
| 分发 | `M2Build` `ProjectInputs::rust_provider`（组合源）→ ABC+DLL+manifest → SDK 重定位运行（1113 链）|
| 旧资产 | `stdlib/auto/*.rs.at` 镜像原样保留至 1115/1117 切换；冻结面零改动 |

## 架构裁定（T-01 冻结）

- **D-1114-01 消费模型**：AV/RS 标准库沿 967 AC-LOAD 交付模型——公共 `.at`
  = 契约面（`pub spec fn` 公共 API 记录，同一 API 各来源同签名）；`.rs.at` =
  Rust provider 实现（`// auto-ffi` 注解 Rust 源，片段形）。选源 =
  `PackageAssemblyContext(Av, Kind(Rs), Windows)` 经 `discover_provider_sources`
  /`assemble_package_inputs`（1043 语义原样，无新 loader/binder）。会话 =
  `open_assembled_rs_stdlib_session`（ace-abi）：闭包内**全部选中 Rs 层**按模块
  序拼接为单一 `avrs/1` provider 单元（`AVRS_PROVIDER_HEADER`），经同一
  importer/shim/build 链构建，开为一个 `F0Session`；跨模块重复导出名 =
  具名 `avrs.duplicate-fn` 拒。
- **D-1114-02 应用消费面**：应用为普通单模块源 `use.rs avrs` + 调 provider 公共
  名（1093/967 已交付前门；包绑定面明文拒 `use.rs`——`source.use-rs.surface`
  冻结诊断不动）。公共契约一致性由测试锚定：provider 每个导出必须在模块 `.at`
  契约面有同签名 `pub spec fn` 声明（`v01_avrs_public_contract_faces_match_provider_exports`）。
- **D-1114-03 源码过渡**：旧 `stdlib/auto/*.rs.at`（`#[rs]` A2R 镜像）**原样保留**
  服务旧 A2R 轨到 1115/1117 切换（保旧文件发现与语义兼容）；Rust 体按本矩阵
  逐操作迁移进 `stdlib/ace/avrs/*.rs.at`（AV 车道 i32 int 域收窄记录在案）。
  冻结面（`stdlib/ace/{std,asm,sys}`、`std` 保留包名）零改动。
- **D-1114-04 帧域边界**：受检帧运行时域 = view 首参 + ≤2 个尾随 i32 标量、
  或单 i32 完成口（`complete_sync`）、owned 三形、回调/出参帧。本 Plan 补
  `(view, i32) -> i32` 车道（`ViewI32ToI32`，T-11 家族 i32 单标量孪生；此前
  该形被误转到 view-only 帧丢弃标量道——已修）。双 i32 纯形（`min/max` 等）
  无运行车道：共享 Auto 单源承载（PLAN-972 适用性说明），2-标量车道扩展为
  具名边界行（见下表）。

## 逐操作矩阵

分类：`R` = Rust provider 真身（经受检 shim 真调用）｜`S` = 共享 Auto 单源
（最终原语来源记录）｜`X` = 显式外部/领域提供者｜`N` = 具名非支持面（本版）。
平台：W = Windows/MSVC 实跑；L = Linux 腿按 1063 延期（代码臂 NOT_RUN）。

### 批 1 · core/text/math/collections（T-02，已交付）

| 操作 | 来源 | 平台 | Rust 体来源/迁移 | 验证入口 |
|---|---|---|---|---|
| str.str_byte_len | R | W | 迁 `stdlib/auto/str.rs.at` `#[rs]`（PLAN-781 nat#1526 镜像）| `v01_avrs_core_values_via_rust_provider`（`A中😀\0`=9）|
| str.char_at | R | W | 迁同上 `#[rs]`（`(view,i32)->i32` 车道首批消费者）| 同上（'中'=20013）|
| math.abs | R | W | 迁 `math.rs.at` `#[rs]`，i64→i32（AV checked int 域；i64 拼写留旧 VM 合同）| 同上（abs(-7)=7，complete_sync 车道）|
| math.min/max | S | W | 共享 Auto（asm 972 单源；最终原语=纯 Auto 比较无外部分派）| 972 `v08_surface_972` 复用 + 本表适用性说明 |
| math double 族（sqrt/abs_f/powf/sin/cos）| S | W | 共享 Auto（972 裁定：不复制三份算法、无 libm 委托）| 同上 |
| collections（list 11 ops/char 7/conv 4/hash 1/bytes 6/map 6）| S | W | 共享 Auto（972 asm 单源；List 值不过受检帧——帧域无集合车道；最终原语=mn arena/底座）| 972/v08 复用 + 适用性说明 |

### 批 2 · I/O/fs/env/time/net（T-03，已交付）

| 操作 | 来源 | 平台 | Rust 体来源/迁移 | 验证入口 |
|---|---|---|---|---|
| console.console_write | R | W | std::io stdout；返回 UTF-8 字节数（计数 oracle：9 vs C-strlen 截断 1/ASCII 4 可区分）| `v02_avrs_console_exact_bytes` |
| file.file_write_byte / file_append_byte | R | W | std::fs 字节粒度写（帧域单 view：批量双串写=B-09）| `v02_avrs_file_exact_bytes_eof_and_errors` |
| file.file_read_len / file_read_byte | R | W | std::fs 读；EOF=-1（idx≥len）、缺路径=-8、NUL=内容（byte 8 读回 0）| 同上 |
| file.file_exists / file_remove | R | W | std::path/std::fs；remove 后 exists 翻转真观察 | 同上 |
| env.env_get_len / env_has / env_remove | R | W | std::env；未设哨兵 -203（1064 D-05/D-06 同拼写）；env_set=双串（B-09）| `v02_avrs_env_time_and_loopback` |
| time.clock_open/ms_lo/ms_hi/close | R | W | owned provider 资源（SystemTime lo/hi i32 对；句柄=会话代次绑定的 guest token）| `v02_avrs_owned_handle_generation_and_close_safety` + `v02_avrs_closed_handle_rejected` |
| time.sleep_ms / now_ms / now_sec | N→B-10 | W | 完成口形属 M2Build 工程执行器（1113 已证；avrs 会话车道不收调度面模块）| sdk_pack 复用 |
| net.net_loopback_echo | R | W | std::net 真回环（127.0.0.1:0 监听+连接+精确字节写+EOF 读+比对）；-1=内容不符/-8=传输失败 | `v02_avrs_env_time_and_loopback` |
| 持久 socket 面（platform.net/1 句柄族）| S | W | sys 车道 native owner（973/1051）；Rs 车道交回环证明 | 973 复用 |
| 旧资产 `stdlib/auto/platform/io.rs.at`（1041 存在性占位 `io_impl_layer_present`）| 保留原位 | W | 注释声明"967 交付后 AV/Rust in-process foreign dispatch 消费此 origin"——该消费由本 Plan 的 avrs 车道实际承接（console/net 行）；占位文件本身属旧 A2R 降低路径，随 1115/1117 切换处置 | 本表 console/net 行 + 1041/1043 复用 |

### 批 3 · 数据与显式外部模块（T-04，已交付）

| 操作 | 来源 | 平台 | Rust 体来源/迁移 | 验证入口 |
|---|---|---|---|---|
| json.json_is_valid / json_sum_ints | R（显式外库）| W | 钉版 serde_json =1.0.151（`dep` 注解=MIX-07 干净工程规则；坏输入=具名 -1 解析失败值）| `v03_avrs_json_external_library_and_rand` |
| json 共享算法（escape/parse/path 族）| S | W | 975 std/json.at 纯 Auto 单源（Rs 车道不复制算法）| 975 复用 |
| rand.rand_next | R | W | splitmix64 非凭据 PRNG 步（Plan 250 裁定：凭据熵不入此面；OS 熵=973 bcrypt native）| 同上（确定性/可区分性断言）|
| sqlite | X | W | 967 钉版 3.46.0 C provider（975 已交 lane；use.c 面）| db_975 复用 |
| redis | S + B-11 | W | 975 RESP2 共享 Auto over platform.net；真服务器验证=外部服务依赖（§10 具名 pending）| 975 复用 |
| storage | S | W | ACE-KV 共享 Auto（1064）；Rs 无资源面义务 | 1064 复用 |
| image / frame | B-03 | — | 领域 owner 980/OS064 | — |

### 批 4 · 安全传输（PLAN-1123，已交付）

| 操作 | 来源 | 平台 | Rust 体来源/迁移 | 验证入口 |
|---|---|---|---|---|
| tls.tls_exchange / tls_result / tls_read_byte / tls_close | R（显式外库）| W（Linux 臂同源，实跑归 1063）| 钉版 `rustls =0.23.46`（`default-features=false features=ring,std,tls12`——默认加密后端 aws-lc-rs 不在离线 registry）+ `rustls-native-certs =0.8.4`；字节传输（连接/CONNECT 隧道/受检握手/写/读到流尾/关闭），HTTP 构造与解析留调用方 Auto（`tests/fixtures/tls1123/https_app.at`）| `ace-abi::conformance avrs_tls_1123::*`（8 例）|
| 显式选择 | — | W | `tls` 为独立选层：整单元，或 `str`+`tls` 选择（`AVRS_TLS_SELECTION`）；plain HTTP 的 `net` 层零依赖，不要求该层 | `compose_delivered_avrs`；`v03_provider_missing_and_prepared_closure_is_source_bound` |
| 依赖闭包构建 | — | W | 显式 `prepare-avrs-tls` 步骤（测试治理工具 `avrs-tls-provider`，`warm` 预热并经 `dir_env` 转发 `ACE_AVRS_TLS_PROVIDER_ROOT`）；生成的 cargo 单元带空 `[workspace]`（闭包位于工作区树下时可独立构建）并在 dev 档内对依赖闭包取 `opt-level = "s"`（单元本体仍不优化）| `cargo t ace-abi` / `ace-m2-build` 的准备阶段收据 |
| 预备闭包消费 | — | W | `ace-abi/provider-closure/1`；会话道 `open_prepared_avrs_session`；工程道 `RustProjectInputs.prepared`（源/shim/工具链/声明锁严格一致，否则 `m2build.rust-prepared`）| `prepared.*` 三反例；`p1123_prepared_closure_must_match_the_project_exactly` |

`// auto-ffi dep` 注解自此接受可选内联键：`<name> <version> default-features=<bool> features=<a,b>`；两词形的生成字节不变，畸形键=具名 `f0.cargo-dep` 拒绝（`provider_dep_specs`）。

### 负例与默认近邻（V-03 附加面）

| 面 | 验证 |
|---|---|
| 缺点名 Rust 来源（删 json.rs.at）| `v03_avrs_named_rejections`（a）：`source.undefined` 具名拒、点名缺失 op |
| 错签名注解（ret=bool 对 i32 体）| 同上（b）：importer 具名拒 |
| 撤权（去 foreign 授予）| `v03_avrs_revoked_grants_rejected`：具名能力拒 |
| 默认 Auto 近邻（移去全部可选工件）| `v03_avrs_default_auto_neighbor_without_artifacts`：无 provider 链路同 byte 面（`A中😀\0`=9）|

### 具名边界与非支持面

| 行 | 内容 | owner |
|---|---|---|
| B-01 | 双 i32 纯形无受检帧车道（min/max/未来 2-标量 op）——`ViewI32I32ToI32` 存在但 `(i32,i32)` 无 view 首参不可入 | 帧域扩展归后续 Plan（1114 记边界；共享 Auto 承载）|
| B-02 | List/Map 值不过帧——集合 op 无 Rust 分派面 | 共享 Auto 单源（972）；如需 Rust 集合=opaque 模型另立增量 |
| B-03 | image/frame（旧资产 17+2 `#[rs]`）| 领域 owner 980/OS064（S-20：媒体域不入 972-975；1114 不交付、不冒充）|
| B-04 | net TLS/proxy | **Windows AV 已交**（PLAN-1123 批 4，显式 `tls` 层；合同与错误表见 [m2-stdlib-network](../../ace-migration/m2-stdlib-network.md)）。代理认证、TLS 客户端证书、更广旧 HTTP API 另列 |
| B-05 | Linux 腿 | 1063（deferred-required/NOT_RUN，Windows required 全跑；`tls` 层同一源码臂随 1123 交 1063）|
| B-06 | inline.rs.at（旧 C-inline 面，无 `#[rs]`）| 旧 C 内联资产，AV/Rs 无义务面（N；旧轨保留）|
| B-07 | json.rs.at 旧 `#[rs]` 镜像 parse P6 坏（D-948-04）| 975 已交 std/json.at 共享 Auto 真身；T-04 显式外库行已交，旧镜像退役记录在案 |
| B-08 | 无参 i32 形（now_ms()/now_sec()/rand_seed()）——pure 需 view 首参、complete_sync 需单 i32 入参，均无车道 | 帧域扩展或 owned-unit 形另立增量（time 面以 owned clock 承载观测）|
| B-09 | 双字符串形（file 批量写、env_set）——帧域每调用至多一个借用 view | 帧域扩展另立增量；当前以字节粒度/单 view 子集交付 |
| B-10 | 完成口形（complete_sync/async）在 avrs 会话车道把模块变调度面——mir 能力域拒绝 | M2Build 工程执行器承载（1113 已证 sdk_pack）；sleep/now 旧 `#[rs]` 面归该 lane |
| B-11 | redis 真服务器验证依赖外部服务 | §10 具名 pending（运维输入）；RESP2 算法=975 共享 Auto |
| B-12 | M2Build 发布把工程全部产物十六进制嵌入一份收据写入工件存储（783 暂存预算 8 MiB，暂存+CAS 临时两份）：原始产物须 < ~4 MiB。未优化的 rustls 单元 DLL 3.1 MiB（收据 ~6.2 MB）不可发布，故依赖闭包在 dev 档内尺寸优化（1.7 MiB）；整单元 1.84 MiB 余量仅 ~0.26 MiB。结构性限制（收据嵌入方式 vs 暂存预算）归 783/978 owner，本计划不改预算 | D-1123-01（KNOWN-DEBT）|
| B-13 | `trust system` 在运行期动态装载 Windows SSPI/Crypt32，不在 DLL 导入表内，SDK 的 `system-lib` 运行依赖因此不含它们（始终存在的系统组件）| 记录在案；如需声明由打包 owner 决定 |
| B-14 | 一次 `tls_exchange`=一次 foreign 调用：默认执行墙钟 2 s 内只能观察 < 2 s 的交换；更长的 `timeout` 需宿主调高 `ExecLimits.wall`。默认请求期限 1500 ms 据此而定 | 语言/执行器合同，非本层可变 |
