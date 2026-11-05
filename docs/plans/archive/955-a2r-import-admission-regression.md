---
plan_id: PLAN-955
status: archived
feature_name: A2R import admission regression
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 2
current_step: 3
total_steps: 4
supersedes_spec_components: [docs/specs/auto-lang/trans/rust-semantic-admission.md]
new_spec_components: []
touched_goals: []
---
# PLAN-955 A2R 跨模块调用准入回归
## 0. 变更摘要
修复 auto-reader library.at 经 auto-ui/vue API assembly 转译失败，保留静态准入负例并改善诊断。
## 1. 目标
AC-01(work): 恢复报告期望的 Vue 可用入口：按既有 README 选择 --server vm，真实启动与 API 返回200，并验证前端生产构建。报告对跨模块回归的判断须以实际符号取证更正。
AC-02(work): 真实 API 组装入口的显式伴生模块导入通过；未知调用仍拒绝，错误显示符号与模块。不是新增 Rust 动态运行时移植。
AC-03(work): 核查 #[test] 模块可见性现状，文档化准确边界。
AC-04(review): 冻结 diff 独立复核，无未授权语义放宽；merge 负责职责组并集及合入收尾。
## 2. 架构方案
保持 parser→静态准入→CTE→Rust 发射管线；先以真实输入查明丢失的可见域，按实际根因修复诊断；VM-only 能力不伪装已支持 Rust。
## 3. 技术栈
Rust / auto-lang-core / auto-generation-core；auto-ui、auto-web 为消费者。
## 4. 需求分析与背景调查
用户 /goal 明确授权解决 reader-008 报告，包含修复与诊断及测试语义说明；无额外预算或模型约束。源码 owner 实际在语言仓，允许依赖修复属于目标所需范围。
依据：docs/specs/overview.md、docs/specs/auto-lang/trans/overview.md、rust-semantic-admission.md（PLAN-916）。现规范要求合法 import 保持。
原报告 D:/autostack/plan-artifacts/reader-008/auto-ui-transpile-regression-report.md。library.at 明确 use pathx: json_escape，不能以缺少 use 为前提。
基线 auto-lang 08df260df8；两仓现有他方 docs WIP 不纳本包。唯一实现树 D:/autostack/.wt/lang-955/auto-lang，branch codex/plan-955-dev。材料统一 D:/autostack/plan-artifacts/plan-955。
## 5. 详细设计
T-01 调查：真实 library.at typed 错误与最小导入对照，定位符号可见域缺口；等价机制由实现阶段确定。
T-02 修复：精确错误信息与永久导入正反例，保持原准入；若 Vue 构建暴露产品小缺陷，在同组 reader 唯一树修复并单独落地。
### 规范增量
| ID | 操作 | target | before/after | 理由 | AC |
| SD-01 | modify | docs/specs/auto-lang/trans/rust-semantic-admission.md | 合法 import 规则不变，补实际边界、诊断及 test 可见性说明 | 消费者回归 | AC-02/03 |
## 6. 测试设计
不变量：合法跨模块引用可发射，普通未知符号继续拒绝；不得以关闭准入解决。
V-01: 真实 reader Vue+VM 启动和 HTTP 200；生产构建用与 run 相同的既有 lenient custom-tag 模式（doc-surface schema 已有警告，不改变准入规则），oracle=vue-tsc/vite 退出0及产物存在；不等同UI走查。
V-02: API 生产包装器显式跨模块导入与 unknown_helper/type_of 对照；前者发射真实 crate import/call，后者输出符号和 library.at。不修改跨模块绑定算法。
V-03: 正式职责组 auto-diagnostics、auto-generation-core；A2R 原准入反例定向保护，因未改准入算法不扩大至全语言/ACE。
实施与最终复核顺序在同一树进行；未指定外部 reviewer，不冒称独立 agent。最终职责并集 merge 负责。
## 7. 验收标准
见 AC-01~04；尚未执行，不能以测试设计代替结果。
## 8. 执行步骤
- [x] T-01(work) 复现与最小用例，关闭影响域调查。
- [x] T-02(work) 修复/回归/SD-01；AC-01~03。
- [x] T-03(review) 冻结提交逐项审查；AC-04。
- [ ] T-04(merge) 职责组、合入、归档与安全清理。
## 9. 复审记录
2026-11-06 stage:new | r1 | pass | next:work；沿用户原目标授权继续，不增加审批。
## 10. 待澄清事项
无；真实失败符号与实现机制由 T-01 取证，不预设新导入约定。

2026-11-06 T-01/r2 证据纠正：真实 full-options 入口 first error = Name(UndefinedFunction { name: "type_of", span:15094:1 })；6组具名 import AST完整。VM native http_client.rs register_json_http_helpers_shims 注册type_of，A2R无裸名映射。CLI --help 与 run_vue.rs 明定缺省 Rust，--no-merge 不切server；reader README 早已有 --server=vm 及 Rust 动态JSON缺口。现行原二进制 --server vm 已真实服务42 routes、17834/17836三端HTTP200；故不沿错误前提发明跨模块新规则/无边界承诺Rust全功能移植。r1 AC01 的“全部back必须转译”是调查前机制假设，r2以用户期望的Vue可用性和现规范更正；不取消用户指定验收或阈值。
reader唯一树 D:/autostack/.wt/lang-955/auto-reader，branch codex/reader-vue-build-repair，base b656e6a；前端源码缺chapters_label引发vue-tsc TS2339，补model字段与demo章节数文本，无框架语义改动。

## 验证与交接（r2）
- 语言实现提交 25677cf48 → 78630ac871f4961b092f8fbb991334cd2f34a378，rebase仅带入他方2份计划簿记，git range-diff全等（1:=1:）；代码/配置/规范目标未漂移。reader提交4a2c748e51d754ecd700c97beffcc56522a7f5e9。
- `cargo t auto-diagnostics auto-generation-core`：两个完整职责组PASS；diagnostics 1个golden重放（38诊断载荷），generation 30测试，含本包导入/诊断两个新反例。code/span/help/debug保持，golden仅批准display/rendered加符号。
- `cargo t a2r --test-name trans::rust::semantics --out .../semantic-regression`：测试体PASS，包装总gate因运行期间commit/rebase的source identity变化为false（明确不是最终门通过）；需在冻结HEAD重放。未改准入算法。
- 现行auto.exe + auto-ui-tool：`run --no-merge --server vm -B 17836 -F 17834`起Vue/42 routes；GET前端、/api/books、/api/library/books均200，未执行写API。初次使用既有用户数据只读查询，后续验证使用独立数据目录。
- `pnpm run build` 首次失败 TS2339 chapters_label，reader补model字段+Init赋值后：`AUTO_BACKEND_IMPL=vm auto build -r vue --lenient`退出0，vue-tsc通过、2183 modules transformed、vite11.60s，dist/index.html及book_detail.js存在。strict build仍因既有自定义doc-surface未注册schema拒绝；未修改schema/strict规则，无“strict全绿”声明。
- #[test] CLI反例：helper.at导出expect，bare.at仅use helper而在未调用test函数引用expect→exit1/Undefined symbol expect；imported.at改为use helper: expect→exit0。最初探针cwd错误/println未导入已纠正为无输出空main的有效对照，不作产品缺陷。
- rustfmt修改文件检查、git diff --check通过；新代码未增加warning，Core已有131条warning按编译日志实录，不称零warning。
stage:work | PLAN-955 r2 | pass | worktree:lang-955/auto-lang + auto-reader | AC01/02/03完成 | blockers:none | next:final review；完整职责并集与冻结语义反例重放pending merge。

2026-11-06 stage:review | review_scope:final | reviewer:当前实现会话，重新读取冻结diff/源码/原始结果，不冒称独立agent/context | PLAN-955 r2 | outcome:pass | reviewed_commit:78630ac871f4961b092f8fbb991334cd2f34a378 | base:fac797755912748423053388d87a01ae3ce941d5 | reader:4a2c748e51d754ecd700c97beffcc56522a7f5e9 | 两实施树clean。
AC01=PASS:原二进制现有可用VM选择路径42routes与HTTP200，reader两行model修正后vue-tsc/vite真构建通过；严格schema警告如实保留，不把它判作本包已解。AC02=PASS:生产语义只有NameError Display一行，未触binder/VM/A2R准入，既有导入不受影响；永久反例覆盖具名调用与无suggestion诊断。AC03=PASS:Parser is_test持久化、codegen函数处理/loader符号域、真实CLI bare1/imported0三方一致；不声称标记语义在某日期改变。AC04=PASS:逐文件/AC/遗漏扫描无在约定范围未解决项；新测试与改动对应，不用关闭检查换绿。reader源码model与生成模板同名、demo分支在has_book之前赋值，真实书progress_label保持。
SD01 canonical path docs/specs/auto-lang/trans/rust-semantic-admission.md hash698a7da56572a2479a3891630e14b7fa976f5811ec9a1f477ad13f1115a96596；无legacy ledger写入。所有框架运行时算法和默认选择保持；变更仅批准的人可读错误格式。pending merge-owned gates:冻结auto-diagnostics+auto-generation-core完整组，6个原准入测试冻结重放；不堆叠全语言门。next:merge。

2026-11-06 stage:merge | PLAN-955 r2 | delivered+canonical_verified，cleanup pending | language landed78630ac871f4961b092f8fbb991334cd2f34a378、reader landed4a2c748e51d754ecd700c97beffcc56522a7f5e9（两仓ff-only，主线tip核对一致） | SD01主线hash698a7da56572a2479a3891630e14b7fa976f5811ec9a1f477ad13f1115a96596。
冻结职责组 `cargo t auto-diagnostics auto-generation-core --out .../frozen-gate` 完整31测试、gate_passed=true；`cargo t a2r --test-name trans::rust::semantics --out .../frozen-semantic` 6测试、gate_passed=true，source/config无漂移。主检出reader重建AUTO_BACKEND_IMPL=vm auto build -r vue --lenient退出0，vite4.75s，产物gen/front/vue/dist已刷新。
原reader-008报告增加owner更正说明，历史正文保留；不修改他方主线WIP。
清理审查：language wt-guard clean；reader guard检测pnpm及auto deps物化链接，故禁止直接worktree remove。自动审批两次拒绝在已确认lang-955/auto-reader范围内删除生成链接（扫描删除和显式单链接删除均blocked by policy，无进一步理由）；未绕过，链接/两实施树/材料目录保留，由本Plan owner持有，关闭条件=允许正常安全链接清理后fresh guard clean再移除树与scope。T04不伪勾全完成。
生产artifact观察：reader bundle已刷新；UI owner target/debug/auto-ui-tool.exe正在重建，邻接auto-man/target/debug/auto-ui-tool.exe原构建UTC2026-11-04T19:04:10（本地11-05 03:04），非首选owner副本，尚未刷新；release二进制未请求且未构建，不称全发布。
批量due检查：receipt last_covered_plan_id953、timestamp2026-11-05T20:24:10Z；955为整五触发。当前另有lang936完整review并集、lang944定向门、UI runtime门运行，regress技能要求无并行重门后在main单实例运行，因此未启动第二批且未推进last-batch receipt。该全局维护门不作为本包已通过；记录待队列空闲后运行。

2026-11-06 production artifact复核完成：`cargo build -p auto-ui-tools --bin auto-ui-tool`在UI owner重建debug工具，退出0/39.27s，文件D:/autostack/auto-ui/target/debug/auto-ui-tool.exe SHA256=2b96cca7563ae665357288fc5d153c06776e3e14417c93a58e6c335eed712bdc，UTC2026-11-05T20:49:04（本地11-06 04:49）。该条替代前述“正在重建”状态；未改变UI源码，未触他方WIP。
真实CLI发现此新工具后重放原命令，产出 `API assembly rejected back module library: Failed to transpile library.at: undefined function: type_of`，符号+模块已到用户可见面；未伪称默认Rust后端已可运行。再以 `AUTO_READER_DATA=D:/autostack/plan-artifacts/plan-955/runtime-data` 隔离数据，`run --no-merge --server vm -B 17836 -F 17834`：42 routes、Vue ready；GET / 200/4791bytes，/api/books 200/566bytes，/api/library/books 200/30bytes（空隔离库）。自建PID25424/工具26452及Vite子树已终止，17834/17836无监听；最初递归找bash的只读辅助进程13408亦已停止，不遗留运行任务。
代码目标与可用路径交付已完成；仅工作流清理T04（删除策略阻止）及独立全局due批量维护保持待处理，不将其报告为通过。材料持有已写真实全局registry D:/autostack/plan-artifacts/.retention/policy.json scopes[auto-lang/plan-955]；初期误写语言仓本地registry的本包scope已移除。两worktree保留，不绕过拒绝；来源均为本计划并已具名。原reader-008材料及他方主线文档/程序集WIP原样保留。
