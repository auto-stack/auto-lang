# PLAN-788 T-01 契约决策稿（v2/IPC/阶段/预算/性能配方冻结）

日期 2026-10-16。stage:work T-01。本文件冻结 T-02–T-06 实施所依赖的全部合同决策；
后续任务不得事后放宽此处冻结的预算/阈值/支持面（计划 §3、§8 T-01）。

## 1. 基面、树与工具链（核树记录）

- 实施树 `D:/autostack/.wt/lang-788/auto-lang`，分支 `plan-788-dev`，基面 master
  `2f6b06c747c1318feeac217939a5e91ffd1307a9`（从 master 开树，非 787 在途树）。
- 787 冲突面（本轮**禁触**）：`experimental/ac-syntax/**`（787 全部新增+在途未提交修复）、
  `experimental/ac-artifact-probe/src/{codec.rs,schema.rs,store/mod.rs}`、
  `experimental/ac-artifact-probe/tests/admission_.rs`。788 的 ac-core 改动（T-03）与
  driver.rs 观察入口（T-05）均在 787 diff 之外，合并无冲突面。
- 参考环境（§5.5 性能配方固定于此机）：Intel i5-13600KF / 31.8GB RAM /
  Windows 11 专业版 / rustc 1.98.0 (88d9e12ae) / cargo 1.98.0。电源高性能、release 工具。
- 环境注记：本机 sccache 服务端口落入 Hyper-V 保留段（os error 10013）；绕行
  `RUSTC_WRAPPER=` 或改 `SCACHE_SERVER_PORT`。属环境问题，不改变任何合同。

## 2. T-01 whole-path 可行性实验（已执行）

`experimental/ac-case-worker/tests/feasibility_t01_.rs`（`#![cfg(windows)]`，非 Windows
整体跳过、required-native 不假绿），3/3 PASS（2026-10-16，worktree 树）：

1. `t01_core_source_to_pe_and_run` — p04 真源码 `compile_source` → `find_entry("d_main")`
   → `lower_object`（真 COFF，`\x64\x86` 机器签名）→ `link_object_staged`（真 rust-lld +
   SDK um/x64）→ `publish_artifacts`（真 PE，`MZ`）→ `run_exe` exit==3 oracle。
2. `t01_raw_hir_negative_is_typed_rejection` — `dangling-expr-ref.atom` 经真 binder 拒绝，
   typed `Stage::Bind` + code `bind.dangling-ref` + span 指向 `e_missing`。
3. `t01_fresh_then_hit_trace` — 783 `ManagedStore`+`vertical_and_run`×2：fresh 计数
   (parse,verify,lower,link,run)=(1,1,1,1,1)、hits.object=="lowered"；第二次 object hit
   (0,0,0,1,1) 且 link/run 仍真实。结论：fresh/hit/重链接语义可直接承载 §5.4 全部断言。

结论：whole-path 可行，无需复制编译管线/执行器；后续按本稿实施。

## 3. v2 协议冻结（T-02 实施依据）

### 3.1 版本分派

- 新增 `autolang.case/2`、`autolang.suite/2`、`autolang.test-report/2`；三个 decode 入口
  按 schema 字符串**精确相等分派** v1/v2（沿用现有 exact-match 拒绝风格，不引入
  major/minor 解析）。
- v1 的合法/非法集合**字节不变**：v1 文档塞入任何 v2 字段仍按 unknown-field 定位拒绝；
  v2 文档中 avm/a2r 检查的行为与 v1 完全一致（含 defaults 继承/deep-merge/FilterNulls）。
- v2 是 v1 的严格超集：v1 已有字段语义不变；仅新增下列字段。所有新字段的 known-set
  封闭在 `Fields::new` 列表（沿用"未知字段=定位错误"政策）。

### 3.2 v2 新增字段（最终命名）

| 层 | 字段 | 取值/默认 | 说明 |
|---|---|---|---|
| case source.syntax | `"hir-atom"`（新增枚举位） | 扩展名 `.atom` | 原始 HIR Atom 文本输入；v1 位 `auto`/`auto-script` 不变 |
| check.adapter | `"ac"`、`"syntax"`（新增） | — | avm/a2r 不变；`syntax` 本轮 decode 受理、执行一律 admission ERROR（见 §7） |
| check.action（ac） | `"check"`/`"build"`/`"run"` | — | 与 avm/run、a2r/emit 并存；矩阵：ac×{check,build,run} |
| check.action（syntax） | `"parse"`/`"edit"` | — | 同上，T-06 绑定执行 |
| check.options（ac） | `profile: "core"\|"scalar"`（默认 core） | 固定首批两 profile | text/record/text-record 属未导入面，decode 即 ERROR（不支持不被 SKIP 掩盖） |
| | `target: "x86_64-pc-windows-msvc"`（唯一合法值） | — | 其他 target decode ERROR |
| | `entry: <string>`（默认 `"main"`） | — | 零参 i32 入口函数名（worker 侧解析 `d_<entry>`） |
| | `cache: "fresh"\|"reuse"\|"read-only"`（默认 reuse） | — | §5 语义见 §6 |
| | `budget: { build_ms?, run_ms?, total_ms? }`（u64） | 缺省=硬上限；申报值只能 ≤ 硬上限 | 上限见 §5；超上限 decode ERROR |
| check.expect（ac） | `exit: <i32>`（仅 ac/run） | — | 原生退出码 oracle；trap=70 是普通可声明退出码 |
| | stdout/stderr | 沿用 StreamExpect | v2 语义=原始 bytes 比较（base64 链路保真；`bytes`\|`lf` 模式沿用） |
| | artifacts | 沿用 ArtifactExpect + kind 扩展 `hir`/`obj`/`exe`（logical 固定表） | 规范 HIR=逻辑名 `hir.atom`；产物=obj/exe |
| | `perf: { total_ms_max?: u64 }`（可选） | 缺省仅记录不强判 | 参考环境性能配方使用；普通单测不承担毫秒断言（§5.5） |
| completion.stage（ac） | v2 typed stage 枚举 | — | 见 §4；avm 检查仍用 v1 Parse/Type/Run |
| syntax 专属字段（edits 序列等） | — | — | **T-06 冻结**，本轮不预留半成品字段 |

- `product_from` DAG 语义不变：ac/build、ac/run 的真实产物 manifest 参与交接；执行失败/
  无产物才 BLOCKED；比较 FAIL 但有效产物仍可被消费（计划 §5.2）。
- report v2：`autolang.test-report/2`，新增 `trace`（每 check 的阶段 span 数组）、
  `cache`（mode/hits/counters）、`perf`（wall_ms 分位样本引用）；功能结果与性能结果分列，
  未实现阶段 not-applicable 而非 0ms。

## 4. 阶段映射冻结（v2 typed stage）

v2 ac 阶段枚举：`parse | resolve | typecheck | bind | verify | capability | backend |
link | run`。映射来源（实测，§2）：

| v2 stage | ac-core 来源 | 诊断特征 |
|---|---|---|
| parse | `source::parser`（观察边界） | `Stage::Text` + code `source.syntax`\|`source.literal-range` |
| resolve | `source::resolve`（观察边界） | `Stage::Text` + `source.{undefined,not-callable,argument,duplicate}` |
| typecheck | `source::typecheck`（观察边界） | `Stage::Text` + `source.type-mismatch` |
| bind | descriptor bind | `Stage::Bind` + `bind.*` |
| verify | `verify::*` | `Stage::Verify` + `verify.*` |
| capability | 能力门 | `Stage::Capability` + `capability.*` |
| backend | `native::lower_object` | `Stage::Backend` + `backend.*` |
| link | `link.rs`（lld/SDK/publish） | `Stage::Link` + `link.*` |
| run | 原生执行 | `run.*` |

- 前端三阶段共用 `Stage::Text`/`source.*`——**阶段归属只能来自 T-03 观察边界**（哪个
  边界返回错误），code 原样透传不猜。worker 不从 render 字符串分类（§5.2 红线）。
- Infrastructure（worker/IPC/IO/工具缺失/panic/非法退出/超时/超输出/响应失配）不是
  stage，也不是 completion 可匹配项：产生 runner 级 `Status::Error`，永不满足语言负例。

## 5. 资源预算冻结（硬 cap，代码常量，不得事后放宽）

每 worker 请求：build ≤ 60000ms、run ≤ 5000ms、total ≤ 70000ms；
stdout/stderr 各 ≤ 1 MiB；IPC 单响应元数据 ≤ 4 MiB；单产物 ≤ 64 MiB；
单 check 实际产物总量 ≤ 128 MiB。会话：≤ 256 请求、总生命周期 ≤ 180000ms。
到达任一输出上限→立即终止全树并报 ERROR（不返回截断成功）；到达时限→杀全树+回收。
case 输入沿用 v1 admission 限额（auto-test-case `Limits`），不另设。

## 6. worker IPC 与 cache 冻结（T-03/T-05 实施依据）

- 进程模型：auto-lang 永不静态依赖 ac-core/Cranelift；`test_cases::ac` 以子进程调用
  worker。解析序：`AC_CASE_WORKER` env → 从仓库根 `experimental/ac-case-worker`
  `cargo build --release`（cargo 空转 ~1s 热缓存；失败=infra ERROR）。性能配方一律用
  release 构建。
- 传输：stdin/stdout 各一行一个 JSON（schema `"acworker/1"`）；stderr 仅转发诊断不作
  oracle。字节载荷（HIR、stdout/stderr、产物内容）一律 base64 保真；非法 UTF-8 输出按
  bytes 比较，lossy 仅用于展示。
- 会话：`{op:"begin", budgets}` → `{op:"request", ...}`×N → `{op:"end"}`；parent 侧
  独立墙钟兜底杀树（不信任 worker 自限）。重启场景=真新进程（同一 store 根复用磁盘）。
- request 携带：schema、kind（check/build/run）、nonce、request_hash（parent 对规范串
  计算，worker 重算核对）、source/hir base64、profile、target、entry、budgets、
  scratch（父 owned 绝对路径）、store_root + cache 模式。worker 只接受本次有界 request；
  nonce/request_hash 不符、未知 schema、越界路径、旧响应→ERROR（§5.3）。
- response 携带：typed diagnostics 数组（§4 形状）、真实产物 manifest（logical/path/
  len/sha256/kind）、run 观察（exit_code Option、stdout/stderr base64、timed_out）、
  trace（阶段 span：wall/等待/计数）、cache（mode/hits/counters）、toolchain 身份
  （lld 路径+文件 sha256、SDK um 目录、ac-core 版本、worker 版本；T-06 加 grammar
  revision）。**PASS 判定永远在父侧**：父重核 manifest 哈希/路径集合，不信任 worker 结论。
- cache 模式（783 语义映射，driver 观察入口实现，不改 store/GC/codec/schema）：
  - `reuse`：现行 get→compute→publish（HIR 命中重 verify，obj 命中跳前端/verify/lower
    但真 link/run）。
  - `fresh`：跳过全部 get，真实重算并按安全合同 publish（证明重算发生用计数+禁调反控）。
  - `read-only`：`open_existing`（缺根=infra ERROR）；只读命中照常消费；miss 明确报告
    并真实重算执行但不 publish。所有模式**每次重读 oracle 重新比较，绝不缓存 PASS**。
  - toolchain_fp 由 worker 从**真实工具文件哈希+版本**构造（非调用方常量）。
- ac-core 窄扩展（T-03，旧 API 不动）：`link.rs` 新增 rawbytes+cap 采集入口
  （现 lossy 转换点 link.rs:344/351 的有界 bytes 版本，旧 String API 作兼容 wrapper）；
  `source/adapter.rs` 新增观察入口（同一管线、边界回调；`compile_parts_with` = 无观察
  wrapper）。worker/linker/PE 全部在 Job containment+总截止内；Job 失败不降级裸跑。

## 7. 787 接缝清单（T-06 绑定前提 = D-787）

- 消费对象：合入后的 `experimental/ac-syntax/{grammar,runtime,adapters/{core,artifacts}}`
  公共 API 与 `grammars/{auto-core.at, interface-mini-v1.at}`；本轮基面无 ac-syntax，
  不从 787 在途树复制任何代码。
- 接入点：worker 侧 `ac-case-worker/src/syntax.rs`（syntax adapter 放 worker 侧，787
  runtime 零产品依赖红线保留）；auto-lang 零依赖不变。
- v2 已受理 `syntax`×{parse,edit} 的 decode；在 D-787 满足前执行一律 admission ERROR
  （"syntax adapter not bound (PLAN-787 pending merge)"），不 SKIP 不假绿。
- T-06 时冻结：edits 有界非重叠序列字段、当前 revision 语义、取消/重播/发布顺序观察、
  grammar revision 进 toolchain 身份；并复用 787 的预算/复用不变量。
- 触发条件：787 独立 review pass + 门禁 + 实际合入 master + Spec 沉淀 + merge 收据。

## 8. 性能配方冻结（T-05 采集、T-08 聚合验收）

- 场景六组×≥30 样本（本机参考环境，release worker，预热 3 弃置）：fresh（每样本新
  store 根）、同进程热点（单会话串行请求）、重启磁盘热（每样本新进程、同 store 根）、
  跨工程相同内容（同内容不同 project 根）、局部 body 编辑（p04 循环界 3↔4 交替）、
  GC 并发（请求流中跑 783 GC）。
- 计时：父侧墙钟=请求写出到响应读完（含 materialize/IPC/校验/发布）；会话首启与排队
  计入相应请求并单列；阶段 span 来自 worker trace，不重复归因、不相加冒充端到端。
- 硬门（§5.5，不达标即明确未达，不删 AC 不扩预算）：原生热点与局部 body 编辑的
  请求→正确 run 结果 **P95 < 1000ms**；syntax 局部 edit 同阈值归 T-06。
- 报告：P50/P95/P99/峰值+长尾+完整 raw 样本（超时/取消/失败样本保留不筛）；落
  `docs/reports/plan788/`。50×100ms 修订流（逆序完成/取消/发布顺序）归 T-06/T-08。

## 9. 实施修订（T-03–T-05 期间的诚实偏差记录，复审核验点）

1. **request_hash 由 worker 重算 → 改为原样回显 + 父侧比对**（§6 原文"worker 重算核对"）。
   两个独立 workspace 无共享规范序列化，重算只会制造伪校验；绑定性质（旧响应/错配响
   应必失败）由父侧 echo 等比较完整保留。nonce 同样回显比对。
2. **请求字段补充**：`project`（store 命名空间，跨工程场景用；缺省 "ac-case"）、
   `include_hir`（父侧声明了 hir.atom 金样时请求 canonical bytes）——均为 §6 冻结面的
   自然组成，无预算/阈值变化。
3. **driver 路径的 stdout/stderr**：probe 的 run 腿是 String（783 既有事实，未改
   subproc）；worker 的 run 腿保留在 worker 侧（run_deadlined_captured），因此预算、
   cap、原始 bytes 在**所有路径**成立；driver 自带 run 腿未被 worker 使用。
4. **本批 V-06 的非 ASCII 生产者**（R3 修订）：core/scalar 程序无 print 内建（trap
   stderr 为 ASCII），语言内非 UTF-8 字节生产者随 text 批次导入；R3 起真实 worker
   路径的生产者已经**产品消费腿**覆盖——测试内 rustc 构建的真实 PE 经
   `ac/run product`（同一 contained 执行器/cap/base64 链/父侧严格解码）：NUL+CRLF
   为可比对 oracle 字节（consume_and_run 全 runner 路径），非法 UTF-8（0xff/0xfe/0x80）
   在未声明 stderr 流上经协议逐字节保真并被完整 admission 接受
   （`r3_native_bytes_real_worker_path_producers`）。语言内生产者仍随 text 批次导入。
5. **跨进程重启复用**：worker 级 exec 双请求经同一磁盘 store 证明复用；真新进程由
   783 native_resume_（跨进程 node 二进制）证明；T-08 验收矩阵在 AC_CASE_STORE 钉根
   下补齐 runner 级重启样本。

## 10. R1 修复后的补充冻结（2026-10-16，复审 R1 后；R2/R3 修订见标注）

1. **工具链身份轴**（R3 定稿，取代 R1 两轴收敛案）：按**每行实际消费路径**绑定可
   重算身份集合——`worker-exe-sha256`+`ac-worker-protocol` 恒在；`lld-sha256`
   （rust-lld 文件内容哈希）与 `sdk-um-sha256`（SDK um/x64 kernel32.lib 内容哈希，
   **非路径**——路径不证明库内容身份）仅绑定真实链接过的 ac/build|run 行（产出
   obj/exe 或含 run 观察）；产品消费 run 与 check/syntax 行只有恒在两轴；声明
   support_lib 时 `support-lib-trace-marks-sha256`（库文件内容哈希）入轴。stale
   按同一适用集合比较：缺轴/多轴/值漂移均 STALE，未知/缺失不静默相等；测试走
   production stale（auto-case 子进程 + AC_CASE_WORKER 换 worker），不注入缩小轴集。
2. **预算语义**：单一绝对 total 自 admission 起算（父墙=total，无宽限）；build 段
   预算覆盖前端→lower→link；run 取 min(run_ms, 剩余 total)；每个出口（含 check、
   语言失败、typed 重放、cache 命中）核预算。R3 补充：**request 写入（stdin）与
   响应等待/进程回收同受该墙钟约束**——写入线程化，不读 stdin 的 worker 由
   deadline 杀树解除满管阻塞，不产生 ~1.2s 的迟到失败。
3. **非参与组合**：`cache: fresh|read-only` 声明在 ac/check 或 hir-atom 输入上为
   admission ERROR（契约层门控 + worker 镜像拒绝）；默认 reuse 不承诺参与。
4. **流 cap 语义**（R3）：stdout/stderr 的 1MiB cap 按父侧**解码后字节数**强制
   （validate_response），与是否声明流 oracle 无关；worker stderr 排空带 cap 信号，
   超量=受控 ERROR 不截断成功。
5. **部分产物语义**（R3）：语言失败响应可携带合法部分产物（canonical hir.atom=
   前端在失败边界前的真实输出，manifest 照常核验/持久化）；obj/exe 是成功产物，
   出现在失败响应中=伪造，一律拒绝（负向保留 typed backend oracle）。

## 11. R3 修复后的补充冻结（2026-10-17，复审 R3 后）

1. **性能计时口径**（落实 §8 原冻结"含校验/发布"）：会话样本墙钟=请求构造→IPC→
   **serde 解析→validate_response→manifest 准入+持久化→oracle 比较**；one-shot
   样本=spawn→同链→close（reap）。判定一律经 runner 同名公共入口
   （validate_response / admit_and_persist_artifacts / compare_stream_field）——
   harness 不复刻判定。原始样本 JSONL **逐样本先落盘**（gate 断言前，失败/拒绝
   样本保留）；CPU/IO/峰值内存为本批缺口如实标注，不 0 填充。
2. **body-edit 配方**（回归 §8 冻结原文）：p04 循环界 **3↔4 真交替**（独立 exit 3/4
   oracle）+ 每样本唯一 marker 注释（不缓存旧结果）；逐样本断言 parse==1 且
   object=lowered。
3. **GC 并发场景**：783 GC 以**独立真实进程**（新 bin `ac-artifact-probe/src/bin/
   store-gc.rs`，与 783 套件同一 ManagedStore+run_gc）并发运行；每样本 barrier
   证明（gc-started 行 + 请求前零等待探测未见结束 + 结束行时间戳 > 请求起点），
   30/30 样本带起止区间与重叠毫秒数。
4. **修订流（V-13）**：双真实 worker 车道、固定 100ms 到达时钟（驱动器从不等响应
   ——到达时刻无累积漂移为断言）；每 7 个修订一个 heavy（31 连插 64KiB+31 连删，
   fresh↔incremental 等价检查按累计文档解析、成本二次增长，~520ms）vs 后继 light
   （~3ms）产生**真实错序完成**；latest-wins 发布仲裁拒绝一切非最新完成（错序/
   superseded 取消均留痕），发布序列严格递增、最终发布=最新修订；全部 50 条记录
   留存（含拒绝），积压深度可见（≥2）。
5. **验收器**：required ID 按 **executed-ok 精确名**解析（libtest 行 `test <name>
   ... ok`），逐 ID 绑定所属 test binary，逐 binary 最小计数；prose 文本与名字前缀
   均不能替代（负控永久入 selftest）。closeout 的根 cargo t/tv/tt 按 **nextest 实际
   输出**（PASS 行 + "N tests run: M passed, K failed, J skipped"）另行核对，跳过
   不设白名单。
