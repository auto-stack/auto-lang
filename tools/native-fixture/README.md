# native-fixture —— 可编程原生窗口夹具（Plan 473）

最小 Win32 顶层窗口 + stdout JSON-lines 协议，供 native dock（NativeSlot）
的 T3 fixture E2E 自动化驱动：B1/B2/B3（dock/undock/几何跟随）、B5（模态）、
B7（自毁回收）、C3（min-size）、C4（倔强窗口）、C5（最大化先恢复）。

独立 Cargo 项目（**非 workspace 成员**，`[workspace]` 空表脱离）；
Windows-only。构建/运行均走 `--manifest-path`，不影响主仓构建：

```bash
cargo run --manifest-path tools/native-fixture/Cargo.toml -- --title t --min-size 300x200
```

## 参数表

| 参数 | 形态 | 语义 | 覆盖用例 |
|---|---|---|---|
| `--title T` | 字符串 | 窗口标题（默认 `native-fixture`） | B1（pid/标题定位） |
| `--min-size WxH` | `300x200` | `WM_GETMINMAXINFO` 强制最小跟踪尺寸 | C3 |
| `--stubborn` | 开关 | 每秒自我复位到初始 rect（倔强窗口） | C4 |
| `--spawn-modal` | 开关 | 窗口内 `modal` 按钮触发模态对话框 | B5 |
| `--self-close N` | 秒数 | N 秒后自毁（`DestroyWindow`） | B7 |

## stdout JSON-lines 协议

每行一个 JSON 对象（即时 flush）：

| 事件 | 形态 | 时机 |
|---|---|---|
| `start` | `{"evt":"start","hwnd":"0x1a2b","pid":4242,"title":"t"}` | 窗口创建后（驱动解析此行拿 HWND/PID） |
| `bounds` | `{"evt":"bounds","x":100,"y":90,"w":640,"h":480}` | 位置/尺寸变化后回显**实际** rect（`GetWindowRect`，写后读回断言） |
| `close` | `{"evt":"close"}` | `WM_DESTROY`（自毁/被关闭——B7/B8 断言） |

## 驱动示例（E2E 模式）

1. `native-fixture --title probe --min-size 300x200` → 解析 `start` 行得 pid/hwnd；
2. 桌面侧发 `dock_native`（pid= 或 hwnd=）→ 等待 `bounds` 行 ≈ 槽位矩形；
3. relayout（分隔条/布局切换）→ `bounds` 行跟随；
4. undock → `bounds` 行回到初始 rect（B2 恢复断言）；
5. `--self-close 3` 场景：3 秒后收到 `close` → 桌面槽位回收（B7）。

## Phase 3 预留（未实现，仅占位）

- `--offer {text|files}`：OLE 拖源（数据交换 A 类用例）；
- 放置目标日志（桌面向夹具拖入的虚拟文件落地断言）。
