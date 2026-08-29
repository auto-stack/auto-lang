# AutoShell 状态投影协议 v1（S2 接缝合同）

> **版本**：v1（2026-08-29，Plan 472 T3 落码）。双端同版本：vm 端（auto-lang
> `ui/iced/renderer.rs::sync_shell_windows`，本版实现方）与 vue 端（465/shell-track
> 后续，按本文档实现同版本对拍）。
> **定位**：Design 25 §3 S2 的正式化——宿主把驱动事实投影为 shell App 的
> 响应式状态；本文件是与 `schema/aura.at` 并列的机器可对拍合同。
> **上位词表**：S1 命令下行（DesktopBus v1）的动词词表见本文 §4。

## 1. 命名与权属

- 宿主注入 shell 的响应式状态一律 `__wm_*` 前缀（双下划线 = 宿主特权命名
  空间）。shell .at 声明这些变量并**只读消费**；写权属在宿主。
- shell 出向唯一写 `__desktop_cmd`（§4 命令总线）。
- 类型约束：平铺可 for 循环数组（宿主 `write_state_vec` 注入 Obj 数组；
  消费侧在 **view** 的 for 循环——VM handler 内对注入 Obj 数组的字段读有
  B12 族已知缺陷（464 探针），需要 handler 消费时用平行字符串列表）。

## 2. v1 字段表（shell.at model 声明同款）

| 字段 | 类型 | 语义 | 权属 |
|---|---|---|---|
| `__wm_wins` | Obj 数组 `{wid:str, title:str, focused:str, workspace:str, app:str, icon:str}` | **全部**虚拟窗（跨 workspace 全集，dock 运行指示消费；可见性由宿主绘制层自过滤）。`focused` = `"1"/""`；`workspace` = 分区下标串；`app` = 注册表 id（boot 窗缺省 `""`）；`icon` = lucide 名（注册表实时查 → 缺省 `"app-window"`） | 宿主写 |
| `__wm_meta` | str `"layout\tfocused_wid"` | 布局名（free/grid/master-stack）+ 焦点窗 wid（无焦点空串） | 宿主写 |
| `__wm_workspaces` | Obj 数组 `{id:str, name:str, current:str}` | 分区清单；`id` = 下标串；`name` = pack 默认 "Desktop N"（M4 settings 可覆盖）；`current` = `"1"/""` | 宿主写 |
| `__wm_running` | str `",id1,id2,"` | 运行中 app id 集合的**派生串**（pinned 运行指示的 view 条件消费——.at 无法跨列表聚合，宿主派生保持 I9 单一事实源；T4 增补） | 宿主写 |
| `__wm_fp` | str | 投影指纹（§3）；shell 不消费，仅门控 | 宿主写 |
| `__desktop_cmd` | str | 出向命令记录串（§4）；宿主**读+清** | shell 写 |

## 3. 更新语义与指纹门控（协议条款）

- 宿主每 update 周期在 DesktopBus 排空点邻位重算投影（O(窗数) 串接）。
- `__wm_fp` = 逐窗 `"{wid}:{focused},{workspace};"` 串接 + `"|{__wm_meta}"` +
  `"|"` + 逐分区 `"{id}:{current};"` 串接。
- **指纹未变 → 整组跳过写**（防每帧 churn，不置 dirty）；**有变 → 整组原子
  换装**（wins/meta/workspaces/fp 全写）+ shell `view_dirty` 置位触发重渲染。
  投影无部分更新。
- 宿主写状态不触发 shell handler（464 实证）；投影消费一律在 view 侧，
  需要 handler 参与的表面（M2 switcher 等）由宿主显式 `call_handler`。

## 4. S1 命令下行：DesktopBus v1 动词词表

传输载体 = 463 实装候选 B（`__desktop_cmd` 状态变量总线；T1 施工图对账定案，
Design 25 §3 原"候选 A 转正"修订为词表规范，builtin 语法化留 v2——触发条件：
命令需返回值/类型化参数）。编码：`verb\u{1F}arg` 单记录，`\n` 或 `\u{1E}`
连多条；shell.at 控件字符串只可直书 `\t`/`\n`，宿主两套分隔符等价接受；
未知动词/坏记录跳过不 panic（前向兼容）。

| 动词 | 载荷 | 宿主语义 | 引入 |
|---|---|---|---|
| `launch` | app id | 注册表启动新实例 | 463 |
| `close` | wid | 关虚拟窗 + App | 463 |
| `focus` | wid | 聚焦置顶 | 463 |
| `layout` | free/grid/master-stack | 全场重排（当前分区） | 463 |
| `summon` | launcher | 召唤 launcher overlay | 464 |
| `workspace` | 分区下标 n | 切换当前分区（窗口随分区隐现，全保留） | 472 |
| `workspace_next` | （无参记录） | (current+1)%N 环切 | 472 |
| `activate` | app id | dock 固定图标点击：运行中 →（窗在隐藏分区先切分区）聚焦其窗；未运行 → launch | 472 |

## 5. 对拍与验收（I8/I9）

- vm 端实现金样：`ui/iced/renderer.rs` tests `projection_*` 四测（往返/
  指纹门控/分区切换反射/registry icon）。
- vue 端（465 后续）按本表实现同版本投影 + 同指纹规则，对拍项登记后
  消费本文件作基线；版本升级 = 文件名/版本号 + 双端同步 + 对拍重跑。
- I7（shell 无几何操作）、I9（窗口/分区列表唯一事实来自本投影）随行。
