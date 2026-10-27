# PLAN-815 T-05 R8 / T-06～T-08 提前 R3 联合定向复审

2026-10-22，**needs_fix：五项中四项CLOSED，一项PARTIAL，无新增finding。** 剩余F-815-T07-R1-01 / P2：只有CSS引用而没有任何JS加载的入口，仍通过dist成功oracle。完整T-07发行/资源/字段等已知未做面继续待办，不重复编号。本轮不是T-09整体终审，Plan r3保持executing10/17。

## 冻结对象与证据

- 唯一树D:/autostack/.wt/lang-815/auto-lang，codex/plan-815-dev，HEAD **c050e718e09b66ebb6d7925c1f4e0ce89bb10dc5**，code **e351cb2a73a78f353457e56ea20cb82f61c1ac88**；diff base e3fdcdc76c5d28ec9274a9101acc3f6a80c5c6a6。docs提交无额外生产diff，M814-B交付祖先93a25c924d1beafab130588bad4d1041dc86e48c保持。
- 实际工具SHA256 **f21db18136403d1124ecbc1338f61c5603544401c434e7d12fad20a60c37dfb5**，与handoff一致。开始/结束同HEAD clean，源码/锁/依赖/配置/Spec输入无漂移，见[baseline](baseline.json.txt)、[终核](final-baseline.json.txt)。
- reviewer=Codex `/root`，延续独立审查上下文，有早期合同/账本参与限制，未参与本次实现；按/auto-plan:review重建证据，未创建agent/第二树。只写主检出复审簿记/附件及私有target测试产物；未改实施/canonical/live ledger，未合入/归档/清树。
- review_scope=phase：用户交接的五项修复及直接受影响邻接路径，非整体完成验收。[冻结交接](frozen-handoff.md)与[SD逐项核验](spec-delta-reviewed.md)绑定本轮输入。允许复用的旧关闭证据仅覆盖未变且适用范围；不据总测试数给完整B面pass。

## 五项关闭表

| ID / P2 | 结论 | 独立结果与边界 |
|---|---|---|
| F-815-T06-R1-01 | CLOSED（本项请求目录贯穿范围） | 默认/自定义Rust真实生成与cargo argv/CWD正控通过；节点优先压过prop及默认目录的实际生成只含所选B标记；同项目切换已有front目录生成B、修改选中目录生成C，产物不再保留A。compile/regenerate/needs_regeneration/delivery源选择/run CWD五位点均读front_dir_for，legacy链保留。资源/发行完整矩阵仍归T-07，不以本项关闭冒称通过。 |
| F-815-T06-R1-02 | CLOSED（原生成ID/诚实结果范围） | 原样生成AppScope/module ID经过真实构建→probe→install→start派发得到干净com.example.t05probe/EntryAbility。缺bundle、缺ability、不可读bundle各具名failed且没有start调用；不再猜默认ID。私有wrapper/hdc仅为argv/失败控制，不计真实HarmonyOS SDK/设备执行。 |
| F-815-T07-R1-01 | PARTIAL | 原缺文件/无任何引用负控现exit101，正常/嵌套JS引用exit0，真实成功构建通过；但CSS-only/no-JS入口仍exit0，详下。 |
| F-815-T07-R1-02 | CLOSED（原排水/关闭等待范围） | 原文辅助器对私有不退出peer在10.11s自行panic exit101，timed_out=False，未用外层34s强杀；正常EOF的lifecycle三项均exit0。131072B stderr失败例1.26s收口且成功oracle变红；Drop改为kill后轮询有界。不是新增产品构建10s SLA。 |
| F-815-T05-R7-01 | CLOSED | Windows/Linux完整pkg模块原命令均9/9；缓存稳定与第一次磁盘读取分开，新鲜子进程独立证明Bun/DiskLegacy；来源断言未削成只看绿色退出，legacy缓存语义未改。原矩阵单独继承/bun环境均0，真实工具无pkg/磁盘npm实际选bun保持。 |

Rust原例及组合详[rust-front-cwd.json.txt](rust-front-cwd.json.txt)、[rust-front-combinations.json.txt](rust-front-combinations.json.txt)；Ark原例及失败控制详[ark-launch.json.txt](ark-launch.json.txt)、[ark-metadata-exits.json.txt](ark-metadata-exits.json.txt)；关闭辅助器详[lifecycle-close.json.txt](lifecycle-close.json.txt)。所有脚本同目录可重放。Rust cargo/Ark工具链为私有记录器，仅验证真实产品生成、来源选择与外部命令派发，不虚构本机完成原生UI编译或设备运行。

## 唯一未闭项：F-815-T07-R1-01 / P2

**AC-04/V-04，T-07，原可消费dist成功oracle的完整性残留。不是新范围，不是实际Vue编译器产物损坏的结论。**

前轮要求入口真实引用应用束；本轮成功面同时检查“目录存在JS”和“入口有某个assets引用”，两者仍无关联。冻结lifecycle.rs:422–434统计任意.js文件；:436–456对任意assets/文件计数，CSS也使references>0成立。于是入口不加载应用JS也可通过。

独立重放[dist-reference-extra.py](dist-reference-extra.py)，逐字提取**冻结源码的产物断言块**（不修改断言），给三个实际资产文件assets/main.js、assets/nested/main.js、assets/style.css及下列index.html：

```html
<div id="app"></div><link rel="stylesheet" href="/assets/style.css">
```

入口没有script或任何JS加载方式，只加载CSS。**实际exit0**；目录中的JS文件未被引用也未执行。对照：正常JS引用及嵌套JS引用exit0，MISSING.js和无任何assets引用各exit101，见[完整原文断言、文件清单与五个控制](dist-reference-extra.json.txt)。原三例[回放](dist-reference-oracle.json.txt)也保留。这是验收器灵敏度控制，不能把它当产品实际生成了这种坏入口。

原静默安装exit7与131072B日志反例均使真实成功测试exit101，实际正常Vue构建也通过；这些已闭事实保留。剩余问题是本Vue打包成功面尚未证明入口会加载应用JS，CSS存在性不能替代它。

**关闭oracle**：按本生成器真实Vue构建产物形态，验证入口实际加载应用JS入口及其存在性；CSS-only/no-JS负控必须使成功oracle失败，正常/嵌套JS、真实构建继续通过，原缺失/无引用负控保持。保留合法支持形态的明确对照，不靠删除CSS-only负控、只看任意JS文件、改名或扩大环境豁免补绿。无需为本项引入通用HTML校验器或新增产品平台。

## 本轮验证账

- Windows：pkg模块9/9、Ark解析1/1、tool_process默认21/21+指定ignored串行8/8、legacy ignored2/2、真实lifecycle ignored3/3（42.35s）。PM双合法环境、Rust声明优先/目录切换/增量内容、Ark正确ID与三类失败、原文dist断言变异、关闭截止及日志控制另列，不重复累加为Rust唯一用例。
- Linux：既有WSL Ubuntu-22.04工具链，冻结源码--locked --offline、target=/home/visus/plan815-unix-target，同pkg模块9/9。未安装/修改用户工具链，不外推完整Linux runtime/real_tree矩阵。
- 777：本分支只读strict独立复跑 **pass，0 violations/3 grandfathered**，见777.log.txt和777-report.json.txt；本轮无manifest/依赖边/基线改动，不是master并集或792发行通过。
- 修复净改四个Rust文件rustfmt逐字相等，见fmt.json.txt，生产diff --check通过；没有本轮修复新增编译诊断，core/runtime与ui-tools22等旧诊断保留，不冒称全库零告警。私有原文harness的未用import诊断不计产品问题。
- 原closed捕获/取消、纯协议闭包负控及fixture卫生按输入未变边界沿前轮证据保持；本轮未重跑全包lib，不将work声称177/181写成独立全绿证据。未运行整档cargo t/tv/tt/tb/tu/taa/tf/docs_gen。

## T-07、T-08与后续

完整master并集、792/fresh/locked发行、资源/许可审计、V-01..04字段/兼容/失败矩阵、协议fixture四种服务端回执与深度负控、已披露的real_tree Linux可移植缺口仍是work承认待办，不每轮重复编号。T-08源/bin身份和本次Rust/Ark-ID声明相符；完整T-07与最终SD发布性仍pending，SD-01/02必须基于现行A规范增补并保留来源。

本次修复范围AC-01/02/03受影响内容通过，AC-04产物oracle一项partial且完整面pending，AC-05本轮格式/新增告警范围通过，完整T-09仍pending。A/L/T-04历史和此前已关闭原例不重开。T-05～T-08整体任务继续未勾，Plan executing10/17；不得将修复范围通过外推整任务/整Plan发布。

原唯一树work修完这一残留并保留正反控，可按既有授权继续补完整T-07、定稿T-08后统一交T-09；后续仅重审受影响范围并复用有效旧证据。最终独立pass后merge owner再执行冻结裸cargo t及实际触面档、串行合入/Spec ledger/发布/归档收口。本轮不提前合入。

探针校正：关闭harness改用独立包名plan815-r8-close-probe，避免与前轮共享target的同名包混淆，无清理共享缓存；Ark缺ability初版沿用前个case的坏bundle，未抵达目标状态，已保存invalid-setup并在每例恢复正常元数据后重跑，修正结果才作为证据。未把这些探针问题计产品finding。
