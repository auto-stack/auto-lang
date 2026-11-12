# auto-test-case 架构（裁判来源与依赖约束）

> **PLAN-858 开发期周转口径**：ACE 有效样本按升序取第 `N-floor(N/10)` 项（从 1 开始）为 P90，硬门为 P90≤2000ms；正式样本数不减少，10 项最多 1 项超过 2000ms，等值通过。缺失、非法或错误结果不得计入绿；保留各路径原有零值准入、真实窗口、结构与资源断言。P95/P99 作为诊断与历史数据保留；架构基本成形后以实测另定长期目标。详见 [TestCase v2 周转口径](../auto-test-case/profile-v2-native.md)。


> 状态：active（A 阶段）。为什么这个 crate 存在、依赖方向为什么是单向。

## CA-D 标量桥 adapter（PLAN-937）

- 正式 recipe `abi/ca-d1-bridge-v1` 经既有 `probe` adapter/父执行器
  （`test_cases/abi.rs` 双 recipe 分发）与同一 abi worker；contract
  绑定腿集 source/typed-self/typed-n2c/typed-c2n，与 `abi/interop-v1`
  严格互斥。严格协议与原 probe 来源隔离：注册不等于认证 PASS，独立
  oracle/原始行证据/退出码纪律（正退出=pe-exit diags）沿 797 父路径。
  细节见 [ace-ca-d-bridge-certification.md](ace-ca-d-bridge-certification.md)。

## 裁判来源：pinned vendored parser

- `vendor/autodown-test-parser/` 三文件快照（markdown_parser/ial/block_model），
  `PROVENANCE.md` 记录来源 revision 与 sha256；指纹守卫测试防漂移。
  不手改、不在 build/run 再生、不依赖浮动主检出。
- 树只提供标题层级/anchor/fence 语义；**载荷字节永远来自原始文档 bytes**，由
  `raw_ranges` 定位并与树逐 fence 校验（计数/语言/EOL 归一内容三向一致，任何
  分歧=硬错误）。不取 serializer 载荷，不另造 Markdown case 语义。
- 报告 toolchain 身份包含 vendored parser 的编译期指纹（`include_str!` 哈希），
  旧报告跨快照漂移可检测。

## 依赖方向（避免 GUI/自举环）

```
crates/auto-test-case  ──仅依赖──▶  vendor/autodown-test-parser
crates/auto-lang（adapters/orchestration/CLI）──▶  auto-test-case
```

- 协议核心零 GUI/零 auto-lang 运行时依赖（`cargo tree --edges normal` 仅 vendored
  parser；auto-atom/auto-val 仅 dev-dep，供解码器测试构造 Atom 值）。
- 执行 adapter（AVM run / a2r emit）与 materializer/编排/`auto-case` CLI 在
  auto-lang 侧 `src/test_cases/`。PLAN-901测试feature拍平：auto-test-case 为
  两 crate 常驻依赖（无 feature 门），测试代码同编译形态可见，运行由固定组/
  nextest 过滤与原 ignore/环境条件决定；无窗口构建运行保持。被测 a2r 不得在
  运行时再生裁判（parser 快照固定）。

## 阶段观测通道（类型边界，非文本）

- AVM：`run_with_capture_and_path_staged`——旧 API 的 staged 孪生，engine 陷阱点
  （typed `VMError` 仍存活处）记录变体名经 thread-local 中继出线程；旧 API 签名
  与行为原样。
- 单文件 a2r 发射：parse/CTEE 错误保持类型化传播（Lexer/Syntax/Type/…）。
- 分类规则（`classify_staged`）：类型化 trap → run；Lexer/Syntax → parse；
  Type/Name/Comptime/MsgWithSource → type；MultipleErrors 全递归取首个叶子；
  其余（Io/panic/合成超时/无名 Msg）= Infrastructure → ERROR，**永不满足语言
  负例**。
- AVM语言失败在oracle比较前持久化并注册`failure.json`
  （`autolang.avm-failure/1`）：实际stage、可空code、组合source哈希及entry字节哈希。
  PASS与oracle不符的FAIL均保留相同事实；基础设施错误不生成该语言失败产物，
  写入失败仍为基础设施ERROR。`error.txt`仅供阅读，不能据文本补造stage/code。
  该产物不提供值类型、位模式、trace或生命周期证明；成功路径仍为result/stdout产物。
- PLAN-941的旧BASE严格reader只认领B14/B20、B16、T07、K04五个原负例：
  先核对原required/binding/source及完整Case身份，再核对注册产物的owner/hash/size
  （单份≤16KiB）和实际stage/code。缺code保持UNOBSERVED，基础设施或文本产物
  不能补足命名错误；父侧非PASS始终保红。K04仅证明原批准的解析拒绝，
  不证明借用规则。登记表仍是executed=0的发现数据，运行证据单独判定。
- 已知边界（F-773-R2-01 债务）：a2r project 模式两入口仍把 parse 错误包成无名
  Msg → 归 Infrastructure（fail-safe：永不假 PASS，但 project-mode 强错误
  oracle 不可满足）。

## B 阶段执行拓扑（PLAN-788：native worker 与全量父侧判定）

- 单管线复用：auto-lang `test_cases::ac` 编排，`experimental/ace/crates/case-worker`
  独立 workspace 持有实验编译栈（ac-core 前端/verify/lower、783 store、787
  runtime——无第二执行器）；**auto-lang 零静态依赖实验编译器/Cranelift**
  （红线），经 `acworker/1` JSON-lines 有界会话 IPC 子进程调用（deny_unknown_
  fields、nonce/request_hash 回显+父侧比对、行限预分配）。
- **PASS 判定永在父侧**：typed admission（`validate_response`：schema/身份/
  形状/解码后流 cap）、manifest 分配前全校验+持久化（`admit_and_persist_
  artifacts`）、流比较（`compare_stream_field`）为可复用公共入口，周转 harness
  与 runner 使用同一入口；worker PASS 不受信。
- 父侧 I/O 生命周期统一：stdin 写入线程化并纳入请求墙钟；stderr 排空超限即
  时杀树（与响应/截止同 ≤10ms 轮询循环）；stdout 读取带界；产品消费 run 经
  同一 contained 执行器（不重编译、不裸 spawn）。
- 身份/stale 见 [profile-v2-native.md](profile-v2-native.md)（逐行适用集合）。

## 执行限制与试点边界

- 可信有限 pilot：无外部进程/网络/FFI/自举重型执行；未实现的 budget/fixture/env/
  resource 声明 = unsupported ERROR；超限 = resource ERROR（不伪装语法错误）。
- scratch 纪律：每 run/case/check 唯一 scratch 子目录；不改全局 cwd/env；不写
  旧目录 `.wrong.*`。
- 测试档位接线：`cargo t`（test_cases_tests 家族）/`cargo tv`
  （`vm_file_tests::pilot_aggregate_vm`）/`cargo tt`（`a2r_tests::pilot_aggregate_a2r`）。

> 来源：docs/plans/773-autodown-testcase-protocol-pilot.md §2/§5（Design 35 A）。

## probe 通道（PLAN-796：sema/query-v1 受限 adapter）

- 契约层：`Adapter::Probe × Run`（v2-only）+ 注册配方表（首个
  `sema/query-v1`；797 按同合同追加 ABI recipe，不改协议）+ 声明式步骤
  （query / edit-source / restart）+ atom-data 源类别。
- 执行层：语义 worker 子进程（`semaworker/1`，与 acworker 同形信封/回显/
  有界 IO/父侧墙钟）；restart = 统一 runner 内真实跨进程持久化；判定全父侧。
- worker 全生命周期：父侧 Job containment（suspended→assign→resume，子孙
  收割）、响应后仍待退出、stderr 越限即杀、4MiB 元数据上限；退出状态/版本
  镜像精确匹配；工具身份四轴（protocol/recipe/exe-sha256/worker-version）
  入报告与 stale 判定（ac/probe 分族）。
- 语言负例：parse 失败以 typed `diags`（stage=parse）返回并按声明 error
  completion 判定，不归 infra ERROR。
- case 永不携带 shell/任意 exe；worker 工具身份仅可信宿主配置绑定
  （SEMA_CASE_WORKER）。


## PLAN-798：已实现的AV消费边界

probe/run新增注册recipe ac-av/core-run-v1，av绑定entry/caps/可选session及run/edit-source步骤；case不能命名任意exe，跨recipe字段/步骤拒绝。保持sema/query-v1、abi/interop-v1和原v1/ac/syntax支持；ABI拒绝AV专用Run，混合报告身份按三recipe独立惰性解析，行顺序不影响结果，篡改某worker只使该recipe stale。AV worker avworker/1协议、avworker/1.0/av.core-i32/1镜像和nonce/request/source/产物身份由父侧准入后判定。worker准备/运行受限启动、Job杀树、双流cap和单截止，超时/截断/错actual为infra/失败，不把expected发给worker。技术计时和驻留增量的边界见[AV缓存验收](../auto-av/cache-and-conformance.md)。

PLAN-828 正式持续会话入口：`av_probe_worker session-serve`（一进程一会话：init/ready
恒定身份+初始派发身份基线、编辑流发布已验证当前值+分相计时+计数快照+派发载荷身份
（对象地址+代际）+worker 线内存峰值、close），父侧 judge 拥有测量窗（编辑入队→
父侧**全部校验完成后**为终点，IPC 与逐请求校验在窗内）与独立期望；每发布绑定期望
修订（revision==req_id）与确切安装目标（`expected_install`），未改函数派发身份跨修订
稳定；双门=P90≤gate ∧ 全对 ∧ 结构（未改零阶段/零整模块/恰一确切安装 ∧ 修订/目标/身份
门）∧ 完整性，gate=0 必红；worker 中途死→缺样本红完整性，不静默缩战役；初始化排除
单列（init_ms）、ready 先判；完整观测（逐样本计数差分/诊断/身份/worker 相位与
输入/二进制身份）经 `JudgeReport::write_evidence` 落盘留存（成功与失败同留）；
`frontend-attribution` 子命令为双线预算 A 线隔离测量口。one-shot run/recipe 保留不变。

## PLAN-817：cte/scalar-query-v1 recipe（旧 recipe 零修改）

正式 recipe 表追加 `cte/scalar-query-v1`（v2 probe/run、atom-data 源、
steps=query|edit-source|restart 以 query 结尾、cte 绑定 entry/caps/session）；
worker 身份轴族 `cte-worker`（stale 第二登记点——配方分支外独立轴，混合 cte/av
行序独立+漂移即红）；outcome admission：judged 步骤上的隔离 resource/cancel/infra
态=ERROR 行（永非语言裁决）、语言类 diag 对照声明的 error completion（cte.*→
parse/verify/capability/run stage 投影+message matcher）、期望 success 默认 FAIL；
未知字段/recipe 拒绝；未注册 recipe lint 拒（列注册集）；坏 worker 工具身份=ERROR
不绿。runner/适配器见 `crates/auto-lang-core/src/test_cases/ac_cte.rs` 与语料
`crates/auto-lang/test/cases/ac-cte/`。
PLAN-818 mn recipe `ac/mir-native-scalar-v1`：mn 绑定 entry/caps/session +
edit-source/run/restart 步骤；含 edit 无 restart 的序列走**增量会话**（D828
单 body 事务+单元级 COFF 重发/历史复用+逐步真 link/run+授权 trace 库全程链接
+逐轮原始 PE 字节），`[run]` 全请求路径保留为冷/恢复场景（不标增量）。
worker 协议 `mnworker/1`（加法扩展 rounds 模式：一常驻进程×每 step 一问一答；
单发 steps 形态原样可解）。

**rounds 边界（失败活跃性与限额）**：全部阻塞 IO 位于会话线程（写=邮箱投递、
读=单一绝对墙下接收、回收=截止治理轮询、teardown 先杀树再 join 全线程）；
stdout 限额在**累积期间**生效、行队列有界、stderr 超限在每次接收与收尾传播
为 infra；协议感知 fault 夹具（no-stdin/stop-after-warmup/close 两形态/stdout
两越界+洪泛/stderr 越界）验证预算内 infra+进程树消亡，边界有效对照全绿。

**计时语义（§5.5 有效窗口）**：正式 V-12.1 门=父侧观测**连续**窗口——父侧
准备（源拷贝+字面量 exactly-once 搜索/插入+载荷序列化）→IPC→worker 定位/
编辑/链接/运行→响应→父侧准入→**独立期望比较**（观察者缝
`AC_MN_INCREMENTAL_EXPECT_EXITS`：期望只在父侧、worker 永不见；mismatch=
verdict Fail；数量失配=infra）；worker 侧连续 wall_ms 保留为被覆盖证据
（断言 pw≥w）。语法回退（warm_edit_fallback）不标记增量、保留已提交前缀、
其诊断按先诊断后结构准入传播——回退不是增量样本。

## ac-av/data-run-v1 recipe（PLAN-820）

新 recipe `ac-av/data-run-v1`：注册三处追加（REGISTERED_PROBE_RECIPES 白
名单、step 词表=run+edit-source、av binding）；worker case-serve data 分支
（av_core_version=av.data/1、parse/verify 诊断分级）；父侧判定同 core 语义
（成功=exit 值、diag 分级、infra=非语言判定）。

**typed 面**：`typed:<id>` 入口（av.data-fixture/1 冻结八样本：u64::MAX/
i64::MIN/-NaN payload/-0.0/f32 1.0/f32 NaN payload 0x7F80_AD00/bool true/
i32 42）经真实准入链（data MIR→AVAD→AV）；worker 报告实际产值的规范
{kind,bits}（i64/u64/f64 十六位、f32 八位、i32/bool 不补零——单字符 "0" 是
唯一规范零），父侧持**独立冻结期望表**逐项比对 tag 与精确 rawbits（worker
之言从非 oracle）；record 源成功同样附 typed 报告（bool/宽标量不再丢失于
i32-only 通道）。

**family 隔离与形状校验**：typed 存在性仅属 data family——core/i32 成功仍
必须有旧 value 通道，非 data 响应携带 typed 即准入门协议违规（infra 非
绿）；data 的 typed 形状校验在 **oracle 无关的准入层**（任何 exit/value
oracle 匹配不可绕过）：六 tag 之一、宽标量固定宽度、bool 恰为 0/1、i32 不
补零，畸形=准入违规非绿。typed 步骤修订哈希绑定入口身份。

V-10 反控：wrong-exit/wrong-diag/未注册 recipe/worker 三故障=ERROR、
wrong-bits/wrong-type-tag（父侧期望破坏缝仅测试用）/未知 fixture id/整数
边界矩阵。验收：corpus 绿（av-data-first/typed/bool/int-zero.case.ad）+
ac_av_data_probe 8/8。


## PLAN-827 源码程序 corpus/验证消费者

PLAN-827 新增已交付 `ac-av/core-run-v1` recipe 的源码程序 corpus 与验证
消费者。源文件字节与虚拟源身份由既有 runner 解析；worker 只收字节、永不读取
期望结果。父侧拥有 typed actual 比较与 source/worker 身份准入。私有 overlay
与同 store/新 worker 路径保留实际执行。不支持的 source、infra 失败与 guest
trap 保持既有区分判定。不新增 recipe 注册、协议 decoder 或第二执行引擎。


## PLAN-821：ac-av/owned-run-v1 recipe

PLAN-821 新增 `ac-av/owned-run-v1` recipe：三缝 append-only 注册
（auto-test-case 围栏 REGISTERED_PROBE_RECIPES / 父侧判决 ac_av.rs 冻结表
OWNED_CASES / mod.rs 分派），owned 面准入与逐面判决；V-10 sabotage 双面
（result→owned-result 轴、ledger→owned-frees 轴）与 worker 故障三态
（exit9/extra-frame/wrong-hash 均非绿）。corpus `av-data/821`：借入正控
（exit 15）与 mut 写回正控（exit 808）。沿实际 auto-lang-core test_cases
owner，无新执行器。

## PLAN-891：ac/mir-native-typed-v1 recipe（native typed 结果通道）

- 形状：probe 适配器 + mn 绑定（entry + typed{fixture,args}）+ 单 run
  步；source 为 `typed-fixture`(`.mnf`) 语法的 mn-typed-fixture/1 文件。
  判定通道新增 `typed` expect 面（`{kind,bits}`，canonical hex 规则与
  AV typed 判定同一来源）——仅 success 完成态可携带，trap/resource/error
  帧按 `mn.typed.<status>` 映射 run 阶段错误完成。
- 身份：typed 行 stale 轴 = [recipe, mir-lowering, mn-worker-exe-sha256,
  mn-typed-abi]；worker 侧请求永不携带 expected，回显 nonce/request_hash
  必须逐位一致，counters 强制 run_invocations≥1（always-execute）。
- 模块发现：suite `typed-native-891`（固定组 case-ace-m1-typed-native-891，
  domain ace）；SDK 对照（绿 12 行 + 错 bits/错 kind 红控）挂 ace-conformance
  language_conformance。

## PLAN-852：ace-m1 认证拓扑与身份分离

认证器是**独立 bin**（`ace-certify` → `auto-lang-core/src/test_cases/ace_certify.rs`），
不属 suite runner 派发面，任何日常 scope 不隐式调用（G08）。三个身份轴各归其位：

- **证书 `toolchain.identity`** = 认证当次实际启动的 rustc 的完整 `-vV`（CLI
  `--rustc` > env `RUSTC` > PATH 单一规则，经 contained 有界通道实测；R2-A 裁定：
  构建期常量不得冒充认证证据）；
- **`default_identity_at_certify`** = 构建期默认（build.rs 捕获
  `BUILD_TOOLCHAIN_IDENTITY`，并作为 `build-rustc-identity` 轴随每次 suite 报告
  输出），供日常快照**零 spawn** 漂移比对；
- **收据身份**：source/golden/actual/wrapper/oracle/reference 六 hash 轴（wrapper
  单源=`oracle/wrapper-main-v1.rs` 版本化文件，代码 include_str 引用）+ 冻结
  flags `["--edition=2021","-O"]` + deps std-only + 双侧 observed 输出。

收据验证为**类型化精确键集 schema**（`ace.m1.cert/1`）：每层未知/缺失/错型字段
拒绝，无 filter 宽松；Rust（`validate_receipt`）与 PowerShell
（`verify-ace-m1-852.ps1` 共享 `Invoke-ReceiptValidation`）双消费者一致，结构破损
=REJECTED（--force 亦不覆盖），文件轴漂移=STALE 显式重认证。日常快照 face =
emit+byte-compare+严格收据验证+周期性 CIM 祖先采样进程观察（采样器而非无损创建
计数；引擎 worker memoized 维护构建属 818 纪律单列，不算认证面）。wrapper 单源、
认证 `--scratch`、流治理（双独立读线程/溢出杀树/墙钟）见
[a2r-golden-certification.md](a2r-golden-certification.md)。

## PLAN-822：ac-av/collections-run-v1 recipe

新 recipe `ac-av/collections-run-v1`：沿 821 三缝 append-only 注册模式
（REGISTERED_PROBE_RECIPES / 父侧冻结表 COLLECTIONS_CASES / 分派）；avworker/1
域 `collections:<id>` fixture 入口，typed 面准入与逐面判决；语料身份
`av-data/822`。父侧持独立冻结期望表（value/allocs/frees/live 台账面）；V-10
反控 seam：`AV822_COLLECTIONS_EXPECT_SABOTAGE`（期望破坏缝）+ AV798_FAULT
复用（worker 三故障态）。未注册/跨 recipe 拒与三 recipe 既有围栏一致。

## PLAN-824：ac-av/dynamic-run-v1

正式注册 `ac-av/dynamic-run-v1` 与 `av-data-824` typed fixture 语料；worker 执行真实 MIR→ABC→AV 和 portable 恢复，父侧持独立冻结期望，检查值/资源与失败反控。漏例、跳过、错误结果或 worker 故障不能冒充通过。M1 中 typed 动态证据与不支持的源码/native 路径分开登记，见 [ace-m1-conformance](ace-m1-conformance.md)。

## PLAN-823：ac-av/closures-run-v1 recipe

新 recipe `ac-av/closures-run-v1`：沿 822/824 append-only 三缝注册模式；语料
`av-data/823`，avworker/1 域 `closures:<id>` fixture 入口（首例=move 捕获
List<record> 的 creator-exit 调用+可变 env+drop，手算 29/台账 5alloc 5free/
live=0）。父侧持独立冻结期望表 CLOSURES_CASES（value/allocs/frees/live 面），
worker 身份/漏例/跳过/错结果=非绿；V-10 反控沿 judge 谎言族与 worker 故障缝。
`scripts/verify-av-closures-823.ps1` 九 Scope 聚合（first-case/semantics/
admission/resources/cache/compatibility/autodown/baseline/turnaround），缺
case/worker、脚本控制与 stale-evidence 身份（41 文件 hash+依赖 HEAD）都有常驻
负控。交叉 owner/类别拒绝与 824 正控由 av_closures_823_dynamic_824_compat
常驻。

## PLAN-825：ac-av/loaded-run-v1 recipe

append-only 注册 `ac-av/loaded-run-v1`（与 closures/dynamic 等既有 recipe 互不复用 id）；`av-abc-825` 语料（B01/B04/B09/B13/B14 + first-case/lifecycle）走真实 source→HIR→MIR→ABC→AV→store→新 worker 恢复。父侧 judge 按独立 expected 判定并要求运行计数轴：成功优化 run 的 loaded build/PC/参数计数非零、reference PC 扫描/拷贝为零（反向 reference 路径要求对偶计数）。零选择、ignored、重复、伪 tier、抹峰值与错结果 counter-control 必使门非零；compat 收据绑定 HEAD 祖先 + 证据 log hash + 脚本身份。

## PLAN-826：ac-av/compact-run-v1 recipe

- 追加 recipe `ac-av/compact-run-v1`（REGISTERED_PROBE_RECIPES + avworker/1 步骤词汇 run|edit-source + av binding；worker case-serve 接受并 `set_wire_compact`）。核心版本轴 = EXPECTED_AV_CORE_VERSION；工具身份轴 `av-wire-codec=auto.abc.av-compact-core-i32/1`。
- fixtures `test/cases/av-abc/826/`（first-case 同 825 首例源）；父侧 `judge_compact_runtime` 以 `abc_wire_compact_units/closure` + loaded 计数器判真，`AV826_JUDGE_SABOTAGE` 四轴反控 + 错 oracle。
- scoped 验证入口 `scripts/verify-av-826.ps1`：first/targeted/compat/incremental/baseline/bench 六 scope 聚合进 acceptance；Assert-Selected 拒绝 0 选择/ignored/缺失/重复必需例（guard self-test 自证）。


## PLAN-868：lowering 契约版本轴（mir-lowering）

HIR→Typed MIR lowering 的契约版本（`auto_mir_lower::MIR_LOWERING_VERSION`，初值
"1"）作为**必报** toolchain 身份轴 `mir-lowering` 进入 av（全 8 族）/cte/mn 三族
probe 报告与 mn 收据外层——测试侧以 admission 镜像常量
`EXPECTED_MIR_LOWERING_VERSION` 上报（与 `EXPECTED_AV_WORKER_VERSION` 同模式，
bump 未同步镜像即身份断言红）。stale 裁决 av/cte 分支 applicable 含该轴：旧报告缺
轴判 STALE（`native.mir-lowering missing (now <v>)`），为声明的重构期一次性代价；
判分/收据规则不变——`ace.m1.cert/1` 精确键集 schema 与 852 双消费者校验零改动，
cache/store 键不含该轴（lowering 出库不改输出）。等价比对口径（逐 check
status/actual，剔除 duration/spans/exe-sha/ledger_peak_* 等易变字段）由 PLAN-868
T-02 冻结，驱动脚本 `scripts/verify-mir-lowering-868.ps1`。

## PLAN-871：native 后端身份轴（ac-native-backend）

`ac` 适配器（`acworker/1`）的 linked 行（build/run，实际发生链接）必须携带
toolchain 轴 `backend`，父侧绑定为报告轴 **`ac-native-backend`**：

- 值 = `mn:<mn-abi>+mir-v<MIR_LOWERING_VERSION>`（共享 MIR，唯一在产后端）；
  历史 legacy 行 = `cranelift-0.126/lower_object`（后端已退役，不再产生）。
- **适用面**：仅 linked 行（与 lld/sdk 内容轴同纪律）；check/syntax 与 product
  consumer 行不带该轴。
- **stale**：按行 profile 重算（开关逐 profile），值来源是
  `auto-lang-core` 对 `AC_NATIVE_BACKEND` 的镜像解析；`legacy` 解析为
  `retired:legacy`，与任何历史值不等 → 旧报告必 STALE；畸形值解析为
  `invalid:<raw>` 同样必 STALE。
- **收据**：`ace.m1.cert/1` 键集不动；backend 不进入收据，只以报告轴表达。
- 反控：worker 真实轴与父侧镜像逐字相等（`r1_backend_axis_mirror_matches_the_real_worker`，
  子进程 env 隔离）；legacy 请求被 worker/`ac-native` 显式拒绝。

## 治理层边界与已验证失效轴（PLAN-887）

- 写面 owner：runner/迁移/退役/floor/scripts/test-governance/** = 测试治理写面（887）；
  experimental/ac-*、auto-mir-lower、native worker/缓存/资源合同 = **S/N 域排除写入**；
  产品代码（entry/stdlib/trans/公开 SDK）= 只读。协议语义（avworker/1 case-serve、单一裁判、
  进程隔离、不缓存 PASS、deadline/资源合同）不变。
- 已验证失效轴（T-05 负结果）：session-less case-serve 强制 per-request 全新 store（nonce+
  删除）→ 内容缓存跨请求不可能命中；session-serve（828 增量协议）与 case-serve 包络不兼容，
  且 worker 域 887 排除 → 不借"会话复用"宣称提速，新计划先核此轴。
- 套件自包含：suite.include 与 case source 的 virtual path 拒绝 ..（admission），跨目录引用
  须以原 suite 独立运行或受管复制+身份记录实现。

### PLAN-863: console byte observations

Append-only recipe `ac-av/console-run-v1` selects profile 8 / `av.host-console/1`, source host-console, AVHC/1, entry `d_main`, exactly one run, no session or profile override. It persists/restores its artifact before executing under current grants. `avworker/1` envelope nonce/request hash/recipe/version/source binding and containment/deadline rules are unchanged; the worker observes, the parent decides PASS.

Only this probe recipe exposes `stdout`/`stderr`. Both explicit expectations are mandatory, compare `bytes`, with combined expected bytes ≤1 MiB. Success requires an integer `exit` oracle or explicit `av_channel: "unit"` (Unit cannot also assert exit). Language errors/traps require an explicit code and both byte oracles, so a trap after output checks the retained prefix. Other probe recipes reject declared console streams; unavailable observations never become empty streams.

The required run face is `console` schema `av.console-capture/1`: `mir_lowering`, `stdout_b64`, `stderr_b64`, `stdout_present`, `stderr_present`, `stdout_truncated`, `stderr_truncated`. Every field is required; unknown fields reject. Both streams must be present, neither truncated, base64 canonical, decoded per-stream and combined bytes ≤1 MiB, and MIR lowering identity must match. Parent independently compares stdout, stderr and exit/Unit/trap. Malformed/missing capture, foreign result faces, resource/cancelled/infra outcomes are ERROR, never a language-negative PASS. Raw resource-prefix evidence is retained as `console-observation.json`; valid observations also enter `probe-receipt.json`. No output is reconstructed from an oracle.

Report/stale axes bind worker SHA/version, protocol, recipe, core version, `mir-lowering=3`, `av-profile=8`, `av-source-profile=host-console`, `av-wire-codec=auto.abc.av-host-console/1`. Missing/drifted axes invalidate old PASS. Whole-suite PASS requires actual required rows; SKIP/NOT_RUN/zero-match are not verified coverage.

来源：[PLAN-863](../../plans/archive/863-av-console-host-import-and-print.md)。

## PLAN-888 测试专用批量与模块执行

原协议和父侧唯一裁判保持；新test-case-batch默认关闭，B逐项复用原编译/VM/准入并隔离状态，C只观察原D3 owned-record。进程先纳入Job再运行，准备至清理在原截止内，原guest/采样预算不变。详见[批量试点](batch-execution.md)。新模块runner消费唯一叶、原子身份和实际消费者闭包，原比较器为单源，见[模块选测](module-selection.md)。

## 2026-10-30 / PLAN-889：共享集合语义与原生资源消费

决策：复用同一source/HIR/共享MIR与候选协议，显式collections profile进入原生Cell/容器原语；
不用AV解释旁路或旧HIR→CLIF。冷态有界scaffold批处理，body编辑保真实producer身份；恢复不持久化trust。
备选整模块重算会超物理峰且冒充carry，扩大typed入口会混淆支持域，因此均不选。
后果：14原L/V格按原源关闭，全帧清理/预算/身份与失败原子性成为此profile合同；
旧AV source前端差额、宽typed入口/在线native帧保持各自边界。状态：accepted，见
[native collections](../auto-mir/native-collections-variants.md)。

## a2r-hir 认证通道（PLAN-892 并列 recipe）

`ac-a2r/hir-scalar-emit-v1`：probe/run seam 上的注册 recipe，预构建
`a2x-worker` 现场发射四文件集，父侧 golden 逐字节判决（日常零 rustc spawn）；
显式 certify 腿=实际＋手写独立 reference 双编译运行对闭式 oracle，严格证书
`ace.a2r-hir.cert/1`。与旧 `a2r/emit`/`ace.m1.cert/1` 并列互不授信。
见 [a2r-hir 认证通道](a2r-hir-certification.md)。

## 2026-10-31 / PLAN-901：测试feature拍平与批量观察常驻

七个语言测试feature（`test-trans`/`test-cases`/`test-ace`/`test-book`/
`test-aavm`/`test-http-e2e`/`test-case-batch`）退役：测试代码常驻编译可见，
运行由固定组、nextest 过滤与原 ignore/平台/环境/资源条件决定。trace 与
批处理观察常驻编译但 env 驱动——无显式执行（`AUTO_TEST_TRACE_DIR` 等）
不 spawn worker、不写报告，普通语言返回值/诊断不变；AAVM/AA2R 默认 ignore
+显式手动入口与重内存分组保持；跨 crate HTTP helper 常驻中立支撑
（经 auto-web re-export 保持原路径）。完整合同见
[test-feature-flattening](test-feature-flattening.md)。

## 计量关联与取消/失败完整性（PLAN-893）

- 成本归因只**关联**既有`process_capture`计量（discovery/inventory/run命令的wall/CPU/Job树
  峰值+逐测试exec_time），不复制meter、不新建Job；顺序命令wall合成观察窗、CPU分列不相加、
  峰值取max（嵌套Job共享树）。Case行工具准备以result内收据（build_seconds/exit）呈现并注明
  收据边界。
- 判定完整性：PASS主张须有真实执行产物（run.command存在、executed>0、Rust行每执行测试有
  terminal `ok`事件）；driver errors/零选择/伪PASS/取消超时（exit 124/meter error）一律红；
  外部杀driver遗留stale lease→后续运行fail-closed拒跑（WinError 183），人工恢复路径成立——
  该行为经刻意取消控制实证（PLAN-893 T-02）。


## 普通源码 CT 证据（PLAN-936，SD-05）

- 新 case 族 `ace-m1/source-cte-936/`（suite `ace-m1-source-cte-936`，组
  `case-ace-source-cte-936`）：复用既有 recipe（`ac-av/core-run-v1`、
  `ac/mir-native-scalar-v1`），caps 携带 `ac.source.cte.scalar/v1`（+
  instances/meta）。闭式 oracle 独立手算（42/43、bool 分支、meta 2→3、模板
  42、链 40）；调用点不改的体编辑翻值即依赖失效的反例。
- 新服务包 `ace-source-cte` 的单元/集成职责显式入 `ace-comptime` 固定组
  （被依赖编译≠测试已执行）；负例（未服务/环/deadline/禁调/篡改 envelope/
  恢复 R 窗/首坏修复）由该组包测试承载。旧 817 recipe/证书不受新面影响。

## 原记录/文本义务与当前证据关联（PLAN-941，候选）

组合器以旧冻结binding的`(namespace, id, target)`认领原required格位，逐项核对旧source-manifest、
当前source合同与严格wire reader产生的事实。R01..R06、T01/T02/T04的数值、T08具名越界及
R09/R10/T06/T07的结构化诊断分别对旧独立oracle重新判定；旧诊断分类到当前code的映射有限且显式。
AV data与native目标保各自recipe、profile/caps和父侧原始证据准入，通用退出码70不证明具名陷阱。
缺少类型化诊断权威保UNOBSERVED，源reader或父Case失败不能被关联变绿。

本关联只覆盖同源的14例、AC/AV共28格。新增M方法源、同ID的scenario、legacy-AVM/A2R及
资源/性能/生命周期义务仍各自认领；返回值相同、formal locator存在或登记成功均不构成执行证据。

CT机制的父侧证据可消费原服务公开的计数与执行边界日志：原sum42冷态真实AV、禁止AV下
同键命中，以及仅callee改变到sum43后的失效分别关联原始native终态、冻结输入与当前测试二进制。
测试二进制须按正式组的Cargo包/特性配置准备；单包配置的另一个产物不能替代。hit日志保留
冻结执行的steps/wall，它们不是本次新AV工作；本次工作由ct_execute变化和from_cache共同认定。
该窄面不证明managed恢复、meta/实例、资源或完整四端HIR机制；声明表仍只作发现，实际准入另列。

实例内的子CT只改callee时，原模板的InstanceRecord键可以保持不变；应分别核对模板声明身份与
常量回灌后的实际执行体版本。原pick链的内层flag和外层pick执行键均须失效，实际值40→42、
真实执行2→4、后续禁止AV下命中且执行数不增长，分别绑定同一服务的日志及成功发布的原ExecKey。
该证据独立认领instance-child-ct-version，不能代替meta能力或实例恢复义务。

## PLAN-938 增量（reviewed）

- 新固定组 `case-ace-ca-d-source-938`（all=false 显式档）；父 typed 判定与
  compiler current 指针分层（PLAN-773 F-03 语义）；late/非法 transport 不授
  认证；预登记 wrong-oracle 控制不折算 PASS。

## PLAN-939 增量（reviewed）

- 资源认证 TestCase 族：`ace-ca-d-resources-939`（62 例：S01..S08×方向+
  wrong-oracle 控制+2 CE 导出）；契约扩展 per-step `pe_exit`（多 query 流
  中间步的合法失败/差异传输出口）；bridge deck 语法增
  `profile text-record`/`caller <file.c>`（source 腿专用，typed/wide 拒）。
- 恢复/预算/采样纪律：恢复首编辑与首错→修复→再恢复、required 页三轴拒、
  超限取消零配额清理、修订 churn+lease+GC、6×30 五暖门 P90≤2000ms——
  详见 [ace-ca-d-bridge-certification.md](ace-ca-d-bridge-certification.md)。


## PLAN-1000 严格观察与正式签章入口（2026-11-11，SD-04 续）

- 组合器实测修复两处（合并文档 arch 行丢失修复：entry-gate 行在机制族重建时并集
  保留，机制行重叠优先；终合并门旗刷新：legacy_required_execution 随最终合并摘要
  更新而非逐链陈旧值）。组合器预留额调优：终合并摘要窗 2 MiB（同一 48 MiB 活数据
  帽内，非放宽）。
- 正式准入入口 `--acceptance`：全八域/证据完整/零 blocker 才 exit0 可签；preview
  恒不可签 exit1（T-13 在冻结证据上复证 V-14 隔离）；坏身份/格式 exit2 具名拒绝。
- 签前审计合同：独立进程于冻结提交重建键宇宙/新鲜组合/U 集/身份轴，凭据绑定冻结
  提交且不自引最终报告；唯一 review 行（ARCH-08/fresh-union-spec-review）的机械
  claim 来自 entry-gates 准入（30 项单组绿门），review 以独立凭据先行验证后才签。
  entry-gates 准入自带八条区分性反控（绿路径准入；翻转 outcome/失败事件/错组身份/
  stale 凭据/外来 entry/多组文档/缺一门的行各拒绝或降级——常驻治理测试）。
- 架构行守恒：49/49 ARCH 行终组合全 PASS（机制族 9 行 + entry-gates 40 行并集；
  delivered provider foreign/foreign-resource 按 resident ace-abi 证据准入）。

## PLAN-1005 架构行证据准入修正（2026-11-12，SD-01）

- 旧 entry-gates 准入（SD-04 续）存在两项已实证准入缺陷，本包有界修复：
  **F-1005-01**（组名+绿色布尔+`executed=0`/空结果即可供应架构行 PASS——1305 行波及面
  在修前代码复现）；**F-1005-02**（无独立审计凭据仍可供应
  `ARCH-08/fresh-union-spec-review` 的 PASS）。历史行为按当时合同记载保留，不回写
  1000 归档结果。
- **严格封套准入**（`ace_m1_arch_entry_gates` 重写）：entry 只从真实单组 fixed-test
  运行目录准入——listing 枚举、raw 终端事件经父解析器重 derive、discovery/run 命令
  收据逐 argv 复核、冻结窗口 source-pins、当前组配置、listing 二进制字节 pin、合同
  source pin；`executed=0`/空结果一律拒绝，绿布尔与失败终端/executed 矛盾一律拒绝。
- **逐行 recipe 准入**：49 行合同 `architecture-entry-recipes.json`（48 非 review 行，
  SHA `cab19420…`）冻结 row→entry→具名 case/test→required observation 映射；行 PASS
  仅当全部声明 entry 绿 **且** 全部具名事实在该 gate 内实际执行绿（事件名以 nextest
  listing 为权威：lib 目标 `pkg::lib$tests::fn`、独立 tests 目标 `pkg::target$fn`）。
  删任一具名事实→行 UNOBSERVED；红 gate→行 FAIL；假形态→拒绝。
- **review 行凭据专属授权边**：唯一 review 行只能由
  `ace_m1_review_evidence` 消费的签前审计凭据供应（绑定 frozen commit∈gate commits、
  签前候选 bindings+报告双 sha、行规则/registry pin、AC/V/SD 全覆盖、findings 全
  resolved、无最终报告自指字段）。缺凭据保持不可签（正式 exit1+具名 blocker）；机制
  族/phased section/entry 循环/测量回灌均不能供应该行（合并边界拒绝重叠冲突与注入）。
- phased 分域准入（`ace.m1.arch-entry-facts/1`）：每 entry 独立 48 MiB 预算跑完整
  严格链后输出紧凑 hash 钉定 facts 文档（1000 认证族分域同范式）。

- **facts=原件受检投影（PLAN-1013，SD-03）**：分域 facts 是严格验证后原始运行结果的
  紧凑投影，不是独立证据源——消费时必须沿 `{'facts', 'root'}` 受检引用回到 gate
  原件重导比对（旧自报 schema 拒绝，无 fallback）；篡改投影并重算局部 hash 不改变
  原件判决。进入 `arch_entry_gate_commits`/review 链的身份只来自原件核验+真实 commit
  对象+冻结锚成员门（raw/facts 共用），杜绝其他入口恢复同源绕过。
- **原件保留至签前复核**：分域/raw/机制/认证原件保留到正式准入、独立复审与主线
  tag_ready 复核完成；清理仅折叠摘要入计划，已删路径标历史；清理后如需重新准入，
  先重采复核，不以已删文件 hash 冒当前验证。
