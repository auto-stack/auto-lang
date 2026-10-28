# autos-desktop-entry

> **Status**: active（PLAN-876 SD-01）
> 范围：iced/VM 轨桌面入口的所有权、依赖闭包与启动健康面契约

## 归属

iced 轨桌面（虚拟桌面 + shell + 注册表启动）是 **AutoOS 产品入口**，owner = auto-os 仓：

| 面 | owner | 位置 |
|---|---|---|
| 入口 bin（`ui-desktop.exe`） | auto-os | `host/desktop`（workspace 成员 `os-desktop`） |
| 渲染/WM/会话/投影/桌面协议 | auto-ui | `auto-ui-runtime` |
| shell 四件 `.at` 与 app 注册表内容 | auto-os | `shell/`、`apps/`、`apps.manifest` |
| 语言/VM | auto-lang | `auto-lang-core`（本入口依赖） |

依赖闭包契约：入口只依赖 `auto-ui-runtime`（`ui-iced`、`mpv-widget`）与
`auto-lang-core`，**不得依赖 `auto-lang`**（编译器层）。旧的
`auto-lang/crates/auto-lang/examples/ui_desktop.rs` 已退役；语言仓不再承载
OS 桌面入口。

## 构建与运行

- 构建：`cd <auto-os> && cargo build -p os-desktop --bin ui-desktop`（根 workspace + `Cargo.lock`）。
- 链接参数 `/STACK:33554432` 与 `jobs` 在 auto-os `.cargo/config.toml`：桌面解释态
  渲染复杂视图树在默认栈上会起动溢出，**从别处 CWD 构建会丢该旗标**。
- 运行：CWD = auto-os 仓根，`--apps-dir <path>`；`AUTO_OS_ROOT`/`AUTO_VM_STORAGE_FILE`
  已设时不覆盖（脚本/隔离测试用）。存储缺省 = `~/.config/autoos/desktop-storage.json`。

## 启动健康面（`[desktop-health]`）

boot 期一次只读自检，输出一行 `key=value`（脚本可 grep）+ 零到多行可执行
`hint=`。键集：`apps_dir`/`apps_dir_src`、`registry`/`desktop_visible`、
`declared_exe_missing`、`apps_empty`、`shell_pack`/`shell_pack_src`、`storage`、
`orphan_icons`、`auto_cli`。语义边界：

- 自检**不**写 storage、不改注册表、不构建产物；
- `apps_empty` 的签名 = 目录存在但无 `pac.at` 且除 `.git` 外无内容（子模块未
  初始化 / 工作区被清空），有内容的非 app 目录不计入；
- `skip=no_pac_at:N` = `apps.manifest` 的 repo 条目中 `pac.at` 缺席（即注册表
  跳过面）的数量；解析失败等原因当前**不**单独统计（不虚构计数）；
- `orphan_icons` = `shell.desktop.icons` 中**既无注册表落点、也不在
  `apps.manifest`** 的 id 数——manifest 落点算已知，避免子模块未初始化时把
  合法 id 误判为失效；且 `apps_empty > 0` 时**不**给"清理 storage"的建议
  （那会丢桌面快捷方式）。

## 缺产物语义

`pac.at` 的 `desktop_exe:` 声明在产物**在场**时才参与原生启动选择：

| manifest `launch` | 产物在场 | 产物缺失 |
|---|---|---|
| `"native"` | 原生 outproc | 严格失败（launch 错误含 `auto build -r rust` 提示） |
| `"vm"` | 解释态 | 解释态（不打缺失告警） |
| 缺席 | 原生 outproc | **降级解释态** + 一行 stderr warn |

降级的用户可见面当前是 stderr 一行（宿主无 toast 通道接入）；boot 自检的
`declared_exe_missing=` 与 hint 是主要可见面。

`auto` CLI 解析序：`AUTO_BIN`/`AUTO_CLI` → 自身目录及上三级 → `PATH`
（Windows 候选名 `auto.exe`/`auto.cmd`）；缺失时 rqhost daemon 不孵化，但桌面
正常进入并给可执行提示。

来源：[PLAN-876](../../plans/876-autos-desktop-entry-and-launch-health.md)
