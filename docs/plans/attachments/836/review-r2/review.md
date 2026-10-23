# PLAN-836 final review R2 — pass

2026-10-23；plan_revision 1；review_scope final；Codex，沿用R1独立审查上下文，未参与本轮实施修复，未委派其他agent或模型。

**结论：pass，F-836-R1-01/P2 CLOSED，无新增finding。** T-07/T-08修复闭合，T-09最终验收通过；Plan reviewed 9/10。T-10和完整merge-owned门仍pending，不能据此宣称已合入/交付。

受审唯一树 `D:/autostack/.wt/lang-836/auto-lang`，分支 `plan-836-dev`；reviewed_commit `5f3e218ba5d605cfb3e007d1c9bade5a5fe399a1`；总体base和当前autographics-dev `2a7b10250245332f46b7867dc140da1a4f9d0969`；修复diff基于R1 `87fb0b051d8ab9b17fba0fcfe2580c774a79d058`。SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`。入口及结束实施树clean，审查未修改实施代码。

完整身份、主Plan输入和Spec/依赖/二进制指纹见 [baseline](baseline.json)、[plan-input](plan-input.md)。本次8文件修复diff限native bin/fixture、永久CLI测试和文档/身份。主检出 canonical Specs 与R1逐字节相同；authoring库/Auto样本、AGP/SDK、资源、锁/manifest也无变化。因此R1的22项定向测试和4项独立holdout只在原不变库覆盖范围复用；新binary/fixture使用本轮fresh证据，不外推旧pass。

## Finding关闭与修复回归

| 范围 | 本轮独立验证 / 实际结果 | 结论 |
|---|---|---|
| 原3个frame1反例 | 永久CLI回归逐一启动普通/wrong/drop `--interaction-check --frames 1`，断言非零、passed=false、报告error含incomplete；原始7子进程报告保留为本目录probe-exit-* | 原例关闭 |
| 全不变量 / 所有正常run_app返回 | 源码核实 `finalize_checks` 在run_app返回后无条件执行且先于报告；检查自己的Probe.done/error，保留原错误，未完成状态不受旧passed标志影响；旧window-probe还须phase>=9。clean early return/partial/stale success/complete/error/observation由bin单测覆盖 | 共同出口关闭，非frames1特判 |
| 报告一致性与合法观察 | 新CLI还测legacy frame1拒绝、observational和普通有界窗口exit0/error=null/passed=false、Host创建前missing Open原错误非零且可读报告 | 合法控制保持，失败传播与report一致 |
| 实际部分进度 | `--interaction-check --frames 52`：exit1、passed=false、phase3、frames52、records1，error `native interaction check incomplete at phase 3 after 52 frames` | 不把已跑部分当完整通过 |
| 同时传两种self-check标志 | `--interaction-check --window-probe --frames 1`：exit1、passed=false、phase0、records0、明确incomplete原因 | 新兄弟状态控制通过；不承诺两模式可同时成功 |
| 完整实际原生合法控制 | `--interaction-check`：exit0、passed=true、phase27、frames456、records6、error=null；原临时Auto画面/accepted隔离、最终端点、Undo/Redo/Esc、原GLB XYZ、最终资源失败与恢复、Save、resize/late-up及DPI断言全部到达 | 原完整路径本轮fresh关闭 |
| 完整实际原生负控 | wrong-target exit1/phase1/原断言；drop-release exit1/phase4/原timeout，后者records1 | 原同门反控仍敏感，未削弱oracle |
| 保存重开 | 本轮positive保存文件新进程inspect exit0；hash `9db84cb370bdb59104de742161376420ff93b4bcee2778840ce61e35ee64a534`匹配保存记录，history=[0,0,0]、selection=null、dirty=false | 后续真实消费通过 |

原生fresh原始结果见 [native-summary](viewport-review-r2-native-summary.json)、[positive](viewport-review-r2-positive.json)、[wrong](viewport-review-r2-wrong.json)、[drop](viewport-review-r2-drop.json)、[partial](viewport-review-r2-partial.json)、[combined](viewport-review-r2-combined-truncated.json)、[reopen](viewport-review-r2-reopen-result.json)。最终合成截图已目视核查，[截图](viewport-review-r2-positive.png)显示末次resize后circle位于x=-33，属性同步；截图不是单独的业务正确性oracle。

## 夹具改动与证据边界

本轮额外改了native fixture的起始条件，故不能只复测退出码。已完整读新旧diff：phase0在任何手势开始前核自身焦点、移动物理pointer到预期对象、等待一次既有12-frame间隔，然后仍通过原OS消息→winit→Host→Auto→GPU路径；没有直接dispatch手势，也未忽略/重试活动期间取消。未取得焦点会由既有phase timeout失败，不会无限等待。生产leave/focus/resize分支仅增加观测记录，原cancel行为保留。

R1普通native正门的未闭合证明本次完整positive及两负控已补齐。旧失败继续保留历史；work r2记录实际CursorLeft取消解释了失败机制，精确OS来源仍是推断。本次不把历史失败改成通过，也不把准备pointer误写为关闭取消功能。

## AC映射与健康

| AC | 任务 / 证据 | R2结论 |
|---|---|---|
| AC-01 | T01/08；来源227 hashes/SDK680/closure479/local17/edges29 fresh通过；库/AGP/资产/锁/root写面核对；打包work证据不冒称本轮重跑 | pass |
| AC-02 | T03/06/07；R1几何/路由/真实鼠标证据加fresh全原生选择/最终端点/Undo/resize/DPI | pass |
| AC-03 | T03/05/07；R1实际camera/OBB holdout和Host三轴；fresh原GLB每轴/临时画面/端点/精度与同名隔离 | pass |
| AC-04 | T04/05；R1不变库候选/最后端点/失败/预算/no-op/owner证据；fresh全原生临时/accepted与最终事务 | pass |
| AC-05 | T04/06/07；F01原例、共同完成性、部分/组合/原错误/观察控；fresh真实Esc/resize/late-up、原wrong/drop | pass |
| AC-06 | T04/06/07/08；R1不变Storage/资源/弱owner/pin/历史证据；fresh Save/新进程reopen | pass |
| AC-07 | T08；来源身份、README/research/closure、SD04澄清与本轮精确CLI结果 | pass |
| AC-08 | T09；本次独立受影响final验收、原证据身份复核及Spec冻结 | pass |
| AC-09 | T10；实际合入/完整门/知识/归档/guard清树 | pending merge |

fresh命令与结果：

- `cargo test --locked --bin autographics-authoring --test authoring_probe_exit -- --nocapture --test-threads=1`：2单测+1集成测试通过，集成含7子进程；0失败/忽略，见 [completion log](viewport-review-r2-completion.log)。动态运行原3反例的对应断言，非源码字串测试。
- `python tools/verify_source.py`：227来源/680 SDK/479闭包/17本地包/29边通过，见 [source](viewport-review-r2-source.log)。本轮未重复改源码/删资产做source mutation；verifier未变，既有source反控按相同冻结输入范围复用。
- `cargo check --locked --all-targets` / `cargo fmt -- --check` / `git diff --check 87fb0b051..HEAD`：通过；SDK146既有warnings保留，无新增driver/AGP warnings。[check log](viewport-review-r2-check.log)。未运行完整driver/AGP或root整档。
- 全修复diff扫描未见新debug打印、未披露延期、Rust预览旁路、源码入master或支持范围收缩。新增trace仅在显式interaction-probe模式；startup无Host报告分支保留原错误，正常完整门断言未删减。

## Spec delta冻结与合入待办

冻结 [spec-delta-frozen](spec-delta-frozen.md)；原始CRLF字节SHA256 `f61a6de1e419ab4357f985f03ac6e0eef168029e797d9c2bc1da4905fc7647fd`，LF归一SHA256 `259137cc772613eca30db99db5b458eda5616215f765992bba5f8b0d2acb8ecf`。后者与work receipt相符，两者是同一文本换行口径，不是语义漂移。SD01..03未变；SD04明确原有no-skip完成性、先定结论后report、观察模式和夹具准备边界，符合原合同，无语义放宽，plan_revision仍1。原5个modify目标、1个add目标与GOAL-007/008元数据有效；本轮未写canonical或ledger。

**pending_closeout_gates（merge-owned）**：在同树新冻结提交上统一跑完整locked driver/AGP test+build、原16 case、双游戏及835原生正负门、新interaction完整正负/重开、两道counter与content合成门、来源/资产/独立source consumer；随后才实际落autographics-dev并沉淀canonical/ledger、archive、wt-guard清理及批量到期检查。重用本轮定向证据须核代码、依赖、配置与输入不变；修复改变代码须受影响复审并重冻门禁。root整档不因本实验自动扩大触面，tf只在主检出到期批量运行。

next：merge（T-10），先完整冻结门，再实际合入；当前仅reviewed 9/10，未合入、未归档、未清树。
