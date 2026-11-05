---
plan_id: PLAN-953
status: executing
feature_name: owned结束捕获最后字节竞态修复
plan_revision: 1
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
supersedes_spec_components: [docs/specs/tool-execution/project.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [tool-execution]
depends_on: [PLAN-816, PLAN-896]
current_step: 0
total_steps: 6
---

# [PLAN-953] owned结束捕获最后字节竞态修复

## 0. 变更摘要

独立关闭F-952-01/D-952-1：原CaptureWorker把早于owned_closed观察的空peek用来判EOF，最后字节可能在两观察之间到达并被丢。修复原设施收尾采集，不另造runner，不靠减并发/删断言/刷完整组补救。

## 1. 目标

正常退出/owned结束必须完整保留已写入stdout/stderr的原始字节；空peek必须在观察owned结束后重新确认。保持原错误/stop/timeout/cancel/PipesHeld/panic/owned树与线程收尸分态，零新依赖、零新公开API或预算。

## 2. 架构方案

在原reader读就绪分支，closed为真后进行一次新的即时poll；有数据则继续正常读取，只有新poll也空才EOF。复用原循环/错误/停止控制，不新用500ms drain作为成功截止（避免把未排空当EOF）。生产变更局限`crates/auto-tool-execution/src/lib.rs::CaptureWorker::spawn`；同模块永久竞态/读错/合法空EOF回归，原exec_lifecycle真实子进程通道回归保留。

## 3. 技术栈

Rust std-only，原PipePoll/Read注入面，原固定组`cargo t auto-tool-execution`。必要真实消费者沿原`cargo t ace-a2x --test-name t04_record_move_and_owned_return`与`--test-name v15_deep_3072_small_stack_witness`，完整ace-a2x归952在D953后冻结合入门，不在953再重复完整ACE。

## 4. 需求分析与背景调查

用户2026-11-06已答复“授权独立修复并继续（推荐）”，覆盖结束判定前重新排空/错误分态/永久竞态回归/验证/复审合入/返回952与941，全链继承无需再问。主检出基线93f044978，main只有其他会话文档WIP，保留不纳提交。原源码SHA c177b278f1d805d21c808054a847adde0bc830e1e24ad0db9b811ce8298e4e09。952两次完整ACE门各188/189、两个失败case单例绿；**不认定这两次具体失败因果已证明**。独立确定性副本probe在合法poll交错下actual空/expected last、exit101，原始证据在952 §9；新包不移动/删952材料。

正式Spec `docs/specs/tool-execution/project.md`要求整次执行/原字节/自然EOF/所有错误与回收具名，现设施共享stdin/null/capture output包装与ACE真实编译消费者。唯一tree拟`D:/autostack/.wt/lang-953/auto-lang`、branch `plan-953-dev`，只读组auto-down。材料唯一`D:/autostack/plan-artifacts/plan-953/`，含所有runtime/TEMP/外置target；正常构建target只在专属树，merge全删。

当前host goal仍blocked，工具不提供resume；本包按本次明确直接授权执行，不新建/伪称恢复goal。原941未完成，返回链保持。无agent/chat授权，同会话实现与冻结后独立复读审查，说明上下文限制。

## 5. 详细设计

生产闭包：PipePoll早空→owned结束→新poll→原Read字节/EOF/IO/panic→原finish_capture分类。stop检查仍在原循环顶部，超时/取消仍原树终止/reader停止；不加默认deadline，不把部分采集变Success。新poll仅在closed且旧poll空时发生，不影响正在执行的普通读分支。

### 规范增量

| delta_id | action | target | before/after | AC |
|---|---|---|---|---|
| SD-01 | modify | docs/specs/tool-execution/project.md | 明确owned结束后必须新确认管道是否空，不能用此前空peek推EOF；保持字节与错误/停止分态 | AC-01/02 |

## 6. 测试设计

| V | AC/entry | 输入/不变式/观察oracle | 边界 |
|---|---|---|---|
| V-01 | AC-01 原CaptureWorker永久单测 | 空peek后最后字节与closed到达；stdout/stderr各完整原字节（含NUL/无效UTF8），修前红修后绿 | 可控合法交错，不是靠sleep碰运气 |
| V-02 | AC-02 同新poll分支 | 最后read真实IO错→ReaderIo；合法空EOF→空成功；已到字节不丢；原stop/panic/PipesHeld现case保持 | 不把新poll的错误抹成EOF |
| V-03 | AC-02 原exec_lifecycle | 自带截止te-child、真实两流/大输出/invalid UTF8/超时部分输出/树与holdout等原用例 | 原样本/ignore/平台条件不减 |
| V-04 | AC-02 原ACE两命名消费者 | record/text move+return值及alloc/free事件完整；3072深链stdout含6144及live0 | 只归采集消费者验证，不冒整个后端能力 |
| V-05 | AC-03/04 | 冻结逐AC/SD/遗漏复审后，完整auto-tool-execution组+必要消费者终态 | 完整ACE只由952后续跑 |

新测试驱动原私有生产闭包，不复制算法作oracle。fixture与target显式本scope/树，零junction/symlink。适配共享多消费者规则但不机械展开全仓AST/实验图；原reader/平台/错误矩阵由完整职责组闭合。Windows实测，其他平台只声明同可移植分支/原条件，不冒实机通过。

## 7. 验收标准

| ID | owner_stage | 条件 |
|---|---|---|
| AC-01 | work | 原生产reader可控竞态修前红/修后完整字节；stdout/stderr分别覆盖，无新500ms截断成功 |
| AC-02 | work | 原IO/stop/panic/timeout/cancel/持pipe/树与线程收尸/合法空EOF保持，真实消费者原观察正确，无新warning/依赖/协议变动 |
| AC-03 | review | 冻结逐AC/遗漏/workaround/SD复审pass；scope与输入身份明确 |
| AC-04 | merge | 完整组与必要消费者门绿、线性落主线、canonical核验、归档/guard/全scope清理、交付952/941 |

## 8. 执行步骤

| ID | owner_stage | deps | 内容 | 状态 |
|---|---|---|---|---|
| T-01 | work | - | 创建登记树/scope，原reader正式永久竞态红与正控 | [ ] |
| T-02 | work | T-01 | 结束后新poll，定向竞态/IO/EOF与格式/编译绿 | [ ] |
| T-03 | work | T-02 | SD与代码提交，execution_done，完整职责门留merge | [ ] |
| T-04 | review | T-03 | 同树冻结独立复读AC/负控/遗漏/SD | [ ] |
| T-05 | merge | T-04 | 完整auto-tool-execution与ACE两命名消费者、线性合入/spec核验、归档 | [ ] |
| T-06 | merge | T-05 | guard/tree/branch/组/scope及实际输出目录对账清理、返回952/941 | [ ] |

## 9. 复审记录

未开始。用户已解阻授权，不能把953立项等同D953已交付或952/941已通过。

## 10. 待澄清事项

无新增需求问题。仅host goal状态未自动resume；不阻本次用户明确授权的独立修复执行，不擅自伪造active状态。
