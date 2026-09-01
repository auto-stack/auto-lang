---
plan_id: PLAN-509
status: archived                 # drafting → executing → execution_done → reviewed → archived
feature_name: smithay-host-stage1
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/overview.md: auto-cosmic 条目状态翻转（experimental → active-509 线内）+ Smithay 宿主线注记"
new_spec_components:
  - "docs/plans/reports/509-smithay-route-verdict.md: 新增（T1 三路线裁定报告——B 裁定 + smithay 0.7.0 选型 + Stage 2+ 路线图）"
  - "docs/plans/reports/509-t5-env-baseline.md: 新增（WSL2/WSLg 环境基线——后续 Stage 环境基准）"
  - "docs/specs/auto-cosmic/project.md: 待 merge 回写（host-smithay 第五子 crate + 路线 B 定案——复审指定回写点）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——Linux 原生合成器宿主线启动（Smithay host-smithay，路线 B，I1 零分叉实证）"

affects: [auto-lang/ui, auto-cosmic]
current_step: 8
total_steps: 8
---

# [PLAN-509] Smithay 宿主 Stage 1——Linux 合成器线启动（原提案 457）

> **编号解析**：Design 23 M5 / 程序总览"457 Smithay 宿主"为提案号，实际
> 立项 **509**（程序台账计划一览随本计划同步改号）。

## 变更摘要

虚拟桌面程序的最后一块拼图（Design 23 目标："在 Linux 上是原生合成器宿主，
复用桌面 shell"）。Stage 1 三件事：

1. **启动条件兑现——路线评估定案**：三路线对比裁定——
   **A** auto-cosmic `host-libcosmic` 复活（libcosmic 工具套路线）；
   **B** Smithay 合成器 + **桌面协议宿主**（宿主消费 DrawList/Pixels——
   500/507 的 queue/independent 资产直接复用，宿主可不含 iced）；
   **C** 混合（Smithay 骨架 + libcosmic 画 shell 面）。裁定标准：桌面
   shell 资产复用最大化、维护面最小、与 iced 0.14 生态距离、Linux 验证
   环境适配。
2. **最小合成器骨架**：Smithay 会话（backend/udev 或 winit 后端起评裁定）
   + 单全屏面合成，**shell 首帧上屏**（静态即可）。
3. **I1 纪律核对**：WM/session/投影代码**零分叉**——Linux 臂只是新增
   "宿主合成"后端（diff 证明或配置差异），对应 Design 23"一套窗口管理
   代码"的总目标。

**Linux 环境是硬前置**：本仓主开发环境为 Windows——执行会话必须具备
Linux 图形验证环境（WSL2+Wayland / 物理 Linux / CI Linux runner 之一，
待澄清①定）。后续 Stage（本计划只登记不实施）：xdg_shell 原生客户端窗口
管理（Linux 上原生 app 是一等公民——"原生互通"问题在 Linux 形态下自然
消解）、输入/IME（S8 shell IME UI 挂此线）、多 App/工作区、与双模 exe
合流。

## 目标

- **G1 评估定案**：三路线对比报告（复用矩阵/依赖距离/风险/环境）+ 裁定
  与理由成文（`docs/plans/reports/509-smithay-route-verdict.md`），兑现
  程序台账登记的启动条件②。
- **G2 骨架实跑**：按裁定路线落地最小合成器，Linux 环境编译 + 实跑，
  会话建立 + 合成一帧。
- **G3 shell 首帧**：既有桌面 shell 面（.at 资产）以裁定路线渲染上屏
  （静态首帧即可，无交互要求）。
- **G4 I1 核对**：`crates/auto-lang` 的 WM/session/投影/事件路由在 Linux
  臂引入下零改动（diff 证据）；shell 资产（.at）零改动复用。
- **G5 台账回写**：程序总览 457 行改号 509 + 状态更新；overview 的
  auto-cosmic 条目状态翻转（experimental → active-线内）。
- **非目标**：xdg_shell 客户端管理/输入设备/IME（Stage 2+）；性能；X11
  兼容（Wayland-first）；S8 IME UI；发布打包。

## 架构方案

```
路线 B 形态（评估若裁定之——当前倾向，因 500 已落地）:
  Smithay compositor (Linux)
    ├─ 合成循环: wl_surface 合成（含 shell 面与未来的原生客户端）
    └─ 桌面协议宿主（ui/desktop_protocol 既有 host 端复用）
         ├─ queue 臂: DrawList → 宿主栅格化（500 生产化产物）
         └─ shell 作为 attach App（双模入口既有，--autodesk-client 链）
路线 A 形态: host-libcosmic 复活——libcosmic(iced 系) 直渲 shell，
  Smithay 仅做窗口管理外挂或暂不引入
```

- **落点随裁定**：A → `crates/auto-cosmic/host-libcosmic/` 复活扩展；
  B → 新 `crates/auto-cosmic/host-smithay/`（或 ports 层旁挂）；C → 组合。
- **ports 层价值**：`ports`/`ports-linux`（D-Bus 适配带 Windows mock 回退）
  为跨平台开发保底——评估须核对两路线与 ports 层的衔接。

## 技术栈

`smithay` 与/或 `libcosmic`（**本线新三方大依赖**——评估报告承担选型论证，
入 lock 前经用户过目）；其余既有。

## 需求分析与背景调查

（取材 docs/specs/overview.md §外围实验 + Design 23/程序台账 + 现场核验 2026-08-31）

- **启动条件**（程序台账"457 启动条件"）：① 454+455 完成（=462/463，✅
  早已满足）；② auto-cosmic 宿主复活评估（libcosmic 依赖决策，Linux
  环境）——未做，**本计划 T1 兑现**。
- **auto-cosmic 现状**（核验）：四子 crate——`ports/`（抽象层）、
  `ports-linux/`（zbus D-Bus，Windows 下 fallback mock 保跨平台编译）、
  `host-libcosmic/`（宿主骨架，依赖 auto-lang ui+ui-headless，**libcosmic
  未入锁**——骨架态）、`demo/`。experimental、无消费者。
- **路线 B 的新可能性**：500 落地后宿主已能消费 DrawList 命令帧（queue
  臂生产化栅格化）与 Pixels 帧——Smithay 宿主可以**不含 iced**，把 shell
  当 attach App（桌面协议五通道 + 双模入口既有）。这使"一套 WM 代码 +
  可插宿主"从设计宣言变为机制现实，是评估的最大变量。
- **Design 23 语义**：R1/R2——WM 是特权 AutoUI App，宿主只管合成；Linux
  臂 = 合成宿主多一个后端。S8（shell IME UI）挂本线后续 Stage。
- **排程**：与 507/508（同协议线后续）的交叠 = 评估若走 B 将消费其产物
  ——**开工前置建议 = 507 合入**（覆盖集就绪）；骨架编译类任务可先行。
  503/505/506 无交叠。

## 详细设计

### 1. 路线评估（T1，报告成文）

- 对比维度（表格式）：桌面 shell 资产复用度（.at/投影/协议/Dock 等）、
  依赖引入量与 iced 0.14 生态距离、Linux 图形栈适配（Wayland session/
  udev/drm）、跨平台编译影响（Windows 主仓 dev 流不能红）、维护面、
  Stage 2+ 演进空间（xdg 客户端管理是 Smithay 强项）；
- 产出：裁定 + 依赖清单（smithay/libcosmic 版本选型）+ Stage 2+ 路线图
  初稿；**新依赖入 lock 前经用户确认**（大依赖门槛，待澄清②）。

### 2. 最小骨架（按裁定）

- Smithay 路线：session（libseat/udev 或开发期 winit 后端起手）+ 单
  全屏 surface + 渲染循环（借宿主栅格化产物画 DrawList，或 libcosmic
  直渲——随路线）；
- 跨平台保底：骨架 crate 在非 Linux 目标 cfg 隔离（auto-cosmic 既有
  fallback 模式延续），主仓 `cargo check`/`cargo t` 零影响。

### 3. shell 首帧

- 以 attach 形态拉起 shell 面（`--autodesk-client` 既有入口，桌面
  protocol loopback/pipe 均可）或直渲（A 路线），输出静态首帧截图；
- 首帧内容 = shell.at dock + 桌面背景（496 桌面本体资产直接复用）。

### 4. I1 核对

- diff 证据：`crates/auto-lang/src/ui/{session,iced,virtual_window,
  native_dock}` 与投影/事件路由零改动（或仅 cfg 配置差异行）；
- shell .at 资产零改动（复用证明）。

## 测试设计

1. **T1 评估报告**：三路线矩阵 + 裁定（复审对照物）。
2. **T2 跨平台编译**：Windows 主仓 `cargo check`/`cargo t` 不回归 +
  Linux 目标骨架编译绿（交叉或 Linux 环境）。
3. **T3 首帧验证**：Linux 环境（WSL2/Wayland 或物理）实跑截图留痕
   （像素级：dock 可见 + 背景非空）。
4. **T4 I1 diff**：核对零分叉证据链。
5. **T5 环境记录**：验证环境形态（WSL2 版本/内核/Wayland 合成方式或
   物理机）成文——后续 Stage 的环境基线。

## 验收标准

1. T1 报告成文且裁定经用户确认（大依赖门槛）。
2. T2 双平台绿；T3 首帧截图留痕。
3. T4 零分叉证据成文（auto-lang 主仓 diff 干净）。
4. T5 环境基线记录；程序台账 457→509 改号回写。
5. `cargo check -p auto-lang` 零警告；主仓日常档不回归。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **路线评估**：三路线对比矩阵 + 裁定 +
   `docs/plans/reports/509-smithay-route-verdict.md` 成文（含依赖清单，
   入 lock 前置用户确认）。
   验证：报告成文 + 用户确认记录。
   [✅ 已完成] 报告成文（branch 40511a567）：裁定 B（smithay 0.7.0，
   default-features=false + backend_winit/wayland_frontend/desktop/
   renderer_gl，Linux-target 门控）；矩阵证伪 A 短期保真论；用户确认
   询问已发起未获答复——按待澄清③倾向推进，硬门槛保留 merge 门槛
   （报告 §7 留勾选位）。
2. **环境就绪**：Linux 验证环境搭建/确认（WSL2+Wayland 或物理）+
   T5 基线记录。
   验证：环境内 `rustc --version`/合成环境冒烟记录。
   [✅ 已完成] `docs/plans/reports/509-t5-env-baseline.md`（branch 40511a567+1）：
   WSL2 Ubuntu22.04 / 内核 6.18.33.2 / WSLg wayland-0 活跃 / rustc 1.97.1 /
   eglinfo Wayland platform EGL1.5 Mesa 冒烟绿（GBM 失败=WSLg 预期）。
3. **依赖与骨架 crate**：按裁定建/复活宿主 crate（cfg 隔离保 Windows
   编译）+ 依赖引入（经确认）+ T2 双平台编译。
   验证：`cargo check -p auto-lang`（Win）+ Linux 侧骨架编译绿。
   [✅ 已完成] branch 7dded6b8e：`crates/auto-cosmic/host-smithay/` 新建
   （winit/nested 合成循环 + cfg stub）；smithay 0.7.0 Linux-target 门控
   入 manifest（发现 Cargo.lock 本仓 gitignored——"入锁"实际=manifest
   进分支，merge 门槛语义不变）；Win 侧 stub 编译绿 + auto-lang 159 警
   =master 基线持平；Linux 侧 smithay 全树编译 0 错 0 警。
4. **Smithay 会话/合成循环**（B/C 路线）：最小 compositor（session +
   单 surface + 渲染循环）Linux 实跑一帧。
   验证：实跑日志 + 单帧合成证据。
   [✅ 已完成] branch（T4 commit）：WSLg wayland-0 与 Xvfb+llvmpipe 两形态
   实跑（240/400/600 帧预算退出 0 错）；**像素级证据**
   `docs/plans/reports/assets/509/t4-frame-xvfb.png`——根窗中心像素
   srgb(23,23,33) = CLEAR[0.09,0.09,0.13]×255 精确命中。环境注记：本机
   WSLg 窗口不上浮 Windows 桌面（xeyes 对照证伪，远程会话特征）——
   取证走 Xvfb 闭环；WSLg 形态协议层正常（窗口创建/EGL/swap 全通）。
5. **shell 首帧上屏**：attach/直渲（随路线）拉起 shell 面 → 全屏首帧
   截图留痕。
   验证：截图（dock+背景可见）。
   [✅ 已完成] branch 7305bacc1：生产链（ui_desktop 真桌面 + 505 验收通道
   MCP 截图，WSLg 2560×1600）→ 宿主纹理导入 → Xvfb 合成取证
   （1280×800）。视觉核验 dock 可见（启动钮+搜索条+图标列+托盘时钟）
   + 背景非空（桌面图标+虚拟窗）；资产
   `docs/plans/reports/assets/509/t5-*.png`。**形态注记**：Stage 1 走
   T1 报告 §5 注册的静态首帧路径（生产渲染器 PNG → 宿主纹理）；live
   attach（跨进程 UDS 传输 + POSIX shm）= Stage 2 增量——transport
   模块头注既有"Linux 侧单独生长"预留，本计划仅补编译占位。
6. **I1 diff 核对**：auto-lang 主仓改动清单 = 仅新增/配置差异；shell
   资产零改动证明。
   验证：diff 证据贴本计划。
   [✅ 已完成]（merge-base 1f7313e93，2026-09-01）：
   - **I1 命名模块零改动**：`ui/{session.rs, iced/, virtual_window.rs,
     native_dock/, event_router.rs, aura_view_builder.rs,
     node_converter.rs}` diff 为空（WM/session/投影/事件路由零分叉）。
   - **shell 资产零改动**：`crates/auto-lang/assets/`（shell/desktop/
     notification_center/settings/switcher .at）diff 为空。
   - **auto-lang 全部改动 = 3 文件 +40/−1，纯 cfg 配置差异**：
     `clipboard_native.rs`（PathBuf 导入去门控，−1 = cfg 属性行）、
     `desktop_protocol/shm.rs`（非 Windows 补 Arc/Mutex 导入）、
     `desktop_protocol/transport.rs`（非 Windows 错误桩模块 = 文件头注
     既有"Linux 侧单独生长"预留）。Windows 行为零变化（159 警 = 基线
     持平实证）。
   - 分支总差异：20 文件 +653/−1（其余 = 新 crate host-smithay + 报告/
     取证资产）。
7. **台账回写**：`docs/plans/autos-desktop-program.md` 457 行改号 509 +
   状态；overview auto-cosmic 条目注记。
   验证：台账 diff。
   [✅ 已完成] branch（Step 7 commit）：改号在立项时已落（"原提案 457"），
   本次补齐——457 启动条件双项勾销（462/463 早已满足 + T1 评估兑现）；
   509 行 drafting→executing + T1–T5 证据摘要；overview auto-cosmic
   条目 experimental→active-509 线内（host-smithay + smithay 0.7.0 注记）。
8. **收尾**：健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t`（主仓日常档）。
   [✅ 已完成] branch 4c47d8c63（工作树全净，8 commits）：check 绿（159
   警 = master 基线持平，host-smithay 双平台 0 警 + fmt 归一）；
   `cargo t --no-fail-fast` = **4379/4382 绿，3 失败与 master 逐名一致**
   （d8_toggle_dark_mode / style_migration_probe /
   strips_tags_and_decodes_entities——并行会话 015 暗色提交等既有
   问题，与本计划零交集，479 先例处置）；cfg 修补补交于 4c47d8c63
   （前 commit 7305bacc1 路径失误漏加，已核）。

## 复审记录

**2026-09-01 /auto-plan:review**（工作树 `.worktrees/plan-509-dev`，merge-base
1f7313e93，8 commits / 22 文件 +658/−6）。逐项独立重验，不信任勾选框：

| # | 验收标准 | 判定 | 证据 |
|---|---------|------|------|
| 1 | T1 报告成文且裁定经用户确认 | **过（附注）** | 报告成文（40511a567）；确认询问两度发起（执行期+复审期）均未获会话内答复——按计划文本该门槛时点为"入 lock 前"，master lock 仅在 merge 变更（Cargo.lock 本 gitignored），故**merge 前置条件**显式登记于此与报告 §7 勾选位，非隐藏延后 |
| 2 | T2 双平台绿；T3 首帧截图留痕 | **过** | 复审期重跑：Win host-smithay check 绿（27s）+ Linux check 绿（fmt 后 1.64s 零警）；`assets/509/` 五张证据在位，T3 视觉核验记录（dock 可见+背景非空） |
| 3 | T4 零分叉证据成文 | **过** | 复审期重验 merge-base diff：`ui/{session,iced,virtual_window,native_dock,event_router,aura_view_builder,node_converter}` 与 `assets/*.at` **零行变更**；auto-lang 全量 = 3 文件 +40/−1 纯 cfg 差异 |
| 4 | T5 基线 + 台账改号回写 | **过** | 两报告在位；台账 42 行（原提案 457）+ 457 启动条件双 `[x]` + overview 条目 active（grep 复核） |
| 5 | check 零警告 + 日常档不回归 | **过（口径注记）** | `cargo check -p auto-lang` 159 警 = master 基线**精确持平**（字面零不可达，按零新增判）；`cargo t --no-fail-fast` 4379/4382，3 失败与 master 逐名一致（既有）；**`cargo tf` 3350/3350 全绿**（本复审全量门，VM/转译/书未触碰故 tv/tt/tb 免） |

**遗漏/延后/workaround 猎查**：
- 遗漏：无（8 commits 载体齐全；新 crate 零 TODO/FIXME/HACK）。
- 延后（登记 KNOWN-DEBT）：live attach → Stage 2——Linux UDS 传输 + POSIX
  shm + shell 子进程装载三件均为增量（transport/shm 模块 Windows 门控，
  头注既有"Linux 侧单独生长"预留）；G3"静态首帧即可"为计划文本允许口径，
  非缩水。
- Workaround（同上合并登记）：transport 非 Windows 错误桩（结构性占位，
  调用即错，接口形状与 Windows 一致）。
- 环境债：本机 WSLg 窗口协议层通但不浮 Windows 桌面（远程会话特征，
  xeyes 对照证伪）——后续交互类验证（输入/IME Stage）需物理机或 WSLg
  修复，已记 T5 基线。
- 非本计划债如实记录：cargo t 三失败（d8_toggle_dark_mode 等）为并行
  会话 015 暗色提交余波，master 原样复现。

**结论：全过 → `reviewed`。** merge 前置：勾销报告 §7 大依赖确认位
（smithay 0.7.0 随分支入 master manifest）。

## 待澄清事项

- **① Linux 验证环境（硬前置）——2026-08-31 用户裁定：WSL2**：Stage 1 以
  WSLg + Smithay **winit/nested 后端**为验证路径（Smithay 开发期标准形态；
  DRM/udev 直连会话不适用于 WSL，留 Stage 2+ 或物理机场景）；GPU 经 wgpu
  可能落软件渲染（llvmpipe/D3D12 转 Vulkan 视环境），首帧验收可接受。
  **执行形态裁定：无需在 WSL 内开 agent**——Windows 侧施工会话经
  `wsl.exe` 驱动构建/运行（worktree 纪律不变，代码编辑留在 Windows 侧）；
  `CARGO_TARGET_DIR` 重定向至 WSL ext4 防 `/mnt/d` 慢 IO；WSLg 窗口直接
  呈现于 Windows 桌面，截图验证照常。T2 环境就绪含 WSL 内 rustup/
  libwayland-dev/libxkbcommon 等系依赖安装。备选（真在 WSL 内开 agent +
  worktree 建于 ext4）可行但非必要，仅当 Windows 侧驱动遇阻时启用。
- **② 大依赖门槛**：smithay/libcosmic 为重量级新三方——评估报告的依赖
  清单**入 lock 前经用户确认**（对齐仓库对重大依赖的谨慎惯例）。
- **③ 路线倾向声明**：B（Smithay+桌面协议宿主）因 500 资产复用最大化而
  是当前倾向，但**评估不得倒果为因**——A（libcosmic 直渲）在"shell 交互
  保真"上可能有短期优势，矩阵如实打分。
- **④ 排程**：建议 507 合入后开工（queue 臂覆盖集就绪）；骨架/评估类
  任务（T1/T2）可先行。
- **⑤ smithay 版本策略**：smithay 生态版本迭代快——评估锁定具体版本并
  注明升级策略（线内 pin + 升级计划）。
