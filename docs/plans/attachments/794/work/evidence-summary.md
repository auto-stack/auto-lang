# PLAN-794 T-05 实机/外部证据收据（work 轮）

基线：worktree `D:/autostack/.wt/lang-794/auto-lang`，branch `codex/plan-794-dev`，
未提交工作树（代码同冻结前最终态）；auto 构建档 = 同树 `target/debug/auto.exe`
（`cargo build -p auto`，SCCACHE_SERVER_PORT=42917 绕开本机 sccache 端口保留段问题）。
探针均为现有 fixture 原样运行（零 golden 修改、零 case 移除）。

## V04 双端定向交互（autoui-verifier 探针族）

| 探针 | 腿 | 结果 | 备注 |
|---|---|---|---|
| p781-scroll-child-anchor `tests/vm_probe.py` | VM(iced, MCP) | **ALL 57 CHECKS PASSED** | 全 781 状态机（queued/pending/applied/invalid:not_found/not_bound/unsupported/重复覆盖）+ v06 旧 offset 族（start/by/end/用户滚动不回拉/managed 拒绝）经新 provider 链 |
| p781-scroll-child-anchor `tests/vue_probe.mjs` | Vue | 首跑 58/59，**二跑 59/59** | 首跑唯一红=`no_page_errors`（`auto-select/overlay.ts` 动态导入 fetch 失败——与 scroll 无关的 dev-server 懒加载竞态；全部 scroll 功能断言两跑全绿）；二跑报告存档 `p781-vue-report-r2-59of59.json` |
| p656-scroll-pane `tests/vm_probe.py` | VM(iced, MCP) | **ALL P656 VM CHECKS PASSED** | 六旧函数全链（native→intent 队列→update 排空→iced scroll_to→on_scroll 测量→注册表快照→`scroll_state()` 读出）：ordinary probe=102、managed mprobe=9_999_776、on-scroll 观察 oy=60/py=0.82 |
| p656-scroll-pane `tests/vue_probe.mjs` | Vue | 环境性跳过 | 预存缺口：该 fixture 无 `gen/`（backend 生成路径缺失，`examples/rust-workspace/p656-*-back` 不存在且 rust-workspace 为 git 忽略生成物）——非本批触面；旧族 Vue 面已由 781 Vue v06（start/by/end/不回拉/managed 全绿）冗余覆盖 |

## V01/V02 外部 no-ui consumer

`cargo test --manifest-path test/fixtures/modularity/scroll-provider-consumer/Cargo.toml`：
**5/5 通过**（独立 manifest、default-features=false、零 UI 类型 provider 实现）。

## V02/V03 in-crate

`cargo t plan794`：**6/6 通过**（剪断锚/ID 钉扎/缺席降级/fake 委托精确参数/
Clone-merge-两 VM 隔离/真实 UI adapter→注册表接线含 781 失锚-驻留-重入队语义）。

## 777 删边检查（T-05）

`scripts/modularity-audit.py --mode observe` 双树对照（工作树 vs 主检出 master）：
`core_to_ui` 文本候选 code 类 **570 → 549（-21）**，净 -20（string 类 +1 为
分类启发式计数波动；test/doc 类不变）。八 scroll shim 的 `crate::ui::scroll`
直引全部删除；新增 4 处 `crate::ui::scroll_host_provider::install_process_default()`
为 composition-root 合法接线（lib.rs 统一 installer + run UI 入口 + VmBridge
两构造，与 779/785/790 同形——lib.rs 两处在 master 基线即有同类 install 调用）。
code_editor 滚动族（9911..9913，非目标）直引保持原样。

## 环境注记

- sccache 默认端口被 Windows 端口保留段阻挡（os error 10013）——全程以
  `SCCACHE_SERVER_PORT=42917` 绕开（与本批代码无关）。
- p781 vue 探针后端冷编译超过 CLI 内建 120s 窗口（fresh worktree 全特性
  重编）——预热一次后探针正常（收据二跑即预热后结果）。

## 附：p781 Vue 二跑（59/59）全 case 清单（原 JSON 被 *.json 母规则忽略，内嵌存档）

| case | ok | detail |
|---|---|---|
| v03_init_target_visible | PASS | iy=41.0 ix=806.0 scrollTop=0 |
| v03_init_state_applied | PASS | applied |
| v01_c50_queued | PASS | queued |
| v01_c50_visible | PASS | iy=526.0 h=1945/525 |
| v01_c50_tall_top_aligned | PASS | targetTop=362.0 paneTop=362.0 tall=1945>525 |
| v01_a80_queued | PASS | queued |
| v01_a80_visible | PASS | iy=526.0 |
| v02_p8_queued | PASS | queued |
| v02_p8_visible | PASS | iy=41.0 |
| v04_missing_queued | PASS | queued |
| v04_missing_state | PASS | invalid:not_found |
| v04_oob_queued | PASS | queued |
| v04_invalid_no_scroll | PASS | 492 -> 492 |
| v04_empty_key | PASS | error:empty_key |
| v04_bad_handle | PASS | error:bad_handle |
| v04_ghost_queued | PASS | queued |
| v04_ghost_not_bound | PASS | invalid:not_bound |
| v04_dup_queued | PASS | queued |
| v04_dup_last_wins | PASS | {"count":2,"iy":41,"text":"dup-B-late 同pane重复键乙。"} |
| v04_side_dup_queued | PASS | queued |
| v04_side_dup_visible | PASS | iy=33.0 |
| v04_no_cross_pane_scroll | PASS | 7219 -> 7219 |
| v04_side_only_queued | PASS | queued |
| v04_side_only_visible | PASS | iy=33.0 |
| v04_unknown_state | PASS | unknown |
| w01_std_handle_queued | PASS | queued |
| w01_std_colon_key_located | PASS | {"target":true,"pane":true,"iy":33,"ix":1342,"handle":"__scrollctl_1"} |
| w01_std_state_applied | PASS | applied |
| w02_nest_outer_queued | PASS | queued |
| w02_nest_outer_same_key_visible | PASS | iy=33.0 |
| w02_nest_no_cross_scroll | PASS | inner scrollTop=0 |
| w02_nest_outer_state_applied | PASS | applied |
| w02_nest_inner_queued | PASS | queued |
| w02_nest_inner_same_key_visible | PASS | iy=33.0 |
| w02_nest_inner_state_applied | PASS | applied |
| w02_inner_only_queued | PASS | queued |
| w02_inner_only_outer_not_found | PASS | invalid:not_found |
| w02_plain_inner_queued | PASS | queued |
| w02_plain_inner_outer_not_found | PASS | invalid:not_found |
| w02_mo_outer_not_unsupported | PASS | queued |
| w02_mi_inner_unsupported | PASS | error:unsupported |
| v03_ch2_swapped | PASS | {"hasC50":false,"hasQ5":true,"scrollTop":0} |
| v03_old_key_queued | PASS | queued |
| v03_old_key_not_found_on_ch2 | PASS | invalid:not_found |
| v03_new_key_queued | PASS | queued |
| v03_new_key_visible | PASS | iy=41.0 |
| v03_new_key_state_applied | PASS | applied |
| v05_large_a80_queued | PASS | queued |
| v05_large_a80_visible | PASS | iy=526.0 h=961 |
| v05_already_visible_stable | PASS | 123 -> 123 |
| v05_narrow_c50_queued | PASS | queued |
| v05_narrow_c50_visible | PASS | iy=526.0 |
| v06_old_start_zero | PASS | 0 |
| v06_old_by_moves | PASS | 320 |
| v06_old_end_large | PASS | 8083 |
| v06_anchor_applied_before_user | PASS | off=2706 |
| v06_no_pullback_after_user_scroll | PASS | anchor=2706 user=1806 stable=1806 |
| v06_managed_unsupported | PASS | error:unsupported |
| no_page_errors | PASS | zero |

## R1 修复轮补录（2026-10-16，绑定 d027e5f61）

| 项 | 命令/动作 | 结果 |
|---|---|---|
| F-794-R1-01 | `modularity-audit.py --refresh-baseline`（干净提交 7a566c89b）→ strict → 入库 d027e5f61 → strict 复跑 | declared_edges=43；strict **pass 0 violations**（2 既有 grandfather 保持），基线入库前后两跑均绿 |
| F-794-R1-02 | vm_probe.py expect_reply 重写 + 永久负控 + 重入队重放；`AUTO_BIN=<worktree>/target/debug/auto.exe python tests/vm_probe.py` | **ALL 60 P781 VM CHECKS PASSED**（×2 跑）；负控 `oracle_failed_press_stale_label_rejected` 确定性拒绝（stub press=False stale=applied）；`requeue_pending_to_terminal_replay` samples=['applied']（重入队确定走 pending 分支，终态经新鲜查询到达；样本序列如实入 detail） |
| F-794-R1-03 | 去 unused_mut 后 `cargo t scroll_host_provider` | **9/9 pass，零 plan794 文件警告**（grep 计数=0） |
| wt-guard | 356 junction 逐链接 `cmd /c rmdir` → 清 p656/p781 gen、rust-workspace、dist → 杀 vite node/esbuild/PID19128 遗留进程 → 重跑 guard | **wt-guard: clean, exit0** |
| 复审原红复现对照 | 复审 R1 原跑 56/57（v03_old_key_not_found_on_ch2 粘 pending） | 修复后同 case 绿（60/60 内） |
