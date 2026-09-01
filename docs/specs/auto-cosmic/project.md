# auto-cosmic

> **Status**: active（509 线内——Smithay 合成宿主线落地）
> 路径：`crates/auto-cosmic`（5 个子 crate）  | 技术栈：Rust（smithay / iced 兼容 VTree / libcosmic 草案 / zbus）

COSMIC 桌面复刻实验（Plan 365）+ **Smithay 合成宿主线（Plan 509，路线 B 裁定）**：
前者把 AutoUI 的 VTree 渲染到 COSMIC（libcosmic）桌面组件，验证"AutoUI → 真实
Linux 桌面生态"的复刻路径；后者按 509 T1 裁定落 `host-smithay`——Smithay
winit/nested 合成宿主消费桌面协议帧（宿主**无 iced**，shell 经生产渲染链产出
像素帧以纹理上屏），libcosmic 直渲路线经矩阵证伪出局（git-only 无版本锚 +
fork iced 与主线 0.14 并存 + Windows 不构建，见
`docs/plans/reports/509-smithay-route-verdict.md`）。

## 目标与范围

- 用 Auto 复刻 COSMIC 系统组件（时钟、电池、通知、电源）作为桌面能力验收样例。
- 提炼系统端口抽象（ports），供其他 host 后端复用。
- 不做：产品化桌面应用（该角色已由 Design 23/24 虚拟桌面线接管——
  2026-08-26 Design 23 翻转了 Plan 365 的 Windows 宿主裁定）。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| ports | 系统端口接口（通知/电源等）+ mock 实现 | active |
| demo | 时钟 + 电池小程序（验收样例） | active |
| host-libcosmic | VTree→libcosmic Element 真实 lowering（Linux）/ Windows headless 回退 | partial（lowering TODO，见债务簿 365 条目；509 裁定后为观察态——libcosmic 发 crates.io 版 + iced 0.14 rebase 再重估） |
| ports-linux | zbus/UPower/D-Bus 真实适配 | partial（通知 D-Bus push 集成 TODO） |
| host-smithay | Smithay 0.7 合成宿主（winit/nested 开发形态；Linux-target 门控 smithay 依赖，Windows stub）——bind→render→Frame 纹理合成循环，`--frame` PNG 纹理 + `--frames` 帧预算取证 | active（509 Stage 1：骨架 + WSLg/Xvfb 实跑 + shell 首帧像素证据；live attach = Stage 2，见债务簿 509 条目） |

## plans

- **plan-364** a2r-cosmic-replication-readiness ✅ archived——W1–W7 就绪度清单（依赖本实验）
- **plan-365** autoui-pluggable-host 🟡 archived——可插拔 Host 抽象落地；W3/W4 留档债务簿
- **plan-509** smithay-host-stage1 ✅ archived——路线 B 裁定（Smithay+桌面协议宿主）+ host-smithay 骨架 + Linux 合成循环实跑（WSLg/Xvfb 双形态，像素级证据）+ shell 首帧上屏（生产链渲染→宿主纹理）+ I1 零分叉实证（auto-lang 仅 3 处 cfg 差异）
