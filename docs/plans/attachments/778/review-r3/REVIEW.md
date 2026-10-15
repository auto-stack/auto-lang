# PLAN-778 final review R3 — pass

2026-10-15；plan_revision=1；独立于实施的当前评审会话，延续 R2 上下文，未实施本 Plan。依据主检出当前合同、提交、源码和独立重放作出结论。

实施树 `D:/autostack/.wt/lang-778/auto-lang`，branch `plan-778-dev`；reviewed_commit=`da17a3d44409b8f002ed17250ae23dd867387d8a`；base_commit=`87f45dc4e2c592c31844e74824e67bf9e6cb419f`；auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。实施树及依赖干净，tracked inputs 前后 SHA256 一致，HEAD 无漂移。frontend Spec 源 SHA256=`fccb811a79b24b95678efa57b87bcc9ded10e5afd2b9abc44236b99aebb16dfe`。完整基线和锁文件指纹见 [fingerprints.json](fingerprints.json)。

## Findings closure

| Finding | 修复与独立证据 | Verdict |
|---|---|---|
| F778-R1-01 | 原 Arc 注册/递归实现未改；再次独立重放内外 model 闭包派发 accepts=2，兄弟块控制通过。 | closed |
| F778-R1-02 | 原错误出口恢复实现未改；pre-restore in_fn_body=false 断言已收编为永久 fixture，本轮再次通过；联合序列 [false,true,false] 正确。 | closed |
| F778-R1-03 | 改动文件格式与 git diff --check 通过，沿用既有未改文件格式检查证据。 | closed |
| F778-R2-01 | 外部 fixture 在 `=`/Newline/RBrace/EOF 边界省略类型；裸字段 Unknown/Nil，未改内建 parser。裸字段、换行反例翻绿；类型/初值四组合、var 组合、数字负控、内建拒绝均通过。 | closed |

本次改动仅 fixture lib.rs、R1永久组合测试与新R2边界测试。F778-R2-01 按既有合同实现，没有削弱合同或扩大默认内建文法，plan_revision=1保持适当。§9覆盖诊断已解释漏检原因，并将合同派生四象限矩阵入永久验收；满足连续两轮 needs_fix 后的 readiness 要求。

新增独立组合检查覆盖混合裸/var/typed字段、注释和CRLF行界、EOF缺右括号的失败出口、数字类型错误offset14。3/3通过，未发现修复引入的新回归或未闭合合同项。

## Acceptance / evidence

| AC | Task | Actual artifact / oracle | R3 result |
|---|---|---|---|
| AC-01 | T01..03 | bridge字段/escape hatch仅crate内、独立consumer真实构造ModelBlock，count/int/1及四组合typed断言；[external.log](external.log)、[independent.log](independent.log)、[boundary.log](boundary.log) | pass |
| AC-02 | T02..04 | 注册优先级/session/matches/实例/None回滚/Ident与token错误退出/委托递归/context pre-restore观察；公开面与派发源码核对 | pass |
| AC-03 | T04..05 | 默认UiDialect/创建点传播边界和未迁清单；[scoped.log](scoped.log)255/255，修复只在外部fixture且内建裸字段拒绝；[combinations.log](combinations.log) | pass（work/review部分）；merge-owned裸t+tv pending |
| AC-04 | T06 | 独立验收/遗漏/API/健康/Spec增量审计，commit/revision绑定及全finding闭合 | pass |
| AC-05 | T07 | Spec/ledger沉淀、合流、归档、到期回归判定、guard-clean清理 | merge-owned pending |

本轮命令与返回码见 [runs.jsonl](runs.jsonl)，可重放入口 [run.py](run.py)。外部 fixture **26/26**（12基础、6R1、8R2），独立历史组合 **7/7**，R2边界 **3/3**；补充独立组合 **3/3**。[independent.rs](independent.rs)由R1输入保留及R2更强oracle生成；新增边界输入与期待未因修复改变。

`cargo nextest run -p auto-lang --lib --features ui-iced,test-cases -E 'test(parser::tests) | test(dialect) | test(model) | test(widget)'`：**255/255**，含parser179与Plan778新4项。`cargo check -p auto-lang --locked` exit0。新增/修改fixture文件 `rustfmt --edition 2021 --check` exit0；累积实现 diff-check pass。既有410 warnings仍在未改的相关旧代码中；没有发现本次新增未处理warning，不声称全仓零warning。

补充重放：将 [combinations.rs](combinations.rs) 复制到实施树 scratch/review778-r3/tests/combinations.rs，使用fixture target，执行 `cargo test --locked --offline --manifest-path scratch/review778-r3/Cargo.toml --test combinations -- --nocapture`。独立harness从fixture锁初始化，未自动升级依赖；锁及输入hash附指纹。

旧核心API/默认派发审面沿用R2源码审计：本轮完整diff只有3个fixture文件，核心源/依赖/配置/Spec相同；仍重跑定向255、check及全部外部/历史组合。未运行裸t/tv或任何整档。

## Spec / routing

SD-01 **pass**，具体规则冻结于 [spec-delta-reviewed.md](spec-delta-reviewed.md)，hash见指纹。metadata目标路径及GOAL-007沿用已审impact，空new_spec_components表示已有frontend组件修改，不新增组件。规范明确外部裸字段与内建默认行为差异、真实接口、五旗标/游标恢复以及非事务边界、未迁生产入口/widget/完整UI文法；merge须替换现有mem::take建议并核对并发Spec源。

Plan status=reviewed，current_step=6/7，T06完成，T07未完成。Next=merge；**合入前必须在此冻结提交执行裸 `cargo t` + `cargo tv`**。未触trans/ui_gen/aavm，不要求tt/tu/taa；tf仅由merge收尾判定到期后在主检出单实例批量回归。本轮仅复审/证据/簿记，没有合入、发布canonical Specs/ledger、归档或清理worktree。review pass不等于交付完成。
