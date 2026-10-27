# PLAN-816 整体 B-R4 受影响复审

**needs_fix：04、07 原样例已改善，仍各有一项原合同内残项（2 P2，沿用原 ID）。此前已闭项未发现回归。**

受审 `bb6b91ca163b2fba4fe25ad50b9bb9a2d1ef2c3e`，差分基线 `3068be74371628132b158219fab19b0ac49c283e`；Plan r2，唯一实施树 `D:/autostack/.wt/lang-816/auto-lang`、分支 `codex/plan-816-dev`，auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。前后实现 clean、HEAD/配置/依赖无漂移。当前聊天独立验证实施交接；未创建 agent/第二 checkout，未实施修复；模型/会话标识未暴露。review_scope=整体受影响复审，非整 Plan pass。

冻结 [Plan](plan-frozen.md)、[handoff](work-handoff.md)、[安装清单](work-install-manifest.json.txt)、[SD](spec-delta-reviewed.md)、[输入指纹](inputs.json)，最终对账见 [final-state.json](final-state.json)。所有构建/测试在唯一实施树，报告在主检出；安装 SDK 是无 .git 的消费方测试输入，不是另一审查 checkout。

## 04：owned 后代反例已闭，但外部持管道时 reader 仍活

影响 AC02/04、V02/V04、T07。原规则是 owned 进程/reader 真停，不能把 caller 返回、kill 已发出或具名错误替代资源回收；不要求终止不归属本次树的进程。既有 A 阶段已核验的 non-owned pipe holder 场景，也不能在新的 CLI driver 排水实现中绕过。

本轮已闭范围：

- 原 R3 run_bounded 反例，**测试断言原文不变**，只为私有测试装配增加现在实际使用的 auto-tool-execution path 依赖。原样当前 driver 字节前缀核对相同；根退出0、owned 后代持管道3秒，当前返回 code=-1、timeout 分类，stdout 保留、心跳4→4，结果含 drains=[true,true]；函数约386ms，原断言绿。见 [reader-lifetime-result.json](reader-lifetime-result.json)。
- 同一方法学的23秒 owned stdout 后代，当前原样 watch 测试约20.07秒完成预期“未就绪”失败及清理，没有再等到23秒。work 的0.41秒是另一更早失败切片，不替代本次原输入时序。
- 从**新测试全文**重做旧字节负控，测试 SHA256 `7e24efb0…` 与私有副本相同，实际输入已变而旧 SFC 保留，正确 exit101；真实 CLI watch 正控通过。见 [watch-oracle-proof.json](watch-oracle-proof.json)、[positive-driver-results.json.txt](positive-driver-results.json.txt)。

**仍未闭的机制**：`plan816_cli_driver.rs:259–264` 在 JoinHandle 未完成时返回 false 并丢弃 handle；`:275–287` 报 drains=false 后返回，但 reader 没有停止机制。`:234–251` 的超时/等待失败出口也没有统一收割 drains。`WatchGuard:80–89` 同样在5秒后放弃仍活的线程。杀 owned 树只能关闭该树的写端，无法保证外部持有的凭证也关闭。

代表相邻反例见 [external-reader.py](external-reader.py)、[原始 OS 快照/结果](external-reader-result.json)：

1. 私有测试先创建**自己负责清理**的独立 receiver，5秒自带截止；owned fault root 通过真实 Windows DuplicateHandle 把 stdout 写端转给该 receiver，然后正常退出0。
2. 当前 driver 全文不改，调用实际 run_bounded(limit=300ms)。owned 树结束，函数约333ms返回 code=-1，保留 root 输出，错误明确记录 drains=[false,true]。
3. 返回后 OS Thread32 快照由基线5线程变6线程，新增 **37208** 在100ms后的第二快照仍存在。测试释放 receiver 的写端后，该线程才消失，回到原基线。不是仅凭总线程数猜测，也不是把独立 receiver 误称 owned 后代。
4. “owned reader 仍存在”断言 exit101。receiver 是探针自己的进程，显式清理；没有结束共享进程，没有修改生产函数、插入阻塞 hook，或要求误杀外部持有者。

因此本轮修好了 owned 后代真停和错误分类，**没有满足 reader 在返回前真停**。具名 drains=false 可以报告问题，但不能让丢弃活线程归属成为合格收尾；该 reader 若遇长寿命外部写端会持续存活。

闭合 oracle：保留原 owned 后代/正常输出/真实 CLI/内容负控；外部凭证控制必须保持 receiver 不受误杀，同时 owned reader 在返回前已停或由明确仍持有的归属完成清理，不能丢掉 JoinHandle。覆盖新 drain 实现的根先退、提前超时、panic/等待失败出口；无需新建执行设施，现有设施的可停止读与归属规则仍是基线。

## 07：SDK 服务路径已闭，完整安装的 delivery producer 仍借用实施树

影响 AC03/04、V03/V04、T05/T07/T08。依据是 Plan §5.3/5.4/5.5 与 contract-baseline §8.2 的默认完整 profile、792资源/锁及旧 gen/delivery 支持面。不是新增加 UI 能力或要求重写发布器。

**已独立验证的安装进展**：按当前清单，只复制双二进制、sdk/crates 跟踪文件全集、根 Cargo.toml/Cargo.lock 与声明的5根级输入；未添加额外 .cargo 配置、源码软链接或主检出搜索配置。5539个 SDK 输入均记录哈希。清除开发定位 env，项目位于独立外部目录。见 [sdk-install.py](sdk-install.py)、[精确输入/生成清单](sdk-setup.json)、[实际编译检查结果](sdk-install-result.json)。

当前安装 CLI 的 Rust service 真实生成链依赖已变为 `../../install/sdk/crates/auto-lang` 等同源 SDK 路径，旧绝对指向实施树的反例关闭。实际生成 workspace 的 `cargo check --offline` **exit0 / 71.51s**，随后同一输入 `cargo check --offline --locked` **exit0**。Cargo target 定位于实施树的私有缓存目录，仅承载编译产物，源输入来自安装 SDK；不把该输出缓存称为源码借用。生成阶段的 cargo 是受控宿主，只为检查生成落点，不冒充 HTTP 监听正控；实际编译检查另走真实 cargo。

**仍未闭的默认调用链**：新 SDK 邻接解析只加入 `auto-api-gen::rust_workspace`，而真实 UI tool 的 `auto-ui-tools/src/rust_ui.rs:4114–4137 delivery_producer_root()` 仍是 env→CWD仓库→编译期源码根，没有安装 SDK 的默认接线。

同一个已完整复制 SDK 的安装目录，真实 CLI→邻接真实 tool 执行 `auto gen --delivery-root <独立输出>`，见 [delivery-install.py](delivery-install.py)、[两向实际结果](delivery-install-result.json)：

| 输入 | 实际结果 |
|---|---|
| 默认安装启动，开发 env 全部清除 | exit0，但生成 delivery-manifest.producer.source_root=`D:/autostack/.wt/lang-816/auto-lang`，git_head=本轮实施提交。仍从编译期实施树拷贝闭包。 |
| 同样安装与项目，唯一差异：显式 AUTO_DELIVERY_SOURCE_ROOT=安装/sdk | exit0，producer.source_root=安装/sdk、git_head=null；同样 ui_gen_fingerprint 和根锁身份，闭包可从安装 SDK 生成。 |

完整 [默认 producer manifest](declared-install-default-delivery-manifest.json) 与 [安装 SDK 对照 manifest](explicit-installed-sdk-control-delivery-manifest.json) 已冻结。该对照证明 SDK 本体足以供应这条生产链，剩余问题是**默认安装接线遗漏了实际工具 producer 消费方**，不是“SDK缺一切”或某个不可用环境。

没有挪走/隐藏共享源树，不宣称本机已实测物理删仓后的失败。已确证默认安装仍读取实施树；没有该编译期路径的机器，当前解析序无法自行找到旁边 SDK，这是源码与产物共同确定的后果。不能把用户额外配置开发 env 当作本清单已承诺的默认完整安装首用。

闭合要求：让正常安装 producer 使用同一声明 SDK/资源/锁来源，开发显式覆盖保持；用默认安装与显式 SDK 合法对照核验真实 gen/delivery-manifest 的来源均不借实施树，并保留新 service SDK 生成/locked 检查及旧缺件/版本/协商拒绝。改动与证据应绑定新提交和真实双二进制；不是把资源完整交付再次延期到R07。

work 的 `r3fix-install-sdk-chain` 实际包含4编译错误、exit1，不把它称为受控 cargo 正控；后续 release build 摘要与旧 fresh/Core 收据亦不能替代本轮默认安装来源核验。本轮用可重放的准确 SDK 输入和真实 check/locked 检查验证已改善的服务子路径，未重复一遍245秒 release 冷编或要求新的性能承诺。

## 此前闭项与验证边界

| 已闭项/路径 | 当前结论及依据 |
|---|---|
| B-R1-01 fresh构建漏字段 | 当前源 locked CLI/tool/execution 三包 bins 构建通过，真实 tool consumer30与CLI driver4通过。 |
| B-R1-02 reader错误聚合/原service owned-holder | 当前真实CLI三流错组合都具名，exit7保留；clean/owned-holder/bad-budget均过，后代心跳停。 |
| B-R1-03 Unix后台serve flags | main、execution及配置未变，具名复用已过原生Linux实际spec→spawn_detached证据；非新UI API路径。 |
| B-R1-05 JSON | 当前 return42 与 new 整份stdout仍可解为单JSON；info缺pac.atom的旧无效成功样例未算正控。 |
| B-R1-06 777 | 当前 strict pass、0 violations、3 grandfathered、141边；输入清单/规则/锁未变。 |
| B-R2-01 Unix Ctrl+C | 本轮原生Linux同源service tail重新执行，实际owned组正常收到SIGINT并写graceful marker，约0.265s正常退出；原有强制兜底路径源/输入未变，复用R3故意不退15.249s收尾对照。 |
| T03/04、旧协议/泵/类型单源 | 相关源、依赖、测试配置未变，保留已过证据；consumer30再次覆盖describe/validate/gen、发现拒绝、选项协商、timeout/reader/孙进程等，不以历史总数替代受影响安装路径。 |

当前定向测试：真实 tool consumer **30/30**，原样真实 CLI driver **4/4**；另有上述共享反例/OS线程控制。`cargo test -p auto-api-gen rust_workspace --lib` 实际选择 **0**（56 filtered），明确不计测试通过数；SDK有效性由真实安装生成和真实 cargo check/locked 承担。未重复不变的R3 execution34/client17/protocol20等；源码差分只有 resolver16行及测试driver，输入等价记录在 inputs.json，复用不扩到新实现。

变更两文件 rustfmt、git diff --check、client依赖围栏过；新unused_mut已去除，driver只余原 wait_bounded dead_code。全链编译的其他既有warnings按原来源保留日志，不冒称全仓零警告。Cargo唯一 package/target/profile/二进制/哈希见 [artifact-bindings.json](artifact-bindings.json)，并与安装输入绑定。

SD01..05及protocol/tools候选只冻结，不改canonical或live ledger。新的SDK来源规则必须最终覆盖服务和tool producer；全部资源真停不能提前沉淀为“所有返回已回收”。handoff旧hash/14与26项/未竟段/fresh-Core旧来源应最终统一为真实当期事实，不能以旧 receipt 名称当新提交证据。

AC01/03/04仍partial、AC02仍有04 reader残项、AC05未通过、AC06未来merge-owned；A阶段AC07..09已发布范围保持。Plan executing6/14，T05..T09/当前T13不勾。下一步同树work修04 reader归属/停止与07工具producer默认SDK接线，提交新SHA受影响复审；原例已过的范围保持，不重做已正确部分。

本轮未整档、未修实施、未提交/合入/归档/清树。最终pass后merge-owned冻结裸cargo t及实际触面档/consumer-workspace、Spec-ledger与合入归档guard仍pending；本轮两项原合同残项不能后置为merge门或自行延期。
