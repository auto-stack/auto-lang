# PLAN-814 R6：三项未闭问题与拟议合同修订对账

2026-10-20 / stage: review / review_scope: **phase / PLAN-814:r2:M814-A，R5三项余项定向复审** / outcome: **needs_replan（仅剩合同修订授权及落文）**。本聊天 Codex 复审上下文，未参与本批实现，未创建 agent 或 reviewer checkout；不授整 Plan final pass。

## 版本与范围

- plan_id: PLAN-814；plan_revision: **2**；status: executing；current_step: 1/15。主计划、实施注记与实施者修订提案均明确“待用户裁定”，没有已生效的 r3 或批准记录。本次“修复完成，请再次检查”是审核请求，不能代替对语义修订的明确批准。
- reviewed_commit: `256613936b6a5e9baea450e5c545899ee7a661dc`；tree: `98252f78e85c5a2339cdfc4e584210f8ee068099`；base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009`；上轮 `16cc4f14a732f593244f4b60377dd6f1f01bef35`。
- worktree_path: `D:/autostack/.wt/lang-814/auto-lang`；branch: `codex/plan-814-dev`；dependency_revisions: 组内 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。实施树与依赖初末 clean，详 [baseline-evidence.txt](baseline-evidence.txt)。主检出起始 HEAD `f79b1b83591310b45b3828dd300e6d368987d6c0`；写簿记前主计划 hash 与 [冻结副本](plan-before-review.md) 相等。
- 本批只有六个 Rust 文件改动：警告清偿、跨 crate Drop 恢复、可见性一致化和事实注记。Cargo 清单/锁、测试别名/nextest 配置与 R5 相同；没有新 UI 装配算法。此轮不重开 R5 已闭的 cache/build 输入/独立 SDK/host/autodown 原反例。
- spec_inputs 与 18 件新旧清单 hash 见 [input-hashes.txt](input-hashes.txt)。受审 SD/提案/精确待批准增量一起冻结于 [sd-frozen.md](sd-frozen.md)，SHA256 `2797376deb5d1825b9d39a0913af15916d01f500ec66120a6ee21089672d9e73`。未修改 canonical Specs/live ledger。

## 三项闭合表

| Finding | R6 结果 | 结论边界 |
|---|---|---|
| F814-R3-05 / P2 | **CLOSED，本次具体迁移映射已独立证明** | 18 件持久清单已存在。工作验证器的上轮删除盲区已修，但其唯一叶名回退不能单独证明身份；本次补齐四条真实源码重命名，禁用叶名推断，逐旧 ID 核销九档、每个新 ID 至多一次消费，并对九档执行删除/同名替换负控。当前 tu/taa 列表另经实际编译核验。关闭具体映射缺口，不把工作脚本称为任意输入均健全。 |
| F814-R3-06 / P3 | **CLOSED** | 当前五包 check 退出 0；按完整源码路径映射、消息与重复次数对原基线日志比较，新增 0、减少 7。新增警告实际清偿，没有转 B。六文件 fmt/diff check 通过；Drop 语义的两项单测与跨 crate 四种独立控制通过。原仓 warning 仍存在，不宣称全仓零 warning。 |
| F814-R4-01 / P2 | **事实纠正/提案准备 CLOSED；合同授权/生效仍 OPEN** | 代码注记与提案已承认启动构造器、撤回“旧行为通过即授权机制”和“孤儿规则唯一合法位置”。实现仍保留 inventory，r2 AC-02/V-02/§5.2 未改变，UiParseContext 归属仍与现行 Spec 冲突。提出待批文本不是已批准的合同，不能阶段 pass。 |

## R3-05：实际迁移已闭，验证器结论按证据分层

本次运行 [工作验证器 selftest](mapping-selftest.log.txt)，九档均映射完整、删除项报缺口；再用 [独立控制](mapping-controls.py) 重放上轮两种同名删除、已声明重写正控、多候选负控及真实 CLI 删除变异，CLI 返回 1 并具名 MISSING，[输出](mapping-controls.log.txt)。这些修复有效。

仍观察到工作 `reconcile` 对“旧 parser::tests::wanted 被 unrelated::tests::wanted 替换”会 PASS，这是唯一叶名自动映射的固有局限。本次不再凭该回退授映射通过，也不要求为审核凭空发明新产品功能：已把实际仅有的四条回退核对到原源码，冻结为精确映射：

| 原完整路径（旧 auto-lang） | 当前包与完整路径 |
|---|---|
| back_prefix::tests::route_page_fetch_reaches_proxied_root_when_guard_active | auto-lang / tests::plan712_back_prefix_route_fetch::route_page_fetch_reaches_proxied_root_when_guard_active |
| design_tokens::recipe::tests::test_style_recipe_in_widget_extraction | auto-ui-compiler / style::plan095_tests::test_style_recipe_in_widget_extraction |
| vm::native::tests::p656_scroll_controller_natives_end_to_end | auto-lang / tests::plan656_scroll_controller_pipeline::p656_scroll_controller_natives_end_to_end |
| vm::tests_clipboard_natives::clipboard_files_and_image_via_vm_shims | auto-lang / tests::plan485_clipboard_natives::clipboard_files_and_image_via_vm_shims |

[原源码定位](four-renames-source-locations.txt) 与 [1](rename-1-source-excerpt.txt) / [2](rename-2-source-excerpt.txt) / [3](rename-3-source-excerpt.txt) / [4](rename-4-source-excerpt.txt) 新旧摘录表明它们是对应的路由 fetch、配方提取、scroll 全管线及 clipboard shim 回归。此处核对身份和原断言，不实际操作系统剪贴板或重跑这些不受本批行为变化影响的测试。

独立 [strict-selection-proof.py](strict-selection-proof.py) 只允许完整路径、已记录路径重写和上表四条确切 ID；不使用唯一叶名。九档 `missing=0 ambiguous=0`，删除及同名无关替换均检出；同名碰撞/多候选控制也按预期拒绝：[输出](strict-selection-proof.log.txt)。完整逐 ID 结果保存为 [strict-selection-mapping.json](strict-selection-mapping.json)。这份独立证据补齐了原 finding 要求，不更改实施代码。

| 档 | 旧唯一 ID | 新唯一 ID | 旧 ID 核销 |
|---|---:|---:|---|
| t | 5303 | 5307 | 全部 |
| tf | 6270 | 6274 | 全部 |
| tv | 163 | 163 | 全部 |
| tt | 5675 | 5679 | 全部 |
| tb | 5296 | 5300 | 全部 |
| tu | 867 | 867 | 全部 |
| taa | 6266 | 6270 | 全部 |
| ta | 6720 | 6724 | 全部 |
| t3 | 6721 | 6725 | 全部 |

tu 原/新均有 869 个 libtest 描述符，但两个完整名称各重复一次，所以有 867 个唯一 ID，[原始旧清单](tu-old-raw-frozen.txt)、[当前完整 list 输出](tu-list-current.log.txt)、[逐项比较](tu-identity-check.txt)。当前 auto-lang 0 项、compiler 869 项；工作清单的 `libtest` 占位包名在独立 proof 中明确还原为旧 auto-lang、新 auto-ui-compiler，没有丢掉两个不同包的测试。重复情况不是迁移新缺口。

本轮还执行当前 taa 配置的 **nextest list**（未执行测试）：常量 `cfg(test-aavm)` 及消费方真实编译成功，[编译输出](aavm-compile-list.log.txt)，[当前 JSON](aavm-compile-list.json.txt) 的匹配集合与持久 new-taa 清单 **6270/6270 完全相同**，[对账](current-taa-selection-check.txt)。其余清单的配置未改，六文件 diff 没有新增/移除测试，证据复用范围限测试选择，不把 list 当整档运行。

后续若复用工作验证器，应采用本次完整显式映射，不能再用唯一叶名自动授予未经核对的迁移身份；本轮当前数据的通过依据是独立 strict proof。

## R3-06：清偿及 Drop 影响验证

- `cargo check --offline --locked -p auto-lang-core -p auto-parser-api -p auto-ui-dialect -p auto-ui-compiler -p auto-lang` 退出 0：[完整输出](check-five.log.txt)。
- 工作原比较器只按“消息+文件名”去重，本次另用 [warning-analysis.py](warning-analysis.py) 按完整旧 owner 路径映射并保留重复数，原日志 406 条、当前 399 条，**新增 0、减少 7**：[结果](warning-analysis.log.txt)。[原基线日志](warnings-old-frozen.log.txt) 从 work 在原提交临时树已采集工件冻结，并核对该树仍为 203e2ce91/clean；本轮未另建/重编旧树，不把该原日志称为独立重跑。当前 check 是本次独立执行。
- 新增净警告已通过删死副本、消费方 feature 条件、未用 import 清理、Drop 恢复及 HTTP 公开签名类型的可见性一致化消除。未新增 blanket allow。SDK 剩余那条 lib.rs rustdoc warning 在原 lib.rs:550 已存在，完整映射也确认其来源，不以全部历史警告为本次要求。
- `cargo test --offline --locked -p auto-lang-core --lib http_base_ -- --nocapture --test-threads=1` **2/2**：[输出](http-base-scoped.log.txt)。独立普通依赖消费者 [guard-probe](guard-probe/Cargo.toml) 验证“原值存在/缺席 × 正常退出/捕获 panic”**四种控制通过**，[输出](guard-probe.log.txt)。因此不只是在 SDK 自己的 cfg(test) 下验证 Drop。
- 六个变化 Rust 文件 `rustfmt --edition 2021 --check --config skip_children=true` 退出 0：[输出](fmt.log.txt)，git diff --check 退出 0。

## R4-01：合同修订完成到哪一步

[实施者本轮提案](assembly-proposal-frozen.md) 已正确区分“拟议允许”与“现行合同”，并如实记录 inventory 0.3.25 的 `__ctor`/`.CRT$XCU`。旧 host finding 的行为闭合不作为授权；trait 迁入 parser-api 是可选设计而非孤儿规则迫使。这些事实纠正均关闭。

但没有以下三件事：用户对窄例外及 trait 归属的明确批准、主计划升级新 revision、AC-02/V-02/§5.2 与实际 SD 的一致落文。当前仍是 r2、仍有 inventory 注册与 trait 实际迁位，故原合同偏移没有自动消失。此次不凭“请再次检查”推定批准。

原提案另有两句容易混淆：启动注册确实修改 inventory 自身全局登记表，所以上层“无全局可变状态初始化”应限定为 UI/VM 业务状态；上层包当然会依赖并链接真实 UI 实现，“无 UI 实现拉入”应限定为纯 SDK 闭包及启动登记期不执行业务。复审已准备 [精确拟议 r3 增量](contract-r3-draft.md)，同时覆盖这两处表述与 trait 的 canonical 目标；它是待批准草案，不改变 r2。拟议范围未发现新的能力缩减，不批准删直达 API 或硬错误替代。

变更目标仍为 frontmatter 已列的真实 Spec 路径和 GOAL-003/007；SD-03 完整 runtime 仍属 B。两个冲突目标为 `docs/specs/auto-parser-api/project.md` 与 `docs/specs/auto-ui-dialect/design/full-grammar.md`，其现行“不入本 Core 包”规则 hash 已冻结，review 未发布新规则。

## 验收与下一步

- acceptance_results：R3-05/V-05 当期测试选择证明 **pass**；R3-06 健康与受影响 Drop 控制 **pass**；AC-02 兼容机制的合同面 **未通过**；AC-08 仍受该唯一合同未闭项阻断，AC-09/T-14 **不授 phase pass**，AC-10/T-15 pending。B/最终 AC 不在本轮重判范围。
- R5 已闭 cache、build 输入、无 UI 源码、host/autodown 原行为保持，不复活。装配代码本轮只改注记，Cargo/依赖/相应函数体未变，R5 行为证据可用于 r2 状态核对；批准新合同后仍需按新版合同重验受影响显式/直达/纯 SDK 正反控，不把旧 pass 自动转授给新语义。
- next_responsible_role：**用户决定是否批准具名合同增量 → new 正式修订 → work/受影响 review**。若保留原约束，则由 new/work 提出满足原合同的方案；不推定拒绝修订就允许破坏旧 API。
- Plan 保持 **executing1/15**，T-02/T-03/T-13 继续未勾，T-01 保留；不因两个 finding 关闭伪造整阶段完成。R4-01 沿用原 ID，仅授权/合同同步范围 OPEN，不另立相同技术缺陷。
- pending_closeout_gates：阶段 pass 之后 merge-owned 冻结裸 t、适用 tv/tu/tt 与必要 docs/schema/book 面；tf 只主检出到期单实例；实际 A 合入/版本来源/consumer 读回/规范 ledger/index 收据仍 pending。本轮仅 scoped check/test 与 list，未跑整档、未改实施代码、未合入/归档/清树，不宣称 A 已交付。
