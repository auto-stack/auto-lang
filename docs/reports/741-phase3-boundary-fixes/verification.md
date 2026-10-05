# PLAN-741 Phase 3(r3)边界修复验证报告(2026-10-05)

r3 基线:worktree `D:/autostack/.wt/lang-741/auto-lang` 分支 `plan-741-dev`,
基线提交 `ed2d00b90`(v0.6-dev 含复审方 r3 再激活提交 fa3abe476)。
源码指纹见 [source-hashes.txt](source-hashes.txt);native.rs/main.rs 相对 r2
零改动(本轮未触)。跨仓依赖按组内布局建只读兄弟检出
`D:/autostack/.wt/lang-741/auto-down`(detached @ fba6563e,与 r1 相同,
无 junction/symlink;fold 时随组清理)。

## 修复摘要与 AC 对账

| 项 | 修复 | 验证 |
|---|---|---|
| R2-QA-01(P1) | verify.rs `check_block_graph`:块包含边(If then/else、Loop body)显式栈 DFS 可达性 + 入口零入边/其余唯一入边;结构失败阻止数据流走查;walk_block 加防御性 visited 防护。数学论证:入边不变量满足时,可达 ⇒ 无环(环必给某节点第二个父或使入口在环上);断开 SCC 由可达性拒绝 | 两份复审输入物化为 `fixtures/invalid/block-{self-cycle,disconnected-cycle}.atom`;hir_verify 拒绝矩阵 +2;cli 反例实测 check/build exit 1、定位 span、0.14s 返回(<5s)、无制品 |
| R2-QA-02(P2) | link.rs `run_with_deadline`:单个单调 deadline 同时覆盖进程等待与输出收集(通道 + recv_timeout);Windows Job Object(KILL_ON_JOB_CLOSE)管控子树,任何出口 drop 即杀后代→管道 EOF→reader join 回收,不再 detach 永久阻塞线程;新增 `DeadlineCollect` 诊断变体(同 link.deadline 码);reader 改 read_to_end+lossy(修复 read_to_string 全有全无 UTF-8 校验在本地化输出下丢弃合法前缀=截断成功输出) | 新增 4 测试:1s 收集截止(后代持管道,3s 内拒绝,tasklist 证实后代被回收)、后代即时退出成功正例、60s 生产 deadline 证据测试(`--ignored`,显式跑 60.01s 拒绝,对比复审实测 114.96s 挂起)、既有 1s 杀挂起+126KB 大输出回归保留。复审 reproduce.ps1 watchdog 实测 60.07s、stillRunningAfterDeadline=false |
| R2-QA-03(P2) | publish_artifacts 暂存所有权:`discard_staged` 统一回收 tmp_exe/tmp_obj/tmp_receipt 覆盖**全部失败出口**(收据暂存/备份/三次发布);回滚自身受阻时返回 `link.restore` 诊断列出幸存备份路径,不吞错、不为无残留删唯一旧制品;用户占位目录永不删;link_object_staged 执行器错误路径同步回收暂存 exe | 单测失败矩阵 5 例(收据暂存失败±旧制品、备份失败、exe/obj 发布失败):旧三件套字节不变、零自有 .tmp/.bak、占位目录幸存;复审 helper 实测 `staged_exe_left=false;old_exe_unchanged=true;old_obj_unchanged=true` |
| R2-QA-04(P3) | 状态簿记:复审方已激活(archived→active、executing/r3);plans.md/ledger 指针已同步 active 路径;最终归档留给 merge(收据与 frontmatter 一致性在 merge 收据复核) | 本报告所属执行轮不归档;README 计划链接按 AC-13 生命周期指向 active 路径 |

## 门禁证据(全部 worktree 实跑,2026-10-05)

- 原型全族:**52 项绿 + 1 ignored 证据项**(text_binding 11 + hir_verify 9(27 反例)+
  native_execution 11 + cli 11 + trace_execution 1 + lib 9;r2 基线 44 → 新增 8 项
  全为 r3 反例/矩阵);`cargo fmt -- --check` 干净;`cargo check --all-targets`
  零 warning。
- 一键 `verify-ac-741.ps1`:原型段 13/13 PASS(见门禁收据);主仓门禁:
  `cargo check -p auto-lang` PASS(组内 auto-down 兄弟检出建立后);
  `cargo tv` 162/162 PASS;`cargo t` 见下方同基线归因。
- 生产面零 diff:`git diff ed2d00b90 -- Cargo.toml Cargo.lock crates/ test/` 为空。
- 复现脚本:
  - r3 复审 [r2-review-reproduce-rerun.json](r2-review-reproduce-rerun.json)
    (完整含 watchdog):块图两例 exit 1 + verify.block-structure 定位;baseline
    exit 0;receipt-prepare `staged_exe_left=false;old_exe_unchanged=true;
    old_obj_unchanged=true`;watchdog configuredDeadlineSeconds=60、observed
    60.07s、stillRunningAfterDeadline=false。
  - r2 复审 reproduce.ps1 复跑:六反例保持翻转、发布失败保旧制品、旧 exe exit 5
    (无回归;readme 段为旧命令形式历史记录,exit 2 预期)。
  - 注:20261005 脚本的 helper 编译需 `windows.0.52.0.lib`(本轮新增 windows-sys
    依赖链),直接 rustc 编译不经 cargo,需 `LIB` 指向
    `~/.cargo/registry/src/.../windows_x86_64_msvc-0.52.6/lib`——复审复跑时同此设置。

## cargo t 同基线归因(THR-D3/THR-D4,零新增)

方法与 r1 P741-R1 相同:双侧同基线全量 `cargo t --no-fail-fast`(本轮两跑并行,
负载高于 r1 对照,红集整体偏大属预期):

- 基线(主检出,v0.6-dev 同一 crates 提交):32 failed + 1 timeout = 33 红
  ([reds-main-baseline.txt](reds-main-baseline.txt))
- worktree(r3 改动):34 failed + 1 timeout = 35 红
  ([reds-worktree.txt](reds-worktree.txt))
- 对称差 8 例全部为 plan484/plan492/plan498/plan499/rqhost/plan705/plan716
  ——THR-D3 既有清偿族与 THR-D4 负载 flake 模式在两次并行重跑间的自然翻转;
  30 例完全共有(musk p053×4/p054×2、dep_parity_017、ffi_dual_017、
  plan498×4、plan499×3、plan484_024、plan502_m3、plan503、plan606_029、
  plan643、plan437、plan707、projector_counter、docs_gen/schema_drift/ash_leak
  围栏环境族等)。
- 归因结论:`crates/**` 零 diff + ac-core 独立 workspace(auto-lang 零依赖,
  AC-01 已锁)⇒ worktree 红集与基线红集同族,**零新增可归因于 r3 改动**。

## 遗留

- 60s 证据测试 `run_exe_60s_deadline_bounds_descendant_pipe_hold` 为
  `#[ignore]`(默认门禁不跑),证据见上;一键脚本 --lib 步已串行化。
- 复审脚本 helper 编译的 LIB 注记已写入本报告,复审复跑沿用即可。
