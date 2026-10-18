# PLAN-809 R2 final review — needs_fix

日期：2026-10-18（Asia/Shanghai）；stage=review；review_scope=final；plan_revision=1。
reviewer=当前 Codex 审查上下文（独立于实施，延续 R1 审查；未创建 agent，未虚构模型/session）。

## 基线和结论

- 同一实施树 `D:/autostack/.wt/lang-809/auto-lang`，branch `codex/plan-809-dev`。
- reviewed_commit=`3b733aa5d667f3fb14a0a2b87855de215797fde6`；修复父提交 `c41a4fe4cc6b87967959521bcb32e8963c9644dc`。
- base_commit=`2046a025b9a445d35a02da2c72a1613259a0cb60`；R1 baseline=`aa1d40e1d90b313b7dd6ba57b79123f5278ed90f`。
- auto-down sibling=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- 主检出当前 Plan 仍为 revision 1；受审时 T01..08 全勾。实现修复确实提交，入场实现树 clean；最终复核见 baseline。
- 源码、配置/锁、Spec 输入和候选 SD 冻结文件/hash 见 `review-r2-baseline.json.txt`；候选 SD 原样保存于 `review-r2-spec-delta.md`。

**结论 needs_fix**。本轮有实质修复，但 R1 的四个不变量尚未全部闭合，另发现修复引入的 Ninja 目标身份回归。
本轮完成了修复路径、图/模型、实际生成和构建反例的定向审查；不是仅据交接自报再做 readiness 判定。
未覆盖的完整三 Builder 成功/失败矩阵、全生成器产物矩阵和正式 792 验收仍明确 pending，不能授信为最终 pass。

## R1 闭合审计

| 原 finding | 已证实修复 | 完整不变量状态 |
|---|---|---|
| F809-R1-01 | Ninja 空输出万能断言已删；真实 cargo build 跑过；本轮独立 VueBuilder 真工具成功/exit 3 负控也跑过 | partial：提交测试因仅找 npm.exe 而跳过本机可用的 npm.cmd，Ninja 成功编译/不可写输出等原矩阵仍缺。见 R2-04 |
| F809-R1-02 | Automan 多条入口增加 `_with_request`，Jet run 等内层有接线 | OPEN：Ark run 丢请求、Jet/Ark build 仍旧入口、Vue 主 pac.at 第二读取仍存在；请求投影还丢节点/输出语义。见 R2-02/03 |
| F809-R1-03 | 核心新增 ordered_build_renames 和四个下载决策入口，生产有消费；核心45测试通过 | extraction 部分成立，语义未闭：Ninja 回查丢失“首次访问的具体依赖实例”，引入回归。见 R2-01 |
| F809-R1-04 | 777 基线 +5 合法边并重跑 strict 通过；候选SD四文件确已提交；本轮两个consumer locked消费通过 | partial：SD“零遗留/无字段丢失”与代码反例不符，正式fresh/locked及792证据仍未交齐，不能以delivery9单测代替。见 R2-06 |

## Findings（全部 OPEN；非批准延期）

### F809-R2-01 / P1：Ninja 按 rename 回查顶层目标，覆盖掉先遇到的依赖实例

类型：R1-F03 修复回归；AC-01/03/04，T-05/06，V-01/04/05。

`crates/auto-man/src/builder/ninja/builder.rs:188-214` 对 core 返回的 rename，优先在整个 `pac.targets` 找顶层对象，之后才找内嵌 deps。
旧臂按“当前 target 的 deps，接着自身”遍历，**第一次访问的具体对象**胜出；内嵌 deps 经过 merge_link，可能有与顶层同 rename 不同的 srcs/属性。
纯 rename 序没有足够身份信息供 adapter 恢复原执行对象。

独立反例：targets=[app, lib]；app.deps=[lib 的覆盖副本]，两 lib rename 相同；副本 srcs=override.c、顶层 srcs=base.c。
冻结旧循环通过同一生产 Builder 的 setup/target/finish 发射 override.c，新 build 发射 base.c。
日志：`NINJA app_first ... old_first_encounter_preserved=false old_base=false old_override=true`。
两个正控（lib-first 与 embedded-only）旧新均一致，排除探针一概报红。
这不是要求稳定 HashMap 全序，而是对一个已给定合法遍历次序保留旧语义。

修复：核心规划结果保留首次访问对象的可回查身份（如 target index + dep index），adapter 消费该身份；不要再全局按 rename 猜对象。
永久回归须比较真实 Ninja 输入/产物，覆盖先依赖/先顶层、共享 rename、link 覆盖；不能只断言 rename 字符串序。

### F809-R2-02 / P2：多条生产入口仍绕过请求，Vue 还回读本工程 pac.at

类型：R1-F02 不变量残留；AC-02，T-06，V-02。

- `ark.rs:760-773` 的 run_ark_project_inner 收 req 却直接 generate_ark_project；req 未使用并产生警告。
  实测：传合法 render=ark 请求、磁盘 pac 为 render=vue，`run_ark_project_with_request` 返回 Backend ark not found；说明读取实际来自磁盘。
- `automan.rs` 的 build 分派仍调用 `build_jet_project` / `build_ark_project`，对应 `jet.rs:1003` / `ark.rs:691` 仍调用旧 generate，入口 grep 只查 generate 名不能覆盖这条链。
- `vue.rs:3648` 的请求路径无条件 collect_dep_front_dirs；`:3521` 仍读 **当前工程** pac.at 并解析 local dep path，没有消费 req.dep_roots。
  独立实测：req.dep_roots=0 后仅修改磁盘 pac.at 加一个本地 dep，请求未变却把 Foreign 组件加入产物。
  这不是“读取 dep 自己的 pac.at 合并资源”的跨文件例外；主工程依赖声明仍由第二套读取规则决定。
- T01 contract §6.4 已明确 SidecarSpec 声明解析归 R-01 纯规则；候选SD将整个 load_sidecar 以“单工程外”延期并不与该约定相符。

修复：检查完整调用链和实际内层分支，把本工程所有项目字段先投影为合同，再给效果 adapter；dep自身资源/代码读取可保效果边，但不能用该例外豁免主工程声明。
以“请求不变、磁盘主声明改变”及反方向控制证明读源，覆盖 run/build/open/gen 各入口和已声明后端。

### F809-R2-03 / P2：动态请求投影丢节点/调用上下文，并改变 Vue 目录和标题选择

类型：原 AC-02/03 完整性与迁移等价缺口；T-02/06，V-02/03。

- `generation_request_adapter.rs:21-93,236-240` 只投影 Pac 字段和 props，未传原求值节点。
  有 app(front)、rust_sidecar 和 mystery_node 的实际已求值配置，经 request_of 得到 `raw_nodes=0, has_rust_sidecar=false, front=None`；未知节点及内容无保存位置。
  apply_evaluated_root 的 sidecar 测试为绿不覆盖实际生产 request_of；生产代码也未调用该 root 追加接口。
- `vue.rs:2704` 仅取 req.front_decl；未恢复原 parse_workspace_path 的 app(front) 优先规则。
  同一工程同时含 front/app.at 与 src/front/app.at，声明 app(front)，旧 from_workspace 选 ./front，新请求选 src/front（实际 front_dir 不同）。
- `generation_request.rs:253` display_title 在取 title_zh 前先对 title 使用 `?`；该函数镜像 Pac 不能证明它等价于旧 Vue 标题读取。
  实测仅 title_zh 配置，旧 index_title=Some("Only Chinese")，新=None；不是新增 locale 要求。
- dynamic 构造的 input_sources=[]、output_root=None、mode=Generate 一直是默认值，生产 adapter/Automan 没有补实际输入、输出/交付根、run/生成模式或本次选定后端。字段存在不等于计划 §5.1 要求的实际请求已承载这些事实。

修复：在上层持有已求值输入时无损构造一次合同，保未知节点、sidecar 声明和来源；明确投影旧生成器真正使用的路径/标题规则与覆盖序，并填调用上下文。
不要求 core 执行语言，不接受把 discarded 声明记成缺省终值；以真实产物与原读取臂对照，增加 node/prop 组合、双 front 布局、仅中文标题、gen/run/delivery 控制。

### F809-R2-04 / P2：V-05 工具发现把可执行的 Vue 验收跳过并算作绿，原效果矩阵仍缺项

类型：R1-F01 部分闭合及验收假绿；AC-04，T-06，V-05。

`plan809_management_core.rs:431,470-475` 的 which 只拼 `{tool}.exe`。本机 node.exe 与 npm/pnpm 的 ps1/CMD 入口实际存在，但没有 npm.exe；定向运行打印 `V05-PENDING: npm/node 不可用` 后 return，libtest 最终仍报告 7 passed。
不能将该 7/7 写成真实 npm success/failure 已执行。

本轮绕开错误探测，通过原生产 VueBuilder 真跑提交的同字节 fixture：产物 marker=true；失败脚本返回 `pnpm exited with code Some(3)`。因此是发现器/验收接线错误，不是环境根本缺工具；这两个实际结果也明确关闭“VueBuilder 是否能执行”旧例。
还需 work 补齐原 V-05 的受控 Ninja 成功产物（真实缺编译器时保持pending，不能判完成）、不可写输出等原约定失败阶段/exit收据；本轮不把它们新增为隐式要求，也不声称全部执行过。

修复：按生产 runner 的实际命令解析支持 CMD/PATHEXT/包管理器，并使“必需验收未执行”的汇总无法声称全闭；保留各环境缺项的真实结果。用正确探测永久复跑成功与失败控制。

### F809-R2-05 / P2：786 接线围栏实测红，新增代码还有未处理告警

类型：迁移验证适配缺口 / health；AC-03/05，T-03/06/08。

本轮命令 `cargo test -p auto-man --lib -- plan809_v02 generation_request_adapter project_display_reader --test-threads=1`：**11 passed，1 failed**，exit101。
失败为 `project_display_reader_tests::display_reader_v05_unified_entry_and_unselected_fields_intact`，`:808` 仍要求源码含旧拼法 `project_display_reader::read(&pac_content)`；新 VueReads 已改为借用参数调用。
这是需要随迁移修订的源接线断言，**不据此断言运行期64/65护栏已坏**；本轮边界/进程隔离/动态错误等相关运行测试均通过。
应将围栏改为验证实际新合同/显式legacy读源及原护栏，不能仅删测或插入注释骗字符串搜索。

新增/本计划引入的告警至少包括：Ninja HashSet unused；adapter RequestOrigin unused；Ark req unused；Cargo base_name unused；新 VueReads/from_workspace_reads 的 root_dir/pac_path/pac_content unused；core 测试 sel_from dead_code 和 same unused_mut；两个新增consumer也有未门控的测试helper/import告警。
这些路径在当前 Plan diff 中，不以完整语言库的411条既有告警作为本计划归因。
全部38个实际改动 Rust 文件 scoped rustfmt --check 通过；无需为格式重写未改内容。

修复：清本计划新告警并修订接线围栏，保留验收区分力；重放本轮12项以及涉及变化的定向测试。

### F809-R2-06 / P2：SD/发行收口与“全闭合”记录仍不匹配实际代码和证据

类型：R1-F04 收口残留；AC-04/05，T-07/08，SD-01..04。

777 strict 已独立复验 pass，violations=0、grandfathered=2；基线新增5条边与实际合法man域依赖相符，本轮不把重采样本身列问题。
两consumer locked离线各3项通过，normal/build/dev闭包无语言/Man/UI/生成栈。

候选 generation-request spec 却宣称“无字段丢失”“gen/run/build/open全量请求/零遗留”，与 R2-02/03 反例冲突；边界把本来约定的声明解析延期R04，也不能默认为已获批准。
T05..07 当前行同时写R1闭合又留旧pending/旧保留算法说明；handoff still以delivery 9/9 作为792复验，没有绑定本受审版的正式fresh/重定位locked/失败控制收据。
`docs/specs/repository-boundaries/generated-ui-distribution.md` 本来就区分内容9项与真实driver；不能互相替代。
本轮未执行正式792重交付、fresh/重定位locked全集，亦未把这些记为pass；新core不是运行期auto-lang闭包必需边，不要求无理由把它塞进792 bundle。

修复：按修好的实际合同更新SD，保原before/after、来源与原AC；使当前任务/交接只有一个清楚状态，历史另保留。提交原合同所需的实际fresh/locked与受影响792 source/resource检查收据，或可验证的原证据复用理由与全部输入hash；最终master并集复验仍另属merge。

## 非阻断兼容性注记 N809-R2-01

独立编译 `review-r2-api-probe.rs`：仅 `use auto_man::Target` 的旧 t.exists()/t.scan() 调用报E0599（exit1），显式导入TargetEffects后编译过（exit0）。
T01 contract §8.2/8.3 已明确扩展trait只保glob调用语法并记非glob风险，当前无已知仓外消费者；**本轮将它作为已声明边界，不单独据此阻断或要求恢复所有旧import写法**。
但最终SD/交接应清楚披露，不能把全字段/glob probe表述为任意导入方式下源兼容。探针和两份原始编译输出已保存。

## 验证结果与复跑

| 验证 | 实际结果 | 原始证据 |
|---|---|---|
| core/model package lib定向 | 45/45 + 38/38（有上列新增告警） | review-r2-core-model.log.txt |
| plan809_management_core串行 | 7/7，但Vue真工具case提前return | review-r2-management.log.txt |
| 生成请求/786 guard定向 | 11/12，1个旧source围栏红 | review-r2-generation.log.txt |
| 两独立consumer locked/offline | 各3/3；闭包normal/build/dev干净 | review-r2-*-consumer.log.txt、review-r2-*-closure.log.txt |
| 777 strict/no-cargo-check | pass，0 violations，2 grandfathered | review-r2-777.log.txt |
| scoped rustfmt | 38个实际改动Rust文件全部通过 | review-r2-format.log.txt |
| 生产反例/正控 | Ninja实例反例；Vue front/title反例；主声明二读；Ark丢请求；Vue真工具正/负控 | review-r2-probe.rs、review-r2-probe.log.txt |
| API import边界 | explicit-only编译红，加trait绿（已声明风险） | review-r2-api-probe.rs、review-r2-api-*.log.txt |

反例复跑：先在809原树构建当前auto-man生产库，再从该树执行
`powershell -File D:/autostack/auto-lang/docs/plans/attachments/809/review-r2-replay.ps1`。
脚本打印实际rlib哈希，链接已安装Windows native库；初次缺native链接目录的探针构建红保存在build日志，补目录后生产探针实际执行成功，未据第一次链接错误判实现失败。
探针打印实际值用于人工审查，**进程exit0不代表所有不变量通过**，应比较上述期待/原始输出。
测试输入/反例源码、命令/输出与源/锁哈希均可在工作树清理后追溯；临时.ninja文件只作为辅助，不作为唯一证据。

## AC 与 SD 结论

| AC | 本轮结果 | 依据 |
|---|---|---|
| AC-01 | partial/fail | 核心及下载决策确有生产消费；R2-01规划身份回归 |
| AC-02 | fail | R2-02/03实际入口和字段/产物反例 |
| AC-03 | partial | 核心图45例、786运行边界过；不能覆盖新目录/标题/身份回归与source围栏红 |
| AC-04 | partial/fail | Cargo/Vue部分真工具通过；原V05完整矩阵与792正式收据pending |
| AC-05 | fail | 尚有开放finding、新warning和SD冲突 |
| AC-06 | pending | merge-owned，未进行合入/归档/接口发布/清树 |

SD-01 partial（规则入口已存在，执行身份不保真）；SD-02/03 fail（投影/无丢失/读源陈述不实）；SD-04 partial（委托描述过宽）。
四个metadata目标路径和GOAL-003保留，但不授信为通过。canonical/live ledger未修改。

## 路由

outcome=needs_fix；status=executing；重开T03/05/06/07/08，保留T01/02/04原实施记录，current_step=3/11；T09待终审、T10/11待merge。
原809树work修复后提交，保持一树；不在review修实现。
已有两次final非通过（R1 readiness + R2），下轮ready前须给“漏检原因→完整不变量→受影响路径→设计/oracle调整→实跑结果”诊断表，逐项解释为什么旧绿数/入口grep/纯rename测试没覆盖这些反例。

pending_closeout_gates：最终code/acceptance/Spec pass后由merge跑冻结裸cargo t+必要包档与实际触面档；777/792最终并集、规范/ledger/归档/已合入接口SHA交接、guard clean和清理仍pending。
本轮无整档t/tv/tt/tb/tu/taa/tf、无实现修改、无合入/归档；不宣称已交付。
