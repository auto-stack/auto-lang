---
plan_id: PLAN-615
status: executing              # 2026-09-12 work 进入：worktree .wt/lang-615/auto-lang @ plan-615-dev（base a4faeb182）
feature_name: calc 修复与增强——按钮居中/VM 布尔短路根修/双主题/Programmer HEX/交互批次
author: [zcode]
created_at: 2026-09-12
updated_at: 2026-09-12
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/vm: SD-01 VM 布尔短路求值语义（&&/|| 与转译后端一致的惰性求值契约）"
  - "docs/specs/auto-lang/ui: SD-02 主题传播链契约（OS→桌面 cfg.dark_theme→应用 dark_mode 播种缺省 + SetTheme 活更新广播）"
  - "docs/specs/auto-lang/ui: SD-03 按钮内容光学居中契约（无高度类按钮同样适用 CSS button 语义）"
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉一致（Vue/VM 双端 parity）——按钮居中/主题跟随/Programmer HEX 双端同源"

affects: [docs/specs/auto-lang/vm, docs/specs/auto-lang/ui]
current_step: 0
total_steps: 9
---

# [PLAN-615] calc 修复与增强——按钮居中 / VM 布尔短路根修 / 双主题 / Programmer HEX / 交互批次

## 0. 变更摘要

用户实测 `examples/ui/011-calculator`（VM 端）反馈四个问题，本计划一次收口：

1. **按钮文字纵向不居中（偏下）**——框架级 iced 渲染缺陷（W1）。
2. **等号结果不显示**（`2+9=` 后表达式行更新但结果区冻结）——**VM `&&`/`||` 无短路求值**为真因，
   Plan 550 T05（2026-09-05）把 GET_ELEM 越界翻转为 IndexError 是引爆点（W2，编译器/VM 根修）。
3. **深浅主题一个样**——应用 `dark_mode` 硬编码 true、播种仅启动期一次性、桌面/系统主题无传播链（W3）。
4. **Programmer 模式缺 HEX**——现仅有 `DEC ${display}` 只读行，无基底切换/位运算（W4，纯应用层）。

另附计算器增强建议一批（§增强清单），高价值低成本四项纳入本计划 P 批次：
键盘直输、浮点显示格式化、复制结果、错误文案细化；其余（历史面板/Memory 键/DEG-RAD/
word size 64 位等）登记 backlog 不在本计划。

## 1. 目标

1. **G1 按钮居中（W1）**：VM 端 calc 全部按钮（模式 Tab/数字/运算符/功能键）文字光学纵向居中，
   与 Vue 端（浏览器 CSS half-leading 模型）对拍一致；不回归既有布局 golden。
2. **G2 布尔短路根修（W2）**：VM `&&`/`||` 实现短路求值（RHS 仅在需要时求值），与全部转译后端
   （TS/JS、Python、C、Rust、GDScript 均原生短路）语义一致；calc `=` 恢复显示结果；
   Plan 550 的 IndexError 语义（合法负索引 -1 尾元素仍合法、越界报错）保持不变。
3. **G3 双主题（W3）**：calc 在深色与浅色主题下均正确渲染；无 per-app 显式配置时**缺省跟随
   桌面当前主题**（桌面缺省跟随 OS 系统主题，boot 期读取一次）；桌面 `set_theme` 切换后
   **运行中的应用活更新**；Vue 端同验。
4. **G4 Programmer HEX（W4）**：HEX/DEC/OCT/BIN 四基切换 + 随基变化的键盘（HEX 含 A-F）+
   四基实时换算读出 + 位运算六键（AND/OR/XOR/NOT/SHL/SHR），双端结果位精确一致。
5. **G5 交互批次（P）**：键盘直输、结果显示格式化（0.1+0.2→0.3）、复制结果、错误文案细化。

### 非目标（v1 明确不做，登记 backlog）

- 计算历史面板、Memory 键（MS/MR/M+/M−）、DEG/RAD 切换、log/ln/factorial 等更多科学函数。
- Programmer word size 64 位（QWORD）——**首版 32 位无符号值域**（双端 parity 决策，见 §详细设计）；
  64 位与 word-size 切换（QWORD/DWORD/WORD/BYTE）留后续。
- OS 主题运行时热监听（WM_SETTINGCHANGE）——首版仅 boot 期读取一次，运行中跟随桌面主题开关。
- 千分位分隔、in-app 主题手动切换按钮（跟随系统后无需求）、无障碍/触觉反馈。
- VM handler 运行时错误上屏的框架级机制（现仅 stderr 日志，`dynamic.rs:1141`）——登记 KNOWN-DEBT。

## 2. 架构方案

四问题分层处置，**一次 VM 语义根修 + 两处框架渲染/宿主小改 + 一块纯应用层特性**：

| 问题 | 层 | 策略 |
|---|---|---|
| W2 短路 | VM codegen（`vm/codegen.rs`） | 根修：`Op::And/Op::Or` 从急切 `emit(AND/OR)` 改为条件跳转短路发射；语料钉语义 |
| W1 居中 | UI 渲染（`ui/iced/renderer.rs` button 臂） | 框架级根修（Plan 414 已确立"CSS button 语义"方向，本计划补全无高度类按钮 + 行盒档位）；双端截图对拍定档 |
| W3 主题 | 宿主链（`ui/osconfig_apps.rs` / `ui/session.rs` / `ui/iced/renderer.rs` execute_set_theme） | 补传播链三环：播种缺省跟随桌面、SetTheme 活更新广播、OS boot 缺省；app 层零新逻辑（light 分支已全） |
| W4 HEX | 应用层（`examples/ui/011-calculator/src/front/app.at`） | 纯 .at：base 状态 + 随基键盘 + int 求值器（算术化实现保证双端位精确）+ 换算读出 |

**载体与仓范围**：单仓 auto-lang。示例资产在 `examples/ui/011-calculator`；框架改动在
`crates/auto-lang/src/{vm,ui}`；无需 auto-os 改动（桌面 settings 外观分区 Plan 487 已有，
`set_theme` 动词 Plan 518/540 已有）。

## 3. 技术栈

Rust（auto-lang crate：vm/ui 模块）、AutoLang .at（示例应用 + VM 语料）、
现有测试基建（vm_file_tests 语料架、autoui-verifier 双端脚本 `tests/test_011_vm.py` /
`tests/test_011_vue.mjs`、gallery golden）。零新依赖（OS 主题读取用现有 `windows` crate
registry API 或 std `reg` 命令等价物，随实现取最小面）。

## 4. 需求分析与背景调查

### 用户授权范围

- 诉求（2026-09-11 对话+截图）：①按钮文字纵向不居中偏下；②`2+9=` 结果不显示（用户疑 recent
  VM 改动回归——已证实，见下）；③系统深/浅主题下应用一个样，要求浅色支持（现套视为深色）；
  ④Programmer 模式缺 HEX，要求设计并加上。另授权：综合分析并纳入增强建议。
- 仓库/动作范围：auto-lang 单仓（crates + examples/ui/011-calculator + 语料 + 计划文档）。
  无预算/时长约束；标准 worktree 流程（`D:/autostack/.wt/lang-615/auto-lang`）。

### 根因调查结论（全部已用代码/实验证据钉死）

**W2 等号不显示——根因链闭合（实验复现）**：

| 环节 | 证据 |
|---|---|
| 症状 | `=` 后 `.expr_line` 已更新（"2+9 ="）、`.display` 冻结在 "2+9"、无 Error 红字 ⇒ handler 在 `eval_expr` 调用内 VM 运行时错误中止（错误仅落 stderr `[VM-HANDLER]` 日志） | `ui/dynamic.rs:1141`、用户截图 2 |
| 直接死因 | `IndexError: index -1 out of range`——`eval_expr` 的算符扫描守卫 `while ops.len() > 0 && ops[ops.len() - 1] != "(" ...` 在空栈时对 `ops[-1]` 求值 | 本计划调查期用 auto-vm CLI 复现（eval_expr 原文摘自 app.at，输入 "2+9" 即崩） |
| 真因 | **VM `&&`/`||` 急切求值**：codegen `Op::And => self.emit(OpCode::AND)`、`Op::Or => OR`，无条件发射单指令，LHS 假仍求值 RHS。探针矩阵实证：`if 1 > 2 && side(1)` 打印 SIDE-EFFECT；索引型 RHS 在 if/while 条件位同崩 | `vm/codegen.rs:7195`；探针（调查期临时件，已清理） |
| 为何"以前是好的" | Plan 550 T05（2026-09-05）之前 GET_ELEM 越界**静默 0 哨兵**：`ops[-1]` 返回哨兵 → `!= "("` 真 → `prec(哨兵)=0 >= 1` 假 → 守卫碰巧收敛正确。IndexError 翻转引爆了潜伏的短路缺失 | `git show bf826c9e0`（"负索引合法语义保留"指 Python 式 normalize 后合法，空栈 -1 仍越界）；`vm/engine.rs:5147 normalize_index` |
| 跨端佐证 | `&&`/`||` 在 TS(`&&`)/Python(`and`/`or`)/C/Rust/GDScript 转译端全部原生短路 ⇒ 唯 VM 端语义分歧；Vue 端 calc 一直正常 | `trans/emit.rs:54`、`trans/python.rs:155`、`trans/c.rs:1427`、`trans/rust.rs:3117`、`trans/gdscript.rs:77` |
| 修复可行性 | opcode 齐备：`JMP_IF_Z/JMP_IF_NZ/JMP/DUP/POP/POP_N`；发射形态 `eval a; DUP; JMP_IF_Z end; POP; eval b; end: [AND]`（|| 对称 JMP_IF_NZ），栈平衡、末端保留 AND/OR 做真值归一，语义面最小 | `vm/opcode.rs:11-12/174-176` |

**W1 按钮文字偏下——渲染路径定位**：calc 按钮（`p-4 text-lg`，无高度类）走
`ui/iced/renderer.rs:3361` button 臂。既有居中逻辑只覆盖**显式高度类**按钮
（Plan 409 §10 续 21 / Plan 414 `plan414_content_alignment`，renderer.rs:1386-1400）；
无高度类按钮内容直接 shrink，文字纵向位置由 **iced 文本行盒**决定——行高相对值（默认 >1.0）
的额外 leading 在 iced 行盒内分布不对称（字形成盒偏下），而 web 端 CSS half-leading 模型
上下均分 ⇒ 双端光学漂移 ~0.1-0.2em（text-lg 下 2-4px，与截图吻合）。修复两因素：
①按钮臂无高度类时同样包 `center_y` 内容容器（CSS button 默认语义补全）；
②行盒档位决策（按钮标签 `leading-none` 等价或半行距补偿）以双端截图对拍定档。
`leading-*` 类解析已存在（`ui/style/iced_adapter.rs:1169-1176`）可复用。

**W3 主题链现状**：VM 全局 `DARK_MODE` thread-local（`ui/style/theme/mod.rs:109`，缺省
true）+ `THEME_EPOCH` 失效回路已有；`dark:` variant 门控 Plan 527 已有。应用层
`dark_mode` state var 仅 **launch 期一次性播种**（`ui/osconfig_apps.rs:60 seed_app_config`，
来源=per-app os-config 文件，优先级 CLI > os-config > pac.at > 内置）；无显式配置时落在
app.at 硬编码 `var dark_mode bool = true`。桌面 `SetTheme(bool)` 执行臂
（`ui/iced/renderer.rs:9272 execute_set_theme`）只翻 `config.dark_theme` + 语义 token +
窗palette，**不广播到运行中应用的 dark_mode var**。OS 系统主题（Windows 个人化
AppsUseLightTheme）无任何读取点——桌面主题缺省硬编码 dark（Plan 408 注记）。
calc 的 light 分支样式已全量在写（app.at 每 style 均有 dark_mode 二元分支）——纯链路问题。

**W4 引擎输入盘点（零引擎改动可行）**：位运算 method 族全量在 VM——
`.and/.or/.xor/.not/.shl/.shr/.sar/.rol/.ror/.count_ones/.leading_zeros/.trailing_zeros/.bit_read/.bit_test`
（语料 `test/vm/02_bit_ops/` 五件）；`0b`/`0x` 字面量 lexer 支持（`lexer.rs:288-330`）；
`uint.to_hex(pad)` native 在册（catalog 236，`vm/native.rs:5988`，u64→零填充 hex 串）。
OCT/BIN 格式化无现成 native——首版在 .at 侧 divmod 算法实现（避免动 native 面），
若 i64 div/mod VM 支持受阻再退 `uint.to_radix` native（小决策点，T-07 内裁定）。

**双端 parity 关键约束（W4 设计核心）**：TS 端位运算符是 32 位语义（`<<`/`&` 截断），
VM 端 method 族是 64 位——**首版裁定 32 位无符号值域 [0, 2³²)**：SHL 结果 `mod 2³²` 补齐、
NOT = `0xFFFFFFFF ^ x`、输入解析与显示同域 ⇒ 双端位精确。64 位值域留 backlog（需 BigInt
面，跨端成本另立项）。

### 规约/文档现状

- `docs/specs/auto-lang/vm`：布尔逻辑语义无短路条目（现状=急切，属语义缺陷非约定）——SD-01 立新。
- `docs/specs/auto-lang/ui/overview.md`：504/506/527 条目覆盖 fit 窗口/播种链/`dark:` 门控，
  主题传播链与应用 `dark_mode` 缺省语义无契约——SD-02 立新；按钮内容对齐仅 Plan 414 注记
  （高度类按钮），无高度类语义缺位——SD-03 补全。
- calc 无独立 spec 组件（examples 资产），W4 设计在本计划 §详细设计承载。

## 5. 详细设计

### W2：`&&`/`||` 短路发射（T-01/T-02）

`crates/auto-lang/src/vm/codegen.rs` 二元表达式臂（`Op::And => emit(AND)` 处，~7195）：

```
a && b:  [eval a] DUP JMP_IF_Z Lend POP [eval b] Lend AND
a || b:  [eval a] DUP JMP_IF_NZ Lend POP [eval b] Lend OR
```

- 末端保留 `AND/OR`：短路时栈顶=LHS 原值，走 AND/OR 真值归一，与现行为（双值归一）结果
  一致，truthiness 语义单点不变；`JMP_IF_Z/NZ` 是否弹栈按 opcode 实测适配（不弹则 DUP/POP
  平衡式如上；弹则去 DUP）。
- 链式 `a && b && c` 左结合自然嵌套成立；条件上下文（if/while）与值上下文（赋值/实参）
  共用同一表达式臂，语义统一。
- **语义裁定**：结果恒为归一化 bool（非 JS 的"返回原操作数"）——.at 中 `&&`/`||` 操作数
  均为 bool 位语境，与转译端在 bool 域结果一致；语料钉死。
- 兼容扫描：存量语料/示例 grep `&&`/`||`，确认无"依赖 RHS 急切求值副作用"的病态用法
  （预期零命中；有则逐个裁定并记录）。
- Plan 550 IndexError 语义正交保持：短路与越界翻转互不回退——`ops.len() > 0 && ops[...]`
  短路后不再触碰 `ops[-1]`；合法负索引（-1 尾元素）语料保持绿。

### W1：按钮内容光学居中（T-03）

`ui/iced/renderer.rs` button 臂（~3660 起的包装段）：

1. 无高度类按钮与高度类按钮同规则：内容包 `container().height(Fill).align_y(Center)`
   （有宽度类时同 Plan 414 水平分支），消除"iced button 内容顶对齐"残余。
2. 行盒档位（决策点，双端截图对拍定档，默认取 a）：
   - a) 按钮标签 text 臂显式 `line_height(Relative(1.0))`（leading-none 等价）：行盒贴合
     字形，光学居中恢复；按钮 shrink 高度收窄 2-4px，由对拍判定是否可接受；
   - b) 若 a 引发不可接受的既有布局漂移：保持行盒，改半行距补偿容器（padding 对冲）。
3. 影响面：全部 VM 应用按钮——受影响 golden/截图基线在 T-09 出重基线清单
   （预期集中在按钮 shrink 高度场景）；纯值语义零变化。
4. 联动验证：calc `window: "fit"` 高度链（Plan 504）——按钮高度变化后 fit 重测，顺带核验
   用户截图所见**末行（`)` `⌫`）被窗口下缘裁切**问题是否同源消除；若仍裁切，在本计划内
   修 fit 测量（P504 链缺陷单列）。

### W3：主题传播链三环（T-04/T-05/T-06）

1. **播种缺省环（T-04）**：`seed_app_config` 调用链上，per-app/pac 均未显式配置 theme 时，
   `dark_mode` 缺省值取 `session.desktop.config.dark_theme`（桌面当前主题）——新增
   "缺省跟随"臂（声明了 `dark_mode` var 才写，语义同现有播种守卫）。calc 的
   `var dark_mode bool = true` 降级为最后兜底。**calc `pac.at` 删除 `theme: "dark"` 显式键**
   （否则按优先级压过系统跟随）——Vue 端同文件消费，双端一致改为跟随。
2. **活更新环（T-05）**：`execute_set_theme`（renderer.rs:9272）翻 `config.dark_theme` 后，
   遍历本会话 live DynamicComponents，对声明了 `dark_mode` 的组件 `write_state(Bool)` +
   触发该组件视图失效（复用 THEME_EPOCH/组件 dirty 既有回路；调查点：多窗会话的组件枚举
   入口，预计在 DesktopSession 持有面）。桌面设置面板切主题 → 全部运行中应用同帧换装。
3. **OS 缺省环（T-06）**：boot 期桌面主题缺省值从 OS 读取——Windows 读注册表
   `HKCU\...\ThemeManager` 个人化 `AppsUseLightTheme`（DWORD，0=dark）；非 Windows/读取
   失败回退 dark。实现位：desktop config 初始化处（cfg 缺省构造点），单点小改。
4. **Vue 端 parity（T-04 内调查点）**：vue 运行时对 `dark_mode` 的播种/`prefers-color-scheme`
   跟随落点（a2ts 产物模板/runtime）——若运行时无系统主题通道，首版 vue 侧跟随 pac/缺省，
   债务登记（不阻塞：主诉求是 VM 桌面端）。

### W4：Programmer 模式 HEX（T-07，纯 app.at）

`examples/ui/011-calculator/src/front/app.at`（双端同源，零引擎/Rust 改动）：

- **model 增**：`var base str = "DEC"`（HEX|DEC|OCT|BIN）；programmer 整数值内部以
  `str`/`int` 承载（32 位无符号域 [0, 2³²)）。
- **view 增**（`mode == "programmer"` 分支重写）：
  - 基底切换行：HEX/DEC/OCT/BIN 四键 seg control（当前基 orange 高亮，同模式 Tab 语言）；
  - 换算读出区：当前基大字显示，其余三基小字 mono 只读行（Windows calc 布局语言，
    现有 `DEC ${.display}` 行升级）；每行可点击=切到该基；
  - 键盘随基变化：HEX 增 A-F 六键；BIN 仅 0/1（其余灰/隐藏）；OCT 0-7；`.` `%` `(` `)` 
    sci 行在 programmer 隐藏；位运算六键区（AND/OR/XOR/NOT/SHL/SHR）独立块；
  - 浅色分支全套（跟随 W3 语义，样式二元分支补全）。
- **int 求值器**（与 float `eval_expr` 并列，programmer 专用）：逐字符 radix 解析 +
  四则整数运算（32 位 wrap：`mod 2³²` 归一）；位运算语义算术化/方法化混合，**双端位精确
  约束**：AND/OR/XOR 用 `.and/.or/.xor` method（TS 端 `&`/`|`/`^` 32 位恰好同域一致）、
  NOT = `0xFFFFFFFF ^ x`（xor method）、SHL = `(x.shl(k)).mod(2³²)` 算术补齐、SHR = 
  `x.div(2^k)` 算术化（规避 TS `>>` 符号位差异）——六运算双端断言钉死。
- **OCT/BIN 显示**：.at 侧 divmod 迭代格式化（避免新增 native；i64 div/mod 可用性在
  T-07 first-light 验证，受阻则登记并退 `uint.to_radix` native 微增面）。
- **输入合法性**：非当前基数字键禁用（灰态，disabled 或过滤双端取一）。

### P 批次（T-08，均 app.at/双端机制内）

1. **键盘直输**：`bind` 段扩 `0-9` `.` `(` `)` `+ - * / =` `%` `^`（+programmer `A-F`）——
   bind 是双端机制（现有 Enter/Escape/Backspace 同段）。
2. **显示格式化**：`fmt_num` 增 ε 相对容差去噪（0.1+0.2→0.3）+ 超大/超小指数格式——
   .at 内实现，双端同源。
3. **复制结果**：显示区点击或快捷键 → 剪贴板（VM `auto.clipboard` natives Plan 485 在册；
   vue `navigator.clipboard`——双端 API 形态差异在 app 内分支，T-08 调查点）。
4. **错误文案**：`eval_expr` 错误通道细化（除零 → "Cannot divide by zero"、语法 → 
   "Invalid expression"），红色错误条复用现有 `.error` 面。

### 规范增量

| delta_id | 类型 | 目标 | before/after | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/vm（语义/overview 布尔逻辑条目） | before: VM `&&`/`||` 急切求值（未成文）；after: 短路求值——`&&` LHS falsy 不求值 RHS，`\|\|` LHS truthy 不求值 RHS，结果恒归一化 bool，与转译后端 bool 域一致 | 跨端语义分歧根修；副作用/索引守卫语义对齐 | AC-2 |
| SD-02 | add | docs/specs/auto-lang/ui（theme 节） | before: 应用 dark_mode 仅 launch 期显式配置播种；after: 传播链 OS（boot 一次）→ 桌面 cfg.dark_theme → 应用 dark_mode（无显式配置时播种缺省跟随桌面 + SetTheme 活更新广播到声明 dark_mode 的运行中应用） | 主题跟随是桌面应用一等语义；现链路断裂 | AC-3 |
| SD-03 | modify | docs/specs/auto-lang/ui（按钮渲染契约，Plan 414 条目扩全） | before: 显式高度类按钮内容双向居中；after: 按钮内容默认双向光学居中（CSS button 语义），无高度类按钮同样适用；行盒档位注记 | 偏下根因是语义缺位非 bug 单点；契约钉住防回退 | AC-1 |

## 6. 测试设计

1. **VM 语料（新增，入 `cargo tv`）**：
   - `test/vm/99_short_circuit/`：RHS 副作用计数探针（`&&`/`||` × if/while/值上下文 ×
     链式嵌套）、索引守卫（空栈 `ops[-1]` 守卫不再崩）、合法负索引 -1 尾元素共存、
     归一化 bool 结果断言；
   - `test/vm/99_misc/`（或独立件）：calc `eval_expr` 全文转正语料——"2+9"→"11"、
     "9+2"→"11"、"2*(3+4)"→"14"、"7/2"→"3.5"、除零→"ERROR"。
   - 兼容扫描报告（T-02 证据）：存量 `&&`/`||` 使用点清单与急切依赖排查结论。
2. **框架单测**：codegen 短路发射的 disassembly 断言（JMP_IF_Z 存在、RHS 调用被跳过）；
   renderer 按钮包装单测（无高度类 → center_y 容器）；seed_app_config 缺省跟随桌面臂单测
   （沿用 `osconfig_apps.rs` tests 既有范式）；execute_set_theme 广播后组件 state 断言。
3. **双端 E2E（autoui-verifier 基建扩展）**：`tests/test_011_vm.py` + `tests/test_011_vue.mjs`
   增断言——等号结果显示、主题浅色渲染（背景色/前景色采样）、programmer 四基换算读出、
   位运算结果、键盘直输、按钮文字光学居中（截图目验 + 像素采样）。
4. **门禁**：日常 `cargo check -p auto-lang` + `cargo t`（scoped：vm/ui 模块）；W2 属 VM
   改动 → **review/fold 前 `cargo tv` 全绿 + `cargo tf` 一档**；按 Plan 568 触发表，
   `codegen.rs`/`renderer.rs` 不在 aavm 触发清单（不动 `auto/lib/*.at`、`test/vm/aavm2/**`、
   `parity/**`）→ **`taa` 零触发**；若执行期触碰共享上游，按惯例 fold 前裸 `taa` 兜底并留痕。

## 7. 验收标准

| ID | 验收内容 | 验证方法 | 期望结果 |
|---|---|---|---|
| AC-1 | calc VM 端全部按钮文字光学纵向居中，与 Vue 端对拍一致 | autoui-verifier 双端截图 + 像素采样（按钮盒中心 vs 文字盒中心偏差 ≤1px） | 偏下现象消除；无既有布局 golden 无故回归（重基线清单在案） |
| AC-2 | VM 端 `2+9` `=` 显示 11；短路语义修正 | 实机操作截图 + 新语料全绿 + `cargo tv` | eval 语料 11/11/14/3.5/ERROR 全过；短路探针 RHS 零副作用；tv 零新增红；IndexError 语义保持（-1 尾元素合法语料绿） |
| AC-3 | 深浅双主题正确渲染 + 跟随 | 浅色主题实机截图（显示区/keypad/Tab 全套 light 分支）；set_theme 切换后运行中 calc 同帧跟随；OS 注册表 mock/实测 boot 缺省 | 无硬编码 dark 残留；无显式配置时缺省=桌面当前主题；桌面切换活更新 |
| AC-4 | Programmer HEX 全功能 | 双端交互脚本：四基切换、A-F 输入（非法键禁用）、四基读出联动、位运算六键（0xFF AND 0x0F=0x0F、1 SHL 31=0x80000000、NOT 0=0xFFFFFFFF 等） | 双端结果位精确一致（32 位无符号域）；渲染布局双端截图对拍 |
| AC-5 | 交互批次 | 键盘直输脚本（数字/运算符/Enter/Esc/Backspace）、0.1+0.2 显示 0.3 断言、复制后剪贴板读回、除零错误文案 | 全部按 §详细设计 P 批次行为 |
| AC-6 | 门禁健康 | `cargo t` / `cargo tv` 零新增红；review 前 `cargo tf` 一档；零新增编译警告；重基线清单 | 记录在执行收据 |

## 8. 执行步骤

worktree：`git worktree add D:/autostack/.wt/lang-615/auto-lang -b plan-615-dev`
（主检出先 commit `.next-id` + 本计划骨架）；计划簿记留在主检出。

| # | 任务 | 内容与落点 | 验证 | AC |
|---|---|---|---|---|
| T-01 | W2 短路发射 | `vm/codegen.rs` `Op::And/Op::Or` 臂改条件跳转短路形态（§详细设计），disassembly 断言 | `cargo check -p auto-lang` + `cargo t vm`（codegen 相关 filter）+ 手动 auto-vm 探针 | AC-2 |
| T-02 | 短路语料 + calc eval 转正 + 兼容扫描 | 新增 `test/vm/99_short_circuit/` 语料族；calc eval_expr 全文语料（期望 11/11/14/3.5/ERROR）；存量 `&&`/`||` 使用点扫描报告 | `cargo tv` 全绿；扫描零急切依赖结论留档 | AC-2 |
| T-03 | W1 按钮居中 | `ui/iced/renderer.rs` button 臂：无高度类 center_y 包装 + 行盒档位决策（双端截图对拍定档）；fit 末行裁切联验 | renderer 单测 + autoui-verifier calc 双端截图 | AC-1 |
| T-04 | W3a 播种缺省跟随 | `ui/osconfig_apps.rs`/`session.rs` 播种链增"无显式配置→跟随桌面 cfg.dark_theme"臂；calc `pac.at` 删 `theme: "dark"`；vue 端主题落点调查（bounded，结论入计划） | osconfig 单测 + VM 窗浅色实机截图 | AC-3 |
| T-05 | W3b SetTheme 活更新 | `ui/iced/renderer.rs execute_set_theme` 广播臂：遍历 live 组件写 `dark_mode` + 视图失效 | 单测（组件 state 断言）+ 实机切主题观察 | AC-3 |
| T-06 | W3c OS boot 缺省 | desktop config 初始化读 Windows AppsUseLightTheme（非 Windows 回退 dark），单点 | 单测（registry mock/回退臂）+ 实机 | AC-3 |
| T-07 | W4 Programmer HEX | app.at 重写 programmer 分支（§详细设计全量：seg/读出/随基键盘/位运算/int 求值器/OCT-BIN 格式化）；i64 divmod first-light 先行 | 双端交互脚本 + 位运算断言表 | AC-4 |
| T-08 | P 批次 | bind 扩展/fmt_num 去噪/clipboard/错误文案（§详细设计 P1-P4） | 交互脚本 + 显示断言 | AC-5 |
| T-09 | 终验与门禁 | 全量双端 autoui-verifier（calc 全功能矩阵）；受 W1 影响 golden 重基线清单；`cargo tv` + `cargo tf`；复审材料 | AC-6 门禁表 | AC-6 |

依赖：T-02←T-01；T-05←T-04；T-07←T-01（programmer int 求值器内部守卫依赖短路）；
T-08←T-07（programmer 键位）；T-09 收尾全部。

## 9. 复审记录

- 2026-09-12 draft/revision handoff：`stage: new`，PLAN-615 rev 1。`outcome: pass`——
  四问题根因全部实证（W2 有 VM 实机复现与代码证据链；W1/W3 有渲染路径与链路缺口定位；
  W4 引擎输入盘点齐备），任务/AC/SD 对齐，路径与命令已落地核验。`next: work`。
  授权范围：auto-lang 单仓、标准 worktree 流程、无预算约束（§4）。

## 10. 待澄清事项

1. **W1 行盒档位**：默认取 leading-none 等价（按钮 shrink 高度收窄 2-4px）；若对拍判定
   既有布局漂移不可接受，回退半行距补偿方案（T-03 内决策，不影响契约）。
2. **W1 golden 重基线**：按钮渲染框架级修复预期影响少量既有截图基线——默认接受重基线
   （清单在 T-09 留档）；若出现大面积漂移（>10 用例），升级为用户决策点。
3. **Vue 端系统主题**：若 vue 运行时无 `prefers-color-scheme` 通道，首版 vue 跟随 pac/缺省
   并登记债务（VM 桌面端为主诉求，不阻塞 AC-3 的 VM 部分）。
4. **W4 值域**：首版 32 位无符号（双端 parity 硬约束）；64 位/word-size 切换入 backlog，
   若用户期望 64 位需另立 BigInt 面计划。
5. **OS 主题读取面**：仅 Windows 实测；Linux/mac 回退 dark（登记，跨平台热监听 backlog）。

---

## 附：增强建议清单（综合分析结论，用户问"还有哪些建议"）

**已纳入本计划（P 批次）**：键盘直输、浮点显示去噪、复制结果、错误文案细化。

**登记 backlog（后续计划候选，按价值排序）**：

1. 计算历史面板（最近 N 条表达式/结果，点击回填）——桌面应用一等体验。
2. Memory 键组（MS/MR/M+/M−/MC）。
3. 科学模式 DEG/RAD 切换 + log/ln/factorial/1/x。
4. Programmer word size（QWORD/DWORD/WORD/BYTE）与 64 位值域（BigInt 面）、
   循环移位 ROL/ROR 键、位直接点选面板（bit kite）。
5. DEC 千分位分隔显示；括号配对高亮（表达式行）。
6. 窗口可调尺寸/最大化布局自适应（现 fit 固定内容尺寸）。
7. VM handler 运行时错误上屏机制（框架级，惠及全部应用——本计划 W2 的症状面）。
