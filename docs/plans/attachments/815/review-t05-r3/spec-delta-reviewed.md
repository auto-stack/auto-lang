# PLAN-815 T-05 R3 — 冻结规范增量核验

Plan r3；review_scope=phase/T-05，不是T-09完整B终审。对象 `64ca649ffe91d61efd20610ecc4b86fc78607c9d`，修复基面 `54aadefadc803af91f6095f8e594b938848d84bd`。本轮未提交完整B草稿，当前文本是当期核验，不是已批准发布稿。未改canonical/live ledger。

| SD | 目标（均为既有A/B文件的后续增补） | R3核验裁定 |
|---|---|---|
| SD-01 | docs/specs/auto-ui-tools/project.md | 独立VM子进程使原初始化cancel/EOF与连续会话原例成立；新真实VM Launched/Input q正控成立。不能据此写完整无语义变化：F01的EOF分类、R3-01上下文交接、R3-02初始化交互仍失败 |
| SD-02 | docs/specs/tool-protocol/project.md | 四能力/wire/版本1保持，19协议测试与真实A请求通过；fatal在输入仍开放时exit1已闭。命名收尾错误仍需按真实EOF/停止/读错分类；InputAck仅表示写入管道，不能用来掩盖已受支持退出命令被内部启动快照忽略 |
| SD-03 | docs/specs/auto-man/project.md | 旧门面/媒体/请求缝6/6；没有授权删除外部后端、请求声明/主题等已有语义；完整兼容仍T-06/07 |
| SD-04 | docs/specs/project-model/project.md | 无模型变更；跨进程仍须传递I-PROJECT必要语义。只给子进程一个入口文件路径不等于完成已求值请求/模块来源交接，不以重新读取pac构造第二真源补洞 |
| SD-05 | docs/specs/auto-lang/ui/overview.md | 分层依赖方向保持；UI执行形态从同进程变子进程，须核销父进程的非env状态，不得把runtime/SDK正常边称成语义已保持 |
| SD-06 | docs/specs/repository-boundaries/generated-ui-distribution.md | 无新清单/资源输入，DELIVERY26与777锚保持；真实tool将自身作为VM helper执行，完整发行/重定位/内部子进程路径需在T-06/07及最终发行证据中核验，旧T-04收据不覆盖此变化 |
| SD-07（候选） | docs/specs/auto-ui-runtime/project.md | 新代计数API替代bool、会话订阅与子进程stdin退出通道。cancel/EOF由owned子进程终止保证，不能把这当成代计数“初始化请求不丢”的证明；Input q初始化反例仍复现。最终文本须在修复后区分本会话请求/历史代数与跨进程状态，不发布错误保证 |

元数据维持R2实际触面登记：supersedes_spec_components含auto-ui-runtime、auto-man、auto-lang/ui/overview、project-model、repository-boundaries/generated-ui-distribution；new_spec_components为auto-ui-tools/tool-protocol（A已建，B增补）；touched_goals=GOAL-003/GOAL-007；affects含auto-ui-runtime/auto-tool-execution/tool/protocol/Man兼容/资产。无新增AC、无缩面，plan_revision保持3。

已发布A/L及T-04关闭范围保留；完整B草案与最终逐条可发布性仍归T-08/T-09。并行canonical来源与本轮基线相同，未覆盖M814/M816现有发布内容。

## 当前canonical SHA256

| 来源 | SHA256 |
|---|---|
| `docs/specs/auto-lang/ui/overview.md` | `80b0dce1066f5eaf608ef457829a33d1ad2dddaaf095f2dbe79ba6f572253b97` |
| `docs/specs/auto-man/project.md` | `edb0ee3782002c77b680eebb1535635ce6a60de24502ac47b04f6798cee57e3e` |
| `docs/specs/auto-ui-runtime/project.md` | `23c3de6cdf52fc34cd86efe526ea54146969a0c10e8b48590088cfcd4a7bd993` |
| `docs/specs/auto-ui-tools/project.md` | `5fc40f83a5c6c793f779e46444d20c16205dbce2f3eec48aaedf67381d2385c5` |
| `docs/specs/project-model/project.md` | `01ab9664d612bcbce325844675f561088a0ad37af6ccf1a7e3a879f1db10fe1c` |
| `docs/specs/repository-boundaries/generated-ui-distribution.md` | `93a82b2de3e4f531c190b75177da2c9ae202ae60b86fa94c51da4147205bd950` |
| `docs/specs/tool-execution/project.md` | `2f6389376c287b9bffb505e318def3c0794bd393b048b9f113355c2168a6a10c` |
| `docs/specs/tool-protocol/project.md` | `bac8179eb2ea1317b7314aee2d5b709eeb231737f64da8090d523d450f45dddf` |
