# PLAN-815 T05～08联合收口 SD核验

冻结a67f114fda8abf52630126d4aa52eeacd6d3248a，base afd1a66bf，Plan r3。T08定稿对象见frozen-handoff.md；当前canonical Spec来源哈希见baseline.json.txt，终核零漂移。结论仅支持T08任务完成并交T09终审，未直接发布规范/ledger。

| ID | 目标与当前阶段核验 |
|---|---|
| SD01 | docs/specs/auto-ui-tools/project.md：A窄子集已存在，B拟扩至真实生成/build/run/资源/生命周期；与已验证服务集及唯一owner相容。merge按现存文件受管增补，不覆盖A来源事实。 |
| SD02 | docs/specs/tool-protocol/project.md：A的v1/framing/预算单源保留，B补五操作、单飞、交互取消/双终态、控制日志与退出；release实际能力一致，生命周期沿既有真实证据。 |
| SD03 | docs/specs/auto-man/project.md：旧门面/legacy结果语义与auto-api-gen非UI单算法owner如实保留，协议严格失败不反写成旧门面行为。 |
| SD04 | docs/specs/project-model/project.md：809单源请求/静态准入边界不改；字段表已纠正真实消费面、相对根、原样载荷及不读主pac；旧source adapter跨文件边界不被抹去。 |
| SD05 | docs/specs/auto-lang/ui/overview.md：使用814已发布公开包，托管外部命令走owned设施；未扩大到所有设备原生执行均已实测。 |
| SD06 | docs/specs/repository-boundaries/generated-ui-distribution.md：源码包tool新发行面33/33和资源已审；T08release精确参考身份已核。源码/目录digest/实际artifact分层不承诺逐字节复现；每文件blake3是crate目录digest计算输入，并非manifest内逐源文件条目表。新发布仍需T10实际冻结收据。 |
| SD07 | docs/specs/auto-ui-runtime/project.md：宿主退出窗口API与每VM会话独立进程，证据闭环及真实边界沿已审来源；T09再完成全计划最终正文核验。 |

supersedes维持主Plan的runtime/Man/UIoverview/project-model/distribution五路径；new tools/protocol两路径对应最初组件增加，A已落盘部分由B增补；touched_goals=GOAL-003/GOAL-007。无产品合同变更，不增plan_revision。T09未完成、merge-owned门禁pending。
