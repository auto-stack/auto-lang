# PLAN-777 r1 / SD-01：已审查规范增量（R4 pass）

操作：modify；目标 `docs/specs/design/repository-layout.md`；源 SHA256 `3800856129eb9b5bd742fef9ea2aa0c94f0db780ab2a8ba01fc7ae8965dde49b`；AC-01/02/03。在目标文档末尾增加下面的持久规则；merge 时同步该文档的 current-state 来源及模块导航。现有布局规则保留。本附件是冻结提案，review 不发布 canonical Spec/ledger。

## 模块边界审计（PLAN-777）

`scripts/modularity-audit.py` 是模块基线与增量检查入口，配置为 `.autoos/module-boundaries.toml`，冻结基线为 `.autoos/module-boundaries-baseline.json`。从仓根使用：

```text
python scripts/modularity-audit.py --mode observe
python scripts/modularity-audit.py --mode strict
python scripts/modularity-audit.py --self-test
```

默认输出标准输出；`--output` 用于报告另存。报告包含 schema、方法、revision、文件 hash、物理行、声明边、文本候选、责任和未知范围。同输入稳定，不含时间戳或检出绝对路径。Rust规模以Git受管 `.rs` 的物理行统计，含空行/注释/测试/生成文件，不代表生产SLOC或可删除量。

声明边主源为workspace成员manifest的path依赖，区分normal/build/dev、optional与target，workspace继承path按workspace根解析。`cargo metadata --no-deps --locked --format-version 1` 作只读一致性对照；缺锁/锁不满足记录gap，不自动生成或更新，不作为编译成功证据。域表按包名登记当前成员及已知外置实现包；未登记目标保持unclassified，不声称完整生态分类。文本扫描只提供启发式候选，注释/字符串/测试/宏分类及别名、re-export、宏等盲区需读报告；不是Rust语义依赖图。

strict以当前工作树声明边对照冻结规则，未提交的新边同样在判定面内；文本候选漂移只告警。规则必须有域方向约束；没有方向约束的scope-only规则作为惰性规则报告，不应把它作为禁向拦截凭证。存量禁向边只能以完整声明身份（from/to/kind/target/optional/scope/dep_key）匹配冻结边，并具备逐边owner/exit_phase才可豁免；optional激活等身份变化不得沿用原豁免。撤销边再现、配置漂移、锚定manifest/配置变更、过期revision和宽目录豁免不得静默通过。ownership登记覆盖调研焊点、资产与consumer，必需责任组缺失在strict判红；根外consumer只登记，不能把路径核查写成完整生态验收。

`--refresh-baseline` 仅在干净树执行，绑定实际revision与配置，不能当作批准新坏依赖的替代步骤；刷新后仍需strict核对逐边责任/退出阶段。保留历史报告和输入指纹，后续迁移按已批准Plan更新边界和存量退出记录。

报告生成必须只读审计输入。输出前保护配置、基线、实际读取的manifest（含外部path目标）、全部受管文件和读取的锁；路径大小写及硬链接文件身份不能绕过保护。合法独立报告可另存；基线刷新有独立目标守卫，不能覆盖配置/manifest/源码等输入。review/merge证据只绑定实际检查的提交和输入，后续修改须重新检查。

该基线不表示AutoUI/AutoMan已拆仓；它只建立当前责任、存量债务和新增边界检查。后续ParserBridge、native provider、项目模型探针和实际迁移分别验收。

影响元数据：supersedes_spec_components=[docs/specs/design/repository-layout.md]；new_spec_components=[]；touched_goals=[]（治理基线，没有新增产品目标）。
