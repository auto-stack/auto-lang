# PLAN-797 合入后收尾核查（2026-10-20）

**结论：实现已合入，收尾核验 blocked / needs reconciliation，仍有4项遗漏。** 归档计划保持 archived 9/9，不重开或回滚已交付代码。本报告不将历史 R17 代码 findings 复活；新增项均是合入/证据/归档收尾缺口。

本上下文此前执行过 R17 review，未实施本分支或本次 merge；本轮按用户“合并后检查遗漏”进行历史交付核查，未创建代理。使用 auto-plan-review，并参照 auto-plan-merge 的收尾检查。未执行 Cargo、构建、整档、批量回归、台账修复或工作树清理。

## 已确认完成

主检出 `D:/autostack/auto-lang` @ **`e918f91d52da54594c981638dbfb28c784aa974a`**。delivery **`1799f86ca113429481e700347f3dd45b4a770913`** 为 HEAD 祖先；R17 reviewed `5a54f588d2f6ea5cf50bd69ebb3d476c09e0d18e` 经 rebase 改写，不要求它仍是 HEAD 祖先。逐文件比较原审查范围50个非docs文件，review/delivery blob **全部相同**（[身份](implementation-equivalence.json.txt)）。ac-core/store/sema-contract、probe锁/配置及冻结SD无差异。未发现遗漏的实施修复。

canonical auto-abi三份正文、auto-ac两处导航/分域、索引登记和归档计划已入库；有限Windows x64/声明IR/candidate边界保留。plan-797-dev分支、lang-797工作树及组目录当前均不存在，归档有guard-clean及清理记录。当前其他计划WIP不属于本次核查改动，不作全仓清理。

## F-797-PM-01 / P1：新增台账条目使真实读取器拒绝整个v60文档

位置：`.autoos/specs.json` 的 `reports/P797-REL`、`designs/P797-D1`、`tests/P797-T`、`reviews/P797-R`。

四条的 `created_at`、`modified_at`、`completed_at` 全部被写成字符串 `2026-10-20T16:30:00+08:00`，共**12处类型错误**。实际 `SpecItem` 的前两者是u64，后者是Option<u64>（真实来源 `D:/autostack/auto-musk/backend/crates/musk/src/specs.rs:287`、`:854`；SHA256 `c124c8449cbf16fe2a856a35910f13d22651a09645ecbac2413dbef47cd914fd`，与815恢复阶段冻结来源相同）。普通JSON解析/ID计数不能验证该合同。

直接使用815留存真实 `SpecsStore::load` probe（未重新编译，二进制SHA256 **`555b17448548599ada228c9b4bc1a6500f3690293e6b0b85779ae6ed04b3233b`**，与L3基线一致），**仅load、无roundtrip/save**：

- 当前v60生产文件 **exit1**：`LOAD-FAILED: invalid type: string "2026-10-20T16:30:00+08:00", expected u64 at line 1 column 230902`。
- 815核准v59候选B（SHA256 `33830b65bd58aebe8a15f94fb71906f64b56883d89140697e6496f32d8475063`）**exit0**，6区1098项。
- 生产文件前后SHA均 **`66451eb3a9a221229f281f0ef355aed682c40968bdb9f7965699ac5ee4dca0d4`**，未变。

[真实正反控/命令/输入哈希](typed-loader-controls.json.txt)、[当前失败](current-v60-loader.log.txt)、[核准对照](approved-v59-control-loader.log.txt)、[四条字段审计](plan797-ledger-audit.json.txt)、[schema与load源码](store-schema-excerpts.txt)。这是797新增投影再次破坏815已恢复的读取合同，不能维持“ledger_refreshed已验证”结论。

**需收尾**：在正确workspace的单写者窗口，通过有完整备份/CAS/回滚/typed前后检的受管投影修复797增量并真实读回；保留815既有恢复与其他条目。不得清库或直接用旧v59覆盖当前文档；其余内容可能已包含新的合法更新。当前读取失败时暂停进一步台账写入，先完成恢复核验。本次只报告，未改台账。

## F-797-PM-02 / P2：最终复审证据未进入版本库

`docs/plans/attachments/797/review-r17/` 本机有 **128文件**（含manifest），`git ls-files` 为 **0**；`git cat-file -e HEAD:.../review.md`、`.../formal-gates.jsonl.txt` 均失败。manifest内127项本机哈希均匹配，证据尚在；**存在于本机不等于归档入库**。[证据留存审计](review-evidence-durability.json.txt)。R9–R16也仍为未跟踪目录，只有R2/R8报告进入历史版本。

归档计划T-08和merge收据依赖R17结果、冻结SD与原始数据。新检出无法取得这些工件，清理未跟踪文件会丢失最终独立review依据，违反merge技能的“untracked docs/ = not archived”收尾要求。

**需收尾**：核对、登记并提交被交付收据直接引用的R17证据及必要历史依赖，检查新检出可解析；不用把target/binary等临时产物混入。台账HEAD仍v57而工作副本v60且含797，仅作事实记录：本仓README把它定义为兼容期本地投影，故不把“未提交本地ledger”单独等同于canonical未沉淀；其真实格式错误见PM-01，tracked/local投影归属需在最终收据说明。

## F-797-PM-03 / P2：交付配置超出R17覆盖，最终门禁证据复用依据不成立

`1799f86ca` 在R17之后修改 `.config/nextest.toml:31–40`，把 `abi_recipe_formal_turnaround_hard_gates` 加入default-filter排除。合入收据明确两次完整日常档该门失败，单跑31.4s通过，然后排除该门的日常档5313/5313通过。[配置差异](post-review-config-diff.txt)。R17的pass绑定旧配置，不能自动覆盖该修改；历史收据未见对新增规则的补充独立复审。

这并非要求恢复并行计时：`verify-autoabi-797.ps1:241` 的父侧abi_recipe仍用libtest及 `--test-threads=1`，tf也未排除；串行归属有实际入口，断言本身未变。缺口在于该验证分档改变后的受影响独立审查与冻结证据未闭合。

此外收据引用的最新完整closeout文件 `docs/reports/plan797/verify-acceptance-20261020-145533.md:4` 记录的HEAD是 **`cf9e4c7cd3cb8d94c06c917a84e7755356ba9c08`**；该HEAD到delivery之间cache.rs及t05测试**确有修复差异**，并非仅docs rebase。[差异统计](receipt-head-to-delivery-diff.txt)。该收据原始路径位于已删除工作树target，没有源码字节/dirty输入绑定，无法只用“代码/依赖/测试配置未变化”证明覆盖最终冻结提交。R17已重新验证受影响代码，不抹消本项merge-owned完整验收的证据缺口。

**需收尾**：补充新增分档的独立审查；绑定最终交付代码/依赖/验证配置，提交可复验的独立ABI完整套件/真实三腿、正式双硬门与实际兼容门收据。已有确证未变的部分可明确映射复用；缺失的受影响数据需补跑并保全，不能把排除后的日常全绿当成被排除硬门通过。本报告不否认记录中的单跑曾执行，只指出目前不能据现存绑定与工件证明完整冻结门禁已收口。

## F-797-PM-04 / P2：归档后引用与当前状态未收尾

归档文件 `docs/plans/archive/797-autoabi-v01-bidirectional-probe.md` 保留原活动目录的相对链接：**81个链接中80处失效（包含重复）**。例如 `attachments/797/review-r17/review.md` 被解析到不存在的 `docs/plans/archive/attachments/...`；`../specs/...`、`../design/...` 同样少一层。即使补交R17文件，这些链接也仍失效。[逐行链接审计](relative-link-audit.json.txt)。

`docs/specs/plans.md:5` 中auto-abi导航的相对目标 `../auto-abi/` 指到不存在的 `docs/auto-abi/`，应按本文件位置导航到specs内模块。归档Plan `:411` 仍把reviewed8/9及T09 pending称作“当前结论”，与frontmatter/merge/cleaned的archived9/9矛盾。

**需收尾**：仅修正本计划归档移动带来的相对链接及规范导航，把R17段标为历史，写明最新合入后状态与未完成核查项。保留所有历史finding/验收/门禁失败，不篡改旧轮结论。

## 跨计划批量回归：已有未完成工作，不能误报完成或重复启动

797 cleaned收据用“超过48小时、813/814/815/816/817/797均已合入”解释到期，不准确。核查时距812最后成功收据约23.5小时，797%5=2；814/816/817尚未整计划交付。**但815-A确已于 `e609caf1a` 实际合入，815可触发%5规则；不能仅看工作树仍在就断言815未落地或批量未到期。** 本轮早期commentary对此已更正。

已有 `docs/plans/attachments/815/t14-closeout/batch-status.md`/`abad97371` 记录：tf完整6270/6270通过；tt=5671通过/3失败/1超时，隔离归因pending；tb中断未完成；last-success仍812未推进。[本轮批量状态快照](batch-state.json.txt)。后续应复用该批进度，在主检出独占窗口归因并完成tt/tb、覆盖新合并窗口后写成功收据；本次没有启动并行重型负载。这是现有跨计划待办，不把它伪装为新797实施finding。

## 路由与边界

原代码review结论保持，归档终态不重开；**T09的“所有收尾均完成”需要上述实际核验修正**。优先PM-01恢复真实台账读取，再完成证据归档、分档/冻结门禁与文档引用收口。若需要代码或测试配置改动，走独立fix-forward工作树/计划与受影响复审，不在master上改实现。核查只新增本目录证据和KNOWN-DEBT登记，未改canonical/ledger/测试配置/归档Plan，未触碰其他计划WIP。

检查可复跑：[audit.py](audit.py)；[基线/清理/等价摘要](audit-summary.json.txt)。生产台账的读取负控是独立现有真实loader调用，命令与身份另存，不由JSON镜像断言代替。
