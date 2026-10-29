# auto-test-case 架构（裁判来源与依赖约束）

> **PLAN-858 开发期周转口径**：ACE 有效样本按升序取第 `N-floor(N/10)` 项（从 1 开始）为 P90，硬门为 P90≤2000ms；正式样本数不减少，10 项最多 1 项超过 2000ms，等值通过。缺失、非法或错误结果不得计入绿；保留各路径原有零值准入、真实窗口、结构与资源断言。P95/P99 作为诊断与历史数据保留；架构基本成形后以实测另定长期目标。详见 [TestCase v2 周转口径](../auto-test-case/profile-v2-native.md)。


> 状态：active（A 阶段）。为什么这个 crate 存在、依赖方向为什么是单向。

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
  auto-lang 侧 `src/test_cases/`，空 feature `test-cases` 门控、无默认 feature
  可无窗口构建运行。被测 a2r 不得在运行时再生裁判（parser 快照固定）。

## 阶段观测通道（类型边界，非文本）

- AVM：`run_with_capture_and_path_staged`——旧 API 的 staged 孪生，engine 陷阱点
  （typed `VMError` 仍存活处）记录变体名经 thread-local 中继出线程；旧 API 签名
  与行为原样。
- 单文件 a2r 发射：parse/CTEE 错误保持类型化传播（Lexer/Syntax/Type/…）。
- 分类规则（`classify_staged`）：类型化 trap → run；Lexer/Syntax → parse；
  Type/Name/Comptime/MsgWithSource → type；MultipleErrors 全递归取首个叶子；
  其余（Io/panic/合成超时/无名 Msg）= Infrastructure → ERROR，**永不满足语言
  负例**。
- 已知边界（F-773-R2-01 债务）：a2r project 模式两入口仍把 parse 错误包成无名
  Msg → 归 Infrastructure（fail-safe：永不假 PASS，但 project-mode 强错误
  oracle 不可满足）。

## B 阶段执行拓扑（PLAN-788：native worker 与全量父侧判定）

- 单管线复用：auto-lang `test_cases::ac` 编排，`experimental/ac-case-worker`
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
