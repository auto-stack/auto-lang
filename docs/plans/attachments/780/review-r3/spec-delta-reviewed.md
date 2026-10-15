# PLAN-780 R3 规范增量冻结

review_scope=final；plan_revision=1；reviewed_commit=61eea96971c1290c745da4ff74e80795834aa9cb；outcome=needs_fix；delta_status=partial。

源docs/specs/auto-man/project.md SHA256=bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3；新project-model Spec尚不存在。

| delta | 操作/目标 | 验证与阻断 |
|---|---|---|
| SD01 | add docs/specs/project-model/project.md | 独立DTO、source、重复后胜、未知children/args、RootWrapper、原终止反例已验证；注释/builtin换行兼容性受R3-01阻断，不能沉淀为完整支持 |
| SD02 | modify docs/specs/auto-man/project.md | explicit adapter、读取错误传播、生产入口不变与后续owner成立；异常清单身份仍仅后缀匹配，R3-02阻断 |

supersedes_spec_components=[docs/specs/auto-man/project.md]；new_spec_components=[docs/specs/project-model/project.md]；touched_goals=[]（本Plan为模块边界pilot，无已登记goal的变更）。

仅冻结审查结论，canonical/ledger未发布。修复后重绑提交复审；merge-owned裸cargo t及T07沉淀/合流/归档/清理pending。
