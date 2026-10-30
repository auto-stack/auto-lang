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
- 最长匹配/同长优先级显式声明；规则非前进/歧义无法裁定即编译期拒绝
  （定位错误，不静默改语法，不留半成品）。

## 身份与版本

- GrammarId = 名称 + canonical 内容 hash；schema/compiler/runtime/scanner
  四版本参与产物键；同名不同内容包隔离（跨 grammar 真读=miss，不串用）。
- canonical 二进制 codec 往返身份绑定：装载后重验（内容 hash 与键的
  grammar-id 不符=受控 miss/错误）；可按（grammar-id, revision）从 store
  装载——跨进程恢复不编译 .at 源。
- 关键字拼写与运算优先级两个配置反控：不改 runtime 也必须改变实际解析
  结果（排除"Atom 仅包装硬编码 parser"）。

## 验证锚

grammar_/grammar_reject_ 族（配置反控、拒绝定位、合法兄弟可装载）；
checkpoint 完整状态合同见 [snapshot-cst.md](snapshot-cst.md)。
