# PLAN-815 T-05 R1 — 规范增量核验（冻结副本）

- Plan r3；review_scope=phase，仅 T-05 提前定向；受审 `31a25f5cf0e40c42617140768cd2488df7b55257`。
- 本轮无完整 B Spec 草稿提交；T-08/T-09 仍开放。以下冻结的是主计划 §5 的 SD-01..06 与本次实际变化的核验边界，不是新的发布稿或语义缩面。
- 当前 canonical 来源 SHA256 见下表；主计划原件与 contract-baseline 的审前身份见 baseline.json.txt。本轮未改 canonical/live ledger。

| SD | 操作/目标 | 已读来源与本轮核验 | 裁定 |
|---|---|---|---|
| SD-01 | add（A 已建，B 后续增补）`docs/specs/auto-ui-tools/project.md` | A 窄 describe/validate_project 已发布；本轮代码增加 Vue generate 及 Vue/VM/Rust run、单飞宿主、日志/输入/取消 | 不得写“完整可取消/EOF 收口/owned children 已退出/交互已完整验证”：F01..05 未闭；真实 Launched/交互 quit 正控仍须补齐。单飞与未服务 profile 显式拒绝可据实描述 |
| SD-02 | add（A 已建，B 后续增补）`docs/specs/tool-protocol/project.md` | 新消息、RunReport、Cancelled/InputAck/Launched 已有 codec/进程定向证据；版本仍 1、老窄请求正控保留 | wire 增量可作为候选事实；能力承诺须与所有已声明执行分支一致，不能用源码的步骤边界取消注记覆盖完整取消/EOF 合同，F01..05 阻断完整生命周期落库 |
| SD-03 | modify `docs/specs/auto-man/project.md` | run_vue/run_vm 迁出、旧模块门面转发；6 条 Man/媒体/请求缝测试通过 | 本轮局部证据成立；不能提前称全部管理闭包/旧消费已收齐；T-06/07 保留 |
| SD-04 | modify `docs/specs/project-model/project.md` | 消费既有 GenerationRequest/wire，无本轮扩模型；实际请求经 codec，初版缺 output_root 探针已纠正且保留原始失败记录 | 维持单源与静态准入，不以执行取消问题改变 I-PROJECT 语义 |
| SD-05 | modify `docs/specs/auto-lang/ui/overview.md` | 新运行接线消费已发布 auto-ui-runtime；M814-B 93a25c924 为受审 HEAD 祖先 | 依赖门已解除；不得把当前执行器生命周期缺口归咎于尚未交付 runtime 或写成其已保证正确 |
| SD-06 | modify `docs/specs/repository-boundaries/generated-ui-distribution.md` | T-05 修改运行/发行投影，新增 runtime 依赖；777 strict stale_baseline；独立消费夹具 2/4 红 | F06/07 及 T-06/07 的新闭包/792 实际运行矩阵未齐，T-04 老 792 收据不能覆盖变更后的运行投影。保留既有 M815-A/M816-A/M814 来源章节，不盲覆盖 |

元数据保持 r3：

- supersedes_spec_components: docs/specs/auto-man/project.md, docs/specs/auto-lang/ui/overview.md, docs/specs/project-model/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md
- new_spec_components: docs/specs/auto-ui-tools/project.md, docs/specs/tool-protocol/project.md（A 已创建；B 合入须增补同文件）
- touched_goals: GOAL-003, GOAL-007

本轮结论 needs_fix，不新增对原合同的许可、降级或延期。已发布 A/L、已闭 T-04 原例与范围保持；B 全量 SD 的最终逐条可发布性仍归 T-08/T-09。

## 当前来源身份

| 来源 | SHA256 |
|---|---|
| `docs/specs/auto-ui-runtime/project.md` | `23c3de6cdf52fc34cd86efe526ea54146969a0c10e8b48590088cfcd4a7bd993` |
| `docs/specs/auto-ui-tools/project.md` | `5fc40f83a5c6c793f779e46444d20c16205dbce2f3eec48aaedf67381d2385c5` |
| `docs/specs/tool-protocol/project.md` | `bac8179eb2ea1317b7314aee2d5b709eeb231737f64da8090d523d450f45dddf` |
| `docs/specs/project-model/project.md` | `01ab9664d612bcbce325844675f561088a0ad37af6ccf1a7e3a879f1db10fe1c` |
| `docs/specs/auto-man/project.md` | `edb0ee3782002c77b680eebb1535635ce6a60de24502ac47b04f6798cee57e3e` |
| `docs/specs/repository-boundaries/generated-ui-distribution.md` | `93a82b2de3e4f531c190b75177da2c9ae202ae60b86fa94c51da4147205bd950` |
| `docs/specs/auto-lang/ui/overview.md` | `80b0dce1066f5eaf608ef457829a33d1ad2dddaaf095f2dbe79ba6f572253b97` |
