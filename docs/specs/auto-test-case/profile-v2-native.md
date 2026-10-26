# TestCase Profile v2 native（autolang.case/2 / suite/2 / test-report/2）

> 状态：active（B 阶段交付，PLAN-788；扩展走后续计划）。
> 本文是 v2 native/syntax 通道的 current-state 规范；逐字段 schema 以
> `crates/auto-test-case/src/contract.rs` 解码器为准（解码器即合同）。
> v1 通道（avm/run、a2r/emit）在 v2 文档中行为不变，见
> [profile-v1.md](profile-v1.md)。

## 版本分派

- 三 schema 精确字符串分派（`autolang.case/2` / `suite/2` / `test-report/2`）；
  v1 接受/拒绝集字节不变；v1 塞 v2 字段仍按 unknown-field 定位拒绝；未知
  schema 字符串精确拒绝（不做 major/minor 解析）。
- v2 是 v1 严格超集：v1 字段语义不变，仅新增下文字段；未知字段=定位错误。

## 新增字段面（ac / syntax）

- source `syntax: "hir-atom"`（虚拟 `.atom`，原始 HIR Atom 文本输入）。
- `adapter: "ac"` × `action: check|build|run`；`adapter: "syntax"` ×
  `action: parse|edit`。
- ac options：`profile: core|scalar`（text/record 等未导入面 decode 即 ERROR）、
  `target`（恒 `x86_64-pc-windows-msvc`，其他值 decode ERROR）、`entry`（默认
  `main`，零参 i32 入口名；raw-HIR 按名回退）、`cache: fresh|reuse|read-only`、
  `budget: {build_ms?, run_ms?, total_ms?}`（申报值只能 ≤ 冻结硬 cap）、
  `capabilities: []`（如 `hir.test.trace.v1`）、`support_lib: "trace-marks"`
  （具名支持库，拒绝任意路径）。
- syntax options：`grammar: auto-core|interface-mini-v1`、`edits[]`
  （1..=64 步、每步 insert ≤64KiB、坐标相对当前修订）。
- expect 增：`exit`（仅 ac/run 成功完成——trap 70 是可声明退出码）、
  `perf: {total_ms_max?}`（声明即判，缺省仅记录）、stdout/stderr（v2 语义=
  原始 bytes 比较）、artifacts（ac 仅 `hir.atom` 金样；syntax 仅三投影
  reconstructed.at / outline.json / cst.canonical）。
- typed v2 阶段枚举：`parse|resolve|typecheck|bind|verify|capability|backend|
  link|run`；前端三阶段归属只来自 worker 观察边界；Infrastructure（IO/工具
  缺失/panic/非法退出/超时/超输出/响应失配/身份失配）永不满足语言负例。

## 资源冻结（硬 cap，不得事后放宽）

- 每请求 build ≤60000ms / run ≤5000ms / **total ≤70000ms（单一绝对墙钟：父侧
  管写入/读取/等待/回收全程，无宽限）**；stdout/stderr 各 ≤1MiB（父侧按**解码后
  字节数**强制，与是否声明流 oracle 无关）；IPC 单响应元数据 ≤4MiB；单产物
  ≤64MiB；单 check 实际产物总量 ≤128MiB；会话 ≤256 请求 / 总生命周期 ≤180s。
- stderr 超 cap = 受控 ERROR 且**即时杀树回收**（与响应/截止同循环，不等待
  worker 自然退出）；不读 stdin 的 worker 由墙钟及时终止（写入线程化，不受满管
  阻塞）。到达任一输出上限→立即终止全树并 ERROR，不返回截断成功。

## 部分产物与伪造边界

- 语言失败响应可携带**合法部分产物**（canonical `hir.atom` = 前端在失败边界前
  的真实输出，manifest 照常父侧核验/持久化）；`obj`/`exe` 是成功产物，出现在
  失败响应中=伪造，一律拒绝（负向保留 typed backend oracle）。
- 必需产物集仅对无诊断的成功响应要求（build=obj+exe；ac/run=exe；syntax=
  reconstructed.at）。

## cache 参与（783 统一 store）

- `reuse`：get→compute→publish（HIR 命中重 verify；obj 命中跳前端/verify/lower
  但真 link/run）；`fresh`：跳过全部 get、真实重算并按安全合同 publish；
  `read-only`：缺根=infra ERROR；miss 真实重算执行但不发布。所有模式每次重读
  oracle 重新比较，**绝不缓存 PASS**。
- 非参与组合（ac/check、hir-atom 输入）声明 fresh/read-only = admission ERROR。
- 缓存 key 入 compiler_fp（worker exe 内容哈希）+ capabilities + support_lib。

## 身份轴（stale 语义）

- 每 native 行按**实际消费路径**绑定可重算身份集合：`worker-exe-sha256` +
  `ac-worker-protocol` 恒在；`lld-sha256` 与 `sdk-um-sha256`（kernel32.lib
  内容哈希，非路径）仅真实链接过的 ac/build|run 行（产物 obj/exe 或含 run 观
  察）；产品消费 run 与 check/syntax 行只有恒在两轴；声明 support_lib 时
  `support-lib-*-sha256`（库文件内容哈希）入轴。
- stale 按同一适用集合比较：缺轴/多轴/值漂移均 STALE，未知/缺失不静默相等；
  报告 artifact 序列化键为 `path`。

## syntax 通道（787 合入 runtime）

- parse = 真全量解析 + typed 诊断（code 按真实 lex/tree 来源分；绝对 span）+
  最终修订三投影；edit = 有序单编辑步（顺序语义），worker 每步证明
  fresh↔incremental 等价（分歧=syntax.inconsistent，infra 永不发布）。
- 语法 negative 是语法诊断，不获语义 Checked；不支持 profile/观察=admission
  ERROR，不被 SKIP 掩盖。

## 验收与门禁

- `scripts/verify-autodown-ac-788.ps1`：-Scope acceptance（七族聚合：合同/
  执行器/worker/runner/783/v1 回归/R3 集成+周转硬门）；-Scope selftest（解析
  器单元正负控）；-Scope closeout（merge-owned：裸 t/tv/tt 按 nextest 真实选择
  语义 + 档位必验集合绑定 + worker/probe/ac-syntax 独立完整套件）。
- 语料 `crates/auto-lang/test/cases/ac-native/`（42 语义 case + legacy-map.tsv
  双向对账）；非 Windows 平台 required-native 不满足（不假绿）。

> 来源：docs/plans/788-autodown-ac-native-turnaround.md（B 阶段）；契约冻结与
> 十轮复审修订见其 attachments/788/contract-decision.md §1–§17。

## probe 期望面（PLAN-796）

- `expect.probe`：逐 query 步的粗粒度 cache 词表（published-*→published、
  hit/diags）、compute_calls、stages_include、bindings oracle（边车文件，
  身份=内容哈希）。
- completion 准入与既有通道一致：success=ok 零诊断；error 需 stage/code/
  message 匹配；协议/资源/infra 失败=ERROR，永不满足语言负例。
- 验收 raw 准入（V-07 口径）：周转报告逐样本数值准入（字段存在/非 null/
  数值/有限/正值，违规即 FAIL 不默认 0ms）再重算百分位；完整编辑请求窗口
  （请求准备→worker→父侧准入）P95<1000ms@1MiB（gen796-1mi/v1），连续分段
  铺满可对账；六类变异控制入 verify 脚本 selftest。

## 外部资产身份合同导航（PLAN-852）

v2 case/suite/report 解码器合同**不变**（接受/拒绝集保持，未知字段仍拒）。
ace-m1 的收据（`ace.m1.cert/1`）、分母登记（`ace.m1.binding/1`）与 wrapper 身份是
**语料侧独立版本化资产**，不进本解码器；其严格验证由
`verify-ace-m1-852.ps1` 与 `ace-certify` 承载，见
[a2r-golden-certification.md](a2r-golden-certification.md)。
