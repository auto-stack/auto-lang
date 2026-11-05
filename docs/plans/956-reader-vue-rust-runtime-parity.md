---
plan_id: PLAN-956
status: executing
feature_name: Reader Vue Rust backend parity
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 1
current_step: 0
total_steps: 6
supersedes_spec_components: [docs/specs/auto-lang/trans/rust-semantic-admission.md]
new_spec_components: []
touched_goals: []
---
# PLAN-956 Reader Vue + Rust 后端能力与 VM 对拍
## 0. 变更摘要
用户明确要求继续补齐默认Rust后端，不能只换VM。归档955仅诊断/可用入口，本计划承接真正的Rust执行能力。
## 1. 目标
AC01(work): Reader同一back源码实际A2R/API生成、Rust编译成功，默认Vue+Rust启动，42条路由保留；禁止VM执行桥/脚手架替代业务。
AC02(work): VM原生type_of及Reader用到的动态JSON、跨模块签名/类型、标准库能力在Rust侧语义正确；普通未知调用仍拒绝。
AC03(work): 隔离同源夹具、同序API操作，对拍Rust/VM响应与持久化；覆盖导入/重复/中文/章节/进度/标注/搜索/导出/元数据/组/移除恢复/坏输入/重启恢复。非确定ID/时间须具名规范化，不抹业务差异；route清单42匹配。
AC04(work): 有永久测试入口，关键原语正反例，原VM oracle与数据不被弱化；Vue前端生产构建与Rust后端均真实成功。
AC05(review): 冻结代码与规范、差异清单和全部AC独立重建复核；merge负责实际受影响固定职责组并集和收尾。
## 2. 架构方案
保持共享Rust转译→a2r-std运行时→Web生成器→原生服务路径。先 bounded inventory，输出真实转译/编译错误族，再修复各owner的共享机制。新增能力以既有VM行为及Reader接口契约为基准，遇到VM缺陷不复制数据损坏，具名核对双方与独立oracle。
## 3. 技术栈
Rust auto-lang-core/a2r-std/auto-generation-core；auto-web API generation；auto-ui工具consumer；auto-reader行为门。
## 4. 需求分析与背景调查
授权：用户2026-11-06明确继续实现Rust后端、构建运行及VM一致性，可执行必要跨仓修复与验证，不再以VM切换算交付。未要求额外审批/模型/预算。
依据：docs/specs/overview.md、docs/specs/auto-lang/trans/overview.md、rust-semantic-admission.md；reader docs/research/20261005-t00-runtime-capability.md §12，src/back全部模块、tests/spec与tests/basic的现有oracle。
955确认 first error type_of；现有代码已有JSON Value跟踪和a2r标准库，历史缺口清单仅线索，不当作今天事实。先跑现行源码生产入口，使用准确symbol/module诊断。
主检出既有他方docs WIP、reader两个未跟踪assembly文件保留。本计划只提交自身文件。唯一实施组 D:/autostack/.wt/lang-956/{auto-lang,auto-web,auto-ui,auto-reader,auto-down}，各repo唯一worktree；branch codex/plan-956-dev或repo后缀，依赖只读。材料 D:/autostack/plan-artifacts/plan-956/。
## 5. 详细设计
T01 bounded probe：真实全部back转译/完整Web生成，收集错误族与闭包；选择等价实现并登记，若需要改变用户可见语义先依据独立oracle解决。
T02 语言/stdlib能力：裸type_of、动态JSON类型和值消费、必要调用/类型解析，保留typed静态准入；正反例从真实语法构造。
T03 API组装：完整back模块声明、引用/依赖闭包与Rust构建，禁止fallback/default空业务/VM委托。
T04 双轨永久对拍与真实默认Vue+Rust运行/生产构建。
### 规范增量
| ID | 操作 | target | before/after | 理由 | AC |
| SD01 | modify | docs/specs/auto-lang/trans/rust-semantic-admission.md | type_of/Reader原先VM-only边界按实际完成能力更新；严格未知符号不变 | Rust真实支持 | AC01/02 |
| SD02 | add/modify待T01定位 | docs/specs/...各实际owner规范 | 写入动态JSON/生成服务运行约定与行为对拍边界 | 持续回归 | AC03/04 |
## 6. 测试设计
V01：真实Reader生成物cargo check/build及原生server ready+42routes，HTTP不是VM进程；无AUTO_A2R_BODY=0/scaffold绕过。
V02：type_of的int/float含0.0/string/bool/null/array/object，独立期望与VM对拍；JSON字段/数组/null/缺失/嵌套值及消费型转换永久正反例。
V03：Reader现有HTTP规格与独立VM/Rust序列对拍，响应状态/JSON业务值/文件字节/重启后值作为oracle，覆盖AC03。资源夹具只写隔离数据；非确定值规范化清单可审查。
V04：必要完整固定组至少a2r、a2r-std、auto-generation-core与实际受影响http/api组；Web/UI使用各owner相应测试。按触面明确并集，不把scope测试称全仓回归，不降低原样本/门禁。
实施和最终复核顺序在同一树；不创建未授权agent，当前上下文复核须明示限制。
## 7. 验收标准
AC01~05均须真实通过；只生成Rust或只过compile不满足行为一致。新旧tool/源码身份绑定，失败不记PASS。
## 8. 执行步骤
- [ ] T01(work) 真实缺口清单与机制决策。
- [ ] T02(work) 语言/标准库能力修复与永久原语反例。
- [ ] T03(work) RustAPI实际生成/构建/启动。
- [ ] T04(work) Reader双轨对拍/重启/前端构建。
- [ ] T05(review) 冻结最终验收与规范复核。
- [ ] T06(merge) 完整职责门、合入与安全收尾。
## 9. 复审记录
stage:new | r1 | pass | 用户已明确授权当前范围；next:work，不再请求同范围审批。
## 10. 待澄清事项
无；具体错误族由T01调查，不预设通过删除业务/改用VM完成。
