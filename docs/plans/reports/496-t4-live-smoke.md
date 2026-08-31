# Plan 496 T4 实机冒烟记录：桌面本体（壁纸/图标网格/入口）

> 2026-08-31 · 基线 plan-496-dev（步骤 1–7 提交后）。宿主：窗口模式
> `ui_desktop.exe --apps-dir examples/ui`（472/478/479/487 同入口，MCP 端口
> 39496）；storage 隔离 `AUTO_VM_STORAGE_FILE=tmp/496-t4-storage.json`。

## 1. 实机交互清单（T4 六步对表）

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| 1 | 壁纸 boot 读入全屏铺底（#hex） | **PASS（实机截图）** | `10-initial-wallpaper-icons.png`——桌面底色 #243b55（视觉复核深蓝调成立）；预写键 `shell.desktop.wallpaper="#243b55"` boot 读入（同一预写帧单帧三断言之一） |
| 2 | 图标网格渲染（pinned∪custom 去重 + hidden 排除） | **PASS（实机截图）** | 同帧：左上网格三图标 = 011-calculator + 015-notes（pack 默认 pinned，`013-todo` 被 `shell.desktop.hidden` 排除）+ 014-weather（`shell.desktop.icons` 自定义增）——增删持久化 boot 生效铁证（487「20-restart」同型）；`15-icons-zoom.png` 放大帧 |
| 3 | 层级：App 窗口拖过时图标自然被覆盖 | **PASS（实机截图）** | 同帧：两枚虚拟窗（459-dual-app + 011-calculator）覆盖图标区——z 槽底序实机可见；headless 几何断言 `desktop_surface_z_slot_window_covers_icons`（T3） |
| 4 | 双击启动/聚焦两臂 + 右键三项菜单 | **BLOCKED（OS 注入通道，487 先例同族）→ headless 全链** | CUA 像素身份守卫对活渲染面持续拒绝（472/478/479/487 前台竞争家族 2026-08-30 变体，487 T4 报告 §4.1）。headless 覆盖：`desktop_surface_at_loads_interactions_and_dispatch`——IconMenu → 菜单三项（打开/从桌面移除/更换壁纸…）渲染、ActivateApp → `activate\t<id>` 动词记录（472 两臂复用，宿主臂 = `activate_app_focuses_running_or_launches_new` 既有测）、MenuRemove → `shell.desktop.hidden` 直写、MenuWallpaper → `open_settings` |
| 5 | 空白点击取消 | **BLOCKED（同 #4）→ headless** | 同测 BlankPress handler → menu_id 清空（463 桌面层语义——GlobalPress 空白命中不改 WM 焦点，代码序实核 renderer.rs `WmCommand::GlobalPress` 臂） |
| 6 | settings 壁纸入口闭环（写键 → 重启生效） | **PASS（写手链 headless + 重启铁证）** | 写手：`settings_appearance_wallpaper_section_writes_storage`（Nav appearance / DraftWallpaper / SaveWallpaper → `shell.desktop.wallpaper` 落键 / 空草稿不写 / cfg 快照刷新 / saved 提示）。重启生效：#1 预写键即该链产物形态的 boot 消费实机帧 |
| — | 壁纸热切换（增强候选，待澄清②） | **未做（按计划 v1 语义）** | v1 重启生效（487 pinned 同语义）；投影字段重注入+指纹门控支持的热切换留增强——boot 注入为一次注入无指纹门控（协议 §2.1），热切换需增注入周期，不在 v1 范围 |

## 2. BLOCKED 项 headless 覆盖指针（487 先例成文）

| 阻塞项 | 覆盖测试（全绿） | 语义链 |
|---|---|---|
| 双击 activate 两臂 | `desktop_surface_at_loads_interactions_and_dispatch` + `activate_app_focuses_running_or_launches_new` | 真 desktop.at 装载 + ondblclick 事件面（VM parser→MouseArea→convert_view_messages→iced on_double_click 四层，转换前后双击臂存活断言）→ activate 动词 → 宿主两臂 |
| 右键菜单三项 | 同上 | IconMenu(e.id) → menu_id 置位 → 菜单面板渲染（打开/从桌面移除/更换壁纸…三 Button label 断言） |
| 移除持久化 | 同上 + `desktop_surface_storage_roundtrip_and_wallpaper_resolution` | MenuRemove → hidden 续写落键 → boot 合并去重排除（T2b 注入形状断言） |
| 空白点击 | 同上 | BlankPress → 菜单关闭；WM 焦点不动（代码序 GlobalPress 臂） |
| 壁纸解析回退链 | `desktop_surface_storage_roundtrip_and_wallpaper_resolution` | #hex 直传 / 坏路径回退默认 #090e1a / 存在路径保留 / 逗号键解析 |
| 合并去重注入 | `desktop_surface_merge_dedupe_and_injection` | pinned 先列 + custom 去重接排 + hidden 排除 + icon/label 注册表解析回退 + `__desktop_bg` 双分支片段 |
| 层级几何 | `desktop_surface_z_slot_window_covers_icons`（iced-layout-tests） | 图标格 ⊂ 虚拟窗矩形（Stack 底序覆盖）+ 窗客户区/chrome 同区共存 |
| 双端同源 | `test_a2vue_desktop_surface_asset`（a2vue 金样） | 真资产 → SFC 对拍（@dblclick/@contextmenu.prevent 事件面） |

## 3. 证据目录

- `assets/496-t4/10-initial-wallpaper-icons.png`：预写键 boot 全窗帧——壁纸
  #243b55 铺底 + 左上图标网格三枚（011/015/014——hidden 013 移除生效）+
  两虚拟窗覆盖图标区（层级）+ 任务栏正常。
- `assets/496-t4/15-icons-zoom.png`：图标区放大帧。
- `tmp/496-t4-storage.json`：隔离 store（预写三键形态保留供复查）。

## 4. 执行期发现

1. **OS 注入通道受阻**：与 487 T4 §4.1 同族（CUA 像素身份守卫对活渲染面
   拒绝）——交互项转 headless 全链对表（§2），实机证据 = 渲染帧 + 重启
   铁证（预写键单帧三断言）。
2. **desktop.at 无 Init 的 boot 噪音**：首跑日志 `[VM-HANDLER]
   DesktopSurface.Init failed: handler not found: Init`——补空 Init handler
   （本地态复位）收敛，无语义变化。
3. **vm 端 MouseArea 转换臂缺口**（步骤 4 执行期发现）：VM 动态路径
   `convert_view_messages` 缺 MouseArea 臂（落 `_ => Empty` 兜底）——484
   图表族经 Rust codegen 不经该转换故未暴露；本计划补显式臂（同时修复
   hover 面板在 VM 动态组件中的潜在缺席）。
4. **vue 生成器插值 class 缺口**（步骤 7 金样暴露）：`${.field}` 此前原样
   落静态 class（浏览器侧废 token）——interpolated_class_parts 拆分修复
   （extract_classes 双臂 + shadcn row/col class/style 臂 + push_style_class）。
