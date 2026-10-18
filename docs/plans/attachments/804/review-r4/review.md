# PLAN-804 final review R4 — pass

2026-10-18，Asia/Shanghai。review_scope=final。reviewer为当前Codex `/root`，独立于实施上下文并沿用R1–R3历史；模型/会话ID未暴露，不虚构额外reviewer。未创建agent/新树，未修改实施或合入。

## Frozen baseline

- 工作树 `D:/autostack/.wt/lang-804/auto-lang`，branch `plan-804-dev`，受审提交 `1ca36a674e9199985f8bc804d1ac0f61d1bcaa97`，工作树干净。
- 总diff base=`b7ecea5e36ddf41d9effc49f0aba53cca47b99ad`；本轮修复base=`21d5f9c018cf64b1c6e7439075819871db7df17a`，两文件+116/-70。auto-down兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- Plan revision1；实际根锁、fixture锁、Cargo/nextest配置及crate manifest与R3/R2保持一致。Spec当前版本/hash、原Plan指纹见 [baseline](baseline.json.txt)。主检出并行新事实不合入804树，本结论不覆盖后续merge冲突修复。
- 宿主与测试均重新构建，命令见 [host](build-host.log.txt)、[test compile](build-tests.log.txt)。本轮test SHA=`5a4c2c36aaf6e3577c9afa5227ef3f1255cb061b8b92d7c893101c3b121507d4`，host SHA=`cf005bb91fc2cb620746b73b2bf084cf71780636fc54b764a9a7f176d6d477a7`；本轮结果绑定这两个实际二进制，未把work的另一host hash当成本轮运行值。

## Findings closure

| Finding | 当前不变量证据 | 结论 |
|---|---|---|
| R3-01 / R2-01 / R1-02 真实宿主链 | 当前提交、无失败钩子的生产run_dynamic_desktop全链实际通过。started=true、finish=copy；生产CALL_HANDLER明确送达InitiatorApp，其他App marker缺席；子进程正常退出 | CLOSED |
| R3-02 失败遗留宿主 | spawn立即进入ChildGuard；Drop统一kill+wait。重放R3原始外部挂起启动反例，exit101/窗口25s诊断后无宿主；NO_DRAG交付失败反控实际started=true但无finish，明确finish缺失诊断后仍无宿主。外层回收数均0 | CLOSED |
| R2-02 输入残留 | 正常与交付失败路径，退出立即/+250ms五键均false；五会话驱动join/输入守卫保持R3已独立核过实现 | CLOSED |
| R2-03 / R1-04 新增卫生/线程说明 | 手工接线测试已删除，当前两改动文件rustfmt均0；当前host/lib-test编译没有本轮新增warning，旧EnumWindows等预存诊断不归本轮新增 | CLOSED |
| R1-01 父窗及双取消 | dialog实现、旧门面与V04测试块相对独立R2完全不变，实际锁/配置不变，明确复用R2真dialog4/4与ui-dialog-only1/1双腿 | 保持CLOSED |
| R1-03 792 | 交付生成器/driver/manifest/锁和实际生产编译路径未变，复用R2独立fresh/重定位locked/保护控制25/25；本轮不冒称又发行了相同hash整包，合流后须更新实际producer/清单 | 保持CLOSED |

两轮失败后的诊断/覆盖调整已实证落到当前实现：手工接线被真实生产入口替代；发起方组件置顶和中心目标点使点击可达；Child启动即归属，不再只有成功marker后的分支才清理。当前未发现新的合同内缺口、未批准任何延期/workaround。

## Independent replay（只做定向验收）

复审 [os-probe.py](os-probe.py) 监视本次测试实际创建的宿主，校验PID创建时间和精确exe路径；在记录测试退出后的原状态之后，才允许外层兜底清理。结果要求外层清理数为0，避免“复审帮忙杀进程”掩盖测试缺陷。失败控制要求exit101和指定诊断，不以任意非零退出（编译失败/预条件失败）当成反控成功。

| Case / invocation | 实际结果 / 证据 |
|---|---|
| `python os-probe.py normal-clean v05_real_host_dnd_finished_delivery`，两失败hook均移除 | test exit0，测试5.15s（探针5.557s）；started=true、finish=copy，干扰方marker缺席；无活宿主、外层回收0，立即/+250ms输入全清。[JSON](normal-clean.json.txt)、[原始日志](normal-clean.log.txt) |
| `python os-probe.py external-startup-stall v05_real_host_dnd_finished_delivery --suspend-host --expect-failure` | 与R3同一外部初始化停滞，不依赖新hook；test exit101、明确“真宿主窗25s未出现”；无活宿主，外层回收0，输入未变化。[JSON](external-startup-stall.json.txt)、[log](external-startup-stall.log.txt) |
| `PLAN804_HOST_NO_DRAG=1 python os-probe.py delivery-failure v05_real_host_dnd_finished_delivery --expect-failure` | test exit101，探针20.885s；started=true证明hook实际生效/handler执行，finish不存在，指定finish缺失诊断；无活宿主、外层回收0，立即/+250ms输入全清。[JSON](delivery-failure.json.txt)、[log](delivery-failure.log.txt) |
| 当前example与lib-test编译 | exit0：21.73s/0.59s。未运行整档回归 |
| 两修复文件rustfmt及git diff --check | exit0，见 [fmt](fmt.json.txt)、[diff](diff-check.log.txt) |

运行条件实录：开始时系统报告预存Return按下，第一次普通探针正确在预条件退出，没有运行测试。随后在已授权实机验收范围内仅发送一次Return key-up复位（[reset](input-reset.json.txt)），再次确认五键全清才启动当前仅鼠标驱动流程。三个场景均未残留本轮输入；启动停滞场景不发送任何输入。端口9247被占后生产MCP回退9248属正常行为，不影响本轮链路。

## AC traceability / evidence reuse

| AC | 任务/代码 | 决定 |
|---|---|---|
| AC01 | T02/03；三shim、双接口、外部no-ui consumer | pass：复用R2独立7/7及源码剪断证据；相关生产路径与fixture没有变化 |
| AC02 | T02–05；缺席/取消/字符串寿命/装配/V04 | pass：复用R2真实V04、no-ui、56定向回归中对应检查；实际输入/配置未变 |
| AC03 | T03–05；同步模态、拖出、事件和资源 | pass：历史五会话+当前真实宿主及两类失败反控共同覆盖；生产订阅/dispatch/发起者锚定已实际交付copy，既有焦点他窗helper控制维持 |
| AC04 | T03–06；去重/删边/剩余族/V06/SD | pass：单一OS实现与其余provider未变；本轮卫生通过，交付证据复用理由如下；共享合流/新整包身份仍需merge实态重核 |
| AC05 | T07 | pass：本轮独立验收/代码/Spec审查完成，绑定当前提交 |
| AC06 | T08 | pending：merge-owned实际回归与收尾，本轮未执行 |

复用不是依据“签名未变”：已逐路径比较R2→当前的lib.rs、vm/native/engine/host_capabilities、host_adapters、ui/dialog/dnd_host_provider/vm_bridge、外部fixture、auto-man交付生成器及driver，差异为空；V04测试前缀完全相同；锁/config/manifest一致。R2后变化只在cfg(test)模块及未被交付应用构建选中的example，生产应用的有效编译路径不变，故复用R2独立no-ui7、dialog4+1、定向56、792 fresh/locked/保护控制25，以及R3五会话正常退出反控。完整路径依据见 [reuse](evidence-reuse.json.txt)。源码整包digest仍随测试/示例文件变化，历史792 digest仅指历史包，**不能**直接贴成当前/最终合流包digest；merge须据实际producer生成清单并重核受集成影响consumer。

## Knowledge delta / next

SD01..04冻结于 [spec-delta](spec-delta.md)，SHA见配套文件；新/替换组件路径与GOAL-003/007已核，保持Plan元数据。规则覆盖当前行为：独立owned槽、降级与会话true/效果分离、父窗策略、ui-dialog-only路径、输入线程内联。未修改canonical Specs/ledger，未把验收example当成新生产API，未宣称其他未迁族已解耦。

outcome=final pass，Plan进入reviewed，T07完成，current_step=7/8；T08仍pending。next=auto-plan-merge。

pending_closeout_gates：合流后冻结裸cargo t+tv（无ui_gen实现改动不加tu、无aavm触面不加taa）；master/802等共享安装与槽入口取并集、777基线与受影响consumer实际重核；Spec/ledger/索引沉淀、归档与wt-guard清理。合流若改动受审生产路径，需受影响复审及在新冻结提交执行相应门禁。tf只按merge到期判定在主检出单实例批量运行。本轮pass不是已交付或已合入。
