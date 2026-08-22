# Plan 418: auto-edit 动作真实化与 Action 配置化绑定

> **状态**: ✅ 实施完成并复验（2026-08-23 finish-plan 复审归档）——Phase 1 全部落地（natives ×11 catalog 2919-2929 + 13 handler + 实机矩阵）；Phase 2 P2-1/2/3/4/6/7 全落地（P2-7 可选项经 §8.7 snapshot 直读超预期兑现），**P2-5 checked ✓ / enabled-if 声明性推迟 → Plan 423**；Phase 3 按计划原文"本轮不实施"整体拆至 423（§8.10）。残留全面分派（420-423/428）。复审矩阵全绿：iced 44 + action_config 3 + code_editor 23 + mcp 6 + layout 13 + **041 实机 40/40**（HEAD `f12bfb51` 独立 worktree 构建）。
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

---

## 8. Phase 2 第一批实施记录（2026-08-22，P2-1/P2-2/P2-4）

### 8.1 交付
- **P2-1 配置管线**:pac.at `ui_config: "auto-edit.at"` 字段(pac.rs/automan.rs/main.rs,沿 title: 先例)→ `auto run` 解析为绝对路径注入 `AUTO_VM_ACTION_CONFIG`(文件缺失则警告跳过)
- **P2-2 ACTION_CONFIG 模块**(`ui/action_config.rs`,无 iced 依赖):auto-atom 解析(action/menubar/toolbar 三段,同名块=列表)+ 校验(id 唯一/handler 必填/引用存在/快捷键冲突)+ `normalize_shortcut`(Ctrl+N→Ctrl+n 对齐 iced 查表形态,命名键保留大小写)+ OnceLock 全局懒加载(坏文件 eprintln+None 优雅降级);单测 3 项锁定
- **P2-4 快捷键回退层**:iced 键盘监听查表(renderer.rs)与 MCP `autoui_keyboard` 双侧在 DSL 绑定之下加配置回退——自动化与真实按键行为一致
- **041 落地**:`auto-edit.at`(13 action + 4 menu + toolbar 全量声明,menubar/toolbar 段暂为声明性数据)+ pac.at `ui_config:`;`view.switch-tab` 的 **Ctrl+D 仅存在于配置层**,作为回退层生效的验证锚点

### 8.2 验证
- 041 MCP 矩阵 **30/30**(+T7b:配置独有快捷键 Ctrl+D 翻转 tab)
- 回归:iced 44/44、action_config 3/3、mcp_server 6/6
- 启动日志:`VM action config: auto-edit.at (from pac.at)` + `[ACTION-CONFIG] loaded: 13 actions, 4 menus, 10 toolbar items`

### 8.3 P2-3 剩余(menubar/toolbar 渲染迁移)设计备忘
- 路线 A 细化:渲染器侧 `MENUBAR_OPEN: Mutex<Option<String>>` 静态态 + `__menubar_toggle` 内部消息(toast/__preview_copy 同模式,update 拦截改态促重绘);item 点击发配置 handler 事件,update 对已配置 handler 先清菜单态再走 VM 派发
- 面板锚定:合成 absolute overlay(复用 Plan 409 hoist),左偏移按 `8 + Σ(标题字符×12 + 24 padding + 4 mr-1)` 估算(2 字按钮 52px,与现手写 8/60/112/164 间距一致)
- **关键缺口**:合成按钮不经 DSL events map → probe.record_event 不记录 → MCP snapshot 不可见——需为合成子树补 probe 记录(路径需与 vtree 节点对齐)或 vtree 侧从 View 闭包提取事件,否则 MCP 测试将失去菜单项覆盖

### 8.4 P2-3 实施记录（2026-08-22 第三批,menubar/toolbar 配置驱动渲染）
- **交付**:builder 合成 `menubar {}`/`toolbar {}` 标签(读 ACTION_CONFIG;面板复用 Plan 409 overlay hoist,左偏移估算 8+Σ(字符×12+28),2 字按钮 52px 与手写间距一致);渲染器 update 拦截 `__menubar_toggle(id)`/`__menubar_close`(preview-card 内部消息同模式,open 态存 action_config::MENUBAR_OPEN);**任意非 `__` 前缀消息自动关菜单**(心跳/tick 排除);toolbar 图标按钮走 PUA label + variant:icon 同路径
- **041 迁移**:app.at 530→338 行——删 4 菜单×手写样板/8 工具栏按钮/4 个 onkeydown DSL 属性/menu_open 态与 3 个菜单 handler,换 `menubar {}` + `toolbar (style: "ml-auto") {}` 两行;**快捷键全部经配置层**(Ctrl+D 仅配置,回退层锚点)
- **验证**:041 MCP 矩阵 **29/29**(菜单项按标签定位/工具栏按 PUA 图标定位/开关菜单经 __menubar 内部消息/选择后自动关闭/quit 进程退出);回归 iced 44/44 + action_config 3/3 + mcp_server 6/6
- **已知缺口**(后续):①~~合成子树 probe 路径不对齐~~ **已修(第五批,§8.7)**;②MCP 自动化在高负载下偶发应用静默退出(无 panic,疑似资源压力,复跑即过)——需隔离环境排查;③~~菜单项 checked 勾选态未渲染~~ **已修(随 8fc0cf30,§8.6)**

### 8.5 真实鼠标点击排查（2026-08-22 第四批,部分修复,未完全闭环）
- **用户反馈**:右下角 console 图标真实点击无反应(应切换 Console)
- **已确认事实**(SendInput 真实鼠标 + DPI 感知几何 + 每实例实测窗口矩形):menubar 文件/编辑按钮可点(菜单开合);**工具栏图标/菜单项/状态栏 console 图标点击后 update 零消息**;MCP 派发(旁路)一切正常
- **已落地的防御性修复**:①code_editor iced widget 的 update 对 ButtonPressed/Released/CursorMoved 增加 `cursor.is_over(bounds)` 门控(此前窗口内任何点击都进编辑器核心并被 capture_event 吞掉——插桩实测 CursorMain 洪流为证);②layout 由无条件 `Node::new(limits.max())` 改 `limits.resolve(Fill,Fill,max)`;③EE03 tooltip 加 300ms delay(iced Tooltip delay=0 时 hover 即 invalidate_layout,会打断进行中的按钮点击)
- **未解**:上述修复后工具栏/console 图标真实点击仍零消息;剩余疑点=状态栏/工具栏区域的 ml-auto Fill 包装容器或 overlay hoist 层的事件遮蔽,需下一轮以 iced 层级 dump/DevTools 实测定位
- **回归**:MCP 矩阵 29/29 全绿(自动化路径无回归)

### 8.6 真实点击排查收官（2026-08-22 第五批,合并 master 后复测结案）
- **前置**:worktree 合并 master(cfcee3c6,renderer.rs 整文件冲突实为行尾噪音——仓库 blob 历来 CRLF,分支侧已归一 LF;三版本归一后 git merge-file 零冲突)。合并带入 master 的**心跳 200ms→2s + 30s 活联门控**(29c3f93e)——这正是第四批"零消息"的第一根因:200ms 心跳令消息队列积压、事件饿死,press/release 配对被打断(同提交注释自证)。
- **插桩实证**(RAW-MOUSE 订阅探针 + EE01 按钮臂探针):①按钮臂收到的 label/EE01/EE02/EE03/svg 全部正确,合成无误;②press/release 事件能到达 iced 且落点在按钮 bounds 内;③**工作站锁定会令 LockScreenBackstopFrame(explorer)接管全部输入**——第五批中段所有"点击零事件"皆此环境陷阱(WindowFromPoint 返回 explorer/LockApp、SetCursorPos 被弹回 (0,0) 即为锁屏特征);④解锁时段内真实点击**实际成功**:ActNew、ActUndo 均经真实 SendInput 触发,menubar 菜单开合正常——**当前构建(2s 心跳)的真实点击路径是通的**。
- **第四批方法学缺陷**:坐标换算"逻辑×2+窗口 origin"漏了标题栏偏移(GetWindowRect 含标题栏,iced 逻辑原点在客户区),系统性上偏 ~30 物理像素——工具栏图标(h-7≈56 物理高)整枚打偏。正确换算:物理 = 窗口 origin + 标题栏高 + 逻辑×2。
- **MCP 矩阵 4 项失败修复**:T1 哨兵 `code_editor` 只在首渲染前的模板快照(~1.5s 窗口)出现,渲染后快照编辑器为 `textarea`;轮询恰在模板窗命中即提前退出→按钮全找不到。改哨兵为 `"(rendered)"`(tests/desktop_mcp.py)→ **29/29 全绿**。
- **清理**:移除第四批遗留 [DBG-BTN] 探针(renderer.rs 按钮臂)。
- **合并回 master(第五批末)**:分支 rebase 至 master@0b180ba1(零冲突,行尾归一手法见 §8.6 前置)后经 8fc0cf30 合入;master 侧回归全绿(iced 44+cfg 3+mcp 6+矩阵 29/29)。
- **§8.4③ 已实现(随 8fc0cf30)**:菜单项 checked-if 勾选态渲染——item 内容行前置 16px 勾选槽,`.field` 布尔真值时渲染 lucide:check(h-3 w-3 text-zinc-200),空槽保持对齐;MCP E2E 验证(切换 Console 开→✓/关→空)。解锁态真实点击复验 **5/5**(ActRedo ×5,窗口定位正确)——结案铁证。
- **遗留(低优)**:①工具栏图标偶发近黑——**最终构建 3 实例采样不复现**(亮度 231/114 一致),此前暗色观察疑为锁屏期 DWM 降级帧假象,暂不处理,复现再查;②合成子树 probe 路径对齐(§8.4①)未变。
- **结论**:P2-3 真实点击问题**结案**——根因=旧心跳事件饿死(已修)+排查方法学两处偏差(坐标偏移、锁屏干扰),非 iced 事件路由/遮蔽缺陷;ml-auto 容器与 overlay hoist 均已实测排除。

### 8.7 §8.4① 收官:合成子树 probe 路径对齐（2026-08-22 第五批续,snapshot 出现合成按钮 onclick）
- **根因(双重)**:①`convert_element_tracked_ctx` 里 P2-3 会话留下了**重复的 `"menubar"/"toolbar"` 匹配臂**——传 `None` 的旧臂在前,带 path/probe 的新臂不可达(Rust match 首臂命中),记录从未执行;②计数器错位——`child_idx` 不计 sep/面板嵌套层(toolbar 的 sep 占位、menubar 面板项被记成行级子节点且令后续按钮索引漂移)。
- **修复**:删重复臂(tracked 臂接管;probe 禁用时 record_event 早退,零开销);menubar/toolbar 一律按 `children.len()` 真实子位置记录;**面板项记嵌套两段路径** `[base, panel_idx, item_idx]`(FNV 哈希与 vtree 实测 id 全对齐,如 [0,0,4,0]=切换 Console 项)。
- **验证**:snapshot onclick 4→16(工具栏 8+菜单栏 4+面板项+catcher 全出:`onclick: .ActNew`/`__menubar_toggle("file")`/`.ActConsole`/`__menubar_close`);矩阵 T1 新增 2 项锁定检查 → **31/31×2**;回归 iced 44+cfg 3+mcp 6。
- **意义**:MCP 客户端(agent)现可从 snapshot 直接读出合成控件的处理器绑定,不必依赖标签定位约定。

### 8.8 editor 后续批(2026-08-22 第五批终,①②③④ 全落地)
- **① search 高亮渲染补齐**:core 早已产出 `search_matches` 数据但 iced draw 从未消费(413 宣称与实现不符)——widget.rs draw 在 current_line 与 selection 之间补画搜索 quad(选区盖于其上)。
- **① 死测试激活**:`core_config_diff_toggles_wrap_and_vi` 无 `#[test]`(编辑事故),补上后直接通过;顺修相邻重复双 `#[test]`;新增 external-dirty 往返单测——code_editor 22/22。
- **② 模型同步 bug(本轮发现并修复)**:菜单/工具栏的 undo/cut 等 native 在 widget 事件流之外改 buffer,从不发布 on_change → `.src_main` 陈旧 → **菜单剪切后保存会存旧文本**。修复三层:core 加 `external_dirty` 标记(native 置位/widget update 消费补发,真实事件路径);041 handler 在 native 后显式 `.src = code_editor_text(...)` 回写(MCP 派发无 iced 事件,widget 消费不触发,应用层保底);矩阵 T6 重写为**真文本断言**(全选→cut 清空→undo 恢复→redo 再清→copy→cut→paste 往返恢复,10 项含 sel>0 与 src 字段核验)——undo 不再可能静默 no-op。
- **③ 离线布局测试台落地(414 §8.2"单独立项"提前完成)**:`iced_test`(headless wgpu)接入,新 feature `iced-layout-tests` + `layout_tests.rs` 四测:冒烟/414 §7.2 Fill 子元素兄弟存活/418 ml-auto 右对齐锁定/§8.1 嵌套行按钮(文本变体;svg 图标 bounds 需 id 插桩,留后续)。4/4 通过(0.83s)。§7.2 "Fill 子元素兄弟消失"经 into_iced 全链路**未复现**(现行路径无此病,回归锁已立)。
- **④ §8.4② 根因定位(环境,非应用 bug)**:WER 全档仅 2-3 月旧构建 3 例 c0000409/c0000374,近期零原生崩溃记录;"高负载偶发静默退出"实为**并行会话 `taskkill //IM auto.exe` 误杀**(按映像名杀全部实例,本会话亲历两次)。缓解:并行活跃时复制 exe 独立命名跑矩阵(`cp target/debug/auto.exe target/debug/auto-uitest.exe` + `AUTO_BIN=...`);套件已复跑即过。§8.4② 就此关闭(环境项登记)。
- **回归**:code_editor 22 + iced 44 + cfg 3 + mcp 6 + 矩阵 **29/29**(T6 为真文本断言版)。

### 8.9 editor 残留批(2026-08-22 第五批续,A/B/C)
- **A. 414 §8.1"嵌套行 icon-button 消失"结案(不复现)**:iced_test 增 FocusableCollector 自定义 Selector(按钮在 iced 0.14 仅走 container 钩子;fixed_both 按钮另有居中 wrapper → 每图标按钮两条同尺寸记录)。复现树(嵌套行 EE01 图标+文本按钮+扁平图标)实测全部 28×28 完好——与 §7.2 同判:414 时代的"消失"系旧构建/截图不稳产物。回归锁:nested_row_icon_button_keeps_bounds(layout_tests 现 5 测)。
- **B. `code-editor` 单独启用编译不过(§8.4 既有项)修复**:根因=style/class.rs 的语义色 alpha 混合引用 `ui-iced` 门控的 iced_adapter。抽取**纯主题层** `style/theme.rs`(DARK_MODE/ACCENT/WINDOW_WIDTH thread-local + resolve_semantic_rgb/resolve_border_rgb,零 iced 依赖),iced_adapter `pub use` 转发保持全部既有调用点兼容,class.rs 改调 theme。`cargo check --features code-editor` 零错误。
- **C. gallery 过时文案**:code-editor 页"vue 后端降级 textarea"描述更新为 CodeMirror shell 已落地(注明 oncursor/oncontextmenu 事件仍缺)。
- **回归**:build 0 error;iced 44 + code_editor 22 + layout_tests 5 + cfg 3 + mcp 6 + 矩阵 29/29。

### 8.10 后续项立项去向(2026-08-22,editor 残留全面分派)
editor 残留盘点(§8.8-8.9 后)经分析分派为 **5 份新计划 + 1 项不立项**:
- **Plan 428**:代码折叠 Phase B(core 逐 run 自绘重构;fill_raw 无法跳行的架构矛盾,P0 先 cosmic-text 能力调研+路线定稿)。
- **Plan 420**:多 tab 工作区(动态 tab 列表/关闭/`+` 打开/脏标记/拖拽;含 AUTO_*_PATH 环境变量旁路解 ActOpen/ActSave 自动化盲区)。
- **Plan 421**:vue 端 code_editor 契约(五 props 消费+oncursor/oncontextmenu codegen+lang:auto 映射;vi 依赖 P3 显式决策)。
- **Plan 422**:弹层原语(anchor 定位 popover,iced overlay 机制)——menubar 估位/2000px catch 退役 + contextmenu(413 §5.5)复用同一原语。
- **Plan 423**:Action 配置层 Phase 3(热重载 ArcSwap 化/OS 用户层 keymap/表达式引擎复用 resolve_expr_to_value/enabled-if 渲染+按钮 disabled 态,顺消 Plan 402 autoui_check 常驻警告)。
- **不立项**:IME/150% DPI/Linux 人工验收——属 413 既有人工验收尾巴(413 头部"待人工验证"),需实机窗口执行,非开发计划;413 归档前完成。
依赖关系:428/420/421 相互独立可并行(428 原拟 419,撞号改序);422 建议先行(420 的关闭确认弹层、421 无关);423 独立,注意与 032 系(键绑定)并行会话的改动面协调。

## 9. finish-plan 复审记录（2026-08-23，代码级核验后归档）

逐项对照 master `f12bfb51` 实况重验证（不信任勾选；主工作区当时为 423 批并行编辑现场，
验证在独立 detached worktree 构建 HEAD 完成）：

- **P1-1/P1-2 natives**:pass——native_catalog.rs 2919-2929 全注册（编辑器动作 ×6 +
  剪贴板 ×2 + 对话框 ×2 + file_basename），ui/clipboard.rs 存在，feature 门与 stub 按设计。
- **P1-3/P1-4**:pass——app.at 13 Act handler 齐备（孤儿 .ConsoleToggle 已并 .ActConsole），
  实机矩阵 **40/40**（含 §8.8 真文本断言 T6；较 §8.7 的 31 项递增来自 420 批扩容）。
- **P2-1/P2-2/P2-4**:pass——pac.at `ui_config:`(:8) → AUTO_VM_ACTION_CONFIG 注入；
  ui/action_config.rs + 校验单测 3/3；快捷键配置回退层（Ctrl+D 锚点）。
- **P2-3**:pass——app.at `menubar {}`+`toolbar (style: "ml-auto") {}` 两行声明，手写
  菜单样板/onkeydown/menu_open **零残留**（完成判据达成）；§8.6 真实点击结案 +
  §8.7 probe 路径对齐（snapshot onclick 4→16，P2-7 可选项超预期兑现）。
- **P2-5**:partial（声明性）——checked-if ✓（§8.6 E2E 勾选态）；**enabled-if 零消费 →
  Plan 423 承接**（§8.10 明示）。
- **P2-6**:pass——041 迁移完成（530→338 行，样板清零）。
- **Phase 3**:不在本轮范围（计划原文），整体拆至 423。
- **§6 验证矩阵重跑**:iced 44/44、action_config 3/3、code_editor 23 过+2 忽略、
  mcp_server 6/6、layout_tests 13/13（超 §8.8 记录的 5，后续批扩容）、041 实机 40/40。
- **债务补登**:§7.4 曾声称"File.write_text 返回值推断显示区间问题另立债务"但
  KNOWN-DEBT 无此条——本次补登（见 KNOWN-DEBT-AND-RISKS.md 418 条）。

**分类:A（范围内完成）**——enabled-if/Phase 3 属计划内明文分派（423），非裸剩余。
归档。