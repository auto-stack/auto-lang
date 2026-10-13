# auto-plan 流程与技能修订稿（2026-10-06）

> 后续安装更新：用户批准后已实施并安装到共享软链接，实际提交、校验与范围见 [INSTALLATION-RECEIPT.md](INSTALLATION-RECEIPT.md)。下文及原 manifest 保留安装前修订稿的历史状态。

建议先落地三个改变：ChatGPT 最终复审在合并前完成；高风险计划提供能区分错误实现的验收输入；修复必须检查同一规则的其他适用路径。四个技能的具体补丁见 [auto-plan-skills.patch](auto-plan-skills.patch)，条件性模板见 [verification-contract.md](verification-contract.md)。

本次交付为可应用的修订稿，尚未安装到共享技能。基线为 auto-musk 的 `edde608709c5dd5b413fc9c534113d41f0bb8afe`；用户级四个技能路径均通过符号链接指向该仓库 `.agents/skills/`。补丁应在 auto-musk 的独立 worktree 应用、核查并按其流程合入；后续使用这些共享技能的计划会读取新版。原有计划、后端状态机和编号脚本本轮不改。

## 1. 下一轮的实际流程

| 阶段 | 负责方 | 交付与允许状态 |
|---|---|---|
| new | ChatGPT | 当前合同、公开验收输入、风险边界、任务依赖与阶段责任；drafting |
| work | GLM | 已提交实现、真实结果、全部 work 任务与高风险路径覆盖；execution_done |
| internal review | 与实现分离上下文的 GLM 检查者 | pass 仅表示可交最终复审；仍 execution_done；不合并或归档 |
| final review | ChatGPT | 对当前提交独立重放与补充反例；pass 才 reviewed |
| merge | 收尾执行者 | 规范/索引/实际归档核查，随后实际清理；最终 delivered pass |

这只是用户当前分工。共享技能按计划中指定的角色运行，不将 ChatGPT/GLM 写成所有项目的固定模型要求，也不自动创建新代理或聊天。

needs_fix 保持计划活动、复用原 worktree，只重开受影响任务与证据。第一次修复就提交“发现→不变量→适用路径→修复/用例→结果”的覆盖表。同一交付连续两次最终复审失败，下一轮先诊断合同/判据、局部修复、新回归或输入漂移，再继续已授权工作；计数不能因提高 revision 或内部 pass 清零。保留现有自动修复上限；诊断不是再次索要同范围批准。

## 2. 计划应该详细到什么程度

计划要写清楚行为、边界、决策、依赖、验收判据和各阶段任务；常规实现机制留给执行者。不要把“逐行伪代码”作为全面性的标准，也不要要求所有普通修改都写风险矩阵。

高风险要求至少回答：保护什么；适用于哪些已承诺路径/状态；合法与非法代表输入是什么；在哪个真实入口验证；观察什么结果；哪种错误实现会被该输入揭露；哪些能力/组合本计划不承诺。

示例（设计输入，不冒称已生成或执行的 fixture）：

```text
AC-CALL-01：bindings 决定参数对应，eval_args 决定求值顺序。
合法输入：params=[i32,bool]，eval_args=[true,9i32]；
          bindings={param0←arg1, param1←arg0}。
          callee 返回 param0。
预期：真实 verifier 接受，native 执行结果为 9。
非法输入：eval_args=[9i32,true]，保留同一交换 bindings。
预期：verifier 定位类型不匹配；不得进入后端、不得 panic。
顺序：另保留有可观察副作用的顺序用例；纯值结果不能证明求值顺序。
边界：当前 profile 的 i32/bool，不要求新类型或新调用语法。
责任：work 实现/接入测试；final review 独立确认上述行为。
```

相比“调用测试通过”，这个合同让执行者知道必须实现什么，也给复审者相同、公开的判断依据。必要的同类维度如普通/mut、自由函数/方法、裸/qualified，应在开工前列出支持边界；不要求穷举所有笛卡尔积。

## 3. 四个技能的具体修改

| 技能 | 补丁内容 | 本轮故障如何被约束 |
|---|---|---|
| new | 高风险验收表与开工前判据审查；任务/AC 标注 owner_stage；明确内部/最终角色；一个当前任务表 | 避免只有原则和 happy path、后续操作提前勾选、旧 Phase 当当前合同 |
| work | 同根因路径/消费者/状态覆盖；变异确实发生；真实执行计数；两次最终失败后诊断 | 避免只修发布出口却漏链接出口、只修错类型却漏组合、换 revision 继续机械修补 |
| review | internal/final/phase 收据与路由；首轮覆盖已识别风险；独立反例；核对原始控制执行；禁止“近不可达”免除合同；语义改动重审 | 避免内部 pass 自动授权 merge、17 实跑称21、发现承诺违例仍放行、review 后补代码沿用旧 pass |
| merge | 指定最终复审是合并门；真实归档路径/必要引用/目标身份；清理真实完成后计数；失败保留树按收尾恢复 | 避免合并后才做最终验收、错指另一个存在文件假绿、反复归档再激活原生工作 |

`owner_stage`、`review_scope` 和角色为正文/收据信息，不宣称当前 AutoMusk 后端已经自动识别。五状态保持：drafting→executing→execution_done→reviewed→archived。

仅 work 任务完成即可进入 execution_done，pending review/merge 任务继续计入总数但保持未勾；final pass 才 reviewed；merge 只接手已明确的收尾项。不得把未实现行为标为 merge 责任来绕过最终验收。归档检查时清理可以是明确待办，最终 merge pass 时所有实际阶段任务必须完成。

## 4. 避免又写一套容易出错的检查器

本补丁提供合同和审查规则，没有为每个计划再添加一套 Python checker。下一步若需要机器执行保障，应做一个有界的共用入口：读当前任务/阶段责任，记录实际 case ID 与结果，核对 revision/commit/制品身份，并检查必要的终态引用。

该入口必须有独立固定 fixtures：错误计数、遗漏控制执行、无效变异、缺必需引用、指向另一个存在文件、active/archive 状态转换，以及失败向整体返回值传播。不得由 ALL-PASS 输出自行证明覆盖。已有仓库门能复用时优先复用；普通文档/小修不因此增加重型检查。

阶段名称和写入报告不能证明审查独立，也不能让自然语言规则自动成为后端门禁。实际驱动必须将内部检查和最终复审分成两次调用，并让 merge 读取最终复审收据。若原驱动会内部 pass 后自动 merge，应先关闭该自动跳转或调整路由，再声称采用了新流程。

本地可见 Relay 模板还存在具体旧规则，需与技能一起对齐；这不是已经证明 741/743 使用过该驱动，也不代表未同步主机器的最新版：

| 可见源码 | 现状 | 建议调整 |
|---|---|---|
| [plan_flow.rs:85](D:/autostack/auto-musk/backend/crates/musk/src/relay/plan_flow.rs:85) | 计划是“唯一工作上下文”，全部任务完成后 execution_done | 计划为主要合同；相关 Specs/源码为证据；仅 work 所有任务完成后交接，后续阶段任务继续待办 |
| [flows.rs:38](D:/autostack/auto-musk/backend/crates/musk/src/relay/flows.rs:38) | 正式链路只有一个 review，随后 document | 最小调整是保留一个正式 review，指定给最终复审者；GLM 内部检查留在 work 内。需要双 review 自动调度时另做有界实现 |
| [plan_flow.rs:107](D:/autostack/auto-musk/backend/crates/musk/src/relay/plan_flow.rs:107) | 单个 review 通过就设 reviewed | 对用户指定分工，内部结果不能设 reviewed；最终收据绑定当前代码/合同/依赖 |
| [plan_flow.rs:125](D:/autostack/auto-musk/backend/crates/musk/src/relay/plan_flow.rs:125) | document 主要检查状态，再调用 merge_plan | 除状态还需实际最终 pass 与版本绑定；按现行 merge 技能分步沉淀/验证/归档，避免旧合并式入口 |

因此落地顺序是：先把正式最终复审前移并调整实际调用路由；应用技能补丁和计划模板；随后再开发共用的机器检查入口。每次高风险交接记录实际读取的技能路径/内容版本及可观察的模型/上下文信息，便于区分旧副本、错误路由与执行能力问题。

## 5. 修订稿核查与后续衡量

核查结果写在 [patch-manifest.json](patch-manifest.json)：四个候选技能通过 skill-creator 的 quick_validate；补丁对固定基线通过 git apply --check；模板和引用可解析。按 741/743 的失败类别进行了纸面场景走查，特别修正了“归档前要求未来清理已完成”的时序矛盾。

这些是格式、可应用性和规则一致性核查，未进行独立模型行为试验，尚不能证明执行者遵循新规则。安装后用下一至两个计划验证：首次最终复审通过率、第三次复审比例、同根因重现、内部误放行，以及实现/证据/收尾分别耗费的时间。目标是通常一遍、失败后通常修复一次；再次出现同根因反复，就依据执行轨迹调整模型或提前安排专项审查。
