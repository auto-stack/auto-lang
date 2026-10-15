# PLAN-778 r1 / SD01冻结（R1 needs_fix）

modify：docs/specs/auto-lang/frontend/overview.md。规范增量：Dialect解析入口为最小ParserBridge；外部实例显式Override/Append注册，跨with_session保持；model冻结子集在外部真实解析；其他UI语法、生产/导入新Parser注入、widget内部结构仍未迁。UI AST保持兼容。

方向与现有frontend规范/Design37一致；接口具体范围可保留，但“注册/上下文在递归与错误后可靠保持”的持久规则目前不能宣称已成立，见F778-R1-01/02。status=partial，修复后复审再冻结具体可沉淀文本；本轮不写canonical Specs/ledger。

supersedes_spec_components=[docs/specs/auto-lang/frontend/overview.md]；new_spec_components=[]；touched_goals=[GOAL-007]。源hash见fingerprints.json，merge须核对并发规范变化。
