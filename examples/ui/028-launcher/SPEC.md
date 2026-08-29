# SPEC — 028-launcher（Plan 464）

桌面 launcher：palette / grid 两形态的应用搜索与启动器。同一份 `src/front/app.at`
跑三端（I5）：vue 独立调试（`auto run`）、desktop vm overlay（`ui_desktop
--fullscreen --apps-dir examples/ui`，由桌面 shell 经 463 接缝召唤）、465 web
形态（由 465 复验对拍）。

## 接缝（463 消费面）

| 方向 | 通道 | 内容 |
|---|---|---|
| 下行 | 宿主 `SummonLauncher` 事件 / shell 总线 `summon\tlauncher` | 懒挂载 → 注入 → `visible="1"` |
| 下行 | 宿主状态注入（平行字符串列表） | `apps_names/apps_titles/apps_icons/apps_cats/apps_lns/apps_lts`（ln/lt 预小写）+ `hosted="1"` |
| 上行 | App `__desktop_cmd`（DesktopBus 记录形） | `launch\t<name>` → 宿主 `LaunchApp`；启动后 App 自隐匿（`visible="0"`） |
| 聚焦 | `__focus_input="1"` 状态约定 | update_inner 尾部消费 → `operation::focus(prompt_input)`（消费即清零） |

**注入形态约束（重要）**：注册表清单以平行字符串列表注入，**不用** Obj 数组——
VM handler 对 `write_state_vec` 注入的 Obj 元素字段读失效（013 audit B12 同族，
探针测试 `injected_obj_array_for_field_read` 钉死 `"0,0"`）；字符串列表下标读
保真。`ranked`/`gridrows` 行对象为 handler 自建（view 侧读取可用）。

## 排序规则（SPEC 钉死 · 465 对拍项）

匹配域：`ln`（id 小写）与 `lt`（title 小写），query 侧 `.lower()`。全整数权重。

### 非空查询

```
tier:  exact(ln==q 或 lt==q)=0 > prefix(lt 前缀)=1 > 词首=2 > 子序列=3；不匹配=剔除
词首:  位置 0，或前一字符 ∈ {空格, '-', '_'} 处开始的 prefix 命中
子序列: 双指针顺序匹配 ql 全部字符

score = tier*100 + 注册表序号(si) - recent 折扣
recent 折扣: 名称在 recent 列表第 rk 位（0 起）→ 减 max(5-rk, 1)
```

- 折扣 ∈ [1,5] < 100 ⇒ **只在同档内重排，不跨档**（SPEC 保证）。
- 同分（score 相等）按注册表序稳定（选择排序先到先得）。
- 输出：score 升序；`9900` 标记已输出/剔除。

### 空查询

1. recent 组（`sub:"recent"`）：按 recent 顺序（最近在前）列出注册表中存在的项；
2. 其余按注册表序（`sub = category`）。

### recent 维护

- `Launch(name)`：去重置顶，上限 5；持久化到 storage 定长 5 槽键
  `launcher.recent_apps.0..4`（`storage.get` 产 `string|null`，vue 生成码无
  `?? ''`——槽值只做 `!= ""` 比较，见 464 实测注）。
- `Init`：按槽序恢复。

## 键盘流（P3）

打开即聚焦（`__focus_input`）→ 输入即过滤重排（`SetQ → ApplyFilter`）→
`↑↓` 移动选中（列表首尾回绕）→ `Enter` 启动（palette=选中行 / grid=聚焦瓦片）→
`Tab` palette↔grid（grid 内 `↑↓` ±4 行回绕、`←→` ±1 回绕）→
`Esc` 逐层退出：清词 → 退 grid → 关闭。

- bind 通道：vm=keyboard_subscription（bind 块）；vue=生成器 `__autoBindKeymap`
  window keydown 层（464 T2 补齐，vm key_bindings 同源镜像）。
- desktop 模式：launcher 可见时键盘独占（vwin App 订阅 focused 门控加
  `!launcher_visible()`）；Esc 被焦点输入框/IME Captured 时由 launcher 订阅的
  `escape_forward` 转发同一 `.Escape`（handler 以 `visible=="1"` 门控，双派发幂等）。
- 输入框本体聚焦依赖固定 Id `prompt_input`（Plan 047）；desktop 多 App 同 Id
  冲突为 v1 已知边界（launcher 层最上，召唤场景无竞争输入框；462 焦点分区
  命名空间跟进）。

## 已知边界 / 记账（T5/T7）

1. **lucide 图标未渲染**：行/瓦片用 title 首字母 monogram 瓷贴。`icon` 字段已
   全链路携带（pac → 注册表 → 注入列表），但 vue `icon` 组件的 lucide 导入
   收集只认静态名字，for 循环动态 `name:` 绑定无导入产出（464 实测）。补图走
   414 先例，待动态名支持后一行切换。
2. **长清单滚动**：结果列表 `max-h-80 overflow-auto` 在 vue 生效；vm 侧无
   scrollable 映射，29 条全量渲染无滚动（iced `scrollable` 适配待需）。
3. **任务栏物理点击**：split_mut 对 windowless 特权 App 的静默丢弃已修
   （`windowless_shell_split_mut_and_bus` 单测钉死）；物理点击的实机复验受
   测试沙箱鼠标注入通道所阻（mouse_event/SendInput 均未达窗口），463 §5.4
   顺延项中「任务栏点击（聚焦/关闭/召唤）」与「空桌面逐个关闭」留待真人
   会话复验。
4. **中文搜索**：注册表标题全拉丁；IME 组合→提交→`SetQ` 管线已实测（提交
   文本正常过滤），中文标题应用就位后中文搜索即天然可用。

## 验收入口

- vue：`auto run` + `node tests/vue_verify.mjs`（5 组断言）
- vm：`auto run -r vm` + `python tests/desktop_mcp.py`（24 断言）
- desktop：`cargo run -p auto-lang --features ui-iced --example ui_desktop --
  --fullscreen`，Ctrl+Alt+Space（IME 安全召唤键；Ctrl+Space 同键位族，中文
  IME 激活时被系统抢占）→ 搜 `calc` → Enter
