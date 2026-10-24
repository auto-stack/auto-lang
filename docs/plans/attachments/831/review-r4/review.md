# PLAN-831 T-10 R4：needs_fix（Plan r3，仅一个窄残项）

日期2026-10-25（跨日收尾），review_scope=final目标下的受影响修复复审。接受r3环境延期；不重开jet/ark缺工具链本身。当前Codex会话检查，非新独立上下文，无新agent/checkout。未发最终pass。

## 绑定输入与复验

- 簿记提交：71ac28000（当前master祖先）；实际主检出检查位：f0c0f9dc79b23b49b6afee85c14c10a15d7c35e5。
- lang：D:/autostack/.wt/lang-831/auto-lang，codex/plan-831-dev，3064b34656af7b90511938d721624e1ba942ca53；repair base=2d47e236eb7de6c9d275bbe7629b822c788f1820。
- UI：D:/autostack/.wt/lang-831/auto-ui，codex/plan-831-dev，425088520e402b1efae9479b9d0b5b3d44a56df4（未变）；down只读c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8。
- 两实现树tracked clean；lang可再生untracked compile-probe/Cargo.lock保留。freeze7材料hash、4原始终源收据hash/sources匹配；canonical spec_inputs无漂移。复审不改产品代码/依赖/config，不跑整档、不合入/沉淀/清树。

**原R3控制均通过**：`review-r3/replay.py --case build-matrix`的5个控制都符合同；`--case endpoint-timeout`现为case红、matches_contract=true，独立TCP取证仍监听，detail明确endpoint_closed=False/TimeoutError。故F03-B端点超时当关闭已闭，F03-A原两个无歧义编译错误样例已闭，正向completed/获准缺环境/required失败均保持。

终源实际收据仍39/39；V06e真实Launched→HTTP200→cancelled→ConnectionRefused(10061)→describe成功证据保留（来源及hash已核对），不重跑无关完整792链。UI所有源码/输入/配置未变，原blueprints5/5、兼容24/24、777/测试选择面/许可归因/净迁移及其他未变范围继续复用。

## F831-09-03 A · PARTIAL / OPEN：错误文本中的任意not found仍豁免真失败

**规则**：r3明确只允许缺环境转后；在场执行后的真实构建失败仍必须传播红。不是要求恢复jet/ark实机工具链为本Plan完成前置。

**已确认机制**：driver中`BUILD_ENV_ABSENCE_MARKERS`含泛化`not found`，`env_gated`对截取的整个error文本作substring匹配，包括用户项目路径和建议命令。于是已执行的失败也可因无关路径字节被错误豁免。新增real_failures只能接住未被误分为gated的错误，尚未闭合分类不变量。

本轮先对有效framed失败报告做了两项诊断控制（Android资源缺失、Ark源模块缺失），均被env-gated。随后检查真实tools实现确认：jet/ark通常只把wrapper退出码与项目路径写入report，编译器原始stderr并非必然进入该字段。**上述模拟编译消息只作为诊断，不把它们单独当生产路径缺陷证据**；正式finding使用以下可从当前真实工具到达的输入。

### 真实入口与反例

1. 在组内自有scratch创建合法Ark工程，根目录名含字面`not found`；正常请求的project_root，未违反任何路径合同。
2. 在其`gen/front/ark/hvigorw.bat`放置有界控制wrapper：写marker后`exit /b 1`，证明wrapper存在且实际执行后失败，**并非缺工具链**。不假装安装过DevEco，不运行真实编译器。
3. 通过当前真实交付`auto-ui-tool.exe`发送合法`build_ui(ark)`。实测wrapper marker存在，协议返回`build_ok`但report.outcome=failed/error如下：

```text
ark build failed (hvigorw assembleHap exited with exit code: 1);
try manually: cd D:/autostack/.wt/lang-831/scratch-ui-repo-fixture/not found-r4-na69wbqk\gen\front\ark && ./hvigorw assembleHap
```

4. 把**此真实错误文本/失败状态**通过合法framed报告输入原样V06c汇总块（其他profile用completed合法对照）：ark被记为env-gated，aggregate ok=True。只把同一错误文本里的目录前缀`not found-r4-`换为`ordinary-r4-`，相同exit1立即传播红。差异只来自与失败原因无关的路径字节。

这同时证明真实来源可达性、错误结果、判据偏差及共享原因。wrapper只是确定性失败注入，不把“执行过wrapper”冒称真正Ark编译成功。该finding在验收器，未断言当前真实生成器有编译回归。

### 最小闭合及边界

只把已明确证实的环境缺失归为r3延期；不能让项目名、文件名、资源/模块缺失或建议命令中的通用字串决定env_gated。适配当前实际producer的具名失败形态并保持误差边界，未知错误应红。不得仅加一个对`not found-r4`的黑名单来修反例。

保留原5个控制与真实缺wrapper/9009环境正控；补真实exit1错误中有/无无关路径标记的对照，以及共享规则的一项兄弟错误或未知异常。仅检查V06c分类和其当前producer错误格式；V06e已闭、其他产品族无需重跑，整档仍待merge。

可重放：[replay.py](replay.py)，调用：

```powershell
# cwd = D:/autostack/.wt/lang-831/auto-lang
python D:/autostack/auto-lang/docs/plans/attachments/831/review-r4/replay.py --lang-repo D:/autostack/.wt/lang-831/auto-lang
```

本基线生成一个自有临时工程，真实工具得到wrapper-invoked=true/outcome=failed；classifier分别输出with_marker=True、without_incidental_marker=False，脚本退出2表示合同反例仍成立。真实响应/原5控制/端点控制摘要见controls.json，代码与输入身份见receipt.json。

## 路由与结论

只保留F831-09-03 A一个窄残项，B关闭；不新增验收ID、不更改Plan revision3或r3环境延期。SD-01..07、当前目标hash/owner/840并集、其他已闭finding方向保留；AC-03/06受上述失败准入oracle限制，AC-07仍未发pass，AC-08及merge-owned裸cargo t/触面档pending。

T-09重开，Plan回executing8/12；保留T-01..08和全部历史绿。下一work修分类不变量、受影响正反控后刷新冻结；不再执行已闭V06e/完整发行矩阵来掩盖该残项。tf仍主检出批量，T-11/T-12 pending，U831未成立，832不放行。当前会话独立性限制如实延续。
