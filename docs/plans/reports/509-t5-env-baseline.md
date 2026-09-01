# Plan 509 T5 —— Linux 验证环境基线（后续 Stage 环境基准）

- 计划：`docs/plans/509-smithay-host-stage1.md`（执行步骤 2 / 测试设计 T5）
- 记录日期：2026-09-01
- 形态（待澄清①裁定）：**WSL2 + WSLg + Smithay winit/nested 后端**；
  Windows 侧施工会话经 `wsl.exe` 驱动构建/运行，代码编辑留 Windows 侧。

## 环境快照

| 项 | 值 |
|----|----|
| 发行版 | Ubuntu 22.04（WSL2，`wsl -l -v` 确认 VERSION 2，Running） |
| 内核 | `6.18.33.2-microsoft-standard-WSL2` |
| WSLg | 活跃：`WAYLAND_DISPLAY=wayland-0`、`DISPLAY=:0`、`XDG_RUNTIME_DIR=/run/user/1000`、`/mnt/wslg` 在位 |
| rust | `rustc 1.97.1 (8bab26f4f 2026-07-14)` / `cargo 1.97.1`（≫ smithay 0.7.0 MSRV 1.80.1） |
| 构建链 | gcc 11.4.0 / pkg-config 0.29.2 |
| 图形库（pkg-config 口径） | wayland-client 1.20.0 / wayland-server 1.20.0 / wayland-protocols 1.25 / xkbcommon 1.4.0 / egl 1.5 / glesv2 3.2 |
| 补装包 | `libwayland-dev wayland-protocols libxkbcommon-dev libegl-dev libgles-dev`（root apt）+ `mesa-utils`（冒烟探针） |

## 合成环境冒烟记录

- `eglinfo -B`（WSLg）：
  - **Wayland platform：EGL 1.5 / Mesa / client APIs = OpenGL + OpenGL_ES** ✅
    ——smithay `backend_winit` + `renderer_gl` 的渲染路径可用（GPU 后端
    形态 llvmpipe/D3D12 视运行时定，首帧验收均接受，待澄清①）。
  - GBM platform：`eglInitialize` 失败——**WSLg 预期**（无真实 DRM 设备），
    印证 T1 裁剪集剔除 `backend_gbm/backend_drm/backend_udev` 的正确性；
    该系仅 Stage 2+ 物理机场景需要。

## 构建纪律（本计划全程生效）

- 源码留 Windows 侧（`/mnt/d/autostack/auto-lang/.worktrees/plan-509-dev`），
  WSL 内构建/运行经 `wsl.exe` 驱动；
- `CARGO_TARGET_DIR` 重定向 WSL ext4（防 `/mnt/d` DrvFs 慢 IO），
  实际路径 `/home/visus/target-509`（默认用户 = **visus**，非
  zhaopuming——环境快照勘误）；
- **WSL 构建网络注记**：WSL git 全局 `http.proxy=http://localhost:10809/`
  指向 Windows 侧代理，WSL2 NAT 下不可达且 cargo 取数回退读它——所有
  WSL cargo 调用须前置 `CARGO_HTTP_PROXY=`（空值覆盖）。
- WSL 发行版为 Ubuntu-22.04，默认用户 visus；rust 工具链
  `/home/visus/.cargo/bin`。
