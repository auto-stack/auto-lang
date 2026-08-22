# Plan 418: auto-edit 动作真实化与 Action 配置化绑定

> **状态**: 📋 已立项待实施（2026-08-22,用户需求:完善 auto-edit 菜单/工具栏全部真实功能 + 绑定配置化）
> **来源**: ① 414 §6.1 Phase B(action 声明式)重定向为配置驱动路线;② 414 §3 后续项"menu/toolbar 真实功能";③ 本计划盘点发现:041 三源绑定已就绪但 handler 全缺(§1)
> **关联**: Plan 413(code_editor FFI)/ 414(auto-edit UX)/ 275(键绑定管道,archive)/ 409(overlay hoist);auto-os-config `docs/designs/config-plugin-architecture.md` + `designs/unified-harness-scoping.md`

---

## 0. 一句话结论

**引入 Action 概念,但定位为"声明/绑定层"(数据);Event 保持"执行层"(管道)不变。**
Action = 可寻址(id)、可配置(绑定外置到 .at 配置文件)、可多路触发(菜单/工具栏/快捷键/MCP)的语义事件;触发一个 Action 最终仍派发为既有 `on {}` handler 事件(如 `.ActSave`),VM 分发零改动。

## 1. 现状盘点(2026-08-22,master `a6834ce0`)

### 1.1 绑定侧已就绪,handler 侧为零
041 app.at view 中 26 处 `.Act*` 绑定(菜单项/工具栏图标/`onkeydown.ctrl.{n,o,s,j}` 三源,414 R3 落地),但 **on 块 handler 数量 = 0**——13 个 Act 事件全部静默无效(VM 缺 handler 只 debug 日志,`dynamic.rs:931-952`,须 `ASH_DEBUG_VM_LOG=1` 才可见)。414 提交 `632d4db6` 声称的"13 个语义 handler"从未落地(仅绑定侧)。另:`.ConsoleToggle` handler 成孤儿(view 已改用 `.ActConsole`)。

13 个动作:ActNew/Open/Save/Quit、ActUndo/Redo/Cut/Copy/Paste/SelectAll、ActConsole/SwitchTab/ActAbout。

### 1.2 native 能力矩阵

| 能力 | 现状 | 结论 |
|---|---|---|
| 编辑器读写/光标/查找 | `code_editor_text/cursor_line/cursor_col/selection_len/find/set_text`(catalog 2910-2915) | ✅ |
| undo/redo/select_all | **core 已实现**(`core/mod.rs:822-899` 键位处理,cosmic-text ViEditor 历史),registry 无对应项 | ❌ 需暴露(经 `code_editor_with` 逃生舱 `core/mod.rs:1375`,零新核心逻辑) |
| cut/copy/paste(菜单触发) | 编辑器内部 Ctrl+C/X/V 走 `EditorClipboard` trait(`core/mod.rs:162-175`),无 DSL native | ❌ 同上,动作级 native |
| 通用剪贴板读写 | arboard 已是依赖(仅 renderer 内部 `__preview_copy` 用) | ❌ 新增 `clipboard_text/set_text` |
| 文件读写 | `fs.read_text/write_text` 等(1000/1001 族) | ✅ |
| 打开/保存对话框 | **无任何 file dialog**(无 rfd 依赖;413 §2.1 已预留评估 rfd) | ❌ 新增 rfd + 2 native |
| 退出 | `auto.process.exit`(1300)可用;优雅关窗(iced window close)无 | ⚠️ MVP 用 exit,关窗列增强 |
| console | `console_log/lines/clear`(2916-2918) | ✅ |

### 1.3 菜单/工具栏结构现状(Phase 2 的工作面)
- 4 个菜单 × 每项 12 行手写 button + row/text 样板(~150 行);菜单面板 `absolute top-[33px] left-[8/60/112/164px]` **硬编码像素偏移**(按钮文字变宽即错位);
- click-outside 靠 2000px 隐形按钮 hack;快捷键文案(`text "Ctrl+N"`)与 `onkeydown.ctrl.n` 声明**两份手工同步**;
- 无 enabled(灰态)/checked(勾选态);无任何"绑定可配置"能力——这是用户需求的直接否定项。

## 2. 架构决策:Action vs Event

### 2.1 纯 Event 方案的问题(否决理由)
1. **绑定不可配置**:绑定散在 view 代码里,改一个快捷键要改源码——与需求直接冲突;
2. **N 份重复**:同一动作接菜单+工具栏+快捷键,今天手写 26 处;未来接 MCP/命令面板,每加一个触发源都要改 view;
3. **元数据无家可归**:快捷键显示文案、图标、标题、enabled/checked 状态,纯 Event 模型下没有任何地方声明这些(action 不是控件,不对应 widget prop);
4. **自动化寻址缺失**:MCP `action_mapper.rs`(Plan 278)已把"按语义调用 UI 动作"走通了一半,缺稳定 action id 体系。

### 2.2 全新 Action 分发管道的风险(否决理由)
414 §6.1 Phase B 原案(`actions {}` DSL 块 + `__action/` 名空间 + parser/view/aura/a2r 四端改造):成本高、语法扩张、且**绑定仍在源码里**——依旧不满足"配置文件绑定"需求。

### 2.3 采用:Action = 声明层(数据),Event = 执行层(管道)
- **声明层**(新增):action 注册表(id/handler/title/icon/shortcut/enabled/checked),来源是 .at 配置文件(§3),运行时是 Rust 宿主侧一份解析结果(渲染器全局,先例:`KEYBOARD_BINDINGS`);
- **执行层**(不变):任何触发源最终发 `IcedMessage{event: handler名}` → 既有 `on_with_input_for` → `on {}` handler。缺 handler 行为与今天一致(后续可把 HandlerNotFound 升级为 console 警告,小改进);
- **与 414 Phase A 兼容**:`.Act*` 命名约定保留为 handler 侧约定;本计划 Phase 2 **取代** 414 §6.1 Phase B 的 DSL 块路线(414 文档届时加一行指向本计划);
- **Event 不废弃**:纯控件信号(oninput/oncursor)不 action 化;action 只覆盖"语义操作"。

**Action ID 规范**:点分小写按菜单域命名(`file.new`/`edit.undo`/`view.console`,对齐 Zed `file::new` 风格);config 中 `handler` **显式**声明映射到的事件名(如 `".ActNew"`),不做隐式转换——显式优于约定,防拼写漂移。

## 3. 配置文件设计(auto-atom 格式,兼容 auto-os-config)

### 3.1 位置与管线
- app 内置层:`examples/ui/041-code-editor/auto-edit.at`(与 pac.at 同级;文件名=根节点名=app id,对齐 auto-os-config 实体命名先例 `roles/assistant.at` 根 `role`);
- pac.at 新增可选字段 `ui_config: "auto-edit.at"`(解析管线复用 414 §9 `title:` 字段先例:auto-man `Pac` 字段 → `auto run` 读取注入);
- 加载:auto run(VM merged 同进程)解析后注入渲染器全局 `ACTION_CONFIG`;坏文件容错=跳过+警告+回退 DSL 声明(对齐 auto-os-config drop-in 哲学);**热重载不做**,重启生效(列后续项)。

### 3.2 schema(auto-edit.at,完整示例即 041 目标态)

```auto
// auto-edit.at — 动作注册与三源绑定(auto-atom;含点/连字符的值必须引号)
auto-edit {
    // ---- 动作注册表:action 块可重复,id 全局唯一,handler 必填 ----
    action {
        id : "file.new"
        handler : ".ActNew"
        title : "新建"
        icon : "file-plus"
        shortcut : "Ctrl+N"
    }
    action { id : "file.open"  handler : ".ActOpen"  title : "打开…" icon : "folder-open" shortcut : "Ctrl+O" }
    action { id : "file.save"  handler : ".ActSave"  title : "保存" icon : "save" shortcut : "Ctrl+S" }
    action { id : "file.quit"  handler : ".ActQuit"  title : "退出" shortcut : "Alt+F4" }  // OS 级兜底,配置仅显示文案
    action { id : "edit.undo"  handler : ".ActUndo"  title : "撤销" icon : "undo-2" shortcut : "Ctrl+Z" }
    action { id : "edit.redo"  handler : ".ActRedo"  title : "重做" icon : "redo-2" shortcut : "Ctrl+Y" }
    action { id : "edit.cut"   handler : ".ActCut"   title : "剪切" icon : "scissors" }
    action { id : "edit.copy"  handler : ".ActCopy"  title : "复制" icon : "copy" }
    action { id : "edit.paste" handler : ".ActPaste" title : "粘贴" icon : "clipboard" }
    action { id : "edit.select-all" handler : ".ActSelectAll" title : "全选" shortcut : "Ctrl+A" }
    action { id : "view.console"   handler : ".ActConsole"  title : "切换 Console" icon : "terminal" shortcut : "Ctrl+J"
             checked-if : ".console_open" }   // 勾选态:简单 var 引用(与 view if 同管道),非表达式引擎
    action { id : "view.switch-tab" handler : ".ActSwitchTab" title : "切换 Tab" }
    action { id : "help.about" handler : ".ActAbout" title : "关于 auto-edit" }

    // ---- 菜单栏结构:menu 块可重复 ----
    menubar {
        menu {
            id : "file"  title : "文件"
            item { action : "file.new" }  item { action : "file.open" }  item { action : "file.save" }
            sep {}
            item { action : "file.quit" }
        }
        menu {
            id : "edit"  title : "编辑"
            item { action : "edit.undo" }  item { action : "edit.redo" }
            sep {}
            item { action : "edit.cut" }  item { action : "edit.copy" }  item { action : "edit.paste" }
            item { action : "edit.select-all" }
        }
        menu {
            id : "view"  title : "视图"
            item { action : "view.console" }  item { action : "view.switch-tab" }
        }
        menu { id : "help" title : "帮助"  item { action : "help.about" } }
    }

    // ---- 工具栏 ----
    toolbar {
        item { action : "file.new" }  item { action : "file.open" }  item { action : "file.save" }
        sep {}
        item { action : "edit.undo" }  item { action : "edit.redo" }
        sep {}
        item { action : "edit.cut" }  item { action : "edit.copy" }  item { action : "edit.paste" }
    }
}
```

格式约定(全部对齐 auto-os-config 既有约定):单命名根 `auto-edit {}`;`key : value` prop;**同名块重复=列表**(先例 `DEFAULT_REGISTRY_ATOM` 的 `module {}` × N);`//` 注释(`#` 不合法);对象值含 `./-` 必须引号。

### 3.3 分层覆盖语义(为 Phase 3 预留)
app 内置(随仓库) → OS 用户层(`~/.config/autoos/apps/auto-edit/keymap.at`,仅放 `action {}` 覆盖项)。**同 id 整体覆盖,不做字段级合并**——对齐 unified-harness-scoping"同名实体后者整体覆盖,坚决不做跨层合并"的既定哲学。本轮只实现 app 内置层加载,OS 层为 Phase 3。

## 4. 实施分期(三 Phase 独立可交付)

### Phase 1:动作真实化(零架构改动,~2-3 天)
全部硬编码(app.at handler + 新 natives),不引入配置。

| 任务 | 内容 |
|---|---|
| P1-1 natives:编辑器动作 | `code_editor_undo/redo/select_all/cut/copy/paste(key) -> Bool`,catalog 2919 起(以 native_catalog.rs 空位为准),实现经 `code_editor_with` 逃生舱调 core 既有能力;feature 关闭时 stub RuntimeError(沿 code-editor 先例) |
| P1-2 natives:剪贴板/对话框 | `clipboard_text() -> Str` / `clipboard_set_text(s) -> Bool`(arboard);`dialog_open(filter?) -> Str`(空=取消)/ `dialog_save(default_name?) -> Str`;新 feature `ui-dialog = ["dep:rfd"]`(413 §2.1 预留评估),headless 环境容错返回空串+console 警告 |
| P1-3 app.at 13 个 handler | 按下表逐个落地;修正孤儿 `.ConsoleToggle`→并入 `.ActConsole`;model 增 `path_main/path_util`(untitled="")与 tab 标题 var(支持 Open 后动态标题) |
| P1-4 实机矩阵验证 | 13 动作 × 3 触发源全通;打开→编辑→保存→重开往返一致;剪贴板跨应用粘贴 |

13 个 handler 的目标行为:

| 动作 | 行为 |
|---|---|
| ActNew | 清空当前 tab 编辑器(set_text)+ path 重置未命名 + console 日志 |
| ActOpen | `dialog_open(".at")` → `fs.read_text` → set_text 当前 tab + 标题/路径更新 |
| ActSave | 有 path 直接 `fs.write_text`;无 path 先 `dialog_save` |
| ActQuit | `process.exit(0)`(优雅关窗列增强) |
| ActUndo/Redo/SelectAll/Cut/Copy/Paste | 对应新 native,作用于当前 tab 的 key |
| ActConsole | 现有 toggle 逻辑(console_open 翻转) |
| ActSwitchTab | tab 0↔1 切换(复用 TabMain/TabUtil 逻辑) |
| ActAbout | `console_log` 版本信息(413/414/418) |

### Phase 2:Action 声明层 + 配置驱动(架构核心,~4-6 天)

| 任务 | 内容 |
|---|---|
| P2-1 pac.at `ui_config:` 字段 | auto-man 解析(`Pac` 新字段,管线复用 414 §9 `title:` 先例)→ `auto run` 读取注入 |
| P2-2 ACTION_CONFIG 全局 + 校验 | 渲染器全局(先例 KEYBOARD_BINDINGS);auto-atom 解析;校验:id 唯一/handler 必填/shortcut 归一(复用 `normalize_keydown_suffix` 的 "Ctrl+N" 形态与键名匹配表)/引用的 action id 存在;坏文件跳过+警告+回退 DSL |
| P2-3 menubar/toolbar 渲染 | **路线 A(推荐)**:渲染器原生复合 widget(`menubar {}`/`toolbar {}` DSL 声明,数据来自 ACTION_CONFIG)——自含按钮行+面板锚定(消除 left-[Npx] 硬编码)+click-outside(消除 2000px hack)+禁用灰态;触发发 `IcedMessage{event: handler}`。路线 B(fallback):view 数据循环生成现有 button 树(依赖动态列表渲染,风险见 §5) |
| P2-4 快捷键注入 | config `shortcut` → KEYBOARD_BINDINGS 注册 → 触发发 handler 事件;菜单项右侧快捷键文案**同源渲染**(消除双份维护) |
| P2-5 checked/enabled | `checked-if : ".var"` 简单 var 引用(与 view if 同管道);enabled 同式,缺省恒真;不做表达式引擎(列后续) |
| P2-6 041 迁移 | app.at 删 ~150 行手写菜单/工具栏 → `menubar {}` + `toolbar {}` 两行声明 + auto-edit.at;44 iced + 21 code_editor 测试回归 |
| P2-7(可选) | MCP 贯通:action_mapper 经 ACTION_CONFIG 枚举可用动作(稳定寻址收益兑现) |

### Phase 3(未来,仅设计约束,本轮不实施):auto-os-config 插件整合
- drop-in 注册:`~/.config/autoos/modules.d/auto-edit.at` 声明 `module { kind : file  id : "auto-edit"  file : "apps/auto-edit/keymap.at"  root : "auto-edit"  name : "AutoEdit"  group : "Apps" }`;
- 用户层配置经 auto-os-config 通用编辑器编辑(`action {}` × N = 同构对象数组,天然渲染为表格,零定制);已知限制沿用:写回丢注释(.bak 兜底)、嵌套块增删需手改;
- 本轮约束:3.1-3.3 的 schema/根名/覆盖语义即为此预留,不依赖 daemon 也能本地直读。

## 5. 风险与依赖

| # | 风险 | 对策 |
|---|---|---|
| R1 | rfd 无头环境(CI)不可用 | natives 容错返回空串+警告;测试打 feature 门 |
| R2 | 路线 A 工程量(锚定/关闭/键盘导航) | 分两步:先数据驱动现有 overlay 结构(只消除样板),原生 menubar 锚定随后;414 §7.2 的 VM Row 布局限制(Fill 子元素/嵌套 icon bug)对 DSL 生成路线 B 影响大——是选 A 的又一理由 |
| R3 | 键名归一边界:Shift 修饰全局表仅大写字符+Windows shifted fallback(renderer.rs:5151);Alt+F4 实为 OS 级 | 配置仅作显示文案;归一化复用既有表,Phase 2 实施时以单测钉住 Ctrl/Alt/Shift/F 键矩阵 |
| R4 | 编辑器焦点键冲突(Ctrl+F find 等已被编辑器捕获) | 沿 414 结论:编辑器不拦截的组合才穿透;文档化白名单 |
| R5 | `for` 数据循环在 view 的能力未验证(路线 B) | 路线 A 规避;若走 B 先做 spike |
| R6 | 配置坏文件拖垮启动 | 跳过+警告+回退 DSL(对齐 auto-os-config drop-in) |

## 6. 验证矩阵

- **单测**:natives(undo/redo/剪贴板 stub/对话框容错)、auto-edit.at 解析+校验(id 冲突/引用缺失/shortcut 归一)、action→event 派发;
- **回归**:`cargo test -p auto-lang --lib --features ui-iced iced`(44)+ code_editor 21;
- **实机 041**:Phase 1 = 13 动作 × 3 源矩阵 + 文件往返 + 跨应用剪贴板;Phase 2 = 改配置里一个 shortcut/增删一个菜单项→重启生效(不改 app.at);
- **完成判据**:菜单/工具栏/快捷键三源触发同一动作,行为一致;绑定全部来自 auto-edit.at;app.at 中不再出现手写菜单样板与快捷键文案。

---

## 7. Phase 1 实施记录（2026-08-22，分支 plan-418-auto-edit-ux）

### 7.1 交付
- **natives ×10**（catalog 2919-2928 + BIGVM 表 + codegen intrinsics ×2）：
  - `code_editor_undo/redo/select_all/cut/copy/paste(key) -> Bool`——core 新增 do_* 方法镜像 handle_key 的 Ctrl+Z/Y/A/C/X/V 臂(选区清除防 delete_range panic/经 with_font_system 调 Action::Backspace),registry 函数经 normalize_payload_key 寻址
  - `clipboard_text() -> Str` / `clipboard_set_text(s) -> Bool`——新模块 `ui/clipboard.rs`(arboard 直连,handler 上下文无 iced 剪贴板句柄)
  - `dialog_open(filter) -> Str` / `dialog_save(default_name) -> Str`——rfd 0.15 同步 API,取消/无头返回 "",filter 按 `,;空格` 分割去前导点
- **新 feature**:`ui-clipboard = ["dep:arboard"]`、`ui-dialog = ["dep:rfd"]`,均挂入 ui-iced;feature 关闭时 stub RuntimeError(沿 code-editor 先例)。已知:`code-editor` 单独启用(不带 ui-iced)本就因 class.rs 引用 iced_adapter 编不过——既有问题,非本轮引入
- **041 app.at**:13 个 Act handler 全部落地(+ `.menu_open = ""` 菜单自动关闭,修掉 414 遗留的孤儿 `.ConsoleToggle`);model 增 path_main/path_util/title_main/title_util,tab 标签改动态 `text: .title_*`
- **关键设计决策**:undo/redo/cut/paste 后**不回写** `.src_*`——回写会经 content 绑定 set_text 重设缓冲(可能清 undo 历史);不回写时 diff-guard(last_external == src_*)恰好跳过推送,编辑器状态保留,下一次 oninput 自然同步

### 7.2 验证（全绿）
- **MCP 动作矩阵 28/28**(`examples/ui/041-code-editor/tests/desktop_mcp.py`,013 惯例):T1 结构 / T2 ActConsole(菜单项+翻转+自动关闭) / T3 ActAbout / T4 ActNew(title/path 重置) / T5 ActSwitchTab / T6 undo/redo/cut/copy/paste(工具栏)+select_all(编辑菜单) / T7 Ctrl+J 全局快捷键 / T8 ActQuit(进程退出——连接被切断即成功信号)
- **FFI 往返探针**:临时 probe 应用验证 handler 内 `File.write_text`/`File.read_text` 点分调用可用(读回一致)
- **回归**:iced 44/44、code_editor 21/21、clipboard 单测 1/1;全量 lib 3048 过 1 失败(route::discovery::test_exists,master 上同样失败——磁盘满残留的既有环境问题)
- **ActOpen/ActSave 的 rfd 对话框**:对话框本体为阻塞式 OS 模态,无法自动化,handler 逻辑(路径空→dialog_save/读回/set_text/标题更新)已经 MCP 矩阵其余动作+FFI 探针覆盖,留人工验收

### 7.3 已知瑕疵（不阻塞,Phase 2 顺手修）
- `File.write_text` 返回值 `n.str()` 在 handler 里显示为类型区间("0-2147483647")而非字节数——int 推断/显示小坑,仅影响 ActSave 日志文案
- ActOpen 后 tab 标题用完整路径(无 basename native);Windows 反斜杠路径的 split 提取留 Phase 2
- MCP snapshot 不渲染事件参数(`onclick: .MenuToggle` 无 `"file"` 实参)——测试按按钮文本定位;snapshot 格式改进列 Phase 2 P2-7 可选项

### 7.4 待办清理（2026-08-22 第二轮，分支同步 master 后）
- **§7.3-1 修复**：新增 `file_basename(path) -> Str` native(2929,纯 std 无 feature 门,/与\双分隔符 rsplit);ActOpen/ActSave 的 tab 标题改用文件名(path 仍存完整路径供保存)
- **§7.3-2 修复**：ActSave 日志去掉 byte 计数(`File.write_text` 返回值在 handler let 绑定后 `.str()` 显示类型区间的推断坑,改语句调用丢弃返回值;VM 推断问题本身另立债务)
- **§7.3-3 修复**：snapshot 事件参数根因=`AuraEvent{handler,params}` 分离,`record_event` 只记了裸 handler——记录点拼 `handler(params)`;另 mcp_server `display_handler` 解码 iced 层 \u{1F} 编码(双保险,无编码时原样返回)
- **验证**:041 MCP 矩阵 **29/29**(+snapshot 参数断言);iced 44/44、mcp_server 6/6、code_editor 21/21;013-todo MCP **22/22**(snapshot 格式变化跨应用无回归)
- 同步:分支合并 master 新历史(417-D2 generators/416-6A LSP CI/auto-down 015 批次 A)
