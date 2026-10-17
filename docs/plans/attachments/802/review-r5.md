# PLAN-802 final review R5 — pass

2026-10-17 / plan_revision=1 / review_scope=final。当前Codex会话独立于实施
上下文，沿用R1–R4审查上下文；未创建agent、未修改实现、未合入。

## 基线与证据身份

- 原树 `D:/autostack/.wt/lang-802/auto-lang`，branch `plan-802-dev`。
- reviewed_commit `b32d4df7d3a4a43746a22ec4b8d41d1856dc9ab2`。
- base `1dd123361a75aa4b683376dcb2049299122724c9`。
- auto-down dependency `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- main入场 `282e50053da41c3b65c1e424ca0d251f1126986b`。
- 原树status入场/结论前均空，HEAD/依赖不变，diff-check通过。
- 当前auto.exe SHA256 `D916FF74D4FE7B2D2D1CFBE1B101F41C847E0C29CA7A1EF51981066CB47158F8`，与新实机JSON一致。

| 输入 | SHA256 |
|---|---|
| Plan入场 | 5ED15B0BE24D0F285FBE4AE35B4549B64897EB3AB4357B107A61A35442D1AA3B |
| v06_driver.py | 36BCCA5211B9159BB5B4771F025AAB67874688A03D0C304126114916CF6B2D72 |
| v06-evidence.json.txt | DEBEED262BADCD48DD38799BCBC70DFE9C0D4241DCA14F2348902356C05E354A |
| docs/specs/auto-lang/vm/overview.md | 9D792387AB8822D21AED0DEBA5818DCF17BE71A5AA8BE0D9F5242DC3307F0E91 |
| docs/specs/auto-lang/ui/overview.md | F7C54E4ED71BB3598F829F06FE0B97C772ACFE0827B0D81505640B4426A1A160 |
| docs/specs/auto-lang/ui/design/code-editor-session-endpoints.md | 2225A35E4C39E7FC69C57E2177FA728FA0FF4A68C4CF9AAE6B175187C39C52C3 |

## Finding闭合与本轮验证

work的R4诊断正确：测量误差不能直接变为成功门限。新三分判定对首次
新值F、末次旧值L区分certified/violation/indeterminate；只有certified
授pass。查询耗时不再进入成功门限，保留独立观察停滞门。固定ε=0.05s
为时间戳精度容差，不能扩展为依赖查询时长的产品延迟；本次真实样本
在原2s内，不依靠ε通过。

- 新六态 `--selftest` 本轮独立重放PASS：有效0.5s通过；2.5s迟到、2.2s带查询耗时、停滞tick、错误clamp、缺Reader均拒绝。见review-r5-selftest.log.txt。
- 独立原样重放R4输入：保留原0.5s/2.2s延迟与每查询0.25s，仅反转预期判定；有效对照保持PASS，2.2s病例不再PASS（跨阈值不可认证）。见review-r5-replay.py / .log.txt。复现脚本引用原R4脚本，未改输入或实际driver。
- work新实机JSON与当前二进制一致：Reader首见1.713s、端点首见1.797s（均从输入前计时）、计时窗内offset7241一致、tick推进；clamp实测7541与几何上界7540.800偏差0.200px；未绑定反控通过。逐条核对12必验+1辅助，未仅信ok_all。
- 程序化路径分相观测窗口150与端点150；同drain命令投影由renderer先note_controller_offset再安排滚动的已核源证据支撑，不将异步采样当亚帧测量。
- 本轮未独立重跑真窗口/滚轮，明确采用已核产物身份的work实机原始证据，并独立验证oracle与失败传播。

F802-R3-01 CLOSED；F802-R2-01、F802-R1-01随完整V06证据闭合；
F802-R2-02格式保持CLOSED。无新增finding，无合同内延期或workaround。
P802-D1局部槽≥16预存限制仍是既有范围外债，fixture拆段没有削减22项覆盖。

## AC与复用边界

| AC | 结果 | 验证 |
|---|---|---|
| 01 | pass | R3独立no-ui真VM全族、互异参数/ID，以及源实现边检查；R2 metadata无renderer闭包 |
| 02 | pass | 原臂矩阵、缺席错误和值形、单位/IO/delta/ownership、真实adapter定向验证 |
| 03 | pass | 默认/实例/Clone/merge寿命与真实adapter、组件滚动/剪贴板，现已补齐可拒错的V06窗口时序证据 |
| 04 | pass | 全族迁位/剩余边/a2r未动/闭包metadata、证据及SD齐全 |
| 05 | pass | 独立逐项检查、旧例和不变量闭合、格式/遗漏扫描、Spec delta审查完成；冻结提交绑定 |
| 06 | pending merge | 全日常/触面门与规范发布、ledger、归档、guard清理未做 |

复用R3独立consumer7/7、定向27/27、fmt PASS及R2 no-ui metadata的理由：
当前源码HEAD与R3/R4完全相同，依赖完整SHA一致，Cargo/feature/测试配置
和Rust测试输入未变；本轮仅验收driver/证据/簿记修复。没有把work测试数
冒充本轮重新执行。没有运行裸cargo t或整档tv/tu/tf。

## Spec delta与下一步

审查冻结文字见[review-r5-spec-delta.md](review-r5-spec-delta.md)，对应
SD01/02/03三条精确canonical路径。before=原shim直引/未登记该迁移，
after=22项owned provider、四装配点与单一registry、同源端点原合同保持；
理由与AC映射在冻结文件内。无新Spec组件，GOAL-003/007真实有效，保留
现有其他宿主族及并行事实；未修改canonical或live ledger。

final pass，Plan→reviewed，T07完成，current_step=7/8。next=merge：先同步
当前master并核对受影响接口/依赖/闭包，必要时复审受影响内容；在最终
冻结提交执行裸cargo t+tv（未触ui_gen/aavm，无额外tu/taa），通过后才能
合入。Spec/ledger/索引/归档/guard清理均pending。tf仅按到期规则在主
检出批量单实例。此pass不代表已交付，不授权跳过merge-owned门禁。
