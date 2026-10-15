# PLAN-780 R2 规范增量冻结

review_scope=final；plan_revision=1；reviewed_commit=ee67ea5df5b7cf47a6eaea30bc798489e1ef6618；verdict=needs_fix；delta_status=partial。

源 `docs/specs/auto-man/project.md` SHA256=bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3。新 `docs/specs/project-model/project.md` 尚不存在。

| delta | 目标 | 本轮验证及限制 |
|---|---|---|
| SD-01 add | docs/specs/project-model/project.md | 独立DTO/consumer、source、重复后胜、未知children/raw_args保留、RootWrapper unsupported已验证；尚不能发布所有坏输入受控结束保证，F780-R2-01阻断 |
| SD-02 modify | docs/specs/auto-man/project.md | explicit adapter、read_error传播、生产入口不变与owner map已验证；不能发布required corpus异常已受限保证，F780-R2-02阻断 |

supersedes_spec_components=[docs/specs/auto-man/project.md]；new_spec_components=[docs/specs/project-model/project.md]；touched_goals=[]。

仅冻结评审结论，不写canonical Spec或ledger；后续修复提交需复审受影响面并重新绑定。merge-owned裸cargo t及沉淀/合流/归档/guard-clean均pending。
