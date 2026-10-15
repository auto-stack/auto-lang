# PLAN-780 R5 规范增量冻结

review_scope=final；plan_revision=1；reviewed_commit=68fcc3bf768a53f5c0b1c9088c02f2165224f7b0；outcome=pass；delta_status=reviewed。

源docs/specs/auto-man/project.md SHA256=bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3；新docs/specs/project-model/project.md尚不存在。本文件冻结待merge沉淀的规则，不发布canonical Spec/ledger。

## SD-01：新增 docs/specs/project-model/project.md

### 模块与依赖

`crates/auto-project-model`为静态工程声明模型pilot，仅直接依赖auto-atom/auto-val；normal闭包不包含compiler、VM、auto-man、UI或生成器。文档层自持扫描器，结构化值层复用AtomParser；不实现完整语言求值。公开入口为`parse_document(text, SourceInfo)`与只读`parse_file(path)`。读取不执行网络、项目物化或构建；Builder/IndexStore/AutoVM不进入DTO。

### 模型与边界

ProjectDescriptor提供identity、render/legacy backend、scene/members/defines、依赖声明、展示/运行字段、toggle、external_backend及source/provenance；冻结字段的归一化对齐当前Pac语义。窗口、port、开关、render形态等按已实现DTO规则解释，展示locale/env覆盖及构建效果仍由消费层负责。

普通未知属性/节点保留在raw_props/raw_nodes；dep体保留props、重复信息和children，未知节点参数保留raw_args。重复path按末声明覆盖，仍记录重复及来源。该保留能力不等于完整源文本往返协议，规范不得额外承诺。

静态支持面为已冻结字面量属性、数组/对象、dep裸名/引号/参数形态、顶层对象文档及已验证注释/分隔形态。变量/调用/插值/赋值/f-string、显式不支持的wrapper及类型形态通过unsupported分类或stage错误暴露，非静态不计入等价；不静默执行回退。轻模型的static标志不是对全部语言语法的兼容承诺，pilot以冻结域和oracle对拍为验收依据。

### 注释、错误与已知限制

块注释内部换行只更新源码位置，不形成语法分隔；注释外真实换行与逗号按已验证容器规则处理。同一行无分隔的相邻值不能因块注释内部LF而获static标志。数组与对象的旧VM逗号规则不完全相同，不能由一类控制推导另一类。

顶层文档支持`#`行注释；当前容器值位`#`形态由旧VM拒绝，模型亦拒绝/非静态，不能宣称容器中任意`#`注释均合法。旧VM允许而AtomParser拒绝的逗号混排（例如`["a", /*c*/, "b"]`）仍在冻结子集外：模型返回值层错误，不宣称等价。这是保留限制，不要求本Plan扩张完整VM语法。

错误携带source path、stage、位置；文件读取/编码失败是Io，不以空配置代替。扫描失败出口保持消费/返回/错误进度保证；终止性控制有外部截止。EOF块注释行为保持当前legacy已验证规则，不额外宣称更严格语法。

### 后续接线归属

静态模型、compiler执行adapter及构建/物化职责分层。auto-man内部标量文本读取可后续逐入口迁移；auto-lang registry/loader接线及依赖边需另立计划。npm_deps和构建targets暂不扩为完整语义DTO，现有生产读取保留。模板渲染和动态配置留在显式上层adapter；本Plan不切换默认生产入口、不建立外仓、不宣称auto-man已脱离语言。

## SD-02：修改 docs/specs/auto-man/project.md

保留现有依赖声明门、pac字段/展示和构建协议。新增“静态项目模型pilot”说明：

- `project_model_adapter`为显式旧新探针：旧臂AutoConfig→Pac，新臂轻模型，只在冻结域比较；生产默认读取保持原路径。
- static成功逐冻结字段比较；dynamic/unsupported只分类，不计相等；读取/编码失败使required corpus门失败。
- 双拒/模板特例仅允许完整仓根相对路径精确命中的有限清单；双拒绑定预期新臂stage；其他意外拒绝必须失败调查。同名/同后缀影子不得获得豁免。
- 当前有限特例为clib模板、dynamic-expressions fixture（模板/旧臂拒绝），以及bad-missing-brace/bad-missing-value fixture（双拒，value stage），实际文件身份在adapter测试清单维护；不允许泛化为任意双臂Err均成功。
- 文本副本消除与动态管线后续按T04 owner map另立迁移计划。当前模型独立消费成立，不等于auto-man仓库拆分或生产切换完成。

## 元数据与合入条件

supersedes_spec_components=[docs/specs/auto-man/project.md]；new_spec_components=[docs/specs/project-model/project.md]；touched_goals=[]（本pilot不改变已登记goal）。SD01对应AC01/02，SD02对应AC02/03。无现有Spec冲突，无合同revision变化。

merge须先在本冻结提交执行裸cargo t；如变更代码、依赖、输入或验证配置，复核受影响审查/证据后重跑相关门禁。之后沉淀canonical/ledger、合流归档、批回归到期判定和guard-clean清理。review pass不等于已交付。
