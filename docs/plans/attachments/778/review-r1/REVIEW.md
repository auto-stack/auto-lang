# PLAN-778 R1 独立 final review

结论：**needs_fix**。外部consumer基本探针可用，但嵌套注册和错误上下文两个合同内不变量仍未闭合；新增文件格式门也未通过。Plan返回executing / 1/7，不merge、不发布canonical Spec/ledger、不清理树。

## 冻结审面和独立性

- plan_revision=1；review_scope=final；worktree `D:/autostack/.wt/lang-778/auto-lang` / branch `plan-778-dev`。
- reviewed_commit=`40c69c890eb2d52854ad846b94846fdce7e498a5`；base=`87f45dc4e2c592c31844e74824e67bf9e6cb419f`；主检出=`075a9b4ab6cb4284f3c259b348f2571322107fec`。
- auto-down组内兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。受管输入完整hash、root/fixture锁hash、HEAD、branch、status及判定前复查见 `fingerprints.json`。
- 本评审会话参与拆分规划及777审查，未参与778实现；从提交及独立输入重建，没有另建agent/树。
- 当前frontend overview源SHA256=`fccb811a79b24b95678efa57b87bcc9ded10e5afd2b9abc44236b99aebb16dfe`；SD01冻结见 `spec-delta-reviewed.md`。目标方向与现有Spec/Design37一致，但恢复保障未成立，不能提前沉淀。
- reviewed实现已提交且树干净。独立测试只放在该树被忽略的scratch包，产物在该树target；真实实施文件未修改。测试源码/原始结果已复制到本目录，可在树清理后查阅并重放。

## Findings

### F778-R1-01 / P1 / blocking：活跃方言在委托解析期间从注册表消失

AC-01/02；T-02/T-03及T-04/T-05依赖证据。

parser.rs第724行先remove当前方言，第727行调用方言后才在729行insert。外部方言通过bridge委托核心表达式解析时，表达式中的闭包body再次走同一Parser语句派发，此时当前方言不在表中。token派发第754–760行有相同模式。返回后注册恢复不代表递归期间注册可见。

独立反例直接使用已交付的ExternalModelDialect：

```auto
model { child = () => { model { count int = 1 } } }
```

真实结果解析成功、AST内有两层ModelBlock，但probe attempts=1/accepts=1；预期两层都由已注册外部model方言接管，accepts=2。内层实际上落回内建UiDialect。这是冻结子集允许的core表达式初值和嵌套core body组合，不是已明确排除的widget体内部结构，也不是新增UI文法。

兄弟控制 `{ model { count int = 1 } }\nmodel { count int = 2 }` 同一外部方言accepts=2通过；区别只在派发回调是否还活跃。当前12项测试仅覆盖从普通外层进入嵌套body，不能证明外部回调委托期间的传播。

修复：保持活跃方言在合法递归派发时可用（例如以可共享的不可变实现句柄消除对表的暂时移除，具体等价机制由work决定），保留优先级、matches场景门控、实例隔离、Err后注册和None回滚。补Ident/token路径的委托递归组合，不能把该输入默默改列未迁范围或只证明返回后表恢复。

### F778-R1-02 / P2 / blocking：委托解析Err后函数体上下文未恢复

原§5明确要求Err恢复registry/context，AC-02/V04；T-02/T-03及后置验收证据。

外部ContextDialect只使用公开桥面：消费`probe`后调用`bridge.parse_expr()`，`tail`是返回合法Int语句的恢复控制。输入：

```auto
probe () => { fn broken() {
tail
```

第一次parse_stmt真实返回缺RBrace错误。测试使用公开checkpoint恢复游标并advance到确切tail，随后同一Parser成功派发tail；回调观察的in_fn_body却为`[false,true]`，预期`[false,false]`。成功委托控制 `probe () => { fn valid() { } }\ntail` 观察值为`[false,false]`，通过。恢复点由harness显式建立，不依赖文件整体parse的synchronize或要求该坏文件成功。

根源：core函数解析在parser.rs第9393行设in_fn_body=true，9394行body()?提前退出，9395行恢复未执行；新派发器第733/764行仅把方言放回后返回Err，bridge checkpoint也只恢复lexer/cur/prev，不能修复残留context。底层异常出口问题虽已有，新桥面明确要求的Err上下文保障仍未实现；记录为原合同缺口，不以“既有恢复行为”延期为已完成。

修复：在本桥面/派发委托错误边界保证已声明解析context恢复，并核查实际委托路径的其他context/scope状态；补正常/错误/明确恢复点兄弟控制。保留diagnostics单调追加的已声明限制；游标恢复不能代替上下文恢复，不需扩展成全编译器事务系统。

### F778-R1-03 / P3 / closeout-required：新增文件未满足格式健康检查

独立 `rustfmt --check --edition 2021` 检查新增parser_bridge.rs、fixture src/lib.rs与tests/v01_v05_external_dialect.rs，exit1。明确差异在bridge第89行及外部测试文件多处，见 `fmt-new-files.log`；不是把全仓旧格式债务归本Plan。按AGENTS独立复审健康门，修复提交需格式化新增/修改代码，再复查受影响部分。

## AC/task与验证结果

| AC | 结论 | 证据 |
|---|---|---|
| AC-01 | pass（已冻结model基本外部面） | 原consumer12/12；typed count/int/1和真实accept、私有escape hatch仅pub(crate)，未公开Parser内部数据 |
| AC-02 | fail | 同实例/session/优先级/普通嵌套/简单Err控制通过；独立递归与委托Err上下文反例失败 |
| AC-03 | partial | 定向兼容255/255通过，未迁清单诚实；上述组合未满足，无全面兼容pass声明；整档门禁merge-owned pending |
| AC-04 | 审计完成，未pass | 本报告、原反例/组合、源指纹、Specdelta及格式检查 |
| AC-05 | pending | merge-owned沉淀/合流/归档/guard-clean，未执行 |

独立定向验证：

- 外部fixture `cargo test --manifest-path test/fixtures/modularity/ui-dialect-consumer/Cargo.toml --locked --test v01_v05_external_dialect`：12/12通过。
- 新crate内test_plan778四项通过；合并定向过滤parser::tests/dialect/model/widget（`--lib --features ui-iced,test-cases`）：255/255通过，包含四项新回归与grid/store等兼容控制。
- `cargo check -p auto-lang --locked` exit0；编译输出仍有大量已有库/测试警告（shim-metadata、lib宏注释、HTTP/UI等），不能称全仓零warning；未在新增bridge/fixture代码发现新警告。日志完整留存，修复不得顺手在master清理旧警告。
- 独立组合 **6项：4pass/2fail**，真实cargo test exit101。通过的token None回滚、token Err后注册、成功委托context和普通嵌套正控排除了“一律不工作”的解释。
- `git diff --check`通过；新增文件rustfmt check未通过。受管源输入前后不变，实施树干净；独立scratch/构建只在该树内。

未运行裸cargo t/tv/tt/tu/taa/tf整档；本轮仅定向验收。将来最终review pass后merge仍需冻结提交上的裸cargo t+tv，当前没有触UI codegen/trans/aavm，不追加tu/tt/taa。批回归到期按merge规则判断。

## Spec与范围审查

SD01可记录真实bridge/注册接口、外部model子集和未迁边界；“嵌套/Err上下文可靠保持”暂不成立。规范源未发布、live ledger未变。touched_goals=[GOAL-007]，supersedes_spec_components=[docs/specs/auto-lang/frontend/overview.md]，new_spec_components=[]。

with_session保留外部表、场景matches门控已证明；生产18个入口、导入新Parser不自动继承、widget体直连内建已明确列范围，不假称完整UI方言迁移。本轮不把这些已声明边界擅自扩大为迁移要求。嵌套闭包body与委托Err属于已承诺路径，不能被该未迁清单覆盖。

## 重放与下一步

原始输入代码 `independent.rs`；最终6项原始结果 `replay-latest.log`；第一次5项结果保留在 `independent.log`；root定向/check/外部gate命令与退出码见 `runs.jsonl`。重放：

```text
python docs/plans/attachments/778/review-r1/replay.py --implementation D:/autostack/.wt/lang-778/auto-lang
```

脚本只物化被忽略的scratch消费包，代码/依赖路径指向该唯一实施树；cargo整体失败会向上非零传播。修复后须按全部6项断言核查，不只看banner。

保留已完成的T-01调查/冻结记录；T-02..T-05因实现和依赖验收证据重开，executing 1/7，r1合同不变。work在同树修复全部findings、按等价机制更新冻结说明、补永久组合控制、提交后再次独立review；本评审不修实现、不合入/归档/清理树，不触他线780/782 WIP。
