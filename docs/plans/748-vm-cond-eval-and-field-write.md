---
plan_id: PLAN-748
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: VM 三断点修复 + vm 轨 iced 渲染器四件缺陷（musk PLAN-101 + jade-edit 供料，双 phase）
author: [agent]
created_at: 2026-10-10
updated_at: 2026-10-10

# /auto-plan:review 结束时填写（R1 二轮定稿）：
supersedes_spec_components:
  - docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md#已知边界
new_spec_components:
  - docs/specs/auto-lang/ui/overview.md#overflow-滚动转写-视口视觉层（PLAN-748）
  - docs/specs/auto-lang/ui/overview.md#autodown-editor-引擎-parity（PLAN-748）
touched_goals:
  - GOAL-007

affects: [auto-lang/vm, auto-lang/ui]
current_step: 12
total_steps: 12
plan_revision: 4
---

# PLAN-748 — VM 三断点修复 + vm 轨 iced 渲染器四件缺陷（双 phase）

## 变更摘要

本计划双 phase，各自独立可执行/可复审，同 worktree 按序执行；两 phase
**无代码依赖**（Phase 2 不依赖 Phase 1 三断点修复，可先行）。

### Phase 1 — VM 模板条件求值/续体字段写/动态样式三断点（auto-musk PLAN-101 供料，T-00～T-05）

auto-musk PLAN-101（2026-10-09 实机）登记了三个 VM 渲染/执行引擎断点并
以 workaround 绕开（musk `docs/plans/101` §10），本 phase 在 auto-lang 侧
根修：

1. **模板 `if` 条件成员访问恒假**——`for` 循环行数据的 `if w.is_current`
   不渲染（裸 binding `if .collapsed` 正常；成员读在 text/attr 位正常）。
2. **handler 续体静态名字段写崩溃**——await/park-resume 后续体中对 JSON
   局部对象 `item.x = v` → `RuntimeError("execution terminated unexpectedly")`
   （段名挂外层 handler）；动态键 `item["x"] = v` 与同 receiver 正常。
3. **动态 style 字段引用半支持**——`style: .w.row_style` 在部分元素臂被
   静态提取丢弃（`extract_style` 只认字面量），绑定感知通道
   `extract_style_with` 未全元素收口。

修复后 musk 101 的绕开代码（mark 预拼 / JSON 克隆+动态键 / 静态样式串）
**回退为直读条件并双端对拍通过**（101 §10 收口候选）。

### Phase 2 — vm 轨 iced 渲染器四件缺陷（jade-edit 供料，T-06～T-11；2026-10-10 增补，plan_revision 2）

jade-edit 供料包（jade-edit `docs/upstream/2026-10-09-vm-layout-stretch-supply.md`
，证据截图 jade `e2e/.runtime/exp5..13*.png`，exp5=健康基线对照）登记
四个「vue 轨 CSS 渲染正常、vm 轨 iced 解释渲染破」的双轨差异，锚点在
iced 渲染器与 autodown_editor 官方件：

1. **件一（布局）**：`items-stretch` 行不交叉轴拉伸 Shrink 高子项——
   StretchLine final 遍声称对 Shrink 高子项以 `min_h=effective` 落位
   （stretch_line.rs 头注「CSS align-items:stretch 语义原语化」），
   实际未兑现——子项几何/背景止于自身内容高，行本身 flex-1 满高正常
   （jade 实测侧栏 bg 底边 550/786，行满高）。
2. **件二（布局）**：`overflow-y-auto` col（view builder 转写
   `View::Scrollable`）挂显式高 `h-full` → 子树零几何（MCP snapshot
   节点在、屏幕零像素），且**类序敏感**：h-full 类串首位=整个应用降级
   为无样式巨字布局、末位=仅本子树消失。同位 `flex-1`（经
   axis_fix_col_child 转写 Height(Full)）正常——显式类与转写产物在
   Scrollable 上行为分叉是定位线索。类序不应影响渲染结果。
3. **件三（布局）**：StretchLine 主轴配给挤掉末位定宽子项——
   items-stretch 行三直接子 `[定宽A, FillPortion B, 定宽C]`：B 越界
   占满整行，C 配到≈0 宽，行 overflow-hidden 下完全不可见（jade 右栏
   检索/反链/标签面板开即不可见：backlinks_open=true、面板行在快照、
   零渲染）。与 C 宽度类形态（w-72 与 w-[288px] 同败）、与 overflow/
   显式高/if 包装均无关。期望 B = 行宽 − A − C（flex third-pass 定宽
   预留）。
4. **件四（呈现缺口）**：`autodown_editor` vm 件缺引擎 wrapper padding
   （auto-down/engine 边界）——vue 轨 padding 由引擎 CSS 承担
   （autodown-editor.css:103 `.autodown-editor-content-wrapper
   padding:1rem 1.25rem`，editor/index.ts:1 随组件导入）；vm 轨官方件
   无此层，正文贴死分隔线。正根修：vm 件内建复刻该 padding。

验收锚点：jade-edit **撤 workaround 层后**（侧栏回单层 overflow-y-auto
col、编辑器去 padding 容器——jade commit 4b398a4 两层 workaround）双轨
逐像素同构 + 右栏面板可见 + 现有全部门绿。

## 目标

1. 复现并定音三个断点的真实根因（master 代码上三处"看起来都有臂"，
   不许在未定音前打兜底补丁——renderer.rs:16457 "静默塌缩恒 false"
   历史教训）。
2. 断点①：模板条件求值对循环行数据成员访问、`.store.X`/computed 混合
   形态在**根件与子件两上下文**均返回真值（子件面嫌疑最大：
   P733-R2 预存红族同族）。
3. 断点②：handler 续体内静态名/动态键/循环绑定写三形态与无 await 对照
   组行为一致（成功或响亮的具体 Err），不再 Terminated。
4. 断点③：`style:` 动态字段引用全元素产出（静态臂收口
   `extract_style_with`），词表过滤至少保留 button 臂同款 WARN。
5. 跨仓回退对拍：musk PLAN-101 worktree（5b2fa9f）的绕开代码回退版在
   本计划构建上 probe 全绿；musk 侧 ui-parity 无新增红。
6. 既有回归零新增红（tv/tf/RC + daily 对拍；**master 预存红 P733-R2 六条
   为零增减基线，不是本计划修复对象**）。

### Phase 2 目标（jade-edit 供料四件）

7. 件一/二/三在干净 master 基面复现定音（供料观测来自 release
   v0.4.2-2747-gab7a650bd 构建——stretch 代码与现 HEAD 一致，仍须干净
   基面重取；件四为确证呈现缺口，无需定音直接修）。
8. 件一：items-stretch 行内 Shrink 高子项（含 overflow-y-auto col）的
   几何/背景拉伸到行高（复现形左 col 高 = 行高）。
9. 件二：overflow-y-auto col 挂显式高（h-full/h-*）子树几何非零，且
   **任意类序渲染等价**（首位/末位/乱序 golden 对照全同）。
10. 件三：`[定宽, flex-1, 定宽]` 行 flex 子宽 = 行宽 − Σ定宽 − spacing，
    定宽子不塌 0 宽（右栏面板可见）。
11. 件四：vm autodown_editor 内建引擎等值 wrapper padding（1rem/
    1.25rem = 上下 16 / 左右 20 逻辑 px）。

### 非目标

- 不改 a2vue/web 生成器面（style "四形态"是 web 轨语义登记，VM 解释器
  通道归本计划，生成器不动）。
- 不修 P733-R2 预存红本体（子件 override `.store.X` 深层链如经 T-00
  定音与断点①同根则顺带修复并在 §9 说明；否则登记边界）。
- 不处理 auto-lang 主检出上的他人 WIP（媒体引擎 8 文件，见 §4 风险）——
  本计划 worktree 基面恒钉 master 已提交代码。
- 不动 park/resume 协议本身（Stakes/RC 敏感区，419/510/624/733 四轮
  修过；修复落在错误处理/求值侧，非弹栈侧）。
- **不处理 PLAN-100 三型读语义病灶**（信封 bool 位型读 / handler 深链读
  MISS / 视图模板 fn 调用返空坑①家族）——已立 **PLAN-749** 根修
  （2026-10-10 分流裁定，749 §4 边界判定）。症状3（视图 CALL 返空）与
  本计划断点①已证异根：①走 `resolve_binding_path` 臂（binding 挂载面），
  749 症状3 走裸名兜底臂（`eval_condition_with_inner` 无 CALL 通道）——
  同函数不同臂，修复面不重叠。两计划严格串行（同碰 engine.rs/
  aura_view_builder.rs），顺序见待澄清#6。

### Phase 2 非目标

- 不改 vue/web 轨与 auto-down 引擎 CSS（件四 padding 数值锚在引擎侧
  autodown-editor.css，vue 轨渲染链零改动）。
- 不在 jade-edit 仓内修任何代码（上游根修，jade 侧只做撤层复验——
  669/682 先例流程；复验依赖 jade 会话协调，见 §10）。
- 不推倒 StretchLine 两阶段架构本体与 line_fill_height 语义（修复落在
  final 遍落位载体、配给数学与 Scrollable 高解析，PLAN-655 语义保持）。
- 不做 iced 引擎级 flex 补丁/升级（PLAN-655 非目标第 5 条裁定延续：
  iced `Length::Shrink` 忽略 limits.min 等引擎级行为在渲染器层补偿）。

## 架构方案

```text
T-00 定音探针（干净 master worktree 重建 auto.exe——musk 实机用的是
     dirty 构建 v0.4.2-2730-g91ae002d3-dirty，主检出另有媒体 WIP，
     全部观测必须用干净基面重取）
  ├─ ① 三形态×两上下文条件矩阵（w.is_current / .store.X==w.id /
  │    computed+bare）×（根件/子件挂载），headless 快照 + VmBridge
  │    new_from_decls 双通道，先定罪到求值器臂/子件挂载/别名快照哪环
  ├─ ② resume FAILED 补 crash-ip 反汇编窗（镜像 vm_bridge.rs:2922
  │    首派形态，补 :1829 resume 路径）+ AUTO_VM_TRACE_OPS 末 50 指令
  │    + 非续体对照组 → 定音 intercept_error 残余 vs 帧 bp 协议
  └─ ③ style：静态臂→extract_style_with 收口（无条件腿，与①②并行）
T-01 断点②修复（按定音：intercept_error/engine.rs:11251 面或
     handler_codegen catch 目标面，红相 spike 先行）
T-02 断点①修复（按定音：eval_computed/store 别名展平挂载面或
     eval_condition_with_inner 补臂，红相矩阵先行）
T-03 断点③收口 + 词表 WARN 对齐
T-04 跨仓回退对拍（musk 101 workaround 回退版 probe + ui-parity）
T-05 全量门 + 提交 + 交接

Phase 2（jade 供料四件，与 Phase 1 无代码依赖，可先行）：
T-06 复现定音（干净 master 基面 + 供料复现形四件 headless/运行态探针）
  ├─ 件一 probe：StretchLine final 遍逐子项入射 limits 与产出 node 尺寸
  │  dump——min_h=effective 死于哪个包装层（iced Length::Shrink resolve
  │  忽略 limits.min 嫌疑；对照 008 同构行 layout_tests.rs:2939 差异定位）
  ├─ 件二 probe：同视图类序三变体（h-full 首位/末位/flex-1 替代）三层树
  │  diff（Style::parse → AbstractView → widget Length）——顺序敏感层
  │  定位 + 首位全局降级机制（全局副作用/静默 panic 定罪）
  ├─ 件三 probe：宽度探测遍逐子项 fill_portion 判定值 + available 扣减
  │  序列 + row 子项包装链外层 Length dump（flex-1 col 的 width=Full
  │  载体是否在包装链上丢失）
  └─ 件四：确证缺口（widget.rs 全模块零 padding 载体 vs 引擎 css:103）
     直接进修复
T-07 件一修复（交叉轴拉伸兑现；显式定高落位候选，不引入 Fill 包装回
  无界语境——P642-D12 红线）
T-08 件二修复（build_scrollable 显式高子树零几何 + 类序无关性收口）
T-09 件三修复（主轴配给定宽预留；build_row 组装期显式传入子项主轴语义
  三元组候选，摆脱 widget 包装层 Length 反推）
T-10 件四修复（autodown_editor vm 件内建 wrapper padding 16/20px）
T-11 判绿收口（iced 布局单测 + 乱序等价 golden + 041/gallery 像素对照
  + jade 撤层跨仓复验 + 全量门 + KNOWN-DEBT 更新）
```

## 需求分析与背景调查

### 授权

用户 2026-10-10 指示"起草修复计划吧"（PLAN-101 §10 三断点的 auto-lang
侧根修）。本阶段只起草；work 授权待用户确认计划后另行给出。

### 已取证事实（PLAN-101 实机，2026-10-09，auto.exe dirty 构建）

| 断点 | 症状 | 对照正常形态 |
|---|---|---|
| ① if 成员访问 | `if w.is_current` 恒不渲染（字面 id 重标后仍假） | `if .collapsed`（裸 binding）、`text .w.name`、`title: .w.path` |
| ② 静态名写 | 续体 `item.x=v` → Terminated（外层段名） | `item["x"]=v` 同 receiver 同续体通过 |
| ③ 动态 style | `style: .w.row_style` 不产出 | 静态串、`if 裸binding{全静态串}` |

### 代码定位（master @ 7bd28888b，调查代理只读实读）

基线 hash（SHA256 前 16）：engine.rs `93a73e6d2df5983e`、
aura_view_builder.rs `f986554d393ad093`、vm_bridge.rs `54487aad1ffb5191`、
vm/codegen.rs `139f1baea40fa564`、handler_codegen.rs `408566622fe0d7fb`。

- **断点①链路**：parser.rs:16390/16532 条件逐字保留（成员链显式拼点）
  → aura/extract.rs:1127 → `AuraNode::Conditional`；求值唯一权威
  `aura_view_builder.rs:12828 eval_condition_with`（iced renderer 不参与；
  vnode_converter 只是序列化）。成员访问臂 :12941→:12943
  `resolve_binding_path`、RHS 成员 :13048、`.store.X` :12996→
  `resolve_expr_to_value` Dot 臂 :12357（`Ident("store")` 特判）、computed
  兜底 :12930→`eval_computed` :12262。**单测
  `test_eval_condition_with_bindings`（:19019，含 `.filter=="active" &&
  todo.done==false` 复合形态）master 实跑通过**——求值器"看起来有臂"。
  `.store.X` 展平依赖 handler_codegen.rs:184 线程级名快照 + bridge
  每组件存档（PLAN-642）双通道；快照空时静默落空。
  **嫌疑排序**：(a) 子件渲染上下文 computed/store 面断裂（P733-R2 预存红
  族 `musk_vm_track_p053_1_widget_computed::*` master 实跑 FAILED 同族）；
  (b) 时序/数据面（101 自身竞争已另行修复）；(c) 求值器真缺臂（低）。
- **断点②**：错误唯一抛出点 `engine.rs:2794`（`drive_handler_segment`
  内 `StepResult::Terminated`；**fn_name 是段名非死点函数**）。SET_FIELD
  实现 :6566-6808 **无任何路径返回 Terminated**（成功或具体 Err）——
  真实错误被 try 拦截后 catch_pc 坏跳 / 帧弹穿 / ip 出界，与 PLAN-705
  已修族（"深帧 try 经 park/resume 后 catch 不生效→ip 出界"，
  engine.rs:11273-11286）同症状族。SET_FIELD（:6566）与 SET_ELEM（:6406）
  引擎臂同构（弹 3 nv + stake 结算），codegen 层 :7129-7396（Dot→
  SET_FIELD / Index→SET_ELEM；静态路径多 `SET_GENERIC_FIELD` 推断分支
  :7255）。字符串池 pinned，resume 后失效假设可排除。嫌疑：静态写路径
  receiver 求值序列与续体栈布局交互 → SET_FIELD 返回 Err → 续体+嵌套
  帧+try 组合触发 705 族残余。
- **断点③**：`style:` prop 仅对象字面量形态走 `StyleBinding`（parser
  :15864）；`extract_style`/`extract_string` 只认 `Expr::Str` 字面量，
  动态 Expr 静默 None——仍有静态臂的转换器：tabs/center/video/
  progress/spacer/checkbox（:1893/:3307/:8118/:8265/:8326/:11950）；
  `extract_style_with`（:14080, Plan 057）已接 row/col/scroll/container/
  input/textarea，**button/div/span/command 已是绑定感知**（:10901/
  :10678）——故 musk"style 不产出"证据同样需干净构建复验。第二丢弃面：
  `Style::parse` 词表过滤静默丢（button 臂有 446-U7 WARN :10910，其余臂
  连 WARN 都无）。
- **733 先例**（archive/733，三腿修复 0b5c8758d/481bcb0b9）：调用返空族；
  验证=矩阵单测 `plan733_fn_call_semantics_tests` + daily 回归与 master
  基线逐名对拍（预存红零增减）+ tv 全量 + 跨仓 probe-g15.mjs（musk 零
  改动）。本计划照搬其"先 bounded 调查定音再动刀"流程。
- **预存红基线（master）**：P733-R2 六条（`musk_vm_track_p053_1/p053_4/
  p053_6/p054` widget_computed 族 + `widget_computed_store_arg_helper_chain`
  /`widget_computed_passthrough_survives_reeval`）——修前修后零增减，
  若 T-00 定音与之同根则一并修复并改判基线。

### 风险与约束

- **主检出 WIP**：auto-lang master 工作区有未提交改动 8 文件
  （aura_view_builder/renderer/session/view/video_uplink/mpv + 020-music-
  player 示例，2026-10-10 媒体引擎方向，与本三断点无关）——**他人 WIP，
  不归本计划**；worktree 基面钉 master 已提交代码，rebase 前须与该 WIP
  owner 协调（向用户上报，不静默包含/丢弃）。
- **敏感度**：Stakes/RC（419/510/624/733 四轮修复区）——改动落错误
  处理/求值侧，禁碰弹栈协议；renderer/aura_view_builder 补丁密度极高，
  禁打静默兜底补丁（恒 false 塌缩史）。
- 观测约束：musk 全部实机证据来自 dirty 构建 + 当日 master 正被活跃
  开发，**T-00 干净基面重取是硬门槛**，不得直接引用 101 的观测定罪。

### Phase 2 授权与供料（2026-10-10 增补，plan_revision 2）

用户 2026-10-10 指示：jade-edit 供料的 vm 轨 iced 渲染器四件缺陷
（三件布局 + 一件呈现缺口），供料包 jade-edit
`docs/upstream/2026-10-09-vm-layout-stretch-supply.md` 全文 + 证据截图
jade `e2e/.runtime/exp5..13*.png`（exp5=健康基线对照，2026-10-10 在位
核验），「走 auto-lang 自身 plan 流程立项，不直接改 jade-edit」「把
上述问题更新到计划 748 里，作为独立的新 phase」。同 Phase 1：本增补
属起草；work 授权待用户确认计划后另行给出。

### Phase 2 已取证事实（jade-edit 实测，2026-10-08/09）

证据载体 release `auto 0.1.0+v0.4.2-2747-gab7a650bd`（2026-10-08 构建
）；供料复验时 auto-lang HEAD `1b05d3d44`——**stretch 代码与 2747 内涵
一致**（ab7a650bd 为 HEAD 祖先，此后 renderer.rs/stretch_line.rs 仅
focus/resize/fit 三件无关修复）。jade 侧实验均在 `auto run -r vm`
merged 形态 + PrintWindow 截图 + 像素测量取得；jade-edit `src/front/
app.at` 双轨单源（vue=CSS 渲染正常，vm=iced 解释渲染破）。

| 件 | 症状 | 对照正常形态 |
|---|---|---|
| 一 | stretch 行 Shrink 高子项止于内容高（窗口 1113×786，侧栏 bg-card 底边 y=550=树内容底，行本身 flex-1 满高） | 行高解析正确；vue 轨 CSS align-items:stretch 正常 |
| 二 | overflow-y-auto col + h-full：末位类序→col 背景/几何拉满行高但子树全部消失（快照节点在、屏幕零像素）；首位类序→整个应用降级无样式（menubar 默认字号巨大化、其余子树不渲染） | 同位 flex-1（axis_fix 转写 Height(Full)）正常；jade workaround 用宿主 h-full + 内层 flex-1 滚动避开 |
| 三 | [定宽A, FillPortion B, 定宽C]：B 越界占满整行、C≈0 宽，行 overflow-hidden 下完全不可见；vm_matrix 测不出（快照断言不含几何） | 两子行 [定宽, flex-1] 正常（jade 主布局在用） |
| 四 | vm 编辑器正文贴死分隔线 | vue 轨引擎 CSS wrapper padding（1rem/1.25rem）承担 |

jade 侧 workaround（已验证双轨等值，上游修复后撤层复验，jade commit
4b398a4）：①侧栏拆宿主 h-full col（无 overflow）+ 内层 flex-1
overflow-y-auto col；②编辑器外包 px-5 py-4 容器 + 任意变体
`[&_.autodown-editor-content-wrapper]:p-0` 清零引擎层（vue 轨 specificity
(0,2,0)>(0,1,0) 构建产物核验；vm 轨变体天然 no-op）。

### Phase 2 代码定位（master @ 7bd28888b 工作区实读，2026-10-10；所引区块在主检出在途 WIP hunks 之前，与已提交 master 行号一致）

- **件一/件三（stretch_line.rs + build_row）**：
  `crates/auto-lang/src/ui/iced/stretch_line.rs` 头注 :1-25 声明
  「CSS align-items:stretch 语义原语化」；final 遍 :164-193 对 Shrink
  高子项 `min_h=effective`（:171-175）+ max=effective 落位——**嫌疑：
  iced 0.14 `Length::Shrink` resolve 忽略 limits.min**，且子项实际
  载体是 Scrollable/背景 Container 包装链（needs_visual_wrap）而非
  裸子项，min 高在哪层死掉待 probe。宽度探测遍 :110-142：非 fill
  子项按序吃自然宽、fill 子项均分剩余——`fill_portion`（:70-76）读
  **widget 层 Length**（`c.as_widget().size().width`）；row 子项经
  apply_*_style/包装链后外层 Length 可能不再反映 style 语义（flex-1
  col 的 width=Full 载体丢失 → 被判非 fill → 按内容宽吃满 available
  → 末位定宽子被 max 钳 0——与「B 越界占满、C≈0」症状吻合，待
  probe 证实）。build_row StretchLine 分支 renderer.rs:2527-2561
  （line_fill_height :2533-2536）。
- **件二（转写链 + build_scrollable）**：aura_view_builder.rs:2716-2742
  （needs_scroll → `View::Scrollable`，**style 整串 clone** 含全部类）
  → renderer.rs build_scrollable :2738-2886——显式高臂 :2840-2863
  （Full/Screen → `s.height(Length::Fill)` :2844-2846）、max-h cap 臂
  :2815-2826 注释 + :2879-2885（Shrink + Container::max_height 封顶，
  供料疑点所在）。对照正常路径 axis_fix_col_child renderer.rs:1141-1156
  （flex-1 → 类级 `StyleClass::Height(Full)` 转写）——**显式 h-full
  类与转写产物在 StyleClass 层同值，from_style 后应同 Length；分叉
  说明分叉点在类→Length 之后的某层（或转写时序差）**。类序敏感先例：
  iced_adapter.rs:1181-1193 Flex1→width=Full 注释明言「avoiding
  order-dependency between flex-1 and overflow-y-auto in the class
  list」（Plan 370 Issue 1）——IcedStyle 层曾做过一次类序去敏，件二
  疑似同族残余在别层（Style 解析/merged_with_variant/vm 侧 style 串
  装配待查）；**首位=全局降级**（menubar 巨字号、其余子树不渲染）
  指向带全局副作用的分支或被吞 panic，须 probe 定罪，不得猜。
- **件四（autodown_editor）**：vm 件 `crates/auto-lang/src/ui/
  autodown_editor/`（widget.rs 766 行/core.rs 8351 行/mod.rs，**全模块
  零 padding 载体**——grep 实证）；vue 轨对照 auto-down 引擎
  `autodown/packages/engine/src/editor/styles/autodown-editor.css:103`
  `.autodown-editor-content-wrapper { flex:1; padding:1rem 1.25rem;
  overflow-y:auto; … }`（经 editor/index.ts:1 随组件导入；1rem=16、
  1.25rem=20 逻辑 px）。DSL 侧注册 aura/schema.rs:3111；examples/ui
  无使用例（jade-edit 为唯一实机消费者——仓内判绿须自带最小 fixture
  /单测，像素对照依赖 jade 跨仓复验）。

### Phase 2 风险与约束（供料包修复约束四条 + 本仓两条）

1. **P642-D12 不回归**（供料约束 1）：StretchLine 两阶段存在的理由
   就是 scroll 内容臂（无界）下 Fill 包装塌缩 0 高（008 定价卡前科）
   ——件一/件三修法不得把 Fill 拉伸产物放回无界祖先语境（显式
   Fixed(effective) 落位候选天然满足：effective 在无界语境=内容高，
   恒有界）。
2. **类序无关性**（供料约束 2）：件二首位/末位分叉说明存在顺序敏感
   路径；修复后**任意类序等价**——golden/像素对照必须加乱序案
   （h-full × flex-1 × overflow-y-auto × 尺寸/背景类的排列组合）。
3. **PLAN-655 既有语义保持**（供料约束 3）：line_fill_height（定高行
   吃满有界入射、无界回落 auto，stretch_line.rs:58-66 纯函数单测）与
   041 auto-edit 主行既有绿面（layout_tests.rs:2907- PLAN-655 段 +
   fix-stretch-fill 回归面）零漂移。
4. 判绿面（供料约束 4）：iced 档布局单测（三件各一）+ ui-gallery/041
   像素对照 + jade 侧复验（jade vm 矩阵 merged 臂 + 供料包复现形——
   jade 仓跨仓动作，见 §10）。
5. **主检出 WIP 同文件冲突**：Phase 2 触 renderer.rs/aura_view_builder.rs
   ——与上文主检出媒体引擎 WIP 8 文件**同文件**；worktree 基面恒钉
   master 已提交代码，rebase 前与 WIP owner 协调（同 Phase 1 前置面）。
6. **iced 引擎级行为边界**：件一根因若为 iced `Length::Shrink` 忽略
   limits.min 的引擎级行为，修复在渲染器层补偿（自持落位），不开
   iced fork/升级（PLAN-655 非目标第 5 条既有裁定延续）。

## 详细设计

### T-00 定音探针（判定产物：定音报告，含每断点根因层 + 修复路线）

1. **干净基面**：`git worktree add D:/autostack/.wt/lang-748/auto-lang -b
   plan-748-dev master`（或 new-wt-group.sh）→ 仓内 cargo 构建 debug
   auto.exe，固定 `AUTO_EXE`。
2. **断点①矩阵**：最小 .at 语料（fixture widget：根件 + 子件各一），
   条件形态 `{w.is_current / .store.x==w.id / computed && w.id}` × 挂载
   上下文 `{根件/子件}`，headless 快照（`ui/headless/renderer.rs` 通道）
   + `VmBridge::new_from_decls` 双通道取真值；对照 ARM：
   `musk_vm_track` 预存红测试在干净基面的失败面逐条记录。
3. **断点②探针**：resume FAILED 路径补 crash-ip + 反汇编窗
   （vm_bridge.rs:1829 镜像 :2922 形态，含 disasm 窗口 20 指令）；
   三臂语料（静态写/动态键写/循环绑定写）× `{有 await, 无 await}` 对照；
   `AUTO_VM_TRACE_OPS=1` 抓续体末 50 指令。定音：Err 被谁吞、catch_pc
   还是帧展开、是否 705 族残余。
4. **断点③核验**：干净基面跑 musk 101 选择器行（`style: .w.row_style`
   形态最小化）确认丢弃是否复现于干净构建；清点静态臂清单。
5. **判定纪律**：每断点输出"根因层（文件:符号）+ 修复路线（A/B 案）+
   回归面"；证据不足时扩探针不猜。

### T-01 断点②修复（红相先行）

按定音落点二选一：
- **路线 A（Err 吞没面）**：`intercept_error`（engine.rs:11251）残余 +
  handler_codegen catch 目标生成；修"错误发生在 try 同帧被调 RAM 帧且经
  park/resume"组合的 catch_pc 错位。
- **路线 B（帧协议面）**：段驱动嵌套 CALL 帧 bp 协议（624 族 push/pop
  对称性）。
红相 spike 语料（705 形状）：handler 内 await HTTP → resume 后静态写 +
  动态键写 + 循环写三臂断言段 Completed(Ok)；测试挂
  `plan748_engine_resume_field_write_tests`。

### T-02 断点①修复（红相矩阵先行）

按定音落点二选一：
- **路线 A（子件挂载面）**：computed 表 / store 别名快照在
  DynamicComponent 子件构建时的挂载（vm_bridge `new_from_decls:741`
  一带 + handler_codegen.rs:184 快照机制）；若与 P733-R2 同根则合并
  修复并改判预存红基线（§9 说明 + KNOWN-DEBT 条目更新）。
- **路线 B（求值器补臂）**：`eval_condition_with_inner` 具体形态补臂
  （可能性低，改动最小）。
单测扩 `test_eval_condition_with_bindings` 矩阵：成员真值/RHS 成员/
  store 混合 × 根件/子件。

### T-03 断点③收口

静态臂转换器（tabs/center/video/progress/spacer/checkbox）
`extract_style` → `extract_style_with` 机械收口；`Style::parse` 词表
丢弃在非 button 臂补 446-U7 同款 WARN；`tests/style_parity.rs` +
038 minesweeper 动态样式 e2e 回归。

### T-04 跨仓回退对拍

musk PLAN-101 worktree（.wt/musk-101/auto-musk @ 5b2fa9f）起两 variant：
1. **workaround 版**（现状）：发送闭环 + 选择器 ✓ probe 全绿（基线）；
2. **回退版**（patch 临时还原：`if w.is_current` 直读 + `item.x=v`
   静态写 + `style: if w.is_current {...}`）：本计划 auto.exe 下必须
   同等全绿——这是三断点修复的权威验收。
锁 AUTO_EXE 指向 748 worktree 构建；musk 侧 ui-parity check 无新增红。

### T-05 全量门与收尾

`cargo test` 受影响套件（tv 3731 / tf 3560 / RC 生命周期 /
plan702/705/707/711 segment 面 / plan733 矩阵 / musk_vm_track——
预存红零增减）+ auto-lang `scripts/ui-parity.mjs run --plan 748`（如
有对拍 case）+ daily 回归与 master 基线逐名对拍。提交 + §9 记录 +
KNOWN-DEBT-AND-RISKS.md 条目更新（三断点销账/新边界）。

### Phase 2 设计（T-06～T-11）

#### T-06 复现定音（判定产物：四件定音报告）

1. **基面**：与 T-00 同 worktree 复用（lang-748，干净 master 构建
   AUTO_EXE）——供料观测同样来自 release 构建，干净基面重取是硬门槛。
2. **件一 probe**：供料复现形（`col h-full w-full > row flex-1
   items-stretch w-full > [w-56 bg-card overflow-y-auto 短内容,
   flex-1 多行内容]`）headless 布局 dump——StretchLine final 遍逐子项
   入射 limits 与产出 node 尺寸 + 子项包装链逐层 Length/min 解析；
   对照 008 同构行测试形态（layout_tests.rs:2939）差异定位。定音：
   min_h=effective 死于哪层（Scrollable viewport 高解析 / 背景容器
   Shrink resolve / min 未传到内容臂）。
3. **件二 probe**：同视图三变体（h-full 首位 / h-full 末位 / flex-1
   替代）三层树 diff（`Style::parse` 结果 → AbstractView → widget
   Length 集合）；首位全局降级变体加 stderr/panic 捕获定罪全局副作用
   层。定音：顺序敏感层 + 显式高子树零几何机制（两者可能同层或分属
   两层）。
4. **件三 probe**：`[w-56, flex-1, w-72]` 复现形 dump 宽度探测遍逐子项
   fill_portion 判定值 + available 扣减序列 + final_w 终值；并 dump
   row 子项 widget 树外层 Length（验证 flex-1 col 的 width=Full 是否
   在包装链上丢失）。w-72 与 w-[288px] 双宽度类形态各跑一遍（供料
   称同败——若同败则与类解析无关，异败则先修类解析）。
5. **判定纪律**：同 Phase 1——每件输出「根因层（文件:符号）+ 修复
   路线（A/B 案）+ 回归面」；证据不足扩探针不猜。件四豁免定音
   （缺口确证），直接进 T-10。

#### T-07 件一修复（交叉轴拉伸兑现）

按定音二选一：
- **路线 A（StretchLine 自持落位）**：final 遍对 Shrink 高子项不再
  依赖 limits.min 传递——以已解析的 effective 显式定高落位（候选：
  包装 Fixed(effective) 高容器或等价注入，Shrink→Fixed 语义 = CSS
  auto 高项拉伸到行高，Fixed/Fill 高子项行为不变）。约束 1 合规：
  注入值恒有界（无界语境 effective=内容高），非 Fill——不触碰
  P642-D12。
- **路线 B（子项载体层补偿）**：若定音显示仅特定载体（Scrollable/
  背景容器包装链）忽略 min——在 build_scrollable/apply_*_style 对应
  分支补偿 min 高传递（影响面更大，须全量 stretch 回归面护航）。
判绿：复现形左 col 高=行高（含 bg 拉伸像素证）；008/041/gallery 既有
stretch 面零漂移。

#### T-08 件二修复（显式高 Scrollable + 类序无关）

按定音落点修两症（可同根合并修）：
- **子树零几何**：build_scrollable 显式高分支（:2840-2863 一带）对
  Height(Full/Fixed) 入射的内容高解析修复；若与件一路线 B 同根则合并。
- **类序敏感**：顺序敏感层去敏（对齐 Plan 370 Issue 1 的 from_style
  先例手法——类集语义在解析层归一，不依赖类串顺序）；首位全局降级
  若为独立全局副作用分支则单独根修。
判绿：三变体（首位/末位/flex-1）渲染等价 + 乱序 golden 案全绿；子树
几何非零（MCP snapshot + PrintWindow 像素双证）。

#### T-09 件三修复（主轴配给定宽预留）

按定音二选一：
- **路线 A（组装期显式语义，结构性根治候选）**：build_row
  StretchLine 分支组装 items 时以 style 层 IcedSize 语义显式传入子项
  主轴三元组（element, main_fill: Option<f32>, fixed_w: Option<f32>）
  ，配给数学显式预留定宽（B = 行宽 − Σ定宽 − spacing）——摆脱对
  widget 包装层 Length 的反推，包装链变化不再影响配给判定。
- **路线 B（fill_portion 判定/配给数学最小修正）**：若定音显示
  Length 载体未丢失而是探测遍扣减序或公式错——最小修正
  stretch_line.rs :110-142。
判绿：复现形 B=行宽−A−C 且 C=288（w-72）；两子行 [定宽, flex-1]
既有行为零漂移。

#### T-10 件四修复（autodown_editor wrapper padding）

vm 件内建复刻引擎 wrapper padding：内容层 padding 上下 16 / 左右 20
逻辑 px（对齐 autodown-editor.css:103 `padding:1rem 1.25rem`）；滚动
语义对齐 CSS（padding 在滚动内容上、随滚动参与范围——引擎 wrapper
自身 overflow-y:auto + padding 的等价形态）。不动 vue 轨/引擎 CSS；
数值以引擎侧为单一真源（引擎常量可导则引用，不可导则注释锚定 css
行号）。判绿：vm 侧快照/单测 padding 断言（仓内自带最小 fixture，
examples 无使用例）+ jade 撤层双轨像素等值（jade 侧，AC-14）。

#### T-11 判绿收口与跨仓复验

1. iced 档布局单测三件各一（挂 layout_tests.rs PLAN-655 段后，
   plan748 前缀）+ 件二乱序等价 golden 案；
2. 041-auto-edit 像素对照 + gallery 围栏（autoui-verifier
   test_widgets_gallery_vm.py）零漂移——重负载走机器级单实例闸门，
   主检出单实例；
3. jade 撤层跨仓复验（jade-edit 仓，用户/jade 会话协调）：撤两层
   workaround（commit 4b398a4）→ vm_matrix merged 臂全绿 + 供料四
   复现形正确 + 双轨 PrintWindow 逐像素同构 + 右栏面板可见；
4. 全量门：裸 `cargo t`（per-plan 复审门禁，fix-test-tiering——
   ui/iced renderer 面无专档触面）+ 预存红零增减对拍 +
   KNOWN-DEBT-AND-RISKS.md 更新（四件销账/新边界）。

### 规范增量（R1 二轮定稿——对齐落地实态；原 SD-02/SD-03 草案按 StretchLine 修法起草，T-06 定音证伪后废弃，见 §R1 复审记录）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md 已知边界节 | before：已知边界③ = P733-R2 六红族（`.store.X` 子件 override 深层待查）；after：边界③收口——`.store.X` 泛名展平三口径归一（resolve_expr_to_value Dot 臂无条件展平 Dot(Dot(self,store),X)，真名限定仍走快照门，`2e2164b5e`）；六红三根分解：三红翻绿 + 两红=图标判别器根修（lucide 名集成员资格替代 `contains("icon")` 粗闸，`3ec6a4914`，PLAN-625 T-09(b) 意图由 AppViewport ∉ 名集保持）+ 一红 p053_4（CALL_NAT 发射）异根改派 codegen 债务观察项 | P733-R2 销账 + PLAN-101 断点①收缩面 | AC-03, AC-06 |
| SD-02 | add | docs/specs/auto-lang/ui/overview.md#overflow-滚动转写-视口视觉层（PLAN-748） | before：无（aura_view_builder needs_scroll 转写把整串 style clone 给滚动内容 col——视觉类随内容止于内容高，「bg 底边=内容底」）；after：overflow-y-auto col 转写 View::Scrollable 时视觉类（bg/border/rounded/shadow/ring，`StyleClass::is_visual_paint`）上移**视口 Container**（镜像原显式 Width/Height 类、缺省补 Full——iced 0.14 `Limits::Shrink` 置 compression 位、Fill 子件塌内容高），内容 col 拿剥离余集；无视觉类零包装层。CSS 同构：滚动容器背景属容器本体。附注：类序无关由 parse 逐 token 保证（layout_tests p748_j2 三变体等价锁在案） | jade 件一根修（4b4a7251e）；StretchLine 无罪（T-06 定音） | AC-08, AC-12, AC-13 |
| SD-03 | add | docs/specs/auto-lang/ui/overview.md#autodown-editor-引擎-parity（PLAN-748） | before：vm 件无 wrapper padding 语义登记（正文贴死分隔线）；after：vm autodown_editor 内建引擎 CSS 等值 wrapper padding（上下 16/左右 20 逻辑 px，单一真源 = 引擎 autodown-editor.css:103 `padding:1rem 1.25rem`）；widget 四点联动契约（layout 内缩测量/draw 原点平移/命中坐标内缩/IME 锚偏移） | jade 件四双轨呈现缺口（bb494c762） | AC-11 |

## 测试设计

| 验证 | 命令/方法 | 期望 |
|---|---|---|
| V01 定音探针 | T-00 探针套件（仓内 test + headless 快照） | 三断点根因层判定报告；干净基面复现成功；非续体对照组结论 |
| V02 红相 spike | `plan748_engine_resume_field_write_tests` + 条件矩阵单测 | 修复前红（绑定干净基面对应形态）、修复后绿 |
| V03 全量回归 | `cargo test`（tv/tf/RC/segment/plan733/musk_vm_track） | 零新增红；预存红六条零增减（若同根修复则改判并说明） |
| V04 跨仓对拍 | musk 101 workaround/回退双 variant probe + ui-parity | 双 variant 同绿；musk 无新增红 |
| V05 daily 对拍 | daily 回归 vs master 基线逐名 | 零增减 |
| V06 P2 定音探针 | T-06 探针套件（headless 布局 dump + 三层树 diff + 类序变体） | 件一/二/三根因层判定报告；干净基面复现成功（件四豁免——确证缺口） |
| V07 P2 布局单测 | plan748 前缀布局单测（件一/二/三复现形各一 + 件二乱序 golden + 件四 padding 断言） | 修复前红（绑定定音复现形态）、修复后绿；乱序案任意排列等价 |
| V08 P2 既有面零漂移 | layout_tests PLAN-655 段 + line_fill_height 单测 + 041 像素对照 + gallery 围栏 | line_fill_height/008 同构行/041 主行零漂移 |
| V09 P2 跨仓复验 | jade 撤层（4b398a4 两层 workaround）后 vm_matrix merged 臂 + 供料四复现形 + 双轨 PrintWindow 像素对照 | 全绿/复现形正确/双轨同构/右栏面板可见 |
| V10 P2 全量门 | 裸 `cargo t`（复审门禁）+ 预存红零增减对拍 | 零新增红 |

## 验收标准

| ID | 可观察结果 | 验证 |
|---|---|---|
| AC-01 | 干净基面复现三断点（或证伪并改判——dirty 构建/WIP 污染情形须报告） | V01 |
| AC-02 | handler 续体静态名/动态键/循环绑定写三形态与无 await 对照组行为一致，无 Terminated | V02/V03 |
| AC-03 | 模板条件成员访问（含子件上下文、store/computed 混合形态）求值真值正确 | V02/V03 |
| AC-04 | `style:` 动态字段引用在静态臂元素产出；词表丢弃有 WARN | V03 |
| AC-05 | musk 101 回退版（直读条件）在本计划构建跨仓 probe 全绿 | V04 |
| AC-06 | 全量零新增红；预存红零增减（或同根改判记录在案） | V03/V05 |
| AC-07 | 禁静默兜底补丁：三修复点均可在单测中观测到真值行为差异（非"看起来有臂"） | review 绑定代码+测试 |
| AC-08 | items-stretch 行内 Shrink 高子项（含 overflow-y-auto col）几何/背景拉伸到行高（供料复现形左 col 高 = 行高） | V06/V07/V08 |
| AC-09 | overflow-y-auto col 挂显式高（h-full/h-*）子树几何非零；任意类序（首位/末位/乱序）渲染等价，无全局降级形态 | V06/V07 |
| AC-10 | `[定宽, flex-1, 定宽]` 行 flex 子宽 = 行宽 − Σ定宽 − spacing，定宽子不塌 0 宽（jade 右栏面板可见） | V06/V07/V09 |
| AC-11 | vm autodown_editor 内建 wrapper padding 与引擎 CSS 等值（上下 16 / 左右 20 逻辑 px） | V07/V09 |
| AC-12 | P642-D12 不回归：scroll 内容臂（无界）语境 stretch 行为零漂移（008 定价卡族测试零漂移） | V08 |
| AC-13 | PLAN-655 语义保持：line_fill_height 单测与 041 auto-edit 主行像素对照零漂移 | V08 |
| AC-14 | jade 撤两层 workaround（4b398a4）后双轨逐像素同构 + vm_matrix merged 臂全绿 + 右栏面板可见 | V09 |
| AC-15 | Phase 2 全量门零新增红、预存红零增减；四修复点均有可观测行为差异的单测（禁静默兜底，同 AC-07 纪律） | V10/review |

## 执行步骤

| 完成/任务 | 依赖 | 文件/符号与产出 | 验证 | AC |
|---|---|---|---|---|
| [x] T-00 | 无 | worktree lang-748（plan-748-dev，基 master **e5a77106a**——重取基面含 PLAN-749 G1-G3/STR_CAT 修复，待澄清#6 前置落实）；探针 `plan748_breakpoint_probes`（提交 e9d953ec4）；**定音报告见 §9 T-00 节** | V01 | 01 |
| [x] T-01 | T-00 ②定音 | **证伪改判**（T-00 定音）：fn 段三臂（静态 `W:5`/动态 `D:6`/无 await `N:7`）全绿 + E2E 完整管线（handler_App_Init resumed to completion，静态写 marker=W:5）绿——musk 101 §10① 观测为 dirty 构建陈旧（v0.4.2-2730-dirty vs e5a77106a 之间的修复，749 E2E 同管线佐证）；无修复对象，探针钉死为回归面（AC-02 按 AC-01"或证伪并改判"条款关闭） | V01/V02 | 02 |
| [x] T-02 | T-00 ①定音 | **范围收缩改判**（T-00 定音）+**首腿已修**（`2e2164b5e`）+**续腿已修**（`3ec6a4914`，2026-10-10）：①`.store.X` 泛名展平三红翻绿（musk_vm_track 97/6→100/3）；②p054×2 图标子件渲染根修——PLAN-625 T-09(b) `tag.contains("icon")` 粗闸把 PascalCase lucide 名（Plus/Info/…）误判为 web 组件→占位卡；判别器改 **lucide 名集成员资格**（`is_icon_component_tag`：含 icon 字样向后兼容 ∥ `iced::lucide_icon_known` 名集二分查表，不动生成文件），AppViewport ∉ 名集保持 T-09(b) 占位卡意图；plan050 契约测试同步纠正（A-05 曾把粗闸钉成契约）+AppViewport 锁死断言；musk_vm_track **102/103**（唯一余红=p053_4 异根，KNOWN-DEBT 在案）+plan050/plan633/plan733/plan051 10/10+aura 53/53；定音副产物：plan051 模块级图标测试为 corpus 未寻址空洞绿（P748-D1 登记）；③p053_4（merged `#[api]` CALL_NAT 3142 发射）**异根改派**（codegen 面，P733-R2 条目③在案） | V02/V03 | 03 |
| [x] T-03 | T-00 ③核验 | **证伪改判**（T-00 定音）：动态 style 字段引用在 row/checkbox/progress 三元素全部产出（`style: .store.X` → Style 载体 BorderColor/TextColor/BackgroundColor 枚举实拍）——计划 §4 静态臂清单（checkbox :8326/progress :8265）与现 master 实态不符（相关臂已收口）；musk 101 §10③ 观测为 dirty 构建陈旧。**残留小项**：Style::parse 词表丢弃的非 button 臂 WARN 对齐（独立于断点③复现与否，静默丢弃面仍在——T-11 收口时顺手或另立小项） | V01/V03 | 04 |
| [x] T-04 | T-01～T-03 | **收缩版完成**（T-00 定音裁定：回退版三形态已被探针等价语料覆盖证明）：musk 真实 app（含 101 workaround 代码）在 748 构建（bb494c762）上 `auto run -r vm` 冒烟通过——snapshot 756 行完整视图树、无崩溃、workaround 形态继续工作（MUSK_BACKEND=vm + AUTOUI_MCP_PORT 通道，2026-10-10 实录）。双 variant 全量对拍随 101 worktree 退役（275a187 cleaned）降为冒烟确认 | V04 | 05 |
| [x] T-05 | T-04 | 全量门 + 提交 + §9 记录 + KNOWN-DEBT 更新 + owned 清理——**R1 复审在 HEAD bb494c762 复跑收口**：tv 162/162 + 裸 cargo t 5130/5145（15 红全归因在案/flake，零新增——见 §R1 门禁复跑）+ §9 定音/交接记录 + KNOWN-DEBT（P733-R2 销账 90aeea96e + P748-D1）+ 六提交全 committed 零脏 | V03/V05 | 06,07 |
| [x] T-06 | 无 | 四件定音探针 + 判定报告（§9 Phase 2 定音报告；提交 29df32ec0——p748_j1/j2/j3 复现形 headless 探针 + stretch_line P748_PROBE dump + live VM 双通道）：**件一复现**（jade 真 app pre-workaround：视口 Fill 拉伸正常、内层内容 col 止于内容高 163/675，bg 随之止步——根因=转写层非 StretchLine，供料定位证伪）；**件二/件三不复现**（HEAD 干净构建 headless+live 双通道、最小形+真 app、类序首位/末位等价、右栏 288 满高可见、配给精确 588=1100−224−288——stretch_line.rs 与 release 2747 逐字节一致、renderer/layout/view_builder diff 均不含布局路径，release 观测无法归因代码差，登记观察项交 jade 撤层复验销案）；**件四确证**（widget 全模块零 padding 载体） | V06 | 08,09,10 |
| [x] T-07 | T-06 件一定音 | **件一根修**（`4b4a7251e`）：滚动转写视觉类上移视口层——split_scroll_visual_classes（StyleClass::is_visual_paint 新谓词：bg/border/rounded/shadow/ring 族）双现场拆分，视口 View::Container 镜像原显式 Width/Height 类缺省补 Full（iced 0.14 Limits Shrink 置 compression 位、Fill 子件塌内容高 42px 实证），Scrollable ensure_full_scroll_dims Fill×Fill，内容 col 拿剥离余集；无视觉类零包装层。**live 终验**：jade-real 侧栏视口容器 224×675 承载 bg-card/border-r 满高（像素 (100,250..700)=rgb(13,20,37) 精确色），右栏 backlinks_open=true 满高可见。布局档 65/67（唯二在案存量红）+裸 cargo t 5133/5145 零新增红 | V07/V08 | 08,12,13 |
| [x] T-08 | T-06 件二定音 | **证伪改判**（T-06 定音）：件二在 HEAD 不复现（真 app + 最小形、首位/末位 h-full 几何全等、子树非零）；回归锁已落地（p748_j2_explicit_height_scrollable_class_order_invariant 三变体等价断言）+ stretch_line 探针 dump 留档 | V06/V07 | 09 |
| [x] T-09 | T-06 件三定音 | **证伪改判**（T-06 定音）：件三在 HEAD 不复现（[w-56,flex-1,w-72] 配给精确、右栏满高可见，live 实测 @rect(812,32,288,675)）；回归锁已落地（p748_j3_fixed_flex_fixed_allocation）；release 2747 观测与 HEAD 代码差无法归因（stretch_line 逐字节一致）——观察项交 jade 撤层复验（AC-14）销案 | V06/V07 | 10,12,13 |
| [x] T-10 | 无 | **件四根修**（`bb494c762`）：autodown_editor 内建引擎等值 wrapper padding（CONTENT_PAD_Y=16/X=20，锚引擎 css:103 单一真源）——widget 四点联动（layout 内缩/draw 原点平移/update 命中坐标/IME 锚）+ plan732 全管线点击坐标同步 + 常量单源锁 + 布局级断言（sentinel y=内容高+32 实测 88.6=56.6+32）；autodown_editor 127 测全绿（autodown feature 档） | V07 | 11 |
| [x] T-11 | T-07～T-10 | 判绿收口——**R1 复审定谳**：布局档 65/67（唯二在案存量红）✓+裸 cargo t 零新增红（HEAD 复跑）✓+autodown 档 127/127 ✓+041 live 冒烟 ✓（主行 741 满高、侧栏新结构正收益——AC-13 证据）+KNOWN-DEBT 落账 ✓；**gallery 围栏归位批量回归档**（fix-test-tiering 语义——本行原文「gallery 围栏零漂移」越档表述修正；日常档内 gallery 语料面=plan606 在案红已跑）；jade 撤层跨仓复验（AC-14）= external-pending（§10.4 在案，merge 后跟踪，669/682 先例） | V08/V09/V10 | 12,13,15 |

## 复审记录

### R1 独立复审第一轮（2026-10-10，needs_fix——R-02/R-03 计划面修正后二轮）

- stage: review | plan_id: PLAN-748 | plan_revision: 4 | reviewed_commit: plan-748-dev `bb494c762`（6 提交，基 e5a77106a，工作区零脏）| base_commit: master `e5a77106a`（master 已并发前移至 515acb47c——Plan 738 会话活跃，merge 时 rebase 注意）
- 门禁复跑（HEAD）：`cargo tv` 162/162 ✓；裸 `cargo t` 5130/5145，15 红全归因（在案 9：REG-4×3/批量回执 known_reds×3/p053_4/P748 探针 harness×2 + 计划 738 会话负载窗 flake 3：clipboard/plan502[P733-R1]/plan705_spike[scoped 复跑 19/19 绿]）——**零新增红**；布局档 65/67（唯二在案存量红）；autodown 档 127/127；style_parity 2/2。
- 运行态复核：041-auto-edit live 冒烟 ✓（主行 741 满高；侧栏呈 T-07 修复后新结构 container 224×741>scrollable 741>内容 200——修复对真实示例正收益）；jade-real pre-workaround 像素终验复述在 T-07 行（bg-card rgb(13,20,37) 满高）。
- acceptance_results：AC-01/02/05/06/07/08/09/10/11/12/13/15 **pass**；AC-03 pass（条件求值面收口；p053_4=CALL_NAT 发射面异根改派在案）；**AC-04 pass（改判，见 R-01）**；AC-14 partial（external-pending——jade 撤层复验，§10.4 在案非阻塞项，merge 后跟踪）。
- findings：
  - **R-01（改判记录，非代码必修）**：AC-04 后半「词表丢弃有 WARN」——原草案设想的每臂 WARN 与落地架构冲突：`Style::parse_reported`（Plan 527 T1）已是显式报告通道（未知 token 按名返回非静默丢，style_parity.rs 审计面 2/2 绿）；每臂 eprintln 属 WARN 风暴反模式（PLAN-062「855 rebuild-WARN/45s」/PLAN-725 前科）。§4 原文「button 臂有 446-U7 WARN :10910」系草案误引（446-U7 实为 MCP 重建 WARN）。裁定：词表可观测性经既有通道满足，AC-04 关闭。
  - **R-02（必修——计划文档）**：规范增量 SD-02/SD-03 按原设计（StretchLine 语义修改）起草，落地实态为**转写层**修复（件一）+ 证伪不改（件二/三）——增量表须重写对齐实态；frontmatter supersedes/new 待定稿（vm/architecture.md → vm/design/vm-fn-call-semantics.md 边界③）。
  - **R-03（必修——簿记）**：T-05/T-11 未收口。裁定：门禁已由本复审在 HEAD 复跑（上录）；gallery 围栏归位**批量回归档**（fix-test-tiering 语义——T-11 原文「gallery 围栏零漂移」表述越档，修正；日常档内 gallery 语料面已跑=plan606 在案红）；041 live 冒烟已补（AC-13 证据）；jade 撤层复验保持 external-pending。T-05/T-11 勾选收口。
  - R-04（观察）：master 并发前移（Plan 738 会话）——merge rebase 协调点。
- outcome: **needs_fix**（R-02/R-03 均为计划文档/簿记修正，无代码改动）→ work 短修 → 二轮复审。

### R1 二轮复审（2026-10-10，pass）

- stage: review | plan_id: PLAN-748 | plan_revision: 4（R-02/R-03 为增量文本与簿记修正，验收 ID/语义范围未变，不递增）| reviewed_commit: plan-748-dev `bb494c762`（代码与一轮同基——一轮门禁/运行态证据直接复用，理由：代码/依赖/测试配置零变化，R-02/R-03 仅计划文档）| base_commit: `e5a77106a`
- 修复核验：R-02 ✓（规范增量表重写为 SD-01 fn-call-semantics 边界③收口 / SD-02 视口视觉层 / SD-03 editor parity；frontmatter supersedes=new 定稿）；R-03 ✓（T-05/T-11 收口勾选、gallery 归位批量档、current_step 12/12）；R-01 ✓（改判记录在案）；R-04（merge rebase 注意）移交 merge。
- acceptance_results（终表）：AC-01 pass｜AC-02 pass（证伪改判）｜AC-03 pass｜AC-04 pass（R-01 改判：parse_reported+style_parity 通道）｜AC-05 pass（收缩版等价覆盖）｜AC-06 pass（15 红全归因）｜AC-07 pass｜AC-08 pass（live 像素终验）｜AC-09 pass（证伪+回归锁）｜AC-10 pass（证伪+回归锁）｜AC-11 pass（布局级断言+127 测）｜AC-12 pass（布局档 p655 段 61 绿）｜AC-13 pass（line_fill_height 单测+041 live 冒烟）｜AC-14 **external-pending**（jade 撤层复验，§10.4 在案用户背书的非阻塞结构，merge 后跟踪）｜AC-15 pass。
- 债务/边界清单复核：p053_4（异根 codegen 观察项，P733-R2 条目③）｜P748-D1（plan051 corpus 空洞绿）｜jade 件二/三 release 观测归因观察项（AC-14 撤层复验终局销案）｜gallery 围栏批量档。无未批准延期；无 workaround 补丁残留（jade 侧 workaround 属其仓内待撤层物，非本仓债）。
- evidence: §R1 一轮（门禁复跑+运行态）+ 各任务行提交哈希与实测数据；规范增量冻结于本表（merge 按 SD-01/02/03 落 specs）。
- outcome: **pass** → status: reviewed，next: merge（rebase 注意 master 并发 515acb47c）。

### T-00 定音报告（2026-10-10，worktree lang-748 @ e5a77106a 干净基面，含 PLAN-749）

**探针**：`crates/auto-lang/src/tests/plan748_breakpoint_probes.rs`（提交 e9d953ec4）
——fn 段（`create_vm_from_source`+`call_fn_by_name_segment`+Parked/Resume 骨架）、
comp（`build_dynamic_component`）、E2E（`auto run --render=vm` + pac.at 工程 +
本地 HTTP server，749 T-05 同款）。基面 = master e5a77106a（含 749 G1-G3 条件
求值器修复与 STR_CAT/ADD 显示修复）——待澄清#6 的重取基面前置已落实。

| 断点 | fn 段 | comp 根件 | E2E 完整管线 | 定音 |
|---|---|---|---|---|
| ①条件成员访问 | — | **绿**（CUR:A/NC:B/SEL:A/NSEL:B 三形态全对） | — | 根件面证伪；**唯一在案红=子件挂载面=P733-R2 六红**（workspace_selector 即子件形态） |
| ②续体静态写 | **绿**（静态 W:5/动态 D:6/无 await N:7） | in-proc 不泵 Http 续体（harness 边界：server 已应答而 marker 恒 unset，静态/动态同败——非断点形态） | **绿**（`[p748-e2e] STATIC marker=W:5` + `handler_App_Init resumed to completion (waited 1.06s)`） | **证伪**——dirty 构建陈旧 |
| ③动态 style | — | **绿**（row/checkbox/progress 动态引用全产出枚举载体） | — | **证伪**——静态臂清单与 master 实态不符 |

**结论**：musk PLAN-101 三断点观测（2026-10-09，v0.4.2-2730-g91ae002d3-dirty）
在干净 master e5a77106a 上**全部不复现**（①的根件面/②③ 全部）——其间 749
合并与更早修复已覆盖，或观测本身来自 dirty 构建/WIP。计划按待澄清#3 条款
修订：T-01/T-03 证伪改判（探针钉死为回归面）、T-02 收缩为 P733-R2 子件挂载
面修复（唯一真实在案缺陷面）。T-04 跨仓对拍价值相应收缩为"musk 真实代码
在新构建上运行确认"（回退版三形态已被本探针等价语料覆盖证明）。

### work 阶段性交接（2026-10-10，executing——T-00 完成，T-02/Phase 2 续）

- stage: work | plan_id: PLAN-748 | plan_revision: 4 | outcome: pass（T-00 单元）
- code_commit: plan-748-dev `e9d953ec4`（T-00 探针）；base: master `e5a77106a`
- task_ids: T-00 ✓、T-01 ✓（证伪改判）、T-03 ✓（证伪改判）；T-02/T-04/T-05/
  T-06～T-11 待续
- evidence: §T-00 定音报告（fn/comp/E2E 三层）；探针 7 测（5 绿 + 2 证伪改判
  留档：handler 两臂红 = comp harness in-proc 不泵 Http 续体的 harness 边界
  登记，非引擎缺陷——E2E 层同形态绿）
- blockers: 无
- next: T-02 续腿（p054×2 图标子件渲染面）→ T-04 收缩版跨仓确认 → T-05
  门禁；Phase 2（T-06～T-11 jade 四件）按计划可先行/续后
- T-02 首腿增量（2026-10-10 续）：`.store.X` 泛名展平根修 `2e2164b5e`
  （P733-R2 三红翻绿 p053_1×2+p053_6；p053_4 异根改派；p054×2 续修）

### Phase 2 定音报告（T-06，2026-10-10，worktree lang-748 干净基面 e5a77106a+，jade 供料四件）

**通道**：headless（iced_test 全管线 .at→DynamicComponent→View→into_iced，
layout_tests.rs p748_j1/j2/j3 探针，提交 29df32ec0）+ live（worktree 构建
auto.exe，`auto run -r vm` merged + MCP include_bounds 几何 + PrintWindow
像素测量；jade 真实 app 取 4b398a4^ pre-workaround 副本，不触 jade-edit 本体）。

| 件 | headless 最小形 | live 真 app（pre-workaround） | 定音 |
|---|---|---|---|
| 一 stretch 拉伸 | 绿（视口 224×768 拉伸） | **复现**：视口 Scrollable 675 满高 ✓，内层内容 col 163（=内容高），bg/border 随之止步 | **转写层根因**：needs_scroll 把整串 style clone 给滚动内容 col；`flex-col` 显式显示类阻断 from_style 对 overflow-y 的 height=Fill 推断 → 内容止于内容高。CSS 语义 bg/border 属滚动容器本体（视口）。**StretchLine 无罪**（供料"final 遍 min_h 未兑现"定位证伪——P748-SL dump 实证 final 遍 min_h=768/节点 768 全兑现） |
| 二 显式高×类序 | 绿（三变体几何全等） | **不复现**：首位/末位 h-full 均正常（视口 224×675、子树非零） | 证伪。类序敏感观测无法在 HEAD 复现 |
| 三 定宽挤尾 | 绿（B=512=1024−224−288 精确） | **不复现**：右栏 @rect(812,32,288,675) 满高可见，中栏 588 精确 | 证伪。release 2747 观测与 HEAD 无代码差可归因（stretch_line.rs 逐字节一致、renderer/layout/view_builder diff 均为桌面窗/条件求值面） |
| 四 editor padding | — | 确证缺口（widget.rs 全模块零 padding 载体） | 直接修（T-10） |

**触发类二分（件一）**：jade-real 侧栏 10 类变体逐跑——最小集
（w-56 bg-card overflow-y-auto）在真 app 也绿（from_style 对无显示类的
overflow-y 推断 height=Fill 使内容 col 满高）；加 `flex-col` 即复现（Fill
推断被显示类阻断）。`hidden md:flex` 为 Plan 409 在案设计（桌面窗=md+ 宽度
语义剥前缀，md:flex→flex/by-design，改判非缺口）。

**结论与处置**：件一转写层根修（T-07，视觉类上移视口层——CSS 同构）；
件二/三证伪改判 + 回归锁钉死（防环境性回归）；件四 T-10 根修。件二/三的
release 2747 观测（exp5..13）与 HEAD 构建差异无法归因代码——可能性收敛
release 构建管线（feature 集/profile）或当次实验环境；**由 jade 撤层复验
（AC-14）终局销案**：撤 workaround 后双轨等值 + 右栏可见即四件全闭环。

**副产物**：①plan051_p2 模块级图标测试 corpus 未寻址空洞绿（locate_corpus
三候选 nextest CWD 全 miss→SKIPPED 静默 pass，P748-D1 登记）；②stretch_line
P748_PROBE=1 配给过程 dump（env 门控，常驻）；③iced 0.14 `Limits::(Shrink)`
置 compression 位、Fill 子件在压缩轴解析内容高的实证（42px 复现，视口容器
尺寸镜像修法的依据）。

### work 阶段性交接（2026-10-10 续，executing——Phase 2 主体完成，T-05/T-11 余量）

- stage: work | plan_id: PLAN-748 | plan_revision: 4 | outcome: pass（T-02 续腿 + T-04/T-06..T-10 单元）
- code_commit: plan-748-dev `3ec6a4914`（T-02 续腿图标判别器）/`29df32ec0`
  （T-06 探针）/`4b4a7251e`（T-07 件一根修）/`bb494c762`（T-10 件四根修）；
  base: master `e5a77106a`
- task_ids: T-02 ✓（续腿 p054×2 翻绿，musk_vm_track 102/103）、T-04 ✓（收缩
  版冒烟）、T-06 ✓、T-07 ✓、T-08 ✓（证伪改判+回归锁）、T-09 ✓（证伪改判+
  回归锁）、T-10 ✓；**T-05/T-11 余量**：041/gallery 像素对照 + jade 撤层
  跨仓复验（AC-14，owner=jade 会话）+ 合并收尾门
- evidence: §Phase 2 定音报告（双通道四件判定）；T-07 live 像素终验
  （bg-card rgb(13,20,37) 满高）；T-10 布局级断言（sentinel y=内容高+32）；
  裸 cargo t 5133/5145 零新增红（12 红全在案：批量回执 known_reds_seen/
  REG-4 跨仓脱同步/p053_4/P748 探针 harness 边界×2/plan606/plan707）；
  布局档 65/67（唯二在案存量红）；autodown_editor 127/127（autodown 档）
- blockers: 无阻断；AC-14（jade 撤层复验）依赖 jade 侧时点（§10.4）
- next: review（T-05/T-11 余量随 review/merge 收口：gallery 像素对照 +
  jade 复验回执 + 收尾裸 cargo t 复跑）

### new 阶段交接（草稿准备完成）

- stage: new
- plan_id: PLAN-748
- plan_revision: 1
- outcome: pass
- next: work（待用户确认计划后授权；主检出 WIP 8 文件的 owner 协调
  为 work 前置面，向用户上报）
- changed_tasks: T-00～T-05
- changed_acceptance: AC-01～AC-07
- evidence: 本节§需求分析（双调查代理文件:行号定位 + 101 实机观测 +
  master 单测/预存红实跑）；基线 hash 与 733 先例。
- review_authority: 独立 auto-plan:review 复验定音报告与修复证据。

### new 阶段交接（Phase 2 增补，plan_revision 2，2026-10-10）

- stage: new
- plan_id: PLAN-748
- plan_revision: 2
- outcome: pass
- next: work（Phase 1+2 同 worktree（lang-748）按序执行；两 phase 无
  代码依赖，Phase 2 可先行；主检出 WIP owner 协调为 work 前置面不变）
- changed_tasks: T-06～T-11（新增，total_steps 6→12）
- changed_acceptance: AC-08～AC-15（新增）；AC-01..07 / T-00..05 /
  V01..05 原样保留（Phase 1 既有授权与契约不动）
- evidence: 供料包全文（jade-edit docs/upstream/2026-10-09-vm-layout-
  stretch-supply.md）+ 证据截图 exp5..13 在位核验（2026-10-10）+ 四件
  代码锚点 master 工作区实读（stretch_line.rs 头注/final 遍/探测遍、
  build_row StretchLine 分支、build_scrollable 显式高与 cap 臂、
  axis_fix_col_child、aura_view_builder needs_scroll 转写、autodown_
  editor 模块零 padding 载体、引擎 autodown-editor.css:103——行号区块
  均在主检出在途 WIP hunks 之前，与已提交 master 一致）；jade
  workaround commit 4b398a4（撤层复验锚点）。
- review_authority: 独立 auto-plan:review 复验 Phase 2 定音报告、修复
  证据与 jade 撤层复验回执。

### 边界增补（plan_revision 3，2026-10-10）

- stage: new
- plan_id: PLAN-748
- plan_revision: 3
- outcome: pass（边界澄清，非语义范围变更：Phase 1/2 任务、验收、授权
  原样不动）
- next: 不变（work 待用户确认）
- changed_tasks: 无新增（仅非目标排除项 + 待澄清#6）
- changed_acceptance: 无
- evidence: auto-os 转交的 auto-musk PLAN-100 三型读语义病灶经本仓
  2026-10-10 实读分流（症状3 与断点①同函数不同臂确证异根），立
  PLAN-749 根修——749 §4"与 PLAN-748 的边界判定"五条依据为决策记录。

## 待澄清事项

1. **主检出 WIP owner 协调**（owner=用户/WIP 所属会话）：master 8 文件
   媒体引擎 WIP 与 aura_view_builder.rs/renderer.rs 同文件——748 work
   前需确认 WIP 归属与 rebase 策略（本计划 worktree 独立，日常不冲突；
   收尾 rebase 时点是协调点）。
2. **P733-R2 同根判定**（owner=T-00）：若断点①定音与子件 override
   `.store.X` 预存红同根，修复范围并入 T-02 并改判基线；异根则登记
   边界与后续计划指针（同 733 对 computed 返空面的处理先例）。
3. **dirty 构建污染面**（owner=T-00）：101 的三项观测若部分在干净基面
   不复现（WIP 引入或已修），按实际定音收缩范围——计划允许 T-00 后
   修订任务/验收（revision 递增），不预设结论。
4. **jade 撤层复验时点与 owner**（owner=用户/jade-edit 会话；Phase 2）：
   AC-14 依赖 jade 仓撤两层 workaround（commit 4b398a4）后复验——上游
   修复合入并可被 jade 取用（构建发布或指定 AUTO_EXE）后，jade 侧
   **零改动**撤层复验；若复验失败回投本计划 reopening（669/682 先例
   流程）。auto-lang 侧 T-11 以本仓门禁先行收口，不阻塞 jade 时点。
5. **gallery 像素对照基建确认**（owner=T-06/T-11；Phase 2）：判绿面
   引用 autoui-verifier 既有脚本（test_widgets_gallery_vm.py /
   test_vue_playwright.mjs）；若 stretch 改动波及 gallery 视口 frame
   用例，围栏跑在主检出单实例（重负载机器级闸门，PLAN-726 T-02）。
6. **与 PLAN-749 的同根甄别与执行顺序**（owner=用户/T-00；2026-10-10
   分流裁定增补）：PLAN-749（JSON 产物/深链读语义 + 视图 fn 返空坑①
   家族根修）与本计划存在两条**候选**同根链路——断点①的 bool 条件观测
   可能被 749 症状1（NaN-box bool 位型读）污染；断点②的 SET_FIELD Err
   可能与 749 症状2（JSON 产物物化/解包异常）同链。建议 **749 先行**、
   本计划 T-00 定音在 749 合入后重取基线（若用户裁定本计划先行——含
   Phase 2 可先行不受影响——T-00 定音须显式甄别值层污染，交叉引用 749
   定音报告）；两计划同碰 engine.rs/aura_view_builder.rs，**严禁并行
   worktree 同时改同文件**。
