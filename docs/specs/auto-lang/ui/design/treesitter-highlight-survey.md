# tree-sitter 语法高亮首批勘定契约（语言集/管线选型/共存策略/增量管线）

> PLAN-714 勘定件（2026-09-30，auto-edit 供④ 承接前半）。本册=语法高亮
> tree-sitter 化的内核侧真源——四勘定结论的契约面；烟测实录与逐项证据
> 在 [docs/plans/reports/714-treesitter-survey.md](../../../plans/reports/714-treesitter-survey.md)。
> before：语法高亮面=syntect 5/two-face 0.4.5 全量内嵌（code-editor
> feature，Cargo.toml:69/255-257；highlight.rs:135 实锚），无 tree-sitter
> 依赖，无专册。本册为 715+ 实施件的立项基；实施语义（路由表落地/增量
> 管线/退役）由实施件另立契约册承接——**本册不预支实施授权**。

## 1. 语言集表（首批 20 语言 · 21 grammar crate）

### 1.1 定界纪律（frozen）

语言集以下游消费域现实定界（防闭门造车）：战略 §2.2「常见 20 语言
起步」× `lang_to_extension` 臂面（现势消费契约）× 围栏实勘分布
（本仓 docs：rust 2628/auto 2066/bash 875/c 137/json 99/toml 84/ts 81/
py 50…；auto-edit 面：js 族主项/bash/ts/html/css）。批次 P0/P1/P2 为
实施排序建议，语义无差。

### 1.2 语言表（crates.io 实勘 2026-09-30）

| 语言 | crate | 版本 | 来源 | 许可 | 批次 |
|---|---|---|---|---|---|
| Rust | tree-sitter-rust | 0.24.2 | 官方 | MIT | P0 |
| Python | tree-sitter-python | 0.25.0 | 官方 | MIT | P0 |
| JavaScript | tree-sitter-javascript | 0.25.0 | 官方 | MIT | P0 |
| TS/TSX | tree-sitter-typescript | 0.23.2 | 官方 | MIT | P0 |
| JSON | tree-sitter-json | 0.24.8 | 官方 | MIT | P0 |
| TOML | tree-sitter-toml-ng | 0.7.0 | grammars-org | MIT | P0 |
| YAML | tree-sitter-yaml | 0.7.2 | grammars-org | MIT | P0 |
| Markdown | tree-sitter-md | 0.5.3 | grammars-org | MIT | P0 |
| Bash/Shell | tree-sitter-bash | 0.25.1 | 官方 | MIT | P0 |
| C | tree-sitter-c | 0.24.2 | 官方 | MIT | P0 |
| HTML | tree-sitter-html | 0.23.2 | 官方 | MIT | P1 |
| CSS | tree-sitter-css | 0.25.0 | 官方 | MIT | P1 |
| C++ | tree-sitter-cpp | 0.23.4 | 官方 | MIT | P1 |
| C# | tree-sitter-c-sharp | 0.23.5 | 官方镜像 | MIT | P1 |
| Go | tree-sitter-go | 0.25.0 | 官方 | MIT | P1 |
| Java | tree-sitter-java | 0.23.5 | 官方 | MIT | P1 |
| SQL | tree-sitter-sequel | 0.3.11 | derekstride | MIT | P1 |
| XML | tree-sitter-xml | 0.7.0 | grammars-org | MIT | P1 |
| Batch | tree-sitter-batch | 0.11.1 | wharflab | MIT | P2 |
| PowerShell | tree-sitter-powershell | 0.26.4 | airbus-cert | MIT | P2 |
| INI/Properties | tree-sitter-ini | 1.4.0 | justinmk | **Apache-2.0** | P2 |

- **弃养件规避**：markdown/toml/sql 用 grammars-org/derekstride 维护线
  （`tree-sitter-md`/`tree-sitter-toml-ng`/`tree-sitter-sequel`），不经
  ikatyang 2021/Mathspy 2022/m-novikov 2021 弃养名引入。
- **许可**：MIT 20/21；`tree-sitter-ini`=Apache-2.0（相容，引入时 Cargo
  注记标记）。
- **.at 不含首批**：无现成 grammar；自建=独立实施件（grammar.js+查询
  三件套+增量验证），过渡期 .at 臂留 syntect（零回退）。
- **tail 语言**：mermaid/vue/console/text 留 syntect/plain（路由表固化）。

## 2. 管线选型决策记录

| 轴 | 定案 | 判据锚 |
|---|---|---|
| crate 形态 | 统一 `tree-sitter` runtime + `tree-sitter-<lang>` 子 crate 族 | ABI 兼容实证：runtime 0.27（ABI v15，最低 v13）与 0.23-0.26 grammar 混搭零障碍；21/21 首批语言有现成 crate；生态默认形态（Zed/nvim 同构） |
| grammar 分发 | build-time 编译（cc 链） | 21/21 自带 build.rs，MSVC 工具链编译零障碍；冷构建秒级；预编译无体积优势；运行时加载与「单 exe 无运行时依赖」（战略 §2.1）冲突——排除 |
| 查询来源 | crate 自带 `queries/highlights.scm` | 21/21 捆绑实证（.crate 包直查）；零查询维护起步 |
| 体积初值 | runtime+highlighter+2 grammar 全栈 exe=3.41MB（MSVC opt3 未 strip） | spike 实测 3,578,368B；单 grammar 包 12KB-1.15MB 量级 |

## 3. 共存策略

- **定案：双轨迁移**——新 feature `highlight-treesitter` 与 code-editor
  （syntect）并存，lang→引擎路由表分流：首批已迁语言→tree-sitter；
  tail（.at/mermaid/vue/console/未知透传）→syntect；`"plain"|"none"|""`
  →不高亮（引擎无关）。判据：.at/围栏现实零回退（auto 2066 例围栏）+
  逐语言灰度回归域小+双付期有界（退役=终态一次性收口）。
- **two-face 退役**（实施件终态任务）：收益量级≈.rdata 12.4MB 主项
  （019：39.4MB 基线/16.46MB 门控组合/距门 714KB）→ 门控形态 ≈4.1MB
  量级，对 15MB 门大幅富余；精确数字实施件实测。退役清点两处：
  `syntax::extra_no_newlines()`（主项）+`theme::extra()`（主题底座——
  现势主题已高度自主[AutoUI 合成+autodown hljs 映射]，底座 fallback
  需换纯合成或内嵌单主题）。
- **供料 §5 want 生命周期**：供④ 实施件收口（双轨+退役）则
  two-face 子集 feature 的价值蒸发——**want 生命周期止于本线**；实施
  显著后移时 §5 可独立承接作过渡瘦身（不并案，上游排程）。

## 4. 增量高亮管线要点（设计约束，实施件继承）

1. **快照消费**：解析输入=rope 快照（后台线程只读，零锁——diff 后台
   任务同款；`core/rope.rs`+`diff/mod.rs` 先例）；`Tree`/`Highlighter`/
   `HighlightConfiguration` 进程级单例纪律继承 highlight.rs 现状。
2. **失效域**：`tree.edit(InputEdit)`（编辑点后 range 平移——文本坐标
   系须用编辑后 buffer）→ `parse(snapshot, Some(&edited_tree))` 增量
   重解析 → `edited_tree.changed_ranges(&new_tree)`=token 级差异域 →
   重高亮域=差异域扩行边界 ∪ 跨行构造受影响尾域（开放构造至其闭合
   节点止）。`has_changes()`=tree.edit 标记面，**不作复用判据**。
3. **big 态 plain 旁路（硬边界，013 语义继承）**：旁路=lang 层语义，
   先于引擎路由；big tab 零语法树构建（不 parse/不投递解析任务）；
   装载墙钟+plain 快照纯色断言为不回退验收面。
4. **全量兜底**：变更域比例超阈或 ERROR 传播域不可定界时整段重算
   （阈值实施件定）。
5. **047 协同**：高亮=文本域派生，与 auto-os memo/keyed/依赖录制
   （视图域）正交；增量高亮产物接 memo 通道属实施件选型空间。

## 5. 基准与验收面（供料 §4 验收建议承接）

- 上游：每语言金样本 fixture→capture 流对照（双轨期对 syntect 做
  **类别级**对照，非逐 token 等价）；增量重高亮延迟档（1KB/100KB/1MB
  阶梯，预算行 715 对齐 budgets 口径后断言化）。
- 下游（auto-edit）：矩阵语法面逐语言复绿（013 T17 族+018 谱系）；
  bench open_100mb 装载墙钟不回退（plain 域不变）。

## 6. 实施件边界（715+ 立项基）

任务骨架/AC 草案/工期量级见勘定报告 §6.3（T-1 runtime 接入→T-2 P0
接入→T-3 增量面→T-4 P1→T-5 P2+tail 清点→T-6 two-face 退役→T-7 .at
grammar 可选件；建议 715=T-1..3、716=T-4..6、T-7 独立）。冻结约束：
本册语言集表/路由边界/旁路硬边界为 715 起草的 before 面；草案不预支
授权（PLAN-714 §2 frozen ④）。
