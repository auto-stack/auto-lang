# PLAN-788 拟沉淀规范增量（T-08 冻结稿，merge 时经 SD 审后入 canonical Specs）

依据：contract-decision.md（§1–§10）+ 实际交付代码（提交链 7e6d64fcb→720df159c→
ff17a8ce1→c898a041a→22132ee5c→T-08）与全量定向反控。review/merge 可修订措辞，
不得扩大支持面或放宽预算。

## SD-01 add `docs/specs/auto-test-case/profile-v2-native.md`

v2 严格超集合同：
- 三 schema：`autolang.case/2`、`autolang.suite/2`、`autolang.test-report/2`；精确字符串
  分派，v1 接受/拒绝集字节不变；未知字段=定位错误（永不 parse-then-ignore）。
- 新字段面：source syntax `hir-atom`（.atom 原始 HIR）；adapter `ac`（check/build/run）
  与 `syntax`（parse/edit）；ac options（profile core|scalar、target 恒
  x86_64-pc-windows-msvc、entry、cache fresh|reuse|read-only、budget ≤ 冻结硬 cap
  build 60s/run 5s/total 70s）；expect 增 `exit`（仅 ac/run 成功完成——trap 70 是可声
  明退出码）与 `perf.total_ms_max`（记录默认、声明即判）；ac 字节金样仅 `hir.atom`、
  syntax 仅三投影（reconstructed.at/outline.json/cst.canonical）；typed v2 阶段
  parse|resolve|typecheck|bind|verify|capability|backend|link|run（avm 仍 v1 三阶段）。
- 非参与组合（ac/check、hir-atom 输入）声明 cache fresh/read-only = admission ERROR；
  默认 reuse 不承诺参与。
- 资源冻结：stdout/stderr 各 1MiB、IPC 4MiB、单产物 64MiB、check 总量 128MiB、会话
  256 请求/180s（含空闲，看守强制）。
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
（ERROR 不吞）。

## SD-04 modify `docs/specs/auto-test-case/project.md`

已实现支持表：v2 native（ac check/build/run，core/scalar，
x86_64-pc-windows-msvc）+ syntax（auto-core/interface-mini-v1，parse/edit）；
预算/身份轴（worker-exe-sha256+协议常量）/stale native 行为/语料
test/cases/ac-native/（32 case + legacy-map）；平台边界：非 Windows required-native
不满足（不假绿）。

## SD-05 modify `docs/specs/auto-ac/project.md`

AutoDown 入口导入范围：引用式 fixture（payload 生成期内嵌），不迁移语料、不退役
旧门禁；text/record 未导入（guard 仍为旧门禁）；trace-marks 变体（p15 评估序）
待 runtime-lib 批次。

## SD-06 modify `docs/specs/auto-ac/artifact-probe.md`

真实消费者接线：`DriverCacheMode{Reuse,Fresh,ReadOnly}`+`vertical_observed/
vertical_and_run_observed`（同一参数化管线；Fresh 跳结果读、ReadOnly 跳全部
put+roots；实测阶段 span parse/resolve/typecheck/emit/bind/verify/lower/link/
run/cache-read）；`VerticalCfg.compiler_fp` 进 A07/A09 key（worker exe 内容哈希，
None=旧 783 key）；store/GC/codec/schema 合同不变；非生产 cache。
