# PLAN-831 T-10 R1：needs_fix

日期2026-10-24；Plan r2；review_scope=final（本次送审目标）；outcome=needs_fix。尚有work-owned必需验收未完成，不进入最终pass或merge门。检查者为当前Codex会话；未创建新会话/子agent，不能把本轮称作全新独立上下文终审。上轮在本会话仅做T-09核查，本轮从实际提交/断言重建结论。

## 基线与实际写面

| 项 | 绑定值 |
|---|---|
| lang worktree / branch | D:/autostack/.wt/lang-831/auto-lang / codex/plan-831-dev |
| reviewed_commit | 1635835d377bd3faf12bb9ab642e14cebc150f94 |
| repair diff base | eaf1027e5e4d7728bd2978ea48ad72927230ed6d |
| UI worktree / branch | D:/autostack/.wt/lang-831/auto-ui / codex/plan-831-dev |
| reviewed UI commit / repair base | 464569d58d9d9c580616a811921b35bbac4265b5 / 461c6098410b836023d490dcbfab588a733e7546 |
| dependency down | c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（组内只读） |
| shared Plan / canonical inputs | 主检出master@2d185e5cb4fd19ca3406aa16c2522f1ded8d31c1；当前文件hash见receipt.json |

两实现树tracked文件clean；语言树唯一untracked为compile-probe/Cargo.lock（可再生锁，未包含未提交实现）。该事实不等于测试设施可以省略对tracked dirty source的检查。复审未改产品代码/依赖/配置，未运行整档t/tv/tt/tb/tu/taa/tf、未合入/发布Specs/修改ledger/清树。

## 定向验证与复用

| check → finding/invariant | 方法与实测结果 |
|---|---|
| 原F831-09-01反例 | UI树 `cargo test -p auto-ui-compiler --lib app_registry::tests::scan_examples_ui_finds_at_least_27_apps -- --exact`：exit0，1 selected/1 passed/0 ignored，原0-app断根已修。 |
| F831-09-02 | lang树 `python test/fixtures/modularity/ui-repository-consumer/driver.py type --phase source-switched --compile-probe --output ../scratch-ui-repo-fixture/review-r1-type.json`：24 pass/0 fail/0 pending；A default运行、B Core check、C显式UI check及双source解析拒绝都执行。此范围闭合，不再要求无关全套测试。 |
| F831-09-04实际围栏 | 777 strict exit0、0 violations/5 grandfathered、warnings=[]；UI新根真实扫描362文件，非旧0文件空转。t/tu/ti/tn与跨仓队列实际存在；保留迁前后选择面对拍工件与ti历史证据，不将原始行数当作本轮独立枚举结论。 |
| F831-09-06 | 根LICENSE存在；重复#[test]与fastrand修复在diff中；定向构建compiler lib test57条预存告警，沿用告警归因材料。`git diff --check 461c609 HEAD`在UI树报node_converter.rs:874新增空白行；属清理项，不单独提升为产品缺陷。 |
| 来源与收据 | freeze.json所列四材料+三证据工件共7个hash均匹配。scratch原始792/dist/drill的sources与当前完整Git SHA一致；792确为38/39，V06c明确failed。dist5/5、drill19/19保留其实际范围，未重跑重型发行全链。 |
| VM验证器区分性 | 原样执行V06e代码块，仅将工具进程替为真实idle Python子进程（不读请求、不输出帧、不创建VM/窗口）。8s后记录 `V06e-vm-run-ui-real ok=True`；能稳定复现假绿。 |
| 安装验证器区分性 | 原样执行V06d代码块，工具替为真实framed子进程：立即返回build_ok/completed后idle，从未安装/构建。6s后仍记录 `V06d-install-cancel-retry ok=True`；能稳定复现假绿。 |
| admission区分性 | scratch真Git仓先提交source.rs=1，合法receipt通过；同HEAD tracked源改999，git status=`M source.rs`，同receipt仍admitted=True。 |

后三个负控是验证器的真实输入替换/源码突变，不代表真实产品一定存在同样执行缺陷；它们证明当前pass不能证明所声称不变量。可复用重放见 [replay-oracles.py](replay-oracles.py)，示例（cwd=lang权威树）：

```powershell
python D:/autostack/auto-lang/docs/plans/attachments/831/review-r1/replay-oracles.py --lang-repo D:/autostack/.wt/lang-831/auto-lang --case vm-idle
python D:/autostack/auto-lang/docs/plans/attachments/831/review-r1/replay-oracles.py --lang-repo D:/autostack/.wt/lang-831/auto-lang --case install-idle
python D:/autostack/auto-lang/docs/plans/attachments/831/review-r1/replay-oracles.py --lang-repo D:/autostack/.wt/lang-831/auto-lang --case admission-dirty
```

负控在本基线输出false_positive=true、退出2；修复后必须能拒绝错误载体，同时实际产品合法正控保持。脚本只写新scratch，不修改产品文件/分支。

## 开放finding（不以债账豁免）

### F831-09-03 · P1 · REOPEN：真实支持/运行/恢复必需面仍未闭合

规则：AC-02/03/04、V-03/04/06和§5.3要求真实适用profile构建运行与寿命/失败恢复；Plan原文明确“环境缺口不可当绿，也不自动批准停用后端”。

已确认事实：终源792原始receipt的V06c=false，jet=9009、ark=no build executed；handoff仍将V-03标pass、F03 CLOSED、blockers无。Vue/jet/ark浏览器/服务器及tauri/vscode运行面被dispositions记账，没有等价实际成功证据。具名环境红有助诊断，但不构成验收完成或支持缩减授权。

验证器缺口：generated-ui-consumer/driver.py:1154仅`mid_alive and retry_ok`，没有证明6s时安装未完成，也未保护/对拍旧安装字节；:1192仅`vm_ok = mid_alive`，丢弃工具所有响应，注释称“协议cancel”但实际仅taskkill树。上表idle/framed反例均假绿。现有MCP窗口/首帧/select_rect/进程终止摘要亦未证明终端/媒体实际操作及owned资源在返回前释放，不能替代寿命合同。

原因已确认：验收断言主要观察工具宿主存活/后续报告，观察对象不足；真实支持矩阵还有明确未成功项。未断言“真实VM产品已坏”，环境红亦不冒认回归。

闭合范围：让上述两个错误载体确定性红；从产品协议/真实VM首帧或子应用事件证明run接受且实际运行，按已支持cancel/退出路径观察owned进程/reader/媒体终端资源收口；安装取消前证明安装活动、验取消/失败阶段旧字节与陌生数据保持、恢复后真应用复绿。补齐原支持矩阵的有效环境实测（或由用户正式修订验收合同）；未成功保持partial/blocked，不能改成pass。保留已绿Rust窗口、Vue/rust/vscode构建、生成六profile等证据；只重跑受影响检查，不启动全局门。

### F831-09-05 · P2 · REOPEN：同HEAD源码变化仍采信旧收据

规则：C1/C3、AC-01/03/04/06、V-01/03/04要求producer/依赖/config/input身份准确，错误source/漂移负控拒绝。driver.py:796..830的admit_receipt只比HEAD，不验证tracked源码状态/内容；其注释“干净树commit唯一承载”是未执行的前置假设。

真Git scratch合法正控通过；tracked source.rs字节1→999且HEAD不变，旧收据仍admitted。该反例覆盖共享lang/UI/down检查逻辑；不是要求拒绝可再生的untracked Cargo.lock。

闭合：按本合同验证receipt实际覆盖的代码/依赖/config/input，tracked变化不可视同冻结commit；dirty与错source负控红，clean及允许的可再生产物正控绿。执行载体须绑定所用二进制/producer身份，而不是仅往JSON写当前HEAD。旧同源绿证据按影响范围复用。该finding是证据准入设施缺口，当前两实现树tracked clean的事实保持。

### F831-09-01 · P2 · PARTIAL：原断根已闭，blueprints同类断言被放宽

原app_registry反例本轮绿；但UI registry.rs:462..473的fallback测试把先前独立expected-root精确比较改为调用被测函数得到expected，再仅检查路径含blueprints。无根时返回不存在`packages/blueprints`同样通过，无法区分正确固定单源和旧失效fallback。测试名仍声称compile-time root。这是F01共享根路径验收遗留，不是重开已通过的app_registry样例。

闭合：独立构造预期owner/路径和实际fixture读取断言，合法cwd/env覆盖保持；无效fallback/错owner控制失败。检查生产resolve在独立clone/外部cwd的原承诺范围，不以contains字串替代真实输入可用。仅跑root-resolution及受影响blueprint消费者代表。

### F831-R1-01 · P2 · OPEN：T-09候选来源与SD仍是旧结论

规则：T-09/AC-06/AC-07要求最终source/SD-01..07可审查，T-10须绑定当前Spec输入。

freeze四文件hash虽自洽，但source-audit.json的lang/UI仍eaf1027/461c609并含旧失败反例；handoff正文称2ea9b5a的830同blob对账，实际source-audit的那次UI是461c609。UI“完整SHA”列是12字符缩写。SD草案仍proposal-only/needs_fix、F02/05/06未闭等旧描述，handoff明确本轮未刷新；不能以另一页声称闭合替代实际终稿。三个canonical Spec目标已被并行PLAN-840更新：auto-ui-runtime/project.md、language-sdk/project.md、auto-lang/project.md，冻结spec_inputs hash不再匹配；应保留新规则并明确对本delta的兼容性。另tier-parity内tu基数865/867表述须对账，不按叙述数字自动判全量ID覆盖。

闭合：保留旧快照为历史，并重新生成最终源净迁移/测试选择面和最终SD；各证据分类来源/实际SHA/hash/范围，校准当前canonical输入及与840并集的关系，保持SD IDs与Plan语义，不盲目覆盖canonical。更新freeze/handoff/闭合表后重新提交，不能仅改outcome或摘要。

## 全AC/SD覆盖与下一步

| AC | 本轮结果 | 证据边界 |
|---|---|---|
| AC-01 | partial | 真实包/DAG和SDK反控历史绿保留；类型专项24项绿；来源admission设施F05仍有缺口。 |
| AC-02 | partial | 兼容专项通过，C2历史语义保留；真实宿主寿命F03未闭。 |
| AC-03 | partial | 终源fresh/locked/重定位和六profile生成保留；V06c实际红/运行缺面，不能pass。 |
| AC-04 | partial | 资源/pin/patch/陌生数据/四状态drill绿保留；真实取消恢复oracle不能区分错误执行。 |
| AC-05 | pass within work scope | 历史保全载荷/恢复和终源drill保留；真默认分支落地仍T-11，不提前宣称完成。 |
| AC-06 | partial | 实际CI入口和围栏改进确认；根同类oracle、最终来源/SD结论仍有缺口。 |
| AC-07 | fail / final pass未发 | 当前会话独立性限制如实记载，必需work面未齐；退回work，不以最终review替代环境验收。 |
| AC-08 | pending merge | 未执行统一门/实际交付/Spec-ledger/归档清理，保持原责任归属。 |

SD-01..07全部已检查：目标集合/GOAL-003,007沿用原Plan；未新增支持/退役操作。共享声明/API owner方向与实现一致；仓属/发行/测试/恢复终态必须按F831-R1-01及F03修正。当前canonical的840输入变化须协调，故SD不能最终pass；审查快照与各输入hash见receipt.json。

结论：撤回T-09 work-complete主张，Plan回executing/current_step8，T-09与T-10保持未勾；不重置无关T-01..08历史勾选。F01原样例/F02/F04主要布线/F06许可证与重复注册修复得到确认；上述开放范围回同组work处理。merge-owned裸cargo t及实际触面档继续pending，tf仍仅主检出批量档。U831未成立，832不放行。
