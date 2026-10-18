# PLAN-809 R5：needs_fix（R4修复闭合核查）

2026-10-19，review_scope=final。按用户“已修复，再次检查”复核新提交；交接仍明确承认两项work-owned验收pending，本轮执行受影响定向复验，不重跑完整终验或整档回归。**剩余问题尚未全部闭合**。

## 基线与范围

- plan_revision=1；commit=`0fba0b604bd81076b8404ffbb9161432acfc0fe3`（修复 b0f8f857e + SD校准）；prior=`6a84a7e28f5579b7c188b54b5ca86a9ae295f3a9`；base=`2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 同一原树 `D:/autostack/.wt/lang-809/auto-lang`，branch=`codex/plan-809-dev`；兄弟auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场/复验后实现与依赖clean，HEAD未漂移，diff --check过。
- reviewer_identity：延续此前独立复审的Codex上下文，未实施本轮修复、未另开agent，不虚构模型/会话ID。以实际代码和生产反例重建结论。
- [冻结基线/哈希](review-r5-baseline.json.txt)、[送审Plan快照](review-r5-plan-input.md)、[SD候选冻结](review-r5-spec-delta.md)。主检出原有其他工作保留，未修改实现、canonical Specs或ledger。

## 关闭情况

| finding | 本轮结果 | 独立证据及边界 |
|---|---|---|
| F809-R4-01 / R3-02 core编译及Block | CLOSED | 测试作用域导入AutoStr恢复，str_list实际补Block；core46/46、model38/38；原生产Block探针现在返回一个m模块 |
| F809-R4-02 对象/嵌套sidecar无损消费 | **OPEN，同一原反例仍失败** | 未改动R4对象输入重新链接当前库：old_extra=true/new_extra=false。新增Value::Node属性对照：head=rust_sidecar保留模块/依赖；head=sidecar_value两者均漏，core原规则均识别 |
| F809-R4-03 C输入非法 | CLOSED（仅输入缺陷） | 当前int main输入用已安装cl.exe独立编译exit0；不等同真实Ninja成功产物验收 |
| F809-R3-03 本工程gen覆盖旁路残留 | CLOSED（该具体入口） | automan.rs:1517后改经gen_request_as，沿用render_override投影并覆盖output_root；原custom/front及上下文定向保持通过。未将源码接线检查虚称为全后端完整验收 |
| F809-R3-04 新增from_request_pub告警 | CLOSED | helper删除、测试直用from_request；普通auto-man库日志不再出现该告警；未把其他既有warning归809或宣称全仓零警告 |
| F809-R2-04 三Builder实际效果矩阵 | PARTIAL / OPEN | 真Cargo/Vue成功失败与不可写输出负控通过；管理8/8包含显式Ninja成功pending，而不是8项全部验收通过 |
| F809-R2-06 SD / 809受影响独立交付证据 | PARTIAL / OPEN | SD-02已修改边界，但“三形态全消费”仍与实际反例冲突；Plan最新交接承认独立交付回归收据仍待补，未定位新增正式证据或完整复用论证 |

旧R2-01 Ninja实例三态、Ark run、Vue目录/主声明/仅中文标题/依赖读源、sidecar节点来源反转、Block/custom-front等原输入和正控均在本轮生产重放中检查；已关闭项不重新判红。未知raw_args、generic display_title与Vue自身语义差异仍不单独列缺陷。

## F809-R4-02 残留根因及修复要求（P2）

AC-02/03，T-06/08。`crates/auto-man/src/rust_ui.rs:3294..3324`把raw Obj放到一个临时root的属性里，随后又把这个临时root挂成最终root的子节点。最终传给core的是另一层root，既无rust_sidecar属性，也无名为rust_sidecar的直接子节点；core的属性兜底自然读不到。Value::Node则直接改成子节点，只有内部名称恰好等于rust_sidecar才会碰巧被选中，改变了原先按属性键识别的语义。

原输入未经改写：[review-r4-sidecar-obj.rs](review-r4-sidecar-obj.rs)。真实pac声明 `rust_sidecar: { modules: ["extra:extra.rs"] }`；请求raw_sidecar=true/raw_nodes=0。当前legacy与request生成均Ok，旧产物有extra.rs，新产物没有。**这是同一原例未闭，不是扩张验收范围。**

补充的[嵌套属性对照](review-r5-shapes.rs)使用已求值配置root中`rust_sidecar: Value::Node`的原支持域，core按属性键解析：两个不同内部head均得到1模块+1依赖。真实Pac→请求→Rust生成，head=rust_sidecar实际两者都在；head=sidecar_value实际两者都不在。它验证的是现有动态值合同，不声称新增某种静态pac语法支持。

修复应把请求求值声明还原到core原本读取的同一层root，保持“具名子节点优先、属性Obj/Node兜底”的原有规则；不是按形态猜一个新head或新增平行解析器。永久用例须断言**真实生成产物**中的模块文件及依赖条目，覆盖节点/对象/属性Node和节点优先对照，同时保持磁盘变化不覆盖请求。当前修复的新增分支没有相应对象产物断言，原节点单测通过无法证明该分支。

## 本轮实际执行

| 命令/检查（均在原809树） | 结果 | 持久证据 |
|---|---|---|
| cargo test -p auto-man-core -p auto-project-model --lib | exit0，46+38 | review-r5-core-model.log.txt |
| cargo test -p auto-man --test plan809_management_core -- --test-threads=1 --nocapture | exit0，8；Ninja成功显式pending | review-r5-management.log.txt |
| cargo test -p auto-man --lib -- plan809_v02 plan809_r2_parity generation_request_adapter project_display_reader --test-threads=1 | exit0，15 | review-r5-generation.log.txt |
| 五个变更Rust文件 rustfmt --check --config skip_children=true | exit0 | review-r5-format.log.txt |
| 原R2/R3探针、未改动R4对象输入、补充Node属性控制，链接本轮生产rlib | 执行完成；对象及异名Node属性仍违背不变量 | review-r5-probe.log.txt、review-r5-replay.ps1 |
| 提取当前C测试原文并以cl.exe编译 | exit0，输入缺陷关闭 | review-r5-c-fixture.py、review-r5-c-fixture.log.txt |

探针exit0只代表观测完成，不能替代其打印结果的断言。首次探针调用在Cargo仍构建时缺auto_lang rlib，未产生行为结论；原日志保留为review-r5-probe-before-build.log.txt，生产库就绪后重新完整执行成功。主重放日志打印实际rlib哈希。

没有执行whole-tier t/tv/tt/tb/tu/taa/tf、777、正式792 fresh/重定位locked及独立consumer全集。未改依赖/manifest/锁/测试档配置；本轮不将旧证据重记为当前实跑，也不凭旧792交付记录关闭新生成链的验收。**792本身已归档8/8、原driver25/25，不重开792。**

## AC / SD / 状态

| AC | 当前结论、任务与依据 |
|---|---|
| AC-01 | 本轮受影响core/兼容规划部分pass；T01/03/04/05定向过，整个生产闭环仍受AC04矩阵限制 |
| AC-02 | fail；T02/06/08，请求Obj/Node属性消费仍丢失 |
| AC-03 | partial；T03/04/07，Block/786/图原例保持，属性声明支持域仍有缺口 |
| AC-04 | partial；T05/06/07，Ninja真实成功及受影响独立交付work证据pending |
| AC-05 | fail；T08/09，SD全消费陈述/闭合证据仍不能授信 |
| AC-06 | merge-owned pending；T10/11 |

SD-02确已改动，不能沿用上轮“文本未改”的判断；但现在写“三形态经请求单源消费”，行为仍未达到。SD-01/03/04候选保持原边界，本轮不授予整套规范增量pass。四个Spec路径及GOAL-003元数据保留；详冻结正文/输入哈希。修正事实陈述不等于重规划或批准缩面。

outcome=needs_fix；status=executing；**T03核心编译/Block及相关门面验证本轮关闭，改完成**；保留T01/02/04/05。T06/07/08仍开放，T09未完成、T10/11待merge；current_step=5/11。历史work“R4-01..03全闭”由本记录明确纠正R4-02；不覆盖旧记录。

next=同树work完成R4-02原反例及属性形态对照，补实际生产产物oracle、根因/闭合表；完成R2-04真实Ninja成功验收、R2-06准确SD及可定位绑定的受影响独立交付证据，再提交复审。已知work-owned项未完成时保持executing；本轮不批准延期或以debt登记豁免。

pending_closeout_gates：最终代码/验收/SD pass后，merge在冻结提交统一执行裸cargo t、必要包及按最终diff需要的触面档，最终777/792并集、规范/ledger、归档、已合入接口SHA交接、wt-guard clean后清理；这些门禁仍pending。本轮未合入或归档。
