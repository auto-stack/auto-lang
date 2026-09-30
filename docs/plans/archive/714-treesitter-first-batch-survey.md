---
plan_id: PLAN-714
status: archived               # r1+r2+r3 三阶段 delivered；r3 merge 收口 2026-09-30（archived → executing，用户指令收纳通道——712 r2 先例同款）；终态待再 merge
completion_kind: delivered    # r1 勘定件+r2 供① 残余解锁（Try 臂）+r3 route-A 深修——三阶段全 delivered
feature_name: tree-sitter 首批勘定件（auto-edit M4 供料包供④ 前半——语言集定界/管线选型+烟测/syntect 共存策略/增量高亮管线要点+实施件契约草案）
author: [agent]
created_at: 2026-09-30T16:15:39+08:00
updated_at: 2026-09-30T20:55:00+08:00
plan_revision: 3
current_step: 16
total_steps: 16
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md（SD-01：勘定契约——语言集表/选型决策记录/共存策略/增量管线要点/实施件边界）"
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（供料驱动面，703/710 先例注记式）
affects: [docs/plans/reports/714-treesitter-survey.md（新）, docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md（新）]
---

# [PLAN-714] tree-sitter 首批勘定件（auto-edit 供④ 承接前半）

## 0. 变更摘要

auto-edit **M4 解阻供料包供④**（`auto-edit/docs/upstream/
2026-09-m4-perf-unblock-supply.md` §4——tree-sitter 首批，2026-09-29
落档）的**承接前半件**：供料原文定**两段式**（「先勘定件定界[语言集/
管线选型/与 syntect 共存策略]，实施件另立」）——本件=勘定决策件。
它是 M4 长尾关键路径（同时清 M3 尾巴「语法高亮联动」与 M4 产出 4
「语法高亮首批」的唯一堵点；auto-lang 侧 705-713 均他域，供④ 至今
无人认领）。四勘定面：**①语言集定界**（常见 20 语言清单——下游
消费域现实[.at/Rust/web 族/配置族]×grammar 来源/许可/维护度表）→
**②管线选型+烟测实证**（tree-sitter crate 形态[统一 crate+feature
vs 子 crate 族]/grammar 分发形[build-time 编译 vs 预编译嵌入 vs
运行时加载]——判据=构建复杂度/体积[installer ≤15MB 联动]/增量 API
面；**镜像拉取+最小 parse 烟测**[703 imara 先例，spike 隔离不入
主线——spike-428-bench 先例]）→**③syntect/two-face 共存策略**
（双轨迁移[feature 并存按 lang 路由]vs 一步切换；two-face 退役
=installer 联动收益量化[供料档 §5 want 的生命周期止点注记]）→
**④增量高亮管线设计要点**（rope 快照消费/失效域[编辑路径增量重
高亮非全量重算——diff 后台任务先例同款形态]/big 态 plain 旁路保持/
与 plan047 memo 依赖录制协同注记）。产出=**勘定报告+实施件契约
草案**（715+ 立项基——任务骨架+AC 草案+验收设计）。**本件零
crates 改动**（烟测 spike 隔离）。

**r2 追加（2026-09-30，用户指令收纳）**：供① 残余两面解锁 phase——
auto-edit PLAN-021（L2 链解阻兑现件）fresh worktree 消费复验发现
**back 模块转译通路缺语句级 `Stmt::Try` 臂**（710 G-B 臂落点=ui_gen
front 通路，back 通路不在其覆盖）：`fsys.at:208` try（PLAN-012
find-in-files 兜底）转译失败→面一 back fsys.rs 整模块跳过→api_impl
E0432（workspace check 红）；面二 front `merged_route_a_impl` 伴生链
`.ok()?` 整体回退→api 客户端族恒空桩（env_str/read_text 死端）→
下游 L2 全预算行不可测。forensics=auto-edit
`specs/auto-edit/tests/evidence-p021-blocked-survey.md`（全档）+
供料档 §6 登记。r2 三行任务：Try 臂镜像移植+corpus 判据三行增补
（710 corpus 陈旧基面掩蔽盲区堵截）+下游 auto-edit 解阻确认收口。

**r3 追加（2026-09-30，用户指令「不需要新计划；而是继续在计划714上加
新的阶段即可」——r2 needs_replan 的有界修订兑现，收纳通道同款，不另立
新件）**：route-A 深修阶段（r2 §10 Q-R2-2 八类清单承 710 §10 延后
边界）——R3-T1 stdlib 面（fs::copy_recursive/write_bytes、re::test、
diff 三件套模块包 703 imara 引擎[code-editor 门双轨]）、R3-T2 内建表
臂+型映射（fs.metadata→file_size/copy_recursive/json.from_value
[struct-literal→json! 形]/Regex.test/File.write_bytes/list→Vec<Value>）、
R3-T3 Try 臂闭包内 return 传播（Option<Ret> 形——710 边界条款的
corpus 实实例）+preview=ln 借用、R3-T4 corpus 收口（fresh regen→
exit 0→fresh check——AC-R2-1 补全）、R3-T5 下游解阻确认（原 R2-T3
顺位承接）、R3-T6 落账。

## 1. 目标

- **G-1 语言集定界**：首批语言清单（战略 §2.2「常见 20 语言起步」
  原文）——定界依据=下游消费域现实（auto-edit tab 面语言分布+
  战略口径）候选族：.at/Auto、Rust、Python、TypeScript/JavaScript、
  JSON、TOML、YAML、Markdown、HTML、CSS、C、C++、C#、Go、Java、
  SQL、Shell/Bash、Batch/PowerShell、INI/Properties（T-01 按下游
  现实定稿 20±）；每语言=grammar 来源（tree-sitter org 官方 vs
  社区）/许可（MIT 大宗+例外标注）/维护度（最近发布/CRATE 状态）
  三列表。
- **G-2 管线选型+烟测实证**：两轴定案——**crate 形态**（统一
  `tree-sitter` runtime+`tree-sitter-<lang>` 子 crate 族[生态主流]
  vs 单 crate feature 门——判据=依赖面/版本管理/构建复杂度）；**
  grammar 分发形**（build-time 编译[cc 链，生态默认]/预编译嵌入
  [体积入 exe]/运行时加载[dll 侧车——与「单 exe 无运行时依赖」
  战略语义冲突面注记]）——判据=构建复杂度/体积[installer 联动]/
  增量 API 面（edit/tree 重算面）；**烟测实证**：镜像拉取 runtime
  +2-3 代表 grammar（.at 邻接形[Rust/Python]）+最小 parse 用例
  （parse→node 树→高亮查询烟测）——可行性+体积初值（spike 目录
  隔离，不入主线 crates）。
- **G-3 syntect/two-face 共存策略**：现状=code-editor feature 全
  syntect 5/two-face 0.4（`two_face::syntax::extra_no_newlines()`
  全量内嵌，highlight.rs:135 实锚）+cosmic-text 自带 syntect 面；
  两案：**(a) 双轨迁移**（新 feature `highlight-treesitter` 并存，
  按 lang 路由——迁移期稳、体积双付）vs **(b) 一步切换**（首批
  语言集内切 tree-sitter，其余落 plain——体积单付、覆盖面收缩）；
  定案+**two-face 退役联动量化**（.rdata 12.4MB 主项域[019 构成
  表]——退役=installer 预算行收益数字；供料档 §5 want 生命周期
  止于本线注记）。
- **G-4 增量高亮管线设计要点**：rope 快照消费（编辑路径零锁——
  diff 后台任务同款形态）；**失效域**（编辑区间→受影响节点重算
  而非全量——tree-sitter 增量 edit API 消费形）；big 态 plain
  旁路语义保持（013——懒语法/lang plain 臂不受扰的边界成文）；
  plan047（依赖录制基建）协同注记——增量面是否消费其通道属实施
  件选型空间（供料原文注记承接）。
- **G-5 基准与验收设计**：上游基准（首批语言集高亮正确性 fixture
  [每语言金样本→token 化对照]+增量重高亮延迟[编辑→重高亮完成
  墙钟]）；下游回归面（auto-edit 矩阵语法面+bench 大文件装载墙钟
  不回退[plain 旁路域不变]——供料验收建议原文承接）。
- **G-6 实施件契约草案+落账**：勘定报告（docs/plans/reports/
  714-treesitter-survey.md）+**实施件契约草案**（715+ 立项基：
  任务骨架/AC 草案/SD 面预定——按四勘定结论成形）+SD-01 勘定
  契约册+specs.json P714-1 投影。

### 非目标

- **实施件**（tree-sitter 管线落地/语言集接入/增量面——715+ 另立，
  本件零 crates 改动[烟测 spike 隔离]）。
- 供②③ 消费与 two-face 子集独立清偿（§5 want 若在供④ 前被承接
  则独立演进——本件只注记生命周期协同，不并案）。
- LSP/符号索引/大纲面（L2 线——战略 §3 裁剪清单域外）；tree-sitter
  之外的语法引擎复评（选型域已由供料定为 tree-sitter——§4.2 内核
  路线）。
- 下游消费件（auto-edit 语法面矩阵/联动——实施件 delivered 后）。
- `.at` 语法 grammar 的自建（若语言集含 .at——tree-sitter grammar
  需自写，属实施件可选任务；本件只定界「含/不含+成本注记」）。

## 2. 架构方案

分层落点（2026-09-30 实勘，auto-lang master@05974f71d）：

| 面 | 现状 | 本期形态 | 依据 |
|---|---|---|---|
| 语法高亮 | code-editor feature=syntect 5+two-face 0.4+cosmic-text（Cargo.toml:69/255-257 实锚）；highlight.rs 两态构建（:135 extra_no_newlines 全量+LazyThemeSet） | 零改动（勘定域）——现状全貌+调用面清单=T-00 勘定首项 | 供料 §4「现势=全 syntect/two-face、零 tree-sitter 依赖」 |
| 体积联动 | two-face 全量内嵌=.rdata 12.4MB 主项（019 构成表） | 退役收益量化+§5 want 生命周期注记 | 供料 §4「体积影响评估（two-face 退役=installer 联动）」 |
| 增量面 | rope 快照消费先例在册（diff 后台任务/673 find_next）；无语法增量面 | 设计要点成文（失效域/edit API 消费形/big 旁路边界） | 供料 §4「增量高亮（编辑路径增量重高亮——rope 快照同款）」 |
| 并行域 | plan046/047（memo/keyed+依赖录制）在途 | 协同注记（正交性+消费空间——供料原文） | 供料 §4 协同注记 |
| spike | spike-428-bench 先例在册（docs/plans/spike-428-bench） | 烟测隔离同形（不入主线 crates/依赖） | 仓内先例 |
| 落账 | ui/design/ 册族（703/710 SD 先例） | SD-01 勘定契约册+报告+账本 | 703/710 SD 家族 |

**关键设计约束（frozen）**：
① **勘定不实施**（零 crates 改动——烟测 spike 隔离，证据入报告
不入主线依赖）。② 语言集定界以**下游消费域现实**为准（auto-edit
tab 面分布实勘——非上游闭门造车）。③ 选型判据可复核（每判据=
实证锚[烟测数字/构成表/许可实勘]——非偏好陈述）。④ 实施件契约
草案=建议性骨架（715 立项时以本件结论重起草——草案不预支授权）。
⑤ big 态 plain 旁路保持为硬边界（013 语义零扰动——实施件继承）。

## 3. 技术栈

勘定=文档+spike 烟测（cargo scratch 项目[隔离目录]+镜像拉取
tree-sitter crate+2-3 代表 grammar+最小 parse/查询用例——703
imara 镜像先例）；下游现实实勘=auto-edit 侧只读（矩阵/会话 fixture
语言分布——零改动）；报告+契约=markdown。无主线 crates/依赖变更。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-09-30 会话指令「OK，auto-plan-new 新建
计划 021；然后再新建 auto-lang 的相关计划」——授权=**起草本件**
（供④ 承接前半——两段式的勘定段）；执行/work 待用户另行启动
（auto-lang 会话——713 稿并行在途，启动时核 714 编号与建组）。
范围=auto-lang docs/spike（零 crates 主线改动）；auto-edit 零改动
（语言分布实勘只读）。无预算/自动续跑授权。

**来源与版本**：

- 供料包：`auto-edit/docs/upstream/2026-09-m4-perf-unblock-supply.md`
  §4 供④（2026-09-29 落档，PLAN-018 T-01；同日 019 SD-05 登记
  two-face 子集 want=§5——本件 G-3 生命周期注记的对接位）。
- 战略口径：auto-edit 战略 §4.2 内核路线（「学 Zed 的三课」——
  tree-sitter 化；Zed=tree-sitter 作者的编辑器）；§2.2 M2 清单
  「语法高亮：常见 20 语言起步（内核 tree-sitter 化后解锁）」；
  §6 M4 行「语法高亮首批（tree-sitter）」；M3 收口注记尾行（
  overview M3 第五件：「剩余语法高亮联动=M4 首批 tree-sitter——
  供料驱动」）。
- 现状实勘（auto-lang master@05974f71d）：Cargo.toml code-editor
  feature 面（:69/:255-257——syntect 5/two-face 0.4/cosmic-text
  0.15[vi+syntect]）；highlight.rs（:135 extra_no_newlines 全量
  内嵌+:144 LazyThemeSet）；零 tree-sitter 依赖（全树 grep）；
  plan046/047 在途注记（memo/keyed 渲染域——正交性勘定首项）。
- 下游实勘（auto-edit main@70c5c60）：013 大文件模式 big 态 plain
  旁路语义（懒语法/lang plain）；019 构成表（two-face .rdata
  12.4MB 主项域）；installer 预算行（29.8MB 分阶段+two-face want）。
- 历史关联：PLAN-673（rope 快照消费先例）、703（imara 选型烟测
  先例+SD 家族）、710（供① 承接件——两段式供料承接的完施先例）。

## 5. 详细设计

### T-00 现势复核（决策件首项）

highlight.rs 全貌（两态构建/调用面/token 流消费形[cosmic-text 侧
配合面]）；code-editor feature 依赖闭包（cargo tree——体积贡献
分解对齐 019 构成表）；big 态 plain 臂实锚（013 面——旁路边界
成文基）；plan046/047 正交性注记。

### T-01 语言集定界（G-1）

下游现实实勘（auto-edit 会话/矩阵 fixture 语言分布+战略 §2.2 口
径）→首批清单定稿（20±）；三列表（来源/许可/维护度）——许可
实勘为准（MIT 大宗预期+例外标注[如 tree-sitter-c-sharp 等]）。

### T-02 管线选型+烟测（G-2）

两轴判据表+**spike 烟测**（隔离目录 cargo scratch：镜像拉取
runtime+2-3 代表 grammar→parse 烟测[最小 .rs/.py 样本→node 树
断言]→高亮查询烟测[queries 目录→capture 流]→体积初值[依赖闭包
编译产物尺寸]）；分发形三案对比（构建复杂度/体积/增量 API/「单
exe」语义冲突面[运行时加载案注记]）——**定案+证据**。

### T-03 共存策略+体积联动（G-3）

(a) 双轨 vs (b) 一步切换定案（判据=迁移风险[矩阵语法面回归域]/
体积双付期/覆盖面[首批外语言落 plain 的下游影响]）；two-face
退役收益量化（12.4MB 域→installer 预算行数字联动）；§5 want
生命周期注记（供④ 实施件收口则 §5 独立清偿案并档）。

### T-04 增量管线要点（G-4）

rope 快照消费形（diff 后台任务同款——Send+Sync 只读）；失效域
设计要点（编辑区间→tree.edit 增量重算——非全量）；big 态旁路
边界成文（013 语义继承条款）；047 协同注记（消费空间——实施件
选型域）。

### T-05 基准/验收设计+实施件契约草案（G-5）

上游基准设计（正确性 fixture 族[每语言金样本→token 化对照]+
增量延迟档）；下游回归面（矩阵语法面+bench 装载墙钟不回退——
plain 域不变断言）；**实施件契约草案**（715+：任务骨架[feature
落位/runtime 接入/首批语言接入/增量面/退役面]/AC 草案/SD 面预
定/工期量级估计——按四勘定结论成形）。

### T-06 落账（G-6）

SD-01 勘定契约册（docs/specs/auto-lang/ui/design/
treesitter-highlight-survey.md）+勘定报告（docs/plans/reports/
714-treesitter-survey.md——四勘定结论+证据+spike 实录）+specs.json
reviews 段 P714-1 外科插入（703/710 先例）。

### R2-T1 back 通路 Try 臂（供① 残余两面单点解锁）

落点=crates/auto-lang/src/trans/rust.rs 语句分派路径（现势 ~:12300
`_ => Err("Rust Transpiler: unsupported statement: …")`——auto-man
`api_gen::transpile_back_module_to_rs`[api_gen.rs:361，16MB 栈线程]
所走通路）。臂形**镜像 710 G-B 臂**（33a5d56c3 落点 ui_gen/rust.rs）：
`catch_unwind(AssertUnwindSafe)` 包装 try 体/Ok 丢弃/Err 进 catch 块/
`catch(e)` 绑定=panic_message 载荷串（VM catch 帧 STORE_LOCAL 同源）/
finally 跟发含 VM 侧 catch 内错误不触发 finally 的同形偏差注记。
判据：fsys.at:208 形态（空 catch 体+for{if break} 体+If 嵌套）可转译；
**单臂两面全解**=①back fsys.rs 恢复发射→workspace `cargo check` 过；
②front route-A 伴生嵌入恢复（merged_route_a_impl[auto-man/src/
rust_ui.rs:903]不再 `.ok()?` 中止）→api 客户端实体形（env_str/read_text
非 D-7 桩——`String::new()` 恒返形消失）。

### R2-T2 corpus 判据三行增补（710 corpus 盲区堵截）

落点=plan710 census 判定面（p710-census.md 判据集+corpus 跑批脚木
面）。三行：①「⚠ … transpile failed (module skipped)」grep 零命中
（缺模块洞机读暴露——710 实录该警告行在案但不在判定集）；②
fresh-copy 卫生（corpus regen 前清 rust-workspace 或生成文件清单
diff——堵陈旧基面掩蔽：710 corpus「cargo check 过」实为 tmp 拷贝
携带旧代 fsys.rs 满足导入的假绿，forensics §③ 实录）；③api 客户端
桩形检测（`String::new()` 恒返形 grep——面二的编译绿假阴性堵截）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md | before：语法高亮面=syntect/two-face 现状无专册（feature 面散于 Cargo 注记）；tree-sitter 化无契约 / after：勘定契约册——语言集表（20±×来源/许可/维护度）/管线选型决策记录（crate 形态+分发形+烟测证据）/共存策略（双轨 vs 切换定案+two-face 退役联动量化）/增量管线要点（快照消费/失效域/big 旁路边界）/实施件边界（715+ 契约指向） | 供④ 前半落账；实施件的内核侧真源 | AC-01..05 |

## 6. 测试设计

- **spike 烟测**（隔离，不入主线）：parse 烟测（2-3 语言最小样本
  →node 树断言）+高亮查询烟测（capture 流非空+类别抽样）+体积
  初值（依赖闭包产物尺寸记录）——证据入勘定报告。
- **勘定可复核性**：每判据=实证锚（许可=crates.io/仓库实勘；体积
  =构成表/spike 数字；正交性=调用面 grep）——复审可逐项重放。
- **主线零改动断言**：crates/ 与 Cargo.toml 零 diff（spike 目录
  隔离+gitignore 或不入 git——spike-428-bench 入档形态沿用的则
  入档）。
- **无运行时面**（勘定件零测试面新增——报告+册为交付物，审校=
  逐判据证据核对）。

## 7. 验收标准

- **AC-01 语言集表**：首批清单定稿+三列表（来源/许可/维护度——
  许可逐项实勘非假设）。验证：勘定报告 §语言集+表在档。
- **AC-02 选型决策记录**：两轴定案+判据表+spike 烟测实证（parse/
  查询绿+体积初值）。验证：报告 §选型+spike 实录（命令可复现）。
- **AC-03 共存策略定案**：双轨/切换定案+判据+two-face 退役量化
  数字+§5 生命周期注记。验证：报告 §共存+数字对照 019 构成表。
- **AC-04 增量管线要点**：快照消费/失效域/big 旁路边界/047 协同
  注记成文。验证：SD-01 册对应节在档。
- **AC-05 实施件契约草案**：715+ 骨架（任务/AC/SD 面/工期量级）
  在档。验证：报告 §实施件契约。
- **AC-06 落账+主线零改动**：SD-01 册+P714-1 投影回读 True+
  crates/Cargo.toml 零 diff。验证：账本断言+git diff 路径断言。

- **AC-R2-1 Try 臂两面绿**：`auto build -r rust` exit 0 且 a2r 日志零
  「transpile failed (module skipped)」警告行+**fresh workspace**
  （regen 前清生成区）`cargo check --workspace` 过。验证：regen 日志
  +check 收据。
- **AC-R2-2 front route-A 恢复**：front 生成物含伴生嵌入（env_str/
  read_text 实体形非 D-7 桩）。验证：生成物 grep 断言。
- **AC-R2-3 单测面**：Try 臂构造用例（成功/异常两臂+空 catch 体+
  for{if break} 嵌套——fsys.at:208 同形）绿；tu/tf 门零回退。验证：
  cargo tu/tf 收据。
- **AC-R2-4 下游解阻确认**：auto-edit 仓 `perf.py a2r` 三重判据全绿
  （下游 PLAN-021 复验位——其 L2 判定谱补跑[bench --l2 三档+smoke_gen
  三域，判据面已备]归 PLAN-021 承接，本件验收含解阻确认单）。验证：
  下游复验收据（跨仓）。
- **AC-R3-1 stdlib 面在位**：a2r_std fs::copy_recursive/write_bytes、
  re::test、diff 三件套（code-editor 门双轨——feature 关=panic 同 shim
  fallback 形）编译+单测绿。验证：cargo t a2r_std 族。
- **AC-R3-2 内建臂发射形**：新臂单测绿（fs.metadata→file_size 等六臂
  +list 映射）。验证：plan710_supply_probes 增测。
- **AC-R3-3 Try 传播形**：return-in-try 两 E0308 形单测转绿（Some 传播
  +Default 尾臂）。验证：同上。
- **AC-R3-4 落账**：census §7 更新+specs.json P714-3 回读 True。验证：
  账本断言。


## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 0 | T-00 现势复核 | — | highlight.rs+Cargo feature 面+046/047 注记 | 全貌+正交性基 | AC-03/04 | [x] 勘定记录在档——报告 §1（调用面清单 4 文件/闭包 syntect 5.3.0+two-face 0.4.5+onig 6.5.3/big 旁路 editor_store.at:595-602 实锚/046-047=auto-os memo 档 C） |
| 1 | T-01 语言集定界 | T-00 | 报告 §语言集（下游实勘只读） | 首批清单+三列表 | AC-01 | [x] 表在档（许可实勘）——报告 §2（20 语言/21 crate 三源定界+crates.io 2026-09-30 实勘+围栏分布；ini=Apache-2.0 唯一例外；.at 不含自建注记） |
| 2 | T-02 选型+烟测 | T-00 | spike 隔离目录+报告 §选型 | 两轴定案+实证 | AC-02 | [x] 烟测绿+体积初值在档——报告 §3（SMOKE-OK：ABI v15 兼容 v13+/parse 双绿/增量 changed_ranges=[44..59)/查询捆绑 21/21 实证/exe 3,618,304B[归档终版源码]/冷构建 3.0s） |
| 3 | T-03 共存策略 | T-01/02 | 报告 §共存 | 定案+退役量化 | AC-03 | [x] 数字对照在档——报告 §4（双轨定案+退役≈12.4MB .rdata→16.46MB≈4.1MB 门富余对照 019+§5 want 生命周期止于本线注记） |
| 4 | T-04 增量要点 | T-00 | SD-01 册 §增量 | 管线设计要点 | AC-04 | [x] 册节在档——SD-01 §4（快照消费/失效域含 spike API 纪要/big 旁路硬边界/047 协同注记） |
| 5 | T-05 契约草案 | T-01..04 | 报告 §实施件契约 | 715+ 立项基 | AC-05 | [x] 骨架在档——报告 §6（T-1..T-7 任务骨架+AC 草案+715/716 拆分+工期量级+基准/验收设计） |
| 6 | T-06 落账 | T-01..05 | SD-01+报告+specs.json | 勘定收口 | AC-06 | [x] P714-1 True+零 diff 断言——specs.json reviews P714-1（外科插入：roundtrip 字节等价 1,183,344B 先证+五段零扰动回读+reinsert stable；git diff master crates/+Cargo.toml=0 行） |

| 7 | R2-T1 back 通路 Try 臂 | — | trans/rust.rs 语句分派（~:12300）+单测 | 供① 残余两面单点解锁 | AC-R2-1/2/3 | [x] 臂落+单测三绿@4534b5004（plan714_back_try_arm_* 入 plan710_supply_probes）+零 skip 警告+front 实体形（AC-R2-2 实质达成）；**AC-R2-1 部分**：exit 0/cargo check 未达——fresh fsys.rs 暴露掩蔽层 26 错/8 类（route-A 深修=710 §10 延后面）→ needs_replan（§10 Q-R2-2） |
| 8 | R2-T2 corpus 判据三行 | R2-T1 | p710 census 判定面 | 盲区堵截（掩蔽/桩形/skip 警告） | AC-R2-1 | [x] 判定集三行入 p710-census.md §7@4534b5004（①skip 0 命中②fresh 卫生③实体形非桩=AC-R2-2 实证）；corpus 复跑=fresh fsys.rs 生成，掩蔽层 26 错暴露在档 |
| 9 | R2-T3 下游解阻确认+落账 | R2-T1/2 | auto-edit 复验位+specs.json | 跨仓收口 | AC-R2-4 | [ ] 下游 a2r 三重判据全绿+P714-2 投影回读 True |
| 10 | R3-T1 stdlib 面 | — | a2r_std.rs（fs/re/+新 diff 模块） | copy_recursive/write_bytes/re::test/diff 三件套（703 引擎包络，code-editor 门双轨） | AC-R3-1 | [x] 五 fns+diff 模块落 a2r_std@96e848ea5（code-editor 门双轨=shim 同 panic 文案）；fsys.at 全文直转探针限定断言绿 |
| 11 | R3-T2 内建表臂+型映射 | R3-T1 | trans/rust.rs Dot 表+rust_type_name | fs.metadata→file_size/copy_recursive/json.from_value[json! 形]/Regex.test/File.write_bytes/list→Vec<Value> | AC-R3-2 | [x] 三处分发器（Bina 元组/两段 ns/Dot method_name）齐装@96e848ea5+3574b47c8 前身（Key 枚举四型引号修 (name X) 错键/list=Vec<Value> 别名 glob 解析/裸名 diff 三件+Regex.replace/remove_dir/list_dir→walk/to_value→parse_str_list）；fsys 直转探针绿 |
| 12 | R3-T3 Try return 传播+借用 | R3-T2 | trans/rust.rs Try 臂 | 闭包 Option<Ret> 形（Some 改写/Default 尾臂）+preview=ln clone | AC-R3-3 | [x] 传播形（体含 return 门控+Some AST 改写+Ok(Some(v))=>return v）+借位 clone 窄门（store 声明+String 族型门）@96e848ea5；传播形/unit 保持/borrow clone 三单测绿 |
| 13 | R3-T4 corpus 收口 | R3-T1..3 | 组内 tmp corpus | fresh regen→exit 0→fresh check（AC-R2-1 补全） | AC-R2-1 | [x] fresh regen **exit 0**+工作区 check 过（1m07s）+skip=0+实体形在位——**AC-R2-1 补全**（r3f 轮；迭代谱 r2b 26 错→r3d 16→r3e 3→r3f 0 在档 a2r-r3*-run.log） |
| 14 | R3-T5 下游解阻确认（原 R2-T3 承接） | R3-T4 | auto-edit perf.py a2r | 跨仓收口（021 复验位） | AC-R2-4 | [x] 下游真仓 perf.py a2r 绿（AUTO_BIN+AUTO_LANG_CRATE 钉 worktree 工具链；deps 预置真实拷贝修复半拉 materialize 态）+三占位 0/0/0+fresh check 59.82s 过——**AC-R2-4 交付，PLAN-021 解阻** |
| 15 | R3-T6 落账 | R3-T4/5 | census+specs.json+计划 | census §7 更新+P714-3 投影+完态 | AC-R3-4 | [x] census §7 r3 收口判定@211ef5237+P714-3 投影（master 外科插入）回读 True——本行即完态 |

## 9. 复审记录

- 2026-09-30 起草 handoff：`stage: new`，PLAN-714，plan_revision 1。
  `outcome: pass`（起草完备：供④ 两段式原文承接[勘定段]——四勘
  定面+烟测实证设计+主线零改动 frozen；语言集以下游现实定界的
  纪律防闭门造车；实施件契约为建议性骨架不预支授权；路径/符号
  经 auto-lang@05974f71d 与 auto-edit 供料档/构成表三源锚定；
  授权=起草[用户指令原文在录]，执行待用户启动——auto-lang 会话，
  注意 713 稿并行协调）。`next: work`。

- 2026-09-30 work handoff：`stage: work`，PLAN-714，plan_revision 1。
  `outcome: pass`。`code_commit`: plan-714-dev 9e935f423（报告
  docs/plans/reports/714-treesitter-survey.md+SD-01 册
  docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md
  +spike 源档 docs/plans/spike-714-treesitter/；基线 master@05974f71d，
  worktree D:/autostack/.wt/lang-714/auto-lang）。`task_ids`: T-00..T-06
  全账（7/7，§8 表逐项证据在列）。`evidence`: 四定案（语言集 20/21
  crate 三源定界+许可实勘[ini=Apache-2.0 唯一例外]；管线=0.27 runtime
  +子 crate 族+build-time cc+自带查询 21/21 实证；双轨迁移+two-face
  退役量化≈12.4MB→门富余；增量要点含 spike API 纪要[tree.edit 平移
  range/has_changes=标记面]）；spike SMOKE-OK（parse/增量/查询三段
  绿+体积初值 3.41MB+冷构建 3.0s）；AC-06 零 diff 断言
  （git diff master crates/+Cargo.toml=0 行）+specs.json P714-1 外科
  插入（roundtrip 字节等价先证+五段零扰动+reinsert stable）。
  Category A 门禁：零 crates 改动、无测试面新增，cargo t/docs_gen 免。
  `blockers`: 无（Q-1 按默认下游现实定界执行；Q-2 烟测定案
  build-time cc；Q-3 实施件排期待用户）。`next: review`。

- 2026-09-30 review handoff：`stage: review`，PLAN-714，plan_revision 1。
  `outcome: pass`（含修复环 1 轮）。`reviewed_commit`: plan-714-dev
  2530e6d6b（终版=work 9e935f423+修复 F-1/F-2）；`base_commit`:
  master@05974f71d（merge-base；执行窗内 master 并行前进至 1f25465e7
  [711 簿记/712 收纳]——与本案文件零交叠实证：git diff master...dev
  仅 5 件本案交付物）。`dependency_revisions`: auto-edit main@70c5c60
  （只读实勘面）/auto-os p047 evidence（046-047 身份锚）。
  `spec_inputs`: SD-01 册（worktree 终版）+供料档 §4/§5+019 构成表。
  `acceptance_results`: AC-01 pass（许可三项抽查复跑 crates.io API：
  ini=Apache-2.0/rust=MIT/md=tree-sitter-grammars 全中）；AC-02 pass
  （spike 复跑 SMOKE-OK）；AC-03 pass（019 数字 grep 4 处锚点吻合）；
  AC-04 pass（SD-01 §4 在档 :82）；AC-05 pass（报告 §6.3 在档 :322）；
  AC-06 pass（crates/+Cargo.toml/Cargo.lock 对 merge-base 零 diff=0 行
  +specs.json P714-1 回读 True+INDEX 再生 no-op）。`findings`:
  F-1（低，已修）exe 体积收据陈旧——旧引 3,578,368B 为高亮段重写前
  构建值，归档终版源码复跑实测 3,618,304B（+40KB，量级结论 3.4-3.5MB
  不变）——报告 §0/§3.1/§3.2+README+计划 §8+账本 P714-1 六处对齐
  （账本外科再编辑：roundtrip 守卫+单行 ±）；F-2（低，已修）SD-01 册
  报告链接相对路径少一级（../../../ → ../../../../，ls 解析断言过）。
  `evidence`: 复审独立声明（实施会话内复审——判定从工件重建：烟测
  复跑/许可 API 抽查/数字 grep/diff 断言/readback 全部本会话重放）；
  修复提交 2530e6d6b（3 文件 ±5 行）；Category A 门禁维持（零 crates
  改动，无测试面）。`next: merge`。

- 2026-09-30 merge 收据：`stage: merge`，PLAN-714:r1，
  `completion_kind: delivered`。**prepared**：reviewed 基线 2530e6d6b
  （pass，AC-01..06 复放在案）；canonical delta=SD-01 add（
  docs/specs/auto-lang/ui/design/treesitter-highlight-survey.md，分支
  内已备）；projection target=specs.json designs 段新项 P714-2
  （file+docsha+commit 标签）；归档目标 docs/plans/archive/。
  **landed**：rebase master 后 range-diff 双等（9e935f423→0697929b6、
  2530e6d6b→41ace6107，补丁全等证明）；ff-only 合并零合并提交，
  master tip=delivery 41ace6107 实证（5 文件 867 行）；docs-only
  集成烟=INDEX 再生 no-op。**ledger_refreshed**：designs 段 P714-2
  外科插入（roundtrip 字节等价先证+五段零扰动 designs 116→117+回读
  True，docsha 5e7c2fa187fee8ef）@ ad95f78e7；INDEX 再生 no-op 复核；
  plans.md/overview 行回写按 710 先例不适用（勘定件无行为面——
  710/711/712 均无行实证）。**archived**：git mv 至 archive/+
  status: archived+completion_kind: delivered（本节收据）。批量回归
  到期判定=**不到期**（.last-batch-regression.json last_covered=712
  @7fcf913eb，收据 2026-09-30T08:04:49Z 仅 9h；714%5=4 非五倍数；
  无 48h+合并触发）。**部署观察**：docs-only 落地（crates/ 零 diff
  断言在案）——backend release/daemon/gen front 三面零重建项维持。
  **cleaned**：移除前 fresh 复核双净——worktree 0 dirty+HEAD=41ace6107
  （全部落库）+wt-guard clean（零 reparse point）；worktree 注销+分支
  删除（was 41ace6107）+组目录 .wt/lang-714 rmdir；worktree list 零
  714 残留复核在案。五检查点全落 delivered。


- 2026-09-30 r2 收纳（用户指令「更新计划 714…给它加一个 phase」——
  auto-edit PLAN-021 Q-4 解锁件路由本件；**712 r2 收纳复开先例同款
  [用户预授权收纳通道]**）：archived→executing、plan_revision 1→2。
  scope=供① 残余两面解锁（back 转译通路 Try 臂+corpus 判据三行+
  下游解阻确认）——**非 tree-sitter 面**（r1 勘定交付保持 delivered；
  715/716 实施件预留位不受扰）。r1 交付收据与五检查点不变。next=work
  （R2-T1 起；r2 执行期簿记沿 712 r2 形态直接落 master docs/plans）。


- 2026-09-30 r2 work handoff：`stage: work`，PLAN-714，plan_revision 2。
  `outcome: needs_replan`（目标不变——供① 解阻成立面已交付；fresh 层
  暴露 710 §10 延后的 route-A 深修=有界修订路由 new）。`code_commit`:
  plan-714-dev 4534b5004（Try 臂 trans/rust.rs:12300 前插+三单测
  plan710_supply_probes+census §7 判定集三行；基线 master@99405c675，
  worktree D:/autostack/.wt/lang-714/auto-lang）。`task_ids`: R2-T1
  （臂+单测三绿+AC-R2-2 实质）、R2-T2（判定集三行+fresh 收据）；
  R2-T3 未启动（下游复验位候深修件）。`evidence`: tt 全谱零新增归因红
  （fail-fast 3 红+no-fail-fast 3 红均基线：e4=P707-R1、desktop_bus/
  surface=主检出同败实证、p053=known flake 族 no-fail-fast 转绿）；
  corpus fresh 跑（AUTO_BIN=v0.4.2-2330-g99405c675-dirty，git archive
  快照零触碰真仓）——skip 警告 0、fsys.rs 全新生成、env_str/read_text
  实体形非 D-7 桩；26 错/8 类清单在 census §7（fs.metadata/copy_
  recursive、json.from_value+struct-literal、Regex.test、list 型、
  diff 三件套、File.write_bytes、E0308×3[try 尾 bool+闭包内 return
  传播=710 边界条款实实例]）。`blockers`: AC-R2-1 exit 0/check 过
  需 route-A 深修件（新增 stdlib 面 fs::copy_recursive+diff 三件套+
  json from-struct+list 型映射+闭包 return 传播）。`next: new`
  （有界修订：扩 R2-T1 或立深修件承 710 §10——715 位）。

- 2026-09-30 r3 收纳（用户指令「不需要新计划；而是继续在计划714上加
  新的阶段即可」——r2 needs_replan 的有界修订兑现，收纳通道同款，
  不另立新件）：plan_revision 2→3，+R3-T1..T6（总步 10→16）。scope=
  route-A 深修（Q-R2-2 八类清单承 710 §10 延后边界）：stdlib 面/内建
  表臂+型映射/Try return 传播/corpus 收口[AC-R2-1 补全]/下游解阻确认
  [原 R2-T3 顺位承接]/落账。r1/r2 交付收据不变。next=work（R3-T1 起，
  worktree lang-714 延用）。


- 2026-09-30 r3 work handoff：`stage: work`，PLAN-714，plan_revision 3。
  `outcome: pass`。`code_commit`: plan-714-dev 96e848ea5+3574b47c8+
  211ef5237（基线 master@dfa9e86e5=r3 收纳，worktree lang-714 延用）。
  `task_ids`: R3-T1..T6 全账（6/6，current_step 16/16）。
  `evidence`: stdlib 面五 fns+diff 三件套模块（code-editor 门双轨）；
  表臂三处分发器齐装（Bina 元组/两段 ns/Dot method_name——r3d 迭代
  实证第三处才拦到 corpus 两段形）；Try 传播形+借位 clone 窄门；单测
  7 枚（plan714_* 六+fsys 全文探针）+cookbook 125/125 全绿；tt 全谱
  5266/5280（14 红全数已知/flake[plan707_cancel 隔离绿]/worktree 环境
  [scan/launch=缺 debug 应用二进制，主检出同测绿]）——零新增归因红；
  corpus fresh regen exit 0+check 过；下游 perf.py a2r 三重判据全绿
  （deps 预置真实拷贝修复半拉 materialize 态=710 配方复刻）。
  AC-R2-1 补全+AC-R2-4 交付——**供① 残余两面全解锁，PLAN-021 解阻**。
  `blockers`: 无。`next: review`。


- 2026-09-30 r3 review handoff：`stage: review`，PLAN-714，plan_revision 3。
  `outcome: pass`。`reviewed_commit`: plan-714-dev 211ef52377655b9eb2d25cf
  47c9eca8050eaa626（=96e848ea5 深修+3574b47c8 夹具+211ef5237 census；
  worktree 净 0 dirty）；`base_commit`: master@99405c675（r3 收纳；
  merge-base 实证）；master 现势 559aa0c45（712 r2/711 并行落地——与本案
  文件零交叠待 merge 时 range-diff 复证）。`dependency_revisions`: auto-edit
  main（只读+perf.py 复验位）/auto-down detached sibling（浮 HEAD——cargo
  解析面）。`spec_inputs`: SD-01 册（r1 不变仍有效——r3 深修无 canonical
  增量申报；知识落位=census §7+账本 P714-3+单测，canonical a2r 书扩展
  =F-2 建议位）。`acceptance_results`: AC-R3-1 pass（stdlib 五 fns+diff
  模块编译+探针覆盖）；AC-R3-2 pass（三分发器臂——fsys 全文直转探针
  限定断言绿）；AC-R3-3 pass（传播形/unit 保持/borrow clone 三单测绿）；
  AC-R3-4 pass（census §7 r3 收口@211ef5237+P714-3 回读 True）；**AC-R2-1
  补全 pass**（复审 fresh regen 重放：exit 0+skip 0+0 编译错+工作区 check
  58.99s 过）；**AC-R2-4 pass**（下游 perf.py a2r 三重判据全绿——同二进制/
  同 corpus 复用 20:15 凭证[tools/perf/logs/a2r-20260930-201506.log+实仓
  rust-workspace check 59.82s]，复用理由=二进制与 corpus 态自凭证后零变化）。
  `findings`: F-1（低，不阻塞）plan714 探针落位 plan710_supply_probes.rs
  （新文件解析异常未定谳的绕行——同族归家，七绿在案）；F-2（低，建议）
  canonical a2r 书（710 SD-01 册）未扩展新映射族——知识已由 census §7+
  账本 P714-3+单测钉住，建议后续 a2r 域件补书；环境注记=scan/launch 两红
  =worktree 缺 debug 应用二进制（主检出同测绿）+plan707_cancel 负载 flake
  （隔离绿）。`evidence`: 复审重放（实施会话内复审——工件重建）=plan714
  七测绿+corpus fresh regen 重放（a2r-review-run.log：EXIT=0/SKIP=0/
  check 58.99s）+P714-3 回读 True+census §7 在案；tt 全谱 5266/5280
  （零新增归因红）+cookbook 125/125。`next: merge`。

- 2026-09-30 r3 merge 收据：`stage: merge`，PLAN-714:r3，
  `completion_kind: delivered`（三阶段累计）。**prepared**：reviewed 基线
  211ef5237（r3 pass，AC 复放全绿@2cbd5e6b8）；canonical delta=r1 SD-01
  （已 landed，本阶段无新增 canonical 申报——知识落位=census §7+账本
  P714-3+单测，F-2 建议位在案）；projection=既有 P714-2/P714-3 验证性
  刷新。**landed**：wt-guard clean→rebase master（4 提交改写）→range-diff
  四 `=`（4534b5004→d22318fc5/96e848ea5→29563c588/3574b47c8→5f07a97eb/
  211ef5237→acf653d3f 补丁全等）→ff-only 落地，master tip=delivery
  **acf653d3f**（9 文件 858 行）；集成烟=INDEX 再生 no-op。**ledger_
  refreshed**：既有投影验证性回读（designs P714-2+docsha 5e7c2fa187fee8ef
  在位/reviews P714-1+P714-3 在位[reviews 179]）+INDEX 再生 no-op——
  r3 账务已于 work 档落（P714-3@559aa0c45），本档零重复写。**archived**：
  本件自 r1 起居 archive/（r2/r3 收纳原位复开先例）——frontmatter
  status: archived+completion_kind: delivered（本节收据）。批量回归到期
  判定=**不到期**（.last-batch-regression.json last_covered=712@7fcf913eb，
  收据 2026-09-30T08:04:49Z；落地集 711/712-r2/714 均非 %5=0；<48h）。
  **部署观察**：本件改 trans/a2r 发射面——主检出工具链二进制（release
  auto 等）相对本件落地为陈旧态；下游 PLAN-021 复验已用 worktree 钉版
  二进制完成（AC-R2-4），主检出二进制随下轮 auto-lang release 周期或
  021 会话钉版重建，不做本档内重建。**cleaned**：待下笔。
## 10. 待澄清事项

- **Q-1 语言集预裁定（可选）**：默认 T-01 按下游现实+战略「常见
  20 语言起步」定界（候选族见 §5 T-01——含 .at[需自写 grammar，
  成本注记后定含/不含]）；若用户对首批语言有明确偏好（加/删），
  执行前示知——否则按勘定定稿。
- **Q-2 分发形倾向确认（无需裁定，烟测定案）**：默认倾向
  build-time 编译（生态主流+「单 exe」语义无冲突）；若烟测显示
  构建复杂度/体积反转（如 cc 链在 Windows 工具链的面），预编译
  嵌入案升位——按 T-02 证据定，两案对比表在档。
- **Q-3 实施件排期（勘定交付后，用户件）**：715+ 立项时点=勘定
  delivered 后（M4 长尾关键路径——建议勘定后尽早排期；同时清
  M3 尾巴「语法高亮联动」与 M4 产出 4，下游消费件随后）。
- **执行注记（2026-09-30）**：Q-1 按默认定界执行（用户未另行裁定，
  §2.2 表定稿可复审调整）；Q-2 烟测定案=build-time cc（预编译案
  未升位——§3.1 判据表在档）；Q-3 仍待用户（715 立项基已备）。
- **Q-R2 下游联动（2026-09-30 r2 登记）**：auto-edit PLAN-021（L2 链
  解阻兑现件）blocked 挂账中——本件 R2-T1 落地后其 T-02/03/04 判据面
  补跑（bench --l2 三档+smoke_gen 三域，zero 重设计）与对比表 L2 列
  转正由 PLAN-021 承接；本件验收含其解阻确认单（AC-R2-4）。裁定量级
  注记=710 D-4..D-8 快速修订件同档（供料档 §6）。
- **Q-R2-2 route-A 深修层（2026-09-30 r2 执行期暴露，needs_replan 路由）**：
  Try 臂解锁后 fresh fsys.rs/前 bin 嵌入段暴露 710 §10 显式延后的深修层
  （26 错/8 类）：①内建表缺臂 fs.metadata[→file_size 语义]/fs.copy_
  recursive/json.from_value[含 struct-literal arg→json! 形]/Regex.test/
  File.write_bytes；②`list` 型映射缺失；③diff 三件套（diff_files/dirs/
  snapshots）无 a2r_std 面（引擎在 ui/code_editor/diff——imara 703 线）；
  ④E0308×3（preview=ln 借用+try 尾 bool×2）；⑤**闭包内 return 传播**——
  corpus 演化出 710 G-B 边界条款「return 不出闭包」的实实例（fsys.at
  copy/delete 路径 return-in-try），臂形需 Option<Ret> 传播形
  （Ok(Some(v))=>return v/Ok(None)=>default）。量级=M 档独立件（715 位
  ——承 710 §10 边界+本条清单）。auto-edit PLAN-021 复验位候其落地。
