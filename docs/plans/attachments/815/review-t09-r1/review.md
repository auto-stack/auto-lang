# PLAN-815 T-09 全计划终审 R1

2026-10-22，**needs_fix：一项 P2，F-815-T09-01**。已披露的shell_pack freshness经实际产物消费验证，属于合同内生成缺陷，不能仅按“预存”豁免。其他已通过协议/真实工具/请求/发行资源/修复证据保留；plan734单独裁定为已有金样排版债，不混为产品失败。

本轮是独立final review，非再一次T07定向审查。Plan不转execution_done/reviewed；重开直接受影响T06/T07及需随生产修复刷新身份的T08，保留A/L/T04与T05完成，计数 **14/17→11/17，executing**；T09未勾，T10/T11 pending。此前联合pass保留为当时的任务收口记录，本次明确保留给T09裁定的债务由真实反例使对应完成标记失效，不推翻未受影响的具体已测结果。

## 冻结、独立性和复用

- 唯一树 `D:/autostack/.wt/lang-815/auto-lang`，branch `codex/plan-815-dev`，HEAD **a67f114fda8abf52630126d4aa52eeacd6d3248a**；生产语义锚 **e22fc75924a7f058a5df8150956b2d66268c5ecd**；Plan r3；M814-B交付祖先93a25c924d1beafab130588bad4d1041dc86e48c。相对T05～08联合审查没有任何源码/依赖/配置改动。
- reviewer=Codex `/root`，延续独立审查上下文；有早期合同/账本参与限制，未参与当期实现。依据完整合同、生产路径、既有原始证据及本轮独立反例裁定；未创建agent/第二实现树、未修生产代码或外仓、未写canonical/live ledger、未合入/归档/清树。
- [baseline](baseline.json.txt)、[终核](final-baseline.json.txt)：同HEAD clean、5943项源码/依赖/配置输入无漂移，七项SD目标及I-PROJECT设计规则均未变。并发auto-graphics提交改变overview及三个graphics规范，已单列旧/新hash；它们不改本次七目标，未将其回滚，也不谎称所有282规范零漂移。
- 复用[四任务证据映射](../review-t05-t08-closeout-r1/task-acceptance-map.json.txt)及各原始收据，已确认适用输入未变；本轮不把旧33/33、377文件或Windows/Linux真实生命周期称为重新执行，不重跑整档。release参考文件仍为1626cec1087e8ba8dec4aede5f24e75ba12245e5b40c40705326ccbfab661696，有限真实消费4/4沿[上一轮probe](../review-t05-t08-closeout-r1/release-probe.json.txt)保留。

## AC逐条终审

| AC | 结果 | 核查与边界 |
|---|---|---|
| AC01 完整UI工具闭包/兼容与唯一owner | **partial** | 新包normal/build/dev清单无auto-man/legacy auto-lang反向大库，Man薄转发/API-gen单算法及历史迁移检查成立；但明确迁入的公开generate_shell_pack_lib(_sources)产物消费失败，见F01。不能用窄协议服务集忽略已登记的原公开生成器。 |
| AC02 I-PROJECT单源/无第二主声明 | **pass（既定支持面）** | 字段×profile/操作来源、相对根、front节点优先、静态/动态投影界线、主pac冲突/移除、sidecar/主题/外部backend及请求PM来源均沿已验证生产缝和反例核对。跨文件source adapter/legacy无请求臂与请求路径有明确区分，不扩张静态scanner。 |
| AC03 真I-TOOL交接/生命周期 | **pass** | 数据owner同源codec、v1预算/framing/错误、控制日志、真实VM/input/cancel/EOF/owned树/readers、串行上下文隔离、capture失败传播和Linux读停边界由完整关闭链及原始证据覆盖；未把私有gradle/hdc/cargo记录器当实际原生设备成功。 |
| AC04 真消费者/兼容/发行/资源/失败及SD | **partial** | 原独立33/33新tool源包fresh/locked/重定位、377资源、协议7/7和raw深度边界、真实dist正负控均保留；shell公开生成物缺双直依赖，且freshness仍红，该新增实证使整体未闭合。777 master并集依法保留T10，不重复当成实现未做的新finding。 |
| AC05 独立最终代码/验收/规范审查 | **未通过** | T09完成本轮审查但有F01，不得勾选任务或标reviewed。格式/既有诊断不被伪写为全库全绿。 |
| AC06 收尾及AC07..12既有A/L阶段 | **pending / 既有范围保持** | A/L历史发布与恢复独立记录保持，未发现本轮漂移；B收尾归T10/T11，本轮未执行。 |

## F-815-T09-01 / P2：shell-pack再生器未交付可消费的canonical依赖清单

**受影响：AC01/04/05；T06完整生成兼容、T07真实消费验证，T08后续源/产物身份。现有规则而非新范围：** Plan §1“既有语义与真实支持不因搬移丢失”、§5.4实际生成工程可用；`contract-baseline.md` §2.1逐符号明确迁入 `generate_shell_pack_lib(_sources)`；已发布 `docs/specs/auto-ui-runtime/project.md` 的生成工程合同要求 `auto-ui-runtime`+`auto-lang-core`双直依赖。无需增加新profile或新运行模型。

### 入口、输入与实际结果

1. 在冻结树运行 `cargo test -p auto-ui-tools --lib test_shell_pack_lib_freshness -- --test-threads=1 --nocapture`：**exit101，0/1**，[日志](shell-freshness.log.txt)。没有跳过，实际读取auto-os/shell五件声明。
2. 私有独立fixture调用真实公开 `auto_ui_tools::rust_ui::generate_shell_pack_lib_sources(&pack)`，生成 **97,143字节** lib.rs 和原Cargo清单；生产源码不改。五个实际声明在运行期间hash稳定，随后冻结为 [frozen-shell-inputs/](frozen-shell-inputs/)，从冻结副本再次生成得到完全相同lib SHA256 **53f876ed29ddf19260cb9310fbb582faade892a789eda358c30814221a26a011**，[记录](shell-generation.json.txt)、[冻结回放日志](shell-frozen-replay.log.txt)。
3. 将原生成清单仅做等价消费布局调整：本地依赖相对路径指向同一producer crate、serde_json.workspace转为同版本族registry声明、显式既有iced_widget patch/独立workspace；**没有补依赖**。实际 `cargo check --offline --manifest-path <probe>/generated-shell/Cargo.toml --target-dir <wt>/target`：**exit101，107个E0433**，涉及auto_lang_core/auto_ui_runtime未声明，[原始日志](generated-shell-check.log.txt)、[诊断摘录](shell-errors-excerpt.txt)。
4. 正控一：当前已入库canonical `cargo check --offline -p shell-pack --lib` **exit0**，[日志](committed-shell-check.log.txt)。正控二：只在私有生成工程的manifest补两项canonical直依赖，保持生成源文件完全不变，检查 **exit0**；结束后恢复私有原manifest，[控制记录](shell-diagnostic-control.json.txt)、[日志](generated-shell-positive-check.log.txt)。未把此诊断实验当成生产修复。

**确认的原因**：当前 `rust_ui.rs:3732`公开生成函数收集的组件已使用canonical SDK（生成物107处canonical引用），但同函数模板 `:3771–3777`及state-access胶水仍发legacy壳；尤其`:3816–3819`的manifest只声明auto-lang，不声明这两项直接依赖。兼容包的传递依赖不能供生成crate直接解析canonical外部crate名。入库canonical库/清单与再生器也不同步，参见[产物差异](shell-lib.diff.txt)、[原生成源](generated-shell-lib.rs.txt)、[原生成清单](generated-shell-Cargo.toml.txt)、[正反控汇总](debt-verdict.json.txt)。问题不是编译环境失败或仅格式差异。

**同因及合法控制**：T07子批2已修app/dev清单的canonical双直依赖，但shell-lib这一明确公开的同族路径未覆盖。原F-815-T07-B2-01对app/dev的关闭结果保留，不将其伪改为全路径都未修。现有入库库和私有两依赖控制均可用；原33/33 app/tool发行不因本反例全部重开。

### 关闭条件与共享回放

- 同步shell公开再生器的真实源/清单与814已发布canonical规则；最终生成工程凭**自身原生生成的**依赖清单通过fresh/locked适用消费检查，不能只补测试工程依赖、改快照去接受错误或让API静默拒绝原支持输入。
- 以本次相同五件冻结输入重放公开生成→真实消费；证明新清单完整、产物与应有入库来源一致，明确live auto-os/frozen输入的身份；freshness有效，不靠skip/ignore/放宽字节断言掩盖差异。私有fixture已证明可只读外仓、在唯一树target验证，不需要向auto-os写代码或另建实现树。
- 复验受影响生成兼容/必要邻接正控，并刷新T08实际生产冻结位/参考artifact来源；提交新冻结SHA后仅对受影响内容复审，未变协议、生命周期、发行/资源证据可保留。

现成fixture位置在[probe-root.txt](probe-root.txt)，源码/清单分别为[debt-probe.rs](debt-probe.rs)、[fixture manifest](debt-probe-Cargo.toml.txt)。在冻结树运行 `cargo run --offline --manifest-path <probe>/Cargo.toml --target-dir <wt>/target -- <probe>/generated-shell`（只读本报告冻结五件输入），随后对原样生成清单运行上述cargo check即可重放。若target被清理，用所存两文件在新的target私有目录重建fixture即可；不要重生成到入库crates/shell-pack来冒充review修复。

## plan734债务裁定（与F01分开）

`cargo test -p auto-api-gen --lib test_plan734_tauri_fixture_matches_generator -- --test-threads=1`实际 **exit101，0/1**，[日志](plan734.log.txt)。差异仅 `tauri::generate_handler![echo,]` 与同一宏的多行排版；两份文本分别经rustfmt解析格式化后**完全相同**，[差异](plan734.diff.txt)、[分类记录](plan734-classification.json.txt)。永久精确断言没有被改宽或报绿。

`git blame/log`将当前单行fixture追至已发布814的 **2aa0529cae**；815原T04修复eea682150及R2确曾通过，后续814修改再次造成布局漂移，[历史](plan734-history.txt)。因此本轮将其保留为auto-api-gen/814来源的**非阻断精确金样卫生债**：本次未观察到命令API或生成行为语义差异，不以它单独判定AC01..04产品失败，也不把它与shell真实107编译错误等同。后续必要包门出现该红仍须如实报告和归因，不能声称全部测试绿或因此豁免新的功能失败。本轮未宣称实际完整Tauri原生构建通过。

## SD01..07与收尾裁定

[逐项Spec终核](spec-delta-reviewed.md)冻结当前拟沉淀文本及七目标hash来源。SD02/03/04/06/07的已审事实可保留；SD01/05所承诺完整工具与公开SDK消费需先闭合F01，生产修复后SD06身份须随新冻结绑定。A阶段已存在的tools/protocol文件在B中是受管增补，不能新建覆盖A事实；未改canonical或live ledger。

**next=原唯一树work修F01，一次提交完整修复与必要定向证据，刷新T08后送受影响T09复审。** 本轮T06/T07/T08完成标记重开，T09保持未完成；Plan executing11/17。T10最终冻结裸cargo t+UI触面tu+777 master并集/串行合入、T11沉淀收据归档清树仍pending，未运行整档或提前合入。
