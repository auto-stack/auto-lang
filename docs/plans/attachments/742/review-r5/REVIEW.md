# PLAN-742 第五次独立 final review

2026-10-14；**needs_fix**。F-742-R4-01 的原反例与控制闭合，旧20个输入全部实编运行通过；仍有一个普通比较实参与字符串借用组合的稳定错误 F-742-R5-01。S10、Vue当前后端复跑通过，VM交互边界保留。

## 基线、身份与连续失败诊断

- plan_revision=1；审面 `2174f0de28dd786369b2e44e94bfa1a1ac806499`，基面 `2c40e1fe4673c2041ae3c366037d7eab38017d81`。
- 同一 `D:/autostack/.wt/lang-742/auto-lang`，branch `plan-742-dev`，实现已提交、源码干净。本次diff只有api_gen.rs；无review专用checkout、无实现修补/合并。
- Codex /root沿用前轮审核上下文，未参与修复，无子agent；从实际制品独立重建结论。auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，Notes `1391bd7eb2f6b3b08e9841d30b25125e47ced058`。
- 主检出入口 `799eac6fb96cacaf1f09467377baf278de4ce95e`；Plan入口冻结 [plan-input.md](plan-input.md)，代码/配置/依赖/规范/制品指纹在 [baseline.json](baseline.json)。主检出原有其他Plan文档WIP与Notes原有 `src/front/tests/` 保留。
- 已审核 §9.3/work-r5 的四轴诊断：新左标识符边界有效，并有永久fn单测+nsfx实编oracle；但“全部扫描器已核对完整token身份/遍历”不足以证明表达式参数拆分。split_top_level_args把比较操作符和类型角括号混为一类，完整签名借用仍未闭合。

## F-742-R5-01：比较实参被误作泛型，后续字符串漏借用

**P2 / blocking，AC-742-02、04，T-1/T-2，连带T-3修复版本证据。既有模块调用与参数借用要求的剩余缺口；本轮改动没改此helper，不归为r5新回归。**

`split_top_level_args`（api_gen.rs:2792）遇任意 `<` 都增angles，直到`>`才减。`borrow_pass`在3077用它拆调用实参，因此 `1 < 2, s` 被视为一个参数：第二个参数的str签名不再对应index1，漏补 `&s`。

[compare_less/api.at](cases/compare_less/api.at)：

```auto
pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var s str = "ok"
    var n int = repository.choose(1 < 2, s)
    if n == 2 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
```

同目录companion声明 `choose(flag bool, value str) int`；真实生成 [repository.rs](cases/compare_less/repository.rs) 为 `choose(flag: bool, value: &str) -> i64`，而 [api.rs](cases/compare_less/api.rs) 发射 `crate::repository::choose(1 < 2, s)`。public generate_api 返回Ok；[compile.log](cases/compare_less/compile.log) 实报 **E0308 expected &str, found String**，并建议 `&s`。

同输入只把布尔实参换成 `true` 的 [compare_control](cases/compare_control/api.at) 和 `2 > 1` 的 [compare_greater](cases/compare_greater/api.at) 均补 `&s`、实编运行返回 `"ok"`。这组三例的wrapper直接include真实生成的repository.rs，未伪造字符串参数签名；对照证明不是API类型、数字比较支持或伴生模块声明失败。

修复调用实参拆分，区分表达式比较符与真实泛型/类型定界；保持真实参数位置、嵌套与签名借用。该helper也用于声明参数解析，修复不能破坏原类型参数分隔。加入永久生成→实编→返回值控制，保留前轮嵌套与模块名回归。此组合是现有布尔表达式+普通str的companion调用，没有新增语法、规模要求或类型支持承诺。

## 原反例、当前矩阵与验收映射

[old-case-closure.json](old-case-closure.json) 核对旧20例的api.at/repository.at SHA256及实际返回值，namespace_suffix/control另核对真实custom companion源hash，全部原样通过。F-742-R4-01具体反例关闭；R3 marker/嵌套、R2 File/Option/void/遮蔽、R1字面量/原返回形态都保持既有通过事实。

独立选取4个新增组合：小于比较/大于比较/布尔常量的多实参调用，加多行字符串（包含标准库、模块调用、List同形文本）。当前 [generated-results.json](generated-results.json) **24个有效用例，23实编运行/值对拍通过，1编译失败**。多行字符串正确转成转义形式、返回值保持。原历史fixture本轮生成Ok；目标测试有既有实编oracle，不用单纯生成日志冒称全编译。

复现：本包 `replay.ps1 -Build` → `execute-generated.ps1`，在既有实施树执行。Cargo JSON唯一dev-profile auto_man rlib及同轮native link paths构建外部probe，调用真实public generate_api；执行脚本编译实际handler、poll并读取返回值。所有源/生成物/编译日志可留存重放。其作用域是handler发射与指定companion，不冒称全HTTP workspace服务覆盖。

| AC | 任务 / 裁决 | 实际证据与边界 |
|---|---|---|
| AC-742-01 构造/业务值保持 | T-1/T-2 pass（已验形态） | 旧字面量/构造、marker及新增多行字符串实编运行保持 |
| AC-742-02 模块调用/借用 | T-1/T-2 **fail** | 模块名/遮蔽/嵌套旧例通过；R5-01多实参比较导致漏借用 |
| AC-742-03 原比较归因/oracle | T-1 pass（原可考范围） | 单==原oracle继续通过；旧WIP仅有类别级归因，不伪造源码行。本轮混合比较反例记入AC02/04 |
| AC-742-04 handler可编译/返回 | T-1/T-2 **fail** | 原Option/void/File/Result已验形态通过；当前合法体E0308；Upload仍源码级接线边界 |
| AC-742-05 Notes battery/legacy/联动 | T-3 partial | work当前v1电池fails0；本轮独立S10、Vue13/13；通用修复仍未闭合，VM交互不冒签 |
| AC-742-06 D7～D9运行时/主题 | T-2/T-3 partial | 历史Rust控制对应源hash不变；当前Vue主题与S10通过；VM未验 |

## 本轮实际门禁与Notes

| 命令/检查 | 本轮结果 |
|---|---|
| `cargo test -p auto-man --lib api_gen` | **48 pass、1 ignored**，3.45s；[api-gen.log](api-gen.log) |
| `cargo build -p auto-man --message-format=json` | exit0；[制品索引](auto-man-build.jsonl) |
| `cargo t --no-fail-fast` | **5278/5278 pass**，143.280s；[daily.log](daily.log) |
| `cargo tt --no-fail-fast` | **5648 pass / 1 timeout**，5649 run，189.817s；[trans.log](trans.log) |
| 超时用例定向 `cargo tt plan707_wait_generator_park_drive_count_static` | **1/1 pass**，1.148s；[trans-retry.log](trans-retry.log) |
| 两个触面文件rustfmt check；base→HEAD crates whitespace | exit0；[health-exits.json](health-exits.json) |
| 独立当前后端S10 | pass；[s10-independent.json](s10-independent.json) |
| Notes现有Playwright smoke | **13/13 pass**，1.7m；[vue-smoke.log](vue-smoke.log) / [身份](vue-identity.json) |

当前CLI SHA256 `4af3e449303460b87a93add15673e775f698f96a6f5baffdf62798dc5e022daf` 与work提交态一致；本轮实际后端 `6b4bcb6a7c4d10c5246b625a50c285f07e0b8cdb68f4c55176a2b55e5d1a17c9`。Notes生成workspace runtime依赖指向主检出auto-lang/a2r-std，未称其为worktree runtime build；本轮测试绑定实际后端hash。S10独立驱动记录创建→旧PID强杀→拒绝连接→同二进制同目录新PID重启→同note_id/revision/title/body/state读回；保存DB路径与前后hash，不要求恢复不写文件。只清理本轮拥有的进程。

收口发现Notes有并行未提交的repository.at、notes_store.at及文档变化，HEAD仍1391bd7；[notes-drift.json](notes-drift.json) 保存状态/当前源hash，后端二进制hash核对未变。没有实施这些修改，也没有回滚它们。上述S10/Vue是实际制品的运行结果，不能绑定HEAD冒称当前Notes源码工作区完整通过；AC05/06仍partial。下一轮如要求当前应用源码验收，须重新核对已提交依赖与生成输入。742审面/配置/规范保持稳定，反例不受此次Notes源码漂移影响。

Vue使用原smoke.spec.ts和既有生成前端，包含主题根dark class、搜索筛选、保存、autosave、新建/删除及console；未改业务测试。S10/Vue脚本在本包可重放。VM沿用r4标准驱动45秒未ready的未验边界，本次未重新运行，不判产品故障、不新增固定19场景或视觉全等门禁。work-v5电池摘要有S10外部驱动提示，本次独立过程另列，不把摘要当强杀证明。

trans首跑超时是plan707_wait_generator_park_drive_count_static，r1已有同款120秒超时/定向绿记录，相关实现未改；本轮定向重跑通过，不把首跑整档改称全绿，也不据此归为742回归。所有原始失败日志保留。

未运行tf（批量档）、aavm/docs_gen（零触面）。本轮没有UI生成器改动，历史tu862 pass/2 ignored与D7/D9 Rust控制保留其旧来源，源码hash不变但不冒称本轮重跑。无新增742 dead-code/debug print，存量warning不称为仓库零告警。

## 规范增量与路由

[冻结spec-delta](spec-delta.md) SHA256 **9691E5B5BA2D146ECE88E16765135EB2BCEBFBBE11DFD2AF6F7D340EA6B0A675**；五个规范目标与GOAL-003/007不变。SD-742-01/02仍partial；不以合格旧输入证明完整parity。没有修改canonical/live ledger，没有缩小既有可编译/业务值合同。D-742-1继续另案；db旧regex保留候选身份，未独立复现；Upload仍原源码覆盖边界。

退回executing；T-1/T-2与T-3依赖版本证据重开，0/3，当前独立S10/Vue及旧例通过事实全部保留。第五次final未通过，历史连续失败账不重置。下一次work在同树修复R5-01，更新表达式/类型token区分与实参位置诊断及永久oracle，提交后重新交接。merge-owned规范沉淀、批量到期判定、归档/清理仍pending。
