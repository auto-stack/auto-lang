# PLAN-693 实机证据——a2r exe 三形态（AC-01/AC-03）

载体：001-helloworld（worktree `D:/autostack/.wt/lang-693/auto-lang` 构建，
`auto build -r rust` → `target/debug/hello-world.exe`；共享 target-dir 指向
worktree 根 target/）。pac.at 自带 `desktop_render: "remote"`（PLAN-683 试点
遗留），恰好充当 sidecar pac 腿素材。

## 三形态（2026-09-23 实机）

| 形态 | 命令 | 观测行 | 证据 |
|---|---|---|---|
| rq（CLI 腿） | `AUTO_RQHOST_WELLKNOWN=p693-rq-a hello-world.exe --render-mode rq` | `[render] render-mode: rq (CLI)` + 采纳 p693-rq-a | rq-cli.png（daemon 窗渲 Hello, World!） |
| rq（sidecar pac 腿） | `pac.at` 置 exe 旁 + 无参跑 | `[render] render-mode: rq (pac desktop_render)` + 采纳 | sidecar-pac.png |
| desktop | `hello-world.exe --render-mode desktop --desktop-endpoint p693-desktop-b` | `[render] desktop 模式：采纳桌面合成器端点（不孵化）` | desktop-endpoint.png（daemon B 宿主窗） |
| independent（CLI 压 sidecar） | `hello-world.exe --render-mode independent --title "P693-独立窗"` | `[render] render-mode: independent (CLI)` + `Running with Iced backend` | independent-cli.png（自开窗+CLI 标题生效） |

## 其他验证行

- **`-q` 语义糖**：`hello-world.exe -q` → `[render] render-mode: rq (CLI)` +
  采纳 + `first frame` ✓。
- **AC-03 缺端点干净报错**（不孵化）：`--render-mode desktop`（无端点）→
  `Error: "desktop 模式需要 --desktop-endpoint <pipe>（未指定；探测 wellknown:
  p693-rq-a——请先启动虚拟桌面或补参）"`（列探测 wellknown，即时退出）✓。
- **exit-on-EOF**：daemon 先杀 → 客户端 HostLost 自退（rqhost 同款策略档）✓。
- **宿主 `-q` 全链**（孵化路径 + PLAN-683 env 链补通）：`auto run -r rust -q`
  → host `ensure_rqhost_ready` 孵化 daemon（默认 wellknown）+ 注入
  `--autodesk-render=queue`；子进程 gate 经 `AUTO_VM_RENDER=remote` env 腿
  （CLI 无、env remote > 注入 queue）进 rq 臂 → 采纳 + 480x320 窗 + perf 行
  ✓。此链此前 rust 轨断裂（生成 gate 不识 AUTO_VM_RENDER + ClientOpts E0063
  不可编），本计划补通。
- **桌面端点合同等价说明**：desktop 形态远端 = rqhost daemon 形态端点
  （adopt 协议同构——PLAN-693 契约即如此定义）；auto-os 桌面 shell 内嵌
  daemon = 跨仓配对项（T-04 后置，AC-04 待其就绪）。

## 测试门禁

- `cargo t render_cli`：8/8（解析矩阵/优先级链/旁车 pac/值域）。
- `cargo t client_entry`：3/3（Desktop 缺席报错不孵化 + 进程内 RqServe
  替身 adopt 通 + 既有冒烟）。
- `cargo t -p auto-man generated_main`：2/2（rqhost 既有 pin + 三模式臂
  新 pin，含 `remote: true`×2 + `remote: false` E0063 归位锚）。
- `cargo t desktop_protocol --no-fail-fast`：213 run，211 绿；2 红
  （`projector_counter_layout_and_hits` / `native_gate_runtime_views_of_six`）
  经 stash 对照 base 4dec4f698 复现同败 = 全预存（PLAN-690 记录吻合），零新增。
