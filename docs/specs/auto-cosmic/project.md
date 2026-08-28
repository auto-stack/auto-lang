# auto-cosmic

> **Status**: experimental
> 路径：`crates/auto-cosmic`（4 个子 crate）  | 技术栈：Rust（iced 兼容 VTree / libcosmic / zbus）

COSMIC 桌面复刻实验（Plan 365）：把 AutoUI 的 VTree 渲染到 COSMIC（libcosmic）桌面组件，
验证"AutoUI → 真实 Linux 桌面生态"的复刻路径。**无任何 workspace 成员依赖它**，
Linux 向实验；Windows 走 headless 委托。

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
| host-libcosmic | VTree→libcosmic Element 真实 lowering（Linux）/ Windows headless 回退 | partial（lowering TODO，见债务簿 365 条目） |
| ports-linux | zbus/UPower/D-Bus 真实适配 | partial（通知 D-Bus push 集成 TODO） |

## plans

- **plan-364** a2r-cosmic-replication-readiness ✅ archived——W1–W7 就绪度清单（依赖本实验）
- **plan-365** autoui-pluggable-host 🟡 archived——可插拔 Host 抽象落地；W3/W4 留档债务簿
