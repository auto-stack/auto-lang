# auto-syntax source front door（九 profile 统一源码前门）

> **Status**: experimental（PLAN-927 交付，PLAN-928 扩至九 profile，PLAN-929 全消费者统一+退役门；实现=`experimental/ace/crates/core/src/source/cst/`）
> 来源：PLAN-927；上游 [project.md](project.md)；规则/身份见
> [grammar-contract.md](grammar-contract.md)；树/预算不变量见 [snapshot-cst.md](snapshot-cst.md)。

## 定位

九源码 profile（Core/Scalar/Text/Record/TextRecord/EntryUnit/HostConsole/
Collections/Closures）的统一冷编译与片段前门：共享词法 + 单规则源
（`grammars/auto-source.at`，profile 裁剪编译）→ 无损 CST → 严格 AST 投影。
方向无环：`syntax-runtime ← syntax-grammar ← core::source::cst ←
syntax-core 重导出/消费者`；core 不依赖 syntax-core；runtime/grammar 不依赖
语义/旧 VM。Auto 专属投影（CST→AST、字面量解码、诊断映射）住 core；
`ace-syntax-core` 是薄重导出接缝（同类型断言，非第二份 AST）。
PLAN-928：Collections/Closures 已完成接管——`parse_program_with` 九 profile
全部路由统一 CST 链；旧手写 parser **保留为差分 oracle**（语料全集
正例 AST/反例首诊断四元组逐字节全等），不是第二实现。
collections 顶层=type/enum 声明（T-04 合同：closures 顶层无 type/enum 项）；
词域差异（`Str`→StrOwned、`char` 仅 closures）在投影镜像层冻结。

## 入口与 typed 结果

- `parse_front(FrontRequest)`：绑定 profile/能力版本（`ac-source-front/1`）/
  入口 kind（WholeModule/FreeFnFragment/MethodFragment——同一语法规则，不是
  四套实现）/显式预算（不得沿 `Budget::default()` 256MiB 冒 64MiB 档授权；
  步1前置检查，超限=资源态）。
- `FrontOutcome`：`Ok(Program+CST+grammar_id)` / `Diag(Vec)` /
  `ResourceExhausted{what,limit}`（资源态穿过投影不是源码拒绝）/ `StaleForeign`
  / `Infra`。兼容桥 `parse_program_via_cst` 保 `Result<Program, Diagnostic>`
  单诊断面；`parse_program_with` 七 profile 路由至此。
- 片段：`FrontKind::FreeFnFragment{range}` 复刻旧 free-only 合同（完整消耗、
  绝对 UTF-8 byte span、恰一自由函数）；`MethodFragment{range,owner,receiver}`
  由请求上下文恢复 owner/receiver（不拼假 free fn）。span 平移经
  `shift.rs` 全 AST 访问器。旧 `parse_function_fragment` 保持原合同不动
  （调用方 929 迁移）。孤立语法成功**不是**会话授权；revision/checkpoint
  准入 producer 由 929 接线。

## 能力清单与身份

- profile 裁剪族（与 `SourceProfile` 谓词一致）：core（`+ * <`/while/无文本面）、
  wide（全算子/一元/条件 for）、textual（str/Str/char/`.to`/`.move`）、
  recordal（type/方法/字段/ctor/路径）、entry-unit 含 **HostConsole**（脚本
  形态/尾值/console 保留名 print/eprint）。
- 语法语义变更必须换键：GrammarId=裁剪 canonical bytes hash；程序 codec v3
  （pratt `postfix_tail` 字段）；SCANNER_ABI /2（跨行 token 记结束行）、
  PARSER_ABI /2（cut 提交深度）。
- 结构化字面量/mode 词全域可解析（str/char/affix-num、move/mut 参数），拒绝
  在投影层按旧 Raw 臂合同（stage/code/span/message 逐字节）；被裁剪 token
  （`[]`/保留字）为 error 叶由结构层分类。

## 诊断保真（分层单源）

1. lex 层：str/char 分类/解码事实 + 非 ASCII=Raw(NonAscii) unsupported +
   unknown character；首个按 token 起点序。
2. 结构层：期望序列游走器按**全树后缀叶列**（跨恢复区域边界——旧 parser 无
   区域概念）镜像旧失败点；类型词/重复字段/EOF 期望/分隔纪律逐字节对齐。
3. 投影层：能力拒绝（expr-stmt 门=旧 `is_entry_unit()` 谓词、stmt-head 起始
   token 分支、ret 位 record 收任意 ident、console 保留名）。

旧 parser 怪癖保真（冻结差分即 oracle）：leading-dot 空实参调用与空 ctor 的
`continue-'outer` 重入 primary（下一 token 须能起始 primary；未括号待归约锚定
同行）。SyntaxWork 计量保真（note_tokenize/record_whole/note_parse 按投影结果
补记，不旁路）。

## pratt 后缀续读（KIND_POSTFIX）

调用收尾后操作数可继续消费点后缀步（`f().len()`、`xs.get(0).name.len()`）：
pratt `postfix: "postfix-tail"` 规则引用；运行时包装 `KIND_POSTFIX`
（孩子=操作数+tail 规则节点）；**键必须落在启用该面的 pratt 表**（wide-ops；
误落被裁剪表=门 None，`bad().len()` 静默回退——曾由差分 oracle 失明掩盖，
已修）。一元操作数同行锚定由规则步 `same-line` 承载（`Save` 携
`expr_anchor`/`same_line_pending` 随回退恢复）。

## 验证锚

`source_cst_927` 差分族（567 组合×profile + 70 冻结 edge + 片段差分 +
typed 预算/whole-path 探针；oracle=`parse_program_with_handwritten` 真旧列）；
`source_units_928(_t09)`（collections/closures 语料全集差分 162/162 + 单元
目录/增量面，见 [composite-source-projection.md](composite-source-projection.md)）。
文件化 `case-ace-source-syntax-927` 与 `case-ace-source-composite-928`
（parse+无损重建 oracle、profile 门负控 `syntax.parse`、值腿、同文 λ
oracle，经 `auto-source@<profile>` 语法名路由 case-worker/contract——两
侧语法门均九值）。固定组 `ace-source-syntax-927`。预算/深链原档见
[snapshot-cst.md](snapshot-cst.md)。

## 迁移余账（929）

全会话/持久化/父侧门与重复 parser 退役归 929：`parse_function_fragment`
旧合同保持至调用方迁移；`closures_fn` 的 `edit_body` 与单元目录
`UnitCatalog` 的会话级统一、来源恢复 store 接线、`%TEMP%` av 会话存储
清理责任由 929 裁定。

## PLAN-929：全消费者统一与退役门

- **生产重复语法清零**：全部真实 Auto 源码消费者经统一前门——常驻/暖
  面消费 `profile_grammar` 进程缓存（`CachedGrammar::program()/scanner()`
  访问器；av 常驻 session、case-worker `auto-source@<slug>` 臂同源）；
  片段生产者（free/method/closure）经 `parse_function_fragment_via_cst/
  parse_method_fragment_via_cst/parse_closure_fragment_via_cst`（方法=
  合成 type 宿主+占位字段，receiver 从语法/owner 从调用方；closure=
  合成宿主 return 位；span 有符号回绝对坐标）。
- **退役门**：旧手写整源/片段 parser 与词法器在
  `legacy-syntax-oracle` feature 后（生产构建编译期不可达；差分 oracle
  经 dev 自依赖供测试）；改名复制由 `retirement_gate_929` 结构门扫描
  （消费 crate 新 tokenize/Parser/parse_program 定义即红；不同领域语法
  白名单：atom 文本/CT deck/interface-mini 等）。
- **计量保真**：fn 片段桥按真实函数名 `note_parse`（结构门"改函恰 1、
  无关 0"对账）；整源诊断/预算/差分 oracle 面不变。

## 显式有限模块能力（PLAN-950）

原九 profile 默认面之外，`ac.source.modules.scalar/v1` 使用同一规则源的
`scalar-modules` 裁剪：公共函数、显式 use 与限定函数名投影到携带可见性/
导入/来源的 ModuleSyntax，再经模块 binder 和原 query/Checked 链消费。
不走 legacy parser，也不将模块元数据丢弃后交给普通 scalar 编译。
范围、来源迁移和恢复合同见[有限公共函数模块](finite-source-modules.md)。
