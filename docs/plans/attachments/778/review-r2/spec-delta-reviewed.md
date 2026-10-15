# SD-01 / PLAN-778 r1 / R2 partial

目标：modify docs/specs/auto-lang/frontend/overview.md，源 SHA256 见 fingerprints.json。

可沉淀规则：Dialect 经 ParserBridge 公开游标、checkpoint、expr/type、场景/上下文、语法错误工厂；Parser 本体/lexer/scope/type_store 不从 bridge 暴露。显式 Override/Append 注册按 pre→builtin→post 顺序，跨 with_session 保留；matches 派发时门控。Arc 句柄保证合法递归派发可见完整注册表；None 回滚游标与五个语句旗标，Err 恢复旗标但保留失败游标，诊断仍单调追加。保持 UI AST 兼容。

限制：仅有限外部 model 消费探针；生产/导入新 Parser 无自动注册继承，widget 内部 model 仍直连内建，其他 UI 文法尚未迁。真实已通过形态为 name type [= expr] 或 name = expr；Plan 当前 name [type] [= expr] 另承诺的裸字段未实现，F778-R2-01 闭合前不得将完整子集写入 canonical。

merge 同时移除/改写 overview 已知坑中的 mem::take 派发表建议，与现状 Arc 机制保持一致。before/after 为内建专用面→外部显式有限注册/API及可靠恢复规则；AC01/02/03。metadata 保持 supersedes_spec_components=[docs/specs/auto-lang/frontend/overview.md]，new_spec_components=[]，touched_goals=[GOAL-007]。本稿尚非 pass delta。
