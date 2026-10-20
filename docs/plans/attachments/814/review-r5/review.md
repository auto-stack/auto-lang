# PLAN-814 R5：未闭问题及 inventory 合同对账

2026-10-20。stage: review；review_scope: **phase / PLAN-814:r2:M814-A，R4未闭范围定向复审**；outcome: **needs_fix**。本聊天 Codex 复审上下文，未参与本批实现、未创建 agent/额外 checkout。此次不重新验收尚未完成的 B 阶段，也不授予整 Plan final pass。

## 冻结对象

- plan_revision: **2**；reviewed_commit: `16cc4f14a732f593244f4b60377dd6f1f01bef35`；tree: `6d00fc90cd36c43e9a3b42438d06785922ca9885`；base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009`。
- worktree_path: `D:/autostack/.wt/lang-814/auto-lang`；branch: `codex/plan-814-dev`；比较上轮 `6418f99efe0af2432a652212e98d39b973515cd9`，本轮修复 `81c8a5743`、`189ca0f77`、追修 `16cc4f14a` 均已提交。
- dependency_revisions: 组内 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；实施树及该依赖初末检查 clean。[末查/锁和测试配置 hash](baseline-evidence.txt)。
- 主检出起始 HEAD `1420d2e0dce5cd47879f358b8ea98f9b8b296714`，复审期间经 `38b356c78e84d74b5869f8b14934de6df37fc14b` 前进到 `665ffbb4b447d0731c35045634882d9d56eda21b`（Plan798 簿记、回退及补记）；814 主计划在写入本记录前 SHA256 与冻结副本相同，相关 Spec 末次再核对不变。这不改变受审实施提交，也未覆盖他人主检出改动。
- [受审主计划](plan-before-review.md)、[候选](candidate-frozen.md)、[合同提案](assembly-proposal-frozen.md)、[测试映射](mapping-frozen.md)、[警告归因](warnings-frozen.md)均已冻结。候选正文头仍指早期 SHA，本次按主计划最新交接的 `16cc4f14a` 审核，未把旧候选头当本次 pass。
- spec_inputs: [input-hashes.txt](input-hashes.txt)，包含 9 份现行 canonical Spec 与提案。完整拟议 SD 冻结 [sd-frozen.md](sd-frozen.md)，SHA256 `fd7a2c6a43d4cb1d0adc80ffd868579e2327f048e79d9dd4bdf07e87b332df66`。未修改 canonical Specs 或 live ledger。

## 旧项闭合与剩余范围

| Finding | R5 结论 | 证据与边界 |
|---|---|---|
| F814-R3-01 / P1 | **原行为 CLOSED，保持** | 原 VM 创建入口及 run 控制均安装宿主；显式默认 123/456、VM 快照继续通过。Parser-only 新进程保持 UI 解析。行为闭合不授权 inventory 机制改变合同，见 R4-01。 |
| F814-R3-02 / P1 | **CLOSED** | None→Some 真身份升级、晚装配旧条目失效、跨进程 F1→F1 保留与 F1→F2 失效均通过。save 不再重签旧条目；两项新增定向单测通过。 |
| F814-R3-03 / 余项 P2 | **CLOSED** | named SDK build 输入清单替代全 crates 扫描；本次实际 428 项输入中 UI/Man/旧包/生成器禁止项 0。禁止输入与 path-dep 覆盖两项测试通过。实际无 UI 源码快照 fresh lock + locked SDK 消费 **5/5**。 |
| F814-R3-04 / P1 | **CLOSED，保持** | autodown 特性真实 native 的 parse/serialize/find/text/insert/template/错误分支与宿主正控通过。 |
| F814-R3-05 / P2 | **选择遗漏修复 CLOSED；完整映射证据 PARTIAL / OPEN** | tb/taa/ta/t3 已扩四包，tt 含 compiler；本次 tb list 四包匹配 5300 项，三个曾漏选/丢失代表项均实际 1/1。九档“原 ID 零缺口”的验证器与原始工件尚不足以证明该声明，详下；没有新发现实际丢失测试。 |
| F814-R3-06 / P3 | **fmt/cfg 子项 CLOSED；新增警告 PARTIAL / OPEN** | 本批全部变化 Rust 文件 rustfmt 与 git diff --check 退出 0。工作归因现已承认相对原 master 净新增约 14 条迁移警告，却未经批准转 B；当前实际日志仍有这些残留。 |
| F814-R4-01 / P2 | **OPEN，原合同偏移** | 新增具名显式装配有实质进展；直达构造仍依赖 inventory 兜底，未满足 r2 AC-02 的“无隐式链接装配”。合同提案还误述 inventory 无静态构造器；trait 迁位的规范变更已提出但未获批准。 |

## R4-01：inventory 与原合同逐项对账

1. **原约束没有改变。** 主计划 r2 §5.2 要求 SDK UI 解析显式获得组合，不能靠 linker inventory 拉初始化；V-02 保留旧默认行为且不靠 inventory 偶然性；AC-02 明文“无隐式链接装配/双状态”。§5.1 又要求 A 的旧默认 API 不能先破坏。单独证明一个行为成功不能删除另一个约束。
2. **本轮真实变化。** 上层 `install_ui_host_capabilities()` 在两个 feature 形态显式 `set_ui_grammar`，compiler 的三个生成入口经 `assemble_ui_grammar()` 显式安装；这些入口的进展成立。SDK `ui_grammar()` 在未显式设置时仍遍历 `UiGrammarSource`，宿主安装仍遍历 `UiCompatInstaller`；上层/编译包的 `inventory::submit!` 被链接后提供回退。Parser-only 正控验证的恰是该兜底。`set_ui_grammar` 是显式入口，不使没有调用它的旧路径自动变成显式装配。
3. **“无静态构造器”与所用依赖不符。** 锁定 `inventory 0.3.25` 的实际源码由 submit 宏生成 `unsafe extern "C" fn __ctor()`，Windows 通过 `.CRT$XCU` 放置 `__CTOR`，启动时注册节点。[源码位置/摘录](inventory-implementation.log.txt)。文法工厂仍可惰性调用，但注册自身确有启动构造器副作用。SDK `lib.rs:1351` 与提案的绝对否认必须纠正；这里不声称当前 Windows 样例随机链接失败或构造器已运行整个 UI。
4. **旧 host finding 的关闭不等于授权机制。** R4 已明确把 R3-01 的行为闭合与新 R4-01 的合同偏移分开。本轮提案以“同机制已被接受”作为授权来源，与该历史记录不符；它只能说明两个收集点应按同一条原合同审查。
5. **trait 归属是另一项真实规范变化。** 现行 `auto-parser-api/project.md:76..84` 与 `auto-ui-dialect/design/full-grammar.md:28` 明确 UiParseContext 不入 parser-api；当前已搬入。新提案补了理由、目标路径及兼容 re-export，优于此前缺失正文，但孤儿规则只约束直接 impl，不自动证明所有保留旧 API 的适配设计都不可能，不能凭此把未批准的归属变化写成现行规范。

**处理路径：** 可以继续在原合同下提出并验证兼容装配方案；若确需保留 inventory，则先走明确的语义修订/批准，而不是由本次复审替 r2 默批。提案应如实写出允许上层经启动构造器注册轻函数表、SDK 无 UI 实现/提交、显式入口及直达兼容兜底各自范围，并同步明确 trait 归属；修订后再绑定新版合同验行为。当前 A/B 两条路线并未证明穷尽全部保留旧 API 的实现，本次不批准删除直达 API 或把它改成硬错误，也不把这种破坏当强制修复。

## R3-05：原遗漏已修，零缺口证明尚不可靠

本次实际调用 `cargo nextest list --offline --locked -p auto-lang -p auto-lang-core -p auto-ui-compiler -p auto-ui-dialect --lib --features auto-lang/test-book --message-format json`，仅列清单，没有执行整档。[原始 JSON](tb-selection.json.txt)、[构建输出](tb-selection-build.log.txt)。四包匹配数分别为 3367 / 1631 / 216 / 86，共 5300，与工作表此档总数相符。SDK 的 `trans::rust::test_transpile_fn_single_function`、compiler 的 `style::plan095_tests::test_style_recipe_in_widget_extraction`、dialect 的 `regression::parser_ui_grammar::tests::test_with_session_registers_ui_dialect` 均被选中并定向运行通过：[三项执行](restored-tests.log.txt)。此处关闭原包选择与代表项遗漏，不机械重开它们。

剩余证据缺口：

- 映射表自己记载迁前九档原始清单随临时树删除；指定 `tmp-814-base/target/r4-old-tb.txt` 当前不存在。存留工作树脚本不是完整可回放的原始新旧映射工件。
- [exact_diff 原样冻结](exact-diff-frozen.py) 去掉包名，对未改名路径使用 `new[rewrite(path)] + new[path]`，相同项被重复计数；只含四条路径重写，未给出完整映射。另一个 [tier_diff](leaf-diff-frozen.py) 只按末级函数名判命中，不能区分不同模块/包的同名测试。
- 独立对原样 exact checker 提供正控两项均保留、负控从两个包的同名完整路径测试删除一个；**八个被检查档位都仍报告 unmatched=0**。另按迁前单包形态，向实际 leaf checker 的原样 load/diff 提供同一包两个不同模块中的同名测试，再删除一个，仍报告 `old=2 migrated-selected=1 selection-gap=0 real-lost=0`，没有逐 ID 报告所删项。[可回放控制程序](mapping-checker-controls.py)、[实际输出](mapping-checker-controls.log.txt)。这是验证器证明不足，不是发现生产测试确实丢了一项。exact 脚本本身也未检查 tu，不能独自证明“九档零缺口”。

须保留旧/新完整 `(package, full_test_path)` 清单，使用显式、无重复消费的迁移映射，并让删除已映射项的负控报缺口；把未验证聚合声明改成相应状态。可重建 list 证据，不需执行整档测试，不需新建 reviewer 专属树；如使用工作阶段已采集原件应直接保存。原 86 项映射在 R4 已逐名核过，相关文本和 dialect 源未变，可复用该局部结论，不能外推全部原 inline 测试。

## R3-06：归因修正有效，延期不构成清偿

[工作归因冻结](warnings-frozen.md) 末节已纠正此前以中途 d905e796c 比较“296 不变”的口径：相对原 master `203e2ce91`，约 14 条净新增属于 T-02/T-03 迁移残留；处置栏写“B 清偿/A 不动”。这不是预存于原 master，也未见获准延期。

[本次编译输出](probe-build.log.txt) 可见 SDK `src/lib.rs:14` 未用 PathBuf、SDK 的 aavm 私有常量/函数转死、上层 `src/lib.rs:217/219` 未用 atomic/AutoError 等。这里只要求处理本 Plan 引入的警告，不要求清除全仓旧 warning。aavm 暂不跑重档也不豁免普通构建下新增死代码告警；移除残留或恢复正确条件/可见性应由 work 完成，归因表与候选清单同步，不用 blanket allow 掩盖。

## 本轮执行、证据与复用

- 独立 probes 用 `cargo build --offline --locked --manifest-path .../review-r5/probes/Cargo.toml --target-dir <原树>/target --features auto-lang/autodown --bins`，实际构建当前源码成功：[build](probe-build.log.txt)。
- 原 host 探针 [输出](host-original.log.txt)、[显式默认/快照与 autodown](host-autodown.log.txt)、[Parser-only](parser-only.log.txt) 各新进程退出 0。
- 缓存 [None升级](cache-none-upgrade.log.txt)、[晚注入](cache-late-save.log.txt)、[同版本](cache-same.log.txt)、[跨版本](cache-upgraded.log.txt) 均退出 0。晚注入结果 retained=0、dirty=true；原早期空缓存“保存时必须盖当前身份”的过强序列化断言不再作为 oracle，按缓存出生身份保存再失效同样满足防止旧产物复用的原合同。
- `cargo test --offline --locked -p auto-lang-core --lib plan814 -- --nocapture --test-threads=1` **4/4**（身份转换两项、build 输入两项）：[输出](plan814-scoped.log.txt)。SDK 实际 [输入摘要](sdk-build-inputs.log.txt) 与 [完整生成集合](sdk-build-inputs-frozen.rs)，禁止项 0。
- UI cache 小家族并行 **11/11**：[输出](cache-family-parallel.log.txt)。新增两条共享进程槽测试有 reset，注释把 libtest 描述为每测试一进程不准确；因而仅重复这两项并行 50 次，未复现失败：[输出](cache-parallel-repeat.log.txt)。本次不据理论竞争另开生产缺陷，也不声称这些测试已证明所有并发路径。
- 实际移除 UI/旧包/Man 源码后的独立 SDK snapshot，新生成消费者锁后 `--locked` 测试 **5/5**：[程序](no-ui-source.py)、[完整输出](no-ui-source.log.txt)，结尾 `NO_UI_SOURCE_FRESH_LOCKED_PASS`。SDK 源级检查范围与实际执行均通过，未将 metadata 成功冒充运行成功。
- R4→当前所有变化 Rust 文件 `rustfmt --edition 2021 --check --config skip_children=true` 退出 0：[输出](fmt-repair-scope.log.txt)；git diff --check 退出 0。原 cfg 清偿代码未变、当前构建未见旧 unexpected_cfgs，复用其局部闭合。
- 其余旧 A 消费/生成支持面不在本次定向重审中重新授 pass；792 driver 25/25 是 work 交接证据，本轮没有把它转记为独立执行。已知不满足的合同/健康/覆盖证据修齐前，不再运行一轮完整最终验收。

## AC、SD 与交接

| 范围 | 结果 | 下一步 |
|---|---|---|
| AC-01 / V-01 的独立 SDK 源、执行、build 输入子面 | pass（本次定向范围） | 保留证据，不等于 B 整体 AC-01 pass |
| AC-02 / V-02 的旧 host、Parser-only 行为 | pass（行为）；合同 fail | R4-01 的机制及 trait 归属对账；不得以行为通过默批合同 |
| AC-05 / V-05 的原别名遗漏 | 修复已验证；完整映射 partial | R3-05 修验证器与持久原始映射证据 |
| AC-08 / M814-A | **fail** | R3-05 证明余项、R3-06 新增 warning、R4-01 修齐 |
| AC-09 / T-14 | **未通过** | 冻结新实现/获准的新合同后受影响复审 |
| AC-10 / T-15 | pending，merge-owned | 阶段 pass 后再冻结门禁、实际合入与发布/知识收据 |

SD-01/02/04/05 的新提案已具名端口与调用顺序；缓存出生身份语义与现有防陈旧目标相符。SD-03 完整 runtime 仍属 B。parser-api 与 dialect 的 canonical 约束尚冲突，提案中的 inventory 允许范围又与 r2 AC-02 冲突，故整个 A 规范子集本次不能授 pass。frontmatter 已列准确目标路径与 GOAL-003/007，本次不擅自改这些目标或发布拟议规范。

Plan 保持 **executing**；重开 **T-02/T-03/T-13** 及其依赖审查证据，`current_step=1/15`，保留 T-01 和各项已通过的局部修复历史。next_responsible_role: **work**；若保留 inventory/迁 trait 的语义变化需经批准，先转 **new** 修订合同，再由 work 落实。不能将 T-14 当作用户已批准变更的替代来源。

pending_closeout_gates：阶段验收通过后的冻结裸 `cargo t` 及适用 `tv/tu/tt`、必要 docs/schema/book 面；tf 只在主检出到期单实例。实际 A 合入、Spec/ledger/index、版本/来源/consumer 读回收据仍 pending。此次未跑整档、未修实施代码、未合入/归档/清树，也不宣称 A 已交付。
