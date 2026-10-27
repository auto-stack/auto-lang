# PLAN-815 T-05 R2 规范增量核验（冻结副本）

Plan r3；phase/T-05，受审 `54aadefadc803af91f6095f8e594b938848d84bd`，修复 `5c4dc277373e2cf388be8e3ea5acbefe891ed5d0`。完整 B 的 T-08/T-09 未完成，以下不是已发布规范。未改 canonical/live ledger。

继承 R1 SD-01..06 的目的/当前来源，按本轮实际修复重新裁量：

| SD | 目标 | 本轮变化与裁定 |
|---|---|---|
| SD-01 | docs/specs/auto-ui-tools/project.md | 新宿主消费已发布 auto-tool-execution 统一树所有权设施，无第二套 OS runner；输入改有界队列；部分反例关闭，但取消/EOF/reader/连续会话/致命输出的完整事实以 R2 报告为准，不得沉淀成“完整已保证” |
| SD-02 | docs/specs/tool-protocol/project.md | wire/四能力未改变，R1 原有 19 codec 复验通过；1=内部致命退出、2=流级致命、0=正常 EOF 仍为合同，不能将只保存退出码但不终止宿主视为完整内部退出 |
| SD-03 | docs/specs/auto-man/project.md | 旧门面/媒体/请求缝定向 6/6；完整消费/媒体/后端矩阵仍 T-06/07 |
| SD-04 | docs/specs/project-model/project.md | 无模型语义改动，保留既有单源/不重复读取主声明要求，不扩大静态准入 |
| SD-05 | docs/specs/auto-lang/ui/overview.md | runtime、工具与 SDK 分层方向保持；本轮确有 runtime 新公开宿主退出入口，应登记其实际规范触面而非只说消费未改的 814 API |
| SD-06 | docs/specs/repository-boundaries/generated-ui-distribution.md | DELIVERY_CRATES 25→26 补 auto-tool-execution，777 新锚 strict 0/3；新的发行/792 实际 fresh/locked/build/run 仍不能用 T-04 旧收据替代，完整 T-06/07 pending |
| SD-07（实际触面补登记，候选） | docs/specs/auto-ui-runtime/project.md | R1 修复新增 renderer::request_host_exit/host_exit_requested、DesktopEvent::HostExitRequested 与会话订阅/清零；必须描述实际宿主生命周期与可重入边界，先修 R2 中初始化丢请求/再启崩溃等问题，再由 T-08 冻结最终文本。若后续修法移除这些 API，重新核销本条，不能提前发布失败实现的保证 |

SD-07 仅登记已发生实现的知识影响，不改变 AC/V-03、不批准缩面、无新增用户能力要求，plan_revision 保持 3。supersedes_spec_components 已补登记 docs/specs/auto-ui-runtime/project.md；affects 同时补登记 auto-ui-runtime 和 auto-tool-execution。new_spec_components 仍为既有 auto-ui-tools/tool-protocol 两目标（A 已建，B 增补同文件）；touched_goals 保持 GOAL-003/GOAL-007。工具执行设施本轮仅为 JobHandle 增加 Send、上层复用既有语义；没有改其 canonical 寿命合同，也不能以复用 spawn 名义削弱 reader 真停要求。

当前 `docs/specs/tool-execution/project.md` 已读：任何返回路径不带活的自有线程、kill 不等于已停、根退出后 owned 后代先停、非 owned 持管道须具名 PipesHeld；R2 需按此审核实际 wrapper 行为，而非只核对依赖边。该来源保持原发布事实。

## 来源 SHA256

| 来源 | SHA256 |
|---|---|
| `docs/specs/auto-lang/ui/overview.md` | `80b0dce1066f5eaf608ef457829a33d1ad2dddaaf095f2dbe79ba6f572253b97` |
| `docs/specs/auto-man/project.md` | `edb0ee3782002c77b680eebb1535635ce6a60de24502ac47b04f6798cee57e3e` |
| `docs/specs/auto-ui-runtime/project.md` | `23c3de6cdf52fc34cd86efe526ea54146969a0c10e8b48590088cfcd4a7bd993` |
| `docs/specs/auto-ui-tools/project.md` | `5fc40f83a5c6c793f779e46444d20c16205dbce2f3eec48aaedf67381d2385c5` |
| `docs/specs/project-model/project.md` | `01ab9664d612bcbce325844675f561088a0ad37af6ccf1a7e3a879f1db10fe1c` |
| `docs/specs/repository-boundaries/generated-ui-distribution.md` | `93a82b2de3e4f531c190b75177da2c9ae202ae60b86fa94c51da4147205bd950` |
| `docs/specs/tool-protocol/project.md` | `bac8179eb2ea1317b7314aee2d5b709eeb231737f64da8090d523d450f45dddf` |
| `docs/specs/tool-execution/project.md` | `2f6389376c287b9bffb505e318def3c0794bd393b048b9f113355c2168a6a10c` |
