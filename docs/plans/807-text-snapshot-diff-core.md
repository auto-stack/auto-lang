---
plan_id: PLAN-807
status: drafting
feature_name: 文本、分页快照与 diff 基础模块整体抽取
author: [Codex]
created_at: 2026-10-18
updated_at: 2026-10-18
plan_revision: 1
current_step: 0
total_steps: 10
supersedes_spec_components: [docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/ui/design/paged-rope.md, docs/specs/auto-lang/vm/overview.md]
new_spec_components: [docs/specs/auto-text-core/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-text-core, auto-lang/ui/code_editor, auto-lang/vm, auto-lang/ui_gen]
depends_on: [PLAN-802]
---

# [PLAN-807] 文本、分页快照与 diff 基础模块整体抽取

## 0. 变更摘要

交付路线R02的完整文本模块：新建`crates/auto-text-core`，一起承接持久化rope、文件后援分页、快照、diff及纯文本/路径envelope；UI仅负责文档注册表和editor交互，VM四native与现有Rust消费路径使用同一算法。包含生产接线、旧算法删除、独立消费、分页资源合同与发行输入登记，不止搬几个函数。本计划不搬Git仓。

## 1. 目标

- 可脱离语言/VM/UI构建和消费的单份文本基础；rope与diff共同保有私有剪枝结构，不为凑分层再拆两份小包。
- 保持现有公开rope/diff路径和行为，补最小跨crate分页入口；不公开全部Node/PageDesc等内部结构。
- 9915/9916/9917/9921不再引用`crate::ui`，快照直接传共享快照类型，不经VM全文字符串中转；实际编辑器和a2r/Rust UI仍消费同份实现。
- 不含语法高亮/cosmic布局、editor22项重新迁移、分页视口新功能、diff算法改版或性能阈值重定。不得以删除文件后援支持换独立构建。

## 2. 架构方案

`auto-text-core::{rope,diff}`拥有数据与算法；标准库FS分页/文件目录观测可在该包的具名子模块，不能反依赖UI。原`ui/code_editor/core/rope`及纯diff路径成为薄重导出；键名→editor快照留UI adapter。VM引入轻快照能力端口并持有该类型；不要给802既有trait新增必实现方法而破坏外部实现。默认provider按现有进程补缺/实例覆盖规则接入，缺能力与缺key出口要明确映射。纯文件/目录diff直接调core，不假装OS provider。

## 3. 技术栈

Rust现有持久化树、Arc、std FS/线程与imara-diff；依赖按实际迁入算法最小复用。不新造文本副本、全局Any表、另一套diff或自建序列化协议。

## 4. 需求分析与背景调查

授权：用户确认较大模块粒度并要求建立下一批计划；当前只起草，不授权本会话执行、搬仓或远端写入。实现限定auto-lang仓；不改相邻auto-ui/auto-man/auto-term。依据[Design 37](../design/37-v06-modularization-and-repo-boundaries.md)、[统一路线](reports/v06-extraction-execution-roadmap.md)与[本批来源清单](attachments/module-batch-20261018/inputs.json)。

权威输入：[UI Spec](../specs/auto-lang/ui/overview.md)、[分页rope](../specs/auto-lang/ui/design/paged-rope.md)、[VM宿主](../specs/auto-lang/vm/overview.md)、已归档[802](archive/802-code-editor-host-provider.md)。观察到：

- `ui/code_editor/core/rope.rs`与`rope/file_backing.rs`只有标准库基础依赖；`open_file_backed`、PageConfig、snapshot prune为crate私有，跨包访问需要最小设计。
- `ui/code_editor/diff/{mod,envelope,dirs}.rs`以RopeSnapshot剪枝；仅envelope中的`editor_snapshot`直接取UI注册表。snapshot与文本diff允许合法的hunk边界差异，不能发明二者逐hunk恒等的验收。
- `vm/native.rs`四shim、`a2r_std.rs::diff`和`ui_gen/rust.rs`发射路径目前直接消费UI入口；`code-editor`关闭时有各自raise/panic契约，不能借抽包扩大启用面。
- 旧分页合同：64KB默认页、6MB cache、±16预取，1GB树+页表+缓存结构计量≤12MB；50MB editor装载阈值与2MB布局窗口留UI。结构计量不等于RSS上界。

前置802已交付；不依赖804/806或本批808/809。777规则/基线、792受管source清单和共同装配写面按[本批并行合同](reports/module-migration-batch-807-809.md)串行合流。

## 5. 详细设计

1. **唯一所有者**：rope、分页缓存/预取和diff算法物理迁出；UI旧路径重导出同一Rope/RopeSnapshot身份，旧算法不得在门面/测试fixture再保留可被生产执行的副本。测试旧臂oracle采用冻结输出和独立构建对照。
2. **最小API**：公开可消费的分页配置/打开入口及既有查询编辑方法；剪枝跨度可继续在同包私有。T-01提交API/所有者图和编译签名pin，不能把editor core/字体/renderer拉入闭包。
3. **快照端口**：新增窄能力（暂名EditorSnapshotProvider，最终名T-01冻结），返回同源owned/Arc快照；UI adapter完成key查找，pure envelope接收快照或具名missing结果。缺key保持`编辑器不存在: <key>`及原先判错优先；缺provider定义为无法找到相应editor，返回既有err envelope。若读取期间删除/替换，已取得快照仍有效。过程默认只补缺，实例覆盖隔离；不借此让VM携UI registry。
4. **四入口冻结**：ID/参数pop顺序、负ctx/offset/limit钳零、JSON字段/转义/行域/counts/错误阶段不变。禁用feature仍按旧shim不取参即raise；a2r原panic文案保留。窗口是全量结果投影，不能重新diff切片。
5. **分页保持**：UTF-8预扫/字符边界、BOM/EOL字节忠实、hash/摘要零驻留、LRU/预取寿命、快照COW、streaming write_backed、基底len/mtime改变拒绝及临时文件结果按旧合同。不能为公开类型调`to_string`隐式全文物化；显式旧全文API复杂度不另作承诺。当前`spawn_prefetcher`的worker持强Arc、store持sender，存在不能随最后外部引用释放的闭环风险；T-01必须用弱引用/线程退出观测确认，迁入时清偿该所有权闭环，使最后rope/snapshot/store外部引用释放后预取线程及文件句柄受控退出。仍存活的snapshot必须继续可读，不可粗暴关闭底层store破坏共享寿命。这是迁入分页模块所需的有界寿命修复，不扩为其他editor资源重写。
6. **文件目录保持**：目录五态、隐藏/构建目录过滤、≤2MB内容域、>2MB同大小`same/uncompared`、5000上限与counts迭代域按旧臂；不顺手改变遍历/软链/元数据竞态语义。支持边界由T-01逐入口取证。
7. **发行责任**：normal/build/dev闭包无auto-lang/UI/VM；独立manifest做真实解析/测试。登记777精确域边、更新792受管包/资源/依赖源和受影响fresh/locked consumer；不假定“workspace能编译”已足够发行。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-text-core/project.md | UI内文本算法→独立文本数据/分页/diff及最小API、支持面 | 单源可消费 | AC-01, AC-02, AC-04 |
| SD-02 | modify | docs/specs/auto-lang/ui/design/paged-rope.md | 路径迁移，全部分页不变式保留；明确UI窗口归属 | 防迁移弱化大文件支持 | AC-02 |
| SD-03 | modify | docs/specs/auto-lang/vm/overview.md | diff四项UI直连→文本core+快照端口，缺席/feature矩阵 | VM脱UI | AC-03 |
| SD-04 | modify | docs/specs/auto-lang/ui/overview.md | editor/Rust消费新core、注册表adapter及旧路径兼容 | 实际生产单源 | AC-01, AC-03, AC-04 |

## 6. 测试设计

T-01先冻结旧臂支持面/签名/输出及失败语义，T-06使下列新fixture可运行；当前不是执行证据。新增独立manifest：`test/fixtures/modularity/text-core-consumer/`；新增上层回归：`crates/auto-lang/src/tests/plan807_text_core.rs`。所有V项owner_stage=work。

| Case | AC / invariant | 输入及真实入口 | 区分性oracle／覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 单源/身份 | 独立consumer、旧rope公开路径、真实editor load/edit/snapshot→diff | 同一快照可跨新旧API传递；包图无语言/renderer；定位生产调用未留旧算法 |
| V-02 | AC-02 字节与持久性 | 内存/分页同操作序列，跨页Unicode/CRLF/BOM/尾部坏UTF8、快照后编辑/淘汰 | bytes/hash/line/point与独立String模型相符，旧快照不变；坏UTF8保持InvalidData；不把非字符边界新定义为有效 |
| V-03 | AC-02 资源/保存 | 旧1GB分页benchmark及受控小页压力；write_backed成功、基底len/mtime变更拒绝、目标写失败、最后snapshot/store释放 | 结构≤12MB@1GB独立记录RSS；摘要不fault全文；保存字节相同，拒绝不误报成功；弱引用可失效、句柄释放、预取线程实际退出，存活snapshot仍可查询。记录temp，不新增更强保存事务承诺。work在唯一计划树预约机器单实例运行必要1GB定向资源测，非整档；不得以缩小样本/旧包结果替代新包资源验收 |
| V-04 | AC-03 四native语义 | VM真实catalog调用9915/16/17/21，a2r_std、Rust生成调用；缺key、缺provider、code-editor开/关、负参数 | 旧臂冻结JSON/错误文案、pop轨迹；新旧快照provider/两VM隔离；快照传输无全文VM副本 |
| V-05 | AC-04 diff语义 | serial/parallel文本、snapshot剪枝、窗口投影；纯增删/重复行/多hunk/空文档/Unicode；目录五态/cap/失败 | 同一路径旧oracle零漂移；serial=parallel，window=full rows slice；snapshot以自身旧oracle及编辑语义校验，不强求text同hunk布局 |
| V-06 | AC-04 外部发行 | 新consumer fresh解析/locked构建、792现有生成工程受影响profile、777 strict | source包齐全、单份类型/资源；缺包/非法反向依赖负控失败，不能宽化forbidden名单 |

work：`cargo check -p auto-text-core -p auto-lang`、`cargo test -p auto-text-core`、`cargo test -p auto-lang --lib plan807`、`cargo test --manifest-path test/fixtures/modularity/text-core-consumer/Cargo.toml`；必要相关旧rope/diff滤串。777：`python scripts/modularity-audit.py --mode strict --no-cargo-check`，792调用既有delivery driver的受影响profile（T-01记录实际命令，不新造平行发行器）。全部在本计划唯一worktree内。

review独立上下文在同树复核AC/SD及反例；用户未指定具体agent/model，不创建其他会话。merge在最终冻结提交跑裸`cargo t`+`cargo tv`（VM触面）+`cargo tu`（Rust生成接线触面）；V-03资源测若源码/依赖/配置/输入及相关编译形态均未变可绑定work证据，否则受影响项定向重放。不在开发/送审重复全档。`tf`只主检出到期批量。文档规划不运行Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收与证据 |
|---|---|---|
| AC-01 | work | 独立文本包可消费、旧路径类型身份保持、生产editor/VM实际使用单份算法；V-01及调用/包图审计 |
| AC-02 | work | 分页、快照、编辑/保存和资源不变式保持，必需资源测证据适用性明确；V-02/03。缺必需运行环境记pending，不能算pass |
| AC-03 | work | 四native/a2r/Rust消费者、缺席/关闭feature与实例装配全部有值/错误/pop对照；V-04 |
| AC-04 | work | diff/目录/窗口所有冻结语义、独立fresh/locked和777/792接线完成；V-05/06，旧重复实现删除 |
| AC-05 | review | 同树独立AC/SD/遗漏/资源证据审查绑定最终代码修订；零新增未处理warning/格式/debug问题，明确merge门pending |
| AC-06 | merge | 最终冻结门通过、Spec/ledger沉淀/归档、来源与资源交接、guard clean后工作树清理均有实际收据 |

## 8. 执行步骤

每行是唯一权威任务；共10项，内部四阶段：冻结→整体实现→生产/消费验收→独立review/merge。

| 状态 | ID | owner_stage | 依赖 | 文件/动作、可验证交付与命令 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 802交付 | 核对上述rope/diff/native/a2r/ui_gen与Specs；附件`attachments/807/contract-baseline.md`（新增）冻结API/支持/资源/旧oracle/命令及快照端口选择；有界probe证明分页最小API可跨包，未解决正确性问题不得开始依赖实现 | AC-01..04 |
| [ ] | T-02 | work | T-01 | 新`crates/auto-text-core/{Cargo.toml,src/}`迁rope+file_backing与最小API，原路径薄重导出，check+签名pin+V-01/02 | AC-01,02 |
| [ ] | T-03 | work | T-02 | 同包迁diff/envelope/dirs；将key查找剥离UI，删原算法；core单测+V-05 | AC-01,04 |
| [ ] | T-04 | work | T-03 | `vm/host_capabilities.rs`/native/实例装配和新快照adapter接线，保802现有实现兼容；plan807定向测V-04 | AC-03 |
| [ ] | T-05 | work | T-04 | `a2r_std.rs`/`ui_gen/rust.rs`及editor实际路径接线，旧公开门面签名、feature/error对照；scoped plan807与生成器定向测 | AC-01,03 |
| [ ] | T-06 | work | T-05 | 建consumer/上层反例，完成V-01..06含必要1GB定向资源测和分页失败/drop观测；777/792并集更新+受影响driver；资源证据列明源码/依赖/配置/输入和实际编译形态 | AC-01..04 |
| [ ] | T-07 | work | T-06 | 去重审计、API/剩余UI边/资源清单及SD草案，附件`attachments/807/handoff.md`（新增）绑定SHA与验证结果；check/相关反例通过，提交后送审 | AC-01..04 |
| [ ] | T-08 | review | T-07 | 同一工作树独立审查完整AC/SD、实际生产路径、分页测适用性和失败清理；必须保留未完成merge门 | AC-05 |
| [ ] | T-09 | merge | T-08 pass | 合流master并集，变更则复审影响面，在新冻结提交运行§6最终门及受影响资源重放；门失败归因、不得跳过 | AC-06 |
| [ ] | T-10 | merge | T-09 | canonical Spec/ledger沉淀、归档与真实交付收据，wt-guard clean再清理；核对最终计数 | AC-06 |

handoff：`D:/autostack/.wt/lang-807/auto-lang`，分支`codex/plan-807-dev`。work先确认实际base/兄弟依赖，不在master改代码；同树用于work/review/merge，不另建同步review树。与808/804共享装配及manifest只改各自精确片段；后入者同步并集和复测。

## 9. 复审记录

2026-10-18：revision 1 drafting。仅建立合同，未实施、未review pass、未执行未来测试。来源hash及本批文档预检见附件；执行后各角色追加修订绑定证据，历史收据不覆盖当前合同。

## 10. 待澄清事项

无须用户现在决定的阻塞项。T-01负责最小分页API与快照端口的有界决策；若需要破坏公开路径、降低1GB/分页支持、改变native/错误合同，回到needs_replan具体提出，不能由实现自行缩小验收。新增core仍留语言仓，真正搬仓由R08/R09负责。
