# PLAN-831 T-10 R5：needs_fix（r3，仍仅F03-A分类残项）

2026-10-25；review_scope=final目标下的受影响修复复审。当前Codex会话，非新独立上下文；未创建agent/checkout。r3环境延期继续有效，V06e和其他已闭面不重开。

## 基线与已通过项

主检出检查位d76b2691def0029f82d777f8775cf0cc39022dc6；用户指明簿记eed3a1712为其祖先。lang权威树D:/autostack/.wt/lang-831/auto-lang，codex/plan-831-dev，reviewed=538e3b8fefef4b651b138201234e7c6fa50bcbfa，repair base=3064b34656af7b90511938d721624e1ba942ca53。UI未变425088520e402b1efae9479b9d0b5b3d44a56df4，down只读c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8。

两实现树tracked clean，lang唯一可再生untracked compile-probe/Cargo.lock保持。7份冻结材料、4份原始收据的hash/sources匹配，当前canonical Spec输入无漂移；修复diff-check通过。当前lang仅改一个driver的36行，V06e、产品代码、配置及依赖均未变。

本轮实跑：

| check | 结果 |
|---|---|
| review-r4/replay.py真实tool、项目名not found-r4-*、wrapper已执行exit1 | **通过修复检查**：带/不带该路径字串的同失败都传播红；脚本exit0。 |
| review-r3/replay.py --case build-matrix | **5/5符合同**：全completed绿、获准缺环境绿、jet/ark原编译错红、required/vue错红；exit0。 |
| 新规则共享边界：真实tool、项目名Hvigor wrapper not found-r5-* | **仍假绿**：wrapper确执行exit1，但汇总env-gated/ok=True；只去掉错误文本中该路径前缀，汇总红；脚本exit2。 |

终源真实792收据39/39及distribution5/5、drill19/19、fixture46/0/1/1，来源/hash均匹配。它们及未变兼容/blueprint/777/选择面/许可/净迁移/SD-01..07与r3延期的有效证据按既有范围复用；不重跑整档或其他已闭族。

## F831-09-03 A · PARTIAL / OPEN：模板片段仍会命中用户路径

R4要求的是分类不变量：仅producer确证的工具链缺席可env-gated，用户路径/文件名/建议命令中的无关字节不得决定延期；不是把`not found`改成更长字符串即可闭合。

新代码`BUILD_ENV_ABSENCE_SIGNATURES`细分了profile，但仍使用`any(sig in err_text for sig in signatures)`在整个error文本搜索。Ark的`Hvigor wrapper not found`仍可能出现在一个合法工程根目录名里；它不在真正失败原因的位置，也不证明wrapper不存在。

**真实来源可达的反例**：复用R4原样重放，仅把工程目录前缀改为`Hvigor wrapper not found-r5-`。对当前交付auto-ui-tool发合法build_ui(ark)，其现有hvigorw.bat写入marker后exit1。实测wrapper_invoked=true、report.outcome=failed，错误原文为：

```text
ark build failed (hvigorw assembleHap exited with exit code: 1);
try manually: cd D:/autostack/.wt/lang-831/scratch-ui-repo-fixture/Hvigor wrapper not found-r5-lwy8nvoa\gen\front\ark && ./hvigorw assembleHap
```

V06c原样汇总将此真实失败文本归env-gated，aggregate ok=True；同错误仅将目录前缀改为ordinary-r5-即RED。wrapper不是缺席，已实际执行，且exit1并非r3批准的缺环境。该控制不需要安装DevEco，也不把失败wrapper冒称真正Ark编译成功。

这是R4同一共享原因的剩余状态，而非新功能/新规模要求：规则仍未把producer失败原因与附带路径文本分开。不能用该具体目录的黑名单或再延长substring修单例；应核对当前producer具名原因的实际语义/位置，未知或真实执行失败保守传播红，已获准的缺环境正控保持绿。

最小闭合范围：保留R4真实反例、原5控制；补本真实路径干扰与一个共享类别的路径/建议命令干扰对照。判定应由真实失败类别决定，而不是错误文本任意位置的同名片段。无需修改已闭V06e、运行整套发行长链或撤销r3环境延期。

## 重放与路由

`replay.py --lang-repo D:/autostack/.wt/lang-831/auto-lang`只在内存适配R4诊断脚本的工程名（不写产品文件），走真实交付工具并保留实际响应/原样汇总。原正反控和本控制结果见controls.json，三仓身份、Spec/工件hash见receipt.json。

结论needs_fix，F03-A仍一个窄残项；原R4例已闭、分类不变量尚未闭。AC-03/06仍受该验证器影响，AC-07未发最终pass；其他未变有效范围及SD方向保留。T-09重开，revision3保持、executing8/12。merge-owned裸cargo t与实际触面档、T-11/T-12仍pending，tf主检出批量，U831未成立、832不放行。未改产品代码/依赖/config、未跑整档、未合入/沉淀/清树。
