# PLAN-815 T-05 R7 / T-06～T-08 提前 R2 规范增量核验

2026-10-22；Plan r3，review_scope=phase，非T-09终审。冻结HEAD e3fdcdc76c5d28ec9274a9101acc3f6a80c5c6a6，code d039f23995cb7763787b86cd27f4144627d69d3d（含9291a7ee3修复）。当期交接全文见frozen-handoff.md，当前canonical/依赖/配置/锁的hash见baseline.json.txt。未修改canonical或live ledger。

实际二进制385ffef8f131e08a08fcc41487881f54ba4a4815cbaebe0968284116ad696f16与交接一致，结案时未漂移。身份核验不代表能力矩阵全部成立。

| SD / target | 当期核验 |
|---|---|
| SD-01 docs/specs/auto-ui-tools/project.md | 原M815-A已发布窄端点必须保留。VM默认/prop/node优先三种入口呈现通过，Rust自定义front仍在compile_rust_ui_app失败。Ark原手工准备假completed已消除，但构建成功后的bundle/ability读取带JSON5标点，真实启动链尚未闭合。不能落为完整运行/全部字段贯穿已通过。 |
| SD-02 docs/specs/tool-protocol/project.md | 捕获式ADB超时/截断具名failed及自然EOF/取消原例关闭；不能外推所有profile完整交互。完整协议消费者回执/深度负控仍属T-07待办。其独立闭包门本轮对真实UI文法/lang-infra注入有效拒绝。 |
| SD-03 docs/specs/auto-man/project.md | legacy Jet失败/Ark缺wrapper返回Ok定向2/2，协议严格失败边界保留。Ark新增真实链不改变旧手工API。不要用旧手工语义豁免协议的真实运行要求。 |
| SD-04 docs/specs/project-model/project.md | 请求快照与front节点优先在VM正反控成立；Rust分派只修准入，生成与运行CWD相邻路径未全接线。PM外部合法bun回退成立，新来源oracle仍受legacy缓存的测试顺序干扰。完整字段矩阵pending。 |
| SD-05 docs/specs/auto-lang/ui/overview.md | 公开包方向/单源沿未变来源保留；新Ark链确有owned命令调用，但正确参数尚不成立。T-07纯协议闭包门四包精确集合在正控及两种真实越界注入中表现正确。 |
| SD-06 docs/specs/repository-boundaries/generated-ui-distribution.md | 777本分支strict pass 0 violations/3 grandfathered；基线相对原冻结仅base_revision更新、未拓宽边/豁免。不是master并集与792发行通过。真实Vue构建成功测试通过，产物引用oracle却接受损坏/无引用，需要修复后才能称可消费内容验证完整。 |
| SD-07 docs/specs/auto-ui-runtime/project.md | VM三个目录形态实际MCP呈现+cancel/EOF退出通过，沿已有公开生命周期边界继承；本轮Linux仅pkg模块实跑7/8（新矩阵缓存失败），不能把工作侧Linux全绿总数直接复用为本轮结论。先前R6 Unix具名4项原始证据按未变代码/配置边界保留。 |

当期handoff的“front贯穿rust判定”仅是准入事实，不能代替Rust完整生成/执行；“所有served run真实执行链”仍需Ark启动正控。SD-01/02仍标add且before为迁移前历史，但目标在A阶段已经存在；最终B补丁应按当前已发布A内容modify/增补并保留来源，不以历史before覆盖当前事实。该已知T-08待办继续保留，不另重复编号。

T-08为准备稿partial，须随本轮剩余问题与完整T-07验收定稿。T-09整体AC/Spec审查、最终pass后merge-owned冻结裸cargo t及实际触面档/合入/沉淀/归档仍pending。
