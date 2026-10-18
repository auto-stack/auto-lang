---
plan_id: PLAN-812
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: playground-linux-backend-deploy
author: [Codex]
plan_revision: 1
created_at: 2026-10-19
updated_at: 2026-10-19

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/auto-playground/project.md]
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 4
---

# [PLAN-812] playground-linux-backend-deploy

## 0. 变更摘要

修复阻塞 Playground 后台发布的两处 Linux 声明缺口，构建并验证 Linux 发布包，备份后部署、重启与验收线上 Run/Trans。有效修改仅为通用依赖作用域与已有常量重导出。

## 1. 目标

恢复 https://auto-lang.cn/zh/playground 的字节码/转译代码右窗。非目标：界面改版、编译算法改变、权限制度调整、其他在途 Plan。

## 2. 架构方案

沿用 nginx → 127.0.0.1:3030 → systemd auto-playground，保留现有 unit。构建基线 687efdd352985a3034bba53f6c8fc8549bc3f459 加本计划两处修复；配套资源按编译期 CARGO_MANIFEST_DIR 绝对布局发布。备份旧程序后原子替换并重启，验收失败恢复旧程序并重启。

## 3. 技术栈

Rust/Cargo、Axum、Ubuntu 22.04 WSL、Linux x86_64、systemd、HTTPS API 与浏览器。

## 4. 需求分析与背景调查

- 规范来源：docs/specs/overview.md、docs/specs/auto-playground/project.md、docs/specs/website/project.md。
- 用户已授权重新部署后台，并提供现有 sudo 凭据。凭据仅经进程输入使用，不入计划/日志/发布包。尚待确认本计划的两文件源码修复。
- 线上 /api/run 缺 bytecode/meta，/api/trans 返回旧 code；当前 usePlaygroundFull.ts 消费 bytecode/meta 和 files[].code。本地当前源码已验证 stdout=7、bytecode 非空、Rust files=1。scripts/deploy-website.ps1 仅部署静态前端。
- 生产旧程序与 unit 已备份到 /var/backups/auto-playground/20261019-687efdd35；旧服务持续运行，尚未切换。
- Linux 原构建 53 个错误：a2r-std 被放在 cfg(windows) 依赖段。独立 fix-playground-linux-build worktree 已做最小试修，未提交/合入/部署；Windows cargo check 在途。
- 试修后 Linux 只剩一错：rqhost.rs 引用 transport::CLAIM_DENIED_MARKER，但非 Windows pipe_stub 重导出漏了已有常量。
- 批准后将 Windows 试修 worktree 承接至 D:/autostack/.wt/lang-812/auto-lang、分支 plan-812-dev。Linux 发布构建副本为 /home/visus/.local/share/auto-playground/releases/687efdd35/auto-lang，只消费同一冻结源码；依赖 auto-down=c39ac7f。

## 5. 详细设计

1. crates/auto-lang/Cargo.toml：移动现有 a2r-std 依赖与注释到通用 dependencies；路径、版本、features 不变。
2. crates/auto-lang/src/ui/desktop_protocol/transport.rs：为非 Windows pub use pipe_stub 补 CLAIM_DENIED_MARKER；不改变占位传输的拒绝行为。
3. 构建 Linux release，随包发布 notes.json、项目示例、自举库及独立前端；保留提交、锁文件、SHA256、ldd 和 smoke 结果。
4. 备份后原子替换 /usr/local/bin/auto-playground，重启现有服务；本机与公网检查失败自动恢复备份。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-playground/project.md | 补充 Linux 发布必须使前后台接口一致、携带编译路径对应运行资源，并验 bytecode/meta/files | 防止仅前端更新造成静默空窗 | AC-02, AC-03 |

## 6. 测试设计

- Windows cargo check -p auto-lang；Linux cargo build --release --locked -p auto-playground；检查两目标依赖图与非 Windows 重导出。必要 transport 定向测试采用仓库已有入口。
- 真实 API smoke：examples>100；Run stdout=7 且 bytecode 非空/meta 存在；Rust/C/Python/TypeScript/ABT 的 files[].code 非空；09-multi-module 输出 Hello, Playground!，验证部署资产。
- 线上浏览器实点 Run/Trans，检查右窗并截图；API 与页面均通过才称修复完成。
- 最终复审前仅 check/定向验收；冻结提交复审 pass 后由 merge 执行裸 cargo t，其他触面档按实际 diff 裁定；不运行无关 tf/taa。复审如在同会话完成须记录上下文局限，重建 diff 与验收证据，不直接采信执行摘要。

## 7. 验收标准

| ID | owner_stage | criterion | evidence |
|---|---|---|---|
| AC-01 | work | 两处声明补齐、Windows check 与 Linux release 成功 | 冻结 diff、构建日志 |
| AC-02 | work | Linux 包启动、动态库与真实 API smoke 通过 | ldd、SHA256、smoke JSON |
| AC-03 | work | 线上两种右窗恢复、服务 active、保留备份与回滚 | 公网 API、浏览器、systemd |
| AC-04 | review | 冻结提交/范围/无遗漏绕行与规范增量审查通过 | 复审记录 |
| AC-05 | merge | 审查后门禁通过、合入、沉淀、归档和 guard 清理完成 | merge receipt |

## 8. 执行步骤

- [ ] T-01 (work; AC-01): 确认计划后承接工作目录，完成 Cargo.toml/transport.rs 最小修复、双平台定向检查并提交。
- [ ] T-02 (work; AC-02): Linux release 与资源包冻结，在本地 Linux 启动、smoke，准备失败回滚脚本。
- [ ] T-03 (review; AC-04): 明确冻结基线，重查差异、验收与 SD-01，记录 merge-owned 门禁 pending。
- [ ] T-04 (merge/work; AC-03, AC-05): 审查后统一门禁；部署已授权后台并公网/浏览器核验；通过后沉淀、归档、guard 清理。线上失败先恢复旧服务。

## 9. 复审记录

2026-10-19: stage=new; revision=1; outcome=pass (contract ready); next=用户确认源码修复范围后 work。本记录不表示源码修复已获计划确认；旧服务继续运行。

## 10. 待澄清事项

待用户确认：是否同意先完成上述两处最小 Linux 声明修复，再继续已授权的后台部署？
