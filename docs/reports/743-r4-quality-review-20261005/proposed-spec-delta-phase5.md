# PLAN-743 r5 proposed Spec delta（SD-09，未沉淀）

Target: `docs/specs/auto-acc/project.md`；modify。仅细化并兑现SD-08，不增加HIR阶段/pass或扩大语言能力。canonical/ledger仍保持r4，独立review pass以后由merge沉淀。

参数遮蔽规则必须覆盖普通及mut参数、自由函数及已观察到的方法（包括静态方法）；裸调用与qualified调用在宿主/类型升级前均检查可观察绑定。命名冲突或不能证明归属者保留unknown；不凭同名升级native-runtime，不引入完整resolver。

族引用消费者只接收类型、字段语义、证据存在、同条证据绑定、新鲜度和全局唯一ID均通过的决定。重复ID整组无效（包括先插入记录）；可采用完整验证失败后停止族消费的等价实现。invalid/stale不贡献族覆盖，不能以最终CLI非零替代消费者可信边界。

`--check`在访问JSON成员、规范化audit-only HEAD前检查顶层和source_identity对象形状；合法JSON但形状非法必须受控非零ERROR，不出现traceback，不修改manifest/manual。保持完整重生成比较及audit-only HEAD兼容行为。

最终归档检查从实际文件父目录解析Markdown相对链接（忽略围栏代码），核对33个任务、frontmatter、模块导航和全部P743-*指针。活动态可做模拟归档，但必须以真实归档检查收口；历史r4断言与收据保留。
