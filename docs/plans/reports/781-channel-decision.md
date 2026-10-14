# PLAN-781 T-09 原生验收通道决策报告（合同冻结）

stage=work | task=T-09 | plan_revision=2 | 产出本文 = 阶段 A 通道合同冻结，后续 T-10（驱动）/T-11（最小观测增量）以本文为硬前置。

- 实施树：`D:/autostack/.wt/lang-781/auto-lang`，branch `plan-781-dev`，base = master@`6ff794eff`（树干净）。
- Reader 只读 consumer：`D:/autostack/auto-reader` @`29b5dec`（=立项 hash 29b5decf…，树干净）。
- 调查方式：3 只读探查（auto-lang MCP/几何面、Python 验收设施、Reader 应用结构）+ 本人源码裁决 2 处探查冲突（见 §1/§6）。
- 本文所有 file:line 相对 `D:/autostack/.wt/lang-781/auto-lang/`（auto-lang 侧）或 `D:/autostack/auto-reader/`（Reader 侧）。

## 1. 运行形态冻结（split 编排）

**验收通道唯一启动形态**（原生窗口 + 真实 HTTP 后端）：

```
cwd   = D:/autostack/auto-reader            （auto run 需在项目根执行）
env   = AUTO_READER_DATA=<隔离库绝对路径>    （数据目录唯一覆盖面，src/back/library.at:62-76）
        AUTOUI_MCP_PORT=<预选空闲端口>       （缺省 9247，见 §6）
argv  = auto run -r vm --no-merge -B <预选back端口>
```

- `-r vm` → `run_vm_ui`（crates/auto-man/src/automan.rs:1498-1528）；`--no-merge` → `AUTO_VM_MERGE=0`（crates/auto/src/main.rs:1304-1308）→ split_mode=true（crates/auto-man/src/rust_ui.rs:4705-4707）→ VM+VM split：**单进程双腿**——iced 前端主线程 + `auto-vm-server` 32MB 栈线程跑 `src/back/api.at`（rust_ui.rs:4729-4742、4329-4428；/health 就绪探测 120s 上限）。
- `-B` CLI 优先于 pac.at 的 `back_port:17825`（main.rs:1328-1365）→ `AUTO_HTTP_PORT`/`AUTO_HTTP_BASE` 自动注入；**`AUTO_HTTP_BASE` 由 run_vm_ui 自动设置（rust_ui.rs:4735-4740），驱动不自行设置**。前端相对 URL 经 `AUTO_HTTP_BASE` 展开（crates/auto-lang/src/vm/ffi/stdlib.rs:8116-8132）。
- **禁用形态（探查冲突 ① 的裁决）**：
  - `--server=vm -r vm` = **仅服务端、无窗口**（automan.rs:1499-1523 直接 `run_file(api.at)`）——Reader 测试脚本 `tests/spec/run_fresh_server.sh --vm-full` 用的就是它（供 t01–t06 curl 直连），不是本通道形态。
  - `-r vm` 不带 `--no-merge` = merged：**无 HTTP 后端**（rust_ui.rs:4743-4750），Reader 的 `Http.*` 全断，不可用。
- 进程拓扑 = **一个 Popen**（无 vite 子进程、后端是线程）→ 收割责任清晰（§7）。VM native runner 会把进程 CWD 切到 `src/front`（automan.rs:1440-1449 注记）→ `AUTO_READER_DATA` 必须给绝对路径。
- **风险 R1（明示）**：该"原生窗口 split"组合无既有运行先例（Reader 现行测试只跑 Vue 形态或 server-only 形态）。T-10 首个 smoke 即验证：书架页能经真实 HTTP 拿到隔离库书目。若形态本身有框架缺陷，按 work 技能 adapt-or-route 表处置（最小修复在通道范围内，超范围具名阻塞），不得静默换形态。

## 2. 实例身份与隔离

MCP 面现状：**无任何工具返回 pid/app 路径/数据目录**（18 工具清单见 §9；`std::process::id()` 仅用于最小化窗自查 mcp_server.rs:2492）。身份由驱动侧三件套建立，不新增跨进程身份工具：

1. **pid↔端口持有核对**：Popen 得 pid；`netstat -ano` 解析预选 MCP/back 端口的持有者 pid 必须等于子进程 pid（纯标准库解析，不引 psutil）。
2. **stderr 监听行断言**：子进程 stderr 落盘后解析 `AutoUI MCP: listening on http://127.0.0.1:<p>`，要求 `p == 请求端口`；出现 `fell back to`（bind 失败自动漂移 +1..+10，mcp_server.rs:637-671）或 FATAL 行 → 本 run FAIL（端口被占=TOCTOU 或孤儿实例，属实例身份违规，不重试连错实例）。
3. **内容级断言**：`autoui_vtree` 必须含本 run 隔离种子的唯一内容（书名/章节标题/段文），双实例（V-07）用互不相同的种子 → 内容断言即不串线证明。

补充绑定：新观测工具（§5）响应携带 `backend:"vm"`（来自 SharedState 的 BackendKind，renderer.rs:19102-19105 设置）——同一 `/mcp` 协议也被桌面 host（ui_desktop.exe，acceptance_channel.py 驱动）使用，该字段排除错连桌面通道。

隔离库：每 run 建 `<data-dir>/reader001-review-<ts>-<tag>/`；种子**只经真实** `POST /api/library/import`（api.at:91-94，`{path, author?, force?}`）在断言前建立前提；启动就绪即 `GET /api/library/books`（back 端口）断言集合 == 预期（空或种子集）。Reader 检出只读：驱动前后 `git -C app-dir status --porcelain` 快照对照，tracked 文件出现修改 = FAIL；框架产生的 untracked 缓存允许但逐条记录。

## 3. 动作合同

- **发现**：`autoui_vtree`（段落 = 真实书模式每段一个 `col`，带 `TapParagraph` 事件与 `¶<idx>` 序号 text，src/front/pages/reading.at:191-201）；`autoui_find`（kind/label=正文子串）/`autoui_exists` 辅助。段落 `idx` = **原始行号**（reading.at:51-57 过滤空行不回退；后端同语义 library.at:623、锚点逐字核对 :850-883）——V-02 的错位负例即源于此。
- **派发**：`autoui_action {element_id:"aura_N|vnode_N", action:"press"}` 走真实 handler 管道（execute_action_vnode → on_with_input_for，mcp_server.rs:2149-2172）→ `TapParagraph(idx)` → `Http.post /api/library/progress`（reading.at:566-591）。**动作发现与派发不要求目标几何可见**（handler 派发与命中测试无关）——保存"pane 外中段"位置无需先滚动。
- **证据链**（保存成功）：ActionResult state_changes + `autoui_state`（restored_note/cur_paragraph）+ UI 提示 `已记录位置：第 N 段`（reading.at:587）+ 后端 `<data>/reading/<book_id>.json` 落盘（library.at:630-639，write_with_backup 协议 :258-292）+ `GET /api/library/progress` 读回一致。失败路径有明示（"后端未确认" reading.at:589）。
- **禁止**：`autoui_fixture`（AUTOUI_TEST_FIXTURES 门控，mcp_server.rs:2704-2792）不用于建立/证明任何保存/恢复终态；驱动不设该 env（工具不可用即负向保证）。同理不设 `AUTOUI_ACCEPTANCE`、`AUTOUI_DRIVE_PROBE`。
- for 循环 vnode id 逐次唯一、跨重建可变 → 驱动在同一次发现窗口内完成发现→派发→断言；**跨重启不缓存 element_id**，重启后重新发现（按内容匹配）。

## 4. 帧 identity（freshness）合同

现状：全部 MCP 响应**无 frame counter/generation/timestamp**；仅有 500ms 快照陈旧上界（renderer.rs:25872-25874）、合成动作 1s applied 栅栏（mcp_server.rs:391-419）、`autoui_wait` 轮询。读者无法原子绑定"观测"与"代际"。

**T-11 增量合同**：u64 布局代数计数器，在每次 `__bounds_collected` 消费完成时 +1（renderer.rs:20613-20723 的发布点），与该次发布的 layout_bounds/styled_vtree/scroll_state 原子同帧；`autoui_vtree`/`autoui_snapshot`/§5 新工具的响应携带 `frame` 字段。

**驱动协议**（§5.A.3 的落地）：action 前记录 frame_before；ack 后必须观察到 frame > frame_before 才读取几何判可见；相等 = 帧未更新 = 不可判 visible（记录为证据缺失，不是 pass）。重启后计数器随进程归零——新实例身份以 §2 三件套为准，不比较跨重启 frame。

## 5. 几何/裁剪合同与最小观测增量

现状三方缺口（本人源码核实）：

1. LayoutCollector 只记节点 border-box 四元组（layout_collector.rs:54-97），**scrollable 钩子显式弃用 content_bounds 与 translation**（:80-89）——无 clip 链、无 offset。
2. ScrollStateReader 六测量（offset/viewport_w/h/content_w/h）只喂 controller 注册表，**不进 MCP 面**；键 = iced Id 原始串（controller 名，scroll_state_reader.rs:31-40）；**丢弃 viewport 原点 x/y**（:82-86 只记 w/h）。
3. controller 绑定 pane 的 iced Id（如 Reader 的 `reading-pane`）不匹配 `vnode_/aura_` 前缀 → **LayoutCollector 不记录其 viewport rect**——两套键空间互不相交，"目标 rect ∩ pane viewport"目前既缺数据也缺计算面。

**T-11 冻结增量**（全部只读，与 autoui_vtree 同类，不加新 env 门）：

1. ScrollStateReader 扩展：记**完整 viewport rect（含 x,y）**+ offset + content extent；键解析复用 layout_collector.rs:39-52 的双前缀逻辑，同时接受 controller 名与 `aura_/vnode_` 键；结果发布进 SharedState 新字段，与 frame 同帧。
2. 新工具 `autoui_scroll_state`：入参 `{pane?: element_id|controller}`；响应 `{backend:"vm", frame, panes:[{key, controller?, viewport_rect:[x,y,w,h], offset:[ox,oy], content:[cw,ch]}]}`，窗口逻辑 px。
3. 新工具 `autoui_visibility`：入参 `{element_id, pane?}`；服务端**同帧**计算：目标 border-box 自内向外对每层 enclosing scroll-pane 的 translation **恰好转换一次**并回加 viewport 原点，与各层 viewport rect 及窗口 rect 求交。响应 `{backend:"vm", frame, target_rect, viewports:[...], intersection:[x,y,w,h]|null, visible:bool, flags:[...]}`。
   - `visible := intersection 宽高均 > 0 且 flags 为空`；flags ∈ `unmeasured`（layout_bounds 无该 id）/`zero_size`/`outside_window`/`no_pane`（目标不在任何 scroll-pane 内且与窗口无交集）。
   - 坐标约定：iced 0.14 translation 与 absolute_offset 同号、正向滚动为正（scroll_state_reader.rs:13-19 已注记）；具体布局空间公式由 T-11 实现时以已知 offset 的 fixture 探针对拍截图钉死，V-01/V-10/V-11 正负例是最终验收。
   - 裁剪边界：本阶段 oracle = 全部嵌套 enclosing scroll-pane viewport ∩ 窗口 rect；非 scrollable 的显式 clip widget 不在范围（Reader 树无此形态，边界明示）。
4. 转换与求交只在服务端做一处（结构上强制"转换一次"），响应同时返回原始分量供驱动复核。

不在 T-11 范围：不新增 action 类型、不动 fixture、不支持 managed content/虚拟列表、不做通用自动化平台重建。

## 6. 门控合同（探查冲突 ② 的裁决）

- **MCP 默认开启、无 flag 门控**（renderer.rs:19097-19101 无条件 `start_mcp_server`；`mcp_port()` = env `AUTOUI_MCP_PORT` 缺省 9247，mcp_server.rs:4953-4958）——"必须显式设 env 才起 MCP"的说法不成立，以源码为准。
- 既有 env 门保持不变：`AUTOUI_MCP_DISABLE`（关 MCP，renderer.rs:25420/25465）、`AUTOUI_TEST_FIXTURES`（fixture 工具）、`AUTOUI_ACCEPTANCE`（autoui_desktop）、`AUTOUI_DRIVE_PROBE`（test-only 直推探针）。
- 新增两个只读观测工具不设新门：仅在被查询时应答，正常产品默认行为零变化（AC-10 的"观测门控不回归"以此界定）；证据链不依赖任何 gated 能力。

## 7. 冷启动与收尾合同

- 收割：terminate 直子 → wait（有界）→ kill 兜底；**reap 验证 = 预选 MCP/back 端口均不再监听的有界轮询**；仍监听 → 控制失败 + 残留记录（pid/端口/归属），**绝不按进程名或端口杀非本 run 子进程**（无 taskkill /IM、无 kill_process_on_port 调用）。
- 冷启动（V-09）：停止 + reap + 端口释放验证 → 同 `AUTO_READER_DATA` 重新 Popen（MCP 端口重新预选）→ 断言新 pid ≠ 旧 pid、§2 三件套重新成立、同书内容可见；恢复验证只观察/等待（§5.A.4），驱动不做任何滚动。
- 全部等待有界（ready/press/state 轮询带 deadline）；超时 = case FAIL + 正常收尾。
- 真实窗口证据：截图成功本身即证据（headless 臂快速失败 mcp_server.rs:2499-2503、最小化窗拒截 :2491-2496）；manifest 记录 env 白名单证明无 headless 变量。

## 8. 驱动合同（T-10 实施冻结）

`scripts/tools/test_native_reader_acceptance.py`，CLI：

```
--auto-bin <path>    必填；781 树构建产物（cargo build -p auto → target/debug/auto.exe）
--app-dir <path>     必填；Reader 检出绝对路径（@29b5dec，只读）
--data-dir <path>    必填；隔离库父目录（驱动建 reader001-review-<ts>-<tag>/ 子目录）
--out-dir <path>     必填；证据包根
--back-port / --mcp-port   缺省各自预选空闲端口
--ready-timeout      缺省 120s
--case <id>          可重复；缺省 ALL（T-10 先交付 V-07/V-12 骨架，V-08~V-13 由 T-12/T-13 填充）
--keep               保留现场（debug）；缺省仍按 §7 收尾
```

就绪序（无固定 sleep，全部有界轮询 + 每轮进程存活早检）：Popen（stdout/stderr 各起 drain 线程落盘 proc/）→ stderr 解析 `listening on`（端口断言）→ `/health` 200 → `GET /api/library/books` 200 且集合==预期 → `autoui_snapshot` 含 `tree:` + Reader 内容标记 → （T-11 后）响应含 frame 字段。

病例注册表：`{id, required:bool, status: pass|fail|skip|evidence_missing, detail}`；退出码 **0** 仅当全部 required=pass 且无 skip/evidence_missing；**1** 用法/环境错；**2** required FAIL；**3** SKIP/缺证据。汇总读取注册表真实状态，不以 banner 证明覆盖。

证据包布局：`manifest.json`（auto-bin path+sha256+size+mtime、`auto --version` 输出或 n/a、框架 commit、Reader commit、数据目录、MCP endpoint、back port、pid、完整 argv、env 白名单值）；`raw/`（每个 MCP JSON-RPC 响应 ndjson）；`trace/`（动作时间线 ts/frame_before/tool/args/frame_after/result）；`shots/`（截图 + sha256；截图由服务端写 CWD 相对 `tests/screenshots/` 或 `tmp/`，name 带唯一前缀，事后搬运，acceptance_channel.py shot() 先例）；`lib/`（前后 library.json、reading/*.json 副本 + sha256）；`proc/`（stdout/stderr 日志）。

## 9. 复用边界

| 复用 | 来源 | 注记 |
|---|---|---|
| `AutoUiMcpClient`（call/press/state/wait_state/screenshot/select_rect…） | .agents/skills/autoui-verifier/scripts/test_vm_mcp.py:21-114 | `sys.path.insert` 导入（test_bp_gallery_click.py:41-42 先例） |
| `pick_free_port()` | 同上 :16-19 | TOCTOU 由 §2 stderr 断言封死 |
| 内容级就绪 + 进程存活早检 | test_vm_mcp.py:167-184 / acceptance_channel.py:98-108 | 合成 §8 就绪序 |
| 截图重试 + 尺寸校验 + 搬运 | test_widgets_gallery_vm.py:297-315 / acceptance_channel.py:139-152 | |
| JSON summary + 0/1 退出骨架 | test_widgets_gallery_vm.py:479-495 | 扩展为 §8 三态注册表 |
| `_png_block_mean`（纯 Python PNG 断言） | acceptance_channel.py:320-403 | 需要像素级复核时 |
| 有界 press 重试 + 状态轮询 | test_bp_gallery_click.py:149-171 | |

**假设修正**：计划 T-10 写"复用 test_vm_mcp.py client 与 owned 进程设施"——目录内**不存在**名为 owned-process 的设施，只有 4 处散布的 terminate→wait→kill 惯用法（timeout 3/5/10/5s 各异）。owned 语义以本文 §7 合同为准（单进程形态 + 端口释放验证），不引用不存在的设施。dead 参数（test_vm_mcp.py `--save-dir` 声明未使用）不继承。

**新建**（T-10）：双端口身份编排、隔离库管理、冷启动原语、病例注册表、证据包、几何 oracle 客户端（T-11 工具落地后）。

## 10. 缺口与风险清单

| # | 缺口/风险 | 处置 |
|---|---|---|
| R1 | 原生窗口 split 形态（`-r vm --no-merge`）无运行先例 | T-10 smoke 首验；框架缺陷最小修复或具名阻塞 |
| R2 | iced 布局空间/translation 公式未钉死 | T-11 探针 + V-01/V-10/V-11 验收 |
| R3 | MCP 端口漂移 + 预选 TOCTOU | §2 stderr 断言 + pid↔端口核对 |
| R4 | vnode id 跨重建不稳定 | §3 同窗口发现→派发→断言；跨重启重新发现 |
| R5 | `auto --version` 是否支持未验证 | manifest 记 n/a 时以 sha256 为身份 |
| R6 | 框架在 Reader 检出产生 untracked 缓存 | §2 前后 git 快照对照 + 逐条记录 |

## 11. 对 T-10/T-11 的交接决定

1. T-10 按本文 §1/§2/§7/§8 实施（驱动 + V-07/V-12 骨架）；T-11 按本文 §4/§5 实施最小观测增量（frame 计数器、scroll_state 扩展发布、两新只读工具）；二者可并行起步，T-13 全量验收要求两者都已落地。
2. T-11 动手前先跑 R1/R2 两个探针（smoke 形态 + 已知 offset fixture），探针结果回填本文 §5.3 坐标公式（以修订记录形式追加，不改合同结构）。
3. 任何与本合同冲突的实现发现 → 停止接线，按 work 技能 adapt-or-route 表记录处置；不接受静默降标（如以字符占比/中心包含替代正面积交集）。

## 12. 修订记录

**2026-10-14 r1（T-10 实施期勘定，worktree commit 7b0427e37）**：

1. **R1 结案（框架缺陷+修复）**：legacy VM 轨实际注册了 `__auto` 控制面，`/__auto/health/ready` 在 run 模式恒 503 `{"state":"starting"}`（ready 态翻转只在 service 面发生）——PLAN-736 注释"legacy 无 /__auto 路由→404"的假设不成立；`probe_service_ready` 返回 `Some(false)` 后 `continue` 跳过 TCP 兜底，`-r vm --no-merge` 形态 120s 必死。修复：`crates/auto-man/src/rust_ui.rs::start_vm_server` 的 Some(false) 不再短路 TCP 兜底（TCP 恢复为最终裁决）。vue+vm split（vue.rs 同函数）同病同治。修复后实测 1.1–1.6s 就绪。
2. **就绪探针修正**：§8 的 `/health` 200 不可用（legacy VM 轨无 /health app 路由，404）；改为 `/__auto/health/live`（实测恒 200，live=transport 应答）。
3. **导入请求形状**：`POST /api/library/import` 的 `author`/`force` 为必填（缺 author 实测 400 `missing param`）；响应体双重 JSON 编码（body 为 JSON 字符串），驱动已 unwrap。
4. **种子时机**：书架不自动刷新（前端 Init 仅启动时 fetch 一次）——种子导入必须插在 backend-live 之后、MCP 窗口就绪之前（wait_backend 与 wait_mcp 两段之间），marker 才进首帧数据。
5. **listening 行**：`AutoUI MCP: listening on` 确为 eprintln→stderr，但实测窗口形态下该行不一定及时出现在进程 stderr（ui_console 侧另有 [boot] 行）；身份断言以 §2.1 pid↔端口 netstat 核对为主，stderr 仅硬性检查 `fell back`/FATAL 两个违规标记。
6. **v12_timeout 确定性**：热缓存下 backend 可 <2s live，超时案例改用 0 预算（唯一保证超时的取值）。
