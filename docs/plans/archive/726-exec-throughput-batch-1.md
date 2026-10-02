---
plan_id: PLAN-726
status: archived                # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: exec-throughput-batch-1
author: [agent]
created_at: 2026-10-02
updated_at: 2026-10-02

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 空（测试基建+工作流工具，未触 docs/specs 模块契约；复审记录内有空值理由）

affects: [auto-lang]   # T-02/T-03 测试基建 + T-04 脚本；T-01 经查已由 PLAN-724 清偿（本计划零 vm 代码改动）
current_step: 5
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
- 重型测试族（画廊围栏 / 1M churn）多进程并发时第二实例快速失败，
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
  panic 输出"heavy family X already running (PID N, cwd W)"。接入点=两族
  重测试首行（画廊围栏 + 1M churn，与既有 heavy_gate 同位；aavm XL 族经
  2026-10-02 用户裁定豁免——aavm/aa2r 未来路线拟取消/替换，相关测试短期
  不再运行，不接入）。**别名方案已否决**：cargo 别名只能
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
**追加裁定（2026-10-02，合同修订）**：aavm/aa2r 未来路线拟取消/替换，相关
测试不再作为关注点（短期不运行）——本计划去掉 aavm 接入与 taa 验证面，
T-02 收敛为两族（画廊围栏 + 1M churn）。

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
- 接入两族重测试首行：画廊围栏族（`gallery-fence`，
  `widgets_gallery_all_front_pages_compile`）+ 1M churn 族（`churn-1m`，
  `str_churn_bounded_large`）。aavm XL/LG 族（`.config/test-mem-weights.md`
  清单，原稿 17 站点）经 2026-10-02 用户裁定豁免：aavm/aa2r 未来路线拟
  取消/替换，相关测试短期不再运行，不接入（曾按原稿接入后依裁定回退；
  恢复运行时补接线即可）。
- CI（Linux 独立机）锁路径机器隔离，天然零影响；日常档（t/tv/tt/tb/tu）
  不触两族，零开销。

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
| SD-02 | add | AGENTS.md §并行纪律（工作流规约，非 specs 树） | 重型族（画廊围栏/1M churn）"单实例纪律"约定 → 机器级闸门机械强制（aavm 族豁免在案） | 2026-09-30 互毁实录 | AC-02 |
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
- **AC-02**：两 heavy 族（gallery-fence / churn-1m）并发第二实例立即确定性红
  （含持锁者 PID/cwd 提示），隔离单跑绿；日常档（t）时长与行为零变化。
- **AC-03**：book 仓脏态→book_listing 族确定性 skip（非红非 fail，原因可见）；
  干净态全量行为不变（基线对照跑）。
- **AC-04**：`new-wt-group.sh` 一键建组含 auto-down 兄弟，组内
  `cargo check -p auto-lang` 直接通过；wt-guard clean；零 junction。
- **AC-05**：修复前后计时回执（日常档 62s→目标 ~32s，标注竞争态）落 §9；
  AGENTS.md 资源表数字刷新项移交 merge 阶段。

## 8. 执行步骤

- [x] **T-01** e4 直驱臂修复（依赖：无）
  - 复现+勘定（§5 T-01）→ 修复 `async_http.rs`/`stdlib.rs` 直驱臂 →
    `cargo t default_headers` 绿 → `cargo tv` 零回归 → 误标更正注记。
  - 验证：`cargo t default_headers && cargo tv`。
  - **work 阶段调整（2026-10-02）**：前提已过时——P707-R1 已被 PLAN-724 T-02
    清偿（commit `2b3736ccf`，2026-10-02 09:16 入 master，早于本计划起草
    `ca00819a8`）。§2 嫌疑域定案：**非** `submit_detached_client_job`（
    async_http.rs:364）消息桥派发缺陷，桥分离逻辑完好；真因=e4 测试三臂
    直接提交 managed job 未先 `register_live_op`，被 707 cancel-before-install
    闭合中止（测试侧协议违规）。本计划零代码改动，收敛为验证回执：
    `cargo t default_headers`→1 passed 0.394s（worktree@17292c07e，冷编译
    2m 另计）；`cargo tv`→162/162 绿 1.9s。AC-01 材料在案：KNOWN-DEBT
    P707-R1 已划销（2026-10-02 PLAN-724 T-02）；4f123a50e 误标更正已在
    `ea2fbf0df` ⑤ 与债务条目内。
- [x] **T-02** heavy 机器闸门（依赖：无，可与 T-01 并行）
  - `heavy_gate.rs` 增 `machine_gate` → 两族接入（画廊围栏 + 1M churn；aavm
    族依 2026-10-02 追加裁定豁免）→ 并发/隔离/陈锁三态实测。
  - 验证：§6 T-02 三场景。
  - **work 回执（2026-10-02）**：`machine_gate` 落地（O_EXCL + `pid|cwd|family`
    + Windows 裸 FFI OpenProcess/Unix kill -0 存活核验 + 死进程陈锁接管 +
    同进程重入协议违规红 + 半写窗口重读）；接入 `gallery-fence`（SKIP 路径
    不取锁）与 `churn-1m` 两族；模块拆除 test-aavm 门并升 pub(crate)
    （跨树引用：lib.rs 顶层 gallery / crate::vm churn / crate::tests）。
    单测 4 态（roundtrip/conflict/stale/reentry）5/5 绿 0.13s 零 LEAK（首版
    conflict 测试 panic 后未收割 sleeper 被 nextest 标 LEAK，已改
    catch_unwind→收割→resume_unwind）。真机三态（churn-1m 滤串，full 配置）：
    隔离单跑 PASS 13.0s；P1 持锁期 P2 确定性红 0.067s（heavy_gate.rs:137
    "already running (PID 23812, cwd=…worktree)"）；taskkill 持锁进程后
    陈锁留存、P2 接管 PASS 11.0s。日常档两族均被 default-filter 排除零开销，
    闸门单测 +0.13s（计时回执归 T-05）。**原稿曾按 XL/LG 清单接入 aavm
    17 站点，依 2026-10-02 追加裁定全部回退**（diff 不再触 aavm 路径，
    taa 复审触发条件随之消除；恢复运行时补接线即可）。
- [x] **T-03** book 脏态守卫（依赖：无）
  - `book_listing_tests.rs` 公共守卫 helper + 文件头契约注释 → 脏/净/缺三态。
  - 验证：`cargo tb book_listing`（脏态 skip / 净态基线）。
  - **work 回执（2026-10-02）**：`book_repo_dirty()`（OnceLock 单探测 + `git
    -C <root> status --porcelain`）落 `test_book_listing` 首行；排除项=`??
    …main.wrong.rs`（本族 mismatch 转储的自家产物，不排除则红一次后书仓
    恒"脏"恒 skip——实测发现后补的设计修正）；generate_book_expected
    （编辑期再生成工作流）不经守卫。验证（干净克隆 + AUTO_BOOK_LISTINGS
    env 注入，`book_ch` 确定性滤串）：净态 68 测=52 过/16 红，双跑逐字一致
    （排除项实证）；+1 未跟踪文件→68 全 skip 0.55s；删文件→复现 52/16
    零滞回；真实书仓（当前 M 脏，PLAN-715 实况）→68 全 skip 0.54s，消息
    含脏项数与样例；缺失路径→原读失败形态（os error 3）不变。**预存发现
    （非本计划范围）**：`generate_book_expected` 无条件改写全部 expected 且
    与对拍族在 tb 档同跑竞态（跨进程读写 expected.rs，红集非确定：同克隆
    实测 7 红/16 红两采样）——先于本计划存在，复审时登记债。
- [x] **T-04** worktree 组脚手架（依赖：无）
  - 新增 `scripts/new-wt-group.sh` → 自建自拆一轮全流程。
  - 验证：§6 T-04 全流程含 wt-guard。
  - **work 回执（2026-10-02）**：脚本落地（组名白名单防穿越/组目录已存在
    即拒/分支已存在即拒/plan-NNN-dev 取号提示/失败中途回滚不留半组/建组
    基面恒钉 master——主检出与 auto-down 兄弟经 porcelain 首行解析，
    可从任意 auto-lang 检出运行；首版"必须主检出运行"护栏实测踩 msys
    `/d/` vs `D:/` 路径形态错配且致自测鸡生蛋，重构撤销）。自测全流程：
    worktree 内运行→建 `fix-scaffold-selftest` 组（lang 分支+down detached）
    →组内 `cargo check -p auto-lang` **直接通过**（54s 冷态，兄弟仓解析即
    开即用）→wt-guard 双仓 clean→双 worktree remove→删分支→rmdir 组。
    负例：已存在组名 exit 1；缺参 usage exit 1。
- [x] **T-05** 计时回执与收尾（依赖：T-01..T-04）
  - 修复前后 `cargo t` 计时（同竞争态标注）落 §9；SD 落点核对；
    P707-R1 销号材料备齐。
  - 验证：§9 回执在档。
  - **work 回执（2026-10-02）**：
    - **计时**：裸 `cargo t --no-fail-fast`（worktree@2feb50a6c，轻度竞争——
      8 组并行 worktree 在场）= **54.1s 测试段 / 55.2s wall**，4976 测
      4962 过 14 红（全为预存，见归因）。对照 2026-09-30 基线 62s（含 e4
      30s 长尾）：e4 尾已消（现 0.39s 绿），但基面 17292c07e 的 14 预存红
      中 plan606（10.2s）/schema_drift_fence（10.5s）/plan707 flake（5.1s）
      的红时长并入读数；fail-fast 形态 28.2s。AGENTS.md 资源表数字刷新
      移交 merge（AC-05 在案）。
    - **红集归因（14 红零归因本 diff）**：stash 严格对照——pristine 基面
      scoped 9 红 vs 带 diff scoped 9 红**名单逐一相同**；musk p053 三红在
      当前 master 主检出复现（master@991c0197e）；plan707 帧时序与
      p053_6 为负载敏感 flake（scoped 双侧绿；flake 族在案 4f123a50e）；
      11 红已由今日 PLAN-725 合入在 master 修复——14 红族=§1 非目标在档的
      THR-D 第二批范围。本 diff 新增/触面测试全绿（heavy_gate 5/5、e4 绿、
      book 四态、tv 162/162）。
    - **SD 核对**：SD-01=无规范增量（T-01 调整为验证回执，无契约文本变化，
      理由在档）；SD-02=AGENTS.md 并行纪律注记（merge 期落）；SD-03=文件
      头契约注释已落码（book_listing_tests.rs）；SD-04=AGENTS.md §1/§5 入口
      规约（merge 期落）+ 脚本已入库 scripts/new-wt-group.sh。
    - **P707-R1 销号材料**：KNOWN-DEBT 已划销（PLAN-724 T-02，2026-10-02）；
      4f123a50e 误标更正注记在 ea2fbf0df ⑤ 与债务条目内——零补作。
    - **复审门禁**：裸 `cargo t`（跑讫，如上）；触面档 tv=162/162 绿；
      book 档=scoped book_listing 族四态验证（**刻意不跑裸 tb**：
      generate_book_expected 无条件改写真实书仓 expected——用户 ch06 WIP
      在场，裸 tb 会覆盖之；该预存破坏性风险登记复审定案）；aavm 触发
      条件经裁定回退后 diff 已不触 aavm 路径，taa 免。
    - **预存发现（复审登记候选）**：①裸 `cargo tb` 经 generate_book_expected
      无条件改写外部书仓（WIP 覆盖风险 + 与对拍族跨进程竞态致红集非确定，
      同克隆实测 7/16 红两采样）；②stage3.rs 内 K32GetProcessMemoryInfo/
      EnumWindows 双声明签名不一致预存警告（本 diff 的 OpenProcess 声明已
      对齐 isize 形态规避）。

## 9. 复审记录

- 2026-10-02 /auto-plan:new 起草：`stage: new`，`outcome: pass`（授权=2026-10-02
  用户分批裁定在 §4），`next: work`（worktree：`D:/autostack/.wt/lang-726/auto-lang`
  分支 `plan-726-dev`）。别名闸门方案否决记录见 §2 T-02。
- 2026-10-02 /auto-plan:work 执行完毕：`stage: work` | PLAN-726 | base=17292c07e
  | `outcome: pass` | code_commit=`2feb50a6c`（plan-726-dev）| tasks=T-01..T-05
  （5/5，T-01 依证据调整=PLAN-724 已清偿、aavm 面依追加裁定移除并回退）|
  evidence=各任务回执（e4 0.394s 绿+tv 162/162；机器闸门真机三态；book 四态；
  脚手架自建自拆含 wt-guard；裸 cargo t 54.1s/14 预存红零归因本 diff——
  stash 严格对照在档）| blockers=无 | next: review（裸 cargo t 已跑讫；
  tb 裸跑避让理由与 aavm 免触发理由见 T-05 回执；复审需定案两项预存发现
  登记去向）。worktree/分支保留待 review/merge。
- 2026-10-02 /auto-plan:review（同会话复审——独立性限定在案：结论自工件
  重建，五 AC 全部复审态重跑取证）：`stage: review` | PLAN-726 | plan_revision
  =draft 修订后（2026-10-02 追加裁定：aavm 面移除）| `outcome: pass` |
  reviewed_commit=`2feb50a6c`（worktree clean 核验）| base=17292c07e |
  deps=auto-down 兄弟 detached@895f8d0（组内解析）| spec_inputs=vm/overview.md
  无直驱臂文本（grep 空）→SD-01 无规范增量成立 | acceptance=**AC-01 pass**
  （e4 PASS 0.412s 复跑；P707-R1 划销在档；4f123a50e 更正在 ea2fbf0df⑤；
  修复归属 PLAN-724 T-02，本计划验证回执）；**AC-02 pass**（冲突态复跑
  0.066s 确定性红含持锁者 PID 6648、P1 独占绿 19.96s；隔离/陈锁=work 真机
  实录+单测 5/5 复跑 0.130s；日常档结构性零开销）；**AC-03 pass**（真实
  书仓脏态 68/68 skip 0.546s 复跑；净/脏/还原/缺失四态=work 克隆实录；
  wrong.rs 排除项设计修正有据）；**AC-04 pass**（全周期复跑：建组→组内
  check 1m29s→guard clean→移除→rmdir）；**AC-05 pass**（计时回执在 §8
  T-05：54.1s/55.2s 轻度竞争 vs 62s 基线；AGENTS.md 资源表刷新归 merge）|
  findings=零阻断项；F-R1 门禁补强——work 期未跑 tb 全面的缺口经
  `-E 'not test(generate_book_expected)'` 安全滤式补跑闭合（1522/1525，
  3 红=已归属 master 的 musk p053 预存三红）；两项预存发现已登记
  P726-R1（generate 破坏性+竞态）/P726-R2（stage3 FFI 双声明）| evidence=
  以上命令/结果均 @2feb50a6c 重跑实录；红集归因链（stash 同名对照+master
  复现+flake 族在案 4f123a50e）在 §8 T-05 | next: merge（SD-02/SD-04
  AGENTS.md 沉淀+资源表刷新+归档+worktree 清理）。**frontmatter spec 字段
  空值理由**：本计划为测试基建+工作流工具，未触任何 docs/specs 模块契约
  （vm 行为零变化，SD-01 论证在档）；touched_goals 无对应 GOAL-NNN。
- 2026-10-02 /auto-plan:merge 收据（`PLAN-726:r<draft+aavm 修订>`）：
  `stage: merge` | `outcome: pass` | **prepared**=reviewed 基线 2feb50a6c +
  AGENTS.md SD 沉淀（纯文档后代 0b042191b→rebase 后 529fb8099；实现/依赖
  零变化）+ 空 spec 增量论证（AGENTS.md 为工作流规约载体，在案先例=
  fix-test-tiering/fix-ui-tier 等计划沉淀惯例）；**landed**=rebase 上
  master（range-diff 补丁恒等：旧 2feb50a6c = 新 8cbd330d7）→ 主检出
  `git merge --ff-only plan-726-dev`，master tip=529fb8099=dev 头（无
  merge commit），落地后主检出冒烟 `cargo t heavy_gate` 5/5 绿；主检出
  在途脏文件仅他会话 723/727 簿记（共享簿记例外，原样保留）；**ledger
  _refreshed**=no-op（有据：spec 增量为空，无 docs/specs 变更可投影；
  `.autoos/specs.json` 未手写——本会话无 store-mediated 写入面且无需
  写入）；**archived**=`git mv docs/plans/726-exec-throughput-batch-1.md
  docs/plans/archive/` + `status: archived`，`completion_kind: delivered`；
  **cleaned**=（见下）；**部署面核查**=N/A 有据——改动全部为 `#[cfg(test)]`
  测试基建+脚本，release 二进制/依赖仓 daemon/web bundle 零消费面；
  **批量回归到期判定**=due（回执 last_covered_plan_id=715 < 已落地的 725
  且 725%5==0；亦超 48h）→ 移交 `/auto-plan:regress` 主检出单实例执行；
  **cleaned**=worktree clean 核验 + wt-guard 双仓 clean（lang+down）→
  `git worktree remove` ×2 → `git branch -d plan-726-dev`（删于 529fb8099
  =已落地 tip）→ 组目录 rmdir——全零残留。

## 10. 待澄清事项

1. T-01 勘定若超出 plan707 消息桥嫌疑域（需动 async 派发架构）→ 不扩围，
   回报 needs_replan 或登记新域再定。
2. 机器锁目录定 `D:/autostack/.locks/`（机器级、跨仓可见）；如用户希望换
   位置（如各仓 .locks）请在 work 前示下，缺省按上执行。
3. th 档（真 TCP 串行，~50s）暂不入闸——固定端口族已改 OS 临时分配（在案），
   如需一并入闸属小改。
4. book 守卫对"book 目录存在但非 git 仓"形态维持现状（不新增失败面）。
