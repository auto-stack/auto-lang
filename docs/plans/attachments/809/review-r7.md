# PLAN-809 R7：needs_fix（告警关闭，仍缺两项实际验收）

2026-10-19；review_scope=final，有界就绪/收据闭合核查。**本轮F809-R6-01已关闭；F809-R2-04、F809-R2-06仍未全部闭合。新增9项delivery单测收据不是正式独立交付构建收据。** 已知work-owned验收未完成，本轮不重跑一轮完整终验，也不以绿数授予reviewed。

## 冻结基线

- plan_revision=1；reviewed_commit=`dca409c94000011016be0fd8c5b5ae9a1baa7ff1`；prior=`23e94eb74666479789345c5c1b81f8ec6545824b`；base=`2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 原树`D:/autostack/.wt/lang-809/auto-lang`，branch=`codex/plan-809-dev`；auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场/记录前实现与依赖clean、HEAD无漂移、diff --check通过。
- reviewer_identity：此前独立复审的Codex上下文，未参与本轮实现、无新agent、不虚构模型/会话ID。未修改实现，未合入、归档或发布Specs/ledger。
- [基线/哈希](review-r7-baseline.json.txt)、[送审Plan](review-r7-plan-input.md)、[冻结SD](review-r7-spec-delta.md)、[行为等价/复用输入核对](review-r7-reuse-check.json.txt)。

## 剩余finding逐项判定

| finding | R7结论 | 依据 |
|---|---|---|
| F809-R6-01 unused_mut | **CLOSED** | 本轮唯一代码变更为rust_ui.rs:3289移除mut；独立逐字比较确认其余文件内容与R6相同，格式检查通过。关闭该具体已知告警，不虚称本轮重跑全库编译或全仓零警告 |
| F809-R2-04 真实Ninja成功产物 | **OPEN** | 最新work行仍承认唯一余项为Ninja环境pending；没有新增真实成功/产物可消费收据，管理用例及工具发现代码未变；R6实际执行日志明确成功分支跳过 |
| F809-R2-06 独立交付fresh/重定位locked及正式入口回归 | **PARTIAL / OPEN** | 新增t06收据记录9/9内容单测；不含正式driver的fresh build、重定位locked build和对应失败控制，也不证明请求入口完整交付 |
| R4-02/R5 sidecar及其他已闭代码问题 | CLOSED保持 | 本轮仅删除不必要mut，运行行为未改；代码/依赖/配置/输入核对后限定复用R6原反例、优先级和失败传播结果，不改记新SHA实跑 |

## 为什么新增9/9收据不能关闭R2-06

新增[收据](t06-792-delivery-regression-receipt.txt)为9个`rust_ui::tests::delivery_*`用例的测试结果摘要，绑定父提交23e94eb74，报告74.40s。该父提交到当前提交仅移除unused mut，复用这份内容测试在行为身份上合理；问题是**验收范围仍不足**，不是单纯绑定短SHA或缺一轮重复运行。

核对实际测试：`crates/auto-man/src/rust_ui.rs:8099`的`delivery_bundle_end_to_end`调用`generate_rust_ui_delivery`，验证文件、manifest、资源hash、投影、再交付和外来目录保护，未运行Cargo；:5353的该入口构造`PacSource::legacy()`，不是:5359的`generate_rust_ui_delivery_with_request`。所以“delivery全族经改造后PacSource签名真跑”不能推出809生产请求入口及独立消费编译已验证。

权威`docs/specs/repository-boundaries/generated-ui-distribution.md`本来就分开两个门：

- 生成器内容门：delivery单测（不跑Cargo）；
- 真实交付门：`python test/fixtures/modularity/generated-ui-consumer/driver.py`，经正式CLI生成，V01全新解析+真实build，V02不带target搬到B后`cargo build --locked`，V03负控和V04开发模式保持。

现有driver.py:210附近为V01真实build，:240附近为V02锁定重建，:605写正式driver-receipt.json；默认`D:/autostack/.wt/lang-809/scratch-consumer`仍不存在。附件新增的832字节摘要只有9项单测结果，未包含这些V项、独立锁/构建身份或完整原证据复用论证。

这也是R2报告已明确的原要求：“不能以delivery9单测代替正式fresh/重定位locked”，不是R7追加标准。当前Plan末行“唯一余项Ninja”应改为“两项实际验收未闭”；handoff仍有旧R2 pending表，也需统一当前状态。

实际缺口仍归AC-04/05、T-07/08，未批准延期。792本身原交付已归档完成，本轮不重开792；需要的是809受影响路径的实证，或能够证明全部相关代码/依赖/配置/输入未变的完整复用依据。R6的sidecar请求链有实质变化，原792交付结果及9项内容单测不足以自动覆盖。

## 本轮检查与证据复用

- **新执行**：读取提交差异、收据及9个测试对应范围；静态追踪legacy/request交付入口、正式driver V项与权威规范；唯一Rust变更文件rustfmt通过；原树/依赖/提交/diff健康核对。
- **精确复用核对**：将R6完整rust_ui.rs仅把`let mut sidecar_nodes`替换为`let sidecar_nodes`，与当前文件逐字相等。core、请求adapter、Cargo.toml/Cargo.lock、Cargo和测试配置哈希与R6相同，见reuse-check。
- **限定复用**：R6生成16/16、管理8/8（其中Ninja成功pending）及sidecar对象/嵌套/优先级/缺源生产探针；更早core46/model38等原有未变范围。没有把它们记为R7重跑。
- **新收据**：9/9是实施方新增摘要，本轮核验测试身份和范围，未独立重新执行这9项，不将摘要当作正式driver输出。
- **未执行**：本轮无Cargo测试/整档/正式独立交付driver/777/真实Ninja成功；已知两项work-owned验收缺口，未花一轮完整最终验收重复确认它们。

初次读取Git对象受到沙箱目录权限限制，未产出比较结论；按只读权限重新读取后逐字等价及哈希核对成功，不将权限失败当代码缺陷。

## AC / SD / 状态与下一步

AC01..03在已验规则/请求支持矩阵内保持pass（按上述限定复用）；AC04 partial（两实际验收仍缺）；AC05 fail（收据范围/当前闭合声明不匹配）；AC06 merge-owned pending。

SD-02三形态行为冲突保持关闭，SD-01/03/04候选规则边界及四条Spec路径/GOAL-003元数据保留，见冻结正文/哈希；没有新增算法或规范冲突，也未发布canonical知识。不能把SD一致等同未执行的交付验收通过。

outcome=needs_fix；status=executing；T01..05保留，T06真实Ninja未验、T07正式独立交付未验、T08收据/交接声明待校准，T09未完成、T10/11待merge；current_step=5/11。F6-01已关闭，当前无新代码finding。

next=同树work补两份实际绑定证据：①选中工具链的真实Ninja成功及可消费产物；②正式独立交付driver的fresh/重定位locked/受影响失败控制及输入、依赖、锁身份。明确9项单测只是内容门，附准确当前handoff和闭合表后再审。若收据另存，可提供实际位置和绑定信息；不要仅把pending移到merge或重命名单测摘要。

pending_closeout_gates：最终代码/验收/Spec pass后由merge在冻结提交运行裸cargo t、必要包及按最终diff需要的触面档，最终777/792并集、规范/ledger、归档、已合入接口SHA交接、wt-guard clean后清理；这些收尾门仍pending，不等同上述work-owned缺口。
