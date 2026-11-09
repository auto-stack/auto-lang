# TestCase：CA-D 标量桥认证（PLAN-937）

> **Status**: delivered（PLAN-937 T-09 首例）
> recipe：`abi/ca-d1-bridge-v1`（同 `abiworker/1` 信封，与
> `abi/interop-v1` 严格互斥——deck 语法/腿/期望互不相借）
> 首例 fixture：`crates/auto-lang/test/cases/ace-ca-d-scalar-937/`
> （固定组 `case-ace-ca-d-scalar-937`；包级组 `ace-foreign-contract`、
> `ace-a2c-abi`）

## 输入 profile

- deck 语法 `deck ace-ca-d1/1`（deny-unknown by construction，前导
  `#`/空行允许）：`leg source`（module+export+src 块）或
  `leg typed-self|typed-n2c|typed-c2n`（冻结 typed 声明，仅 deck+leg）。
- DATA 输入 `.atomdata`；步骤 query/edit-source/restart（父侧应用编辑，
  每请求携带当前 deck）；run/resident 类拒绝同 797。

## 腿与判定

- `source`：统一前门编译真实源→C 库 fileset→CRT 闭包编译→独立
  bridge_source_caller.c 真实调用。compute=真实 producer 次数（冷=facts
  +plan+逐函数片段+library；全命中=0）；**正 PE 退出码 = `pe-exit`
  diags**（797 退出码纪律：冻结 caller oracle 失败是判别 oracle，不是
  绿步）。
- typed 三腿：Cranelift↔cl.exe 对象真实链接执行，exit 0=全检查过。
- 负控：坏腿/坏 deck → parse/deck diags；body 编辑后冻结 oracle 失败 →
  run/pe-exit（值翻转可观察）。

## 协议身份与互斥

- 父 adapter（`test_cases/abi.rs`）双 recipe 分发；contract 注册
  `abi.ca-d1-bridge-v1` 绑定腿集 source/typed-self/typed-n2c/typed-c2n
  （`abi/interop-v1` 保持 a/b/c，互借即契约错误）。
- 信封/schema/nonce/request_hash/worker 版本/containment/行上限/原始
  行证据（abi-request-samples.jsonl）沿 797 父路径；stale 轴族同
  abi-worker 家族。
- 继承的协议族属性（记录在案）：空 steps 集空过为 797 以来族属性，
  非 937 回归（938+ 可加固）。


## PLAN-938 增量：source-foreign 认证（reviewed）

- recipe `abi/source-foreign-certify-v1` 沿原 `abiworker/1` envelope；deck
  `abi.source-import-deck/1` 严格 typed、report `abi.source-import-report/2`
  （旧版互拒）；独立 ADD（1 函数）与 Scalar（7 函数）人工审定契约 + 10+1 份
  ordinary-source deck；actual/reference 各对独立 parent exit oracle，期望仅在
  父侧；wrong-oracle 预登记 FAIL 控制。正式门 `cargo t case-ace-ca-d-source-938`
  gate_passed（8by/8cb 冻结并集）；暖 body-edit P90≤2000ms 常驻门。

## PLAN-939 增量：资源认证 recipe 与逐步出口（reviewed）

- 新 recipe `abi/resource-foreign-certify-v1`（deck
  `abi.resource-import-deck/1`、report `abi.resource-import-report/1`、
  逐 deck 受审 reference oracle、wrong-oracle 预登记 FAIL 控制）；固定组
  `case-ace-ca-d-resources-939`（62 例含 2 CE+控制位）。
- **per-step `pe_exit` 探针契约**：多 query 流的中间步可携带逐步 PE 与
  reference 出口（缺省保持 check 级 oracle）——首错→修复→再恢复流的
  中间态（修复态不同出口）在受审场景切换下诚实判定。
- worker 级一致性测试（ace-abi conformance `resource_recovery`/
  `resource`：凭证/代际 lease-GC/故障扫描/超限取消/指针原子/6×30 采样）。

## D937 终态 store seam 与总门映射（PLAN-941，SD-05）

- 桥接套件六检查（source-export/source-body-edit/typed-self/n2c/c2n/bad-leg）经 case
  parent 走 store seam（journal.db + bridge-scratch 产品）；组合侧按 parent Job 样本逐
  query 严格准入（completion 语义、probe oracle 三轴、cache/compute_calls/stages/meter），
  compute 发生时按 leg 要求真实编译产品在场（source=generated.c+a2x_c_runtime.h+
  bridge_source_caller.c；typed=self|n2c|c2n 道 .obj；parse 级拒绝合法无 scratch）。
- 交换帧/nonce/request_hash 不可观测即不冒认；typed 三腿与 source 腿不混同，937 不冒
  完整 FFI。（AC-08/09）
