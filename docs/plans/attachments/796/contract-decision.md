# PLAN-796 T-01 合同冻结（contract-decision）

2026-10-16；stage:work；树 `D:/autostack/.wt/lang-796/auto-lang`（分支 `plan-796-dev`）。
本文件冻结 §10 授权范围内的技术选择（"只允许在本文范围内选择"）；每条决策标注来源条款。
变更须走计划修订并使受影响证据失效。797 只读消费本文件及其引用的 schema 常量。

## D1 workspace 与依赖边界

- 新独立 workspace `experimental/ac-semantics/`（自带 `[workspace]`、独立 Cargo.lock，
  members = `contract` / `core` / `probe`）；不改根 members、不改旧 crate manifest。
- `contract`（ac-sema-contract）：仅 blake3 依赖，零产品后端依赖。
- `core`（ac-sema-core）：依赖 contract；ac-core AST adapter 在 `core-ast` feature 后
  （默认关——默认构建即证明模型无 auto-lang/UI/VM/Cranelift/auto-man 依赖，V-09 用
  `cargo tree -p ac-sema-core` 复核）。
- `probe`（ac-sema-probe）：依赖 contract + core(core-ast) + ac-artifact-probe(783) +
  ac-core；子进程复用现有合入设施，不自建无限等待执行器。

## D2 名义身份（logical id）

`LogicalId = blake3("ac796.sema-probe/1" ‖ "ac796/logical/v1" ‖ len₈·(package, module, kind, name))`。
组件顺序固定于函数签名，登记顺序/重启 intern 顺序不影响；同名不同 package/module 不合并；
kind 参与（fn `Pair` ≠ record `Pair`）。持久形态只含该 hash + 组件，永不含 intern 索引/表槽/指针。

## D3 ref envelope 与 producer

`SemaRef { owner: "ac796.sema-probe/1", schema: "ac796/semaref/v1", payload_id }`。
消费端 admit 校验 owner+schema，不匹配=受控 `RefError`，绝不重解释。`DeclRef.verify()`
重导 logical id 防篡改（坏图不 panic，V-01）。

## D4 内容版本（ContentVersion）

`ContentVersion = domain_hash("ac796/content/v1", canonical_bytes)`；canonical 为
codec 的确定性编码（排序 map、声明序保留字段/参数、span/空白排除）。只改来源位置
（span 变、语义内容同）→ ContentVersion 不变（§5.3 来源更新走 source-map 依赖，
不炸语义值）；改签名/body/字段集 → 变。

## D5 页（page）envelope

`PageHeader { schema: "ac796/sema-page/v1", producer, codec_version: SEMA_CODEC_VERSION=1,
kind, content }` + 紧凑二进制 body（FnMeta / RecordMeta / InferBody / ModuleIndex /
CachedResult 五类）。解码全有界（≤512KiB body、vec ≤2²⁰、嵌套 ≤8192），尾字节拒绝，
TypeDesc 解码迭代式（显式栈，4096+ 深链不触原生递归）。typed view（`PageView`）按 kind
检查访问，不跨 kind 重解释。

## D6 描述 codec 覆盖（给 797 的表示支持）

TypeDesc：Unit/I32/U32/I64/U64/F64/Bool/Char/ByteBuffer/Opaque(name)/Fn(params,ret)/
Record{DeclRef 间接}/Mode{view|mut|owned}。递归 record 一律经间接 DeclRef 组成可验证
RecGroup（组内容版本=成员排序后 hash；成员重复/非 record 成员受控拒绝）；按值递归布局
是 797 的受控拒绝面，本表示层结构上不可表达按值无限递归。u32/f64 等仅是元信息表示，
不扩大 i32/bool 推导实现面（§5.1 边界原文）。

CT/实例/runtime 接缝 Schema：`ac796/ct-seam/v1`（CTRequest/CTResult/PortableValue/
RuntimeProjectionPolicy/InstanceKey；admission 拒 Unknown/地址/宿主 handle/未检可执行
输入；结果标记仅 `model_compute`，`vm_execute` 由真 AVM 路径独占）。

## D7 查询键（query key）

组件：query、package、module、algo=`ac796-bidir-worklist/1`、schema、producer、
target-sem=`logical-only/v1`（语义目标无关；797 拥物理布局，将来 target 耦合语义须显式
换 namespace 而非静默）、caps=`core-i32-bool/1`，及按查询类型的输入指纹：
- InferBody：目标 fn-sig-fp + fn-body-fp + fn/type 候选集 fp + **called-sigs-fp**
  （目标语法调用的被调签名集指纹——T-05 换轮测试暴露"同键不同内容"与 783 CAS
  冲突后冻结：被读签名变化必须是键变化，一个键结构上不可能持有两种内容；
  783 的 Conflict 行为是正确的，语义层不得绕过）；
- FnSig：fn-sig-fp；RecordFields：field-set-fp；ListCandidates：candidate-set-fp。
细粒度依赖 manifest 随 payload 存放，命中时整体重验（§5.4"先验证完整新鲜"，
覆盖 body/负依赖/source-map 键外输入）。
调用者不变而 callee 签名变 → called-sigs-fp 变 → 键变 → Miss（fresh 重算）。
target body 变 → 键变 → Miss。callee body 变且调用者未读其 body → 仍命中
（消费者读什么才算什么，不搞一刀切接口 hash）。

## D8 783 store 受管隔离 namespace

- layer = `"S796"`（绝不写 `"A06"`——不冒充 783 Future 登记的旧 producer）。
- carrier ChunkKind = `SnapshotRecord`（通用版本化记录框；绝不用 `Hir`——
  不把 HIR kind 冒充语义值）。783 代码零修改。
- 读入端三重 admission：783 CAS 地址校验（framed bytes hash）→ 页 envelope
  schema/producer/codec 校验 → consumer 身份校验（页 kind=CachedResult 且 target
  logical id 与当前查询一致）。坏地址/owner/schema/kind/target 全部受控错误。
- `lookup_only` 无 compute 路径（禁调反控是结构性的）；`query` 流程返回
  compute_calls 计数，命中断言为 0。

## D9 资源配方（探针运行策略，非语言上限）

- 单次普通查询：步数 ≤1,000,000、wall ≤5s。步数定义：一次 worklist 约束求解、
  或一次 AST 节点访问、或一次作用域/候选查找，各计 1 步（单一单调计数器）。
- 超限状态：`ResourceExhausted`（基础设施状态，非类型错、不可缓存）；取消：
  `Cancelled`（同样不缓存）。PASS/取消/资源失败永不入 cache；Ok 与"同完整键的
  确定性 Diags"可缓存。
- 页 body ≤512KiB（随机小签名查询 payload 上限）；(T-06 起) worker 请求/响应
  元数据各 ≤4MiB、stdout/stderr 各 ≤1MiB——本阶段在库边界以 Budget 检查落实，
  IPC 边界等 788 协议后再接线。

## D10 内存预算（§5.4 压力面）

- 热点页 LRU ≤8MiB；索引/暂存/求解合计另 ≤8MiB；进程内受管总额 ≤16MiB
  （计入 hot + index + scratch 的字节记账）。
- 压力面：≥32MiB 实际语义页流式生成落盘（逐页 encode+publish，不全量驻留）；
  private bytes 增量目标 ≤64MiB（OS 计数）。
- 深链 ≥4096 级全路径验证（编解码、求解、Drop 全迭代式）。
- 512 次修订发布/释放后无 live 超额 → 按 783 继承水位跑 GC 收敛；用户哨兵
  （软 root + lease）数据不变。
- 预算不足报资源状态，绝不报语法/类型错。

## D11 周转配方（§5.5 硬门）

- 参考机器/负载在 T-05 冻结记录（CPU/OS/工具链版本、1MiB 等价 core 模块生成器
  版本+种子）；测量路径 = 请求→依赖验证→cache 读取+解码→推导→发布（单函数
  body 编辑场景），P95 < 1000ms，30 样本；另报 P50/P99 与全冷/广泛失效。
- 六场景各 30 样本：no-op / 局部 body 编辑 / 签名或字段集合变化 / 重启磁盘热 /
  跨工程 / GC 并发；正常/超时/取消样本不删除。
- 阶段 trace 从首例（本 T-01）即记录：请求/依赖验证/cache 读解码/推导/发布各段
  独立计时，嵌套不重复计；真实 compute/hit 计数、I/O 字节数、状态码入 trace。
- 工具准备时间单列，不从用户路径等待中扣除；AutoDown 实际 adapter/IPC 接入后
  重测同一路径（T-06）。

## D12 首例与 .case.ad 合同

- 首例 = `sema_cache_first_case_*`（7 用例）：跨进程 put→exit→get 命中不调 compute、
  callee 签名变 Stale、target body 变 Miss、query 流程 compute_calls 控制、
  负依赖（新定义 g 后旧 unknown-name 裁决不得作为命中复用）、跨 package 不合并、
  非法 payload 受控拒绝。
- `test/cases/ac-semantics/*.case.ad`（T-06 起移至 `crates/auto-lang/test/cases/
  ac-semantics/` 并翻转 `status: executed`，协议升级为 autolang.case/2 +
  adapter probe + recipe sema/query-v1，见 D13）；oracle 为独立审阅的期望查询
  输出快照；Rust 定向对应用例与其同输入同期望。

## 首例验证记录（T-01 收据）

- `cargo test -p ac-sema-contract`：5 passed（identity 名义/重启序/tamper/envelope/域分离）。
- `cargo test -p ac-sema-core`：13 passed（含 occurs 环注入受控、步数超限基础设施态、
  取消、5000 深操作链迭代走查）。
- `cargo test -p ac-sema-probe --test sema_cache_first_case`：7 passed（见 D12）。
- 环境注记：sccache 默认端口撞 Hyper-V 保留段（os error 10013），本 workspace
  验证统一以 `SCCACHE_SERVER_PORT=44001` 运行。

## D13 probe/run 接缝冻结（T-06，按 788 最终协议修订）

- **adapter/action**：`adapter: "probe"` × `action: "run"`（v2-only；v1 带 probe
  字段=解码拒绝，v1 接受/拒绝集字节不变——plan788_r3_ 兼容面全绿）。
- **注册配方表**（解码器常量 `REGISTERED_PROBE_RECIPES`）：`sema/query-v1`。
  case 只能引用注册名——绝不携带可执行/路径/shell（未知 op/字段=定位错误）。
  797 在同表追加 ABI 配方，不另写 runner。
- **步骤面**（`options.steps`，1..=64）：`query`（完整查询流：lookup→miss/stale
  才真算→发布；报告 cache/compute_calls/bindings/diagnostics/stages）；
  `edit-source {from,to}`（字面量须恰好匹配一次=可判别编辑 oracle；链式生效）；
  `restart`（结束当前 worker 会话，父侧以**新进程**续跑剩余步骤、同一 store
  root——统一 runner 内的真实跨进程持久化路径）。
- **sema 绑定**：`options.sema {package,module,kind:"infer-body",fn}`。
- **期望面**（`expect.probe[]`，与 query 步一一对应）：`cache ∈
  hit|miss|stale|published`（worker 报 `published<miss>` 等带原因细形，父侧
  归并为粗词表）、`compute_calls`（命中断言 0=禁调反控可声明 oracle）、
  `bindings_file`（case 相对边车 JSON，oracle 身份含其内容 hash）、
  `stages_include`。诊断结局（diags<miss>）v1 不发布不声明——由库级套件覆盖。
- **源类别**：`syntax: "atom-data"`（.atomdata，Atom 数据载荷）随接缝注册；
  sema/query-v1 只消费 `.at`（执行期 admission 拒绝其它类别）。
- **IPC**：`semaworker/1`（acworker/1 同形：begin/request/end、nonce+
  request_hash 回显、deny_unknown_fields、单响应、父侧墙钟+有界读+stderr
  即杀）。worker=sema-probe `serve` 子命令；工具身份经 `SEMA_CASE_WORKER`
  env 覆盖或源树定位+release 自建（可信宿主配置绑定，case 无工具字段）。
- **判定**：全在父侧（auto-lang `test_cases::sema`）：信封/schema/回显/形状
  admission → 逐 query 步比对 oracle → 结构化回执 `probe-receipt.json`
  （NativeRecord 携 query_steps/compute_calls 计数与 toolchain 轴）。
- **边界保持**：auto-lang 零静态依赖实验语义栈（worker 经子进程 IPC）；
  ac/syntax 既有语义零改动（plan788_r3_/ac_case_tests 全绿）；共享文件
  （contract.rs/ir.rs/suite.rs/mod.rs）本窗口单写者=796。
