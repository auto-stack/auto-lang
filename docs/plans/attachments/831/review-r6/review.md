# PLAN-831 T-10 R6：needs_fix（r3，仅分类修复回归）

2026-10-25；review_scope=final目标下的受影响修复复审。当前Codex会话，非新独立上下文，无新agent/checkout；未发最终pass。r3延期、V06e及其他已闭结论保留。

## 基线与已闭验证

主检出7b686ed25aef3631c0a50d1be361e3d950244067；lang权威树D:/autostack/.wt/lang-831/auto-lang / codex/plan-831-dev，reviewed=c79d343111b85314b1412570d94ae7f26a1605cf，repair base=538e3b8fefef4b651b138201234e7c6fa50bcbfa。UI未变425088520e402b1efae9479b9d0b5b3d44a56df4，down只读c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8。

两实现树tracked clean，lang唯一可再生untracked compile-probe/Cargo.lock不变；7材料及4原始终源收据hash/sources匹配，canonical spec_inputs无漂移，修复diff-check通过。仅改V06c driver分类，无产品代码/配置/依赖变化；既有其他有效证据复用，不重跑无关族或整档。

**R4与R5的两个真实工具反例现在均关闭**：原not found路径、完整Hvigor wrapper not found短语路径的已执行exit1都正确传播红，各自重放exit0。原五项构建控制亦5/5符合同（全完成/获准缺环境绿，原jet/ark编译错/required错红）。不能把本轮结果描述为这些旧例仍失败。

## F831-09-03 A · PARTIAL / OPEN：从缺席路径中误读退出码（本轮修复回归）

当前分类先在**整段**error中`re.search(r"exited (?:with )?(?:exit code: )?(\d+)", err_text)`，找到任一数字就按“已执行”处理，随后才检查缺席原因句头。对实际执行失败形态，首个退出码一般位于真实原因内；但对没有执行退出码的缺wrapper形态，首个匹配可来自路径。这仍未做到“路径字节不能翻转失败类别”。

**真实工具反例（合法环境延期被误红，而非旧假绿）**：在自有scratch创建Ark工程，目录名包含`exited 1-r6-`，**不创建hvigorw.bat，也不安装SDK**。经当前真实交付auto-ui-tool发送build_ui(ark)，实际report.outcome=failed，wrapper_exists=false，原因明确：

```text
Hvigor wrapper not found at D:/autostack/.wt/lang-831/scratch-ui-repo-fixture/exited 1-r6-j5a_ghw0\gen\front\ark\hvigorw.bat
— open the project in DevEco Studio to build (no build was executed)
```

原样V06c汇总把路径里的`exited 1`当作真正退出码，将其判为real_fail/aggregate=False。仅把**该真实错误文本的目录前缀**换成ordinary-r6-，原因仍是同一个未执行/缺wrapper，汇总立即恢复env-gated/True。r3规定此缺环境可延期，所以两者应同为绿。这里未伪造执行失败、未要求安装已移出验收的工具链，也未增加新的规模或后端承诺。

该控制覆盖的是同一分类器的另一条真实producer形态（缺席句头+附带路径），与已闭的“真实执行退出码+附带路径”互为合法对照。应从producer形态的语义位置识别退出码/缺席原因，不能继续在附带路径内搜索数字；对真实失败及未知失败仍红，对r3已批准缺环境仍绿。不要只对exited 1目录加特判。

最小闭合：保留R4/R5真实exit1红控制和原五项；补该真实缺wrapper的带/不带路径数字对照，确认原因类别由原因决定，而非用户路径；如对共享退出码规则调整，再选一个未知/兄弟形态。V06e已闭、其他产品族与整档不用重跑。

## 证据及路由

`replay.py --lang-repo D:/autostack/.wt/lang-831/auto-lang`在内存适配R4诊断脚本：只改工程名、移除控制wrapper创建、把缺环境的期望改为True。它走真实工具，产品源文件只读；原始响应/分类对照及旧控制摘要见controls.json，源码/依赖/Spec/工件hash见receipt.json。

结论needs_fix，仅F03-A分类仍partial（旧假绿已闭、本轮为缺环境误红的修复回归）。AC-03/06的分类验收未闭，AC-07未发pass；其他未变有效范围与SD-01..07方向/r3延期保留。T-09重开，Plan revision3保持、executing8/12。最终pass后merge-owned裸cargo t/实际触面档、T-11/T-12仍pending；tf主检出批量，U831未成立、832不放行。未改产品实现/跑整档/合入/沉淀/清树。
