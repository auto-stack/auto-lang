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

- PLAN-1046：`profile(name: "...", includes: [base-a, base-b])` 可声明组合 profile。
  带 `profiles:` 的项在标签匹配组合名或任一成员时参与编译。
  组合使用原 DSL 声明与备选顺序，成员清单提供成员关系。
  空列表、未知或重复成员、自包含及组合嵌套均拒绝。
  普通 profile 的 canonical bytes 保持各自编译结果，`aac` 有独立规则程序与身份。
- 共享扫描行为：规则程序含字面 token `..` 时，以 ASCII 数字起始的字符集匹配
  在首个相邻双点前结束。扫描器下步继续消费区间标点（含适用的 `..=`），
  单点小数保持原匹配。相关规则由实际编译的 program 决定。

## 身份与版本

- GrammarId = 名称 + canonical 内容 hash；schema/compiler/runtime/scanner
  四版本参与产物键；同名不同内容包隔离（跨 grammar 真读=miss，不串用）。
- canonical 二进制 codec 往返身份绑定：装载后重验（内容 hash 与键的
  grammar-id 不符=受控 miss/错误）；可按（grammar-id, revision）从 store
  装载——跨进程恢复不编译 .at 源。codec v3（PLAN-927：pratt postfix_tail
  字段）。
- 关键字拼写与运算优先级两个配置反控：不改 runtime 也必须改变实际解析
  结果（排除"Atom 仅包装硬编码 parser"）。

## 目录与租约恢复边界（PLAN-1029）

- CST manifest 声明目录 inline 与 paged（`DeclIndexPage`）**同等准入**：
  分页条目取出后、消费前复用 inline 同一验证——页存在、节点页归属
  （下界先判后算）、node_lo ≤ node、origin ≤ total_len；
  `decl_range`/`decl_subtree` 的 origin+start+len 全部 checked（坏目录
  受控 Admissions，不 panic、不 wrap 成合法跨度；合法恰等跨度不误拒）。
- TreeSession 租约索引有界：最后一份 `TreeLease` 释放（计数 1→0）
  **立即删除**零计数键；仍为正的键保留；旧树 entry 由下次 publish 淘汰
  （无租约才淘汰），活租约句柄跨代可读。

## 验证锚

grammar_/grammar_reject_ 族（配置反控、拒绝定位、合法兄弟可装载）；
checkpoint 完整状态合同见 [snapshot-cst.md](snapshot-cst.md)。

## 普通 Auto 的 Python 冒号切片（PLAN-1127）

`auto` 与 `auto-packages` 的下标后缀识别 `[start:stop]` 和
`[start:stop:step]`，三个槽位各可省略。CST 保留分隔冒号、槽位节点和
右方括号，槽位按普通表达式解析，内部命名实参或对象键的冒号由内部规则消耗。
普通下标 `[key]` 和范围 `[start..end]` 沿用原分支。

额外冒号和逗号轴在语法阶段拒绝。以切片作为写入 place 的文本诊断为
`source.py-slice-write`。单模块普通 Auto 的绑定要求接收者经 Python
provider 和对象来源校验，普通 Auto 值触发 `source.py-slice-receiver`。
包入口依现有 `source.use-py.surface` 拒绝 Python 导入。

来源见 [PLAN-1127](../../plans/archive/1127-ace-python-object-slice-syntax.md)。
