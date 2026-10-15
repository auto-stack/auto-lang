# PLAN-780 R4 规范增量冻结

review_scope=final；plan_revision=1；reviewed_commit=3ab22eeb7b50dcbdc012cc2ce0a7aede9cc5a4dc；outcome=needs_fix；delta_status=partial。

auto-man/project源SHA256=bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3；新docs/specs/project-model/project.md尚不存在。

| delta | 操作/目标 | 审查后可沉淀内容与阻断 |
|---|---|---|
| SD01 | add docs/specs/project-model/project.md | 独立DTO/source、重复后胜、未知子树/args、显式unsupported、原终止性控制成立；块注释换行兼容及无假static仍由R4-01阻断。不得宣称完整VM语法；双逗号夹注释等值层限制如实记载 |
| SD02 | modify docs/specs/auto-man/project.md | 显式adapter、read_error、生产读取未切换/后续owner成立；完整仓根相对路径异常清单与影子拒绝已独立验证。不宣称auto-man已脱离语言或静态默认切换 |

supersedes_spec_components=[docs/specs/auto-man/project.md]；new_spec_components=[docs/specs/project-model/project.md]；touched_goals=[]（边界pilot不改变已登记goal）。

未写canonical Spec/ledger；修复后重新绑定提交复审。最终pass后merge-owned裸cargo t、T07沉淀/合流/归档/批到期判定/guard-clean pending。
