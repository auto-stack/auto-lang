# auto-ac 函数级增量源前端（PLAN-828）

> Status: active（scalar-i32/bool profile；实现=`experimental/ac-core/src/source/*`
> + `experimental/ac-av-probe/probe/src/session.rs` 热路径）。

## 受检接口环境与单 body 检查

- 冷初始化走完整前端（`compile_scalar_source`：parse→resolve→typecheck→Atom
  render/bind→verify），产 `CheckedEnv`（驻留受检接口环境：每函数签名/owner 域/
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
