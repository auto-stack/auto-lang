# Plan 509 T1 —— Smithay 宿主路线评估与裁定

- 计划：`docs/plans/509-smithay-host-stage1.md`（G1）
- 日期：2026-09-01
- 证据基线：现场核验（crates/auto-cosmic + crates/auto-lang/src/ui）+ 外部调研
  （crates.io / docs.rs / pop-os 仓库，2026-09-01 检索）
- 待澄清对照：③ 路线倾向声明（B 倾向但矩阵如实打分）——本文 §2 矩阵未预设结论，
  A 的"shell 交互保真短期优势"论断经资产形态核验后**被证伪**（见 §3-R1）。

## 0. 现状盘点（现场核验）

### 0.1 本仓资产（三路线共有的复用基数）

| 资产 | 位置 | 状态 |
|------|------|------|
| 桌面协议五通道（host/broker/client/shm/transport） | `crates/auto-lang/src/ui/desktop_protocol/` | 生产化（500/507） |
| 三态渲染裁决（queue / independent 像素臂） | `desktop_protocol/coverage.rs`（`RenderMode::resolve`，`--autodesk-render=` 三态参数） | 507 覆盖集爬坡完成 |
| 双模 attach 入口 | `desktop_protocol/dual_mode.rs`（`--autodesk-client={pipe}`，v1.3 双模并存） | 既有 |
| independent 像素臂生产链 | 生产二进制 `auto run` + iced 隐藏窗截图降采样（`stage3.rs` T3 实测口径） | 既有——**iced 在子进程侧** |
| shell 面资产 | `crates/auto-lang/assets/{shell,desktop,notification_center,settings,switcher}.at` | 496 桌面本体，零改动候选 |
| ports 系统抽象层 | `crates/auto-cosmic/ports`（10 端口 trait + mock）/ `ports-linux`（zbus，Windows fallback） | 跨平台保底既有 |
| host-libcosmic 骨架 | `crates/auto-cosmic/host-libcosmic/`（libcosmic **注释态**未入锁；`fallback.rs`→headless） | 骨架，无消费者 |
| 主线 iced | `auto-lang` `iced = "0.14.0"`（ui-iced feature 门控） | 生产 |

### 0.2 外部依赖形态（2026-09-01 检索）

- **smithay 0.7.0**（crates.io，约 2025-09 发布，MSRV 1.80.1，MIT）：
  crates.io 最新稳定锚。默认 feature 含 `backend_winit`（winit 0.30）、
  `wayland_frontend`、`desktop`、`renderer_gl`，也含 DRM/GBM/libseat/udev
  重系统依赖集（可 `default-features = false` 裁剪）。COSMIC（cosmic-comp）
  与 jay 合成器的公共地基。
- **libcosmic**（pop-os）：**git-only，无 crates.io 版本化发布**；基于**内部
  fork 的 iced**（非 0.14，rebase 跟踪 issue #1089 进行中）；**Windows 不
  支持**（issue #505，"Iced runs on Windows but this is a libcosmic-specific
  issue"）。任何引入都是无版本锚的 rev pin。

## 1. 三路线定义

- **A — host-libcosmic 复活**：`auto-cosmic/host-libcosmic` 从骨架转实，
  VTree→libcosmic Element 降阶层直渲 shell；Smithay 不引入或仅后补。
- **B — Smithay 合成器 + 桌面协议宿主**：新 `auto-cosmic/host-smithay`，
  Smithay 管合成循环（winit 嵌套起手），宿主消费协议帧（independent 像素臂
  起手），shell 以 attach App 形态复用双模入口。宿主**不含 iced**。
- **C — 混合**：Smithay 骨架管窗口/合成 + libcosmic 画 shell 面。

## 2. 对比矩阵

打分 1–5（5 最优）。"依据"列可复查。

| 维度 | A host-libcosmic | B smithay+协议宿主 | C 混合 | 依据 |
|------|:---:|:---:|:---:|------|
| 桌面 shell 资产复用度 | 2 | **5** | 2 | A/C 需新写 VTree→libcosmic Element 降阶层（第二套 widget 投影，host-libcosmic 骨架从未动工）；B 的 .at 资产、双模入口、shm/pipe 传输、像素臂生产链全部原样复用，宿主只做纹理搬运 |
| 依赖引入量 / iced 0.14 生态距离 | 1 | **4** | 1 | A/C：libcosmic 的 fork iced 与主线 iced 0.14 **并存两套**（类型不通、编译面翻倍）且 rev pin 无版本锚；B：smithay 0.7.0 crates.io 锚定，宿主无 iced，winit 同为 0.30 代 |
| Linux 图形栈适配（Wayland session/udev/drm） | 1 | **5** | 4 | libcosmic 是 app 工具包**不是合成器**——A 最终仍需 Smithay（退化成 C）；B 即合成器本职：winit 嵌套开发 → udev/DRM 生产路径平滑 |
| 跨平台编译（Windows 主仓 dev 流） | 1 | **5** | 1 | libcosmic Windows 不构建（#505）→ A/C 真路径在主开发环境永远不可测，fallback-only；B 的 smithay 放 `[target.'cfg(target_os="linux")']` 门控，Windows 编译图零新增 |
| 维护面 | 1 | **4** | 1 | A/C 的降阶层 = 与 `ui/iced` 并行的第二投影实现（≈万行级 renderer 的镜像）；B 宿主仅合成循环+纹理上传（百行级），投影零新增 |
| Stage 2+ 演进（xdg 客户端管理等） | 1 | **5** | 3 | xdg_shell 原生客户端窗口管理是 Smithay 强项（desktop space/tiling/layer）；A 无此能力 |
| **I1 纪律风险**（G4，一票项） | **高风险** | **零** | **高风险** | A/C 引入第二套 WM/投影通路的分叉引力，直接顶撞 Design 23 "一套窗口管理代码"；B 结构上保证 WM 仍是特权 AutoUI App（shell attach），宿主只合成——与 R1/R2 同构 |
| **合计（含 I1 一票项）** | 出局 | **裁定路线** | 出局 | |

> 待澄清③回应：原设想的"A 在 shell 交互保真上有短期优势"**不成立**——本 shell
> 不是 libcosmic widget 应用，是 .at 投影产物；交互保真来自既有投影链本身。
> A 反而要为新降阶层重新证明保真。矩阵如实反映：A 无任何单项占优。

## 3. 裁定

### 裁定：路线 B —— Smithay 合成器 + 桌面协议宿主（independent 像素臂起手）

理由（按权重）：

- **R1 资产复用最大化**：500/507 的协议资产（三态裁决/双模入口/shm 传输/
  像素臂生产链）是 B 的直接燃料；shell/desktop .at 零改动。A 的复用仅停在
  .at 文件层，其上整套投影要重写。
- **R2 I1 结构性零分叉**：宿主不渲染 UI，只合成——WM/session/投影代码没有
  产生第二实现的缝隙。这是对 G4 最强的保证形式（结构保证 > 纪律保证）。
- **R3 依赖可锚定**：smithay 0.7.0 crates.io 版本锚（对照 libcosmic git
  rev pin）；宿主无 iced，主线 iced 0.14 无并存冲突。
- **R4 Windows dev 流保绿**：target-cfg 门控，主仓 `cargo check`/`cargo t`
  零影响（T2 验收直接对应）。
- **R5 Stage 2+ 主线正交**：xdg_shell/输入/IME 演进全部沿 Smithay 生态展开，
  无 A/C 的迁移债。

### 落点

新 crate `crates/auto-cosmic/host-smithay/`（根 workspace member，
affects: auto-cosmic）。`host-libcosmic` 维持骨架态不动（不入锁、不删除——
若 libcosmic 完成 iced 0.14 rebase 且发 crates.io 版，可再评估，见 §6）。

## 4. 依赖清单（待澄清②：入 lock 前置用户确认）

### 4.1 新增直接依赖（仅 Linux 目标编译图）

```toml
# crates/auto-cosmic/host-smithay/Cargo.toml
[target.'cfg(target_os = "linux")'.dependencies]
smithay = { version = "0.7.0", default-features = false, features = [
    "backend_winit",    # 嵌套开发后端（winit 0.30，WSLg 验证路径）
    "wayland_frontend", # wl_surface/wl_seat 等协议前端
    "desktop",          # 窗口空间抽象（Stage 2 xdg 管理复用）
    "renderer_gl",      # EGL/GL 纹理上传（RGBA→texture）
] }
```

- 裁剪说明：剔默认集的 DRM/GBM/libseat/udev/X11（WSL 嵌套不需要，省
  `libdrm/libgbm/libseat/libudev` 系统包与构建风险）；`renderer_pixman`
  （纯 CPU）保留为 EGL 阻塞时的备选，不入首锁。
- 传递依赖要点（Linux 侧）：wayland-server/client、wayland-protocols、
  xkbcommon、EGL 动态加载（libloading，运行时链 mesa）、winit 0.30。
  **Windows 侧编译图零变更**（target 门控，lock 全平台共账但不参与构建）。

### 4.2 WSL2 系统包基线（T2 环境就绪清单）

`build-essential pkg-config libwayland-dev wayland-protocols
libxkbcommon-dev libegl1-mesa-dev libgles-dev`（rustup 走官方脚本；
`CARGO_TARGET_DIR` 重定向 WSL ext4——待澄清①执行形态）。

### 4.3 版本策略（待澄清⑤）

- **线内 pin smithay 0.7.0**（crates.io 锚，锁 Cargo.lock）。
- 升级策略：跟进 crates.io 后续 release（0.x 无 semver 兼容承诺，升版 =
  小批 API 迁移，按需立项）；git master 仅当出现阻塞性缺口才考虑临时
  rev pin，且必须登记 KNOWN-DEBT + 退出计划。
- MSRV 1.80.1，低于当前稳定工具链，无压力。

## 5. Stage 1 首帧路径（B 形态细化）

1. **宿主侧**：host-smithay 起 winit 嵌套后端（WSLg 下开一个"显示器"窗），
   Smithay 合成循环就绪。
2. **shell 侧**：生产链 spawn `auto run`（shell.at attach，`--autodesk-client
   ={pipe} --autodesk-render=independent`）——子进程 iced 隐藏窗在 WSLg 下
   栅格化，像素帧经协议到达宿主，宿主上传纹理合成上屏。
   - 最小兜底（若子进程链在 WSL 首跑遇阻）：预渲染 PNG（
     `run_dynamic_iced_pixels` 一次性产物）作静态纹理源——同机制（纹理上屏），
     不改宿主代码路径。
3. **验收**：截图 dock + 桌面背景可见（T3）。

## 6. Stage 2+ 路线图初稿（登记不实施）

| 项 | 内容 | 依托 |
|----|------|------|
| S2a | xdg_shell 原生客户端窗口管理（tiling/floating/layer）——Linux 原生互通自然消解 | smithay desktop space |
| S2b | 输入/seat 直连（libinput）+ IME（挂 S8 shell IME UI） | smithay input backend |
| S2c | queue 臂宿主栅格化（可选：tiny-skia CPU 栅格化器进宿主，免子进程 iced；或长期维持像素臂） | 独立评估 |
| S2d | udev/DRM 直连会话（物理机/libseat）、多 GPU | smithay udev backend |
| S2e | 与双模 exe 合流（宿主形态与 auto 同体性） | 程序台账 |
| 观察 | libcosmic 若发 crates.io 版 + 完成 iced 0.14 rebase → A 路线重估窗口 | §4.3 |

## 7. 用户确认记录（大依赖门槛——待澄清②）

> 裁定 **B** + smithay 0.7.0 入锁（§4.1 清单，Linux-target 门控）。

- [x] **用户确认：路线 B 裁定 + 依赖清单入锁**——2026-09-01 执行会话与
      复审会话两度发起确认询问均未获会话内答复；复审记录将其登记为
      merge 前置（本位）。**同日用户经 `/auto-plan:merge` 发起合入**
      （复审交付摘要已明示该动作 = smithay 0.7.0 随分支落 master
      manifest）——视为大依赖门槛正式勾销，本位落章。
