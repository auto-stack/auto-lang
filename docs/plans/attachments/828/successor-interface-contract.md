# PLAN-828 T-09 后继只读消费接口合同（r3 回写：817/818、820–827）

日期：2026-10-22（r3 回写，替代 2026-10-20 r2 版；r2 版中 `FnEnv::interface_unchanged`/
`credential_current`、`ProgramSnapshot::link` 为生产路径、`HISTORY_CAP 驻留条目永不逐出`
等表述与当前代码不符，均已按真实 API 更正）。

冻结：828 交付的公共接缝与版本/身份语义。后继计划**只读消费**这些接口；改变语义
须回 828 的 owner 走有界修订，不得私建平行 loader 或整模块前端捷径。

## 1. 会话与事务（`auto-av-probe::session`）

| 接口 | 语义 |
|---|---|
| `warmup(src, entry_def_id, caps)` | 冷启动：一次整模块编译 + 单元准入 + `link_with_maps` 快照 revision 0。`entry` 用 def id 约定（`d_main`） |
| `warmup_persisted(src, entry, caps, store_root, project)` | 冷启动 + A09 单元/sidecar/会话 manifest 落盘（798 同一身份方案，与 Driver 写入的 store 互通） |
| `restore(src, entry, caps, store_root, project)` | 温恢复：先消费 manifest 根（`restore_manifest_matched/absent/rejected`），再逐单元 decode+verify_abc+载荷名核对+sidecar 适用性（primary→repair），任何 miss fail-closed 回冷；恢复会话继续持有 store（后续编辑 changed-only 再持久化）；A07 行**存在**即置 `persisted_hir_cv` 归属追踪器（B3-C01：root 归属与 untrusted HIR 可采纳性分轴——合法 span 拒绝不漏退役） |
| `prepare_edit(start,end,insert)` / `commit(PreparedEdit)` / `cancel_prepared(PreparedEdit)` / `edit(...)` | 编辑坐标=旧源字节；`prepare_edit` 产 `PreparedEdit`（候选自持新 body/owner/基面；不动当前修订；`code_weak()` 观测口）；commit=owner/基面/stale/closed 守卫后**同一发布点**原子更新 root/source/revision/env 槽位；`edit`=prepare+commit 直通 |
| `run(limits, cancel)` / `current()` | 顶层调用捕获当前快照；`execute_program(&pinned)` 固定任意旧快照 |
| `counters()` / `STAGES` | 11 轴 per-fn 计数 + `reuse_hit` + `whole_module_*` 结构零 + `syntax_*` 证据 + `hir_soft_root_retired/rerooted`/`persist_error` 等存储轴 |
| `query_body_fact(consumer, callee)` | body 事实查询（闭会话拒；凭据属具体受检产物，重读不刷新） |
| `resident(name)` / `resident_weak(name)` / `resident_inventory()` / `resident_cold_hir_nodes()` | 身份/Weak 观测/驻留盘点（水位轴）；`resident_weak(name)` 观测**当前驻留对**，返回 `Option<(Weak<Body>, Weak<FunctionCode>)>`（B3-C05 更正：无 generation 参数） |
| `last_run_publication()` | B3-C03（F-02 轴在真实消费处）：**最近一次 `run` 的发布对**——身份观测取自该次执行返回的 trace（`ExecTrace.dispatched`：av-core 执行器在四入口采集 def_id/对象地址/代际/内容摘要+`execution_seq`），并含**同一次调用**发布的值；每条身份另带 `resident` 布尔（活地址对比驻留 `Arc`——深副本不可能冒充驻留对象，堆地址回收不可欺骗）。返回 `(Vec<(def_id, ptr, gen, content, resident)>, Option<i32>, seq)`；worker 的值与身份同源于此单一访问器 |
| `close()` / `closed()` / `session_id()` / `function_names()` / `history_len()` / `line_col(off)` / `src()` / `revision()` | 终态与来源查询；close 释放 env/store 拥有引用（幂等；外 pin 存活） |
| `HISTORY_CAP=32` | **每函数**内容历史上界（r3 更正 r2 表述）：有界历史真实淘汰运行（512 编辑不随编辑数增长）；驻留当前修订条目永不逐出 |

**快路径前置**（后继不得放宽）：单编辑、单既有函数体、声明集与签名不变、语法不回退；
违反=显式拒绝并记录原因（`non_fast_path_reason`），绝不静默降级为整模块重算。

## 2. 受检环境与候选（`auto-ac-prototype::source`：`resolve.rs`/`adapter.rs`）

| 接口 | 语义 |
|---|---|
| `CheckedEnv`（`resolve_env` 构造） | 驻留受检接口环境：`close()`/`closed()`/`generation()`；每次成功对外发布（候选 commit+公开 recheck）generation+1——旧候选经 base_generation 失效 |
| `FnCandidate`（`prepare_function_candidate` 构造） | 候选**自持**新条目（move 非 clone）+`env_id`（进程唯一）+`base_generation`+槽身份+工件（`parts()`/`receipt()`）；`commit()` 单次安装、`cancel(self)`/drop 零 env 引用 |
| `prepare_candidate_document(...)` | 免验证单 body 生产事务（resolve/typecheck/emit 单 body；接口目录脚手架；返回前恢复 current） |
| `compile_scalar_source` / `compile_program_parts` / `compile_parts_with` | 整模块冷编入口原样保留（冷初始化/truth 源；非编辑热路径） |

## 3. 函数级凭证与查询（`auto-av-probe::function_queries`、`auto-ac-prototype::verify`）

| 接口 | 语义 |
|---|---|
| `verify_function(bundle, def_index) -> CheckedFunction` | 仅验证器构造的函数级 HIR 凭证：目标 body 检查 + 全 def 签名上下文；`context_revision` 为内容规范化指纹（can3/can4 同族，类型按形状非表下标） |
| `CheckedFunction::{body, def, def_id_text, owner, referenced_intrinsics, context_revision, domain}` | 凭证字段只读 |
| `FnEnv::{from_bundle, interfaces, interface_of, def_id_of, context_revision, domain, revision}` | owner（模块）作用域接口环境；未知名与他 owner 同名=缺席；接口判等=`FnInterface::same_interface` |
| `FnEnv::{set_body_fact, body_fact_of, body_fact_current}` + `BodyFactRead` | body 事实查询核心：当前真值/读取快照/失效判定；凭据挂具体产物（history 条目自带冻结 deps），重读返回新值不刷新旧凭据 |

## 4. 受检代码与快照（`auto-av-core`）

| 接口 | 语义 |
|---|---|
| `FunctionCode::admit_unit(loaded, mint_revision)` | 仅接受 ABC 准入的**单元**凭证；`generation()`=铸造修订 |
| `ProgramSnapshot::link_with_maps(codes, entry, revision, span_maps)` | 生产受检链接（`link`=零图薄包装）：B2-C01 单一 owner 上下文门 + B2-C02 实际调用 import 闭包门 + `fn:<name>` 签名等值绑定表（scratch interned 类型表仅作内容等值校验，不保留）；**无 merged 执行载荷**——不复制 blocks/slots/spans/指令 |
| `ProgramSnapshot::unit_view()` → `UnitView` / `ModuleView` trait | 三执行入口共用的单元派发视图（单元局部 const/type/slot/PC + 绑定表；span 经帧执行槽投影 `map_span`） |
| `ProgramSnapshot::{dispatched_payload_same_object, dispatched_payload_same_content, code, find_code, generation_of, revision}` | 实际派发载荷身份轴（强制副本负控的 oracle）+ 稳定 def-id 身份/代际查询 |
| `execute_program(prog, …)` / `execute_program_steps(…, max_steps)` / `resume_program(paused, …)` / `PausedRun::step_count` | 原子执行 / 步数屏障暂停（PausedRun 固定快照）/ 恢复；分段预算（B2-C04：段内增量比较，累计硬限额跨段）；三入口同派发 |

**身份律**：跨修订函数身份=def id 文本（`d_<name>`）；不使用数组序号/源偏移/代码地址。
未改函数的 HIR body、FunctionCode 对象与代际跨修订不变；内容命中复用原对象原代际。

## 5. 存储/恢复/身份（`auto-av-probe::cache` + 783 ManagedStore 合同内）

- A09 单元身份沿用 798（`abc_unit_key`/`EnvelopeIdentity::abc_bound`）；单元 ABC 行与
  span sidecar 为 **store-internal**（无 soft root），liveness 挂 manifest `RequiredData` 边；
  sidecar 冲突走 primary→repair 双行（F10-S02 适用性标准）。
- 会话 manifest：`format=2` **双指纹键**（body 指纹 + 规范化 required 闭包指纹，T07-R3-01）；
  restore 消费注册根（严格目录判等 T07-R2-01）；superseded 根在发布点退役——被取代修订
  载荷可被真实 GC 回收。
- A07 函数 HIR 工件（层 schema `ac828/a07-fnhir/v2`，can4 内容身份）：soft root **有界**
  ——每函数至多一个当前 cv 的 root；发布新 cv 退役旧 root、内容回访重挂、restart 采纳置
  追踪器（计数 `hir_soft_root_retired`/`hir_soft_root_rerooted`）；512 修订存储水位有界
  （V-07 完整存储轴测试为 oracle）。
- 恢复从不信任落盘：manifest 只拒 warm 不加信；每单元重验；HIR 采纳走五门
  （codec/header→rebind→canonical→span→verify+intrinsics）。

## 6. 前端接缝（`ac-syntax-core-adapter`、`grammars/auto-scalar.at`）

| 接口 | 语义 |
|---|---|
| `project_decl_strict(grammar, kinds, tree, snap, idx)` | 单声明严格投影（span=快照绝对坐标；error/missing 拒绝） |
| `incremental_reparse` + `PageStore`/`Snapshot` | 787 增量词法/CST 编辑路径（会话驻留态） |
| `grammars/auto-scalar.at` | 全 scalar 表面版本化语法；与 ac-core scalar parser 的 AST+span 全等由 `scalar_projection_` 差分钉死 |

## 7. 版本合同

- 语法/身份/指纹变更=新 revision：auto-scalar rev 递增；`context_revision`/canonical
  （can4）语义变更须同步迁移所有存量凭证并换内容键（828 owner 裁定）。
- 计数器名（`STAGES`、`whole_module_*`、`reuse_hit`、`syntax_*`、`restore_*`、
  `hir_soft_root_*`）为正式结构门词表，等价改名须同步 `formal_828::validate_and_gate`
  与父侧 judge（`formal_worker.rs`）。
- 正式测量入口：`av_probe_worker session-serve`（常驻 worker）+ `formal_worker.rs`
  父侧 judge + `formal_campaign.rs` 战役脚本；`frontend-attribution` 子命令=双线预算
  A 线隔离测量（798 §5.9 同口径）。

## 8. 后继接入方式（817/818、820–827）

- **只读**：以上接口直接消费；828 的会话/快照是增量机制的唯一实现基座，后继不得另走
  整模块前端或私建 loader（T-09 合同；计划 §4/§8 后继交接说明）。
- **817（AV CTE 查询冻结缓存）**：body 事实消费者经 `FnEnv::{set_body_fact,
  body_fact_of, body_fact_current}`/`session::query_body_fact` 接入失效轴（V-03 语义：
  已读事实变化→消费者失效，重读不刷新）；CTE 冻结缓存身份复用 783 store 合同
  （同 layer+key 单身份），不建第二套数据库（A07 接 Design40 统一引用）。
- **818（AC 共享 Typed MIR 原生一致性）**：`lower_checked`/`lower_function_unit`/
  `verify_mir` 的单元 MIR 与 `OwnerScope` 身份为共享面；单元 wire 身份（A09）与
  owner stamp 语义（verify_abc 调用方上下文）保持；一致性对拍消费
  `dispatched_payload_same_content`。
- **820–824（AV 类型化数据/Place/集合/闭包/动态冻结）**：语言面扩展走 auto-scalar
  语法+投影体系补齐+差分对拍验收，不悄改旧 core/scalar grammar；新数据形态的装载/
  执行经 `FunctionCode::admit_unit`+`link_with_maps`+UnitView 单元派发（无 merged 回退
  路径）；新语义字段进 canonical（canN 换键）与 A07 codec（版本化层 schema）。
- **825（装载执行表示与运行效率）**：驻留复用/水位（`resident_inventory`）与三入口
  预算（分段 resume）为基线；效率改动不得破坏身份轴/结构门（`STAGES` 词表冻结）。
- **826（紧凑 ABC 格式与体积）**：ABC wire 版本升级沿用 798 信封/单元身份律
  （schema 语义版本变化即失效对应层）；单元键/manifest 双指纹不受编码压缩影响。
- **827（源码程序验收基线）**：验收负载经 `load256::generate_revision` 家族与
  `formal_campaign`/`formal_828` 双门框架扩展（多场景=新战役脚本，同 judged 窗语义）；
  快路径准入与结构门（未改零阶段/零整模块/恰一安装）为验收不变量。
- 数据/堆对象、CTE、JIT 等扩展落地时：`FnEnv` 增加相应依赖轴（body 事实消费者失效），
  依 V-03 语义补负例。
