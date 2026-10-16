# PLAN-789 T-01 决策附件：前端依赖 DAG 与词法错误映射冻结

生成环境：worktree `D:/autostack/.wt/lang-789/auto-lang`（branch `codex/plan-789-dev`），
基面 `e4ac72a6265c85cdadeedbabc7a0c6fd93948e48`（= inputs.json 冻结基面 `3e65540cb`
之后的 master 推进，token.rs/lexer.rs/error.rs/parser_bridge.rs 四文件 SHA256 与
inputs.json 指纹逐一核对一致，词法面零漂移）。auto-down 兄弟组内 worktree
detached @ `c39ac7f`（master 基面，PLAN-726 T-04）。

## 1. V02 旧臂 golden 冻结

- 生成器：临时 `#[cfg(test)] mod zz789_dump`（已从树中清除，未提交），直接调用
  **旧** `crate::lexer::Lexer`（非新实现），逐 token 导出
  `kind(Debug)/text/line/at/pos/len`，错误导出 `code/display/debug` 投影，
  raw-block 案例走 `capture_raw_block`。
- 语料 34 例：keywords/on-header-vs-ident、Unicode（CJK 标识符、em-dash prose 跳过）、
  CRLF、注释三型+doc、str/char 转义（正/负）、f-string（$/`${}`/backtick raw/
  `set_fstr_note('#')`/`{{`转义）、数字后缀族（f/d/u/u8/i8/hex/binary/`_`）、
  算子族（含 `**`/`|>`/`?.`/`.?`）、EOF（空文件/尾换行）、`.view/.mut/.move/.take`
  属性 token、连字符标识符、`#if/#for/#is/#{/#ident/#[` comptime、c-string、
  三引号多行 str/f-str、RBrace-换行-else 吞并、UI widget 源（view/on）、
  `use pac/super`；错误例 7：E0001 坏转义、E0002 未闭合 char、E0003 空 char、
  E0005 未知字符、E0004 非法标识符起始、raw-block 未闭合（CSS 注释/块体，
  Msg 投影）。
- 冻结件：`docs/plans/attachments/789/v02-golden-frozen.json`
  （SHA256 `37beb2384a8a9f7721d1c6cc0a7a6c08440798f4d881befd91da75b32dffdc21`）。
  重放测试的期望一律取自该冻结件，抽取后新实现的输出与之逐字段比对；
  新实现不得参与期望生成（V02 oracle 界）。
- v2 再生记录（2026-10-16 同日）：v1 生成器漏写 case 输入元数据 `note`
  （fstr_note 覆盖参数，期望 token 数据本身无误），重放侧无法还原输入。
  v2 在组内临时 detached 树 `lang-789/golden-tmp`（基面 e4ac72a62 旧实现，
  用后已 guard+移除）全量重新生成：34 例期望数据（tokens/error/raw）与 v1
  逐字段一致（脚本核对零差异），仅补 `note` 元数据；冻结件与 auto-lex
  fixture 同步替换为 v2。v1 SHA256 `d23a26cf…d309` 留档作废。

## 2. 词法错误映射（Lexer 的局部错误 → 底层/上层职责切分）

| 旧路径（auto-lang） | 新路径（auto-lex） | 上层投影（auto-lang） | 身份保持 |
|---|---|---|---|
| `LexerError`（E0001..E0005 结构化五变体） | `auto_lex::error::LexerError`（逐字搬运，derive/文案/`#[diagnostic]` code 不变） | `AutoError::Lexer`（re-export 同一类型，`#[from]` 不变） | 同一类型零转换损失；code/label/span/Display 逐字一致 |
| `capture_raw_block` 的 `String.into()` → `AutoError::Msg` | `LexError::Raw(String)` | `From<LexError>` → `AutoError::Msg` | display 一致，code 均为 None（旧臂即无码） |
| `Lexer::next()` 返回 `AutoResult<Token>` | 返回 `LexResult<Token> = Result<Token, LexError>` | parser `?` 经 `From<LexError> for AutoError` | 诊断单调性/错误上限逻辑不动（在 parser 层） |

关键裁定：**auto-lang 的 `LexerError` 变为 auto-lex 同名类型的 re-export**
（`pub use auto_lex::error::LexerError;`），不保留第二份定义——"原词法代码不保留
第二份"覆盖错误枚举。`error.rs` 其余聚合器（AutoError/WithName/Msg/...）因
`ast::Name`（TypeError::CannotModifyViewParam）与全仓消费面留在 auto-lang，
**不倒灌**入 auto-lex。

## 3. 前端剩余依赖 DAG（抽取后真实状态，逐边 owner）

抽取后前端 DAG（→ = depends on）：

```
auto-lang::parser ─→ auto-lex（Lexer/LexerState/Token）        [PLAN-789 本批消除]
                  ├→ ast（Stmt/Expr/Name/...）                  [残留]
                  ├→ types（TypeStore）                          [残留]
                  ├→ infer（InferenceContext）                   [残留]
                  ├→ session（CompileSession/Scenario）          [残留]
                  ├→ module_cache/ModuleTracker（模块解析）       [残留]
                  ├→ error（AutoError 聚合 + span_from）          [残留]
                  └→ dialect/ui.rs（UiDialect 内建方言）          [残留]
auto-lang::parser_bridge ─→ parser + lexer::LexerState + token   [桥面；checkpoint 语义 PLAN-778 冻结]
auto-lang::error ─→ ast::Name（TypeError 变体）+ token::Pos      [残留；Name 边阻碍 error 整体下放]
auto-lex ─→ auto-val（AutoStr）+ strum + miette + thiserror      [本批建立；无 auto-lang/auto-man/UI 反向边]
```

| 残留边 | 下一阶段 owner |
|---|---|
| parser → ast/types/infer/session/module_cache | 下一批 UI 文法外置（完整前端外置批次；Design 37 A 波后续，778 只交付桥面不交付无环） |
| parser → dialect/ui.rs（UiDialect） | 同上——UI 方言经 ParserBridge 外置后内建方言退役/外迁 |
| error → ast::Name | 前端外置批次的 error 拆分决策（或将 Name 降为轻类型）；本批仅切口不搬 |
| ParserBridge → LexerState（不透明 checkpoint） | 保持 PLAN-778 冻结矩阵：字段不公开、恢复语义不变；789 抽取后 LexerState 换为 auto-lex 类型身份，桥面 API 面不变 |
| token.rs/lexer.rs 兼容 re-export 出口 | 789 交付面；后续批次按消费方迁移进度收窄 |

资源依赖：lexer/token 无文件系统/网络/注册表依赖（纯内存字符流）；
`fstr_note` 状态与 `capture_raw_block` 均为实例内状态，随类型迁移。

## 4. 与同批计划的文件交集复核（§4 合同执行）

- 790（clipboard files provider）：无实现文件交集（其触面为 vm/native 剪贴板族）。
- 791（auto-gen-core）：共改**根 Cargo.toml workspace members**——本批先加
  `crates/auto-lex`，791 后续串行加 `crates/auto-gen-core`；短窗口串行合流，
  无其它文件交集。
- 792（生成式 UI 独立构建）：依赖本批交付的 auto-lex 存在后再冻结新闭包；
  本批不改其触面（examples/rust-workspace、auto-man rust_ui）。

## 5. T-01 结论

词法错误可在**不引入 AST/推断依赖**的条件下保持诊断：五变体结构化错误自包含
（仅 miette SourceSpan），raw-block 字符串错误旧臂本就是 Msg 投影，无环。
§10 预设的"无法保持诊断则先修订合同"分支不触发。
