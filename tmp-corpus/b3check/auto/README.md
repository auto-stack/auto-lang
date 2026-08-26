# auto-os-config — Auto sources (Plan 006 + Plan 007)

`src/` 的 Vue 组件层已 100% 由本目录的 Auto 语言源码生成（Plan 006 完成，
2026-08-24）。本目录是嵌入式 Auto 工程（jade-garden plan 011 同款模式）。
Plan 007 在此基础上加出 **VM 桌面版**（`vm/` 子工程，视图分叉、逻辑统一）。

## VM 桌面版（Plan 007）

单工程双后端（widgets-gallery 模式）：同一 `auto/` 工程，CLI `-r vm` 切换。
`use back.api:` 一行导入双后端各自解析——vue codegen → `@/lib/api`（TS），
vm 解释器 → `src/back/api.at`（全文本实现：transport 配方 + inferField 移植 +
投影/编辑 text 手术）。3 个 store 为两轨共享单一真源（handler 按 vm gotcha
改写、model 形状不变，vue widget 零改动）。

```
auto/src/back/api.at          # vm 侧 back.api 实现（~70 pub fn，纯文本管线）
auto/src/front/app.at         # vm 桌面真根（vue gen 产出未引用的 App.vue，无害）
auto/src/front/*_vm.at        # vm 视图层：sidebar/theme_picker/vm_editor/vm_daemon/vm_collection
```

运行与测试：

```sh
cd auto && auto run -r vm             # 桌面窗口（1280x800），直连 daemon :17701
AUTOOS_DAEMON=http://… auto run -r vm # 远端 daemon 覆盖
node scripts/e2e-vm.mjs               # vm 轨回归门（MCP 驱动，9 断言）
AUTOUI_MCP_PORT=9320 auto run -r vm   # 自定 MCP 通道（调试用，端点 /mcp）
```

与 vue 轨的禁令不同：**vm 工程内可以 `auto run`**（无 gen 直写风险）；
仓库根的 `auto run` / `auto build` 依旧禁止。
上游锚定：auto-lang commit `3d45fb10d`（上游 rebuild 后先 vue regen+e2e 再 vm 冒烟）。

### vm 编码规范（VG 清单，Phase 1/2/4 实证）

- VG1 fn 模块禁 `use auto.str`（与 http 共编 → `str.status` 链接失败）；字符串用方法调用形态
- VG2 handler 多语句禁同行（吞链）
- VG3 http 三不用：`res.status()`（哨兵）/builder 链（二次调用崩）/`res.body()`；写后 GET 验证
- VG4 map 字段读取仅 fn 模块可用；handler 内对 store/返回值仅单跳链读（`r.data.x` 两跳为空）
- VG5 禁 `concat`/handler 双变量 for/动态 map key——loop+push 重建
- VG6 模块导入用冒号形式 `use x: fn`
- VG7 handler 崩溃回滚整次状态写入
- VG8 `json.parse` 是占位——全走 `json.keys/get/get_at/type_of/len` 文本工具链
- VG9 `File.write_text` 同进程读后写延迟可见（model 权威、文件作下次启动）
- VG11 `json.get_at` 仅接受 JSON 文本（数组用索引计数 for-in）
- VG12/13 数组不跨 fn 边界作实参；fn 返回一律扁平（无嵌套 map/数组字段）
- VG14 `.find(闭包)` 不可用——选择逻辑下沉 api fn（扁平返回）
- VG16 一 widget 一 store + **store 方法名全工程唯一**（消歧按方法名，撞名落到错误 store）
- VG17 `json.keys` 返回裸 key（输出需 quote_json 重包）
- 事件参数只能标量/裸循环变量：store 列表循环上的字段访问/map 实参会打死 MCP——
  store 提供平行 `names`/`entry_keys` 字符串数组，view 用 `for i, e` + 索引参数
- msg 声明必须含全部 handler（vm 消歧表按 msg 声明匹配）
- `popover` 在本构建为解析毒药（确认层用普通 if 块）；`substr(a,b)` 闭区间；
  map 字面量内禁空数组/`.len()` 调用

### vm 轨已知偏差（v1，登记 KNOWN-DEBT）

- 侧栏无分组折叠；集合无过滤框；select 控件为自由文本+提示；markdown sidecar 单行；
  表格/subform 以只读 JSON 文本展示；块增删（Plan 005 特性）未暴露
- `json.keys` 字母序（serde_json 无 preserve_order）——字段顺序与 web 版不同

## 共享样式词汇（Plan 008 Phase 2 定稿，双后端单一真源）

vue（Tailwind 3.4）与 vm（auto-lang `ui/style/class.rs` → iced）消费**同一套类串**。
定稿对照（`styles.css` 令牌 → 共享类串；旧令牌随 Phase 3 各批退役）：

| styles.css 令牌 | 共享类串 | 备注 |
|---|---|---|
| `--accent` | `bg-primary` / `border-primary` | vm 端 primary 由 accent thread-local 动态算色，与 web `hsl(var(--primary))` 同源 |
| `--accent-light`（10% tint） | `bg-primary/10` | alpha 修饰双端可用（P1 实证）；选中态底色 |
| `--bg-active` | `bg-primary/10` | 同上（合并为一条） |
| `--accent-hover` / `--bg-hover` | `hover:bg-primary/90` / `hover:bg-[#ededed]` | **hover: 仅 web 增强层**，vm 静默跳过；核心状态禁依赖 hover |
| `--bg-app #f3f3f3` | `bg-[#f3f3f3]` | 中性色一律确定性 hex/色板（P4：不受 vm DARK_MODE 漂移） |
| `--bg-sidebar #f9f9f9` | `bg-[#f9f9f9]` | 同上 |
| `--bg-card #ffffff` | `bg-white` | |
| `--bg-search #f0f0f0` | `bg-[#f0f0f0]` | |
| `--border #e0e0e0` | `border-[#e0e0e0]` | |
| `--text-primary #1a1a1a` | `text-[#1a1a1a]` | |
| `--text-secondary #616161` | `text-[#616161]` | 注意：`text-gray-500`(#6b7280) ≠ 原值，勿用色板近似关键中性色 |
| `--text-muted #8a8a8a` | `text-[#8a8a8a]` | 同上 |
| `--danger #c42b1c` | `bg-[#c42b1c]` / `text-[#c42b1c]` | Win11 danger 精确值 |
| `--success #107c10` | `text-[#107c10]` | |
| `--radius 8px` / `--radius-sm 4px` | `rounded-lg` / `rounded` | 双端同值（P3 实证 vm rounded-lg≈8px） |
| 字号 base/sm/lg/xl = 14/12/16/20px | `text-sm`/`text-xs`/`text-base`/`text-xl` | 基准 14px = text-sm |
| 字重 500/600 | `font-medium` / `font-bold` | vm 三级递进 ✓（P3） |
| `--ring` 焦点环 | （web-only 增强） | vm 无焦点环，登记残余差异 |

**词汇硬规则（Phase 2 探针实证，违反即 vm 静默降级）**：

1. **间距禁小数**：`py-1.5`/`px-2.5`/`gap-1.5`/`gap-0.5` 在 vm **被静默丢弃**
   （class.rs 按 u16 解析尺寸）——需要 6/10px 等值用任意值 `py-[6px]`/`px-[10px]`，
   或整数档 `py-1/2`、`gap-1/2`。web 端两者皆支持。
2. **中性色禁模式自适应 token**：`background`/`card`/`foreground`/`secondary`
   （vm 解析 secondary 为 indigo 品牌色）全部不进共享词汇；唯一语义 token = `primary` 族。
3. **hover:/焦点类**只作 web 增强，写法合法（vm 容错跳过），核心状态必须有静态等价。
4. 条件样式 = store/handler **预计算完整类串**存字段（`row_class` 型）或视图
   `if` 双分支静态串（仅限无子元素的简单控件——见规则 6）。
5. **类串绑定必须用 `class:` 属性**：vue codegen 对 `style:` 只有字面量才映射
   class，绑定形态（`style: m.nav_class`）会编译成 `:style` 内联样式；
   `class: m.nav_class` 两端皆正确（038 minesweeper 官方姿势，批 1 实证）。
6. **if 表达式 style 禁用于带子元素/事件的按钮**：vm 会把 style/onclick 提升
   到包装容器（按钮本体裸奔）——静态串按钮不受影响；条件类一律下沉 store。

**D6 组件基线串（Phase 2 定稿，Phase 3 迁移即用）**：

| 组件 | 基线类串 |
|---|---|
| 按钮 | `px-5 py-2 rounded text-sm border border-[#e0e0e0] bg-white text-[#1a1a1a] hover:bg-[#ededed]` |
| 主按钮 | `px-5 py-2 rounded text-sm bg-primary border-primary text-white` |
| 危险按钮 | `px-5 py-2 rounded text-sm bg-[#c42b1c] border-[#c42b1c] text-white` |
| 小按钮 | `px-3 py-[6px] rounded text-xs border border-[#e0e0e0] bg-white` |
| 输入框 | `w-full px-[10px] py-[6px] text-sm bg-white border border-[#e0e0e0] rounded` |
| 卡片 | `bg-white border border-[#e0e0e0] rounded-lg p-4` |
| 选中态行 | `bg-primary/10 text-primary font-medium` |
| muted 文字 | `text-xs text-[#8a8a8a]` |

双端定稿对照图：`tmp/phase2-dual-baseline.png`（左 web / 右 vm，Phase 2 归档）。

**store 迁移配方（D3/008，Phase 3 各批执行）**：

- 条件类串在 store handler 内拼**完整字符串**入 model（如
  `nav_class = if active { "bg-primary/10 text-primary font-medium px-3 py-2 rounded" } else { ... }`），
  视图 `style: x.nav_class` 字段直读（P1 实证三种形态全可用）；
- 搜索过滤/分组展开投影从 widget `computed`（ext fn）下沉 store（`.includes()`
  等视图不可用调用一并消除）；
- 列表数据维持 007 的"文本 + 计数 fn + 逐项扁平 getter"形状，行类串进扁平 getter 字段。

## 特许手写清单（src/ 下唯一的手写前端文件）

- `index.html`、`src/main.ts`（7 行壳：挂 AppShell + styles.css）
- `src/styles.css`（CSS 变量设计系统，139 行，原样保留）
- `src/lib/api.ts`（传输层：fetch/超时/枚举缓存/主题持久化/entries 投影）
- `src/editor/types.ts`（inferField/inferColumn 推断引擎——值形状字符串
  数学，按 ext 政策留 TS）

## 重新生成

```sh
bash auto/gen/regen.sh
# = auto build -d . --gen-only（Parse error / 编译告警都会中止部署）
#   + 组件 sed 改写 @/ext/... → ../../auto/src/front/utils/...，落 src/components/
#   + store composable sed 改写 @/lib/api → ../../lib/api，落 src/stores/auto/
```

改完跑：`npm run build`（vue-tsc + vite）+ `./scripts/e2e.sh`。

**禁止**在仓库根跑 `auto run` / `auto build`——会以占位覆盖真实代码
（jade-garden 踩过的地雷）。

## 架构约定（Plan 006）

- **D4 描述符驱动动态表单**：handler 把 config/entity body 投影为值内嵌的
  描述符数组（ext：configEntries/bodyEntries/tableInfo），view 纯
  `for`+`if/else` 渲染——零函数调用、零动态索引（vm 模式约束）。
- **D5 整体替换（跨后端规范，2026-08-24 起不再是被 vue bug 强制）**：
  auto-lang Plan 443（`38adb1ef`）已修复未绑定 model 变量的深变异响应性
  （回归 `ref<>`，深写/深 push 视图就地更新——本仓库已运行时实证）。
  但**不可变重建继续作为规范保留**：vm/rust 桌面目标的安全基线 +
  TableField 单元格 v-model 深写等残留形态的一致性。ext 的
  setCfgEntry/setCell/setEntry 承担重建，`.at` 侧不直写深变异。
- **store facade**：生成的 composable 返回裸 ref 对象——模板嵌套访问不解
  包，ext 里 `reactive()` 包一层。
- ext 政策：只装 DSL 真表达不了的（fetch/localStorage/document/confirm/
  推断引擎/不可变重建辅助）。
- 深变异回归钉子：auto-lang `examples/capability-tests/041-model-deep-
  reactivity/`（`ab34fa9f`，自本仓库 Phase 6 提案归档）。

## Gotcha 清单（21 条，编号供回填 auto-lang）

- **G1** 元素的属性与子元素必须换行分隔：`label { class: "x"  text y }`
  会把 `text` 解析进 class 表达式（"Expected infix operator"）。
- **G2** 事件内联参数只能是：model 引用 / 循环变量 / 字面量 / map 字面量
  / `$event`。`!x`、`a + b` 等表达式被拒（strict 报错）——计算进 handler。
- **G3** `for x in <path>` 头部只接受裸路径，`.map[key]` 动态索引是解析
  错误——把值预投影进描述符（mergeCols/bodyEntries 即为此）。
- **G4** `col`/`row` 即使 `default_classes: off` 也注入 `flex flex-col
  gap-4` 布局类——移植手写 CSS 时一律用原生标签 + 显式 `class:`。
- **G5** handler 局部变量的类型标注（`Array<str>`/`Array<any>`/`[]any`）
  被 codegen 丢弃，strict 模式下 `var out = []` 报 TS7034。惯用法：
  表达式初始化累加器 `var out = .xs.filter(x => false)`。
- **G6** `[]str` 发射 `string[]`（按内容选）；model 层 `Array<str>` →
  `any[]`。
- **G7** 解析错误打印 `Parse error in ...`，与 `Warning: Failed to compile`
  格式不同且 `--gen-only` 仍 exit 0——regen.sh 两种都 grep（已加固）。
- **G8** strict 模式对 `th`/`td` 级 v-for 也强制 `:key`（R006）；混合
  if-分支子节点的 for 会被包进无 key 的 div——加带 key 的 wrapper div。
- **G9** 手写入口（main.ts）只挂生成组件，不需要 h()——但任何
  `createApp({ template })` 需要 runtime 编译器（vite 默认没有）。
- **G10** store 不支持 `use { }` 块（只认 model/msg/computed/on/watch）；
  store 导入只能用文件级 `use back.api:`（映射到 `@/lib/api`）。
- **G11** `view` 是保留属性名：局部变量 `r.view.x` 被静默改写成 `.x`——
  字段名避开（我们用 `data`）。
- **G12** store 文件里的模块级 `fn` 参数被当 ref 发射（`expanded.value`）
  ——纯函数放 ext。
- **G13** ext 导入（use 块）不自动 await（只有 back.api 会）——异步用
  `p.then((r) => { ... })` 惯用法（jade StatusBar 先例）。
- **G14** computed 里读另一个 computed 不要手写 `.value`（codegen 自动加，
  手写变双 `.value.value`）；handler 里读 model 也由 codegen 处理。
- **G15** `ul`/`li`/`list`/`list-item`/`pre`/`code`/`strong`/`b` 全部静默
  降级 div/span（registry 的 vue 映射未生效；仅语义白名单 header/footer/
  nav/main/aside/article/section 透传）——用 div/span + class + CSS 适配，
  e2e 选择器同步。
- **G16** input 上 `value:` + 任意 `on*` 事件必转 v-model——绑 prop 值
  （`value: .modelValue`）时生成 `v-model="modelValue"` → Vue 编译错
  （prop 不可写）。绕法：本地镜像 var + `watch { .prop.immediate -> }` 同步。
- **G17** 普通名空 handler 是 no-op **不 emit**（`function Value(v) { // TODO }`）；
  自动 emit 仅对引号名（`."update:modelValue"(v) -> {}`）。
- **G18** handler 里 `.Msg(x)` 调用紧跟 `let y = ...` 语句会被吞成链式
  （`[].Value(kept)`）——emit 调用放在块首（if 块内首语句）。
- **G19** view 的 if 条件里不能有箭头闭包（`find(x => ...)`）或复杂索引
  （`.row[c.name]` 于条件位）——挪进 computed 或改写。
- **G20** `type: "number"` 里 `number` 与类型名碰撞（解析器以为是 type 块）
  ——引号键 `"type": "number"`。
- **G21** use 清单必须含 computed/handler 引用的全部 ext fn——漏了不在
  编译期报错，运行时 `ReferenceError`（e2e 抓）。

## 已知残留差异

- TableField 单元格 input 生成了 `v-model="row[c.name]"`（对 prop 数组的
  深写）——与 `$event` 重建路径并存，行为正确但违反 D5 字面约定（登记）。
- 表格空行的 colspan 丢省（原 `columns.length + 1`）；表格 select 列的
  "(current)" 回退 option 恒渲染（原为条件）——均无视觉差异。
- 01-ai-daemon / 02-auto-musk 两张截图与手写版有非布局性像素差（视觉
  检查无异常，e2e 全绿）。
