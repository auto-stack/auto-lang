# auto-semantics 普通源码有限标量模板实例（SQ-B，935）

> **Status**: experimental（PLAN-935 冻结；实现 = `experimental/ace/crates/source-query/`
> 的 `instances`/`instance_host` 模块 + core `scalar-instances` 语法面）。
> 934 SQ-A 事实层不变；本层是**并行新命名域**，不占用也不改写 934/796 字节含义。

## 定位与边界

- 935 位于 **core 统一前门** 与 **934 事实路径** 之间：模板 AST 由 core 的
  `scalar-instances` 能力面产出（`ac.source.instances.scalar/v1`），实例化层
  消费模板 AST、以 **796 求解器真 unify** 发现调用点 substitution，产出
  **脱糖具体程序**（模板移除、实例落位、调用点重绑），934 事实路径照常
  在脱糖程序上跑 bind/emit/verify。四端（AV/native/A2R/A2C）**只看到具体
  函数**，目标后端不做自己的模板替换。
- 本层拥有：模板→实例的调度（需求发现/fixpoint 物化）、实例身份（逻辑
  实例 + 实现键）、实例页（S935 层持久化）、实例集合的候选/原子切换与
  单飞。**不**拥有：名字/类型绑定（core）、求解算法（复用
  `ac796-bidir-worklist/1`，扩展输入不复制求解器）、具体函数的事实页
  （934 域，实例体就是普通具体函数）、HIR 发射/校验（core）。
- 有限面（冻结）：单类型参数 `T`；参数/返回/局部注解位 = `T/int/i32/bool`
  （int/i32 同型并 I32）；现直接调用（位置/命名实参）；期望类型传播；
  同实例自递归与跨模板调用（fixpoint）。**覆盖边界外具名拒**
  （`source.instances-unsupported` / 求解器原码），不当 token 跳过：
  >1 类型参数、重名类型参数、模板方法、模板入口 `main`、约束/const 形状
  （`<T has B>`/`<const N>`）、owned/nominal/文本类型位、HOF、显式类型
  实参语法、const 泛型、spec/trait 约束。
- 无能力（capabilities 不含实例能力）时统一前门走 plain scalar 裁剪——
  `<T>`/类型位裸 ident 维持原语法拒绝面，不改旧负例。

## 身份与版本

`producer=plan935/source-instances`、`schema=ac935/instances/v1`、
发现算法 `algo=ac935/instances-discovery/2`（T-04 起为 796 求解器
mint_var+unify；算法换面必须换键）。

| 层 | 身份 | 不入 |
|---|---|---|
| 模板逻辑 | `TemplateLogicalId = ac935/instance-logical/v1` 哈希(package+module+声明名) | span、调用点、body 版本 |
| 逻辑实例 | 模板逻辑 + 有序 canonical substitution（单条 T→I32/Bool） | span/callsite/body 版本、装配名 |
| 实现键 | `ac935/instance-key/v1` = 逻辑实例 + 模板签名轴版本 + 模板体轴版本 + substitution + algo/schema | 装配名、调用者内容 |
| 实例集合 | `ac935/instance-set/v1` = 全部实现键有序合成 | — |
| 装配名 | `{模板}_{ty}`（`choose_i32`），确定性探测避让用户名（碰撞后缀 `__2`…） | **不是身份**——仅脱糖程序内拼写 |

具体函数自身的 sig/body 事实页仍走 **934 域**（`choose_i32` 就是普通具体
函数）；S935 页只承载模板→实例调度事实（信封 producer/schema/algo 严格
准入，他域字节不重释）。

## 调度语义

- **发现**（求解器真 unify）：模板以**签名 stub** 参与 body-scoped 推理
  （模板体不泛型求解——类型错在被请求实例时检查）；调用钩子每类型参数
  mint 一个共享 var、签名替换、实参与期望真 unify；诊断是求解器原码
  （type-mismatch/unsolved-variable/arity-mismatch/unknown-parameter，
  原码原 span 透出）。
- **fixpoint 物化**：每轮真推理发现需求 → 新实例的具体体入列再发现其
  体内模板调用（跨模板/自递归外层）→ 无新需求收敛（每模板至多
  |{I32,Bool}| 实例，有界）。
- **单飞**：同一完整实现键一次生产生命周期；同键复用需页内容**等值**
  再准入（A→B→A 允许复用历史字节，伪内容强制重产）。
- **集合原子切换**：prepare（绑定宿主 revision + 集合指纹）→ attach
  target → commit 重验（取消/late/当前源重推导指纹漂移）→ 一步换入；
  任何拒因保旧集合完整。
- **失效分层**：模板**签名**变化 → 实例键变 + 调用者重验；模板 **body**
  变化 → 仅实例体重产（调用者 body 页 hit、推导 0——装配名与签名轴
  不动）；调用者自身编辑 → 恰该调用者重解，实例复用。

## 恢复

- 实例调度是**纯函数**：同源同 capabilities 同推导（新进程/新会话重推
  导确定性成立）；恢复 = 重推导 + 逐页准入（S935 页信封/键/内容验，
  934 具体函数页照常 `restore_bind`/`seed`，恢复编译 0 求解）。
- 未改实例恢复后 produce 0；首有效编辑恰改动函数重解；首坏编辑具名
  拒、零求解。slot/intern 编号不入任何持久身份（结构化页本无 slot）。

## 资源锚

页 ≤512KiB（IN935P/v1）；发现预算沿求解器 1M steps/5s（预算态是资源
拒绝 `source.instances-resource`，不是语言拒绝）；语义受管 16MiB 不因
实例加份；多读者共享实例（读者数不放大求解次数）。
