---
plan_id: PLAN-955
status: executing
feature_name: A2R import admission regression
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 1
current_step: 0
total_steps: 4
supersedes_spec_components: [docs/specs/auto-lang/trans/rust-semantic-admission.md]
new_spec_components: []
touched_goals: []
---
# PLAN-955 A2R 跨模块调用准入回归
## 0. 变更摘要
修复 auto-reader library.at 经 auto-ui/vue API assembly 转译失败，保留静态准入负例并改善诊断。
## 1. 目标
AC-01(work): 真实 reader 全部 back 模块通过 A2R/API 组装，Vue 构建成功。
AC-02(work): 合法显式/限定/传递导入不误拒，未声明本地调用仍 typed 拒绝；失败信息包含符号与模块。
AC-03(work): 核查 #[test] 模块可见性现状，文档化准确边界。
AC-04(review): 冻结 diff 独立复核，无未授权语义放宽；merge 负责职责组并集及合入收尾。
## 2. 架构方案
保持 parser→静态准入→CTE→Rust 发射管线；先以真实输入查明丢失的可见域，再修复共享生产者，不在产品端添加绕过。
## 3. 技术栈
Rust / auto-lang-core / auto-generation-core；auto-ui、auto-web 为消费者。
## 4. 需求分析与背景调查
用户 /goal 明确授权解决 reader-008 报告，包含修复与诊断及测试语义说明；无额外预算或模型约束。源码 owner 实际在语言仓，允许依赖修复属于目标所需范围。
依据：docs/specs/overview.md、docs/specs/auto-lang/trans/overview.md、rust-semantic-admission.md（PLAN-916）。现规范要求合法 import 保持。
原报告 D:/autostack/plan-artifacts/reader-008/auto-ui-transpile-regression-report.md。library.at 明确 use pathx: json_escape，不能以缺少 use 为前提。
基线 auto-lang 08df260df8；两仓现有他方 docs WIP 不纳本包。唯一实现树 D:/autostack/.wt/lang-955/auto-lang，branch codex/plan-955-dev。材料统一 D:/autostack/plan-artifacts/plan-955。
## 5. 详细设计
T-01 调查：真实 library.at typed 错误与最小导入对照，定位符号可见域缺口；等价机制由实现阶段确定。
T-02 修复：共享准入可见性与精确错误信息；永久正反例保护，保留原七族准入。
### 规范增量
| ID | 操作 | target | before/after | 理由 | AC |
| SD-01 | modify | docs/specs/auto-lang/trans/rust-semantic-admission.md | 合法 import 规则不变，补实际边界、诊断及 test 可见性说明 | 消费者回归 | AC-02/03 |
## 6. 测试设计
不变量：合法跨模块引用可发射，普通未知符号继续拒绝；不得以关闭准入解决。
V-01: 真实 reader back/API→Vue 构建，oracle=构建退出0及产物存在；不等同UI走查。
V-02: 单/全选项/项目入口的具名导入、限定调用、传递重导出对照；unknown 负例=NameError，输出明确名称。
V-03: 原静态准入反例与 a2r 职责组保持；诊断改动另跑 auto-diagnostics，生成层跑 auto-generation-core。
实施与最终复核顺序在同一树进行；未指定外部 reviewer，不冒称独立 agent。最终职责并集 merge 负责。
## 7. 验收标准
见 AC-01~04；尚未执行，不能以测试设计代替结果。
## 8. 执行步骤
- [ ] T-01(work) 复现与最小用例，关闭影响域调查。
- [ ] T-02(work) 修复/回归/SD-01；AC-01~03。
- [ ] T-03(review) 冻结提交逐项审查；AC-04。
- [ ] T-04(merge) 职责组、合入、归档与安全清理。
## 9. 复审记录
2026-11-06 stage:new | r1 | pass | next:work；沿用户原目标授权继续，不增加审批。
## 10. 待澄清事项
无；真实失败符号与实现机制由 T-01 取证，不预设新导入约定。
