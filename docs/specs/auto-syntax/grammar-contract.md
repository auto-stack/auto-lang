# auto-syntax grammar contract（规则程序合同）

> **Status**: experimental（PLAN-787 冻结；实现=`experimental/ace/crates/syntax-grammar`）
> 来源：PLAN-787；上游模块 [project.md](project.md)；身份/codec 见
> `runtime/src/program_codec.rs`。

## DSL 面（v1）

- token matcher 五类（literal/受限字符集/重复/声明的 mode 规则）；trivia 声明。
- 步骤：sequence/choice/repeat（Once/Optional/ZeroOrMore/ListPlain/ListNewline）
  /rule/field/terminal/Pratt；same-line 换行约束步属性。
- Pratt 表 + 调用后缀配置（binding power/assoc/call_paren/callee_same_line）；
  newline/sep/sync 旋钮由语法包配置驱动（无语言分支代码）。
- PLAN-927：pratt 表新增 `postfix: <rule>` 点后缀续读引用（调用收尾后
  `.name` 步，如 `f().len()`——运行时包装 KIND_POSTFIX）；规则步级
  `same-line` 属性（一元操作数同行锚定）；`profiles:` 裁剪可标注任意
  规则/alt/pratt 键（单规则源 `grammars/auto-source.at`，compile_profile
  按 slug 裁剪，被裁 pratt 引用=None 合法、未知规则=编译错）。
- PLAN-928：profile 集扩至**九**（+collections/closures；slug 与
  `core::source::cst::profile_slug` 同集，contract/case-worker 的
  `auto-source@<slug>` 语法门同步九值）。collections 复杂声明面：
  `type`/`enum` 顶层项（enum 臂 list-nl 分隔、载荷备选**前置**——同首
  token 备选先试带括号形，纯臂由回退承接）、泛型类型位（generic-ty
  备选前置防 ident 先占）、`is` 臂（qual/payload 子规则，臂 span 恒自
  模式词起）、`for-in` 迭代器=裸 ident（合同 C7）、postfix
  `.mut`/`.move`/方法链。closures 函数面：`fn-type`、lambda（参数/表
  达式或块体/`move` 拼写/嵌套）、间接调用。
- **有序选择回退完整语义**（PLAN-928 修复）：alt 失败回退必须同时恢复
  光标**并丢弃该 alt 已建子女**——只恢复光标会令同首 token 备选重消费
  后叶重复（无损重建破坏；enum 纯臂与 postfix `.mut`/`.move` 备选实测
  暴露）。CST 无损重建（文档序叶拼接=逐字节原文）是本合同的验证
  oracle 之一（AST 差分对叶重复不敏感，不能替代）。
- 最长匹配/同长优先级显式声明；规则非前进/歧义无法裁定即编译期拒绝
  （定位错误，不静默改语法，不留半成品）。

## 身份与版本

- GrammarId = 名称 + canonical 内容 hash；schema/compiler/runtime/scanner
  四版本参与产物键；同名不同内容包隔离（跨 grammar 真读=miss，不串用）。
- canonical 二进制 codec 往返身份绑定：装载后重验（内容 hash 与键的
  grammar-id 不符=受控 miss/错误）；可按（grammar-id, revision）从 store
  装载——跨进程恢复不编译 .at 源。codec v3（PLAN-927：pratt postfix_tail
  字段）。
- 关键字拼写与运算优先级两个配置反控：不改 runtime 也必须改变实际解析
  结果（排除"Atom 仅包装硬编码 parser"）。

## 验证锚

grammar_/grammar_reject_ 族（配置反控、拒绝定位、合法兄弟可装载）；
checkpoint 完整状态合同见 [snapshot-cst.md](snapshot-cst.md)。
