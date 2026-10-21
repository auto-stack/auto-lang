# PLAN-814 合入恢复核对（2026-10-21）

**最新独立核验：pass，F814-MERGE-01 CLOSED。** 修复 `f97a9e2a9` 已在master；5328项逐ID无缺失/无新增，迁移测试真实执行1/1 PASS。下文blocked为修复前历史，独立闭环证据见末节。

stage: merge；mode: archived recovery-only；plan_revision: 3；outcome: **blocked（测试选择缺口待对账/后续修复）**。

## 已验证交付与恢复范围

- 当前主库核对基线 `75b26c2536c3956305ac28fbb070366de163cc03`；实际B交付 `93a25c924d1beafab130588bad4d1041dc86e48c` 是其祖先。master reflog明确记录 `merge codex/plan-814-dev: Fast-forward`。
- 原review `6f1cac8cbe618e509bf85d5c000fa0c26f35df6e` 因rebase不是主库祖先；映射到 `a3f50d15b`。独立重建range-diff：20旧提交→17新提交，13个 `=`、4个 `!`、3个基线提交丢弃（不是原粘贴摘要所说的2个）。R8至R14全部修复提交为 `=`，当前RustGenerator文件与原review逐字节一致。
- 四个 `!` 为T04主体、T04收口、fmt与T05组合；app_registry/i18n_lookup当前单源在compiler，runtime pub use，符合现有主库815消费者和DAG。但“补丁相同的类型修复”不覆盖这些语义组合的测试归属，发现一项遗漏如下。
- canonical Spec、归档计划均已跟踪；ledger v64的五项P814条目存在，file/tags齐。未重复写Spec/ledger、未再次合入、未重跑已完成整档。
- 原收据的t5327/tv163/tu868/tt5699/tb5303与主库consumer5+4+5结果作为历史记录保留；批量回归提交75b26c253及其收据确实存在（tf6277/6278、唯一已知ffi_dual_019 flake solo绿；tt/tb全绿、无新红）。本次没有冒称重新执行这些门禁。

## F814-MERGE-01 / P2：合入组合后静默丢失一项会话启动测试

影响原AC05/T07选择面零丢失与T11并集变更验收。原受审代码的
`auto-ui-runtime ui::app_registry::tests::launch_three_real_apps_via_registry_resolver`
会实际运行。合入时app_registry移回compiler，测试体也原样迁入
`crates/auto-ui-compiler/src/app_registry.rs:1681`，仍为：

```rust
#[cfg(feature = "ui-iced")]
#[test]
fn launch_three_real_apps_via_registry_resolver() {
    use crate::ui::session::{DesktopSession, LaunchSpec};
```

compiler清单只声明`ui`/`ui-headless`，没有`ui-iced`，也不应反向依赖runtime的session。这一测试在compiler中无法被正常feature集合编译，不能靠完整测试档全绿证明它仍在执行。全仓同名测试仅此一份，没有runtime替代测试。

本次只枚举、没有执行测试：按当前`.cargo/config.toml`的t参数，将`cargo nextest run`改为`cargo nextest list --offline --locked --message-format json`，读取filter-match=matches的逐ID集合。与R10/R15沿用的5328项冻结集合相比：

| 项目 | 结果 |
|---|---|
| 原集合 | 5328 |
| 当前实际选择 | 5327 |
| 明确owner重写 | runtime `ui::app_registry`/`ui::i18n_lookup`→compiler根模块，共28项 |
| 另一个明确迁移 | old auto-lang `store_deps_scan_tests::extract_store_deps_ignores_comment_examples`→compiler `store_deps::store_deps_scan_tests::extract_store_deps_ignores_comment_examples`，已核销 |
| 最终未匹配 | 仅 `launch_three_real_apps_via_registry_resolver` 一项 |
| 无解释新增 | 0 |

测试数少一项已得到具体ID与失效门控解释，不是仅按总数猜测。建议后续修复将这项依赖runtime会话的测试迁到runtime测试层，使用compiler单源注册表；不要给compiler虚设ui-iced门或引入反向依赖。恢复测试后需独立审查、定向执行与相应冻结门，保持原三真实App/session断言，不将其删除或降为文档。

原R15 pass仍准确绑定旧受审提交；问题发生在后续 `!` 并集归属变化，不能声称原pass已覆盖它。B实际合入与canonical发布事实不回滚，但本次不能再确认“全部收尾验收通过”。遵循[auto-plan-merge](C:/Users/zhaop/.codex/skills/auto-plan-merge/SKILL.md)归档恢复边界："report blocked for reconciliation rather than routing an archive into work"。保留archived终态与原15/15历史记录，不把归档Plan重新送work；代码应由独立后续修复承接。本次不修实现、不建新计划、不重写默认分支历史。

## 已完成清理与归档文档修复

- 未注册残留 `D:/autostack/.wt/lang-814/auto-lang` 实查文件数为0；根及组无reparse point，fresh wt-guard clean。
- 仅结束明确归属该路径的旧验证进程：counter Vite PID39196（port33197、strictPort）、其子esbuild PID54876。首次删除仍受退出过程锁影响，进程退出后重跑fresh guard再删除成功；未停止任何其它会话进程。
- `D:/autostack/.wt/lang-814` 整组已移除；原开发分支、临时基线与兄弟worktree注册均不存在。旧收据的“残锁目录pending”现在已关闭。
- 原归档移动遗漏相对链接层级，50个唯一目标/65次引用无法从archive目录解析；原目标全部存在。仅给这些链接补一层`../`，不改历史证据内容，全部目标复核可达。
- 归档状态仍archived、原任务历史15/15；追加本恢复记录说明清理完成及新增对账阻断，避免覆盖历史实际交付。其它会话脏文档、未跟踪冻结证据及ledger备份均保留。

本次仅提交本报告和归档计划的链接/恢复簿记，不改canonical、liveledger或实现。新提交为纯文档，不触发整档、docs_gen或再次批量回归；既有批量收据保留。

## F814-MERGE-01 闭环（2026-10-21，L0 fix-p814-launch-test）

- 修复提交 `f97a9e2a9`（master ff-only，基于本恢复提交 `6ec93ed61`）：
  孤儿测试从 compiler `app_registry.rs` 迁出（留指针注释），新建
  `auto-ui-runtime/src/ui_tests/plan814_app_registry_launch_tests.rs`
  ——经 `crate::ui::app_registry` 再导出消费 compiler 单源注册表，
  三真实 App/session 断言逐字保持；挂载门 `all(test, ui-iced)`
  （runtime 真实 feature），不虚设 compiler feature 门、不引反向依赖。
- 验证（scoped，按 L0/修复轮口径）：选择面 **matches=5328 恢复**
  （nextest list 逐 ID，新 ID=`auto-ui-runtime
  ui_tests::plan814_app_registry_launch_tests::
  launch_three_real_apps_via_registry_resolver` 在集合内）；迁移测试
  真实三 App 启动 **PASS**（0.53s）；compiler `app_registry` 25/25；
  双包 `--tests` check 0 错；rustfmt 过。
- 清理：wt-guard clean → worktree/分支/组目录（含 auto-down 兄弟
  @c39ac7f9）全部移除。
- 本节闭合上文 blocked 判定：PLAN-814 收尾验收的最后一项缺口（原
  AC-05/T-07 选择面零丢失）恢复，收尾核验可判 pass。未触发整档
  （测试迁移 scoped 验证即 L0 门禁）；批量回归收据 75b26c253 覆盖
  窗口不含本提交，下次到期判定按新 last_covered 顺延。

## 独立闭环复核（2026-10-21，pass）

stage: merge；mode: archived recovery-only；reviewer: 本聊天Codex，未实施该修复；outcome: **pass**。受验修复 `f97a9e2a964198b366105cad68b246360a9c5211`，核对主库基线 `e4f655be17116ea3596d99d3b83e22465e502c64`；两者间仅本报告簿记，无实施漂移。

- 与`6ec93ed61`中compiler原件比对，去除模块层级带来的缩进后，迁移测试整个函数（含三个真实App、窗口数、标题与焦点断言）完全一致。新模块经runtime真实`all(test, feature = "ui-iced")`门挂载，消费compiler注册表的runtime再导出，没有新增compiler feature或反向依赖。
- 独立运行当前t别名的`nextest list`，以修复前同一5328项冻结集合逐ID对账：使用app_registry/i18n owner迁移、store_deps迁移及此测试的明确新路径映射，**old=5328/current=5328/missing=[]/added=[]**。恢复ID `auto-ui-runtime ui_tests::plan814_app_registry_launch_tests::launch_three_real_apps_via_registry_resolver` 确实为matches，不只核对总数。
- 独立执行 `cargo nextest run --offline --locked -p auto-ui-runtime --lib --features ui-iced -E 'test(launch_three_real_apps_via_registry_resolver)'`：exit0，run ID `ff5133c2-7955-4359-9b5f-7f6943a65859`，**1 test run /1 passed**，单测0.507s、测试段0.516s（另有本机构建时间）。1887 skipped是该定向命令未选择的其它测试，恢复测试本身实际执行。
- 三个改动Rust文件rustfmt检查exit0；修复是当前master祖先。Plan814原组与fix-p814-launch-test组、相关分支/worktree注册均不存在。compiler25/25及双包check保留work的验证记录，本轮不冒称重跑。

**F814-MERGE-01与本恢复报告的blocked判定正式闭合，PLAN-814收尾核验pass。** 归档终态与15/15保持，未重开计划，未重复合入或改canonical/ledger。批量收据75b26c253保持其原覆盖窗口，不声称已覆盖本修复；本轮只做定向执行、选择面与文档核验，没有运行整档。
