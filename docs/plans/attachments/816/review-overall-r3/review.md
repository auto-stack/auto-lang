# PLAN-816 整体 B-R3 受影响复审

**结论：needs_fix，原四项 P2 中两项 CLOSED、两项 PARTIAL/OPEN；没有新增产品 finding。**

- F-816-B-R1-02（reader 错误聚合）CLOSED。
- F-816-B-R2-01（Unix Ctrl+C 路由）CLOSED，含实际信号送达与超窗强杀对照。
- F-816-B-R1-04（driver 截止/收尾）PARTIAL/OPEN：原低频噪声超时改善，但 caller 返回后自有后代/reader 未停、回收未完成仍报成功；WatchGuard 的普通 join 仍可拖过总预算。
- F-816-B-R1-07（独立发行闭包）PARTIAL/OPEN：邻接工具/脚本/升级拒绝有实质进展，但双二进制安装布局的 Rust service 仍依赖实施树源码，不能算完整 CLI/service 安装闭包已交付。

基线：Plan r2，受审 `3068be74371628132b158219fab19b0ac49c283e`，差分基线 `bd50b0abaf61077e2f413d39b93766c8d9fb7ae6`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。唯一实施树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`。实施前后 clean、未发生输入漂移。当前聊天独立核对实施交接，没有实施修复、子 agent、第二 checkout；模型/会话 ID 未暴露，不伪造身份。review_scope=整体受影响复审，不能因部分修复通过而授整 B/整 Plan pass。

冻结 [Plan](plan-frozen.md)、[handoff](work-handoff.md)、[安装清单](work-install-manifest.json.txt)、[规范增量](spec-delta-reviewed.md)、[输入指纹](inputs.json)、[最终状态](final-state.json)。历史 B-R1-01/03/05/06 及 M816-A/T03/T04 已过范围保持。

## 两项闭合

### F-816-B-R1-02 / P2：CLOSED

`compose_failure` 在 READY 前后失败出口保留主退出分类，并追加 cancel/reader 已发生的错误，修复原匹配覆盖与 `let _` 丢弃。

从当前源重建原宿主、同一真实 CLI→generate_api→owned cargo 路径重放：[adjacent.py reader-error](adjacent.py)、[当前三态结果](reader-error-results.json)。READY 前非法 UTF-8、READY 后非法 UTF-8+exit0、READY 后非法 UTF-8+exit7 均 exit1 且具名 `backend stdout reader: stream did not contain valid UTF-8`；exit7 同时保留 `backend exited with 7`。无流错 clean/owned-holder/bad-budget 三态均 exit0、无悬挂、持管道后代心跳停止，非法预算诊断保持，见 [service-results.json.txt](service-results.json.txt)。未把原悬挂重报为仍存在，也未以反例模拟宿主证明真实 HTTP 正控。

### F-816-B-R2-01 / P2：CLOSED

`OwnedProcess::interrupt()` 原生 Unix 向 owned 进程组发送 SIGINT，service 观察终止意图后先转发、再排空、超窗后 cancel。树归属没有改为 DirectChildOnly。

[原生 Linux 同源 service tail 重放](unix-signal-result.json)：实际 ProcessTree 收到对父会话组的 SIGINT 后，子 handler 写 graceful marker、正常退出，parent `stopped cleanly`/Ok0，约 **0.264s**；继承父组的合法送达对照约0.032s。探针使用当前 `serve_rust` 从 ExecutionSpec 到 return 的原样 tail 及实际 facility，只替换生成/config 前置为常量，子进程为安装真正信号 handler 的受控宿主，未声称真实 HTTP 在途 drain 全覆盖。

[额外强制兜底对照](unix-signal-fallback-result.json)：同一宿主收到 SIGINT 写 marker 后故意不退出，parent 等约 **15.249s** 后 owned 树强杀收尾，`stopped (terminate)`/Ok0。该结果与正常优雅退出分开，不把强杀称为优雅关闭。可重放 [unix-signal.py](unix-signal.py)、[signal-fallback.py](signal-fallback.py)。

## F-816-B-R1-04 / P2：原样例改善，资源与结果不变量仍未闭

影响 AC-02/04、V-02/V-04、T07 测试基础设施。依据是原 finding 明确要求的有界回收与 owned 子进程/reader 真停，**不是新增测试负载承诺，也不是宣称真实生产 watch 的产物生成失败**。

已闭部分：从当前新测试全文重建旧字节故障宿主，[watch-oracle-proof.json](watch-oracle-proof.json) 记录当前测试 SHA256 `1f3c6bf8…` 与私有副本一致；真实输入改变、SFC 仍旧，测试 exit101。正常真实 CLI watch 过；原每100ms噪声、22秒才就绪样例现在约20.09s报红，见 [adjacent-watch-results.json](adjacent-watch-results.json)。其中 noisy case 的旧 audit 文件来自前一正控，不能当作该次已生成证据；该 case 只采用真实退出码/耗时与失败日志。

残项 A：`crates/auto/tests/plan816_cli_driver.rs:194–204` 的 `join_bounded` 超时返回 None，会直接丢弃仍活的 JoinHandle；`:232–234` 把结果转成空串，再沿根退出码返回成功。`:221–224` 的进程超时出口也直接丢掉两个排水线程归属。caller 返回不是清理完成，且会丢弃已收到的 stdout。

[新同源资源重放](reader-lifetime-result.json)、[脚本与受控宿主](reader-lifetime.py)：保留当前整个 driver 字节前缀，只另附测试调用它的实际 run_bounded；故障根 exit0，自己的后代持 stdout/stderr 3秒并打心跳。limit=200ms，实际函数约 **274ms 返回 code0、stdout=""、stderr=""**；返回时心跳 **4**，300ms 后 **10**。子进程仍活是直接观测；两个阻塞 read_to_string 线程在写端仍持有时不能完成，是由实际管道状态与源码共同确定，不冒报独立 OS 线程计数。新断言 exit101。旧仅检验“函数是否预算内返回”的控制现在绿，恰好不能证明其资源清理或超时分类。

残项 B：`WatchGuard::release/Drop` 的 `:78/:90` 仍是普通阻塞 join，只有根进程 kill/wait，没有后代树归属或可停止读。相同故障 CLI 用23秒自带寿命的持管道后代，当前**逐字不改的 watch 测试**20秒触发预期 panic 后，守卫又等到 **23.02秒** 才真正结束。固定寿命保证探针自行退出，没有清理共享进程。该状态仍违反本轮声明的“等待/断言/收尾共用总20秒”。

闭合要求：保留线程/树归属直至确认停止；不能将回收未完映射为空串+成功。timer、停止、reap、输出/错误传播需共同覆盖 timeout、根先退出、panic、持管道后代与合法正常退出；用上述心跳/结果断言及原内容正反控验收。当前 left(total) 在预算尽后仍返回1ms，循环还需核对绝对到期判断，不能只依赖逐次 recv_timeout 的名称。新增 `mut guard` 触发 unused_mut，作为同一修复收尾健康项处理，不再新开 ID；旧 wait_bounded dead_code 诊断为历史位置。

## F-816-B-R1-07 / P2：安装矩阵部分通过，但仍依赖实施源树

依据：Plan r2 §1/§5.3/§5.4/§5.5、AC03/04、T05/T07/T08：默认完整 profile 与安装清单包含实际 CLI/service/protocol/tool+资源/锁闭包，首次启动/重定位/升级失败由 work 验收。当前未有批准把资源/完整服务交付延期至 R07 的合同修订。

本轮按工作清单，只把当前两个二进制复制到随机新安装目录、项目置于另一个外部临时目录，清除 AUTO_UI_TOOL_BIN/CARGO_MANIFEST_DIR/AUTO_LANG_ROOT/AUTO_LANG_CRATE/AUTO_DELIVERY_SOURCE_ROOT 等开发定位 env。独立 [install-replay.py](install-replay.py)、[完整命令/环境/生成清单结果](install-replay-result.json)：

- 裸脚本 return42、Vue gen 均 exit0；邻接发现真实工具生成17文件。
- 原升级选项拒绝的有效 Rust profile，实际 CLI→旧 v1 codec 受控宿主：exit1、`request NOT sent`、无工作请求 marker、无指定输出目录。该宿主是版本/协商负控，不称真实已发布旧二进制的多版本安装升级。
- 隐藏本次自己复制的邻接工具后：具名 NotFound / exit1；没有隐式源码回落。
- **安装版 CLI 的真实 Rust service 生成链**创建的 workspace Cargo.toml 仍含：`auto-lang = { path = "D:/autostack/.wt/lang-816/auto-lang/crates/auto-lang", ... }`。解析后的框架依赖确实在实施树，`inside_install=false / inside_plan_checkout=true`，生成清单原文已存结果文件。

最后一项只用受控 cargo 截获生成后的调用，以便不重复冷编完整 Rust 服务；生成本身为真实代码路径。**没有隐藏/改权限/搬走共享源树，也不声称本机已经实测源树物理消失后的构建失败。** 已确证的是该安装形态的必要生成框架源依赖不在交付根，仍从编译期路径借用实施 checkout；在没有该路径的机器上无法解析，是由生成清单直接得出的依赖后果。

对应 `auto-api-gen/src/rust_workspace.rs` 的编译期 source fallback 仍要求源码存在；当前 manifest 只有二进制/哈希/锁引用、没有供应该运行期生成框架闭包。handoff §3 又把“正式独立安装包资源布局”宣布为 R07 非本轮，§6 仍留“未交付/需用户裁定”的旧段。二进制邻接发现不是完整 CLI/service 源资源闭包证明，既有 UI-app 792 relocatable_delivery 也不能替代新的 CLI/service 安装路径。不是要求无关重做792发布器，而是兑现本 Plan 自己的交付边界。

work 的 fresh83.4秒收据只有摘要/产物 hash，缺少该次 source revision、展开命令、清洁源构造/所含资源与当前安装根的映射；不能用一次仓库源码 fresh build 为两文件安装形态补闭。Core 负控收据也未绑定 Core 二进制 hash/实际调用参数。当前因上述真实闭包残项已足以判定不就绪，没有再跑一遍完整冷构建来掩盖安装差异。

闭合要求：给出能兑现原支持面的实际安装/开发 bootstrap 布局及源/资源/锁身份；从该交付输入实际生成并构建/消费代表服务，核验路径不借用实施树，并保留已过的邻接发现、普通脚本、缺件与旧 codec 拒绝对照。Core/full 的证据需有真实 profile/产物绑定，fresh/locked 验的是交付输入。不得只把本段残项重新命名成“R07 正式打包”或以 work 注记自行缩约。

## 验证汇总、知识增量与路由

当前源 locked CLI/tool/execution bins 构建成功；实际 Cargo package/target/profile/executable/hash 见 [artifact-bindings.json](artifact-bindings.json)。auto=`3d45e82a…`、tool=`4a1af25d…`。不要求跨不同构建上下文确定性同 hash，但每份证据要绑定实际产物。安装重放/真实 CLI driver 均采用本轮二进制。

本轮执行 **85 项定向测试通过**：execution lib11+生命周期23、client17、真实 tool consumer30、原样真实 CLI driver4。并有上述流错/owned-holder/预算/JSON/Unix信号/安装独立控制；这些控制不混入85测试数。777 strict仍 pass/0违规/3grandfathered/141边，client 依赖围栏过、四变更文件 rustfmt/diff check过。driver 新增 unused_mut 未冒称零警告；其余编译缓存既有 warnings 留日志，未做无关修理。

变更仅四个 Rust 源，Cargo.lock/清单/777规则配置、协议/发现/producer源与 auto-down 不变。R2已过的协议20、旧seam/类型身份及其对应未变路径、777 selftest15、依赖围栏等具名复用；execution/lib 已变，故生命周期/client/真实tool消费及 signal 均重新执行，不用旧计数冒充本轮执行。

规范增量 SD01..05 与 protocol/tools 候选及 spec-impact 元数据仍只冻结审查；主源 Specs/ledger 不写入。SD01/04 的 interrupt→排空→强杀语义可按实测补充，SD05/完整安装与所有资源真停不能在未闭时沉淀为已交付。handoff 中旧14/26项、777旧锚点、hash/未竟清单应统一为当前交付事实，而非互相冲突的历史片段。

AC01/03/04 partial；AC02仍受04资源/分类残项影响；AC05不通过；AC06未来merge-owned；历史AC07..09/M816-A保持。Plan executing6/14，T05..T09/当前T13不勾。本轮未整档、未改实施、未提交/合入/归档/清树。

下一步同树 work 完整修齐原04与07两项，再提交新 SHA 做受影响复审；已闭02和Unix信号范围保留。最终 pass 后才运行 merge-owned 冻结裸 cargo t 与实际触面档/consumer/workspace 门，tf仅主检出到期批量；Spec-ledger、合入归档与guard清理仍pending。记债不授权延期。
