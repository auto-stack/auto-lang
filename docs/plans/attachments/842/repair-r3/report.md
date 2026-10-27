# PLAN-842 R3 修复与重新验证

连续两次独立 final 非通过的共同不变量、根因及调整见 [诊断](diagnosis.md)；上一轮真实反例见 [独立 R3](../review-r3/report.md)。最终源码 lang `729d5a060e1c5e3b7f004f5a7188f27a8831f7d2`、UI `b6072a664e4b20d49724ad26ce92df23d9374dcb`、Man `e44e7391df6b17b8e343d685522a95fb28dec759`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，语言并集基线 `a547f52c5fbd5a24a203e94051af1e040c808f42`；三树 clean、全计划差异格式检查通过。

- 受管 SDK 单一冻结收据校验 SDK 全部受管文件集合及 bytes/SHA-256；检测缺失、额外、内容变更并排除不会复制的 `target` 等构建缓存。兼容现行有效 `man-install/1`，收据不是签名。工作区自动发现与显式安装源均先核验，再写输出；当本机邻接 SDK、另一份具 Full 双执行文件的安装或有安装收据的 SDK 被选中时，清单缺失不能将其降为普通开发源。
- 交付使用同一收据核验逐 crate（含所有权）、patch 的实际 staging 原始副本，根输入与根 Cargo.toml/lock 核验被消费的字节，投影在验证后进行；最终 swap 前检测收据漂移；UI 生成器指纹从已校验的 staging 副本计算。原有效的外部显式 UI 开发源和非受管开发仓依旧支持。
- 非受管语言和 UI 开发源补本地路径依赖闭包预检，涵盖 workspace 继承、目标条件、循环及源码入口；注册表依赖不要求联网，正常开发仓和显式覆盖真实正控。此前仅直接包存在但缺语言或 UI 本地依赖的真实生成现具名拒绝，旧 workspace/app 不变。
- 定向回归：闭包坏源码先红后绿；SDK 完整性、预检后 staged 改变/缺失/额外与收据变化的单元反例通过；来源模块 8/8、UI 7/7、真实源码交付 1/1、备份恢复 2/2，语言类型检查通过。原始测试输出在同目录和 [安装分项](installed-logs/)。

## 最终 Full/Core 与真实反例

Full 实装 6806 文件，manifest SHA-256 `0f9e3e19afb631b01b22686869e2ee1a8fec5969a891114506978dc62f95edb1`；真实 CLI 生成、fresh lock、locked build、生成宿主 probe、依赖树及资源 verify 通过。安装 SDK 注入非直接语言包变更/缺失/额外文件、根输入、根锁、patch、开发传递依赖，均具名拒绝并逐字保留旧输出；预检后 staging 期间更改尚未复制的真实语言包被拒绝、旧交付保留，见 [完整门记录](r7-final-delivery.log) 与 [复制期控制](installed-logs/installed-closure-staged-drift.log)。安装 receipt 缺失时，当前邻接 SDK 和另一份安装 SDK 的显式语言/交付根均拒绝，旧文件不变；清单和源恢复后实际生成及安装 verify 复绿。

Core 636 文件，manifest SHA-256 `ae5471b9403aaa0f03316b0049ee8f94071009ea05295be8d1a032b188a22cb5`；安装 verify、实际 VM、UI 缺席、SDK locked check 和独立外部 consumer fresh lock/locked 3/3、无 UI/旧门面闭包通过。最终安装实际 Vue 生成/生产构建，blueprint 邻接读取与恢复、交付资源等长篡改/未来 schema 拒绝与恢复通过。

[最终测试选择摘要](final-selection-summary.json) 绑定以上代码/锁：language 3212、UI 3117、tt 3563、tb 3102 实际 selected，21/21 迁移目标有真实 nextest 列表匹配；此为 list-only，非全档执行。原 9 recipe、10 token、2 parser 定向测试及 18 项驱动控制，代码与输入未变化，可复用 [R2](../repair-r2/report.md) 证据。保留既有 compiler warnings、MPV 视频未实测和 1 项既有 ignored；主检出其他 WIP 未改。

最终独立复审与合入统一门、到期的批量回归、实际落地/归档仍 pending；不能将本工作记录当作复审 pass 或交付收据。
