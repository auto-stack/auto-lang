# Design 28：Rust 互操作架构（Auto ↔ Rust Interop Architecture）

> **状态**：现行 canonical design（2026-09-07 立档）
> **来源**：2026-09-07 auto-musk 会话全链推导 + 两仓代码实勘（musk VM 轨 JSON 桥的
> 架构追问，逐层深挖至类型系统边界），结论定案与机制清单沉淀于此。
> **关联**：[PLAN-591](../plans/591-use-rust-any-crate-direct.md)（V1/V2 落地计划）、
> [consumer-parity-strategy](strategy/consumer-parity-strategy.md)（GOAL-006 战略面）、
> [442 跨平台收口](../plans/archive/442-cross-platform-closure.md) §7.2-7.4（musk
> 桥接形态定案）、430 管线（shim-metadata/auto-cache/dep_methods，本仓）。

---

## 0. 问题与范围

本文回答一个问题族：**Auto（VM 轨）调用 Rust 代码，边界在哪、代价在哪、为什么。**
具体包括：为什么 musk 的 extern 走 JSON 桥而三方库 shim 不用；为什么 `use.rs std`
零成本而 musk 的函数要注册表；"AutoVM 动态调用任意 Rust 库"的理论上限是什么；
a2r 与 VM 两轨的迁移分工。范围覆盖 Auto ↔ Rust 全部互操作面：墙内 native、
430 dep 管线、宿主桥（host_bridge）、a2r 转译轨、反向调用（当前缺位）。

---

## 1. 铁律：类型信息的生命周期

Rust 互操作一切设计的根约束：

> **签名与布局信息只存在于编译期。编译产物不携带运行时类型元数据
> （release 连符号都 strip）；无稳定 ABI（`#[repr(Rust)]` 布局未指定、跨版本可变、
> 字段可重排、niche 优化、泛型单态化后符号名不同）。**

推论：

1. 运行时"动态扫描一个编译好的 crate 获取类型和签名"**不存在**（rlib metadata 是
   rustc 内部不稳定格式；dlsym 只给裸地址，不知签名盲调 = UB）。JVM/Python/C# 的
   classpath 扫描 / Reflection 依赖运行时元数据，Rust 的设计里没有。
2. "Rust 程序随意调 cdylib"是错觉：每个调用点的签名是**开发者手写死在调用方代码里
   的假设**（`lib.get::<unsafe extern "C" fn(u32)->u32>>` 写错不报错，直接 UB）。
   类型知识的来源是人的脑子 + 编译器检查，不是 cdylib。
3. 实现语言的能力不传导给被解释的语言：AutoVM 用 Rust 写，不等于 `.at` 继承
   Rust 的编译期（同 JVM 是 C++ 写的、Java 仍需 JNI；CPython 是 C 写的、Python
   仍需扩展模块）。**VM 是机器，.at 是程序；机器用 Rust 造，程序仍需绑定层。**

---

## 2. 三时刻能力模型

互操作机制的本质分野只有一个变量：**签名（与布局）知识在什么时刻被捕获**。
捕获越早，marshalling 越"类型化"（越接近零序列化）；捕获于运行期 = 只剩通用编码。

| 签名捕获时刻 | 机制（本仓实体） | marshalling 形态 | 典型延迟特征 |
|:---|:---|:---|:---|
| **编译期（墙内）** | auto-lang 自己的 cargo 依赖（syntect/image/axum…，feature 门控）+ `native.rs` ShimFunc + `native_catalog.rs`（~1600 条） | 定型直调，零序列化 | 零 |
| **构建期（管线）** | 430：nightly rustdoc 提签名 → classify → emit cdylib shim 包 → C3 指纹缓存；`methods_pack.rs` + `dep_methods.rs`（manifest ABI 码） | C ABI 按码定型弹参压栈，元数据驱动，非序列化 | 首次构建一次，之后缓存零成本 |
| **运行期（宿主桥）** | `host_bridge.rs`（Plan 060 M3）：名字→闭包注册表，`fn(&str)→Result<String,String>`；musk 的 `musk_extern_dispatch` 网关（222 桩） | JSON 串全序列化 | 每调用一次编解码 |

**关键辨析**（易混三点）：

- "手写 shim vs 自动生成"不是分野变量——生成的 shim 一样要经 rustc 编译（编进
  auto-lang 或编成 cdylib），自动化改变"谁写"，不改变"何时定型"。
- 默认三方库不需要桥，不是因为它们特殊，而是它们**住得进 auto-lang 的 Cargo.toml**
  （依赖方向合法）；musk 的代码进不去（musk → auto-lang，反向成环），只能走运行时
  注册表。
- 430 的三方库不用 JSON，因为它在**构建期**拿到签名（生成器替它写好 wrapper，
  manifest 带类型化 ABI 参数码）；musk 落在运行期档，不是能力缺失，是"222 个函数
  三天通 parity vs 每函数办构建期手续"的取舍（442 §7.3"新 ABI"评估定案）。

---

## 3. 机制实勘清单（2026-09-07，含定位）

### 3.1 墙内 native（编译期档）

- `vm/native.rs`：`ShimFunc = Arc<dyn Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError>>`
  ——统一签名，每个可调 Rust 函数一份胶水（pop 动态值 → 解码 → 调真函数 →
  push 回）。静态段（id<10000）编译期注册；动态段（id≥10000）运行时登记名字。
- `vm/native_registry.rs`：Plan 250 懒注册 + `NATIVE_NAME_SET` 白名单门控名字解析；
  `RUST_STDLIB_METHODS`（Instant/Duration/PathBuf/Arc/Mutex/Box/RefCell）自动登记
  方法名 → 统一分发器（dispatch 3000）。
- 标尺实证：`std::env::var` 一行函数的 shim（`shim_env_var`）约 25 行，含 RC
  stake/配平纪律（Plan 510 G1-2 修复记录在案）——per-signature 胶水成本的实物样本。

### 3.2 430 dep 管线（构建期档）

- 提取：`shim-metadata/src/rustdoc.rs`（nightly JSON v53；固有 impl 归属在
  `inner.impl.for`；trait impl 全排除）。
- 分类：`classify.rs` 四类 skip——generic 无 mono 提示 / closure·未知参数 /
  Option 返回（None 语义 pending）/ 例外表；`types.rs` 字母表 `Ty`：
  整型×8/浮点×2/bool/Str/StrOwned/**Opaque（借用→句柄）**/**OpaqueOwned（按值
  →skip："VM 侧无法构造"）**/Generic/SelfTy/Void。
- 生成：`emit_cdylib.rs`——方法 wrapper 裸指针跨 C ABI（`auto_<Type>_<method>_<sig>`），
  对象本体留 shim 包堆，VM 侧持 `DepOpaqueObject`（ptr + 析构符号 + 库保活）；
  shimpack v1 三份元信息（manifest/signatures/rules）。
- 编排与缓存：`auto-cache/src/methods_pack.rs`（C3 指纹 = 工具链 × crate × 生成器 ×
  签名集，输入含 nightly rustc 版本串；nightly 缺席降级 Ok(None)）+
  `auto-cache/src/lib.rs`（Plan 082 全局内容寻址缓存 `~/.auto/cache/`）。
- 装载：`vm/ffi/dep_methods.rs`——manifest 指纹校验（`manifest.fingerprint`），
  METHODS 表挂 dispatch 3000 兜底段，FUNCTION_SIGS 自由函数签名元数据。
- 现状能力实证：`tests/ffi_dual_tests.rs` `ffi_dual_013_dep_method` + fixture
  `autolang_counter`——句柄构造/方法/基元串返回/builder 链（chain 标记）/static
  可用；字段、trait、泛型、Option 不可用（字母表墙）。

### 3.3 宿主桥（运行期档）

- `vm/host_bridge.rs`（Plan 060 M3，2026-08-22）：为 ash-gui 而造——merged 进程内
  直调与 HTTP 跨进程**双模式同构**（JSON 与传输编码一致，`.at` 桩单一代码路径）；
  `backend_abi.rs`（Plan 061）cdylib 插件装载，注册签名同 `HostCallFn`，要求
  同工具链同 target 树（ABI 版本号拒载兜底）。
- musk 复用（PLAN-044）：宿主构建 `Arc<AppState>`，数据 extern 注册为捕获 state
  的闭包——**AppState 永不过 JSON ABI**（442 §7.2 阻塞点 1 的解法）；`.at` 侧
  `extern_sigs.at` 222 桩统一 `musk_extern_dispatch(name, args)`，状态参（`_s/_ws @T`）
  不进 args。注意：双模式同构的原始动机在 ash-gui 路径；musk 缝走 JSON 是**机制
  复用**（现成注册表 + serde 替 musk 写掉全部 marshalling——DTO 本是 HTTP DTO），
  非 musk 自身有双模式需求。

### 3.4 musk 双轨实勘（互操作语义的完整案例库）

- 结构：手写 `src/*.rs`（hw 轨）∥ `auto-src/*.at`（Auto 源，42 文件）∥
  `src/auto_generated/*.rs`（a2r 产物）。`MUSK_BACKEND=vm` 时 `vm_backend.rs` 内嵌
  AutoVM 跑 `vm_entry.at`（宿主仍是 Rust musk 二进制）。
- 三件套分工：`extern_sigs.at`（签名+VM 桩转发，VM 轨专用）∥ `extern_impl.rs`
  （真实现，**两轨共享**）∥ `vm_backend.rs register_host_calls`（宿主闭包，VM 专用）
  ——接口成本"一付两轨"，VM 只多付桩+注册的适配层。
- **数据面/身份面分层**：JSON 只搬数据快照；对象身份走 `HANDLES` 侧表（i64 id →
  全局注册表；mpsc/broadcast 句柄）。drop-关流语义显式建模（run 结束摘除 pair →
  Sender 析构 → `mpsc_recv` 得 None → `.at` break 终止流，等价 hw 的
  `while let Some(v) = rx.recv().await`）；**a2r 产物同样调 id 版 mpsc**（一致性由
  构造保证——两轨跑同一 extern_impl 代码，非两实现对拍）。
- **已知形态差**：`musk_extern_dispatch` 宿主未注册时回退 null（空数据形态），
  a2r 轨无此分支（全静态链）——仅宿主缺席场景（独立 `auto run`）显形，musk
  完整宿主下 parity 门禁覆盖。

### 3.5 a2r 轨（编译期定型档）

- `use.rust` 编译期消费（生成 Rust `use`）；`#[rs]` 逃生舱原文透传；extern 桩在
  a2r 轨编译为 **dead 私有 fn**（链接器丢弃），真实调用经 `extern_impl::*` glob
  直连——**桥在 a2r 产物中不存在**，与手写 Rust 同 crate 图，LTO/内联可用。

---

## 4. 核心定理：编译预言机原则

> 管线反正要用同一工具链编译目标 crate（同编译保证已由 C3 指纹落地），因此
> **任何关于类型的问题，都可以通过"再编一段代码去问"获得答案。**

- 字段偏移 → 探针 crate 用 `core::mem::offset_of!`（1.77+ 稳定，**支持
  repr(Rust)**）/`size_of`/`align_of` 记入 manifest；
- 泛型 → 为指定实例生成 `foo::<ConcreteType>` 转发 shim，单态化由 rustc 完成；
- trait 方法 → 生成 `<T as Trait>::method` 单态转发，分发在 shim 编译期定死；
- 按值构造 → 构造器 shim（`new`/`From`）或按布局 manifest 逐字段写入。

**结论：现有 430 字母表是这栋楼的一层，不是天花板。**"动态调用任意 Rust 库"在
Rust 约束下的最大可行形态 = **构建期由编译器本尊制备绑定（签名+布局+marshalling
wrapper），运行期按指纹装载**。"动态"的准确含义是运行时装载构建期制备的绑定包；
"零构建、纯运行时、面对裸 rlib 现场学习"不存在（信息论意义上的不存在，元数据
不在产物里）。

**同编译保证的边界**（需钉全才完备）：C3 现为 工具链×crate×生成器×签名集——
签名集抓不住"私有字段变而公开签名不变"的情形，**features 组合与依赖图版本应入
指纹**（不同 features 即不同布局，无需恶意 ABI 随机化）。

---

## 5. 双轨迁移模式（a2r 先行、VM 后适配）

渐进式 Auto 化的实际引擎是 **a2r + `use.rust` + `#[rs]`**（auth 域为证：
`AppState.auth` 接线指向 a2r 版 store；store 内部继续 `use.rust std` + `#[rs]`
逃生舱——域内任意粒度混编，想停就停）。分野的根源是**方向性**：

| 轨 | 方向 | 渐进粒度 | 适配成本 |
|:---|:---|:---|:---|
| a2r | 双向（产物即 Rust，同 crate 图） | 任意函数级 | O(0)（use.rust 直连） |
| VM | 单向（VM→宿主；**无反向桥**） | 域级（自顶向下剥） | O(接口面)（桩+注册） |

- VM 无反向桥的后果：底层先迁不可能、跨域混编受限（某域迁进 VM 后未迁域无通道
  调它）——域退役必须以"域簇"为单位整体推进（KNOWN-DEBT musk #044 队列：
  auth → specs/wiki → relay）。
- VM 轨迁移进度天然落后 a2r 一拍（auth 在 a2r 已闭、VM 仍走 extern）——设计内
  时差，parity 门禁（26 测试 + SSE 对拍）保证滞后不漂移。
- `#[rs]` 块解释器不能执行、VM 的 `use.rust` 只认白名单原生库——两轨共享同一份
  `.at` 源时，这些边界点在 VM 侧仍需桩。bridge 逐域缩圈，域迁完 stub 退役。

---

## 6. 定案与在途

**定案（不再重议）**：
1. musk 数据 extern 走 JSON 桥 = 依赖方向 + 运行期注册表 + 机制复用的合计结果；
   性能非议题（请求粒度、前有 LLM/VM 解释开销；热路径 SSE/channel 早已走
   HANDLES 侧表不过 JSON）。
2. `dep` 声明保留显式形态（version/git/path 源配置有价值）——**dep+use 合并、
   use.rs 自动解析、auto.lock 取消**（2026-09-07 用户裁定，PLAN-591 修订）。
3. 三时刻能力模型与编译预言机原则为 V1/V2 演进的架构依据（PLAN-591：
   V1 字段透明=布局 manifest；V2 行为等价=trait 单态转发+泛型实例化+反向
   adapter 原型）。

**在途/待决**：
- 反向桥（宿主→VM 调 `.at` 函数）：整个互操作面唯一无人动过的方向，解锁底层
  先迁与跨域混编——候选独立 plan。
- 指纹钉全配置（features/依赖图）——430 补强项。
- 借用/生命周期敏感返回：保守 clone 已定；跨界借用记账（激进路线）研究级。
- Send/Sync 线律、多线程回调泵——PLAN-591 T5 原型后的后续计划。

---

## 7. FAQ（高频追问速查）

- **为什么 JSON 不直调？** 运行期档的注册表只能存一种擦除签名；JSON 是该档的
  通用编码（遗产+serde 省胶水+可调试）。直调存在于编译期档（a2r）与构建期档
  （430），见 §2。
- **为什么 std 库不用桥而 musk 要？** std/三方库是 auto-lang 自己的依赖（墙内，
  编译期档）；musk 的代码进不了 auto-lang 的 Cargo 图（环），只能运行期注册。
  见 §2 关键辨析。
- **natives 表不是有动态注册吗？** "动态"= 名字晚绑定到**已编译**的 shim
  （懒注册 + 白名单）；shim 本体躲不掉 per-signature 胶水（§3.1 标尺）。
- **rustdoc 拿到签名后"重定义成 Auto type"为何不够？** rustdoc 给语义签名，
  不给布局（偏移/填充/niche）；直读字段需布局权威=同编译探针（§4）。签名知识
  ≠ 布局权威。
- **全局缓存/单次编译解决了 ABI 稳定？** 在管线产物域内成立（指纹+装载校验）；
  边界是 musk 等宿主自建二进制在担保域外 + 指纹需钉全配置（§4 末段）。
- **句柄 id 会不会破坏两轨一致性？** 不会——id 机制是两轨共享层（同一
  extern_impl/HANDLES），一致性由构造保证；残留差异=宿主缺席时 VM 的 null 回退
  与时序边界，parity 门禁覆盖（§3.4）。
