# PLAN-816 / T-05 受影响复审 R2

2026-10-23；**needs_fix，5 项 P2 + 1 项 P3**。原 R1 的9条断言本轮均绿，但全不变量尚未闭合，且出现新的编译/报告/兼容回归。T-05 与当前复用 T-13 保持未完成，executing **6/14**；T-06 不越门。A/T03/T04 历史阶段结论保留，不据此声称当前受影响消费方全部无回归。

## 基线与方法

- phase review，独立于实施会话的本聊天 Codex `/root`；无新 agent，无已暴露模型/会话 ID。未修改产品实现、未提交或合入。
- 唯一树 `D:/autostack/.wt/lang-816/auto-lang`，分支 `codex/plan-816-dev`；受审 **`3c09bdf5b59d3f1aa4103b50c433a1bad3a327d8`**，repair base **`b6148dd90d716038395ab32d5ce8d1d926267dfe`**，16文件 +1034/-230；前后 clean、HEAD不变。auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- Plan r2、启动门与已发布814/815祖先保持；[当前输入/源与产物](inputs.json)、[冻结SD](spec-delta-frozen.md)、[最终身份/证据哈希](final-state.json)。不沿用未绑定新源的工具二进制：本轮重建 default CLI 与真实 tool 后取证。
- [原反例源码](replay.py)与[原客户端探针](client-probe.rs)从R1逐字复制并验证哈希；输出独立保存在R2，不覆盖旧收据。R1结果文件已被工作回放改写，不能再作旧红色原始记录；冻结源码/断言未变。排除的窄工具与初版代理设置错误见[说明](excluded-probes.md)，不算产品finding。

## 原 findings 的闭合边界

| R1项 | 旧例 | 完整边界 |
|---|---|---|
| 01 输出/交付 | Rust显式输出目录和delivery-manifest现在真实存在 | 旧落点缺陷已修；新输出的报告不实、旧工具静默丢选项仍未闭（R2-02/04） |
| 02 InputAck | 合法InputAck→RunOk被正确驻留，shutdown Success | 原反例关闭；控制请求保留自身回执终态，源码路径及原包测通过 |
| 03 输入泵 | steady held stdin正常回收，4→4 | **仍OPEN**：先取消后进入读的窗口导致成功返回且残留；Unix新增路径还不能编译（R2-01/03） |
| 04 有效后端 | pac=rust、CLI -r vue --gen-only现在产Vue | 产品旧例关闭；新seam签名破坏三个旧消费测试目标（R2-05），不称旧控制全过 |
| 05 strict/argv | 当前新tool的strict拒绝、lenient正控都通过；argv源码链已贯穿 | 旧工具语义不相容却接纳仍未闭（R2-04）；本轮未冒称实际各UI后端argv矩阵已验证 |
| 06 ui-iced cfg | 改为ui-extra；Core检查/CLI测试不再出现该新增告警 | 原例关闭；本轮新改测试的格式失败另记R2-06 |

## 本轮 findings

### F-816-T05-R2-01 · P2 · Unix输入泵引用未声明的libc，客户端无法编译

既有规则：AC-02/04、已发布客户端双平台合同及此前Linux健康不可回退。`crates/auto-tool-client/src/lib.rs:410-425` 新增 `libc::pollfd/POLLIN/poll/read`，但 Cargo.toml 没有libc依赖。

实际执行 `cargo check -p auto-tool-client --target x86_64-unknown-linux-gnu`，exit **101**，这四处均 **E0433**，见 [linux-check.log.txt](linux-check.log.txt)。目标标准库已安装，失败发生在本包名字解析，不是缺链接器、外部网络或运行环境。Windows15包测不覆盖此cfg分支。

闭合：使Unix路径有合法的依赖/实现，再跑上述真实target check与适用Linux泵控制；不要只记录“编译面已核”或拿Windows回收外推Unix。若引入新依赖，按原777规则登记/重采样。该项为本轮修复新引入，非旧三条execution warning。

### F-816-T05-R2-02 · P2 · 显式输出已写出，生成报告却指向不存在的默认根

既有规则：AC-01/04，815已发布GenerationReport应报告**实际产物根和实际文件数**；R1-01的闭合也不能只看目录存在。

[adjacent.py](adjacent.py) `new-output`：合法Rust工程，经真实CLI `gen -o <absolute>`，exit0，指定目录真实有 **2文件**。CLI从ToolClient报告打印的却是 **generated 0 files under <project>/rust-workspace/<member>**；该默认目录并未生成。原9条脚本只断言指定目录存在，因此仍绿。

确证原因：`crates/auto-ui-tools/src/tool_ops.rs:130-147` 将新选项传给生成器后，依然用 `profile_output_dirs(&root, profile)` 的固定默认目录收集/计数。R1保留的旧聚合规则适用于旧默认布局，不能描述新增重定向后的真实结果；“honest_notes:canonical布局报告”不是豁免。

闭合：报告绑定实际本次生成落点并对真实文件作oracle，覆盖Rust/Jet/Ark/VSCode同族、默认布局与delivery正控；不用硬编码报告0或只修改显示文字。永久控制应同时核验实际输出、报告output_root、files_written以及错误根未被报告为成功产物。

### F-816-T05-R2-03 · P2 · 取消先于ReadFile时，回收超时静默丢失线程归属

既有规则：AC-02、已发布owned寿命合同、R1-03要求真停/有界错误传播，不能只看stop位或退出进程。

`crates/auto-tool-client/src/lib.rs:342-354` 在Wait返回到ReadFile之间仍可被停止；`:871-893` 只发一次CancelIoEx，等宽限后直接drop JoinHandle，返回`()`且pump槽已take。注释声称“宽限尽具名暴露”，实际没有错误出口；JoinHandle消失并不意味着线程停止。

[pump-race.py](pump-race.py)冻结**当前实际Windows pump模块和完整reclaim_pump方法**，仅在Wait ready与ReadFile之间加一个调度屏障；没有改停止/读取/回收算法。宽限250ms，原TEARDOWN_GRACE=3s，stdin由外部持续持有。两态同源控制：

- 已进入ReadFile再取消：`alive_on_return=false`，释放后300ms仍false。
- 在屏障处执行实际reclaim后才恢复读：`reclaim_returned_unit=true`、`pump_slot_empty=true`、`alive_on_return=true`，恢复执行300ms后 **仍true**，读已经进入但早前一次取消不再生效。

见[源码与执行证明](pump-race-proof.json)、[冻结探针源](pump-race-source.rs)。同方法哈希、去掉屏障后模块相等；这是合法调度窗口的确定性反例，不伪称未插桩CLI必然每次触发。

闭合：停止必须覆盖“将要进入的读”与“已挂起的读”，不能在失败时静默丢掉回收归属；线程未停/panic须按约定具名传播，并确认后续shutdown不会因槽空误以为已经处理。永久回归增加此早/晚取消时序，保留原steady held stdin和InputAck正控。只把宽限调大不能闭合该窗口。

### F-816-T05-R2-04 · P2 · 新执行选项未协商，旧v1工具可静默忽略却被接纳

既有规则：AC-03/04、Plan §5.3，缺版本/能力在执行前具名失败，旧flags/默认保持。可选字段的serde解码兼容不等于旧生产者支持新语义。

`crates/auto-tool-protocol/src/message.rs:58-78` 添加output/project/delivery_root/strict和run args，却保持协议版本1、tool_version0.1.0和describe五操作能力集不变；CLI发现仍只要求原窄能力，无法识别这些必需执行选项是否实现。

[legacy-proxy.py](legacy-proxy.py)使用 **b6148dd90真实message.rs逐字冻结的serde codec**，原请求反序列化/重序列化后转发给当前真实tool；它是受控旧wire消费方，**不是旧producer二进制**。源码/哈希见[proof](legacy-proxy-proof.json)。旧与新describe完全相等；基础默认真实Vue生成控制 **1/1通过**，排除不工作的宿主。`adjacent-results.json.txt`：

- `new-output`写指定目录2文件；旧codec路径同CLI请求 **exit0、指定目录0文件**，转而在默认根生成2文件。
- `new-strict`对解析错误exit1；旧codec路径同请求 **exit0**，生成17文件。
- 旧codec确认打印 `LEGACY_CODEC_DROPPED_NEW_OPTIONS`；请求未经任何拒绝便产生副作用。固定源显示旧完整v1接收者具有同样的未知字段忽略语义，不是单纯伪造版本号。

闭合：让新消费者能在有副作用前确定工具实际支持这些语义（由owner选择明确版本/能力/身份策略）；保留适用旧基础请求的正控，并对升级前完整工具/冻结旧codec的新选项请求作具名拒绝负控。不能只在最新版工具上跑通，也不能把“可忽略新增字段”当成strict/output已被执行。

### F-816-T05-R2-05 · P2 · seam签名改变后，三个旧注入消费目标无法编译

既有规则：AC-01/04，R1-04有效请求修复应保持此前显式注入优先、默认兜底和链接compat后的显式胜出控制可用。

新增request参数的 `crates/auto-man-runtime/src/ui_ops.rs:24-28,100-113` 没有同步三个实际consumer：

- `plan816_t03_ui_inject_explicit_first`：5个 **E0050**；
- `plan816_t03_ui_default_fallback`：5个 **E0050**；
- `auto-man --test plan816_t03_ui_inject_compat`：5个 **E0050**。

见[t03-controls.log.txt](t03-controls.log.txt)、[compat-controls.log.txt](compat-controls.log.txt)。两条可重放命令为：`cargo test -p auto-man-runtime --test plan816_t03_ui_inject_explicit_first --test plan816_t03_ui_default_fallback --test plan816_t03_non_ui_service`；`cargo test -p auto-man --test plan816_t03_ui_inject_compat --test plan816_t03_create_jet_default --test plan816_t03_legacy_util_paths`。均exit101。118个runtime lib测或compat内部4测不能证明这些外部测试目标编译通过。

闭合：同步真实消费实现/调用点并保留原三向注入断言，必要时补有效请求传播观察；不得删测试、ignore或改变first-wins要求。另行隔离执行的非UI服务2、create_jet默认/显式2、旧util路径1及argv复位1均通过；因此不把编译失败夸称这些产品路径已经执行失败。

### F-816-T05-R2-06 · P3 · 本轮修改的两份测试未通过格式检查

必需健康收尾。`rustfmt --check --edition 2021 --config skip_children=true` 对改动文件检查exit1，新加的字段/闭合括号在 `crates/auto-ui-tools/tests/tool_process.rs:958-963` 及同族、`tests/lifecycle.rs` 缩进不正确。见[fmt-changed.log.txt](fmt-changed.log.txt)和本轮diff：这是新增行，不归因旧vue/examples格式噪声。修复后重新定向格式检查即可；本轮未修改这些文件。

## 验证边界与规范/路由

- [原9断言](replay-results.json.txt)：9/9，exit0；不降低/替换旧断言。它们只能证明原具体例与3个合法控制，不能识别新报告不实和早取消窗口。
- **110个不同现有定向测试通过**：CLI24 + client15 + protocol19 + consumer24 + tool_process22 + argv1 + 非UI服务2 + Jet/旧util3；另重复一个默认真实生成用例通过旧codec路径。**3个消费目标编译失败**。tool_process的 **9个ignored未执行**，不把22个选择面冒充完整B build/run/VM窗口验收。
- Core check通过，当前CLI新增cfg告警关闭，Core normal/build/dev树0禁项；default CLI/tool重建通过。未跑整档alias、未跑完整UI argv/跨平台运行矩阵。本轮Unix证据是实际Linux target类型检查，明确没有Linux运行通过。
- 新增永久pump回归确实验证steady held stdin线程回落、InputAck驻留，质量优于R1；但未覆盖reclaim宽限尽或取消早于内核读的时序。报告与旧版本字段忽略控制也缺少区分性断言；本轮给出独立输入和原始结果，不将ALL-PASS总数当闭合证据。
- SD五行hash与R1相同，拟受影响的工具执行/CLI/管理规则已审；本轮实际还修改了已发布protocol/UI-tools语义，T-08需要把`docs/specs/tool-protocol/project.md`、`docs/specs/auto-ui-tools/project.md`的执行选项协商与真实产物报告规则明确纳入增量/元数据，再调和当前canonical规则。旧附录“output_root即delivery通道”也不能沿用；未发布任何新Spec或修改live ledger。
- AC01当期partial/fail（有效请求旧例通过但consumer不能编译）；AC02 fail（R2-01/03）；AC03/04 partial/fail（R2-02/04/05）；当前T13 needs_fix。A历史发布不撤销，T03/T04历史pass保留，当前受影响证据必须在T05修复轮重建，不能声称全部无回归。
- 777仍按原T-07处理已知auto Cargo锚过期；本轮未刷新或宣称开工前134边绿色适用。T-06服务/watch、T-07完整安装/bootstrap/792/JSON矩阵及T-08终稿仍必做；最终review通过后merge-owned冻结裸cargo t和实际触面档、发行consumer门仍pending，tf仅主检出到期批量。

下一步：同树work修齐R2-01..06，补新执行选项报告/旧codec拒绝及早晚取消的永久定向正反控，恢复旧consumer编译与原断言；提交新SHA后再交T-13受影响复审。未改产品实现、未提交、未合入、未归档/清树。
