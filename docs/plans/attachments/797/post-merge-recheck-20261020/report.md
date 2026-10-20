# PLAN-797 收尾修复复核（2026-10-20）

**结论：PM-01、PM-02、PM-04 CLOSED；PM-03 执行/配置审查已补齐，仍有4份引用日志未入库，故整体尚不能记四项全部CLOSED。** 原归档状态 archived9/9 不回退，本轮不重开Plan或旧代码findings。

主检出 `D:/autostack/auto-lang`，入审HEAD `cb2194c2461a203712bd601c7d314ab35544fec3`；本上下文做过R17及上轮历史收尾核查，未实施此次修复、未创建代理，按真实输入重建本轮结果。修复提交 `401e9867c`（证据入库）、`8d2fda1ee`（链接）、`cf95b6431`（对账）；delivery `1799f86ca`至当前 `crates/experimental/.config/scripts/Cargo.toml` diff为空，原实现/测试/配置未改变。本轮不跑Cargo或整档；运行已有真实loader及已构建的具名worker测试二进制，无构建。

## 已关闭项

| finding | 独立核验 | 结论 |
|---|---|---|
| PM-01 台账格式 | 真实loader SHA仍555b1744…；当前生产文件SHA b06ce5fb90f08def65b5e5e698c5b912fafd7116884598f692ae400f685fea39，load exit0/v60/六区1102项；备份匹配原66451eb3…，仅12处预期字符串→epoch替换即可逐字得到当前文件；读取前后未变 | CLOSED；[读回/外科范围](ledger-readback.json.txt)、[真实输出](live-loader.log.txt) |
| PM-02 R17证据入库 | R17 128文件现全部在Git，manifest127项逐个哈希匹配；必要R9–R16历史链已提交。无需重跑未变的原代码验收 | CLOSED；[留存检查](evidence-durability.json.txt) |
| PM-04 归档移动/状态 | archive相对路径与specs/plans.md导航修正，当前结论已为archived9/9及对账态；余12处引用对应9个唯一已记录丢失的旧工件（R1/R4/R5/R6/R8辅助基线），均非新增路径迁移错误 | CLOSED（原迁移/现行导航缺口）；历史工件丢失事实仍在，不称所有历史链接可打开；[逐行剩余清单](link-audit.json.txt) |

## PM-03：执行和分档审查通过，留存仍PARTIAL

补充配置审查的六点可从当前源复核：日常default-filter仅增加一个具名排除，R17计时断言不变，tf不排除此门，verify父侧使用libtest `--test-threads=1`串行；tv原本不选择该计时测试。无需为审查再跑整档。只将其认作分档放置，不能用排除后的日常绿代替正式门。

重跑工件绑定 `8d2fda1ee`，至本轮HEAD仅docs变化：

- 第一次verify套件完整90测试通过（12+8+24+7+6+26+7，另worker/main与doc-tests为0），随后专用t06性能门失败1189.5ms；该失败完整保留。
- 第二次verify套件6/7 t06通过，concurrent-GC出现SQLite store-open锁失败；保留失败，不把该次整体称ALL-PASS。脚本两次均未整体ALL-PASS，按同输入的各面执行证据审查。
- 父侧4/4、完整ABI11/11，auto-test-case87、runner26+1既有ignored的本机日志均存在。正式61条JSONL已入库、31 body/30 reject且全ok；独立从原始行重算body P50/P95/P99=610/966/1182ms、reject83/98/99ms，既定P95硬门通过。[样本/日志结果核对](closeout-evidence-check.json.txt)。

针对历史concurrent-GC失败，使用原已构建test与worker（SHA记录于证据）重放具名测试，不构建或改源码。最初受限环境两次在callee produce报os error5；随后以允许写入临时测试目录的同一环境权限作对照：`abi_worker_gc_actually_reclaims`1/1通过，`abi_worker_concurrent_gc_same_store`连续两次各1/1通过（2.96s/2.96s）。本轮没有重现原SQLite锁失败，不用受限环境访问拒绝宣称新代码缺陷；也不把“背靠背执行”当已经严格证明的历史根因。历史失败应继续保留，受影响具名路径本轮已经独立通过。[受限重放](concurrent-gc-replays.json.txt)、[允许写入的正反对照](gc-unrestricted-controls.json.txt)。

**剩余同一证据入库问题（PM-03/P2）：** 下列4份日志被仓库 `.gitignore:44:*.log` 排除，`git ls-files`/`git cat-file -e HEAD:<path>`确认均不在Git中：

| README引用日志 | 内容 |
|---|---|
| `pm03-closeout-rerun/parent-abi-family-serial.log` | 父侧4/4及正式门 |
| `pm03-closeout-rerun/parent-abi-family-full-serial.log` | 完整父ABI11/11 |
| `pm03-closeout-rerun/auto-test-case.log` | 兼容87 |
| `pm03-closeout-rerun/test-cases-tests.log` | runner26+1ignored |

位置前缀均为 `docs/plans/attachments/797/post-merge-audit-20261020/`。README把它们作为实际执行证据并提供链接；本机可读不代表新检出可获取。原R17的128文件已经修好，不重开PM-02；这里是本次补交门禁新增工件的同类遗漏。[四文件SHA/忽略规则/HEAD缺失](ignored-log-identities.json.txt)。

**修正方式**：将4份日志以可跟踪扩展名（如`.log.txt`）提交并同步README链接，或显式纳入版本库；也可把必要的完整命令/结果/输入绑定整理为受跟踪的等价收据并更新引用。无需再次重跑已验证、输入未变的门禁。本次review不修改实现、台账、忽略规则或替提交者发布这些文件。

## 批量与最终路由

上一轮跨计划批量续作现有 `a93e4b922` 成功收口记录，last_covered_plan_id817；tt5685/5685、tb5296/5296通过，tf6267/6270，3项已单独登记known-reds/观察并有隔离复跑，未冒称tf无失败。本轮不再把815旧“tt未归因/tb未完成”当当前待办，也不以批量新记录代替上述4份797日志入库。

next：只补PM-03这4个执行工件的版本控制留存与引用，然后核对新检出可解析。其余3项关闭、执行与配置已独立核验；不要求重新实施、全套复跑或重开归档Plan。历史9个丢失工件按既有记录保留，不虚构恢复。
