# PLAN-778 r1 / SD-01 / R3 pass

Target: `docs/specs/auto-lang/frontend/overview.md` (modify). Reviewed source SHA256: `fccb811a79b24b95678efa57b87bcc9ded10e5afd2b9abc44236b99aebb16dfe`. AC-01/02/03; rationale: externally verified explicit dialect registration and reliable dispatch/recovery, while preserving the default UI AST and parser behavior.

Before: documentation describes builtin Dialect/UiDialect and recommends mem::take dispatch. After: retain builtin compatibility and document the following public interface and pilot boundaries. Replace the obsolete mem::take pitfall with Arc dispatch semantics; do not leave conflicting advice by merely appending a section.

## Proposed canonical text

方言外部化（PLAN-778 pilot）：`Dialect::try_parse_stmt/try_parse_token_stmt` 接收最小 `ParserBridge`。桥面提供 token 游标、checkpoint/restore、核心表达式/类型解析、场景与 fn/on 体上下文查询、当前位置语法错误工厂；不暴露 Parser 本体、lexer、scope、符号表或 TypeStore，不提供通用 warning 出口。

`Parser::register_dialect/with_dialect` 显式注册外部方言。`Override` 注册位于内建段之前，`Append` 位于其后；各段按注册顺序咨询，首个返回 Some 的匹配方言接管。`with_session` 只重建内建段，外部注册保持；派发时检查 matches 场景门控。Arc 共享句柄使外部委托核心解析时，合法递归语句派发仍能看见完整注册表。

派发器在 Ok(None) 后回滚 lexer/cur/prev 游标与 in_on_body/in_fn_body/in_dot_rhs/with_header/body_depth 五个上下文旗标。Err 时注册不丢失，恢复这五个旗标，游标保留失败位置；errors/warnings 单调追加。checkpoint 不承诺符号表等全部解析副作用的事务回滚。

独立外部 consumer 已验证无注解的 `model { [var] name [type] [= expr] }` 字段子集：类型和初值可同时省略；裸字段在行界/块界形成字段，类型 Unknown、初值 Nil；EOF 后仍须满足 model 块闭合。类型或初值存在时委托核心解析。字段注解走 None 回退内建；其他 UI 文法保持内建，UI AST/consumer 继续兼容。

裸字段支持属于显式注册的外部 pilot，默认内建 model 仍要求类型或初值，行为不变。生产入口和导入中新建的 Parser 不自动继承外部注册；需要各入口显式注入。widget 体内 model 由结构解析器直连内建，不经过语句方言派发；widget/msg/component/actions/style/view/on/store 的完整外部化、生产接线与 UI AST 迁移尚未完成。

Metadata: supersedes_spec_components=[docs/specs/auto-lang/frontend/overview.md]; new_spec_components=[]; touched_goals=[GOAL-007]. Merge must recheck source hash for concurrent Spec updates. This is a reviewed proposal, not a publication to canonical Specs or the live ledger.
