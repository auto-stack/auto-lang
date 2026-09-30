# PLAN-714 勘定报告：tree-sitter 首批（auto-edit 供④ 承接前半）

> 2026-09-30 实勘。基线：auto-lang master@05974f71d（worktree
> `D:/autostack/.wt/lang-714/auto-lang`，分支 `plan-714-dev`）；下游只读：
> auto-edit main@70c5c60；供料原文：auto-edit `docs/upstream/
> 2026-09-m4-perf-unblock-supply.md` §4/§5（2026-09-29 落档）。
> 本报告=四勘定面（语言集/管线选型/共存策略/增量要点）+烟测实录
> +实施件契约草案。**零 crates 主线改动**（烟测 spike 隔离，见 §7）。
> 契约册（真源）= `docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md`。

## 0. 结论速览（四定案）

| 面 | 定案 | 关键证据 |
|---|---|---|
| ① 语言集 | **首批 20 语言**（21 个 grammar crate；P0=10/P1=8/P2=3），**.at 不含**（自建注记） | 围栏实勘+lang_to_extension 臂面+战略 §2.2 三源定界；crates.io 许可实勘 2026-09-30（§2） |
| ② 管线选型 | **tree-sitter 0.27 runtime + `tree-sitter-<lang>` 子 crate 族 + build-time cc 编译 + 查询用 crate 自带 highlights.scm** | spike 烟测全绿（ABI v15 兼容 v13+、parse/增量/查询三段）、21/21 grammar 自带查询、体积初值 3.41MB exe、冷构建秒级（§3） |
| ③ 共存策略 | **双轨迁移**（新 feature `highlight-treesitter`，按 lang 路由；tail 语言留 syntect）；two-face 退役=实施件终态任务（收益量级 ≈.rdata 12.4MB → installer 门富余） | 围栏现实（auto 2066+rust 2628 等）+覆盖零回退+双付期有界；019 构成表（§4） |
| ④ 增量要点 | rope 快照后台解析 + `tree.edit`/`changed_ranges` 失效域（token 级）+ big 态 plain 旁路上移不变 | spike 增量烟测（edit 标记 warm=true/cold=false；changed_ranges=[44..59)⊂[40..60)）；013 旁路实锚（§5） |

## 1. T-00 现势复核

### 1.1 highlight.rs 全貌（`crates/auto-lang/src/ui/code_editor/core/highlight.rs`）

- **单例构建**：进程级 `SyntaxSystem`（OnceLock+Mutex+Box::leak，
  highlight.rs:78-84）——`two_face::syntax::extra_no_newlines()` 全量
  语法集 + `.at` 自定义 YAML 追加（:19-41/:135-141）+ `two_face::theme::
  extra()` LazyThemeSet 主题底座（:143-146）+ AutoUI 合成主题预烘焙
  （:147-178，builtin×dark/light×accent）+ autodown hljs 主题（:179-186）。
- **调用面清单**（grep 实勘，全树仅 4 消费文件）：

| 调用点 | 消费形 |
|---|---|
| `code_editor/core/mod.rs:510` | 编辑器创建取 `syntax_system()` 挂 cosmic-text 编辑态 |
| `code_editor/core/mod.rs:523` | `warm_language(&config.lang)` 后台预热（regex 编译成本） |
| `code_editor/core/mod.rs:592` | 编辑态高亮（cosmic-text ViEditor syntect 配合面） |
| `autodown_editor/core.rs:332,346` | autodown fence：共享单例+预热 |
| `ui/iced/renderer.rs:1537` | 只读高亮 `highlight_segments(lang, code, dark, accent)`（markdown code_block/Plan 442 A6） |

- **语言入口**：`lang_to_extension`（:44-72）——`"auto"|"at"|"autolang"`
  等 22 臂 → syntect 扩展名；`"none"|"plain"|"plaintext"|""` → None（旁路）。
- **token 消费形**：syntect `HighlightLines::highlight_line` 逐行 →
  (style, seg) 区域流 → 按 (r,g,b) 合并色段（`highlight_segments` :292-337）；
  编辑态走 cosmic-text 的 syntect 配合（同单例）。

### 1.2 feature 依赖闭包（cargo tree 实勘 @05974f71d，主检出 `--locked`）

`code-editor = ["ui", "dep:cosmic-text", "dep:syntect", "dep:two-face"]`
（crates/auto-lang/Cargo.toml:69）：

| 依赖 | 锁定版 | 角色 |
|---|---|---|
| cosmic-text | 0.15.0（`features=["vi","syntect"]`） | 编辑引擎+SyntaxSystem 宿主（:255） |
| syntect | 5.3.0 | 语法引擎+高亮器 |
| onig / onig_sys | 6.5.3 / 69.9.3 | syntect regex 后端（C 依赖） |
| two-face | 0.4.5（Cargo.lock） | 语法集+主题集全量内嵌（.rdata 主项） |

### 1.3 big 态 plain 旁路实锚（013 面）

- 内核白名单：`lang_to_extension` `"none"|"plain"|"plaintext"|""` → None
  （highlight.rs:64）——高亮跳过+零 warm_language（013「懒语法」）。
- 下游绑定：auto-edit `specs/auto-edit/src/front/editor_store.at:595-602`
  ——big tab 强制 `lang_active="plain"`（SyncByteMeta）；app.at code_editor
  `lang: .store.lang_active`。**旁路语义=lang 层，先于任何引擎路由**（§5.3）。

### 1.4 plan046/047 正交性注记

- 身份实勘：auto-os `docs/plans/evidence/p047/README.md`——「PLAN-047
  证据包（memo 档 C：动态读拦截 + per-path 版本 + computed 信号网）」，
  分支 `os-047-dev`，046=其起点（「046 落地 tip」9b5a10e51）。**视图域
  （memo/keyed/依赖录制）在 auto-os 仓**。
- 正交性：高亮=文本域派生（highlight.rs 消费链全在 code_editor/core），
  memo=视图域缓存——**无路径冲突**（供料 §4 协同注记原文成立）；
  tree-sitter 增量面若消费 memo 依赖录制通道属实施件选型空间（§5.4）。
- 仓内 worktree 解析注记：lang-714 组无 auto-down 兄弟，cargo 主线
  解析在组内失败（autodown-core 路径悬挂）——勘定类 cargo 操作在主检出
  `--locked` 执行（零改动面）；spike 为独立 crate 不涉。

## 2. T-01 语言集定界（AC-01）

### 2.1 定界依据（三源）

1. **战略口径**：auto-edit 战略 §2.2「语法高亮：常见 20 语言起步（内核
   tree-sitter 化后解锁，§4.2）」（002-north-star-v2.md:225 实锚）。
2. **lang_to_extension 臂面**（现势消费契约）：auto/rust/python/js/ts/
   json/toml/yaml/markdown/html/css/c/cpp/go/java/sh/xml/sql + 透传臂。
3. **围栏实勘**（下游消费域现实，2026-09-30 grep）：
   - 本仓 docs（autodown fence 面，`highlight_segments` 消费）：
     rust 2628 / auto 2066 / bash 875 / text 212 / c 137 / json 99 /
     markdown 86 / toml 84 / typescript 81 / python 50 / yaml 46 /
     javascript 38 / at 31 / mermaid 29（裸 ``` 9508=plain）。
   - auto-edit docs/specs/perf：js 1887 / javascript 353 / bash 247 /
     sh 234 / ts 208 / html 56 / typescript 54 / json 52 / css 43 /
     vue 20 / console 19 / tsx 12 / shell 12 / auto 12。

### 2.2 首批清单定稿（20 语言 · 21 crate；crates.io 实勘 2026-09-30）

| # | 语言 | crate | 版本 | 最近发布 | 来源 | 许可 | 下载量 | 批次 |
|---|---|---|---|---|---|---|---|---|
| 1 | Rust | tree-sitter-rust | 0.24.2 | 2026-03-27 | tree-sitter 官方 | MIT | 20.4M | P0 |
| 2 | Python | tree-sitter-python | 0.25.0 | 2025-09-11 | 官方 | MIT | 16.4M | P0 |
| 3 | JavaScript | tree-sitter-javascript | 0.25.0 | 2025-09-01 | 官方 | MIT | 13.9M | P0 |
| 4 | TS/TSX | tree-sitter-typescript | 0.23.2 | 2024-11-11 | 官方 | MIT | 15.3M | P0 |
| 5 | JSON | tree-sitter-json | 0.24.8 | 2024-11-11 | 官方 | MIT | 5.2M | P0 |
| 6 | TOML | tree-sitter-toml-ng | 0.7.0 | 2024-12-03 | tree-sitter-grammars | MIT | 2.0M | P0 |
| 7 | YAML | tree-sitter-yaml | 0.7.2 | 2025-10-07 | tree-sitter-grammars | MIT | 4.5M | P0 |
| 8 | Markdown | tree-sitter-md | 0.5.3 | 2026-02-26 | tree-sitter-grammars | MIT | 1.8M | P0 |
| 9 | Bash/Shell | tree-sitter-bash | 0.25.1 | 2025-12-02 | 官方 | MIT | 14.0M | P0 |
| 10 | C | tree-sitter-c | 0.24.2 | 2026-04-22 | 官方 | MIT | 11.5M | P0 |
| 11 | HTML | tree-sitter-html | 0.23.2 | 2024-11-11 | 官方 | MIT | 4.4M | P1 |
| 12 | CSS | tree-sitter-css | 0.25.0 | 2025-09-28 | 官方 | MIT | 4.5M | P1 |
| 13 | C++ | tree-sitter-cpp | 0.23.4 | 2024-11-11 | 官方 | MIT | 11.7M | P1 |
| 14 | C# | tree-sitter-c-sharp | 0.23.5 | 2026-04-14 | 官方镜像 | MIT | 6.4M | P1 |
| 15 | Go | tree-sitter-go | 0.25.0 | 2025-08-29 | 官方 | MIT | 13.4M | P1 |
| 16 | Java | tree-sitter-java | 0.23.5 | 2024-12-21 | 官方 | MIT | 11.8M | P1 |
| 17 | SQL | tree-sitter-sequel | 0.3.11 | 2025-10-01 | derekstride（社区主流） | MIT | 1.2M | P1 |
| 18 | XML | tree-sitter-xml | 0.7.0 | 2024-11-13 | tree-sitter-grammars | MIT | 0.8M | P1 |
| 19 | Batch | tree-sitter-batch | 0.11.1 | 2026-04-21 | wharflab（社区） | MIT | 0.03M | P2 |
| 20 | PowerShell | tree-sitter-powershell | 0.26.4 | 2026-05-04 | airbus-cert（社区） | MIT | 3.6M | P2 |
| 21 | INI/Properties | tree-sitter-ini | 1.4.0 | 2025-12-08 | justinmk（社区） | **Apache-2.0** | 0.2M | P2 |

- **批次依据**（下游现实排序）：P0=本仓围栏+auto-edit 面 Top10
  （覆盖实测围栏请求 ≥95% 量）；P1=常见语言面（战略 §2.2 清单余量，
  auto-edit html/css 族）；P2=Windows 脚本族+配置族（战略口径内、
  实测频次低）。批次仅为实施排序建议，语义无差。
- **许可面**：MIT 大宗（20/21）；**唯一例外 tree-sitter-ini=
  Apache-2.0**（合规相容，引入时在 Cargo 注记标记）。
- **维护度口径**：全部 crate 最近发布 ∈ 2024-11..2026-05（无弃养件）；
  `tree-sitter-markdown`（ikatyang 2021 弃养）**不经此名引入**——用
  grammars-org 维护线 `tree-sitter-md`；TOML 同理弃 `tree-sitter-toml`
  （Mathspy 2022）用 `tree-sitter-toml-ng`；SQL 弃 `tree-sitter-sql`
  （m-novikov 0.0.2 2021）用 `tree-sitter-sequel`。
- **.at 定界：首批不含**。无现成 tree-sitter grammar（crates.io 无
  AutoLang grammar；`tree-sitter-autoit` 为同名异语言）。自建成本注记：
  grammar.js（.at 语法的 node 类型集+冲突消解）+ 增量兼容验证 + 查询
  三件套（highlights/tags/folds）——量级=一个独立实施件（715+ 可选
  任务骨架 §6.3 T-7）；过渡期 .at 臂留 syntect（双轨路由下零回退）。
- **扩展复核面（不入首批）**：vue（auto-edit 围栏 20 例）、mermaid
  （29 例，无 tree-sitter 实用 grammar）、console——留 syntect/plain。

## 3. T-02 管线选型+烟测实证（AC-02）

### 3.1 两轴判据表+定案

**轴一 crate 形态**：

| 判据 | 统一 runtime+子 crate 族（定案） | 单 crate feature 门 |
|---|---|---|
| 版本管理 | runtime 0.27 单点 + grammar 各自独立节奏（0.23-0.26 混搭实证兼容） | 20 语言 × feature 集中维护，上游节奏被单一 crate 绑架 |
| 依赖面 | `tree-sitter`+`tree-sitter-highlight`+按需 grammar crate（feature 门控） | 同左但 grammar 需自建/垫片包 |
| 生态 | 生态默认形态（Zed/nvim 同构）；21/21 首批语言有现成 crate | 需自维护 19 个垫片，无生态对齐收益 |

**轴二 grammar 分发形**：

| 判据 | build-time 编译/cc（定案） | 预编译嵌入 | 运行时加载 dll |
|---|---|---|---|
| 构建复杂度 | 低——cc 链 MSVC 实证零障碍（21/21 crate 自带 build.rs）；冷构建秒级（§3.2） | 自维护 20 语言 × 3 平台预编译产物缓存 | grammar 分发基础设施+版本对齐 |
| 体积 | 3.41MB 初值（runtime+2 grammar+highlighter 全栈，§3.2）——按需 feature 门控增量 | 同为静态链接，无体积优势 | 最小，但破坏「单 exe 无运行时依赖」（战略 §2.1） |
| 「单 exe」语义 | 保持 | 保持 | **冲突面**（供料 §4 注记成立）——排除 |

**查询来源（第三轴，烟测升格实证）**：**crate 自带 highlights.scm**——
21/21 首批 grammar 的 .crate 包均捆绑 `queries/highlights.scm`（§3.3
全量表）——零查询维护起步；查询定制属实施件选型空间。

### 3.2 spike 烟测实录（隔离目录，§7 复现）

spike=独立 scratch crate（`D:/autostack/tmp/spike-714-treesitter/`，
`[workspace]` 脱离，不入主线；源档 `docs/plans/spike-714-treesitter/`，
spike-428-bench 同形）。依赖：tree-sitter 0.27.0 + tree-sitter-highlight
0.27.0 + tree-sitter-rust 0.24.2 + tree-sitter-python 0.25.0（aliyun
sparse 镜像拉取）。

| 段 | 命令/断言 | 实测 |
|---|---|---|
| 版本兼容 | `LANGUAGE_VERSION`/`MIN_COMPATIBLE_LANGUAGE_VERSION`/grammar ABI | runtime ABI **v15**，最低接受 **v13**；rust/py grammar 均 **v15**——0.23-0.26 grammar 与 0.27 runtime 混搭零障碍（`set_language` 绿） |
| parse 烟测 | 最小 .rs/.py → 根节点 kind+函数节点存在+无 ERROR | rust：`source_file` 52 节点；python：`module` 41 节点——双绿 |
| 增量烟测 | 插入一行 → `tree.edit`+带旧树重解析 | **edit 标记**：warm 子树 `has_changes()==true`、cold `==false`（标记面挂在被编辑旧树）；**失效域**：`edited_tree.changed_ranges(&new_tree)` = 恰 1 段 `[44..59)` ⊂ 字节编辑域 `[40..60)`——**token 级差异域**（行首共享空白不计入）；微样本计时 full 45.4µs / inc 34.4µs（129B 样本，量级参考非基准） |
| 查询烟测 | 捆绑 highlights.scm 直载 → Highlighter 事件流 | rust 103 事件/10 类捕获、python 76 事件/7 类——绿；最小内联查询（正确节点名 rust=`line_comment`）类别断言 comment/string/variable 全命中——绿。**API 注记**：0.27 `highlight()` 带 `encoding: Option<u32>` 参数；`Highlight.0` 为 usize 索引入 configure 名单 |
| 体积初值 | release exe 尺寸 | **3,578,368B（3.41MB）**=runtime+highlighter+2 grammar 全栈（MSVC，opt3，未 strip）；对照 two-face 路线 .rdata 单节 12.4MB（019）——量级同段 |
| 构建复杂度 | `cargo clean && cargo build --release` | **3.0s**（Rust 侧 sccache 命中[RUSTC_WRAPPER 在效]；C 侧 cc 链真冷编译两 grammar——量级=秒级；首次构建含镜像拉取≈分钟级） |
| exit | 全段断言 | **SMOKE-OK**（exit 0） |

**烟测 API 面纪要**（0.27 形，实施件直接消费）：
`Parser::set_language(&Language)`；`tree.edit(&InputEdit{byte+point 六元})`
（**编辑点后节点 byte range 随之平移**——文本查询须用编辑后 buffer，
烟测实测踩点）；`Tree::changed_ranges(&other)` 配对语义=self=编辑旧树/
other=新树；`has_changes()`=tree.edit 标记面（新树重解析子树为新建节点
默认 false，**不作复用判据**——复用判据用 changed_ranges/节点级比对）。

### 3.3 查询捆绑全量实勘（21/21 自带 highlights.scm）

.crate 包直查（static.crates.io tarball 列目录，2026-09-30）：

| crate | 包体积 | queries 捆绑 |
|---|---|---|
| tree-sitter-rust 0.24.2 | 360KB | highlights+injections+tags |
| tree-sitter-python 0.25.0 | 175KB | highlights+tags |
| tree-sitter-javascript 0.25.0 | 147KB | highlights+highlights-jsx+highlights-params+injections+locals+tags |
| tree-sitter-typescript 0.23.2 | 810KB | highlights+locals+tags |
| tree-sitter-json 0.24.8 | 12KB | highlights |
| tree-sitter-toml-ng 0.7.0 | 22KB | highlights |
| tree-sitter-yaml 0.7.2 | 104KB | highlights |
| tree-sitter-md 0.5.3 | 317KB | highlights+injections |
| tree-sitter-html 0.23.2 | 20KB | highlights+injections |
| tree-sitter-css 0.25.0 | 50KB | highlights |
| tree-sitter-c 0.24.2 | 243KB | highlights+tags |
| tree-sitter-cpp 0.23.4 | 948KB | highlights+injections+tags |
| tree-sitter-c-sharp 0.23.5 | 1154KB | highlights+tags |
| tree-sitter-go 0.25.0 | 107KB | highlights+tags |
| tree-sitter-java 0.23.5 | 155KB | highlights+tags |
| tree-sitter-sequel 0.3.11 | 866KB | highlights+indents |
| tree-sitter-bash 0.25.1 | 428KB | highlights |
| tree-sitter-xml 0.7.0 | 73KB | xml+**dtd** 两套 highlights |
| tree-sitter-ini 1.4.0 | 18KB | highlights+folds |
| tree-sitter-powershell 0.26.4 | 241KB | highlights |
| tree-sitter-batch 0.11.1 | 62KB | highlights |

- 21/21 均 `build.rs`（cc 链）——轴二判据全量成立。
- 包体积域 12KB（json）..1.15MB（c-sharp）——产物增量量级参考。

## 4. T-03 共存策略+two-face 退役量化（AC-03）

### 4.1 双案对比+定案：**(a) 双轨迁移**

| 判据 | (a) 双轨迁移（定案） | (b) 一步切换 |
|---|---|---|
| 迁移风险 | 低——新 feature `highlight-treesitter` 并存，按 lang 路由；矩阵语法面逐语言切换逐语言验收 | 首批外语言（.at/mermaid/vue/console/text）一次性落 plain=**覆盖回退** |
| .at 命运 | 留 syntect 臂（自建 grammar 后再迁）——围栏实勘 auto 2066+at 31 例零回退 | **.at 高亮回退 plain**——产品身份语言，不可接受 |
| 体积双付期 | 有界——双付期=迁移窗（P0..P2 接入+tail 清点完成前）；two-face 退役=终态任务一次性收 | 单付即时——但以覆盖收缩为价 |
| 矩阵回归面 | 逐语言灰度，回归域=切换中的语言 | 全矩阵一次性重验 |

- **路由面设计要点**：`lang_to_extension` 层升级为 lang→引擎路由表——
  首批已迁语言→tree-sitter；tail（.at/mermaid/vue/console/未知透传臂）→
  syntect；`"none"|"plain"|""` → None（旁路，引擎无关，§5.3）。
- **主题面注记**：two-face 退役清点两处——`syntax::extra_no_newlines()`
  （语法集主项）+`theme::extra()`（主题底座，highlight.rs:144）。主题侧
  现势已高度自主（AutoUI 合成主题预烘焙+autodown hljs 映射链），退役件
  需将底座 fallback（stella 兜底链）换纯 AutoUI 合成或内嵌单主题。

### 4.2 two-face 退役收益量化（对照 019 构成表）

- 019 实锚（auto-edit `docs/plans/archived/019-m4-portable-exe-slim.md`
  +供料 §5）：portable 基线 **39,411,200B（39.4MB）**；**.rdata 数据节
  12.4MB**，主体=two-face 全量语法集内嵌（highlight.rs:135 实锚，@5bb3f53be
  复核在位）；门控组合实测 **16,459,264B（16.46MB）**，距 ≤15MB 门
  **714KB**。
- **退役收益量级**：two-face 语法集退役 ≈ 清掉 .rdata 12.4MB 主项 →
  门控组合形态 16.46MB **− ~12.4MB ≈ 4.1MB 量级**，对 15MB 门**大幅
  富余**（基线形态 39.4 → ~27MB 量级）。精确数字由实施件在真实
  build 上实测（本件为量级判定——够成 installer 预算行解锁依据）。
- **§5 want 生命周期注记**（供料 §5 协同原文承接）：供④ 实施件落地
  （双轨+退役收口）则 two-face 整体退役，子集 feature（`syntax-common`）
  的存在价值随之蒸发——**want 生命周期止于本线**；若实施件排期显著
  后移，§5 可独立承接作过渡期瘦身（两件不冲突，先后由上游排程——
  本件只注记，不并案）。

## 5. T-04 增量管线设计要点（AC-04，契约册真源节）

### 5.1 rope 快照消费形

- 解析输入=rope 快照（`code_editor/core/rope.rs` 面）——后台线程消费
  只读快照，零锁（diff 后台任务先例同款形态；`code_editor/diff/mod.rs`
  快照面在册）。解析线程独立于 UI 线程：编辑路径投递快照+编辑区间，
  产出新树+高亮段回流。
- tree-sitter 侧持有：`Tree`（Send+Sync）+ 每语言 `Highlighter` 复用
  （0.27 `Highlighter` 内嵌 Parser 可进程级复用）；语言配置
  （`HighlightConfiguration`）进程级单例（现 highlight.rs 单例纪律继承）。

### 5.2 失效域（spike 实证的 API 消费形）

1. 编辑到达 → 旧树上 `tree.edit(InputEdit)`（byte+point 六元；**tree-sitter
   平移编辑点后 range**——快照文本须与 edit 后坐标系一致）。
2. `parser.parse(&snapshot, Some(&edited_tree))` 增量重解析——未触子树
   结构复用（烟测：cold fn 未标记）。
3. `edited_tree.changed_ranges(&new_tree)` → **token 级差异域**（烟测：
   `[44..59)` ⊂ 字节域 `[40..60)`——共享前缀空白不计入）。
4. 重高亮域=changed_ranges **扩至行边界** ∪ 跨行构造的受影响尾域
   （块开合情况：变更起点的开放构造延伸到树中其闭合节点止——消费
   节点 extent 而非仅 token diff 域）；行级高亮状态从变更域之前的最近
   有效行状态续跑（tree-sitter-highlight 流式状态面，per-line state
   缓存策略属实施件选型）。
5. 全量重算兜底：变更域比例超阈（实施件定阈）或树 ERROR 传播域不可
   定界时整段重算——正确性优先于增量收益。

### 5.3 big 态 plain 旁路边界（硬边界，013 语义继承条款）

- 旁路=**lang 层语义**（`"plain"|"none"|""` → 不高亮），位于任何引擎
  路由**之前**——引擎替换不触该层。
- big tab（`editor_store.at:595-602` big→`lang_active="plain"`）：
  **零语法树构建**（tree-sitter 路径同样被旁路——big 态不做 parse、
  不建 tree、不投递解析任务）；`wrap:false`+渲染旁路臂不变。
- 回归断言面（继承 013/019 口径）：大文件装载墙钟不回退+plain 快照
  纯色断言——实施件验收项（§6.2）。

### 5.4 plan046/047 协同注记

- 高亮=文本域派生，memo/keyed/依赖录制=auto-os 视图域缓存——正交
  （§1.4 实勘）。
- 增量重高亮产物的视图域缓存（memo 通道消费）属**实施件选型空间**：
  高亮段可作 memo 依赖录制的一个 producer（文本 seq+epoch 键），是否
  接入由 715+ 依 047 基建成熟度定——本件注记不预支。

## 6. T-05 基准/验收设计+实施件契约草案（AC-05）

### 6.1 上游基准设计（供料 §4 验收建议承接）

- **正确性 fixture 族**：首批每语言≥1 金样本（含 keyword/string/
  comment/number/嵌套构造）→ tree-sitter capture 流 → 金 token 对照
  （golden 文件随语言接入入库）；双轨期逐语言与 syntect 输出做**类别
  级**对照（token 切分口径两引擎不同——对照=颜色类别覆盖，非逐 token
  等价）。
- **增量重高亮延迟档**：编辑→重高亮完成墙钟，样本阶梯 1KB/100KB/1MB；
  预算行由 715 立项时对齐 auto-edit budgets 口径后断言化（本件不定数）。

### 6.2 下游回归面（auto-edit 承担）

- **矩阵语法面**：auto-edit 矩阵语法快照断言（013 T17 族高亮旁路/
  018 谱系）——逐语言切换后复绿；tail 语言（.at/mermaid）面零扰动。
- **bench 装载墙钟不回退**：open_100mb plain 旁路域不变断言（019
  budgets 联动）；供④ 回执形态=供料 §4「下游矩阵语法面回归+bench
  大文件装载墙钟不回退」原文。

### 6.3 实施件契约草案（715+ 立项基——建议性骨架，不预支授权）

| # | 任务 | 内容 | AC 草案 |
|---|---|---|---|
| T-1 | runtime 接入 | `highlight-treesitter` feature；tree-sitter 0.27+tree-sitter-highlight；lang→引擎路由表骨架；`HighlightConfiguration` 单例 | feature 关闭=零 diff 语义（现行为逐字节等价）；开启后 tail 行为不变 |
| T-2 | P0 语言接入 | §2.2 P0 10 语言：crate 依赖+金样本 fixture+双轨切换 | 每语言 fixture 绿+矩阵该语言面绿+big/plain 旁路不变 |
| T-3 | 增量面 | §5.2 失效域管线（快照投递/changed_ranges/行边界重高亮/兜底） | 增量延迟档在档+正确性 fixture 增量路径复绿 |
| T-4 | P1 语言接入 | §2.2 P1 8 语言 | 同 T-2 形 |
| T-5 | P2+tail 清点 | §2.2 P2 3 语言；tail（.at/mermaid/vue/console）路由固化 | 同 T-2 形+tail 面回归绿 |
| T-6 | two-face 退役 | 主题底座替换（§4.1 注记）+syntect/two-face/onig 依赖摘除+依赖收口 | 产物尺寸实测（.rdata 对比）+全矩阵绿+bench 不回退+installer 预算行刷新 |
| T-7 | .at grammar 自建（可选） | grammar.js+查询三件套+增量兼容 | .at fixture 绿后 .at 臂切 tree-sitter（tail 清单减员） |

- **工期量级估计**：T-1..T-3=一件（715，M 量级）；T-4..T-5+T-6=一件
  （716，M 量级）；T-7 独立评估件（M 量级，可后置）。SD 面预定：
  715 立 `treesitter-highlight` 实施契约册（本册姊妹——勘定结论为
  before 面，实施语义为 after 面）；716 归并退役面注记。
- **冻结约束继承**：⑤big 旁路硬边界/④草案不预支授权（PLAN-714 §2
  frozen ③④⑤）——715 立项时以本件结论重起草。

## 7. spike 复现指引

```bash
# 源档（已入档，spike-428-bench 同形——构建在仓外 scratch 执行）
docs/plans/spike-714-treesitter/{Cargo.toml,src/main.rs,README.md}
# 复现
cp -r docs/plans/spike-714-treesitter /tmp/spike-714 && cd /tmp/spike-714
cargo run --release   # 输出 §3.2 表各段 + SMOKE-OK；exe 尺寸 ls -la target/release/
# 查询捆绑全量实勘（§3.3）：python 直拉 static.crates.io .crate 列目录
# 依赖闭包（§1.2）：cargo tree -p auto-lang --features code-editor --locked（主检出）
```

## 8. 证据索引

| 证据 | 位置 |
|---|---|
| 现势单例/调用面/白名单 | crates/auto-lang/src/ui/code_editor/core/highlight.rs（:44-72/:78-146/:292-337）@05974f71d |
| feature 闭包/锁定版 | crates/auto-lang/Cargo.toml:69/255-257 + Cargo.lock（syntect 5.3.0/two-face 0.4.5/onig 6.5.3/cosmic-text 0.15.0） |
| big 旁路下游实锚 | auto-edit specs/auto-edit/src/front/editor_store.at:595-602 + 013-m2-largefile-mode-v1.md |
| 体积构成 | auto-edit docs/plans/archived/019-m4-portable-exe-slim.md（39,411,200B 基线/.rdata 12.4MB/16.46MB 组合/714KB 差） |
| 046/047 身份 | auto-os docs/plans/evidence/p047/README.md（memo 档 C/os-047-dev） |
| 语言实勘 | crates.io API 2026-09-30（§2.2 表）+static.crates.io .crate 列目录（§3.3 表） |
| 烟测 | 本报告 §3.2 + docs/plans/spike-714-treesitter/（源档+README） |
| 围栏分布 | 本仓 docs + auto-edit docs/specs/perf grep（§2.1） |
