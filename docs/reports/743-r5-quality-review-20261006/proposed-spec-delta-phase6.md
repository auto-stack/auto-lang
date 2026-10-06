# PLAN-743 r6 SD-10 proposed delta（未沉淀）

Target: docs/specs/auto-acc/project.md；modify，兑现现有SD-09，不扩展HIR/语言/ACC实现。

族消费者可信边界：绑定文件不存在与hash不匹配同样视为该记录校验失败；失效记录不进入可信索引，也不贡献族覆盖。全局唯一性覆盖所有可辨识非空字符串ID的记录：即使同ID的某条记录另有类型/缺字段错误，整组仍无效，包括先前合法记录。可采用人工层验证失败后完全停止族消费的等价实现。最终CLI拒绝不能替代该内部边界。

分类准入：已有参数、导入、let/var等可观察绑定同名时，在宿主以及本地类型/enum构造或type-qualified升级之前检查冲突，身份不能证明保持unknown；参数名与本地类型名相同不能当作类型身份。没有绑定冲突的可证明本地类型和宿主正例维持原分类。仅对已有词法观察修正规则，不实现完整resolver。

原r5 SD-09正文已覆盖这些约束，SD-10只是把本轮漏过的组合边界写明。supersedes_spec_components=[docs/specs/auto-acc/project.md]；new_spec_components/touched_goals=[]。auto-hir阶段/pass无变化。canonical/live ledger在独立review pass后由merge沉淀，本次review不改写。
