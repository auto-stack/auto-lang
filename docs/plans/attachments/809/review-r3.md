# PLAN-809 R3 — R2 问题闭合复审（needs_fix）

日期：2026-10-19，Asia/Shanghai。stage=review，review_scope=final（按用户重点请求做定向闭合复审），plan_revision=1。
reviewer=当前 Codex 审查上下文，独立于实施、延续 R1/R2；未创建其他 agent，未虚构模型或 session 身份。

## 受审基线与结论

- reviewed_commit：`ef6f05cf439389d92ccbd9f1ca5ae4206abac962`。
- worktree：`D:/autostack/.wt/lang-809/auto-lang`；branch：`codex/plan-809-dev`。
- 修复比较基线/R2：`3b733aa5d667f3fb14a0a2b87855de215797fde6`；Plan base：`2046a025b9a445d35a02da2c72a1613259a0cb60`。
- auto-down sibling：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场及结论前实现树clean，结论前兄弟树clean；HEAD/分支/base/依赖未漂移，修复diff --check过。
- 主检出 Plan、handoff、规范输入及冻结候选SD、源码/锁/证据hash见 `review-r3-baseline.json.txt`；候选SD副本为 `review-r3-spec-delta.md`。

**没有全部闭合，结论 needs_fix。** 交接自己明确 R2-06 的 SD/792 正式收据仍待补，本轮不启动另一轮完整最终验收。
但用户明确要求重点核查修复，因此实际重放 R2 原例及正控，并定向检查修复后的声明解析/生成/上下文路径；不以“已知F06未完”代替其余问题的核实。

## R2 六项闭合结果

| 原 finding | 旧例/原位置 | 完整不变量与当前结论 |
|---|---|---|
| F809-R2-01 P1 Ninja实例身份 | **CLOSED**：app-first发override.c不发base.c；lib-first及embedded-only两正控也与冻结旧循环相同；永久回归实跑通过 | **CLOSED（本次声明矩阵）**。core用BuildOrderStep(item_index,from_dep)，adapter按索引取首次实例，未再按rename猜对象 |
| F809-R2-02 入口与二次读取 | Ark run原例现在Ok且产物名来自request；Vue请求不变/磁盘后加dep原例不再导入Foreign；Jet/Ark build新入口及Automan分派已接请求 | **PARTIAL/OPEN**：sidecar_spec新增却无生产调用；真实Rust生成仍读磁盘侧车。另核心sidecar解析漏Block旧形态。见R3-01/02 |
| F809-R2-03 投影/产物与上下文 | 原unknown/sidecar节点保留、flag=true；front节点形态新旧目录相同；仅中文标题两臂均Some。mystery多参数在求值props的_arg0/_arg1保留，不把raw_args=None独自判为丢失 | **PARTIAL/OPEN**：实际输入清单忽略请求front，gen/member-run等调用仍默认mode/空output，未投影本次有效选择。见R3-03 |
| F809-R2-04 工具发现/原V05矩阵 | **假绿旧例CLOSED**：PATHEXT发现修复，提交的VueBuilder成功marker及exit3反控本轮真正执行，缺工具改为panic；Cargo真实构建也执行 | **PARTIAL**：原完整V05仍缺已约定的真实Ninja成功产物/不可写输出等验收；交接没有新收据。不能把探测修好等同原三Builder效果矩阵全部闭合；本轮未虚称运行 |
| F809-R2-05 786围栏与health | 围栏红例关闭：12项原定向+另选新增parity1项均通过；原HashSet/RequestOrigin/base_name/core两条等警告已清 | **PARTIAL/OPEN**：仍有本计划原root_dir告警及本轮新死代码/未消费helper告警，见R3-04；零新告警声明不成立 |
| F809-R2-06 SD/发行与交接 | 诊断表确实补了漏检原因和设计/oracle调整，对Ninja身份/工具发现的调整有效 | **OPEN（自认未完）**：SD-02/03未随修复更新，正式792 fresh/locked收据仍无。不是merge-owned裸t可以代办的工作验收；不批准延期 |

已关闭的原例不再列为原缺陷；下面区分仍开放的不变量与修复新回归。

## 本轮确认的缺口

### F809-R3-01 / P2：sidecar请求读取helper未接生产，真实生成仍从磁盘取声明

归属：R2-02残留；AC-02/04，T-06，V-02。

`crates/auto-man/src/rust_ui.rs:3283` 新增 `PacSource::sidecar_spec`，但没有任何调用点；开发生成`:585`、交付生成`:5382`等仍直接 `load_sidecar(project_dir)`。
声明纯解析移入core只解决规则归属，未解决“本工程声明来自请求”的消费不变量。

实际生产探针直接调用 `generate_rust_ui_with_request`，三个隔离项目相同App与extra.rs：

| 请求声明 | 磁盘声明 | 期望生成extra.rs | 实际 | 返回/主产物 |
|---|---|---|---|---|
| 有sidecar | 无 | 是 | **否** | Ok，main.rs存在 |
| 无sidecar | 有 | 否 | **是** | Ok，main.rs存在 |
| 有 | 有 | 是 | 是（正控） | Ok，main.rs存在 |

这排除了“没有生成/临时工程语法错误”导致假差异；来源反转说明真正读取来自磁盘。原始日志 `R3_SIDECAR request_only/disk_only/both` 可直接复放。
修复：所有携带PacSource的生成/交付入口消费该源上的声明，再执行合法FS复制；覆盖已有node/obj/嵌套形态，不以加一个无调用helper作为闭合。

### F809-R3-02 / P2：sidecar规则迁移漏掉旧Value::Block列表形态

归属：R2-02修复引入的支持面回归；AC-01/03/04，T-03/06。

旧 `auto-man/src/sidecar.rs:112` 的read_str_list接收 `Value::Array | Value::Block`。
新 `auto-man-core/src/sidecar.rs` 的str_list只接Array和Str；Block落空。
独立构造合法数据层Node：rust_sidecar.modules=Block(["m:extra.rs"])；旧已声明规则应得一个模块，新 `sidecar_decl_from_root` 实际 modules=[]（日志 `R3_SIDECAR_BLOCK`）。
这不是扩大语言语法或新规模要求，而是保留原迁移函数明确接受的数据形态；Array节点正控与core单测通过，不覆盖Block。
修复：恢复原支持域、依同一输入值对拍Array/Block与缺省控制；删除或仅在明确oracle测试中保留旧提形副本，生产规则保持单源。

### F809-R3-03 / P2：input_sources枚举与选定front不一致，调用上下文只在部分入口填充

归属：R2-03残留；AC-02/03，T-06，V-02。

`generation_request_adapter.rs:258-269` 通过 `resolve_front_dir_public` 从src/front、source/front、front依次猜目录，忽略已经投影的front_node_form/front_decl。
独立工程声明front="custom/front"，该目录有app.at和pages/child.at，另设src/front/wrong.at正交干扰：实际input_sources仅包含 **src/front/wrong.at**，没有真正front的输入。
只枚举首层也与生成器递归消费面不一致；本轮不要求新规模承诺，只要求清单对应已知实际输入。

调用层还存在两类原问题：

- Automan::gen 构造成员请求`:1298`、自身请求`:1498`仍调用request_of，固定Generate/None；实际output参数、delivery_root和本次选择并未进入这些请求。
- workspace run的成员请求`:2035`仍用同一默认Generate入口；新增gen_request_run只覆盖部分单工程run分支。render_override/实际选定后端仍未应用到请求投影。

修复：基于请求的有效目录/调用参数传递观测结果与模式/输出/有效后端；避免再用第二套路径猜测。永久验收需覆盖自定义front、节点形/双目录、递归实际源、gen/delivery与成员run，核对实际调用而非仅调用request_of_in的helper测试。

### F809-R3-04 / P2：围栏已绿，但零新增warning的health部分仍未闭合

归属：R2-05残留及修复新告警；AC-05，T-03/08。

本轮实际编译仍出现：

- `vue.rs:2665` VueReads::scan的root_dir unused（上轮原位置项仍在）；
- `sidecar.rs:71,112` 的fill/read_str_list已无生产调用，dead_code；
- `vue.rs:3553` 的collect_dep_front_dirs不再使用；
- `rust_ui.rs:3283` 的sidecar_spec不使用（也与R3-01根因一致）。

这些是本计划创建/本轮迁移形成的告警，不把完整语言库411条历史警告算给809。
19个本轮修改Rust文件scoped rustfmt检查全部过；本轮core46/model38编译未报原sel_from/same告警。
修复：清理或正确门控oracle/测试辅助，真正接消费；不要以allow(unused)遮盖未接线。重放相关定向health后再声称全清。

## 证据、复跑和限制

| 检查 | 本轮实际结果 | 持久证据 |
|---|---|---|
| core/model定向lib | 46/46 + 38/38 | review-r3-core-model.log.txt |
| 管理/Builder串行 | 8/8，含真实cargo、实际Vue命令成功及exit3负控；不再提前跳过 | review-r3-management.log.txt |
| R2原生成/786定向 | 12/12 | review-r3-generation.log.txt |
| 修复新增parity单项 | 1/1（与上行无重复选择） | review-r3-extra-parity.log.txt |
| 原R2独立生产反例/正控 | Ninja三态闭；Vue目录/zh-only/Foreign读源闭；Ark run闭；真实Vue正/负控通过 | review-r3-probe.log.txt |
| 新状态组合 | sidecar双方向+正控、Block列表、custom-front输入清单反例 | review-r3-extra.rs、review-r3-probe.log.txt |
| scoped格式 | 本轮19个Rust文件全过 | review-r3-format.log.txt |

`review-r3-probe.rs`由未改动的R2原探针（只将main重命名以包装调用）+`review-r3-extra.rs`组成；原反例源码hash与R2证据中的hash核对一致。
`review-r3-replay.ps1`在809原树重放，先要求当前auto-man生产库已构建；打印实际rlib哈希。探针exit0只是程序运行完成，不代表打印的所有不变量成立。
未知节点多参数这次看到求值props的_arg0/_arg1保留；**不把raw_args=None单独当作新字段丢失**。R2原REQUEST打印的通用display_title仍None，但旧Vue产物标题已经按其独立语义修好，本轮按真正消费产物关闭该旧例。

未重跑正式792全集、独立fresh/重定位locked、777、全后端CLI/工具成功失败矩阵；R2旧证据不伪称为新提交实跑。
原因是本次明确的闭合焦点以及交接仍承认work-owned项未完；这些required项保持pending。
没有运行整档t/tv/tt/tb/tu/taa/tf，没有修实现、发布Specs/ledger、合入或归档。

## AC / SD / 状态

- AC-01 partial：Ninja身份不变量已闭；sidecar提形迁移还有旧形态回归。
- AC-02 fail：sidecar真实读源与调用上下文仍不完整。
- AC-03 partial：图/786边界过，Block与实际输入清单仍有差异。
- AC-04 partial：本轮真实Cargo/Vue通过，未等同完整三Builder/发行验收。
- AC-05 fail：health与规范/证据尚未闭合。
- AC-06 merge-owned pending。

SD-02/03在修复提交中未更新，旧“全量/无丢失/sidecar跨文件延期”等陈述与代码和当前合同仍冲突；R2-06不予关闭。原metadata四个路径及GOAL-003保留，不表示SD通过。

outcome=needs_fix；status=executing；T05本轮独立验证完成，改为完成；T01/02/04保留原完成记录；T03/06/07/08仍开放，T09待最终审查，T10/11待merge；current_step=4/11（入场frontmatter写8但实际任务行只有3勾，本轮纠正）。
next=work在同一树完成R3-01..04、原R2-04完整矩阵与R2-06；更新诊断/闭合表区分原例和完整不变量，提交后再审。
当前诊断表的Ninja身份/工具发现调整已被实跑验证；sidecar“单源且消费”、输入清单、全部新告警清零的闭合声明需撤回并补路径证据。

pending_closeout_gates：最终code/acceptance/Spec pass后，merge在冻结提交跑裸cargo t及必要包/实际触面档，再执行最终777/792并集、规范/ledger/归档/已合入接口SHA交接及wt-guard clean后清理；本轮不授权跳过任何门禁。
