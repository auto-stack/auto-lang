# PLAN-815 M815-L / T-16 复审 L2：needs_fix

2026-10-20；r3 phase复审，不授予整Plan reviewed。

- reviewed_commit：`1082bde52e41d602409ead78a9753277feb6b3f1`；repair_base：`feacf79e8a9c52575a6f64dae3e068a3cc3ae1c7`。
- 原树：`D:/autostack/.wt/lang-815/auto-lang` / `codex/plan-815-dev`；主合同为main `a9d81eb26` r3。全量输入/schema/真实loader/脚本指纹见baseline.json.txt，进入/结束实现均clean。
- 限制：本/root会话曾参与前期合同及旧守卫候选，不声称另一独立上下文/模型。新修复从实际代码、真实store及独立反例重建；未创建agent，未修实现/合入。
- 生产ledger仍v57，SHA `c24b174ee3c0eba25555c0a31f0c988a0a465af880c99388bf0f537925b03038`；所有apply仅操作原树target临时副本，生产零写入。

## 六项逐项关闭表

| 原 finding | 原例闭合 | 完整不变量 / 本轮结论 |
|---|---|---|
| F-815-L1-01(P1) CLI退出码 | CLOSED：真实CLI错source→2，探针失败→1且原件精确恢复，旧freeze重复→2且不改文件 | 原返回值丢失已修；最终指纹错仍假成功为本轮新增相邻路径F-L2-01，不借旧例通过覆盖 |
| F-815-L1-02(P1) 并发/回滚 | 原外部更新两例CLOSED：checkpoint间→4、失败后→1，外部v777均逐字保留 | **PARTIAL / OPEN**：写到一半失败留下本运行残文件，详下 |
| F-815-L1-03(P2) 前像/删除保护 | 原8个related前像、source不可变、ID改名行、原title未声明删除均独立CLOSED | **PARTIAL / OPEN**：213处status缺失仍记null、24个legacy删除未记行，section值保护漏检 |
| F-815-L1-04(P2) schema/typed验证 | CLOSED：负u64/bool/越界/非字符串tags/缺区/type错配均拒绝；真实loader必填并核对六区身份，正常路径可加载往返 | 本轮未见相同不变量新缺口 |
| F-815-L1-05(P2) checkpoint/version | **代码 CLOSED**：真实CLI57→58→59，两候选分别绑定；B精确SHA吻合 | **收据残留 OPEN(P3)**：报告§7第6项仍给相反的“不递增”待核准规则 |
| F-815-L1-06(P2) source指纹错 | CLOSED：报告实际命令source/A/B/conversions四hash逐一匹配新summary，正常CLI完整通过 | 不再复用旧错指纹 |

## 阻断 findings

### F-815-L1-02（续开，P1）：本次部分写失败被误判外部更新，损坏账本不回滚

位置：`scripts/plan815_ledger_recover.py:665-679,740-756`；AC-10/12，V-L3原“写失败”路径。

独立反例使用真实`_ledger_lock`与真实文件句柄，在第一次write仅写入/截断257字节并flush后注入OSError。apply返回1，但账本剩257字节、exact_original_restored=false。日志将本运行的半写内容称为“外部更新”，rollback-skipped。原因：回滚只承认完整A/B字节，且wrote只在整个step返回后append，部分写阶段不被识别为本运行所有。

原“探针失败后回滚”只覆盖已完整写好的候选，不能证明写入本身失败安全。修复须在持锁写入路径管理本次部分写的归属，写失败仍能恢复精确原件/前checkpoint（最终按合同回原件）；恢复本身失败必须准确报告己方残留与恢复收据，不能假称保留了外部内容。保留旧外部更新不覆盖反例，补A/B部分写、truncate/flush失败和失败后的准确归属/原件断言；不要求新建通用事务框架。

### F-815-L1-03（续开，P2）：原值审计/声明保护仍有实际缺口

位置：`scripts/plan815_ledger_recover.py:295-299,321-342,445-500`；AC-10/11，V-L1..3。

1. status先赋`it["status"]="empty"`，随后才判断`"status" not in it`，分支永远为false。真实v57中213条缺失status的转换original仍为null，无法与显式null区分（例如reports/P789-M1）。
2. `pop(created/last_modified)`的两条已存在store时间字段分支只写sidecar、不记录转换，显式null也被pop而不记行；实际24个legacy字段删除没有转换声明（含P745-4/P741-2等非null数据），保护器又无条件豁免这两个字段删除。旧8个related反例虽过，原“逐项声明原值/删除”不变量未闭。
3. 未检查section的声明值是否与候选一致。将候选reports.title替换成未声明值，protections仍[]。该负控是已有“section越界保护”的断言验证，不是新增产品能力。

修复：在赋值/pop前冻结存在性及旧值，真实记录所有移动/sidecar/删除；删掉legacy名的无条件豁免，核对候选值/缺失与转换行，包括section/root保护边界。补缺失/null/合法有值、已含created_at+遗留字段、显式null遗留字段和section改值的区分性控制。新转换hash受影响，须重冻后复审，不能沿用0a1e1cbb。

### F-815-L2-01（新增相邻路径缺口，P2）：最终SHA不符仍applied=true / exit0

位置：`scripts/plan815_ledger_recover.py:775-787`；AC-10/11/12，V-L3/V-L4发布身份门。

真实CLI包装探针先调用真实store读取B并输出合法摘要，再模拟外部更新修改一个title（version59/六区1098项不变）。命令观察到`after_matches_checkpoint_b=false`，却仍输出`applied=true`并exit0。外部内容确实保留，不存在数据覆盖；错误是把与核准候选不符的结果报告成恢复成功，自动化会误放行。此例原退出码丢失已修后仍存在，是不同失败出口；不据此声称该行为由本轮修复新引入。

修复：把最终读回hash/身份结果设为实际成功门。观察到冲突时非零并保留外部内容、准确记录该checkpoint状态，不继续发布成功收据；补真实CLI“最终probe后外部更新”负控及完全匹配的正控。不是要求在任意未来时间持续锁定文件，而是不能无视本函数已经观察到的指纹不符。

## 收据残留（F-815-L1-05，P3）

`docs/plans/attachments/815/t15-recovery/report.md:95`仍写“恢复不递增version…彩排幂等跳过”，而§0和新代码已为57→58→59。§7标题为“待T-16裁定/核准清单”，并非明确标记的历史段，不能同时作为当前操作规则。应更正为双checkpoint实际规则，读回检查整份收据的一致性。代码双版本缺口已关闭，本残留不等同于原实现仍不递增，也不是单独推翻已通过的正控。

## 已执行验证 / 覆盖

- 新候选A `d571dbe3f87289c180f7e0d8cbfbe519409162ec860ad8d069ccf9b71bef8f43`、B `33830b65bd58aebe8a15f94fb71906f64b56883d89140697e6496f32d8475063`、规范化转换SHA `0a1e1cbba1eb7d01ce3904d1da742d48bff961cfa2b107aba01dee41980cb961`均重现。
- 真CLI+真loader正控：57→58→59，最终字节精确B，六区1098项，真实load/save内容相等；重复旧freeze不写且exit2。旧并发两例、错误source、失败探针、schema负控和四hash命令对照均执行，见results.json.txt与extra-results.json.txt。
- 两工具自检与`git diff --check feacf79e8..1082bde52 -- scripts`通过；完整自检没有测部分写，status fixture一直有active，不能覆盖本轮新状态。
- AC-10 fail；AC-11 needs_fix；AC-12及T-17/T-14仍merge-owned pending。V-L1/2 partial、V-L3 fail、V-L4正常/旧freeze路径通过、冲突成功门fail。原A/R4未变证据保持，B原最终范围未到。
- 无新canonical产品Spec增量，见spec-delta-reviewed.md；无需Rust整档，未跑cargo t/tv/tt/tb/tu/taa/docs_gen。旧batch tt归因/tb仍另待主检出独占窗口。

## 两轮未闭的根因与下次送审前要求

| 剩余不变量 | 旧检查为何漏过 | 下一轮必须补的设计/断言 |
|---|---|---|
| 失败写入归属与恢复 | 测试只在“完整write成功”之后让探针失败，所有权只认完整A/B | 在真实锁/真实fd下切进write/truncate/flush失败，不仅在探针出口失败；准确区分己方部分写与外部更新 |
| 原始存在性与声明完整性 | 自检status总是active，legacy pop受名称白名单豁免，section只比ID | 缺失/null/有值三态；全字段删除/移动声明核对；候选声明值和section变异控制 |
| 已观察冲突不得成功 | 自检只注入checkpoint A后的外部更新，B后只走正常路径 | 在最终真实probe出口注入外部写，必须非零且保留外部内容；正常B精确匹配仍过 |
| 规则与收据一致性 | 只重写hash/§0，未校对§7当前裁定清单 | 全文版本/checkpoint规则交叉核对，旧规则显式历史化或删除 |

仍沿用r3目标和原树，不需要以降标/另开计划解决。work先修齐以上范围、更新闭环表和永久反例，提交新SHA与新转换后再T-16。T-15重开，T-16/T-17/T-14继续未勾，Plan executing **5/17**。没有核准任何生产写、合入、归档或清树。
