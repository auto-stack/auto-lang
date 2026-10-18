---
plan_id: PLAN-812
status: reviewed
feature_name: playground-linux-backend-deploy
author: [Codex]
plan_revision: 2
created_at: 2026-10-19
updated_at: 2026-10-19

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/auto-playground/project.md]
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 3
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
- 用户已授权重新部署后台，并于 2026-10-19 明确确认“修复并继续部署”本计划两文件修复。现有 sudo 凭据仅经进程输入使用，不入计划/日志/发布包。
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

- [x] T-01 (work; AC-01): 确认计划后承接工作目录，完成 Cargo.toml/transport.rs 最小修复、双平台定向检查并提交。[✅] Windows check pass（409 既有警告）；transport 定向 9/9；Linux release pass（3m44s，268 既有 lib 警告+后台 1 项既有字段警告）；rustfmt 与 diff check pass。实现 e7177e3fa → b2a75c1c2d3f83068b2d262d1448245292326c6c，rebase 仅同步主分支文档簿记，range-diff 1/1 完全等价；依赖 c39ac7f。
- [x] T-02 (work; AC-02, AC-03): Linux release 与资源包冻结，在本地 Linux 启动、smoke；使用已批准的回滚脚本部署，核验公网与浏览器。[✅] 发布二进制 847132bb...a71b11；最终包 cf13cb38...5fc390；本地隔离包/服务器/公网 HTTPS 均通过（1343 examples、stdout=7、15 条 bytecode/meta、五种 trans、多模块 greet）。线上 Run/Trans 两右窗已实点验证，截图已入证据。首次漏带 stdlib 自动回滚已验证，补齐后成功部署。源码基线 7cab6ca2b7498c5a8ef28e070f8d107267ac18c9；详见 attachments/812/verification.md。
- [x] T-03 (review; AC-04): 明确冻结基线，重查差异、验收与 SD-01，记录 merge-owned 门禁 pending。[✅] final pass，reviewed_commit=7cab6ca2b7498c5a8ef28e070f8d107267ac18c9；复审实际 diff、依赖 target=null、Linux 构建、定向 9/9、隔离包/回滚/公网与 UI 证据，SD-01 与代码一致。裸 cargo t 仍 pending，未合入。
- [ ] T-04 (merge; AC-05): 完成所有 work 验收并最终复审后统一执行 merge-owned 门禁，合入、沉淀、归档和 guard 清理。

## 9. 复审记录

2026-10-19 final review: stage=review; review_scope=final; plan_revision=2; outcome=pass; worktree=D:/autostack/.wt/lang-812/auto-lang; branch=plan-812-dev; reviewed_commit=7cab6ca2b7498c5a8ef28e070f8d107267ac18c9; base=ab08955c2（实现之前主分支簿记基线）; auto-down=c39ac7f。reviewer=本 Codex 会话的独立复审步骤；局限=与实现共享上下文，未声称有另一独立代理；从冻结代码/原始结果重建结论。AC-01..04 全部 pass；AC-05 pending merge。Spec source SHA256=8bd7c084e979417dda827a19f1fe36472227024b288287b71c7cccdff06b38c2（docs/specs/auto-playground/project.md）。检查表：仅移动既有依赖到通用段，Windows 依赖不变；仅重导出已存在的 Linux 常量，不改变 stub 行为；无编译器/VM/转译算法变更；发布完整 stdlib 与编译期路径资产；失败回滚实际演练通过；凭据不入交付文件；无遗漏、未经批准延后或页面绕行补丁；既有 409 warnings 有界记录、零新增警告/债务。复审另从公网执行 print(3+4)：stdout=7、bytecode=8、meta 存在；服务 active、已装二进制 SHA 与发布包一致。pending_closeout_gates=bare cargo t（touched surface tiers=none，tf=batch-only）。next=merge gate→沉淀/归档/guard 清理。

2026-10-19: 用户明确批准 r1。r2 仅修正任务时序：线上验收 AC-03 属 work，须在最终复审前完成，避免原 T-04 同时承载 work 验收和 review 后收尾的循环依赖。目标、修复范围、门槛及部署授权不变，继承已有授权；实现文件两处，另准备 SD-01 与可重放 smoke/install 证据。

2026-10-19: stage=new; revision=1; outcome=pass (contract ready); next=用户确认源码修复范围后 work。本记录不表示源码修复已获计划确认；旧服务继续运行。

## 10. 待澄清事项

用户已确认全部范围，无待决审批。源码修复、双平台验证、部署与验收继续自主执行。
