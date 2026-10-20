# PLAN-815 M815-L / T-16 复审 L3：pass（阶段）

2026-10-20，Plan r3；仅M815-L phase，不授予整815 reviewed或交付完成。

- reviewed_commit：`0a65ee2a706a5c1320655c1874f14876f743cc3d`，repair_base：`1082bde52e41d602409ead78a9753277feb6b3f1`。
- 唯一实现树：`D:/autostack/.wt/lang-815/auto-lang`，分支`codex/plan-815-dev`；主合同读自主检出`9aaeb42b5`的r3。
- 审查边界：同/root上下文曾参与前期合同/旧守卫候选；未参与本轮实施修复，不声称另模型/独立会话，从提交、真实store和独立反例重建。未创建agent，不修实现、不合入。
- 已跟踪实现clean；另有未跟踪`patch_l2c.py`旧补丁生成辅助文件，已读回/指纹列入baseline，未被scripts/.cargo/实际入口引用，未执行、不纳入获准实现且保留未动。merge冻结前由owner处理工作区归属，不能顺手把它带入交付。
- 所有反例只操作原树target临时副本；生产ledger v57 SHA `c24b174ee3c0eba25555c0a31f0c988a0a465af880c99388bf0f537925b03038`全程未变。完整代码/合同/schema/loader/来源/候选指纹见baseline与final-baseline.json.txt。

## 四项闭合（原例→完整不变量）

| Finding | 独立结果 | 证据/完整范围 |
|---|---|---|
| F-815-L1-02(P1，部分写/恢复续开) | **CLOSED** | 原257字节切断/flush后OSError：exit1且精确恢复原件；A/B的write、truncate、flush六种一次性故障均注入成功、exit1、原件逐字相等。持续存储故障使恢复本身也不可用时，非零、明确报告“本运行部分写残留/恢复失败”、备份逐字完整，未谎称外部内容或成功；这是已约定的准确残留/手工恢复边界，不冒称硬件持续失败仍可成功写回。旧checkpoint间/写后外部更新两例继续保留外部v777，退出4/1，不被回滚覆盖 |
| F-815-L1-03(P2，前像/声明续开) | **CLOSED** | 原213条缺status误记null归零；真实缺失/null/合法值区分有效。原24个legacy删除未记录归零，显式null与store字段已存在的legacy副本均有行；原related、ID改名、原title删除控制保持通过。section候选值突变现在被拒。额外对全部原件中1273个实际改变/删除的既有字段独立比对转换original，零不符；A/B均1098唯一ID |
| F-815-L2-01(P2，最终指纹门) | **CLOSED** | 同一真实CLI包装真实store探针：最终B摘要后外部改title，exit4、保留外部内容、stdout无applied。正常真CLI仍exit0，最终字节精确B；旧freeze重复exit2且不改文件。观察到的指纹不符已成为实际失败门 |
| F-815-L1-05(P3收据残留；原双checkpoint代码上轮已闭) | **CLOSED** | 当前报告§7第6项已统一为57→58→59，各checkpoint独立写与真实验证；§0/§4/§6/§7一致，旧“不递增”的当前规则删除，四个命令hash匹配summary |

更早F-L1-01/04/06继续CLOSED；本轮重新执行错source拒绝、探针失败精确回滚、六组schema负控（负u64/bool/越界/非字符串tags/缺区/type错配）、实际四hash命令核对，未发现新actionable finding。

## 正控与核准身份

真实CLI（副本）执行57→58→59，每步由真实`SpecsStore::load`核对project/version/六区type和总项数；最终6区1098项，真实load/save往返内容相等；四个P815-A ID各一条，canonical source-sha256逐一匹配，未见漂移。

| 冻结对象 | 核准值 |
|---|---|
| source v57 | `c24b174ee3c0eba25555c0a31f0c988a0a465af880c99388bf0f537925b03038` |
| checkpoint A / v58 | `d571dbe3f87289c180f7e0d8cbfbe519409162ec860ad8d069ccf9b71bef8f43` |
| checkpoint B / v59 | `33830b65bd58aebe8a15f94fb71906f64b56883d89140697e6496f32d8475063` |
| conversions / 12187行 | `c0a4aac04c95eed2965f83eee2ee60a3f55645d72bce52c67bb2f92eaecf2949`（json.dumps ensure_ascii=False,sort_keys=True后UTF-8 SHA；不是美化文件的原始字节SHA） |

A/B候选与L2字节一致是经本轮重新生成SHA证实，不仅引用实施报告；转换变化已重冻。脚本修改与schema、真实loader及stage输入完整SHA见baseline。两工具自检、原例/相邻失败控制、全字段前像审计、scoped diff-check均执行。另以真实CLI复核source/A/B/conversions四种错误冻结值分别exit2且原件不变；同名异字节备份保持原样并拒绝；真实loader摘要version错误时exit1并精确回滚，见freeze-controls.json.txt。复用只限未变A/R4生产代码与精确Spec范围以及L2的历史设计说明，本轮不把未跑的整档算成通过。

## §7裁定清单（绑定以上转换，不推广为全库新业务规则）

1. reviews/P014-1→P014-1-R、designs/P014-1保持：核准。两个原记录仍在，只有声明ID变更，不丢历史；source hash未变且现存related/depends_on无P014-1歧义引用，后续原件变更须重查。
2. 缺失/null→empty；active/current→stable；delivered→done；accepted→approved：核准本次冻结原值到store表示的映射。这里只保留原有声明的表示，不用该映射证明历史验收；原值留表与逐字原件，不能据stable/done给815/其他计划新打勾。
3. 旧日期按记录精度转为UTC秒；无可用历史时间用0未知哨兵、completed_at不补造：核准冻结转换，0不得被解释为1970实际事件。原日期/存在性与移动路径保留表中。
4. store load/save会丢弃的未知字段：核准sidecar+逐字原件/转换的保留方案；T-17必须保全、记录这些附件身份，不能只发布最后的候选文件而遗失恢复链。
5. PascalCase SectionType：真实store确认；snake section id与Pascal type一一配对、六区精确身份保持。
6. 恢复+阶段upsert两个checkpoint57→58→59，各+1，分别真实读回，最终SHA不符非零保外部内容：核准。

## AC / owner_stage / 后续门

| 项 | 结果 | 边界 |
|---|---|---|
| AC-10 / T-15 work | pass | 恢复实现、候选、typed oracle、前像/保留/失败验收完成 |
| AC-11 / T-16 review | pass（本阶段） | 四项已闭、无新增，核准上述提交/候选/转换 |
| V-L1/V-L2 | pass | 六区1098项真实加载/往返，原值可恢复、合法/错误输入区分 |
| V-L3 | pass | 代表性A/B写失败、读回失败、外部更新/备份/残留控制；持续存储故障准确残留分支不冒称自动恢复成功 |
| V-L4（work/review范围） | pass | 固定四ID唯一、来源匹配、重复旧freeze不改/不增版本、真实副本应用；实际发布仍pending |
| AC-12 / T-17 merge | **pending** | 单写者窗口、正确workspace/source+schema+loader+stage四hash再冻结、处理未跟踪辅助文件归属，实际恢复与受管四项upsert、两个version/备份+sidecar+转换/真实读回与失败收据 |
| AC-09 / T-14 merge | **pending** | T-17完成后重新生成发布收据/来源与祖先终验，才能关闭阶段发布 |

Category A触面（仅恢复脚本与docs/plans证据；无crates Rust变化），依AGENTS不跑新增裸cargo t/docs_gen或其他整档；本阶段merge-owned pending的是T-17适用的冻结/单写者/实际应用/读回/收据门，不能把本次review pass当作这些门已完成。原T-14a裸t等A代码冻结门沿用已有绑定证据，若代码/依赖/配置漂移按影响重审。旧批量回归tf已绿、tt归因/tb未完成收据仍独立pending，不在本次复审中隐含清偿。

## 重放与最终状态

review_probe.py / extra_probe.py / failure_matrix.py / freeze_controls.py是独立重放入口；结果见results.json.txt、extra-probe.log.txt、failure-matrix.json.txt与all-field-audit.json.txt。L2的257字节故障改为一次性故障以允许恢复路径成功；持续存储故障另外明确测量并要求准确残留/完整备份，未弱化该边界。原生产账本始终不写，所有路径为target临时副本。

Plan保持executing **7/17**（T-15/T-16完成）；T-17/T-14未勾，原B任务/上游814依赖保持。原M815-A代码R4 pass保留，本次M815-L pass不等同于生产账本恢复或815整体交付。不合入、不归档、不清树。
