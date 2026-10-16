# PLAN-788 拟沉淀规范增量（T-08 冻结稿，R3 修订版；merge 时经 SD 审后入 canonical Specs）

依据：contract-decision.md（§1–§11）+ 实际交付代码（提交链 7e6d64fcb→720df159c→
ff17a8ce1→c898a041a→22132ee5c→230825e78→3393707e6→4fcc4bfd9→R3 修复链）与全量定向
反控。review/merge 可修订措辞，不得扩大支持面或放宽预算。

## SD-01 add `docs/specs/auto-test-case/profile-v2-native.md`

v2 严格超集合同：
- 三 schema：`autolang.case/2`、`autolang.suite/2`、`autolang.test-report/2`；精确字符串
  分派，v1 接受/拒绝集字节不变；未知字段=定位错误（永不 parse-then-ignore）。
- 新字段面：source syntax `hir-atom`（.atom 原始 HIR）；adapter `ac`（check/build/run）
  与 `syntax`（parse/edit）；ac options（profile core|scalar、target 恒
  x86_64-pc-windows-msvc、entry、cache fresh|reuse|read-only、capabilities 声明、
  support_lib 具名库 "trace-marks"、budget ≤ 冻结硬 cap build 60s/run 5s/total 70s）；
  expect 增 `exit`（仅 ac/run 成功完成——trap 70 是可声明退出码）与
  `perf.total_ms_max`（记录默认、声明即判）；ac 字节金样仅 `hir.atom`、syntax 仅三投影
  （reconstructed.at/outline.json/cst.canonical）；typed v2 阶段
  parse|resolve|typecheck|bind|verify|capability|backend|link|run（avm 仍 v1 三阶段）。
- 非参与组合（ac/check、hir-atom 输入）声明 cache fresh/read-only = admission ERROR；
  默认 reuse 不承诺参与。
- 资源冻结：stdout/stderr 各 1MiB、IPC 4MiB、单产物 64MiB、check 总量 128MiB、会话
  256 请求/180s（含空闲，看守强制）。
- R3 边界补遗：流 cap 在父侧按**解码后字节数**强制（1MiB），与是否声明流 oracle 无关；
  worker stderr 超 1MiB = 受控 ERROR（不截断成功）；request 写入（stdin）与响应等待/
  回收同处一个绝对 total 截止（不读 stdin 的 worker 被及时杀树，不受满管阻塞）；语言
  失败响应可携带**合法部分产物**（canonical hir.atom——前端在失败边界前的真实输出），
  obj/exe 是成功产物、出现在失败响应中一律拒绝。
- 接受 AC-01–06。

## SD-02 modify `docs/specs/auto-test-case/profile-v1.md`

v1 合同不变；补 v2 导航（case/2 解锁的字段面）与版本拒绝边界
（`wrong_schema` 仍精确拒绝未知版本，case/2 现为受支持分派）。

## SD-03 modify `docs/specs/auto-test-case/architecture.md`

B 阶段执行拓扑：auto-lang `test_cases::ac` 编排+全量判定（比较永在父侧，worker
PASS 不受信）；`experimental/ac-case-worker` 独立 workspace 持有实验编译栈（Cranelift
红线：auto-lang 零静态依赖），serde JSON-lines 有界会话 IPC（acworker/1，
deny_unknown_fields，nonce/hash 回显+父侧 schema/ok-error 互斥/形状 admission）；
单管线复用（ac-core 前端/verify/lower、783 store、787 runtime——无第二执行器）；
产物 DAG：manifest 分配前全校验（固定集合/唯一/scratch 内/声明长度/64·128MiB/按
action 必需），消费=manifest 核验后运行（不重编译）；消费者 verdict 状态传播
（ERROR 不吞）。R3 补遗：父侧 I/O 生命周期统一——stdin 写入线程化并纳入请求墙钟、
stderr 排空带 cap 信号、stdout 读取带界；判定入口（validate_response /
manifest 准入+持久化 / 流比较）为可复用公共函数，周转 harness 与 runner 使用同一
入口。

## SD-04 modify `docs/specs/auto-test-case/project.md`

已实现支持表：v2 native（ac check/build/run，core/scalar，
x86_64-pc-windows-msvc）+ syntax（auto-core/interface-mini-v1，parse/edit）；预算/身份
轴与 stale 语义（R3 冻结）：**每 native 行按实际消费路径绑定可重算身份集合**——
worker-exe-sha256+ac-worker-protocol 恒在；lld-sha256 与 sdk-um-sha256（SDK
kernel32.lib 内容哈希，非路径）仅出现在真实链接过的 ac/build|run 行（产出 obj/exe 或
含 run 观察）；产品消费 run 与 check/syntax 行只有恒在两轴；声明 support_lib 时
support-lib-trace-marks-sha256（库文件内容哈希）入轴。stale 按同一适用集合比较：
缺轴/多轴/值漂移均 STALE，未知不静默相等；语料 crates/auto-lang/test/cases/ac-native/
（42 case：core 8/scalar 3/源码负例 8/HIR 负例 7——引用 2、结构 2、类型 2/HIR 正例 2/
红控 2/syntax 3 + 动作矩阵等，legacy-map.tsv 43 行双向对账）；平台边界：非 Windows
required-native 不满足（不假绿）。

## SD-05 modify `docs/specs/auto-ac/project.md`

AutoDown 入口导入范围：引用式 fixture（payload 生成期内嵌），不迁移语料、不退役
旧门禁；text/record 未导入（guard 仍为旧门禁）；p15 评估序与 p16 trap 前可观察输出
已经 **trace-marks 具名支持库**（capabilities `hir.test.trace.v1` + support_lib
"trace-marks"，worker 构建/链接 PLAN-741 marks runtime）落地（R2 起）；V-06 字节保真
生产者经**真实 worker 产品消费路径**覆盖（NUL/CRLF 为可比对 oracle 字节、非法 UTF-8
经协议链路逐字节保真——core/scalar 语言本身无可表达非 ASCII 输出的内建，语言内生产
者随 text 批次导入）。

## SD-06 modify `docs/specs/auto-ac/artifact-probe.md`

真实消费者接线：`DriverCacheMode{Reuse,Fresh,ReadOnly}`+`vertical_observed/
vertical_and_run_observed`（同一参数化管线；Fresh 跳结果读、ReadOnly 跳全部
put+roots；实测阶段 span parse/resolve/typecheck/emit/bind/verify/lower/link/
run/cache-read）；`VerticalCfg.compiler_fp` 进 A07/A09 key（worker exe 内容哈希，
None=旧 783 key）；`capabilities`+`support_libs` 同入 VerticalCfg 与缓存 key；新增
一次性 GC 工具 bin `store-gc`（与 783 套件同一 ManagedStore+run_gc 入口，无新 GC
语义——并发场景作为独立进程运行）；store/GC/codec/schema 合同不变；非生产 cache。
