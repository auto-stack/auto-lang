# A2R Project Runtime & Distribution

> **Status**: experimental
> 路径：`experimental/ace/crates/a2x-rust`（语言通道运行时/降落）+ `experimental/ace/crates/a2x-worker`（`run-language` op）+ `experimental/ace/crates/m2-build`（A2R build lane / SDK） | 依据：Design 60（PLAN-1116）

本文件冻结 `IF-ACE-A2R-PROJECT/1`：普通 Auto 工程的 A2R 运行时面（任务/生成器/异步）、同源 `use.rs` 外库受检调用，以及 m2-build 的 A2R 真编译腿与仓外 SDK 分发。语言通道身份见 [ordinary-language-and-stdlib](ordinary-language-and-stdlib.md)。

## 调度与生成器运行时（AC-01 / V-01）

- **任务/actor**：沿用 968 共享脱糖（`sched_desugar.rs` → `__sched_*` intrinsics）与 `sched_rt.txt`（std 线程 + 有界通道；FIFO 邮箱、ask/reply 端口、orderly drain）。语言通道经 `lower_coll.rs` 的 sched 分发臂降落。
- **异步块 `~{...}` 与 `~T` future 函数**：**惰性**——`__sched_future_*` 创建不占线程的 lazy future（负句柄槽），`await`/`go` 首用物化（spawn + port 发布）；正数=ask 活 port，`await` 按符号分派（`a2x_sched_await_any`）。AV 惰性序合同：`let f = work(42); print("queued"); f.go` 必先观察到 `queued`。thunk 陷阱=进程级 sched fail 发布（可捕获跨线程陷阱传播归 catch 链债）。
- **生成器**：前端 `agg_lower.rs` 状态机 + `for-in` drain 已通语言通道。显式 `next()/close()` 协议的 A2R 所有权合同：
  - 非 Copy owned 的整局部 Assign=移动（杀源）；owned 槽（VariantCtor payload 等）消费杀源；
  - **状态字段写回**：`&mut __ace_iter` 字段 Option 解包物化（`__gen_v*`）被变方法（`push/remove/...`）变更后，语句末 `state.field = Some(temp)` 写回（AV 句柄就地别名 vs A2R 逐读克隆的桥）；此后 freed/moved 过滤。
- **具名债**：生成器跨挂起捕获 **List 视图参数**需真共享句柄别名（交错双生成器 oracle 排除 copy-in/out；`A2xList` 为 904 冻结拥有式 Vec）；模块全局 finally 变体需包绑定缝（`ac.source.modules.packages/v1` 路由）。

## 同源 `use.rs` 外库（AC-02 / V-02）

- **接面**：worker `run-language` 请求增 `foreign` 段（声明形 `kind`/`provider`/`artifact_root`，与 AV 同形状）；仅 `language` profile 接受。
- **准入**：`load_prepared_foreign` 复验全身份轴——descriptor schema（`plan1093.provider-descriptor/1`）/kind/provider、由冻结源**重导**的 provider/shim digest、prepare stamp（digest+rustc 身份）、工件在场、DLL sha256；C provider 具名拒（`a2x.foreign-kind`）。编译走公共前门 `compile_language_source_with_foreign`。
- **能力**：请求 caps 须含该外库 facts 的全部 capabilities（`foreign.scalar`/`foreign.resource`），缺=具名拒 `source.foreign-capability`。
- **降低**：`lower_coll.rs` foreign 臂（文本道=视图、标量=纯值）；bridge 域在**降低时**校验（出域=`a2x.unsupported-expr` 具名拒，先于装配）；语言运行时挂 generated bridge（同 967 受检 shim exports，`#[link(name=...)]`）。
- **运行**：链接=cdylib 的 MSVC import lib（`X.dll.lib`→常规 `X.lib` 名），DLL 随产物 exe 进程依赖；语言通道 trap 增 `AutoTrap::Foreign(i32)`（collections 通道冻结字节不变）；跨 FFI 的 fallible status=`trap:Foreign`。
- **冻结域**：967 窄域（参数 ⊆ {ViewText,I32,Bool} 至多一个 ViewText ≤3 标量；Pure{I32|Bool}/Fallible{out:I32}）。owned 资源/回调/完成端口 A2R FFI、直接 Cargo adapter 等价证据=后续批。

## 工程构建与仓外分发（AC-03 / V-03）

- **A2R build lane**：m2-build `lane_a2x` 对 A2r 目标在 emit 后走真实 rustc（scratch 工作目录、spawn 计数 `rustc-a2r`），产出 `targets/a2r/<name>.exe` + build-receipt（rustc 身份/entry）。source 腿只产文本；编译是本显式腿（同 native 腿模式）。
- **SDK**：`build_sdk` 的 A2r 臂记 `msvc-crt` run_dep + `rustc` build_input；exe 以 `role=exe` 入 artifacts（sha 绑定）。
- **仓外消费**：`run_sdk` 对 A2r 将观察协议归一为值（`value:<n>`/`trap:` stdout → exit/无 exit，同 a2py 车道）；重定位 SDK 只需 crt 运行时族。
- **拒绝与恢复**：工件 sha 失配具名拒；`restore` 读回旧可用根；业务 edit 身份键变（构建命中分验）。
- **具名边界**：多模块（包）工程走 a2x-store 包冷路径（`ProjectInputs` 现单源限制）；m2-build 层 Rust 依赖 edit 缓存分验随包缝批。

## 边界

图形 W3 与 Linux 专用验证按原 owner（1074/1103/1076/1077、1063）；AutoMan 自有入口由其 owner 接收；v0.5 接管=1117。
