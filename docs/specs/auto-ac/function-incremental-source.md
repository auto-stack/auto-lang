# auto-ac 函数级增量源前端（PLAN-828）

> Status: active（scalar-i32/bool profile；实现=`experimental/ace/crates/core/src/source/*`
> + `experimental/ace/crates/av/src/session.rs` 热路径）。

## 受检接口环境与单 body 检查

- 冷初始化走完整前端（`compile_scalar_source`：parse→resolve→typecheck→Atom
  render/bind→verify），结合 `resolve_env` 保留 `CheckedEnv`（驻留受检接口环境：每函数签名/owner 域/
  generation）——整模块入口原样保留，作为 truth 源与冷启动路径。
- 编辑热路径**不再构造全部 stub Program**：ac-syntax 增量重扫（`incremental_reparse`）
  →单声明严格投影（`project_decl_strict`）→`prepare_candidate_document`（真单 body
  resolve/typecheck/emit 事务：接口目录脚手架、恰一 body 发射、返回前恢复 current）
  →FnEnv 接口门→**内容版本判 hit/miss**——hit 复用已验对象（零重验证）；miss 才
  `verify_function` 凭证门→全链（lower/mir_verify/abc/decode/admit）。会话生产
  consumer 不经 `prepare_function_candidate`（那是 ac-core 的组合公开接缝=document
  +verify；B3-C05 更正早期把两者顺序写反的表述）。
- 结构门：生产者窗口内改函恰 +1×4 阶段（project/resolve/typecheck/hir_build）、
  其余函数全零、整模块轴零；强制走旧整 Program 管线时同门必红（永久负控）。

## 函数凭证与上下文身份

- `verify_function(bundle, def_index) -> CheckedFunction`：仅验证器构造；绑定 body
  内容、owner/域、实际读取事实与 `context_revision`（内容规范化指纹，类型按形状
  非表下标）。
- 候选事务（`FnCandidate`）：候选自持新条目（move）+env_id（进程唯一）+
  base_generation；commit 单次安装（与 root/source/revision 同发布点）；
  cancel/drop 零 env 引用；公开 recheck 成功同属发布（generation+1，旧候选经
  base_generation 失效）。
- canonical 内容身份（can4）覆盖 hir PartialEq 全部语义字段（绑定含 moved、
  Binary overflow；cosmetic 仅 span/mutable_explicit 不进键）；格式标记换键即
  版本隔离旧行。

## 来源与失效轴

- 跨修订函数身份=def id 文本（`d_<name>`）；body 长度/行号变化不使未改函数凭证
  按全局 source revision 失效（目录/词法边界/来源映射单列）。
- 签名/声明集/配置/授权变化、跨函数编辑、词法不收敛=显式非快路径（记录
  `non_fast_path_reason`），不静默降级。
- body 事实查询核心（`FnEnv::{set_body_fact,body_fact_of,body_fact_current}`）：
  凭据属具体受检产物（history 条目自带冻结 deps）；已读事实变化→消费者失效；
  重读返回新值不刷新旧凭据；未读不失效；只读接口 caller 零重编。

## 含 type/enum 声明族的暖片段（PLAN-904 共享修复）

- Scoped 片段文档中，保留的 type/variant 声明是 **inert 接口元数据**（span 置零），跨
  record/text-record/collections 族共享（整模块发射保留真实声明 span，902 字节不变）；
  902 含 type 的 Record/TextRecord 自由函数 body 编辑恢复**真 warm 片段路径**（不再走
  整源冷路径），902 原文件集/golden/语义键不变。
- 凭证绑定（V-13）：`context_revision` 声明指纹匹配**不证明** body 相同——
  `CheckedFunction` 携带规范 span 无关 `body_digest`，`SemanticView::from_parts` 同时核
  声明上下文与 body 摘要；同 owner+同上下文+异 body 是具名 `a2x.credential` 拒绝，
  合法凭证照常准入。
- collections 族的暖编辑授权是 **ranged**（见 auto-a2r/collections-hir-backend.md）：
  base 完整源 sha 绑定 + UTF-8 range 精确覆盖目标声明 + 同名单声明片段；manifest/证书
  绑定**替换后完整源** sha，不是 fragment-only hash。中文/多字节变长编辑合法；整源/
  零宽/错名 range 具名拒。


## closures 源码入口的 owner 粒度增量（PLAN-909）

- lambda 稳定身份 `__closure_<owner>_<k>`（owner 内序号，resolver 按 owner 计数）；
  resolved IR 的 `CreateClosure` **按名**引用 callee（typecheck/adapter 按名查），
  无关 lambda 的插入/删除/重排不移动任何 DefId 或单元身份；模块递增序号退役。
- owner 粒度前端缓存（`ClosuresOwnerCache`）：每顶层函数记条目文本指纹、resolved
  条目与其 lambdas；模块声明区上下文指纹（各 item 签名区/非 fn item 全文）与闭包
  签名表快照（`VT::Function` 索引稳定——暖路径先播种旧表，intern 值去重只追加）。
  未改 owner **零 resolve/typecheck walk**（`body_work` 真实计数观测），其单元跳过
  lower/encode；提交仍由 wire 身份经 ClosuresUpdateSession 结构门决定。
- PLAN-912 后继：closures 域暖路径已收口为**单函数生产**——见
  [闭包单函数前端生产](closures-function-production.md)。lambda 单 body 改经保留
  创建点事实只重解析/发射/验证/lower 该单元；owner body 改经 retained-lambda 复用
  oracle（兄弟 lambda 不重走，`resolve_walks` 对真实走过的 lambda 逐单元可见）；
  签名/拓扑/跨单元编辑具名非快路径走整模块 truth；未改单元全阶段 P=0 且同对象/
  代际，whole 轴在暖路径必须为 0。残余如实：restart 仍先重 lower/encode（R3，属
  913）；`ClosuresOwnerCache` 路径保留为 865 recipe 的观测基线。

## 声明/布局变更的显式失效边界（PLAN-914，2026-11-02，SD-06）

- **策略**：declaration/layout 变更走显式失效（collections 增量会话的
  `prepare_declaration_change`）：冷声明前端（`declaration_frontend_invalidations`
  如实计量，不伪造为零）+ 依赖精确 rebuild；字节相同 unit 保留 resident MIR/code 对象，
  只有不兼容 unit 进入 object producer（`codegen_units` 只计真实产物）。
- **冻结判据（2026-11-02 用户裁定 D-05）**：未受影响 defs 的 lower/encode/codegen/load/
  工件IO 增量为 0、同进程对象/代际不变、旧值不被新 layout 重解释；禁止用「一律拒绝任何
  type 编辑」或全量重编冒充通过。单 body 快路径只吃一个函数体，声明编辑必须走显式轮
  （body 路径的拒绝文案「use explicit declaration invalidation」即该边界）。
- **可观察面**：`declaration-invalidate` 轮上报真实 counters 与 `identities_before/after`
  （MIR/COFF 指针承载）；父侧按声明 oracle（`min_frontend_invalidations`、
  `max_codegen_units`、`stable_defs`）判定，反控（超界/把重建 def 当 stable）必须红。
- 该边界只覆盖 AC collections 记录域；字面/文本/借用族的声明失效仍在既有域内，
  此处不扩大语言承诺。
