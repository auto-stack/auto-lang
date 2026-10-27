# PLAN-814 R8 / T-10：B 最终复审就绪核查，needs_fix

2026-10-21 / stage: review / review_scope: **final / T-10 就绪核查（不是完整最终验收 pass）** / outcome: **needs_fix**。
reviewer_identity: 本聊天 Codex 复审上下文，未参与 B 实现，未创建 agent 或额外 checkout。

## 冻结基线与覆盖边界

- plan_id: PLAN-814；plan_revision: **3**；reviewed_commit: `780415083650ddbfdd34587baf7e6e93af25e9ee`；tree: `1920f4e260a6f0a8a2d742cdf803d10520041f06`。
- base_commit: A 发布点 `f6e07aceeeea3cb12698c747d61db5be43f7458c`；worktree_path: `D:/autostack/.wt/lang-814/auto-lang`；branch: `codex/plan-814-dev`；dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。原树/依赖初末 clean，见 [baseline-evidence.txt](baseline-evidence.txt)。
- 主检出初始 HEAD `e447b7470576882d3e3e36bc4404dcddc024c970`，期间前进到 `3baf255ffb9e1b30432acead2d76235954b9f2cf`（Plan816 R4 簿记）。814 主计划在写本记录前与 [冻结副本](plan-before-review.md) hash 相同，未覆盖其他会话改动。
- [handoff 原文](handoff-frozen.md) 明确承认 Vue 端未跑，仍把 V-03 与全部 work 标绿。实际提交已冻结，但已知必需 oracle 没有完成，不能用 review/merge 的整档回归替代。
- 依 [auto-plan-review](C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md) 的原文 **“Known unfinished work-owned criteria or required oracles return to work”**，本轮先核就绪与相邻交接问题，不展开约20万行迁移的又一轮完整最终评审。不据此将未独立验证内容判绿；下一次应在这些已知义务补齐后进入完整 T-10。
- 本次实际检查：冻结/依赖与任务状态、合同及拟议SD、四包check、五个代表性新/改Rust文件fmt、迁移清单显式身份核对。未运行 UI、fixture 行为测试、整档或合入；完整 B 行为/生命周期与生成修复仍未独立验收。

spec_inputs/清单 hash 见 [input-hashes.txt](input-hashes.txt)。拟议 SD 表与 B 交接正文冻结 [sd-frozen.md](sd-frozen.md)，SHA256 `9d34b34a9d1b3e0e912b1c7a200af8aec624437c1864681e3a647691313e9c4d`。未写 canonical Specs/live ledger。

## 三项 OPEN findings

### F814-R8-01 / P2：V-03 仍有明确未完成的双端验收

类型：原合同内未完成 oracle；影响 AC-03/AC-06、T-08及其T-09交接。

证据：[handoff](handoff-frozen.md) §3 的 V-03 是“✅（Vue端pending）”，§4 又称“Vue/Playwright 双端 parity 截图未跑”，把它列为“非本Plan面”。但主计划 §6 明确要求真实跨端沿 autoui-verifier 设施验证，V-03/AC-03要求真实生成/运行/行为保持。B阶段不能把当初A所保留的完整运行面继续归为非目标。

VM snapshot、两个生成工程 cargo build、792构建及unit测试各有价值，但没有 Vue 实际运行与同输入双端状态/交互对照，不能证明该 oracle。这里没有认定现有产品必然有视觉错误，也没有独立诊断“浏览器栈不可用”；事实是 work 自承未跑，没有获准取消要求。

修复：按现有 autoui-verifier 流程完成有代表性的 Vue/VM 同输入、关键状态/交互及截图/日志对照，绑定冻结SHA/输入/运行方式；无需新增逐像素完全一致或无关全组合规模。若确实受环境阻挡，提交可复现的环境阻塞与剩余矩阵，保持 pending，不勾 T-08/T-09 或声称完整就绪。不得把它后置成 merge-owned 裸t/tu 的替代任务。

### F814-R8-02 / P2：SD-03“全文”实际仍为提纲，T-09交接不完整

类型：规范/收口缺口；影响 AC-05/AC-06、T-09。

[handoff §5](handoff-frozen.md) 标“SD-03（新沉淀全文）”，下面只有“包内容/canonical依赖/装配单点/资产与动态管线归属/缺席”等待展开的短语；其他SD也是“补X”的工作提纲，没有可审定的规范正文、相对当前A Spec的精确 before/after 与逐目标操作。实施分支 `docs/specs` 无拟议diff；这不要求提前写 canonical，而是交接没有提供声称已有的完整备稿。

当前 `docs/specs/language-sdk/project.md` 与 `docs/specs/auto-ui-compiler/project.md` 已由A实际发布，主计划SD-01/02仍沿整Plan原始add叙述；B须以现行A版本为输入表达增量，而非覆盖A已有规则。新 `docs/specs/auto-ui-runtime/project.md` 的完整提案还未提供。交接§5仍写“T-14/T-15”也误指已结束的A阶段，本轮最终审查/合入归T-10/T-11/T-12。

修复：在附件或handoff中补可直接审定的B规范正文/拟议diff，逐目标标当前源hash、保留/修改/新增规则、理由及AC；包括 runtime公开能力/feature及纯SDK边界、显式装配与r3登记权、单源状态/生命周期、资产与测试所有者、canonical导入/manifest/发行和旧兼容支持。两项生成器行为修复应具名说明与反例，不能仅称纯搬迁。同步frontmatter影响元数据和既有A文档的增量操作，T-11/12按审定结果沉淀；review不替实施补写尚未验证的整包事实。

### F814-R8-03 / P3：健康“零新增”声明未成立，新fixture格式也未通过

类型：B迁移/验收资产健康问题；影响 AC-05/AC-06、T-07/T-08/T-09。

本次 `cargo check --offline --locked -p auto-lang-core -p auto-ui-compiler -p auto-ui-runtime -p auto-lang` **exit0**，[完整输出](check-four.log.txt)。但新 runtime 根部两个提取后增加的内部再导出产生未处理warning：

- `crates/auto-ui-runtime/src/lib.rs:66`，`dynamic_build::load_ext_imports_for_vm` unused import；
- 同文件 `:72`，`stmt_symbol_name` unused import。

它们是 T-04 新增根再导出点，真正内部调用在 dynamic_build 模块中；A原根函数定义不产生这两条“未用导入”告警。应删除不需要的再导出或按实际消费者组织，不用 blanket allow 掩盖。这里只锁定两项因果明确的新warning，不把runtime全部237条历史warning称为新增。上层残留 `VueGenerator/VueMode` 与两个API辅助函数的3条warning在R6/A日志已存在，本次明确不重开它们。

对新增consumer、import_policy、V-02 pin、runtime lib/dynamic_build五文件做 `rustfmt --edition 2021 --check --config skip_children=true`，**exit1**：[输出](fmt-selected.log.txt)。`test/fixtures/modularity/ui-runtime-consumer/src/lib.rs:24` 的 View builder链未格式化；其他四文件未报diff。git diff --check exit0不能代替rustfmt。

work所称“637→479/净-158”是聚合下降，不证明新增集合为空。修复上述新项、格式化fixture并按匹配配置提供B新增warning归因/清偿记录；不要求清除全仓既有warning。

## 未误报的迁移证据与待验范围

提供的18件清单已复制到 [t07-frozen/lists](t07-frozen/lists)，输入不随之后work改写而丢失。原 [工作验证器](work-selection-checker-frozen.py) 仍有唯一叶名自动映射；对无关模块同名替换返回接受，因此不采信其任意输入“健全”宣称。[工作收据](work-selection-receipt-frozen.log.txt) 本身也没有 selftest输出，不将未保存的负控口述当独立执行。

本次 [限定清单审计](selection-inspection.py) 不使用叶名推断：复用R6已验的原→A完整映射，加B显式路径迁移，并核实原lib与当前ui_tests挂载，为 `p614_tree_vm_probe`、`p618_filter_tree_probe`、`plan042_scan_probe_tests` 三个根模块补确切前缀。提供的九档清单最终 **missing=0/ambiguous=0**，[输出](selection-inspection.log.txt)。最初未补源码pin时的7项“待映射”是已找到真实迁移的模块，不是实际丢测；不新增漏测finding，也不复活A已闭R3-05。

这仅证明所供t07清单的旧ID可核销，不是当前780415083的nextest重新采样，也不是测试执行；T-08之后新增生成器测试的当前集合、删除负控、feature/alias实际配置仍待最终T-10按冻结输入核对。runtime消费者/API pin源码已读，不能据源码把本轮未跑的5/4测试记独立通过。

## AC、状态与下一步

| 范围 | R8结果 |
|---|---|
| AC-01/02/04 B完整行为、生命周期 | pending完整T-10；仅迁移图/入口与消费者断言读审，work报告不替代本轮行为验收 |
| AC-03 / V-03 | partial，明确缺Vue/VM双端对照；不得记全通过 |
| AC-05 | partial，提供清单的身份核对通过；规范正文与健康余项未完成 |
| AC-06 / T-10 | needs_fix；不授最终pass、不勾T-10 |
| AC-07 / T-11..12 | merge-owned pending |
| AC-08..10 / M814-A | 已发布历史保留，不因未合入B的就绪问题撤销A发布或重新封锁下游已发布接口 |

next_responsible_role: **work**，原唯一树一次补齐R8-01..03，提交实际修复、更新handoff与精确SD/覆盖矩阵，再申请完整最终T-10。没有更改r3合同；如欲取消必需验收，须另行正式修订，不能用“环境”自动缩面。

重开 **T-07/T-08/T-09**，保留T-04/05/06已记录的迁移/公开面/非UI消费者实施进度，但不据此授AC-03/04完整通过。特别核对：受审frontmatter写10/15，任务表实际勾了 **12项**；重开上述3项后按真实勾选重算为 **executing9/15**，不是按错误计数减成7。T-01/02/03/13/14/15的A历史进度保留，T-10..12仍未勾。

pending_closeout_gates保持主计划 §6：最终代码/验收/SD审查通过后，由merge在冻结提交统一裸t+适用tv/tu/tt及必要docs/schema/book等；tf主检出到期单实例，已有aavm豁免按用户裁定范围处理。本轮不把handoff“tt按需”擅自当作修改主合同的授权。原审定之后若合流/门红修码，要受影响复审和重冻。

本轮未运行整档、未操作GUI、未修实施代码、未合入或沉淀Spec/ledger、未归档/清树，不宣称B已验收或Plan814交付完成。
