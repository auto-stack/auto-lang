# 043-clipboard-bridge — 原生剪贴板三族互通演示(Plan 485)

OS 剪贴板 ↔ 虚拟桌面 App 的 **文本 / 文件 / 图片** 三族双向演示:

| 卡片 | 读取(→App) | 写出(App→OS) | 底层格式 |
|---|---|---|---|
| 文本 | `clipboard_text()` | `clipboard_set_text(s)` | arboard(Plan 418) |
| 文件 | `clipboard_files_get()` | `clipboard_files_set([...])` | Win32 `CF_HDROP` |
| 图片 | `clipboard_image_get()` | `clipboard_image_set(path)` | Win32 `CF_DIBV5` + registered `"PNG"` 双挂 |

## 起跑(示例目录为 CWD)

```bash
cd examples/ui/043-clipboard-bridge && auto run -r vm
```

> 在示例目录内起跑(`cd` 后 `auto run -r vm`):资产路径(`assets/…`)与
> image src 均按进程 CWD(=示例根)解析。

## 实机演示剧本(T4)

1. **文本**:输入框改字 →「写出」→ 去 notepad Ctrl+V 出字;别处 Ctrl+C
   → 回 App 点「读取」→ 上屏。
2. **文件**:Explorer 选中 1-2 个文件 Ctrl+C → App 点「读取剪贴板文件」
   → 路径列表上屏;App 点「写出示例文件」→ 去 Explorer Ctrl+V →
   `demo.png`/`hello.txt` 粘贴落地。
3. **图片**:截图工具/浏览器右键复制图片 → App 点「读取剪贴板图片」→
   缩略图 + 尺寸上屏(临时 PNG 路径);点「写出示例图」→ 画图/浏览器
   Ctrl+V 出图。

## 降级语义(非 Windows / vue 远程端)

`clipboard_files_get/image_get/image_set` 等内建在非 Windows 或未启用
`native-clipboard` feature 时**优雅降级**:`files_get → []`、
`files_set/image_set → false`、`image_get → None`(nil)——.at 代码零平台
分支。vue/web 远程端无 OS 剪贴板场景,同语义(不做 web polyfill)。
文本族降级语义见 Plan 418。

## 结构

```
pac.at              # render: "vm", front_port 4043
src/front/app.at    # 单 widget:三卡 + 根 handler 调 clipboard_* 内建
assets/demo.png     # 64×48 RGBA 渐变(写出示例图/文件载荷)
assets/hello.txt    # 文件族写出载荷
```

> 内建调用留在根 handler(041 纪律:内建编译进 store handler 会产出坏
> 字节码)。剪贴板为进程全局资源,演示按钮为显式触发(本期不做全局
> Ctrl+V 自动路由——Plan 485 非目标)。
