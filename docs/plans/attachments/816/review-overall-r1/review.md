# PLAN-816 整体核查 R1（含T05-R2修复及T06..T08）

2026-10-23；**needs_fix：1项P1、6项P2**。当前不能给整B/整Plan最终pass，也不能仅把T05修复diff通过当成后续提交已获覆盖。状态保持executing **6/14**，T05..T09/当前复用T13未闭。

本轮按用户“修复已经完成，后续也做了，请整体检查”检查**当前完整提交**，不是只读历史e73ec6b06。已知work-owned Rust真链/发行矩阵未交齐，因此未进入merge-owned全档；仍完成可执行的独立回归和下述具体反例。没有修改产品实现、提交、合入、归档、清树或发布canonical Specs/live ledger。

## 基线、身份与边界

- reviewer：本聊天Codex `/root`，独立于实施会话；未建agent，未获模型/会话ID。
- Plan r2；唯一树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`；受审 **75dc3c058faa99fe4a52284dc790797c404b7089**，repair base **3c09bdf5b59d3f1aa4103b50c433a1bad3a327d8**，30文件 +2045/-204；中间修复e73ec6b06、T06=7ce08b5cf。auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；前后HEAD不变、实施树clean。
- [输入/Spec指纹](inputs.json)、[冻结SD](spec-delta-frozen.md)、[最终源/二进制/证据哈希](final-state.json)。T05 client/protocol修复源从e73至HEAD零漂移，但执行设施新增字段使其UI工具消费者在当前树不能编译，历史产物不能代替当前fresh验证。
- 当前CLI单独重建成功，并核验同字节副本；**UI工具重建失败**，既存auto-ui-tool.exe仅作旧产物身份记录，不把它作为当前提交真实正控。root auto.exe后来被既有测试daemon占用，未强杀共享实例，正控用相同hash副本/同源码driver独立manifest运行；设置误差和排除项见[说明](setup-notes.md)。

## Findings

### F-816-B-R1-01 · P1 · 新ExecutionSpec字段使真实UI工具无法编译

**规则**：AC01/03/04，完整profile真工具可构建可消费；T06新增设施API必须同步其真实消费者。

执行 `cargo build -p auto -p auto-ui-tools -p auto-tool-execution --bins`，exit **101**：[build.log.txt](build.log.txt) 的 **E0063** 位于 `crates/auto-ui-tools/src/proc_host.rs:160`，`ExecutionSpec { ... }` 缺少新增 `windows_creation_flags`。该结构体字段由7ce08b5cf引入，to_spec构造点没有同步。纯CLI构建成功不覆盖独立工具/旧Man兼容闭包；旧工具可执行文件仍存在也不证明新源码可构建。

**闭合**：同步实际构造点及其同族，当前冻结源重建tool/CLI/compat后，重跑工具实际生成/选项报告和相关B面consumer。应重新绑定新二进制hash，不复用旧exe来消掉编译错误。

### F-816-B-R1-02 · P2 · Rust service根退出后，stdout后代使CLI无限等join

**规则**：AC02/04、Plan §5.2、HTTP部署Spec §7，owned树和reader真停、受控退出有界，不能等到OwnedProcess Drop才清理仍持管道的后代。

[service-probe.py](service-probe.py)用合法pac和 `#[api] ping` 驱动**真实CLI→真实API生成→实际cargo spawn路径**，仅cargo替换为自带截止的寿命故障宿主。marker证明实际收到 `cargo run --release --manifest-path <真实生成backend/Cargo.toml>`。宿主写READY后根exit0；异常态只额外派生一个属于同Job的持stdout子进程。

- clean对照：CLI **exit0**，正常结束。
- owned-holder：根已退出，CLI **8秒仍未退出**；后代心跳71→74仍推进。外部终止本次CLI后心跳保持74，证明该后代确属它的Job，清理直到外部结束才发生。

见[原始结果](service-results.json.txt)。这是异常寿命反例，不把故障宿主当真实HTTP正控。

**原因**：`crates/auto-man-runtime/src/http_service.rs:331-342` 通过wait_deadline取到根状态后直接阻塞 `reader.join()`；并未先终止仍持stdout的owned后代，reader也没有可停读/有界join。READY失败分支的join同样无界，且错误路径可能先返回而未回收reader；`let _ = reader.join()`还吞掉panic，读错误被当普通EOF。A阶段已经解决过的“根退出不等于管道EOF”没有在新service adapter保持。

**闭合**：正常根退出、READY失败、取消/超时、读错/reader panic都应按真实终态先完成树/reader回收或具名报告，保留退出分类，不用共享端口kill。增加干净EOF、owned持管道后代、取消失败/reader错误的定向正反控，并交实际Rust服务bind/health/排空/端口重绑证据。

### F-816-B-R1-03 · P2 · 非Windows后台serve确定性被拒绝

**规则**：AC01/04，旧daemon命令行为和跨平台支持保持。

`crates/auto/src/main.rs:2861` 无条件 `spec.windows_creation_flags=0x0800_0000`，随后调用spawn_detached。新设施在非Windows对此非零值正确返回InvalidSpec，所以普通 `auto serve` 后台分支在Unix上不能启动；前台serve/req控制无法覆盖此分支。

[unix-serve.py](unix-serve.py)逐字取当前CLI的spec构造片段，调用当前真实设施API，在Ubuntu22.04原生执行，结果是 **Err(InvalidSpec("windows_creation_flags is Windows-only"))**，见[输出](unix-serve-result.json.txt)。没有启动daemon；这不是用Windows结果外推Linux，也不伪称已运行整份Linux CLI。

**闭合**：平台标志按平台消费，增加非Windows后台启动/真实req/退出归属控制；保留Windows CREATE_NO_WINDOW行为和设施对非Windows误配置的负控。

### F-816-B-R1-04 · P2 · 新watch验收对不再生产物的故障仍绿，截止也不包住阻塞读

**规则**：AC02/04、V02/04、T07，真实变更必须引起正确SFC再生，回归必须具区分性且自带有效截止/收尾。

新 `crates/auto/tests/plan816_cli_driver.rs:191-226` 只看进度文本、产物存在；没有检查变更后的文件内容/身份。其Ready/SFC循环的deadline包在blocking read_line外，也不能保证到期退出；run_bounded先等子进程退出才读管道，管道满/持写端时同样不能仅靠外层时间比较保证有界。断言失败前也没有保证child收尾的guard。

[watch-oracle.py](watch-oracle.py)将**当前测试文件逐字冻结，hash相同**，Cargo把AUTO绑定到明确的异常CLI宿主：先真实写旧SFC、收到实际app.at变更后保留旧字节、仅打印SFC标记。原 `watch_regenerates_sfc_on_change` 仍 **exit0**；故障记录同时证明 `input_changed=true / incremental_write=false / artifact_still_old=true`。见[proof](watch-oracle-proof.json)。这是验收oracle的异常控制，不代替真实CLI/生成器正控。

相同测试源绑定本轮真实CLI同hash副本，正常watch、脚本argv、serve/req三控制均过，见[positive-driver-results](positive-driver-results.json.txt)。因此并非单纯构造一个永远红的假宿主。

**闭合**：核对新内容/产物身份与缓存再生的实际效果，给阻塞IO与所有失败出口有效watchdog及owned收尾；从新测试块重做“第二次写被跳过/事件不再到达”负控，应及时红且不残留子进程。不要只加banner断言或把deadline常量调大。

### F-816-B-R1-05 · P2 · 机器JSON矩阵的已知必需缺口仍未收齐

**规则**：AC04、T05/T07、Plan §1/5.1，机器输出和逐命令矩阵是原work验收，不是可选改进。R1已明确将管理诊断污染JSON保留到T07核验，不因它预存就算完成。

[json-probe.py](json-probe.py)：合法 `auto --format json new <temp/newapp>` **exit0**，stdout却先打印Copying与完整模板文本，再打印成功JSON；整体不能作为单个JSON解码。相同入口的纯脚本返回42对照 **exit0、合法JSON**。见[json-results.json.txt](json-results.json.txt)。info样例缺pac.atom导致合法错误，已单列为观察，不拿它充当成功路径反例。

**闭合**：完成约定机器输出矩阵；框架诊断/模板进度与机器结果的流和格式需符合合同，保留真实脚本stdout语义，不以抽取“最后一个JSON”冒充正确生产输出。补成功与错误两路真实CLI断言。

### F-816-B-R1-06 · P2 · T07所称777清偿不适用于最终提交

**规则**：AC04、T07，777精确边基线必须绑定实际被检查的输入。

在受审clean HEAD运行 `python scripts/modularity-audit.py --mode strict --output <report>`，**exit2 / 1 violation / 3 grandfathered**：[777.json](777.json) 明确为 `stale_baseline`，锚定输入 **Cargo.toml** 自base_revision=7ce08b5cf已变。T07自身新增fixture exclude又改了根清单。没有新禁边，问题是声称本次最终基线strict pass却仍过期；不能把开工前或提交前结果记为清偿。

**闭合**：代码/清单修齐提交后，以最终输入重采样并在新冻结SHA复验strict与原负控；不能只改收据文字为pass或关闭stale检查。

### F-816-B-R1-07 · P2 · 必需发行/集成验收被延期，T07/T08尚不具备最终就绪条件

**规则**：原Plan §5.3/5.4、AC03/04、T07/T08：完整/Core独立profile、首次启动/重定位/升级失败、fresh/locked及真实service Rust组合必须有实际交付与验收；“owner_stage=work”不能移交review来决定是否补做。

冻结[工作handoff](handoff-work-frozen.md) §3明确把CLI/service发行闭包增补推给R07搬仓，称“本Plan不改792表”；§6又把fresh/locked/重定位全矩阵及service Rust真链留给最终review判定。原Plan却承诺792/安装manifest覆盖新的CLI/service/protocol/tool闭包。本次30文件中没有替代的完整profile清单/安装交付物；现有26项UI-app DELIVERY_CRATES不含auto/auto-tool-client/auto-man-runtime，UI-app交付证据不是完整CLI安装profile证据。**不强制必须将CLI塞进UI-app同一张表**，但需要有本Plan承诺的实际发行闭包与验收，不能全部未经批准后移。

T08还没有给当前CLI的具体文件hash；所引t06-smoke-results既无源/二进制hash，链接`.json`与实际`.json.txt`不符。仅列源码路径、早期修复SHA或历史UI-app fresh结果，不能覆盖之后改动的CLI/执行闭包。新增service预算600s/env的单位、非法值诊断和边界控制也需在合同/增量中明确：本轮bad-value实际静默用默认并exit0，未把未决非法值语义单独发明成产品finding。

**闭合**：补齐本Plan原有的实际profile/安装清单、来源hash/锁/资源与矩阵；实际Rust真链检查bind/ready/失败/关闭，不以VM真链或设施单测替代；更新可追溯handoff及Spec增量元数据。若确需删减/延期，走显式合同修订，不能用honest_notes或“review判定”消掉work验收。canonical沉淀仍归merge，本文不要求work提前改live ledger。

## 旧修复、通过项与证据限制

- T05-R2-01：Linux target client check **通过**，旧libc E0433消除。
- T05-R2-03：机械适配旧时序探针，保留当前实际pump/回收helper/method，仅加调度屏障；晚取消/早入核两态均真停，宽限尽具名PumpStuck且保留归属，释放后重试真停。见[pump-replay-proof](pump-replay-proof.json)；client17包测含残留/panic控制通过。当前旧泵例已闭，不将新service reader问题混作旧泵修复失败。
- T05-R2-04：两条当前option gate/lying-relay负控通过，协议20通过；旧版本字段忽略已有正面修复。UI工具当前不能重建，所以真实生成报告/各profile正控**不能凭旧exe补绿**；R2-02在代码层看到真实落点锚定修复，但本冻结整体仍须修01后重验。
- T05-R2-05：runtime三个指定目标4测试通过，旧trait签名已同步；compat仍被01的UI工具编译缺口挡住，不宣称所有旧消费无回归。
- T05-R2-06：本轮全部30文件中的改动Rust使用同样rustfmt参数检查 **通过**。
- **74个正向/既有定向用例通过**：client17 + execution生命周期23 + protocol20 + 独立execution consumer5 + runtime seam/service4 + option负控2 + 同源真实CLI driver3。异常watch宿主的假绿不计通过项；额外泵调度3态及service寿命/JSON反例单列，不拿总数代替AC。
- 当前CLI单独构建、Linux client检查、execution独立消费通过；完整UI工具构建失败。未跑整档cargo t/tv/tt/tb/tu/taa/tf；未跑尚未就绪的完整发行重面，不称完整UI/真实Rust服务本轮通过。既有测试daemon文件锁未被强杀，环境问题与01类型错误分开。

## AC / SD / 后续

| 验收 | 结果 | 说明 |
|---|---|---|
| AC01 | partial/fail | CLI基座/单源旧证据保持；01真实producer编译失败，03旧平台命令回退 |
| AC02 | fail | 02 service-owned reader退出；04验收有界性；旧泵与设施控制通过 |
| AC03 | fail/incomplete | 01真实工具构建；07完整profile/安装/首次及重定位矩阵未交齐 |
| AC04 | fail | 04/05/06/07组合、JSON、fresh身份与实际工具证据不齐 |
| AC05/T09及当前T13 | needs_fix | 不是整B/整Plan pass；既往A/T03/T04历史结论不撤销 |

SD五行候选与R2同hash。handoff虽增列protocol/UI-tools目标，但主Plan的spec-impact元数据仍未同步这些实际修改目标，正文也需描述正确的可交付规则，不能把不实“有界join/完整profile”沉淀为规范。T08修齐候选与来源，review后merge再受管发布。

下一步：work同一权威树修齐01..07，保留已通过旧修复与合法控制、补定向区分性负控，提交后整体受影响复审。最终review真正pass后，merge-owned冻结裸cargo t及实际触面档、发行consumer与Spec-ledger收口仍pending；tf仅主检出到期批量。当前不合入/归档/清树。
