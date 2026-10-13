# PLAN-742 r3 修复（work 第四轮）证据包

绑定：worktree `D:/autostack/.wt/lang-742/auto-lang`，分支 `plan-742-dev`，
base = r3 审核面 `8c9b2d3d9`；修复提交 `2c40e1fe4`。
生成器 CLI（提交态）SHA256
`211d5eb87a80f8d21f7564124ee0286b3a20e42199932f8b0caaddac4a5a2e75`。

## 内容

- `replay-r3-generated-results.json`：审核 r3 replay.ps1 -Build +
  execute-generated.ps1 于提交态重放——**17/17 compile_exit=0**，逐项对拍：
  13 个旧例保持；r3 两个反例翻转——`marker_literal` 返回
  `"__AUTO_A2R_STD_QUAL__ __AUTO_A2R_STD_SQLITE__ __AUTO_A2R_STD_SYS__"`
  逐字节保持（此前被恢复步改写）；`nested_borrow` 编译+运行 ok（此前 E0308）；
  `qualified_literal`/`borrow_positive` 控制组通过。
- `s10-full-lifecycle-transcript.log`：完整生命周期转录（R3-03）——
  后端 exe SHA256 `e58396eb…` + 生成器 CLI SHA + generation.json 时间戳；
  隔离 setup → 创建（request_id 契约全文往返）→ 强杀前读回（PRE_COUNT 7
  HAS True）→ **存储身份**：inbox.db SHA256 `ec850326…` 强杀前后一致 →
  backend PID 28848 taskkill /F → 确认下线 → 同二进制重启（新 PID 47932）→
  同目录 setup → 读回 POST_COUNT 7 HAS True STATE active。
- `vue-smoke-13of13.log`：Vue 联动（R3-03）——提交态 CLI `auto run` 双端
  （后端 17819 + Vite 17818），Playwright smoke **13/13 passed**
  （NOTES_SEED=1 + 净库数据目录；首轮 5 过 8 败为净库外残留数据所致，
  重置后全绿）。环境解锁：pnpm 出站 fetch 全镜像 UND_ERR_DESTROYED（curl
  对同镜像 200，判定为本会话 pnpm fetch 层故障）→ `--offline` 走本机
  warm store；file 依赖 vendor/auto-down 子模块初始化至钉住 c39ac7f9。

## VM 面（如实边界）

- 已验证：`auto run -r vm` 以提交态 CLI 启动——后端再生成 + "Running VM
  interpreter UI (backend: rust, split over HTTP)" + "entering vm
  interpreter"，后端 API 种子数据可读（/tmp/p742-vm-try.log）。
- 未运行：VM autotest 19 场景套件。前提：有头桌面 MCP 会话——
  autoui-verifier `test_vm_mcp.py` 两轮尝试（90s/300s 等待窗）中 VM 窗口
  进程在本无交互会话退出、MCP server 未就绪。此为 NOTES-001 交付面的
  复验项，不在本计划 §4 原验证清单（auto run + v1 电池 + legacy 冒烟）内，
  如实记录不冒称。

## 门禁

- `cargo test -p auto-man --lib api_gen`：**48 passed / 1 ignored**（×2 稳定；
  新增 test_qualify_a2r_std_code_regions_only fn 级单测；矩阵 oracle 扩
  marker/nested 两端点，13 值编译+运行全绿）。
- `cargo tt --no-fail-fast`：**5649/5649**。
- rustfmt 触面干净；742 新增 dead-code 零。
