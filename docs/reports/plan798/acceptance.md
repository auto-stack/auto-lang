# PLAN-798 验收/覆盖/并行审计（T-08，work-owned 收口）

## 2026-10-20当前收口（优先于下列历史）

实现修复冻结d1357870a，随后仅规范/索引说明提交。R15六项独立反例重放关闭；R16发现的多单元/manifest lease漏接已修复并补永久反例。正式30样本门通过：edit P95=867.8ms、sig=385.8ms；六场景同轮180样本正确性及双线内存通过（A=211615744B，B残差=6193152B），但body-edit P95=1026.9766ms，exit5，V-11/AC-06仍开放，不能授最终review/合入。全部raw、summary、正式日志与失败收据见主检出`docs/plans/attachments/798/closeout-r6-20261020/`。没有删除失败样本、提高门槛或用正式门替代warm门。798完成后优先828，其余ACE扩展须保留完整增量机制。

下列旧状态/旧数量/旧版本为当时work记录；现行wire/schema/ABCsemantics=2，MIR/AVsemantics=1、AVEN=2、A09=/v2，具体以冻结实现合同及拟Spec为准。

## r6 当前验收边界（2026-10-20，优先于历史摘要）

代码基线 `8e78502ba`，受测证据提交 `dbae2e634`；本次仅文档收口，实际冻结提交以随后commit为准。R15六项修复仍须最终独立review，不以work自检代替。原V/K14和安全/资源/正式门不削弱，merge-owned整档保持pending。

| 阶段 | 本计划实际实现 | 尚未实现 / 后续owner |
|---|---|---|
| source/HIR | 每请求整模块前端/Atom render-bind/verify | 函数级语法、语义、HIR驻留与依赖查询：828 |
| MIR | 缺一个函数单元仍whole-module lower/verify/split | lower_function及未改Checked MIR对象复用：828 |
| ABC缓存 | 函数产物独立身份/持久化/恢复；body编辑1 emit、114 hit | 热态内存命中不读盘、不重解码其他函数：828 |
| 装载 | 命中单元重decode/verify，再完整closure编解码/准入 | 常驻已验证函数代码、稳定绑定、局部受检装载：828 |
| 安装/寿命 | store root/required-edge/lease/GC；有限native代际准入 | 在线函数安装、新旧snapshot可见性/活代码保活回收：828 |
| 周转 | 已实现warm Driver链路与跨进程formal技术门分别记录 | 稳定会话完整函数增量结构+P95双门：828 |

798完成不表示全链路函数增量或真实在线替换完成。warm bench 874.5993ms不是单函数编译成本；sig51.1369ms是前端参数不匹配的诊断早退。最近formal edit980.1/sig1200.4ms，含逐请求启动/store-open/父侧校验，不能用于判定用户稳定会话目标失败；它在798原合同中仍是独立开放技术门，不能以bench替代或删失败样本。

SD-08拟沉淀到 `docs/specs/auto-av/cache-and-conformance.md`：逐阶段说明上表，禁止“cache hit/1 emit/时间绿”冒领完整增量。原SD-02历史v1字样以r5 wire/schema/semantics /2、kind/module-unit规则为准，保留公开输入/恢复重验；实际代码没有由本次说明改变。

798主计划部分早期r1/r2/r3/r6附件路径在主检出缺失，列于 `docs/plans/attachments/798/function-incremental-audit-20261020/preflight-result.json.txt`。这些是不可复用的历史证据，不伪造文件或据其授pass；最终验收使用当前可重放测试/当前来源与R15以后在库证据。

T-08本次范围：更新acceptance/coverage/contract-decision/turnaround的实际交付声明；不增加完整增量实现。T-07正式技术门仍待受测冻结提交实证，最终review/merge后才能声明本计划完成。


2026-10-19。代码基线：`plan-798-dev`（本 worktree，T-06..T-08 修复+续作后）；
依赖：master `13367d127`（796 已归档合入，交付 b76eca8d2；788 已归档合入）。
本文件是 work 自检产物，最终验收归 review；canonical Spec 沉淀归 merge。

## V-01..V-13 覆盖矩阵（实际入口）

| V | 断言族 | 实际入口（测试/脚本） | 状态 |
|---|---|---|---|
| V-01 | 私有阶段信任 | mir_tier 7 + abc_semantics（only-verify 句柄/篡改内层必拒） | 绿 |
| V-02 | 类型/初始化/CFG | mir_admission 变异矩阵+哈希证明+半初始化合流+循环不外溢 | 绿 |
| V-03 | 数值/短路 trap | av_semantics trap 全矩阵+未走 RHS 潜伏+native 对拍 | 绿 |
| V-04 | 参数/控制/递归 | p-binop-args/p-gcd-euclid 降级+fib(15)+帧预算两侧 | 绿 |
| V-05 | wire/恢复 | abc_codec 截断/尾随/校验和/未知版本-opcode+envelope 拒绝 | 绿 |
| V-06 | 真缓存 | cache_tier 命中跳产仍执行+跨进程 worker 恢复 | 绿 |
| V-07 | 依赖/配置 | body-only 单元重键/签名变更全 caller 重键/跨工程共享 | 绿 |
| V-08 | 错身份/并发/回收 | envelope 错 producer/schema/stage+Conflict 保旧+lease/GC | 绿 |
| V-09 | 规模/内存 | 4096 链+33.5MiB 冷产物+512 真实 churn+分区反控三件 | 绿 |
| V-10 | 有界执行 | fuel/取消/1024 帧两侧/trace 预算 | 绿 |
| V-11 | 亚秒 | 六场景30样本bench与正式跨进程门分列；bench不能代formal | formal待新冻结实证，非绿 |
| V-12 | AutoDown 真实执行 | corpus 6 checks+负 oracle+decode 负例，统一 runner | 绿 |
| V-13 | 绑定/边界 | binding_tier 4（同名异 owner/重启稳定/schema/tamper） | 绿 |

## 正式 AutoDown（T-07 交付面）

- recipe `ac-av/core-run-v1` 注册（REGISTERED_PROBE_RECIPES；配方专属步骤词表
  run/edit-source、专属 `av` 绑定、probe 通道 exit 观察、run 完成态）。
- 共享侧 `crates/auto-lang/src/test_cases/ac_av.rs`：Job 收容+resume、
  envelope/nonce/request_hash/recipe/双版本镜像准入、stale 按配方族路由
  （av-worker 四轴）、父侧判定、probe-receipt.json。
- worker：`case-serve`（会话步骤+持续 store+逐步墙钟）与 `load-revision`
  （冻结负载单一来源）；从不读 expected。
- 用例：`crates/auto-lang/test/cases/ac-av/`（suite+5 case 文件 6 checks，
  成功/trap/诊断/能力门/资源负例全 oracle 形态）。
- 兼容：probe 接缝 17 腿绿（decode 负例/v1 pilot 原样/atom 准入/stale 分族/
  收容/类型负例）；contract_v2 20 测绿。**sema corpus 腿阻断在案**：其
  bindings oracle JSON 未被 master 跟踪（根 `.gitignore `*.json``）——**r7 已闭合**：
  Plan 813 已归档合入（master 45fcd09c9/602a13527），两个 oracle JSON 已随本分支
  同步入库并实际重放（probe_adapter 18/18）。

## SD 提案（merge 沉淀输入；review 复核后定稿）

| SD | 目标 | 要点 |
|---|---|---|
| SD-01 | auto-mir/project+core-i32 | 块/槽/CFG/初始化合同+CheckedMir 私有凭证+工件族预算（r2） |
| SD-02 | auto-av/project+abc-core-i32 | wire v1 布局/验证次序/LoadedModule/unsupported 面 |
| SD-03 | auto-av/cache-and-conformance | A08/A09 producer 身份、单元/manifest/sidecar 分轨、恢复重验、计数语义 |
| SD-04 | auto-ac/artifact-contract | A08/A09 的 798 实现范围追加（carrier=LayoutData） |
| SD-05 | auto-hir/stage-contract | HIR→新 MIR 消费前门+128 前门递归风险表 |
| SD-06 | auto-test-case/architecture | ac-av/core-run-v1 recipe 支持表（步骤词表/绑定/exit 通道/run 完成态/stale 分族） |
| SD-07 | auto-av/cache-and-conformance | AC/AV/AJ 合同边界与 native 产物引用准入（T-08 有限代际模型） |

**r4 更正与实装（F798-R3-09）**：T-07 收据曾把 SD-07 的"有限代际/native 产物
准入"写成已完成——当时只有非目标声明，无模型/反控（复审判虚假主张）。本轮
实装 `probe/src/native_admission.rs`：`NativeArtifactRef`（generation/profile/
capabilities/format/target/abi/backend/flags/recipe + 内容绑定 blake3）+
`admit_native_ref`（未知 generation/profile/capability/target/abi/backend/
recipe、非重定位格式、哈希/长度失配、缺 manifest 全拒）+ COFF 结构校验
（AMD64/有节有符号表/拒绝 MZ-PE 进程映像）。**正例 = landed Cranelift 真实
COFF**（`mint_and_admit_lowered`）；真 PE 拒绝用 `native_reference` 产出的
真实 module.exe。反控 `native_admission_tier` 5 测绿。仍不做（非目标不变）：
AJ、DLL 加载、热替换、发布器、新 native emitter。

touched_goals=[] 说明：798 全程为 experimental/ 探针+测试接缝扩展，不改产品
目标行为；目标级 goal 映射由 review/merge 判定。

## 并行审计

- 写面：`experimental/ac-av-probe/**`、`crates/auto-lang/test/cases/ac-av/**`、
  `scripts/verify-ac-av-798.ps1`、`docs/reports/plan798/**`、
  `test/cases/ac-av→crates/auto-lang/test/cases/ac-av` 迁移（git mv）。
- 共享接缝最小扩展（按 796 接缝预留的追加点）：contract.rs（注册表+ProbeStep::Run
  +ProbeAvSpec+av 已知字段+exit/run-stage probe 放开）、suite.rs（probe_av 字段
  透传）、mod.rs（按配方分派）、sema.rs（containment pub(crate)+防御 arm）、
  stale.rs（配方族路由）。未改 decoder 协议/共享 IPC。
- 依赖只读：ac-core/ac-artifact-probe/ac-sema-contract 均未修改。
- 组内 auto-down 兄弟树：`D:/autostack/.wt/lang-798/auto-down`（detached @
  master c39ac7f，根 workspace 路径依赖所需，无本计划改动）。
- 交叉：Plan 813 已归档合入，其 oracle JSON 已入库；本计划分支同步后兼容 18/18。

## merge-owned pending

冻结提交统一门禁（根档按最终 diff）、native/796/788 实际触面兼容整档、tf 到期
批量、Spec/ledger 沉淀与归档清理——最终 review pass 后由 merge 执行。

## r5 增补（R12/R13 修复轮 + readiness，2026-10-20；基线 `a7850407e`）

V-01..13 矩阵（上文）仍绿；本节补 V-14/K14 完整矩阵、双线内存门与 r5 合同增量。

### V-14 与 K14-01..06 完整矩阵（实际入口）

| K | 子case（§5.8 输入/阶段） | 实际入口与证据 | 状态 |
|---|---|---|---|
| K14-01 | 完整单函数 module：false 分支直接自调用、else 42；多函数合法对照 | `abc_kind_survives_roundtrip`（MIR is_unit=false → wire v2 FLAG 翻转排除 → 公开 loader 执行 Success{42}；driver fresh 42）；多函数对照 = `av_semantics_` p-binop-args/p-gcd-euclid；**正式 runner 腿 = 语料 `av.core.kind-selfcall-module`** | 绿 |
| K14-02 | 零参/带参 unit、module 合法 entry：unit 可验证/缓存不可执行；module 执行 | `abc_unit_credential_split_at_public_loader`（零参 unit：Infra "unit kind" dispatch=0；同载荷翻位 module：执行 7）；core `av_unit_kind_param_credential_not_executable`（带参 unit `execute_entry` Infra）；`av_unit_kind_zero_param_credential_not_executable`（零参 unit Infra + module 正控 7） | 绿 |
| K14-03 | module entry 带参/错返回/越界、未闭合 imports → 准入拒、dispatch=0 | 带参 module → `mir.entry-sig`（mir_verify:363 完整模块零参/i32 合同；R12 探针 parameter-complete-module-negative + core lib entry-sig 断言）；未闭合 imports → 任何 Call→Import 即 executable=false（`av_unit_with_import_call_is_not_executable`，dispatch=0） | 绿 |
| K14-04 | 可信 kind 冲突受控拒；**仅翻位且完整规则满足的正控按完整语义判定** | 冲突轴 = 未知 flag 位 decode 拒（`abc_codec_rejects_unknown_version_opcode_and_illegal_bool`）+ 手工版本/profile 复核（verify_abc Identity）；翻位正控 = `av_unit_kind_zero_param_credential_not_executable` 正控臂（is_unit=false 同体执行 7）+ kindless 测试的 v2 翻位形态（R13 复审验证：不把 v2 合法翻位当旧版本错误） | 绿 |
| K14-05 | 手工 DecodedAbc/raw bytes/driver fresh/hit/新 worker restart 每入口重验 | manual+raw+fresh = `abc_kind_survives_roundtrip`；hit/restart/跨工程 = `av_cache_hit_lifecycle`（命中跳产仍执行、逐次重比较）；新进程恢复 = `av_cache_worker_restore_new_process`（manifest→artifact id→重验→执行）；**正式 runner 腿 = `av.core.kind-selfcall-module`（两 run 步：fresh 后命中路径仍重验执行 42，末步判定）** | 绿 |
| K14-06 | 无 kind/歧义旧缓存：raw 受控拒、受管 miss 重建/拒绝、不恢复 trusted | raw v1 → `abc.decode.version`（`av_cache_kindless_v1_wire_isolated`：真实 /1 伪造=版本字段+FNV 重算）；受管当前键陈旧载荷 → 确定性 miss 重建（hit0/miss1/emit1、重建写 Conflict 容忍旧行保留、双请求确定性）；旧键结构性隔离 = A09/A08 键 abc-codec/abc-sem/av-sem 代际轴 + envelope ABC_SCHEMA /2 | 绿 |

V-14 行（并入上文 V 矩阵口径）：unit/完整 module/entry/imports/旧缓存在
公开 verifier/loader、driver fresh/hit、store 新 worker、正式 AutoDown 各入口
保真——上表六行为其展开。

### r5 双线内存门（§5.9，用户裁定方案 C；contract-decision r7）

- A 线（ac-core owner）前端编译瞬态 ≤224MiB、B 线（798）探针残差 ≤128MiB，
  均为 `bench::validate_and_gate` **硬门**；缺归因测量整峰保守计入 B 线。
- 永久测试 `av_turnaround_two_line_memory_gate` 四态（A 越/B 越/双过/保守）。
- R13 复审独立证据（六场景×1 冒烟）：A=211,423,232B(201.6MiB) ✓、
  B=3,731,456B(3.6MiB) ✓、`hard_gate_failures=[]` exit 0。
- 30 样本形态：见下方正式门记录（本轮 r5 增补执行）。

### r4→r5 合同增量（历史矩阵行更新）

- wire **v2**（ABC_VERSION 2 / ABC_SCHEMA "/2" / ABC_SEMANTICS 2 / A09 v2；
  FLAG_IS_UNIT 类别位）：SD-02 行的 "wire v1 布局" 以 contract-decision §4
  （v2，r6/r7 修订记录）为准。
- unit→闭包执行凭证（类别+imports 闭合，F798-R12-02）：SD-02 的
  LoadedModule 行补"unit 凭证不可执行、闭包装配为唯一转换"。
- 前门计深 = 真实结合最长构造路径（R12/R13：shunting-yard 求值器 + 一元前缀
  绑定完整调用表达式）：SD-05 的前门行以此为准；判别测试
  `av_resources_tail_operand_deep_call_not_concatenated`（71/131）与
  `av_resources_prefix_unary_binds_to_full_call`（131/132 七态）。
- 语料 6→**7 checks**（新增 `av.core.kind-selfcall-module`）。

### r5 正式 30 样本验收记录（2026-10-20，@`a7850407e` + 语料增补）

- **T-05 六场景×30 硬门 PASS**（release `bench-turnaround --samples 30
  --gate-ms 1000`，exit 0 全门通过）：edit P50/P95/P99=827.3/**888.8**/910.0ms、
  sig P95=41.4ms、180/180 样本成功、受管分区无越界、**双线内存门 30 样本
  通过**（A=211 656 704B=201.8MiB≤224 ✓/B=3 985 408B≈3.8MiB≤128 ✓，另见
  首跑显式数字）。raw：`turnaround-r5-30sample-pass.jsonl`（180 行）与
  `turnaround-r5-30sample.jsonl`（含显式内存字段的首跑）。
- **T-07 正式 AutoDown 门：环境受阻 pending**——当日多次 edit P95
  978.9(过)/1001/1018.9/1028/1041/1076/1157ms 于 1000ms 边界抖动；本机共享
  负载周期性尖峰（当日观测 8%↔93% 波动，多会话并行构建）；**A/B 对照：基线
  `bf7e44f8e` 同时刻 1041.2ms 同样 FAIL**（无码回归）；同语义 30 样本 bench
  于当前代码 PASS（888.8ms）。其余正式腿全绿：语料 **7 checks**（新增
  `av.core.kind-selfcall-module` fresh+hit 两 run 42）、负 oracle、decode
  负例、worker 故障注入 Infra、probe_adapter 兼容 18/18。
- 解锁动作：静置机单测重放
  `cargo test -p auto-lang --lib --features test-cases
  tests::ac_av_probe_tests::ac_av_probe_formal_turnaround_gates -- --exact
  --test-threads=1`，pass 后 T-07/T-08 复勾、execution_done 送审。

### R14 修复（F798-R14-01 同轮证据 + 正式门解锁，2026-10-20 静置窗口）

- **同轮 30 样本 bench（受控、单一运行、完整保留）**：
  `av_probe_worker bench-turnaround --samples 30 --gate-ms 1000 --out
  turnaround-r5b-30sample-rows.jsonl`（stdout 同轮捕获为
  `turnaround-r5b-30sample-summary.jsonl`）——**exit 0、`hard_gate_failures=[]`**；
  edit P50/P95/P99 = 401.1/**414.7**/419.6 ms、sig P95 = 20.7 ms、180/180；
  **A 线 211 058 688B = 201.3MiB ≤224 ✓（met=true）、B 线残差
  4 354 048B ≈ 4.2MiB ≤128 ✓（met=true）**。同轮 summary 已写入
  `turnaround-summary.json`（含命令/退出码/提交/rows 绑定），三件 sha-256
  见 `r5b-evidence-hashes.txt`。替代并更正此前"两轮拼接"的引用
  （旧 `turnaround-r5-30sample*.jsonl` 保留为历史，其摘要数字不再引用）。
- **正式 AutoDown 周转门（V-11 formal）PASS×2（当前代码 `c8c598f0a`）**：
  静置窗口连续两次 30 样本通过——edit P50/P95/P99 = 919.4/**961.7**/986.1 ms
  （sig 598.9）与 903.5/**948.9**/996.9 ms（sig 599.6），均 <1000ms，test ok；
  第二次带 tee 落盘 `autodown-formal-r5.log`。逐样本分解：窗口 ≈ step
  ~410-430ms（含前端 ~285-297ms）+ spawn/IPC/admit ~240-300ms；白天劣化
  为共享负载下 spawn 侧膨胀（基线 A/B 同红在案），静置后回落。
- T-05 同轮证据闭合、T-07/T-08 正式门解锁复勾。
