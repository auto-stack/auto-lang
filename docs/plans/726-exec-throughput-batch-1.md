---
plan_id: PLAN-726
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: exec-throughput-batch-1
author: [agent]
created_at: 2026-10-02
updated_at: 2026-10-02

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-lang]   # T-01 触 vm ffi；T-02..T-04 为测试基建/工作流（无 specs 树对应）
current_step: 0
total_steps: 5
---

# [PLAN-726] exec-throughput-batch-1

## 0. 变更摘要

治理"计划执行变慢"的第一批高性价比项（2026-10-02 用户裁定分批：本批四项落地，
余量登记 KNOWN-DEBT-AND-RISKS.md「执行吞吐治理第二批挂账」THR-D1..D7）：

1. **T-01** 修复 e4 直驱臂确定性红（P707-R1 清偿）——日常档 `cargo t` 62s 中占
   30s 长尾，修复后预计 ~32s（AGENTS.md 资源表在档预期）。
2. **T-02** 重型测试族机器级单实例闸门——扩展既有 `heavy_gate.rs` 体内守卫为
   跨进程机器锁，把"并行互毁/挂死 30+ 分钟"（2026-09-30 实录）转化为"第二实例
   立即确定性红 + 持锁者 PID 提示"。
3. **T-03** book_listing 外部书仓脏态守卫——book 仓在途 WIP 期间 7 测确定性红
   （PLAN-715 登记），改为脏态确定性 skip，消灭无关计划的复审对账税。
4. **T-04** worktree 组脚手架脚本——一键建组（含 auto-down 兄弟仓），消除
   L0/L1 冷启动 cargo 解析失败摩擦（2026-10-02 本会话实录：组内缺兄弟仓，
   `cargo check` 直接失败，手工补建后才通过）。

## 1. 目标

- 日常档验证时长 62s → ~32s（e4 红修复，长尾 30s 消失）。
- 重型测试族（画廊围栏 / 1M churn / taa XL 族）多进程并发时第二实例快速失败，
  不再出现挂死与资源互毁。
- book 仓 WIP 期间 book_listing 族不再产生需要人工定案的红。
- 新建 worktree 组从"手工多步+踩坑"变为一键，组内 cargo 解析即开即用。

**非目标**：crate 拆分、共享 target 评估、master 14 红族清偿、release 档红册、
走查脚本沉淀（全部在第二批 THR-D1..D7）；不改任何产品语义（UI/VM/编译器行为面
除 e4 缺陷修复外零变化）。

## 2. 架构方案

无产品架构变更，四项均为缺陷修复 + 测试基建 + 工作流工具：

- **T-01**（缺陷修复）：`crates/auto-lang/src/vm/ffi/async_http.rs` 的
  `submit_detached_client_job` 消息桥（plan707 T-02 分离产物）疑似不再派发
  fire-and-forget 直驱 job，导致 `stdlib.rs:10376`
  `default_headers_reach_wire_on_plain_get` 的 `spawn_async_http_handle` /
  `spawn_async_http` 直驱三臂恒不到线（30s recv_timeout 恒红）。勘定→修复→
  回归锁，同域不动 SSE/merged 臂（plan705/707 已绿面零扰动）。
- **T-02**（测试基建）：`crates/auto-lang/src/tests/heavy_gate.rs` 既有模式
  （env 门 + SKIP）扩展出 `machine_gate(family)`：锁文件置于机器级固定路径
  （`D:/autostack/.locks/<family>.lock`，Linux 回落 /tmp），O_EXCL 创建 + 写入
  PID/cwd/tier，冲突时读持锁者并核验进程存活性（死进程=陈锁可夺），存活则
  panic 输出"heavy family X already running (PID N, cwd W)"。接入点=三族
  重测试首行（与既有 heavy_gate 同位）。**别名方案已否决**：cargo 别名只能
  组合 cargo 子命令（现有 `t = "nextest run"` 依赖 cargo-nextest 外部子命令
  约定），无法直接调外部包装脚本——此结论入档防止后续会话再走弯路。
- **T-03**（测试守卫）：`crates/auto-lang/src/tests/book_listing_tests.rs`
  运行前 `git -C <book_repo> status --porcelain` 非空 → 全族确定性 skip
  （打印"book repo has WIP; skipping"），干净 → 现行为不变。
- **T-04**（工作流脚本）：`scripts/new-wt-group.sh <group> [--branch <name>]`：
  建 `D:/autostack/.wt/<group>/` + auto-lang worktree（带分支或 detached）+
  auto-down 兄弟（detached @HEAD）——与 lang-719/724 既有组布局一致；组目录
  已存在即报错；输出后续步骤（开发→合入→wt-guard→移除）提示。零 junction
  （worktree 红线）。

## 3. 技术栈

Rust（stdlib e4 测试、heavy_gate、book_listing guard）、Python 3 无涉（别名
方案已否决）、Bash（scaffold 脚本）、cargo/nextest 既有档位体系不变。

## 4. 需求分析与背景调查

**授权记录**（2026-10-02 用户裁定）：按性价比分两批治理计划执行变慢；第一批=
本计划四项（e4 红修复 / 重型闸门 / book 守卫 / worktree 脚手架）；第二批登记
DEBTS 下次再做。仓库范围 auto-lang；无预算与自动续行限定。

**背景数据**（2026-10-02 本会话实测/在档）：

- 规模：10,363 跟踪文件 / 1,650 .rs（76.6 万行）/ Cargo.lock 907 依赖；
  主检出 target 28G；2026-09 单月 3,710 提交、202 计划；当前 8 组并行 worktree。
- 变慢三因子（已向用户报告）：①并行资源互毁（2026-09-30 多 agent 并行 tf
  实录，一跑挂死 30+ 分钟=AGENTS.md 在案）；②验证信号噪音（预存红/flake 每
  红一次定案税：P694 蓝屏 13 轮复现矩阵、P716 MCP 端口 6 次启动矩阵）；
  ③Rust 单 crate 编译链（sccache 79% 命中下冷 worktree check 57s 样点，
  链接/build script 801 次 non-cacheable）。
- e4 红：KNOWN-DEBT P707-R1（high）——30s recv_timeout 恒红实证，plan705 e2e
  与 plan707 SSE 同机绿⇒机器网络正常；4f123a50e 曾误标为环境红。
- book 红：PLAN-715 登记——`tests/book_listing_tests::*` 7 测输入源为外部
  兄弟仓 `D:/autostack/book/rust/listings` 实时内容，该书仓在途编辑期确定性红。
- L0 摩擦：crates/auto-lang/Cargo.toml 跨仓 path 依赖（autodown-core）按
  "组内 ../auto-down" 解析（AGENTS.md 解析序），单仓组直接 cargo 失败。

**Spec 状态**：vm ffi 直驱臂行为属既定期望（default headers 到线），无文本
漂移；测试基建/工作流无 specs 树对应（AGENTS.md 为规约载体）。

## 5. 详细设计

### T-01 e4 直驱臂修复

- 复现：`cargo t default_headers_reach_wire`（隔离单跑确认 30.0s 恒红形态）。
- 勘定：读 `async_http.rs` `submit_detached_client_job` 与 plan707 T-02 diff
  （git log -S 定位分离提交），确认直驱 job 是否仍入队/被消费；产出一句
  定案结论入 §9。
- 修复：恢复派发或修正桥协议；不引入新通道。
- 回归锁：现有测试本身即锁（转绿）；如勘定出独立边界补 1 枚单测。

### T-02 heavy_gate 机器级闸门

- 新增 `machine_gate(family: &str)`（heavy_gate.rs 内）：锁目录
  `D:/autostack/.locks/`（存在性创建；不可用回落 std::env::temp_dir）；
  O_EXCL 建锁→写 `pid|cwd|family`；冲突→读持锁 PID→存活核验（Windows
  OpenProcess / Unix kill -0；进程亡=陈锁删除重建）→存活则
  `panic!("heavy family '{family}' already running (PID {pid}, cwd {cwd}); single-instance rule — retry after it exits")`。
- 接入三族重测试首行：画廊围栏族、1M churn 族、taa XL 族（aavm 21 测中
  78-303s 档）——具体测试名执行期按 `.config/test-mem-weights.md` XL/LG
  清单对齐。
- CI（Linux 独立机）锁路径机器隔离，天然零影响；日常档（t/tv/tt/tb/tu）
  不触三族，零开销。

### T-03 book 脏态守卫

- `book_listing_tests.rs` 定位函数（ancestor 遍历 `book/rust/listings`）取得
  repo 根后：`Command::new("git").args(["-C", root, "status", "--porcelain"])`
  输出非空→`eprintln!` 原因 + 提前 return（逐测试守卫放公共 helper，7 测
  共用）；git 不可用/仓库缺失→维持现状（不新增失败面）。

### T-04 new-wt-group.sh

- 用法：`bash scripts/new-wt-group.sh <group> [--branch <name>]`（--branch
  缺省=detached @HEAD；分支名带 plan-<NNN>-dev 时提示取号规则引用
  new-plan.sh）。
- 步骤：mkdir 组目录（已存在报错退出）→ `git worktree add` auto-lang（主检出
  HEAD）→ `cd ../auto-down && git worktree add --detach <组>/auto-down` → 打印
  后续流程（开发/验证/合入/wt-guard/移除）。
- 红线合规：不创建任何 junction/symlink；移除仍走 wt-guard.sh 前置。

### 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md（直驱臂注记，若修复涉契约文本；纯缺陷修复则记录"无规范增量"理由） | 直驱 fire-and-forget 派发期望在档 | P707-R1 清偿 | AC-01 |
| SD-02 | add | AGENTS.md §并行纪律（工作流规约，非 specs 树） | 重型族"单实例纪律"约定 → 机器级闸门机械强制 | 2026-09-30 互毁实录 | AC-02 |
| SD-03 | add | crates/auto-lang/src/tests/book_listing_tests.rs 文件头契约注释 | 无守卫 → 外部书仓脏态确定性 skip 规则 | 测试基建无 specs 树对应 | AC-03 |
| SD-04 | add | AGENTS.md §1/§5 + scripts/（组脚手架入口规约） | 手工多步建组 → 脚本一键（含兄弟仓） | L0/L1 冷启动摩擦 | AC-04 |

## 6. 测试设计

- T-01：`cargo t default_headers`（转绿）；`cargo t`（全档绿+计时）；`cargo tv`
  （触 vm ffi，语料 golden 零回归）。
- T-02：并发两进程同跑单 heavy 族滤串→第二进程秒级确定性红（信息含 PID）；
  隔离单跑绿；kill 持锁进程后重跑可夺陈锁；`cargo t` 计时无变化。
- T-03：book 仓 touch 一个未跟踪文件→`cargo tb book_listing` 全 skip 绿；
  还原→全跑行为不变；仓缺失路径不新增失败。
- T-04：脚本建临时组（fix-scaffold-selftest）→组内 `cargo check -p auto-lang`
  通过→`bash D:/autostack/wt-guard.sh` clean→worktree remove 双仓→rmdir 组。

## 7. 验收标准

- **AC-01**：`default_headers_reach_wire_on_plain_get` 日常档绿（直驱三臂到线
  断言通过）；KNOWN-DEBT P707-R1 复审销号；4f123a50e 误标更正注记在案。
- **AC-02**：三 heavy 族并发第二实例立即确定性红（含持锁者 PID/cwd 提示），
  隔离单跑绿；日常档（t）时长与行为零变化。
- **AC-03**：book 仓脏态→book_listing 族确定性 skip（非红非 fail，原因可见）；
  干净态全量行为不变（基线对照跑）。
- **AC-04**：`new-wt-group.sh` 一键建组含 auto-down 兄弟，组内
  `cargo check -p auto-lang` 直接通过；wt-guard clean；零 junction。
- **AC-05**：修复前后计时回执（日常档 62s→目标 ~32s，标注竞争态）落 §9；
  AGENTS.md 资源表数字刷新项移交 merge 阶段。

## 8. 执行步骤

- [ ] **T-01** e4 直驱臂修复（依赖：无）
  - 复现+勘定（§5 T-01）→ 修复 `async_http.rs`/`stdlib.rs` 直驱臂 →
    `cargo t default_headers` 绿 → `cargo tv` 零回归 → 误标更正注记。
  - 验证：`cargo t default_headers && cargo tv`。
- [ ] **T-02** heavy 机器闸门（依赖：无，可与 T-01 并行）
  - `heavy_gate.rs` 增 `machine_gate` → 三族接入 → 并发/隔离/陈锁三态实测。
  - 验证：§6 T-02 三场景。
- [ ] **T-03** book 脏态守卫（依赖：无）
  - `book_listing_tests.rs` 公共守卫 helper + 文件头契约注释 → 脏/净/缺三态。
  - 验证：`cargo tb book_listing`（脏态 skip / 净态基线）。
- [ ] **T-04** worktree 组脚手架（依赖：无）
  - 新增 `scripts/new-wt-group.sh` → 自建自拆一轮全流程。
  - 验证：§6 T-04 全流程含 wt-guard。
- [ ] **T-05** 计时回执与收尾（依赖：T-01..T-04）
  - 修复前后 `cargo t` 计时（同竞争态标注）落 §9；SD 落点核对；
    P707-R1 销号材料备齐。
  - 验证：§9 回执在档。

## 9. 复审记录

- 2026-10-02 /auto-plan:new 起草：`stage: new`，`outcome: pass`（授权=2026-10-02
  用户分批裁定在 §4），`next: work`（worktree：`D:/autostack/.wt/lang-726/auto-lang`
  分支 `plan-726-dev`）。别名闸门方案否决记录见 §2 T-02。

## 10. 待澄清事项

1. T-01 勘定若超出 plan707 消息桥嫌疑域（需动 async 派发架构）→ 不扩围，
   回报 needs_replan 或登记新域再定。
2. 机器锁目录定 `D:/autostack/.locks/`（机器级、跨仓可见）；如用户希望换
   位置（如各仓 .locks）请在 work 前示下，缺省按上执行。
3. th 档（真 TCP 串行，~50s）暂不入闸——固定端口族已改 OS 临时分配（在案），
   如需一并入闸属小改。
4. book 守卫对"book 目录存在但非 git 仓"形态维持现状（不新增失败面）。
