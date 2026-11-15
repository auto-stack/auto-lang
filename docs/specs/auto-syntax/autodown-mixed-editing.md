# AutoDown 混合语言增量编辑（PLAN-1058 库级原型）

> **Status**: experimental（PLAN-1058 交付；实现 =
> `experimental/ace/crates/syntax-runtime/src/autodown.rs`，测试入口 =
> 同包 `tests/autodown_mixed_editing.rs`）
> 格式参照：auto-down @ `5af11a3`（只读；`markdown_parser.at`）。
> 上游：[project.md](project.md)、[snapshot-cst.md](snapshot-cst.md)（snapshot/
> LineIndex/edit/parse 复用）；Auto 语法资产 = `experimental/ace/grammars/auto-core.at`
> （`ace-syntax-grammar` dev-dep 编译，真实 GrammarProgram/CompiledScanner）。

## 定位

AutoDown 为父层（标题/正文/围栏），围栏正文按标签选择子 parser（现装载
`auto`）。库级原型，无 UI；AutoEdit 产品接线另行决定。文档适配在语法服务
消费者侧；不依赖 AutoDown 渲染器/AutoEdit UI/ACE 编译执行——解析与查询
不求值。未装载标签（如 `ts`、任意未知标签）保留原标签与正文区间，查询仍
返回父层数据。

## 支持子集（对齐现版 parser，差异即合同）

- ATX 标题 `#{1,6}`（后随空格或行尾；尾随 ` ##` 剥离；` ^anchor` 提升，
  锚点字符集 `[A-Za-z0-9_-]` 镜像 `extractAnchorBlock`）。
- 段落 = 连续非空行合并；列表行/`key:: value` 行按段落正文处理。
- 围栏 ```/`~~~`：缩进 <4 开栏；信息串 trim = 语言标签；闭栏 = 同字符、
  同 run 长度独立行；EOF 未闭合 = 正文吞到结尾（尾部全标记字符行剥离；
  末行剥离仅空格 0x20，tab 保留；单行全空格保留）——取完整文档模式
  （isFinal=true），流式 loading 不在此层。
- 不在子集：setext 标题、thematic break、数学块、$ 组件、表格、
  frontmatter、列表结构。
- 换行归一化镜像 `normalizeNewlines`：`\r\n`/孤立 `\r` 均为行界，`\r`
  属行界不属内容；行区间直接落原文字节，围栏正文逐段 1:1 映射（无去缩进）。

## 坐标与查询契约

- 所有对外位置 = (FileId, 修订号, 半开 UTF-8 字节区间)；UTF-16 光标经
  LineIndex 双向换算（emoji 代理中点 floor）。位置句柄不是内容 NodeId；
  无出版 OccurrenceId。文档身份 = 文件 + 基线内容摘要 + 子语法 GrammarId。
- 父子边界：开/闭栏行、行界、空行属父层；正文 `[content_start, content_end)`
  属围栏语言；`content_end`（紧邻闭栏）属父层；空正文零宽内部点报围栏
  语言（显式裁定）。`language_at` 按此返回 Parent(标题/段落/空行/围栏标记)
  或 FenceBody{label, loaded}。
- 大纲仅含真实标题（围栏内"貌似标题"永不进入）；高亮 = 已装载围栏内非
  trivia 子 token（带围栏来源与错误位）；子诊断（词法+解析）按正文基址
  投影回原文坐标，只落正文区间，永不指向标记/父层字节。

## 增量与复用（每修订计量如实记录）

- 父层：编辑首行所在区域起局部重扫（其上邻段落紧邻时后退一步——段落
  吸收只向后达一行；编辑在首区域之前的前导空行区则从 0 重扫）；越过
  编辑区后，新扫描位置与旧区域起点按总位移对齐（旧区域全在未触尾部）
  即嫁接尾部；到 EOF 未对齐 = 全量重扫（记 `NoConvergence`）。
- 子层：未变围栏的子解析产物以 `Rc` 原样复用（真实子树复用，非重算）；
  重导出区域按标签重新子解析或降级为未装载。每步编辑结果须与同输入
  从头解析等价（区域/大纲/高亮/诊断；`RevisionAnalysis::equivalent_to`）。
- 计量：重扫字节区间、重导出/嫁接区域数、子树复用数、重解析围栏数、
  回退理由（NoConvergence / ForcedFull）。强制全量入口供反控验证。

## 修订发布与会话屏障

- 一次修订 = 不可变 `RevisionAnalysis`（区域/大纲/高亮/诊断一致发布）。
- 无 UI `EditorSession`：打开（新 FileId，重开即新身份，旧位置只对旧
  snapshot 有效）、编辑、定位、查询；交付屏障按 (FileId, 修订) 键控——
  过期修订结果永不覆盖（入 late 表）；同修订重复交付首个胜出（对齐
  `edit::VersionedPublish` 语义）；更新的当前修订可替换旧发布。
  值/诊断/高亮不跨文件。

## 真实例与回退条件

- 语料：`parity.ad` `hardbreak_and_fence` 分段（原例逐字节）+ Cards Probe.ad
  派生中文文档（派生例不冒充原文件）；全部经现版 parser 逐例确认
  （PLAN-1058 §9 记录 auto-down 版本与文件哈希）。
- 回退：围栏标签/终止条件变化 → 重导出受影响后继区域直到对齐或 EOF
  （吞并/恢复如实重导出）；强制全量 = 与 fresh 等价但零复用（反控区分）。
- 已知边界：多编辑 EditRequest 收敛界保守成立（未专测）；F3 tab 反例
  无专测（镜像已代码级对拍）；`DeliveryOutcome::Published`/`published`
  字段两处注释措辞滞后（语义以 `deliver()` 与本文为准）。

## 验证入口

- 定向：`cargo test --manifest-path experimental/ace/Cargo.toml -p
  ace-syntax-runtime --test autodown_mixed_editing`（V-01～06 + F1 回归）。
- 固定组：`cargo t ace-syntax`。
