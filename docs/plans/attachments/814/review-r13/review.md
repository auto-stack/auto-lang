# PLAN-814 R13：R12 修复后完整 T-10 复审，needs_fix

2026-10-21；stage: review；review_scope: final；plan_revision: 3。
reviewer_identity：本聊天 Codex 独立复审上下文，未参与实施；未创建 agent/新 checkout。以提交代码与独立重放判定，不以 work 的关闭标签代替验收。

## 冻结身份与范围

- 原唯一实施树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`。
- reviewed_commit `feae28e0f16beba7637f81c9499a4ed62fe40282`；tree `034f49985eaca1e67ab1e30b99e20d32dc5264cf`。
- 修复 base=R12 `1bbf986208b2486e4e899dd58209aecacc3a0608`；B base=A 发布 `f6e07aceeeea3cb12698c747d61db5be43f7458c`。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。实施及依赖树初末 clean/HEAD 一致；main HEAD、源码/Spec/附件指纹见 `input-hashes.json`。
- diff 仅 `crates/auto-ui-compiler/src/ui_gen/rust.rs`（337+/141-）：替换操作数区间/表达式类型解析，字段分组及调用括号判别；未改测试声明、选择配置、Cargo/锁、SDK、runtime、发行驱动或其它三个生成器。
- work 已落实两连败诊断与设计调整（主计划 §9 续7），原 R12 冻结快照无漂移，本轮复制到 review-r13 独立再生。不是因为缺少诊断拒收；以下为新解析路径的实测未闭内容。

## R9-01 / P2：原八例 CLOSED，类型保持整体仍 OPEN

影响 AC-03/06、T-05/T-08/T-09。仍为原 F814-R9-01，不增加独立 finding。

原 R12 八项失败本轮全部编译通过：字段自身左右分组、双侧嵌套、Value prop、声明 f32 算式左右位的投影现在正确。work 的 **38 feature+默认正控=39 组真实编译全部 exit0**，另 overlay 八组合运行 mismatches0/exit0；即 work 所报 40 个检查均独立重放通过。既闭 R11-01 告警项、R10 浮层/777/serde 项保持闭合。

但替换解析器仍有以下两种同一类型不变量缺口：

### 1. 后向区间扫描没有完整消费 `<=` / `>=`（本轮替换引入的回归）

位置 `rust.rs:8853-8857`：`operand_span_after` 将 `<`/`>` 无条件设为长度1，面对 `<=`/`>=` 时留下前导 `=`，把 `= .limit` / `= (.threshold + 0.5)` 当操作数。`classify_operand_expr` 得到 Unknown，字段错误回落 as_str。前向 `operand_span_before` 正确处理二字符运算符，所以交换位置又通过。

| fixture / 条件 | 实际结果 |
|---|---|
| le-loop：`p.rank <= .limit` | `p["rank"].as_str().unwrap_or_default() <= self.limit`，str 与 i32，E0277，exit101 |
| ge-loop：`p.rank >= .limit` | 同构 str 与 i32，E0277，exit101 |
| prop-le / prop-ge：`note.rank <= .limit` / `>=` | Value prop 同构 str 与 i32，均 E0277、exit101 |
| group-le：`((p.rank)) <= ((.limit))` | 分组不再误进 truthy，但操作数错误分类 str，E0277，exit101 |
| group-ge-float：`(p.score) >= (.threshold + 0.5)` | str 与 f32，E0277，exit101 |
| le-loop-rhs / ge-loop-rhs：`.limit <= p.rank` / `>=` | 正确 i32 投影，均 exit0 |
| known-ge：相同 `p.rank >= .limit`，列表元素形状已知 rank:int | 正确 shaped i32 访问器，exit0。证明语法及操作本身可用，缺陷在未知字段投影路径。 |

原 R12 base 的旧扫描明确会在 `<`/`>` 后继续跳过 `=`（`git show 1bbf98620:...`），本替换将其丢失。本轮没有另建旧 checkout/冒称旧提交执行；新提交失败与现有正确方向、形状正控均为真实编译证据。

### 2. 已支持的布尔取反操作数仍被当成字符串

位置 `rust.rs:8894-8939`：`classify_operand_expr` 处理一元 `-`/`+`，却没有布尔 `!` 的类型传播。`(!.allowed)` 剥壳后为 `! .allowed`，已声明 bool 信息丢失，Unknown→str，两个方向及 prop 都失败。

| fixture / 条件 | 实际结果 |
|---|---|
| not-bool：`p.dirty == (!.allowed)`，allowed:bool | `p["dirty"].as_str().unwrap_or_default() == (!self.allowed)`，str 与 bool，E0277，exit101 |
| not-bool-rhs：`(!.allowed) == p.dirty` | bool 与 str，E0277，exit101 |
| prop-not-bool：`note.dirty == (!.allowed)` | 同构 str 与 bool，E0277，exit101 |
| known-not-bool：同条件、已知元素形状 dirty:bool | `as_bool().unwrap_or(false) == (!self.allowed)`，exit0。与原 `p.dirty == .allowed` 正控一起，隔离既有 bool 表达式组合的类型保持问题。 |

这不是新增任意表达式或额外语言规模承诺；`!`、bool 状态及该比较本来就能解析/发射，形状正控完整编译成功。未知字段不能仅因为操作数多了一层既有布尔取反而降为字符串。SD-02“另一操作数完整表达式类型/任意透明分组”也尚不能据此审定为全部已实现。

## 重放与健康

67 个输入均成功解析/生成；**56 feature+默认正控=57 组实际编译，48 pass / 9 fail**。新增代表组合18组（含两个形状正控）为9 pass /9 fail；原39组全保持。新通过的其它组合包括整型逻辑链、嵌套 f32 算式左右位与逻辑或、负嵌套浮点、字符串中带算符、字符串状态双侧分组。

完整逐项退出码在 `results.json`；源/产物/命令脚本、错误块及哈希均在本目录。`known-ge/known-not-bool` 为第一轮 replay 后追加的两个正控，已分别重新生成并真实 check exit0，结果已并入同一矩阵。重现全部：

```text
cargo build --offline --locked --manifest-path D:/autostack/auto-lang/docs/plans/attachments/814/review-r13/Cargo.toml --target-dir target --bin r10-generate --bin r10-overlay
python D:/autostack/auto-lang/docs/plans/attachments/814/review-r13/replay.py
```

在原 worktree 运行；修复时先复制整个探针目录到新 work 证据目录，保留本冻结快照。`replay.py` 逐项保存真实退出码；进程完成本身不是所有检查通过，必须读取结果矩阵。

四包 cargo check、手写改动 rustfmt、diff whitespace 均 exit0；新解析器改动段没有新增警告，匹配 compiler36/runtime235 与 R12 相同，既有其它告警保留。实际 777 strict exit0，violations0、grandfathered3，基线/窄豁免不变；实际 overlay 八组合 mismatches0。未跑整档 t/tv/tt/tb/tu/taa。

## 完整 T-10 与规范对账

| AC / 任务 | 当前判定及复用边界 |
|---|---|
| AC-01 / T02,T06 | **pass，限定复用** R9 物理无UI源码 SDK consumer fresh+locked5/5、四非UI consumer及 BUILD_INPUTS 证明。R12 后44份冻结源码/配置输入只 RustGenerator 变化；工作树 SDK/依赖/锁/配置及输入未变。 |
| AC-02 / T05 | 旧 API/type pin 随本轮默认正控再次编译通过；r3 inventory边界、Parser/VM单源及provider/feature证据按未变路径复用。**T05仍因所承载生成路径缺陷开放**，不回滚 A 已发布兼容事项。 |
| AC-03 / T04,T05,T08 | **fail，R9-01如上**。T04 runtime迁移保持，本轮overlay八组合通过；R9 counter Vue/VM同源三状态与R11 runtime65定向测试，按未变 runtime/SDK/双端脚本/示例/配置复用。其它三生成器、import策略/模板未改；新 RustGenerator 本轮已实际验证，不能以旧 e2e 代替本次反例。 |
| AC-04 / T06 | **pass，限定复用** provider/frame/i18n/theme/mem_guard44、六provider43及R11 runtime65原结果；状态/宿主与寿命实现未变，当前 strict 再次验证没有逆向实现边。 |
| AC-05 / T07,T08,T09 | 选择/发行所验 **pass**，SD02 **partial**。R6原→A九档strict逐ID、R8 A→B实际挂载路径、R9九档逐ID/R10 t及tu枚举（5328/868 unique）按未变测试声明/别名/配置复用；不认证通用unique-leaf回退验证器。R9冻结792真实driver24/24与runtime consumer5/5，按未变工作树producer/模板/manifest/锁/输入限定复用；新条件产物的失败已独立揭示。777本轮strict绿。 |
| AC-06 / T10 | **needs_fix**，本轮完整审查完成，代码类型不变量未闭，不能授pass。 |
| AC-07 / T11,T12 | **merge pending**；本轮未合入/发布/沉淀canonical或liveledger/归档/清树。 |
| AC-08..10 / T13..15 | 保留R7 r3阶段审定及A实际发布边界，不因B缺陷回滚，也不提前宣称B已交付。 |

SD-01/03/04与A/r3保留规则及未变runtime/SDK实现继续相符；SD-05/06/07事实注记没有新冲突。SD-02诊断/方向调整有实质落地，但上述运算符及bool表达式缺口仍需修复后同步正文。

**并发 canonical 对账**：十份 Spec 与 R12 完全一致，`repository-boundaries/generated-ui-distribution.md` 已由 M816-A 在主库追加新节，当前 blob=`de5e9d4a395f956e7847bcfb5e731d55f35acfde`，旧SD08 before=`3e1beae817cc`。追加内容要求工具经真实进程/协议消费、客户端无静态实现依赖、M816-A不改发行闭包；与本Plan B canonical双直依赖及runtime投影增量无规则冲突。审查已对照当前全文，**后续 SD08/merge 必须以当前版本增补并保留 M815-A/M816-A 章节**，不可用冻结A版覆盖。当前全文、旧→新差异及哈希均冻结；这属于已完成的并发对账与T09备稿版本更新，不另造代码finding。十一现有Spec目标+新增runtime、GOAL-003/007元数据仍正确。

## 下一步

outcome **needs_fix**；status **executing**；重开 T05/T08/T09，T10..12未完成；完成 T01/02/03/04/06/07/13/14/15，**9/15**。next_responsible_role: **work**，同原树修齐完整运算符消费与既有bool表达式类型传播，保留分组、调用边界、shape、声明宽度和纯条件truthy。按同一不变量核对已支持表达式节点与六种比较符的两方向，不只更改这九条产物；相关定向反例/正控通过并提交后再审。

最终pass后由merge新冻结 **裸t+tv+tu+tt** 与必要docs/schema/book，再实际B合入/Spec沉淀和发布收据；主库并集变化须按T11复审。tf仍主检出到期单实例，aavm原用户豁免保持。本轮无实施修复、无整档、无合入或清树。
