# auto-ac: AAC 语法、源码与模块候选

> **Status**: experimental（PLAN-1046 已交源码前端切片）
> 主体与阶段范围见 [AAC 基线合同](aac-baseline-and-subject.md)。

## 语法入口和产物身份

Auto 源码主体位于 `experimental/ace/aac/syntax/` 和 `experimental/ace/aac/source/`。
规则程序来自共享的 [grammar 合同](../auto-syntax/grammar-contract.md)，由 Gen0 生成 canonical bytes。
阶段驱动消费该程序及原始源码，词法、解析、编辑、CST/AST 投影、位置和模块候选提取由 Auto 执行。
Rust 前端继续提供独立参考。Windows x64 阶段入口已有 A2R 与真实 AC 文件宿主验证。

已内嵌 `auto`、`age-package`、`packages-ct`、`packages-modules`、
`text-record-instances`、`scalar`、`scheduler`、`aac` 八个 profile。
`aac` 组合普通、AGE 和包内编译期语法，同时保留类型修饰、泛型、标注的目标与参数，
以及条件臂和领域节点。类型、标注和值按语法边和源码范围归属其声明或字段。

## 原始字节、恢复与编辑

原始 UTF-8 字节是 CST 无损重建、AST 半开字节范围与诊断坐标的依据。
`auto` 和 `aac` 使用常规插值，`scheduler` 使用 Scheduler 插值。
规格化后的格式串子树带生成标记，映回原始分隔符、片段和表达式范围。
手写 `__fstr` 调用保留作者源码身份。位置采用一基行列、UTF-8 字节列与 UTF-16
单元列，换行和代理对中点规则沿用共享 `LineIndex`。

快照内容身份随原始 bytes 改变，撤销到同一内容恢复其身份。
逻辑文档、工程和来源定位由调用方持有。合法冷/热解析交付同一观察结果。
增量编辑可嫁接影响范围外兼容的声明，跨未闭合构造等不安全状态转冷路径。
坏 UTF-8、无效编辑、语法恢复和资源拒绝分别保留其阶段结果。
恢复树可查询且保留错误标记，严格准入拒绝有语法错误的候选。失败保留已发布的旧快照与树。

## 模块候选和下游归属

`source_catalog.catalog_directory` 按调用方 inventory 顺序分配 sourceId，
对公共文件及 `.at.at`、`.c.at`、`.rs.at`、`.py.at` 各自解析并保留所属逻辑模块。
`use`、`pub use` 和符号表导入保留原 AST 和来源，符号表按成员顺序形成逐成员路径。
`#in` 保留条件臂及其候选。公共无条件本包导入进入依赖图并检查缺失与环；
未选 provider、条件导入及外包依赖保留候选，由下游决定绑定。
可区分函数体依赖与值初始化依赖。名字绑定、条件求值、provider 选择及初始化环语义归
PLAN-1047/1048。后续代际和整机验收归 PLAN-1053/1054。

PLAN-1055 在共享条件循环规则中从组合 profile 选取一个 `cond` 表达式，保留 `body`
范围及缺条件诊断。AAC 和标量面的嵌入程序与当前 grammar 重新绑定。
组合 `aac` 面接受现有整数值实参的调用形状，实例化权威仍在后继阶段。

交付记录见 [PLAN-1046](../../plans/archive/1046-aac-syntax-source-and-modules.md)，
装配模型见 [ACE 标准库 context 装配](../stdlib/design/context-source-assembly.md)。
