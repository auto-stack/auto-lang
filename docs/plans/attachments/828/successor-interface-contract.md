# PLAN-828 T-09 后继只读消费接口合同（817/818、820–827）

冻结：828 交付的公共接缝与版本/身份语义。后继计划**只读消费**这些接口；改变语义
须回 828 的 owner 走有界修订，不得私建平行 loader 或整模块前端捷径。

## 1. 会话与事务（`auto-av-probe::session`）

| 接口 | 语义 |
|---|---|
| `warmup(src, entry_def_id, caps)` | 冷启动：一次整模块编译 + 单元准入 + link 快照 revision 0。`entry` 用 def id 约定（`d_main`） |
| `warmup_persisted(src, entry, caps, store_root, project)` | 冷启动 + A09 单元/会话 manifest 落盘（798 同一身份方案，与 Driver 写入的 store 互通） |
| `restore(src, entry, caps, store_root, project)` | 温恢复：逐单元 decode+verify_abc+载荷名核对（fail-closed 回冷启动）；`restore_units_admitted`/`restore_materialize` 计数证明零前端可运行 |
| `edit(start, end, insert)` / `prepare_edit` / `commit` | 编辑坐标=旧源字节；prepare 产 `PreparedEdit`（不动当前修订），commit 按 base revision 守卫拒绝陈旧候选 |
| `run(limits, cancel)` / `current()` | 顶层调用捕获当前快照；`execute_program(&pinned)` 固定任意旧快照 |
| `counters()` / `STAGES` | 11 轴 per-fn 计数 + `reuse_hit` + 语法证据累计 + `whole_module_*` 结构零 |
| `resident(name)` / `function_names()` / `history_len()` / `line_col(off)` | 身份/来源查询 |
| `HISTORY_CAP=32` | 内容历史上界（驻留条目永不逐出） |

**快路径前置**（后继不得放宽）：单编辑、单既有函数体、声明集与签名不变、语法不回退；
违反=显式拒绝并记录原因，绝不静默降级为整模块重算。

## 2. 函数级凭证与查询（`auto-ac-prototype::verify`、`auto-av-probe::function_queries`）

| 接口 | 语义 |
|---|---|
| `verify_function(bundle, def_index) -> CheckedFunction` | 仅验证器构造的函数级 HIR 凭证：目标 body 检查 + 全 def 签名上下文；`context_revision` 为**内容规范化**指纹（类型按形状，非表下标）——布局无关，桩程序与真实程序同指纹 |
| `CheckedFunction::{body, def, def_id_text, context_revision, domain}` | 凭证字段只读 |
| `FnEnv::from_bundle` / `interface_of` / `def_id_of` | owner（模块）作用域接口环境；未知名与他 owner 同名=缺席 |
| `FnEnv::interface_unchanged` / `credential_current` | 快路径接口门 + 凭证流通性（铸造上下文修订必须仍是当前值；重验，不复盖戳） |

## 3. 受检代码与快照（`auto-av-core`）

| 接口 | 语义 |
|---|---|
| `FunctionCode::admit_unit(loaded, mint_revision)` | 仅接受 ABC 准入的**单元**凭证（executable/is_unit/单函数三重校验）；`generation()`=铸造修订 |
| `ProgramSnapshot::link(codes, entry, revision)` | 受检链接：`fn:<name>` 签名等值 patch、重名/入口检查；无整闭包 codec 往返、无整模块重验证 |
| `ProgramSnapshot::{code, find_code, generation_of, revision}` | 稳定 def-id 身份轴 + 代际查询 |
| `execute_program(prog, …)` / `execute_program_steps(…, max_steps)` / `resume_program(paused, …)` | 原子执行 / 步数屏障暂停（`PausedRun` 固定快照）/ 恢复；fuel/frames/slot 预算跨暂停携带，wall 按段 |
| `verify_mir` / `encode_abc` / `decode_abc` / `verify_abc` | 798 既有准入链（828 单元路径复用，未改语义） |

**身份律**：跨修订函数身份=def id 文本（`d_<name>`）；不使用数组序号/源偏移/代码地址。
未改函数的 HIR body、FunctionCode 对象与代际跨修订不变；内容命中复用原对象原代际。

## 4. 前端接缝（`auto-ac-prototype::source`、`ac-syntax-core-adapter`）

| 接口 | 语义 |
|---|---|
| `compile_program_parts(program, src, …)` | 已解析 Program 直入 resolve→…→bind（编辑热路径用）；`compile_parts_with` 等整模块入口原样保留 |
| `project_decl_strict(grammar, kinds, tree, snap, idx)` | 单声明严格投影（span=快照绝对坐标；error/missing 拒绝） |
| `grammars/auto-scalar.at`（rev 1） | 全 scalar 表面版本化语法；与 ac-core scalar parser 的 AST+span 全等由 `scalar_projection_` 差分钉死 |

## 5. 版本合同

- 语法/身份/指纹变更=新 revision：auto-scalar rev 递增；`context_revision` 语义变更须
  同步迁移所有存量凭证（828 owner 裁定）。
- A09 单元身份沿用 798（`abc_unit_key`/`EnvelopeIdentity::abc_bound`）；会话 manifest
  为 fail-closed 目录（单元自身重验，错名=miss）。
- 计数器名（`STAGES`、`whole_module_*`、`reuse_hit`、`syntax_*`）为正式结构门词表，
  等价改名须更新 `formal_828::validate_and_gate` 同步。

## 6. 后继接入方式（817/818、820–827）

- **只读**：以上接口直接消费；828 的会话/快照是增量机制的唯一实现基座，后继不得另走
  整模块前端或私建 loader（T-09 合同；计划 §4 r2 补记）。
- 扩展语言面（新 profile/构造）：走 auto-scalar 语法+投影体系补齐 + 差分对拍验收，
  不悄改旧 core/scalar grammar。
- 数据/堆对象、CTE、JIT 等扩展落地时：`FnEnv` 增加相应依赖轴（body 事实消费者失效），
  依 V-03 语义补负例（coverage-manifest 开放项 2）。
