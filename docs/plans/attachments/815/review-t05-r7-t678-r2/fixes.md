# PLAN-815 T-05 R7 / T-06..T-08 R2 修复轮交接（2026-10-22）

修复提交 `e351cb2a73a78f353457e56ea20cb82f61c1ac88`（code，4 文件
+208/-57）+ 本 docs 提交（无生产 diff）。五项 closure：

| Finding | 修复与永久反例 | 当期结果 |
|---|---|---|
| F-815-T06-R1-01（P2 Rust front 未贯穿） | `front_dir_for(project, source)`：声明优先/回退 find_front_dir——贯穿 compile_rust_ui_app、regenerate_code_only、needs_regeneration（增 source 参）、generate_rust_ui_delivery_inner、run_rust_ui_inner 运行 CWD（原先固定 src/front） | 回放 rust-front-cwd：自定义目录请求 run **completed 且 cargo CWD=ui/front**、argv 抵达（原 11ms "Front directory not found: …/custom/src/front"）；默认目录正控保持 |
| F-815-T06-R1-02（P2 Ark ID 带标点） | `json5_string_value`：`"k": "v",` → 尾逗号剥净后再剥引号（`trim_matches('"')` 对 `"v",` 留 `v",` 的根因）；bundleName/mainElement 缺失/不可读=具名 Err（删除默认猜值——不静默启动错误应用）；新增解析单测（生成器真实形状：带尾逗号/无尾逗号/非串/空值/异键） | 回放 ark-launch：全链抵达，`-b com.example.t05probe -a EntryAbility`（原 `…t05probe",` / `EntryAbility",`）；RunOk completed（私有 hvigor/hdc 受控） |
| F-815-T07-R1-01（P2 dist 引用假绿） | 断言重写：html 的**每个** `assets/` 引用必须解析到存在文件（token 分割扩至引号/括号/空格；引用名支持 `/` 嵌套——原 take_while 不含 `/` 产生 assets/assets 的根因）+至少一真实引用；删除 `contains("<div id") && bundles>0` 兑底 | 回放 dist-reference-oracle：valid exit0；**broken-reference（MISSING.js）exit101 具名**；**no-bundle-reference exit101 具名**；真实构建正控 lifecycle_build_produces_dist 保持通过（dist/index.html+assets 束+引用核对） |
| F-815-T07-R1-02（P2 close 无界） | `close_and_wait`：EOF 后 try_wait 轮询 + `CLOSE_BOUND=10s` 界（超时 kill+panic 带日志尾巴诊断）；`Drop`：kill 后有界收尸（不再无界 `wait`） | 回放 lifecycle-close-bound：**timed_out=False、10.07s、exit101、panic "tool did not exit within 10s after EOF"**；正常 EOF 面保持 exit0（lifecycle 三测）。披露：首轮复放 34s 假象=审查同名探针包在共享 target 的 cargo 陈旧 fingerprint（旧 helper 复用），清除 `target/debug/.fingerprint/plan815-r7-close-probe-*` 后复现真实行为——非产品代码问题 |
| F-815-T05-R7-01（P2 PM 矩阵受缓存顺序影响） | 矩阵 legacy 断言改为**来源+缓存稳定**（值=进程 OnceLock 冻结语义——不假装能重读磁盘）；磁盘读取证明移至**新鲜子进程** `pkg_legacy_disk_probe`（父测试 spawn current_exe --exact，cwd=磁盘 pac 目录、无先前缓存）——免排序/免清缓存/不改产品缓存语义；`test_resolve_is_cached`/`test_command_names_are_consistent` 纳入 PM_STATE_LOCK | 审查原命令复跑：`cargo test --locked --offline -p auto-ui-tools --lib pkg::tests:: -- --test-threads=1` Windows **9/9** + WSL Ubuntu（--locked 同冻结源码）**9/9**（原均 7/8）；矩阵单独继承/合法 bun 回退环境与真实工具 bun 对照保持（pm-fallback 双环境 0/0） |

## 当期验证账

- 回放：rust-front-cwd / ark-launch / dist-reference-oracle /
  lifecycle-close-bound 四脚本全闭（fix-evidence/）；R6 pm-fallback
  连续性复验（双环境 0/0+真实工具 wire 无 pkg/disk npm→bun）。
- scoped：pkg 模块串行 Win 9/9 + WSL 9/9（审查原命令）、lifecycle 重串行
  3/3（含强化后正控）、tool_process 21/21+重串行 8/8、lib 177/181
  （4 预存在案+1 既有 leaky 负控）、legacy 2/2、consumer 5/5、闭包门
  PASS、fmt 全净。
- 777：本轮代码改动不触 manifest/依赖边——R2 审查已独立复跑 pass
  （0 violations/3 grandfathered，base=d039f2399）；本轮提交后基线
  base_revision 待 merge 前统一按受管流程更新（edges 预期不变）。
- 测试时序裁定遵守：未整档；merge 门 pending。T-07 完整 792/fresh/
  locked 发行、资源审计、V-01..04 全矩阵、协议 fixture 服务端回执/
  深度负控仍为已承认待办。
