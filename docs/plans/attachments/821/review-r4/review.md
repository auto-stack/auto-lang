# PLAN-821 final R4 — needs_fix

2026-10-26，r5；受审 `5fc1a0aa2d9848a9d6e652cb0798c8c0719fa653`，前审 `18dee14f9b5a7181a94cc2f48e3431e8a8108898`，基面 `b1b8b425e563ba8358e4adc99c3e6c819a8fc137`。唯一树 `D:/autostack/.wt/lang-821/auto-lang`，plan-821-dev，入/出审clean。当前聊天为R1–R3独立审查上下文，未参与实施/未委派代理。

[baseline及输入hash](baseline.json)、[入审Plan](plan-r5-at-entry.md)、[SD冻结件](spec-delta-frozen.md)。未变Spec/依赖及已关闭旧例证据继续有效；新单函数会话和逐字段状态不能套用旧机制的完整通过结论。交接仍承认V05必需矩阵缺口，本轮按就绪性条款核修复及相关反例，没有启动完整最终验收/统一回归。

**结论：needs_fix。** 上轮具体现象全部翻转、R2-01健康关闭；但原F07/08/09完整不变量仍有修复回归，F11及V05未完成。保持原finding ID，不将旧例关闭等同完整不变量关闭。

## 验证与通过事实

在实施树：

```powershell
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_owned_821_r1_replay --test av_owned_821_incremental
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_owned_821_resources av_owned_821_r1_worker_memory_hard_gate_parent_sampled -- --exact --nocapture
cargo check --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe
git diff --check 18dee14f9 HEAD
```

9+6+1=**16个定向测试通过、0 ignored**；check成功且无警告，diff-check通过。[族测试raw](scoped-tests.txt)、[资源同门raw](resource-gate.txt)、[check](check.txt)。资源单例实际含8个新worker、独立量纲的零阈值/越线/合法跨线控制；未扩成完整资源或性能矩阵。

独立probe（主检出docs附件，不改实施）：

```powershell
cargo run --quiet --manifest-path docs/plans/attachments/821/review-r4/probe/Cargo.toml --target-dir D:/autostack/.wt/lang-821/auto-lang/experimental/ac-av-probe/target
```

[源码](probe/src/main.rs)、[raw](probe-output.txt)、[原R3输入首次重放](r3-probe-replay.txt)。确认：130层/5619B链Success；body-only frontends1→1、gate绿、结果8；候选Trap失败revision1/production相同/旧程序7；单mut、混只读、双mut、owned Str返回依次**808/810/8917/801**。这些修复有效。

## F821-R1-07 / P1 / AC09、T09 — 连续变长编辑使用陈旧声明坐标

原whole前端缺口已接入D828真实单函数通道，旧例闭合。但新声明索引没有跟随成功编辑更新，合法第二次编辑被误拒。

独立probe的BASE是已有合法record/move程序（结果7），编辑索引来自**当前源文本**：

| 序列 | 预期 | 实际 |
|---|---|---|
| main `n: 7→n: 1234` | 1234 | Ok，1234 |
| 接着同main `n: 1234→n: 8` | 8 | `parse main@slice: source.syntax expected ',' or ')'`，仍1234 |
| 新会话，前方read的 `return p.n→return p.n + 100` | 107 | Ok，107 |
| 接着后方main `n: 7→n: 8` | 108 | `expected 'fn', found integer literal`，仍107 |

原因已定位：`owned_session.rs:500–519` 的CommitBody发布只commit环境，没有更新self.decl_fns/decl_types坐标；`build_single_body_candidate:538–543` 继续拿冷态decl.lo/hi加**本次**长度差截取。修改函数自身hi和后续声明lo/hi都已陈旧。等长单次编辑样本未触达该状态。

关闭：当前源坐标/身份/来源随成功编辑一起原子更新，拒绝编辑不更新；重放上述两序列、正负长度变化、后方type声明与后续函数定位合法对照，同时保证未改函数不重前端。不能用整模块fallback把body编辑变长后的错误隐藏为结构门绿，不需要重放无关性能战役。

## F821-R1-08 / P1 / AC02/03/04、T03/04/06 — 字段状态不随重初始化重置，且未检查路径重叠

旧三个mut反例全闭合；新逐字段状态引入两个共享机制问题。

### 1. 新owner继承旧moved_fields

合法源码 [mut_loop.at](mut_loop.at)：set(mut p)令p.n加1，caller从7开始循环两次set(a.mut)，应返回**9**；前端通过后MIR verifier却以 `field-move of already-moved path [1] on slot#10` 拒绝。

独立typed合法对照更小：Const A→Aggregate(root)→FieldMove(root,[0])→Drop提取值；然后Const A→**重新Aggregate同root槽**→FieldMove(root,[0])。当前第二个FieldMove仍 `mir.use-after-move`。新完整owner已经初始化，必须允许再次消费；源语言未要求部分move，本例是机器wrapper的整根重建。

原因：所有Initialize/Replace只clear moved/dropped，未clear该槽moved_fields；循环回边合并旧字段状态，Call新wrapper/完整Aggregate都不能重置。因此不是循环新支持需求，而是§5.5现有循环/真实初始化责任被新状态破坏。

### 2. ancestor/descendant重叠消费漏前门

独立未信任typed输入：Outer.inner:Note、Note.body:Str，先FieldMove(root,[0])移出inner，再FieldMove(root,[0,0])读已移出inner里的body。预期授信前use-after-move、dispatch=0，实际ABC**授信并执行**后 `Infra("field-move path [0, 0] does not resolve on the root (admission violation)")`。

`OwnState::field_moved:167–171`只比较path完全相等，不检查前缀重叠；FieldGet/FieldMove共用该错误事实。已移走整父字段，任何子字段都不能继续消费；反过来子字段已移走时，整父字段也不是完整已初始化值。当前实现把未初始化残骸交给executor，不满足非法状态消费前拒绝。

关闭：整根Initialize/Replace建立新的字段状态；move/clone/Call等定义的状态转移一致；所有读取/消费按实际投影检查父子重叠，分支/回边有界合流。永久回放包括mut循环、typed整根重初始化、父后子/子后父、合法互不重叠兄弟字段与已有双mut/owned返回对照。不得把有效loop改为unsupported，或只改重复路径错误码。

## F821-R1-09 / P1 / AC05/06、T02/04/06 — 指标量纲已正，heap估算漏RecordView载荷

旧磁盘/进程峰值混量纲与同gate反控已修，资源定向单例通过。新hot/staging公式依赖 `MirModule::heap_estimate()`，但该函数仍把 `RecordView(_)` 放在inline零动态载荷分支（mir.rs约657–669），没有计入其RecordTypeRef的owner/name/字段Vec及字段String堆；新FieldMove路径Vec也没有计入动态路径支路。它不是只含一个引用的类型：RecordView直接拥有RecordTypeRef及字段数据。

独立counterexample `memory_estimate_holdout`（源码见probe，未跑额外大规模压力）：3000个只读record字段，字段名各约1000B，wire为**3,035,011B**，仍小于冻结4MiB，表族也在65536以内；真实encode/decode/深授信通过。与owned_driver相同hot公式给出：

```text
hot_estimate = flat.len + mir.heap_estimate + loaded.module.heap_estimate
             = 3,037,375 B < 8,388,608 B
```

此时flat、原MirModule、loaded.module都同时存活，仅flat字节+两个独立字段名String的实际长度，**载荷下界就有9,068,791B > 8MiB**，尚未算字段向量/名称头/原wire pages。该下界直接从同时活着的独立owned String和flat载荷相加，不是RSS猜测。因此现公式能把已超hot预算的输入判为低于预算。

边界：该例为已支持的**未信任typed MIR→ABC→深验**，不冒领此大类型的完整源码能力，不是新增规模承诺；其大小明确在原冻结输入/表预算内。本例证明计量函数漏载荷，不声称已经经source owned-request跑过同源大工程。

关闭：新类型/指令的动态堆载荷均纳入实际工作集保守上界，原对象/loaded/编码暂存同时存活时正确记账，分配前适用预算门用同单位。重放上述合法范围输入与阈值两侧，保留当前独立四线checker/合法跨线控制；不要又改成磁盘尺寸/进程峰值代计量。阶段峰值与终态区分保持。

## F821-R1-11 / P2 / AC07、T06 — 现状规范尚未冻结且V05仍明确未完成

SD文件仍有`<本次提交>`占位、将frontmatter称作execution_done但实际frontmatter是executing7/9；新完整支持/资源文字又被本轮状态与计量反例否定。必须先修上述不变量再冻结准确before/after与版本证据。

V05在原冻结case清单明确必需（旧帧/跨store/generation，typed AV句柄表、无效访问前拒）。当前工作记录仍具名承认无runtime探测、待review裁定；没有新的实现/运行证据或获批合同修订。review不能把“旧例测试绿”裁成此行已过。需要work给出原矩阵可执行oracle，或回new明确实际机制与原要求的合同差异并走必要授权；当前不是允许下游继续保持pending的最终pass。

不要求修改canonical Spec/ledger来过review；这些仍归merge。

## 关闭与保留

| Finding/范围 | R4结论 |
|---|---|
| R1-01/03/04、F06深度、R2失败安装 | 既有有效证据保持，原具体现象已闭；未随新HEAD无故重跑所有族 |
| R1-05 | 旧35MB/n²问题证据保持；本轮不以新的字段数据流为完整内存审查pass |
| R1-02 | 旧输入闭，新借用/字段组合按相关路径仍待最终核验 |
| R1-07 | 旧whole前端例闭；新连续变长编辑不变量OPEN |
| R1-08 | 808/810/8917/801四例闭；新owner/循环/重叠投影OPEN |
| R1-09 | 旧量纲与checker反控闭；载荷漏计OPEN |
| R1-10 | 本修复未改，仍保留work证据，完整独立恢复复核pending；已知必修未完时不启动全轮验收 |
| R1-11/V05 | OPEN/具名必需缺口 |
| R2-01 | CLOSED，本轮真实check/15族测试编译无警告 |

保留T01/T05，重开T02/03/04/06/T09，**executing2/9**。AC01..06 partial、AC09 fail、AC07 fail、AC08 merge-owned pending。该计数保留T02的首例已实现历史，但其work-owned资源证据尚有F09缺口，不能勾完整任务。

第四次final提交。下一次work诊断需覆盖“单例→连续变长编辑”“同槽新owner/循环→旧字段事实”“路径完全相等→前缀重叠”“新类型inline分支→真实owned堆载荷”，附本轮输入、永久oracle、新commit与原ID闭环；不要以再次增加aggregate总数替代。修复范围与原验收一致，无新产品需求。

未改实施、未合入/归档/清树，未跑裸cargo t/触面整档/完整acceptance或tf。最终代码/验收/Spec通过后，merge-owned统一门仍pending。
