# PLAN-830 r3：G829放行与实施来源交接

2026-10-22。用户说明829已完成并集成到autographics-dev，询问830能否由另一个agent实施。本轮仅核实并修订交接，没有创建agent、工作树或运行实现/测试。

## 已核实

- 829归档文件：`docs/plans/archive/829-autographics-independent-research-and-game-prototypes.md`，status archived、11/11；终收据记录五个merge checkpoint全部关闭。
- R18冻结代码：`ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74`。
- 实验集成分支：`autographics-dev`，当前固定tip=`ab56c32ec4cb28e01fbcca74e9861164ffad43f0`。
- 实际执行 `git merge-base --is-ancestor ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74 autographics-dev` 返回0。
- 实际执行 `git diff --exit-code ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74 autographics-dev -- experimental/autographics-prototype` 无差异、返回0；集成树AGP与验收冻结代码相同。
- `docs/specs/auto-graphics/project.md`已沉淀用户分支裁定：830+各自plan分支审毕后并入autographics-dev；master仅沉淀文档，实验代码不提前进master。
- prototype-profile/integration-contract/plans均已存在。当前依赖为stock iced_widget 0.14.2，不恢复早期vendored patch。
- 829旧实现树、plan-829-dev及组目录已清理；实施830不要引用已退役路径。

## 复用既有验证收据

829 [merge gates](../../829/merge-t11/gates.md)记载：冻结代码上fmt/check通过、117测试全过、十探针全过，包含真实Iced窗口/输入与consumer重定位。归档的后续landed/spec/cleaned收据覆盖此前gates.md中仍写pending的迁移放行状态：用户已经明确以实验集成分支承接，不要求829先进入master。

本轮不重跑这些测试，不冒称重新验收，也不改变829任何状态或历史记录。

## 830实施交接

- G829已满足，830合同预检结果从r2的来源blocked更新为r3 pass；仍drafting、0/10，等用户给实施agent发出work指令后由其切executing。
- 唯一实施树：`D:/autostack/.wt/lang-830/auto-lang`；分支：`plan-830-dev`；基面：上述autographics-dev固定tip。
- 主检出维护当前830 r3；该历史代码基面可能没有当前计划文件，实施agent须显式读取主检出合同并记录hash，不以基面旧稿覆盖当前计划。
- T-01先核dirty/实际SHA、源码/锁/资产指纹与current Spec，然后按T-02 whole-path可行性进入。
- 审查与冻结门通过后合回autographics-dev；不得顺手把原型合入master。真实Auto/生产UI/RQHost/Forge与AutoShader/CAP实现仍不在830本轮范围。
- 本次修订只更新来源事实、stock依赖、代码基面及集成分支；组合/输入核心、案例/验收与r2 CAP/AutoShader接缝范围保持。
