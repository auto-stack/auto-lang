# PLAN-811 独立复审 R1

2026-10-19（Asia/Shanghai） · stage: review · review_scope: final · **outcome: needs_fix**

本轮先做就绪核查，再针对资源关闭进行独立重放。交接已承认 work-owned V-05 未执行，因此不继续把本轮扩成完整最终验收，也不授予 reviewed。下列实测缺陷与必需验收缺口均属于 revision 1 的原合同；登记不批准延期或缩小范围。

## 1. 受审基线与独立性

- Plan：主检出 `docs/plans/811-media-host-runtime-extraction.md`，revision **1**。入场原件为 [plan-input.md](plan-input.md)，SHA-256 `d4e36ba26c6c6678c526ba38f241408df7359139f1a5883a84b8433ec36a9e4a`。
- 实施树：`D:/autostack/.wt/lang-811/auto-lang`，branch `codex/plan-811-dev`。
- reviewed_commit：`0f2ab56a991e5c1ca7ac599ba2ca0bd68bf4f488`；base_commit：`ab08955c241f6a824091c53d3b02894ca17e193b`；22 个差异文件、10 个实施提交。入场和出场均核对原树，不创建复审树。
- 依赖：同组 auto-down detached `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，入场 clean。
- reviewer_identity：当前 Codex 用户请求的复审上下文；本会话没有实施这些代码。可见接口不提供具体模型/会话 ID，未编造身份，也未创建其他 agent。
- [baseline.json](baseline.json) 冻结实施文件、Cargo.lock、测试配置、技能、权威 Spec、Plan/交接/合同和依赖。主检出已有其他任务的文档 WIP，本轮仅增加 811 复审附件并修改 811 计划与债务簿相关条目。
- 权威 Spec 采用主检出版本：vm/overview 与 ui/overview 已有 807/808 的并行沉淀，和实施树版本 hash 不同；事件、Man、发行合同相同。目标 `docs/specs/auto-media-core/project.md` 尚不存在，作为新增目标正确。GOAL-003/007 在 `docs/specs/goals.md` 中存在。

## 2. Findings 与可直接重放的证据

### F811-R1-01 · P1 · shutdown/drop 未管理扫描与打开线程

**AC-03；T-01/T-03/T-07/T-08；原合同生命周期缺口。**

`crates/auto-media-core/src/image_pipeline.rs:1739–1755` 的 `scan_directory` 和 `:1782–1787` 的 `open_session` 均 `spawn(...).ok()` 后丢弃 JoinHandle。闭包持有 `Arc<RuntimeInner>`；循环没有 closing/cancel 检查。`:1682–1687` 的 shutdown 只关闭池中的两个 decode worker 和一个 resize worker，然后关闭 registry、清空 session，不能保证这些生产者退出。Drop 调同一函数，仍有该缺口。

独立生产入口探针建 4096 个普通空文件，开始 `scan_directory`，调用 shutdown，确认池内 `worker_count=0` 后 Drop runtime，等待 200ms。该进程未启动默认服务，也未并行运行其他媒体任务。三次结果：

| 重放 | shutdown 返回时媒体代次 | Drop 后 200ms 代次 | 关闭后生产者继续活动 |
|---|---:|---:|---|
| 1 | 21 | 8203 | true |
| 2 | 8209 | 16395 | true |
| 3 | 16403 | 24586 | true |

这证明 shutdown/drop 返回后，原 runtime 的扫描仍创建票据并修改 registry。扫描最终收敛；本轮不据此声称永久线程泄漏或任意 RSS 超限。结合源中 Arc 所有权，扫描完成前 RuntimeInner 及其持有资源仍存活。会话打开线程有同样的未跟踪所有权路径，本轮未为其另作受控暂停实测。

修复要求：runtime 统一拥有 scan/open/decode/resize 的退出和准入，关闭时先阻止新生产者、让在途生产者受控退出并等待其回收，再完成资源关闭；避免持锁 join 或自身等待。默认共享池仍不能因 VM Drop 被关闭。补 scan/open 受控暂停与 Drop/shutdown 退出、弱引用/文件/bytes 释放证据。

**现有 oracle 的覆盖缺口**：`:2466–2501` 的“满队列截止”测试提交空字节，decode 在 metadata 失败，不能证实 resize channel 已满；`accepted > 0` 也不能证实 ingress 满。截止断言写在同步 shutdown 之后，若 shutdown 卡死，不会自行触发清理。close/populate 测试依靠立即 close 和睡眠，没有控制并发窗口或观察生产者退出。需按 V-03 使用有效图像、可控阶段和真正的外部截止/清理。

### F811-R1-02 · P2 · 关闭没有封住新票据发布及会话准入

**AC-02/03；T-01/T-03/T-07/T-08；新增 owned runtime 未封闭旧 registry 行为。**

Registry `shutdown` 设置 `shutting_down`，但 `queue(:270)`、`transition` 和 `publish_ready(:358)` 不检查它。runtime 的 `open_session(:1760)` 也不检查关闭。现有关闭测试只验证 shutdown 前已存在的票据，被置为 Stale 后自然拒绝发布；不能证明整个关闭不变量。

独立公开 API 重放：

```text
ADMISSION shutdown=true old_publish_rejected=true fresh_pending=1
fresh_transitions_ok=true fresh_publish_ok=true fresh_ready=true
new_session={"session":"image-session-1","selected":0,"count":0,"generation":1,"name":"","uri":""}
```

旧票据拒绝是通过的正控；相同已关闭实例经公开 registry 新建票据后，Reading→Decoding→Transforming→publish_ready 全部成功、lookup=Ready。`open_session` 又往已清空的表插入有效会话并派生线程。此处不是说旧全局 registry 的迁移前行为出现回归；问题是新 owned runtime 以它实现关闭合同，却没有覆盖这些准入/发布路径。

修复要求：关闭锁存与 queue、session/scan 准入、状态更新及发布共用一致的同步规则，保持原公开签名的兼容或明确内部拒绝表示。覆盖 shutdown-vs-submit/open 的代表性交错、新/旧票据、重复关闭与独立 runtime 正控，不能仅给 `worker_pool.submit` 加门后宣称全部服务关闭。

### F811-R1-03 · P2 · 必需真实消费者验收被移交给 R04

**AC-04/05；T-06/T-07/T-08；work-owned 必需 oracle 未完成。**

[handoff-input.md](handoff-input.md) §3 明确记 V-05 pending，§7 又把浏览器实机走查交给 R04；原 Plan §6/AC-04/T-06/T-07 要求其在本 Plan work 阶段完成。没有批准该范围变更。

新增 `plan811_media_consumers.rs` 的前两测试直接调用 core，第三项断言源字符串；它没有生成后端 TCP、真实部分文件流或渲染拍/事件断言。core 的 `media_surface_status` 单测验证状态投影，不能证明 loaded 晚于实际渲染、单发、参数投影、换源退订、A→B→A、卸载和空闲完成唤醒。back_proxy TCP 与默认 api_gen 字符串测试也不能替代这些 oracle。

已有 792 收据实际存在，已复制为 [work-792-receipt.json](work-792-receipt.json)，25 项全绿；它记录的 app 是 `uiminimal`，证明该发行工程 fresh/locked/重定位及负控，不证明 031 媒体消费和事件。本轮未重新跑该 driver，也未把旧收据冒记为本轮实跑或覆盖所有媒体生成路径。Man wire 档基线编译红的归因本轮未重放；即使归因成立，所需生产 oracle 仍 pending。

修复要求：在原树补 VM/Iced、Vue split、Rust merged 的真实媒体运行及 V-05 事件断言、生成媒体 HTTP GET/HEAD/ETag/错误/非媒体路由与实际 Range 流输出，保存绑定提交/输入/原始结果的收据。可使用隔离的真实生产 harness 避开无关基线编译红；不要求代修全部无关测试，也不能用 mock 或字符串检查替代。环境缺失应保持 executing 并具名记录前置条件。

### F811-R1-04 · P3 · 新建代码与门面的格式检查未通过

**AC-06；T-07/T-08。**

只读执行 rustfmt `--check --edition 2021 --config skip_children=true`，检查八个新建/门面文件，exit **1**；[format.log.txt](format.log.txt) 给出六个文件的差异：core image_pipeline/media_service、plan811 VM 测试、Man consumer 测试、独立 consumer 和旧 image_pipeline 门面。例：core `:1697` 新方法签名、core media_service `:522`、VM 测试 `:300`、门面 `:17` 的重导出排序。均可定位到本 Plan 新增/改写内容。

修复后对受影响源码做 scoped format 检查。本轮未运行全 workspace format，未把其他文件的预存格式问题归到本 Plan。

## 3. 本轮实际执行与限制

重放入口（在原实施树执行；构建和探针输出位于其 ignored target 内）：

```text
python D:/autostack/auto-lang/docs/plans/attachments/811/review-r1/replay.py D:/autostack/.wt/lang-811/auto-lang
```

| 检查 | 结果与原始证据 |
|---|---|
| `cargo build --offline -p auto-media-core --message-format=json` | pass，基于原树构建；[build.stdout.txt](build.stdout.txt) 保留精确 rlib 来源。runner 使用 cargo 返回的 artifact，不选任意缓存库 |
| 独立公开消费者探针 | exit 0；关闭缺陷如上，原始 [probe.stdout.txt](probe.stdout.txt) / [probe.stderr.txt](probe.stderr.txt)，源 [probe.rs](probe.rs)。exit 0 表示反例执行完成，不能解释为产品验收 pass |
| 真 PNG decode 与 Weak 控制 | decode Ready，shutdown 后 worker_count=0；decoded Weak 在 shutdown 后仍活、在普通 runtime Drop 后失效。该正控证明普通 Drop 能释放缓存；本轮不单独把 shutdown 保留缓存认定为额外缺陷 |
| `cargo test --offline -p auto-media-core --lib` | **30/30 pass，0 ignored**，2.02s；[core-tests.stdout.txt](core-tests.stdout.txt)。build/test 无 core warning；这组全绿不能抵消独立反例 |
| scoped rustfmt | fail；上述八文件检查中六文件有差异 |
| committed diff whitespace | `git diff --check ab08955c2 HEAD` pass |
| VM 7、Man 3、777、792、其他媒体族 | 源与交接/现有收据核对，未重跑、不授信为本轮最终验收 pass；先修就绪与生命周期缺口，再作完整复审 |

第一次探针试用 `stats().queued` 观察扫描进度，但该字段是瞬时 Queued 状态数，不能计累计任务；清理条件因此超时。这轮不可用指标的日志保存在 `probe-initial.*.txt`，未据其作资源缺陷结论。修正为公开媒体代次，三次独立 runtime 都观察到 Drop 后变更，最终 runner 和清理均 exit 0。

本轮不运行整档 t/tv/tu/tt/tb/taa、tf 或 docs_gen。没有实施修复、规范/ledger 发布、合入、归档或 worktree 清理。

## 4. AC → task → artifact → evidence

| AC | 本轮结论 | task / artifact / evidence |
|---|---|---|
| AC-01 | partial | T-02/04/05：单源 core、薄门面、VM/Man/renderer 直引已检查；独立 core 编译/30测过。17 shim/两VM/feature 关闭及完整消费路径本轮未全部重放 |
| AC-02 | fail | T-01/03/07：F02 关闭后新 session/票据仍准入；其他 cap/TTL/引用矩阵需修复后完整复审 |
| AC-03 | fail | T-01/03/07：F01/F02；数据/Range/codec 基础单测通过，受控满 resize/在途工作/scan/open/引用回收矩阵仍需补足 |
| AC-04 | fail | T-06/07：F03；真实三形态/渲染后事件及生成媒体 HTTP/流输出证据未完成 |
| AC-05 | partial | T-08：结构与发行接线存在，旧 792 收据已保存；本轮未完成 fresh/locked 及所有负控独立复验，SD 也未最终通过 |
| AC-06 | fail | T-09：F04 与前三项阻止最终 pass；普通 core 编译无 warning，不代表 auto-lang/Man 完整 health 已审结 |
| AC-07 | pending（merge-owned） | T-10/11：最终 review pass 后，冻结提交 t+tv、必要受影响面/合流验收、知识沉淀/归档/guard 清理 |

## 5. Spec delta 审查

已冻结 Plan 表与 handoff SD 草案：[spec-delta-input.md](spec-delta-input.md)，SHA-256 `f4a8d43e4a158fb929561c58266270bbcd1e725eb568ce68e9f07b278e88d597`。所有目标均为有效仓内路径；五个 supersedes 目标及一个 new 目标、GOAL-003/007 保留，未发布 canonical 或 ledger。

| SD | 本轮结果与 work 后续 |
|---|---|
| SD-01 | fail：新 core 归属描述成立，但“shutdown 无复活/满队列有界收敛”与 F01/F02/现有测试覆盖不符；实现和 oracle 完成后再冻结准确文本 |
| SD-02 | partial：17项/native ABI/stub/feature 口径未见本轮直接改号；完整运行矩阵待最终复审。明确 ticket handle 表仍属 VM shim adapter |
| SD-03 | partial：媒体迁出与非媒体剩余 owner 口径合理；最终基于主检出当前 807/808 规范调和，不能恢复旧树对 terminal/diff 的历史未迁描述 |
| SD-04 | partial：canonical 渲染后单发/投影/换源/50ms 原则必须保持，真实证据未完成，不能仅依据 registry Ready 确认 |
| SD-05 | partial：Man 发射核心路径及依赖接线已检查；真实媒体产物/HTTP/merged 验收待 F03，不应把直接 core 测试表述为生成媒体实跑 |
| SD-06 | partial：新源闭包已登记且有 uiminimal 792 旧收据；媒体消费者证据及最终并集合流后冻结输入仍需覆盖 |

本轮没有改变语义合同，`plan_revision` 仍为 1。对 Spec 的不实完成表述必须随 work 修正，不能通过降低 AC 消除 findings。

## 6. 状态与下一步

Plan → **executing，current_step=3 / total_steps=11**。保留已完成 T-02（抽取骨架/类型）、T-04（文件服务规则迁移）、T-05（VM 接线）；重开 T-01（owner/oracle 对账）、T-03（生命周期）、T-06（真实消费者）、T-07（验收）、T-08（SD/交接）。T-09 仍未完成，T-10/11 保持 merge-owned pending。

next：在同一原树按 work 修复 F01/F02、补强 V-02/03/05、清理 F04 并校正 SD/交接；提交后带 finding closure 表送审。未来 merge 门为最终冻结提交上的裸 cargo t + cargo tv；本 diff 未改 ui_gen/trans，因此当前不触发 tu/tt，若修复触面变化须重算。未通过最终审查及这些门之前，不可合入或宣称交付完成。
