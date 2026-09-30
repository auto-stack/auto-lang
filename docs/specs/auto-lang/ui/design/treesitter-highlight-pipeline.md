# tree-sitter 高亮管线（实施契约册）

> SD-A（PLAN-716 组A 落账真源——714 勘定册姊妹篇：before=勘定结论
> `treesitter-highlight-survey.md`，after=本文实施语义）。基=
> auto-lang plan-716-dev（714 契约 T-1..T-6 承接实施）。

## 1. feature 语义（T-1）

- feature 名 `highlight-treesitter`（crates/auto-lang/Cargo.toml），
  依赖 `code-editor` + tree-sitter 0.27 / tree-sitter-highlight 0.27 +
  21 grammar crate（P0 10+P1 8+P2 3，版本表见 §2）。
- **feature 关=零语义**：ts 代码面（`core/treesitter.rs`）整体
  `#[cfg]` 门控，`highlight_segments` 直落 syntect 路径——关闭态产物
  与 syntect 基线逐字节等价（golden 守卫：highlight.rs 既有测试族在
  两 feature 态全绿）。
- **plain 旁路硬边界（013 继承）**：`lang_to_extension` None 臂
  （none/plain/plaintext/空）先于任何引擎路由——big 态零 parse、
  零语法树构建；引擎替换不触该层。

## 2. 语言路由表（T-2/T-4/T-5）

- `LANG_TABLE`（core/treesitter.rs）：21 语言 22 登记位
  （markdown + markdown-inline 注入键）。
  P0=rust/rs, python/py, javascript/js, typescript/ts, tsx, json,
  toml, yaml/yml, markdown/md(+markdown-inline), shell/sh/bash, c；
  P1=html, css, cpp/c++, csharp/cs, go, java, sql, xml；
  P2=batch/bat/cmd, powershell/ps1/pwsh, ini/properties（ini=
  Apache-2.0——许可注记在 Cargo 依赖节）。
- **tail 路由固化**：.at/mermaid/vue/console 恒不入表（syntect 臂）；
  `tail_langs_never_route` 测试为固化锚。.at 语法经 in-tree
  AUTO_SYNTAX_YAML 注册（highlight.rs）；T-7 自建 grammar 后再迁。
- 路由键大小写不敏感（与 lang_to_extension 同口径）。
- ts 管线失败（parse/查询异常）→ None 回落 syntect 基线（正确性优先）。

## 3. 查询与配色（T-2）

- **查询零 vendoring**：21/21 grammar crate 导出
  `HIGHLIGHT(S)_QUERY`/`INJECTION(S)_QUERY` 常量（714 §3.1「查询用
  crate 自带」定案全额成立）。typescript/tsx = javascript 基础查询 +
  TS 专有层**叠加拼接**（上游官方用法——TS 查询仅覆盖 TS 专有节点）。
- markdown 注入：block 查询的 `injection.language` 元数据 →
  `markdown-inline` 键路由（INLINE_LANGUAGE + HIGHLIGHT_QUERY_INLINE）。
- **配色=同主题 scope 匹配**：capture 名按前缀归类（keyword/string/
  comment/constant/type/function/tag/attribute/module/property/
  variable/operator/punctuation），每归类以优先级 scope 探针在**同一
  AutoUI 主题**（syntax_system().theme_set，(theme,dark,accent) 同键）
  上做选择器匹配取色；基前景色归一 None（与 syntect 路径同规则）。
  AutoUI 主题=9 槽位词表（keyword/storage、string、comment、
  entity.name.function/support.function、constant.numeric、
  entity.name.type 族、constant、variable 族、punctuation/meta.brace）。
- **双轨类别级对照口径**（验收）：锚点着色（金样本 fixture 每语言
  keyword/string/comment/number 锚）+同源层（ts 用色 ⊆ 同主题色板）+
  交集层（双引擎同 fixture 用色交集非空；素语种如 markdown 双方皆素
  色即平价——AutoUI 无 markup.* 槽，md 标题/粗体/行内码双引擎素色）。

## 4. 编辑面与只读面（双轨边界）

- **路由面覆盖只读三面**：markdown code_block（iced renderer
  highlight_code）、autodown fence、其余 highlight_segments 消费形。
- **编辑面（cosmic-text ViEditor/SyntaxEditor）保持 syntect**：
  ViEditor 硬绑 SyntaxEditor（syntect feature），无自定义高亮 API
  ——20 语言编辑态高亮由缩减 syntect 集承载（T-06 后=syntect
  default-syntaxes ~76 语法 + in-tree .at/TOML YAML）。ts 语法树/
  失效域供给层（§5）为编辑面切换（cosmic-text 解耦件）预留。

## 5. 增量管线（T-3）

- `IncrementalSession`（core/treesitter.rs）：rope 快照消费形
  （&str 快照+编辑域三元）。流程：旧树 `tree.edit`（InputEdit 六元；
  point 依**编辑后** buffer 坐标）→ 带旧树重解析 →
  `changed_ranges` token 级差异域 **∪ 编辑域**（等长 token 文本替换
  树结构零差——实勘修正：重高亮窗必须覆盖编辑域）→ 扩至行边界
  → `UpdateOutcome{full_reparse, changed_ranges, rehighlight_window,
  parse_elapsed}`。
- **兜底全量**：新树带 ERROR（构造开合传播不可定界）或差异域比例
  >40%（FULL_RECALC_RATIO）——正确性优先。
- 延迟档在档（debug 构建，rust fixture 编辑→重解析墙钟）：
  ~70KB full 30.6ms/inc 10.1ms；~758KB full 307ms/inc 103.5ms（≈3×）。
- 活编辑路径投递随编辑面切换件另档（本件管线层实证正确性+延迟档）。

## 6. 退役面（T-6）

- **two-face 已退役**（语法集+主题底座双清点）：底座=syntect
  default-syntaxes（~76 语法，含编辑面 fallback 词表）+default-themes
  （base16-eighties.dark bootstrap 键保持）+in-tree .at/TOML YAML。
  单例首建 477ms（debug 实测在档）。
- **onig/syntect 保留（偏差在案）**：cosmic-text 0.15 fontconfig
  (default) feature 以 default-onig 拉 syntect——特性统一强制在场
  （iced 0.14 钉 cosmic-text 0.15，外部契约）。摘除归上游解耦另档。
- **尺寸实测三态**（auto.exe release，PE .rdata）：

  | 形态 | .rdata (B) | exe 总 (B) |
  |---|---|---|
  | BASE@57b9afa60（two-face 在） | 20,284,928 | 81,998,336 |
  | 本件 ts-off（two-face 退役） | 19,655,680（−629KB） | 81,317,376 |
  | 本件 ts-on（21 grammar） | 39,063,040（+18.8MB vs BASE） | 102,394,368 |

  **installer 预算行联动注记（019 收口弹药——实测修订）**：two-face
  数据为压缩内嵌，退役实收 −629KB（019 口径「.rdata 12.4MB 主项」
  在本仓 auto.exe 实测不成立）；ts 首批 21 grammar 增量 +18.8MB
  —— `highlight-treesitter` 门控成为 installer 约束构建的**载荷
  开关**（关闭=−629KB 基线收益，开启=编辑器全批次语言面+19MB）。
  下游收口件按此表重算预算行。

## 7. 验收锚（AC 映射）

- AC-A1：feature 两态 golden（highlight.rs 测试族两态绿+tail 不变量）。
- AC-A2：fixture 矩阵 22 语言（roundtrip+锚点+三层双轨对照）。
- AC-A3：增量正确性（单点/链式 sexp 结构零漂移+兜底路径）+延迟档数字。
- AC-A4：cargo tree 断言 two-face 零匹配+尺寸三态表+全矩阵绿
  （highlight 12×2/autodown 78/code_editor 118/ts 12）+装载 477ms 在档。
