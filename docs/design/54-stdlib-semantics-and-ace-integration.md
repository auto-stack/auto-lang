# Design 54：Auto 标准库语义与 ACE 接入合同

> **target-state / proposed r2｜2026-11-09（初稿2026-11-05）**。r2按最新决定将未执行945–948的研究和接入统一放到M1正式tag之后，门以[Design55 §1.1](55-ace-m2-complete-language-and-product-replacement.md#11-m1-tag标准库准备计划的统一进入门)为准。初稿的现状/日期属于历史快照；943/944已经归档，当前能力读取其正式Specs。本文仍不切默认、不修改current Specs或代替实际交付证据。
> 取材主检出 HEAD `a0750086c` 及当时磁盘文档；存在其他会话 WIP，Design 51 等的磁盘 hash 单列于 §12，不能将其等同该 SHA 已提交内容。当前实现以 Specs、实际 producer/consumer 与正式交付收据为准，后继开工须重核来源。

## 1. 决策与适用范围

保留一套 Auto 公共标准库，建立可版本化的操作语义、分层实现与 ACE 接入合同。不另建 `ace.std`，不把旧 `NativeInterface` 或整个 a2r-std 当作 ACE 兼容层，不用新增 `.ace.at` 后缀代表全部新引擎能力。

本文固定架构和兼容原则；具体首批符号、可执行 profile、物理适配与资源预算由各 Plan 的有界前探冻结。前探改变已承诺语义/支持集时必须修订 Plan，不得静默缩分母。全部实施仍等待用户后续授权。

| 既有设计 | 保留的权威边界 | 本文补充 |
|---|---|---|
| [Design 33](33-stdlib-runtime-and-http.md) | 公共 API/后台实现分层、真实装配、VM/生成服务的不同拓扑 | 将装配合同接到 ACE 语义身份、适配证据与新 producer；不重写 HTTP 实现 |
| [Design 46 §7/§10](46-context-profile-capability-cross-ecosystem.md) | core/alloc/host/device 分层、能力与 effect、CAP 进入门 A/B | 近期只登记显式有限需求和已有 profile 准入；完整推导、求解、自动选择/裁剪仍须原门，不提前实施 CAP |
| [Design 51 §5](51-shared-a2x-and-hir-to-rust.md) | Checked HIR→A2X，MIR→AV/native；旧 a2r-std 经核验才可复用 | 定义公共操作到各 producer 的绑定、类型/资源/错误适配与增量失效 |
| [Design 40](40-compiler-artifacts-and-cache-lifecycle.md)、[41](41-interface-metadata-and-foreign-abi.md)、[49](49-native-single-mir-lowering.md) | store/恢复/安装、公开 foreign ABI、唯一 MIR 语义权威 | 接入既有服务，不复制 cache、类型系统、资源分析、ABI 或解释器 |

非目标：全语言模块/泛型/反射扩展、自动 header importer、所有标准库重写、跨平台认证、GPU/MCU 扩面、默认产品切换、旧引擎 disable、M1/M2 分母裁定。旧 required/accepted-diff/NA、原 oracle 与资源预算保持。

## 2. 现状与差额（只读事实）

1. [stdlib Spec](../specs/stdlib/project.md) 和 [后台装配](../specs/stdlib/design/backend-assembly.md) 记录公共 `.at`、目标层、VM native、两 Rust facade 与领域服务。它们不是各模块×各目标的等价矩阵。
2. [AssemblyManifest](../specs/stdlib/design/assembly-manifest.md) 已区分 inventory 与 actual，具备双指纹及 declared/resolved/bound/signature_checked/executed 分级。应保留，签名通过不冒充行为等价。
3. `stdlib_assembly/model.rs::AssemblyTarget` 当前为 Vm/Rust/C，Environment 为 Native/Browser；它不完整表达 ACE producer、语言 profile、宿主授权与平台 ABI。
4. `io.at` 仍含 `#[vm]`，`str.at` 方法也有 VM 标记；公共声明与旧执行机制未完全解耦。旧组织教程的装载序与现行 Spec 不一致时，按实际入口核查，不从教程推定支持。
5. PLAN-872 内建矩阵保留有意差异。`primitive-str-byte-len.md` 对 `len()` 的旧描述与项目卡的后续绑定矩阵存在差异；943 必须按具体拼写/静态或动态派发/消费者复核，禁止全局选择一份描述覆盖另一份。历史 738 解析失败数量不作为当前计数。
6. `math.at` 的公开 abs/min/max 参数是 **i64**；ACE 有限源码 i32 profile 不因此取得这些 API。typed wide 成功也不等于宽源码支持。`square/cube` 的 int helper 可作公共算法复用候选，但需验证可见性与整数 profile。
7. ACE 已有 text/collections 等 runtime；collections 的真实机器调用、所有权与失败提交见 [CONTRACT](../../experimental/ace/crates/collections-runtime/CONTRACT.md)。`data-runtime` 是探针支撑，不能因 staticlib 存在便宣布语言标准库支持。
8. `av-core/src/host.rs` 已有 `console/1` 的精确签名、授权、单次/单轮预算；`av/src/console_host.rs` 明示为探针缝，不是产品 CLI 默认。已有机制必须优先接入，不能将探针当作已迁移产品。
9. [共享 foreign facts](../specs/auto-abi/shared-interface-and-scalar-bridge.md) 已交有限标量双 consumer；938 源 import、939 资源桥各有独立合同，未交付不能当正式依赖。私有 text ptr+len 不别名公开 AutoABI。

## 3. 分层与所有权

| 层 | 逻辑内容 | 依赖及 owner |
|---|---|---|
| L0 最小基础 | 基础类型/协议、Option/Result 操作、静态数学、只读视图算法 | 无隐式 OS/heap；不重定义编译器已拥有的内建类型。标准库 owner 实现公共算法，ACE 语义 owner 维护合法性 |
| L1 分配与数据算法 | 拥有字符串、动态集合、编码、JSON 等 | 方法级标出分配与失败；纯算法尽量 Auto 单源，原语由 runtime/backend 负责。JSON 在本轮只有迁移前探，不随集合交付自动获得支持 |
| L2 宿主服务接口 | console、fs、clock、random、net、task、stream | 显式服务合同及每次运行授权；实现可由 CLI/嵌入宿主注入，不暴露 Tokio/VM 内部对象 |
| L3 领域库 | HTTP server、UI、图形、数据库集成 | Web/UI 等保原 owner；语言核心保留中立值转换与宿主端口，不吞并领域实现 |

首批运行验证平台限既有已认证 Windows x64；Rust/C 具备跨平台能力不等于 Auto 已认证其他平台。其他平台逐项另验，不推定支持。

逻辑层不等于立即改包名/移动目录。`auto.core/auto.alloc` 仍是设计名，现有 `use auto.*` 不批量重命名；确需新公开名字由符号合同与兼容审查决定。读取 str 视图与构造 Str 分别计能力，不能在包根粗标一次 heap。

## 4. 标准操作语义合同

每个可接入操作具有稳定逻辑 ID 与单独的语义版本；拼写、别名和 receiver 形式另行绑定。ID 不采用 native 数字 ID、Rust 路径或 private ABI 符号。首版序列化名/存放模块由 943 冻结，不在本稿发明已发布格式。

| 必须信息 | 规则 |
|---|---|
| 公共身份 | 模块、符号、方法/静态、可见性、参数/返回及泛型域；同名重载与用户遮蔽按语言解析规则 |
| 数值与文本 | 整数宽度、checked/溢出、浮点边界；UTF-8 字节/Unicode scalar/码元单位及索引边界分别明确 |
| 所有权 | view/mut/move、是否分配/clone、借用有效期、失败后的输入/out、最后消费者与析构 |
| 可观察结果 | 正常值、可恢复库错误、语言 trap、宿主失败、取消/资源耗尽分开；精确 EOF 与空结果 |
| effect/需求 | 显式有限需求、stage、确定性、宿主依赖；未知不得当 pure 或默认授权 |
| 适配与实现 | provider 身份/版本、target/profile/platform、调用模式、逻辑到物理转换和证明 |
| 验证与兼容 | 独立 oracle、允许的旧差异、支持/未支持/未知、真实执行身份与适用范围 |

编译器原语、公共 Auto 函数、宿主函数不是同一类 provider。原语只覆盖必要内建操作；普通算法不得为了复用变成一大批 MIR opcode。由同一事实生成声明/绑定可以减少重复，但不能再用生成的 expected 自证实现。

### 4.1 兼容政策

- 原拼写/消费者的旧结果先记录再迁移；不在本设计把 `len`、`parse_int`、File 的历史空值行为统一改成新含义。
- 新的明确单位 API 可复用已有 `byte_len` 合同；歧义旧入口不放入无差异认证集。API 语义改变需要显式版本/新入口/迁移裁定，不能只换 provider。
- i64→i32 不属于普通无损适配；bool→int、异步→同步、资源→整数也不能靠 cast/名字相似证明。
- 不支持的公共调用在绑定/准入阶段具名拒绝，不返回 0/空串冒成功，不 fallback 到旧执行器。
- 只对选中/实际引用的闭包检查实现支持；坏导入描述仍须结构校验。用户自定义同名函数不得误判为标准库。

## 5. ACE 接入与装配

逻辑路径：公共声明/算法 + 操作合同 → **现有统一 source 前门** → 绑定/语义校验 → Checked HIR；其后 A2R/A2C 消费 HIR 及共享资源事实，AV/native 经唯一共享 MIR。宿主导入在被选中后进行完整签名/需求/授权准入。

公共模块的有限加载接在统一前门/现有绑定环境。944 先证明 namespace、依赖、可见性与恢复闭环；可采用受检声明环境承载有限模块，但不得字符串替换、预执行标准库函数、手写 HIR 冒 source 或回到 legacy Parser 生成 ACE 事实。若当前模块能力不够，具名记录最小缺口并修订依赖，不能悄悄扩成全语言模块工程。

装配身份分离以下轴：

| 轴 | 例/边界 |
|---|---|
| producer | legacy VM、ACE AV、ACE native、HIR→Rust、HIR→C；Rust/C 语言名不足以区别新旧 |
| 语义 profile | 既有 scalar/text/collections 等有限域；不等同 CAP 场景 profile |
| 环境/宿主 | CLI、嵌入会话、浏览器、UI、Web；编译支持不等于运行授权 |
| 平台/运行库 | target triple、ABI、runtime 内容/版本、工具链及适配策略 |

扩展既有 Manifest 的版本化模型与读取边界，保留共同装配指纹/消费者收据指纹的不同用途；legacy 旧读取路径不因新字段被静默重新解释。未知新 schema 在对应入口拒绝/按现行策略判陈旧，不能因 null 指纹相同算新鲜。中立模型不得依赖 legacy VM/AST emitter，可在有真实消费者的最小范围抽取，不另建无人消费 registry。

`executed` 只表明某个 case 在某身份下运行；“语义符合”还必须绑定 oracle 与适用输入。backend 缺能力与当前宿主未授权、库错误与语言 trap，应分别诊断。

## 6. Runtime 与宿主边界

AV/native/Rust/C 可以使用不同物理表示，但共享语义合同。既有 runtime 按真实用途保留：文本分配/回收、集合节点与帧、类型化入口、trace、探针各自分账。no_std 不等于没有系统依赖——当前 text runtime 仍直接使用 kernel32 并有分配/poll hook。

首批 console 优先复用 863 的精确导入与 per-run authority；保持现有预算/输出原子提交/取消 poll 语义，不新增即时取消承诺。产品 SDK/CLI 接管另走原迁移门。trusted native provider 不是内存隔离机制。

File/Socket/Stream/Task 目标态采用类型化不透明资源；跨边界只传受校验的凭据或规定的 ABI 表示，禁止把 VM 引用编码成整数跨线程。合同须覆盖：创建失败、拥有/借用、重复关闭策略、move 后误用、错误回滚、取消与最后 owner 回收、会话退出。保存状态不序列化进程指针或把旧权限当新运行授权。

947 首期只实现有限 console 接入，File/任务/网络等做可执行合同前探；若触及 public foreign 资源，则等 939 正式交付或据缺口新立包，不强行把 console 简单导入当完整资源桥。文件系统路径权限须由宿主落实，不能由编译期 capability 标签代替。

## 7. 增量、CTE、恢复与安装

- 依赖分别记录操作语义版本、被读取的签名/effect、公共算法 body、provider 内容、profile/ABI/runtime/toolchain。body 编辑只重产真实受影响者；provider/link 变化可以更新装配，但不能无理由重做无关函数。
- callee effect/需求变动即使签名不变，也使依赖其摘要的调用根重新准入。完整自动推导待 CAP；近期显式需求按同一身份接入，不建立第二套长期求解器。
- CTE 只用已支持的真实 AV 执行及显式允许输入，宿主 app 权限不继承。当前阶段无 CTE adapter 的 API 诚实拒绝；纯标记不意味着所有操作已能在编译期运行。
- 恢复字节先验 shape/版本/依赖/预算，再消费；恢复后第一次编辑也要证明正确失效。序列化不包含函数地址、句柄或每轮授权。
- 新候选准入/安装失败保旧版本可用，资源代际沿既有安装/lease；本系列不添加动态 loader 或热卸载。
- 仓外 clean 安装、重定位和生成产物独立消费由原安装/发行 owner 认领。开发态 locate-or-build runtime 不能充发行证明；948 形成真实差额与关闭条件，不把它的准备完成当 M2 发布绿。

## 8. 行为与证据合同

| 规则 | 区分性样例/观察 | 边界 |
|---|---|---|
| 文本单位 | `A中😀` UTF-8 为 8 bytes、3 Unicode scalars；`e`+U+0301 为 3 bytes、2 scalars；NUL 不截断 | 具体拼写/返回宽度由 943 冻结，不能据此重新定义旧 len |
| 整数 | i32 MAX+1 与 MIN/-1 在已支持 profile 为原具名 trap；公开 i64 min/max 不伪装 i32 支持 | 沿原 profile，不新增全源码宽数值 |
| 绑定身份 | 错同宽签名、单位、mode、provider 版本在调用前拒；用户同名合法函数仍接受 | 对实际选中闭包，独立 callee 计数证明拒绝发生时机 |
| 资源 | clone 独立；失败前检查→提交保旧 owner；重复/moved/错代际资源受控拒；最后消费者释放 | 集合用原资源 profile；File 等仅前探，不冒已交付 |
| 宿主授权 | 同一个合法 console 调用，授权时输出精确 UTF-8，未授权时 provider dispatch=0；restore 后需新授权 | 先覆盖已有 AV/native；Rust/C 未接入不算同绿 |
| 增量恢复 | 改一个操作 provider、一个函数 body、一个需求字段分别观察生产计数与实际新结果；坏缓存拒绝后下一次恢复读回 | 复用已有 producer 观测/store，不造摘要自报 PASS |

各端对独立 oracle，不只四端相等；数值、诊断、资源、真实输出与编译失败分别记账。新 fixtures 的源码/expected 由人工合同给定并经语义复核，不由 producer 生成 expected。被排除/未跑/环境失败保持原分类，不从分母消失。

## 9. 分期计划与进入门

| Plan | 交付范围 | 依赖与停止边界 |
|---|---|---|
| [943](../plans/archive/943-stdlib-semantic-contract-and-compatibility-inventory.md) | 已归档：实际公共 API/provider/consumer 盘点、语义合同与兼容实册 | 保历史；后继只重核将消费的实际合同，不重新立项冒新成果 |
| [944](../plans/archive/944-ace-stdlib-binding-assembly-and-cache.md) | 已归档：有限模块绑定、装配与恢复 | 保正式支持边界；不由归档推定全库接通 |
| [945](../plans/945-ace-stdlib-core-math-and-text.md) | 冻结的基础算法/文本最小切片，四端实际语义验证 | M1-TAG + D943/D944/D960/D947；不偷换i64，完整职责交972 |
| [946](../plans/946-ace-stdlib-collections-and-owned-algorithms.md) | 有限公共集合算法及拥有资源验证 | M1-TAG + D945/D962等自身依赖；Storage接口在M2讨论，不预设实现 |
| [947](../plans/947-ace-stdlib-console-and-resource-host-contract.md) | 公共库分层、Storage方向/平台条件与装配合同，保原console与资源前探 | M1-TAG + D943/D944/D960；不等待完整966/967/972，实际资源按正式能力消费 |
| [948](../plans/948-stdlib-domain-migration-and-release-readiness.md) | 全域差额、有限消费者前探、FFI/平台/资源及发行交接 | M1-TAG后方可按D943开始调查；最终消费D945/946/947，不反向等待972等实现 |

当前时序：M1正式验收/合并/tag→对应M2实施授权→947合同→945→946；948在tag后可按D943先调查，最终收齐基础三包后交接。tag前不执行945–948调研任务。每包仍使用自己的唯一worktree，簿记在主检出；本轮只改文档。

本六包是**基础接入与迁移准备**，不声称做完即全库/全产品替换。948 必须把尚未迁移的每个具名需求交给对应 owner，记录具体依赖、关闭 oracle 和后继立项入口；JSON、task、fs、HTTP 等不能靠一句“后续”消失。完整 CAP 仍遵守 Design46 的 A/B/C1–C3；M1/CA-E 与 M2 原准入由其 owner 判断，本稿不新增或减免原发布门。

## 10. 验证及工作流

正式入口采用固定 `cargo t` 组，实际包/语义触面决定并集；共享语义修改补 AV/native/A2X 必要消费者。初拟 stdlib、ace-source、ace-semantics、ace-lowering、ace-av、ace-native、ace-a2x、ace-runtime、ace-console 等，详各 Plan。`cargo t --list` 本轮只读核验，产品测试全部未执行。

work/review 覆盖相关完整组及区分性反例；merge 在最终冻结 revision 核对应并集，身份未变证据可复用。不是每包跑 `cargo t ace` 或 `cargo t all`；周期全量另按既有 due 门。新增公共合同包若需要新固定组，由对应实施任务登记，不能只跑叶用例。

证据/探针/日志/导出副本统一放 `D:/autostack/plan-artifacts/plan-<NNN>/`，开工登记 owner/用途/状态；源码与常规 target 留专属 worktree，必要外置 target 也只在该 scope。merge 将摘要折入 Plan 后删除 scope，guard clean 后移除树；不写 attachments、不用 junction、不跨树共享可写 target。

## 11. 尚待有界研究的决策

| 决策 | owner/任务 | 关闭条件 |
|---|---|---|
| 首批公共符号及源 profile、旧 len 差异、宽度/错误合同 | 943 T-01..03 | 完整实册与逐项兼容决定；不确定行有具名 owner，不能进入无差异支持集 |
| 中立模型位置、有限 module 绑定与 schema 升级 | 944 T-01 | source→两个实际 consumer→恢复首编辑闭环，零 legacy fallback |
| 公共算法可见性/所需最小扩展 | 945/946 T-01 | 正式前门源码正反例，若缺语言能力先修订依赖，不手写 HIR 顶替 |
| File/任务/流跨界及 939 适用性 | 947 T-04 | 资源状态/错误/授权表与真实可行/不可行证据，明确待立项 producer |
| JSON 泛型/运行时类型策略、领域入口和发行清单 | 948 T-02..04 | 有界独立 probe、原消费者身份、owner/接收条件与阻塞分类 |

## 12. 来源身份与外部参考

以下为本轮编辑关联文档**之前**读取的 SHA256；当前 Specs 未改。开工应记录届时实际版本，不将这些静态快照当持续证据。

| 相对路径 | SHA256 |
|---|---|
| `docs/specs/stdlib/project.md` | `d144494d7e3b5e2d913c76bfeae5c568ab8e3a6e6057517831ffdea1de2db512` |
| `docs/specs/stdlib/design/assembly-manifest.md` | `9c802df4673706a3dea3a33d230a98f6d834f0cc3779d2c21db940ab5267df96` |
| `docs/specs/stdlib/design/backend-assembly.md` | `0038af77d5e5b2e98c6f09743193d63c43ec3191b4485e7d0d9159fd5f350657` |
| `docs/design/33-stdlib-runtime-and-http.md` | `5c753fc0660a786d1188bb17acc14955debf2dc50e769dc9573f18252f0db986` |
| `docs/design/46-context-profile-capability-cross-ecosystem.md` | `b3b119fe7b0569386eb564682a7af3e310d79ef459f3ccac263a8a5d8221a92e` |
| `docs/design/51-shared-a2x-and-hir-to-rust.md` | `d2109f629df8b129c531d091c7c2584bbbd90059f4b51404112e3cb3c9430231` |
| `crates/auto-lang-core/src/stdlib_assembly/model.rs` | `87f14f7e2cf19ddce991c260ed186434796b83eabb17c78d0bf6417aaca118dd` |
| `experimental/ace/crates/av-core/src/host.rs` | `30d6dcc475850cd007f4032352fda01644e4ff6ccc012b13a34854867c1eb7d4` |
| `experimental/ace/crates/foreign-contract/src/lib.rs` | `c5d3d347ea64dfa8d42d4e03ceb72a381d69a70b38614b906f261a42a6e86913` |

外部设计输入沿上一轮官方调研：[Rust core](https://doc.rust-lang.org/core/) 的平台无关基础与 [no_std/alloc](https://doc.rust-lang.org/core/attribute.no_std.html) 的分离；[Go WASI](https://go.dev/blog/wasi) 的 runtime/系统调用适配；[WIT](https://component-model.bytecodealliance.org/design/wit.html) 的资源所有权/借用合同；[Wasmtime](https://docs.wasmtime.dev/api/wasmtime_wasi/struct.WasiCtxBuilder.html) 的宿主文件授权。本文借鉴分层/接口边界，不引入 WASM/WIT 依赖，不据其他语言能力推定 Auto 支持。
