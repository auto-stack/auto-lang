# ACE 有限模块绑定、装配与恢复（PLAN-944）

## 支持边界与真实入口

采用950统一 CST/source-query 前门的有限 scalar 模块：独立 ModuleInput、
pub/私有可见性、use/限定绑定、同名用户遮蔽及既有 i32/bool 自由函数域。
保留其语法/预算/拒绝条件；不引入全语言包系统、完整 CAP 或 legacy
Parser/AST emitter fallback。fixture `std.answer()->42` 只证明模块绑定，
不是实际标准库算法支持。真实943 `io.read_line` 证据进入两端清单；
其字符串/I/O行为交付仍由相应标准库实现承担。

`source-query::assembly::compile_assembly_product` 先真实绑定 namespace/
visibility，再将清单全部声明与实际签名/mode、源内容逐项比较，最后复用
既有事实 producer、type supply、HIR emission 和 verifier。缺声明、
重复/未知声明、类型/mode不符在 body solver/两端后端之前拒绝。
新数据模型与 legacy4 读取边界见 [assembly-manifest.md](assembly-manifest.md)。

## 依赖轴

显式 `Declaration::version` 包含 module/symbol、语义版本、签名/mode和
requirements；`SignatureCatalog` 在新 `ac944/declaration-context/v1`
域增强对应签名版本。空 context 保原 SQ-A 键。实际 infer-body 正/负读取
新鲜度仍由原 query session/solver 处理，不把全 manifest 放进每个 body 键。

批候选由已准入模块 binder 的实际 call edges 选择声明读者；不能将取消
候选留在 query cache 的读集当当前源图。一个正文编辑只解析被改声明，
仅元数据变化时以共享所有权保留同一已绑定源图，不复制整棵AST或环境。
需求变更不解析正文；provider 输入只影响其声明的 owners。共享候选并集
不强制两端都生产：目标自己的 context 未变且正文未改时直接沿原代码/
片段复用。入口 requirements 仅沿原实际绑定图收集显式声明需求，
不是另一套 capability 推导、求解或自动选型系统。

AV 的每函数 `module-unit-wire/2` 将物理 context 与真实 ABC 内容身份
绑定，继续经原 envelope/FunctionCode admission/ProgramSnapshot link。
Rust 的 context 只扩物理片段键，不污染 AXS1 语义键；见
[A2X增量与cache](../../auto-a2x/incremental-and-cache.md)。

## 版本与恢复

| 数据 | 新版本 | 原版本处理 |
|---|---|---|
| 中立装配清单 | `ace-stdlib-assembly/1` | 与 legacy schema4 分域，旧 FNV 含义不变 |
| source/query根 | `source-modules-root/3` | `/2` 无声明context，仍按原义读；不能充当新装配根 |
| AV模块根 | `av-source-modules/2` | `/1`/wire1 原义；旧入口拒新根 |
| A2X模块根 | `a2x-source-modules/2` | `/1` 原键/原文件字节；旧入口拒新根 |

根均复用同一个 ManagedStore/SyntaxArtifactStore，不另建数据库。恢复
从调用方持有的 receipt 与当前清单/源/requirements/物理输入准入：逐项
核源、grammar、CST origin/token、声明环境、事实键/依赖及 target required
闭包；AV 复验实际 ABC，Rust复验原片段/runtime/文件集。无扫描、解析、
solver/lower/print 冒充恢复；下一次正文编辑走原单函数事实及发射路径。
错误当前context、旧producer/不符ABI/runtime等必须拒绝。

Manifest codec 上限4MiB；原 source/target 根编码预算512KiB继续是资源
拒绝，未改为新的语言语法阈值。持久化不含 env 身份、grants、宿主地址。

## 原子发布与当前运行

`AssemblyHost` 在一个独占会话中协调 source 和两个独立 consumer，
先验证共同身份/所有候选，再准备包含两端 receipts 的 source根。
只有一次既有存储事务 CAS 替换 CURRENT，完整 required闭包在其活性门
保护下先成立，然后安装已验证的内存候选。没有先后两个 CURRENT。

FnCandidate 批量提交先校验同 env/同 generation/不同 slot，全部通过
才逐 slot swap 并递增一次 generation。目标候选绑定完整根身份，
provider-only同 source 变化也拒绝旧候选。AssemblyHost另有不持久化的
会话身份；空正文/link-only候选也不能跨宿主安装。

绑定失败、取消及数据库 root 退休失败均保旧可执行代码、Rust文件字节和
旧 CURRENT，下一次真实恢复读回旧态。候选只写不可变未发布数据，不能
取代当前源图或授权；成功后的下一恢复读回新态。

`run_authorized` 每次使用当前调用方检查当前 host 和入口实际可达的显式
requirements，拒绝先于 dispatch；恢复不授予权限。纯 scalar 模块不
伪装成 host-console，不序列化 checker、grants 或 native指针。
unused `io.read_line` 的审计证据不会成为源码可调用项或 stdin grant。

## 证据入口

- `cargo t ace-stdlib`：显式 SDK真实证明→两端真实manifest/执行42、7，
  不同绝对根身份一致；host/platform/producer/ABI/runtime/配方坏身份拒绝，
  与legacy4互拒。该测试包只在显式 ACE 验证中依赖真实SDK和两个后端。
- `ace-av` conformance 的 `assembly_944`：两端需求失效、当前授权、
  provider owner/消费者隔离、原函数代码/片段复用、恢复首编辑和再读回，
  同一事务失败保旧、foreign/link-only候选拒绝。
- `ace-source-query` 单测：实际签名/mode对照、声明context失效、
  source根恢复/首次编辑，以及取消候选读集不替代当前绑定图。
- SDK stdlib单测：真实独立adapter漂移负控、schema4独立wire/FNV金样、
  直接数据依赖进入SDK身份且整个ACE backend/worker目录不进入该身份。

完整职责组冻结并集为 `stdlib ace-source ace-semantics ace-artifacts
ace-av ace-a2x ace-stdlib`；不是每次迭代全跑，也不替代到期的独立批回归。

## 四轴 provider 选择合同（PLAN-947，2026-11-13）

标准库公共声明的装配选择按四轴分别建模（`crates/auto-lang-core/src/
stdlib_assembly/provider_selection.rs`）：**执行后端**（AV/native）×**宿主
平台**×**目标平台**×**provider 来源**（自有 Auto/薄 OS 适配/显式外库）。
选择只看**目标平台**（宿主 Windows 而目标 Linux 时选 Linux 的 provider，
V-09 钉）；默认序=自有 Auto→OS 适配，外库必须点名（唯一外库候选未点名=
`assembly.auto-foreign-fallback` 拒，永不自动 libc 回落）；公共边界的
AutoABI 与平台 ABI 分开（平台 ABI 冒充 AutoABI=审计反例
`assembly.platform-abi-as-auto-abi`）；无 provider/同轴重复/混 ABI/删
operation proof/缺本轮 grant 一律具名拒绝（`assembly.no-provider`/
`ambiguous-provider`/`mixed-target-abi`/`unknown-operation`/
`missing-grant`，后两者在 dispatch 前拒——恢复的旧授权永不入场）。
core/alloc/host/domain 分层随选择记录入账；Storage/allocator 接口仍待
M2 内存专题（不冻结签名/ABI）。`#if`/`#is` 目标条件语义与 sys/OS 迁移表
见 [ace-host-services.md](ace-host-services.md) §5——源码级 CTE 归 966，
本合同不另造。File/Task/Stream 资源面前探分账同文件 §3（capability-missing
不冒 PASS）。

## 平铺 provider 来源装配（PLAN-1043，2026-11-13）

在上述四轴记录合同之上，实际来源装配落位于 packages 面（设计见
[context-source-assembly.md](context-source-assembly.md)）：公共 `.at` +
`.at.at`/`.c.at`/`.rs.at`/`.py.at` 平铺候选经 `packages-ct` 裁剪
（`ac.source.cte.in/v1` 能力）逐文件 parse/语义合并；三轴（backend×
provider×target）显式上下文选择，Auto 默认=纯 Auto、缺则具名拒不回落；
`pub spec fn` bodyless 需求由所选实现同签名 fn 填充，重复/漂移/缺实现
调用前拒；`#in sys` 最小平台臂按目标轴选择导入（未选分支仅语法检查）。
packages store 根 `/2` 携实现层/上下文/候选存在集；未选 body 不入身份；
`/1` 根互不相认。旧 `.vm.at/.rs.at/.c.at` 目标层配方与 legacy inventory
（`.at.at`/`.py.at`→属主模块 Other 层）行为不变；`io.at.at` 永不冒公共
模块。AV 的 c/rs 配置执行受检 Auto 适配源，真实外库 dispatch 仍归 967；
A2PY 执行经 PLAN-1042 producer（拒绝旧 AST emitter）。

## platform.io/1 与 platform.mem/1 目标选择及恢复身份（PLAN-1039，2026-11-14）

- **目标装配选择**：`crates/auto-lang-core/src/stdlib_assembly/provider_selection.rs` 新增
  `platform_io_providers()`、`platform_io_requests()` 与 `select_platform_io()`；按四轴合同为 Windows 目标在 `Alloc`/`Host` 层分别选择
  `OwnAuto` 或 `PlatformAdapter`（`PlatformAbi`），拒绝未声明 grant（`assembly.missing-grant`）、缺失/非 Windows 目标或显式 `LibcCompat` 回落（`assembly.no-provider`）。
- **调用边界与持久化恢复**：所有 `PlatformCall` 参数/返回值受限于 `I32`/`StrView` 并绑定 1-based 带代际句柄（区分 `StdStream` 与 `OsMemRegion` 来源），
  每轮运行前经 `av-core/src/host.rs` 检查 `platform.io.{stdout,stderr,write}` 与 `platform.mem` 权限；
  `persist`/`restore` 不序列化任何旧 grant、OS 指针或句柄槽位，恢复后的产物在无当前授权时于 dispatch 前以 `host.unauthorized` 拒绝，并在重新授予以当前上下文恢复可执行。详细规格见 [native-platform-io.md](native-platform-io.md)。
