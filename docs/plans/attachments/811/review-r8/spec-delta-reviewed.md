# PLAN-811 规范增量审核稿

2026-10-20 · revision 1 · implementation `5df643efafda4bbf0bd3aae9ea4e0b319510e41b`

本稿供merge沉淀，未发布到canonical或live ledger。原始Plan/交接提案保存在`spec-delta-input.md`；本稿将历史修复记录归并为现行规则，不复制旧“未实现/移交R04”的已修媒体事件缺口。

| ID / 目标 | 审核后的现行规则 | AC |
|---|---|---|
| SD-01 add `docs/specs/auto-media-core/project.md` | `auto-media-core::{image_pipeline,media_service}`是renderer-neutral资产类型、registry、cache、调度/worker、图像会话、本地媒体索引及Range值规则的单源；闭包不依赖auto-lang/VM/UI/Man/iced。image-codec为默认可关闭特性，纯媒体文件服务仍可独立使用。HTTP socket/server、语言编解码和纹理/事件执行留adapter。进程默认服务共享；owned MediaRuntime关闭准入，收回scan/open生产者、decode/resize线程并将registry/session归终态，重复关闭等待同一完成结果；不承诺抢断不可取消codec指令。编码/解码缓存保64MiB/256MiB策略及hard-pin例外，1GiB/400MP拒绝界、会话/引用/TTL与旧行为保持；不冒称RSS强上限或无限规模支持。关闭完成性用线程join、registry终态、引用释放及占用归零联合验证。 | AC01..05 |
| SD-02 modify `docs/specs/auto-lang/vm/overview.md` | 17项image native及媒体HTTP快路径直接消费core；保16固定ID（2960..2975）、thumb按名分配、ui-iced注册开关及原签名/值域/stub语义。控制面保持不透明票据/URI和元数据，像素、绝对路径不回灌VM状态；关闭/retain/release/会话owner保持已冻结合同。 | AC01/02/04 |
| SD-03 modify `docs/specs/auto-lang/ui/overview.md` | 原`ui::{image_pipeline,media_service}`仅薄重导出同一类型/服务；非renderer机制实现在core。Iced查同进程registry/decoded pixels，纹理和UI事件在UI；HTTP数据面不复制registry或嵌另一份core源码。photo_service、mpv、i18n、直接frame及renderer等剩余owner如实列明，不把本Plan完成写成所有UI/宿主依赖已拆完。 | AC01..05 |
| SD-04 modify `docs/specs/auto-lang/ui/design/image-surface-events.md` | 保留Ready后且下一渲染拍单发loaded、失败单发error、handler身份+src订阅、换源/卸载退订及A→B→A恢复等既有事件合同。VM轨参数投影按canonical规则；typed merged使用其消息事件面。registry/代次/资产状态由core拥有；注册、渲染及派发留UI。merged同步/async入口共用50ms媒体唤醒；回调改写视图后继续注册/退订，不要求应用另加timer。 | AC04 |
| SD-05 modify `docs/specs/auto-man/project.md` | api_gen/rust_ui的媒体生成/HTTP桥及生成manifest消费core；与PLAN-809已落地的auto-man-core单一管理规则源、语言效果adapter和效果承载者分工并存。UI生成算法本体及其他既有未迁面按当前canonical保留，旧handoff“项目规则尚未迁移”不回写。 | AC04/05 |
| SD-06 modify `docs/specs/repository-boundaries/generated-ui-distribution.md` | `auto-media-core`及其codec/source闭包进入受影响生成/交付profile；fresh/locked、重定位、缺包与非法反向边负控沿777/792机制验收。最终合流/发行门按冻结实现和配置执行或注明等价证据复用；不能以生成字符串检查代替真实消费者。 | AC05 |

## 元数据与沉淀边界

- supersedes：上述SD02..06五个精确路径；new：SD01路径；touched_goals：GOAL-003、GOAL-007（现有目标，未新增目标）。
- merge同时按仓库§4扩展登记全局overview的新crate、相关module/plans及目标进展，随后兼容ledger upsert与spec-index；本review不执行这些发布动作。
- `specs/README.md`的旧整档时序由当前AGENTS和2026-10-14裁定覆盖：最终review pass后由merge在冻结提交统一裸cargo t+tv及适用触面档；tf为主检出到期批量。review pass不等于交付完成。
- 旧行为/设计差额仍按contract-baseline具名记录；不以规范稿批准延期、取消既有验收或升级原stub能力。
