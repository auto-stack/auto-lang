---
plan_id: PLAN-621
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: autoui-icon-state-contract
author: [zhaopuming]
created_at: 2026-09-14
updated_at: 2026-09-14

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [ui/icon-state, ui/icon-library-policy]
touched_goals: []

affects: [auto-lang/ui]
current_step: 0
total_steps: 8
---

# [PLAN-621] autoui-icon-state-contract

## 变更摘要

给 `icon` 元素建立**开关状态契约**：新增 `state` prop（`on`/`off`，字面量或 bool 绑定），
on → `primary` 语义色、off → `muted-foreground` 语义色（双双落在既有 shadcn 双盘 token 上，
深色/浅色主题自动正确），激活态同时把描边加重 +0.5（lucide 描边技法独有的非颜色线索）。
同时把 2026-09-14 图库选型调研的裁定沉淀为设计注记与规范条目：
**lucide 保持主力、Remix/IconifyJSON 作为补位管线（记录 playbook，不在本计划实施）**。

三件互相咬合的事：

1. **`icon` 的 `state` 双端契约**（schema + VM + Web），优先级口径沿用 620 的
   「显式作者意图 > 平台机制 > 默认」。
2. **激活态描边加重**：VM 侧 `lucide_svg_doc_with(name, stroke_width)` 已是参数化注入点
   （renderer.rs，默认 2.0、≥48px 档 1.5），Web 侧 lucide-vue-next 组件原生接受
   `stroke-width`。
3. **图库策略成文**：颜色模拟 line/fill 的裁定、Remix 不换血的理由、补位 playbook
   （IconifyJSON 统一管线，按需引入）。

## 目标

- **G1**：作者用一个 `state` 属性就能表达图标开关态，双端渲染**由构造成立地一致**
  （同 token、同优先级、同联动），深色/浅色主题下各自自动正确。
- **G2**：状态表达不依赖色相（亮度型 on/off + 描边加重线索），对色盲用户与黑白
   场景可辨；「关」态用固定 dim token 保底可见性，不做字面近背景色。
- **G3**：未声明 `state` 的 `icon` 输出**逐字节不变**（既有金样零扰动）。
- **G4**：图库选型裁定与补位 playbook 成文进规范/设计注记，防止后续重新翻案或
  无序混入第二图库。

**非目标**：不换主力图库（Remix 迁移否决，理由见设计注记）；不引入 duotone/多色
图标着色契约；不动 `button`/`nav-item` 内嵌 icon 的状态表达（其 active 语义已独立
存在）；不实装 IconifyJSON 补位管线（只写 playbook）。

## 架构方案

### 2.1 state 契约（G1/G2）

```
作者面:  icon (name: "bell", state: "on")        ← 字面量
         icon (name: "heart", state: .liked)     ← bool 绑定（响应式）

展开面（两端同规则，等价于注入两个样式类）:
  state=on   → text-primary + 描边 = 基档 + 0.5
  state=off  → text-muted-foreground + 描边 = 基档
  （未声明 state → 不注入任何东西，输出与今天逐字节一致）

优先级:  显式 text-* 类 > state > 继承/默认
         （镜像 620 的尺寸口径：显式作者意图不被平台机制静默覆盖）
```

token 落点（**全部复用既有基建，零新增颜色设施**）：

| 语义 | token | VM 侧 | Web 侧 |
|---|---|---|---|
| on | `primary` | `Color::Primary`（**运行时 accent 预设驱动**，见下） | `text-primary`（`--primary`，dark 变体自动） |
| off | `muted-foreground` | `Color::OnSurface`（style/color.rs:123，双盘 dim） | `text-muted-foreground` |

**`primary` 与 `accent` 的关系（2026-09-14 用户问询后实勘定案）**：仓库里
`Color::Primary` 的色相**由当前 accent 预设运行时驱动**——`theme/mod.rs::resolve_semantic_rgb`
对 Primary 有专门臂，查 `design_tokens/registry.rs::accent_hsl`（indigo/coral/ocean/
sage/amber 五预设，dark 下 L+10 双端统一，PLAN-601 D2）。即「primary = accent 预设
的主色」，正是预期语义。而 token `accent`（`Color::Accent` / shadcn `--accent`）是
**另一个东西**：交互高亮 surface（hover 浅色底，scaffold light = hsl(210 40% 96.1%)
近白），不是强调色族——Plan 601 T-08 从 Secondary 拆出的独立 role。故 state:on
用 `primary` 是唯一正确落点，`accent` token 语义不符（近白不可辨）。

「关」态选 `muted-foreground` 而非字面近背景色/`opacity-40`：语义 token 双盘自动、
可见性有保底（shadcn 惯例的 dim 灰），且亮度型区分对色盲安全（裁定记录进设计注记）。

### 2.2 激活态描边加重（G2 的非颜色线索）

- **VM**：`ui/iced/renderer.rs::lucide_svg_doc_with(name, stroke_width)` 已参数化
  （默认 2.0；≥48px 档 1.5）。激活态 = 基档 + 0.5（即 2.5 / 2.0），组合而非覆盖。
- **Web**：lucide-vue-next 组件接受 `:stroke-width`；vue.rs icon 臂在 state=on 时
  发射该 attr（需先核实现有 ≥48px 细线档在 Web 侧的落点，保持同一组合规则）。
- 传递通道：VM 侧新增 `StyleClass::StrokeWidth(f32)`（style 管线内流转，renderer
  lucide 路径读取）；**不动 `View::Image` 协议结构**（避免 620 §4.2 警示的跨后端
  协议类型全构造点改造）。Web 侧 vue.rs 臂直接读 props，无需经 style 类。

### 2.3 图库策略（G4，文档性）

裁定（2026-09-14 调研，数据见设计注记）：**lucide 主力不换血**。
- Remix line/fill 与 lucide 同名交集为 0（后缀结构 + 语义改名），全量迁移 =
  永久别名映射表 + 全金样重基线，而为的只是 5% 场景（heart/star 真填充 + 品牌图标）。
- 颜色 + 描边加重覆盖「开关/启用」类状态语义；「收藏/评分」类真填充需求与品牌
  图标走**补位 playbook**：IconifyJSON 统一格式（`@iconify-json/*` 锁版本、
  `set:name` 双段名），按需引入，实体管线不在本计划范围。

## 需求分析与背景调查

### 4.1 授权记录

- 2026-09-14 用户在 PLAN-620 图标调研对话中确认方向 B（留 lucide + state 机制，
  Remix 降级补位），并指示「根据你的综合建议来规划」。范围：本仓 auto-lang/ui；
  无特殊预算/限额。

### 4.2 实勘证据（2026-09-14，master @ 557a81547）

| # | 事实 | 出处 |
|---|---|---|
| P-1 | `icon` element schema 仅有 `name`/`class`/`size` 三 prop；无 state；`backends.iced` 标 "unknown"（620 已实装 native，stale） | `schema/aura.at:713` |
| P-2 | schema `size` 标 `default: "24"`，实际双端共享默认 20px（620 定案） | `schema/aura.at:719` vs `aura_view_builder.rs::with_icon_size` |
| P-3 | VM icon 臂：`name` → `View::Image { src: "lucide:{name}" }` + style 类 | `aura_view_builder.rs:6269-6274` |
| P-4 | shadcn 语义 token 双盘已在：`primary`→`Color::Primary`、`muted-foreground`→`Color::OnSurface`、`accent`→`Color::Accent`（Plan 601） | `style/color.rs:105-128` |
| P-5 | `dark:` 主题门控已在（Plan 527 T8）；`theme::dark_mode()` | `style/mod.rs:22,113`、`style/theme/mod.rs:140` |
| P-6 | `StyleClass::Opacity(u8)` 存在但本计划不用（裁定走 token） | `style/class.rs:413` |
| P-7 | VM 描边注入点已参数化：`lucide_svg_doc_with(name, stroke_width)`，默认 2.0、≥48px 档 1.5（PLAN-619 §8.5 ink 回归锚在案） | `ui/iced/renderer.rs:5566-5590` |
| P-8 | 尺寸口径先例：显式 `w-*`/`h-*` > `size` > 默认 20px，两端同实现（620 G2） | `aura_view_builder.rs:6290`、`ui_gen/vue.rs` icon 臂 |
| P-9 | `View::Image` 是跨后端协议类型，加字段需触全部构造/映射点（620 §4.2 钉死） | `ui/view.rs` |
| P-10 | 全仓示例 + 语料实际在用图标名仅 29 个（examples 23 + corpus/schema 7，table 重叠 1）——`state` 的波及面评估基数 | 2026-09-14 grep 实测 |
| P-11 | `convert_video` 已有 `extract_bool_expr`（bool 绑定求值先于臂内处理）先例可循 | `aura_view_builder.rs` PLAN-617 T-19 |

### 4.3 调研结论（2026-09-14，详见设计注记）

- lucide 官方不支持 fill 变体（纯描边哲学，无 roadmap）→「满/空」走颜色+描边模拟。
- Remix 3244 图标 = 1541 line + 1541 fill 完美配对、24×24；但与 lucide 命名交集 0，
  语义改名录（clock→time、copy→file-copy、undo-2→arrow-go-back 等）+ 反向缺名
  （bomb）→ 不换血。
- 亮度型 on/off（显著 vs dim）对色盲安全；色相型（红/绿）不安全 → 本契约用亮度型。

## 详细设计

### 5.1 文件清单

```
schema/aura.at                                        改：icon element 增 state prop；
                                                        backends.iced unknown→native；
                                                        size default 注记 20px（P-2 hygiene）
crates/auto-lang/src/ui/style/class.rs                改：新增 StyleClass::StrokeWidth(f32)
                                                        + 解析/合并
crates/auto-lang/src/ui/aura_view_builder.rs          改：convert_image_or_icon 增 state 臂
                                                        （with_state_tint，镜像 with_icon_size）
crates/auto-lang/src/ui/iced/renderer.rs              改：lucide 路径读 StrokeWidth 类，
                                                        与尺寸档组合（基档 + 0.5 已由臂内算好）
crates/auto-lang/src/ui_gen/vue.rs                    改：icon 臂读 state（字面量 → class + attr；
                                                        绑定 → :class/:stroke-width 三元）
crates/auto-lang/test/a2vue/…/expected.vue            改：新 state 金样（新增，不改旧金样）
test/ 语料 .at                                        新：state 字面量 + 绑定两语料
docs/design/autoui/icon-state-and-library-policy.md   新：设计注记（调研数据 + 裁定 + playbook）
docs/design/00-intro.md                               改：登记新注记
docs/specs/auto-lang/ui/overview.md                   改：SD-01/SD-02 两条契约
docs/components/core.md                               改：docs_gen 重生成（schema 触发）
```

### 5.2 关键实现约束

1. **state 未声明 = 零扰动**（G3）：臂内仅在 `state` prop 存在时注入；既有金样
   不得有任何 diff（测试反断言）。
2. **优先级判定取已解析类集**（镜像 620 的坑）：显式 `text-*`（`StyleClass::Text`
   已在类集）→ state 不注入颜色；描边同理不覆盖显式值。
3. **绑定语义**：`state: .flag` 求值走 `extract_bool_expr` 先例（P-11）；VM 端
   随 View 重建自然响应；Web 端发射 `:class`/`:stroke-width` 三元绑定。
4. **StrokeWidth 不进 `View` 协议**（P-9）：走 StyleClass 管线；`style/class.rs`
   合并规则 = 显式设置胜出，臂内计算好的绝对值（2.5/2.0）入类，renderer 不再做
   加法（组合逻辑单点化在 builder）。
5. **schema 的 docs_gen 联动**（Category C 门禁）：schema 改动必须重生成
   `docs/components/core.md` 并跑 `cargo test -p auto-lang --test docs_gen`。

### 规范增量

| delta_id | 类型 | 目标 | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-lang/ui/overview.md`（icon 契约节） | before: icon 仅 name/class/size 口径（620 T-05 待落）；after: 增 state 语义——on→primary / off→muted-foreground、显式 text-* 优先、on 态描边 +0.5、未声明零扰动 | 开关态成为平台原语，双端由构造一致 | AC-01/02/04 |
| SD-02 | add | `docs/specs/auto-lang/ui/overview.md`（图库策略注记行） | before: 无图库策略记载；after: lucide 主力 + Remix/IconifyJSON 补位 playbook 指针（指向设计注记） | 防止图库翻案与无序混排 | AC-06 |

## 测试设计

| 层 | 手段 | 覆盖 |
|---|---|---|
| builder 单测 | `with_state_tint` 三态（on/off/未声明）+ 优先级（显式 text-* 在场）| G1/G3 |
| 绑定单测 | `state: .flag` 求值 → 双态类集切换 | G1 |
| style 单测 | `StrokeWidth` 解析/合并/显式胜出 | G2 |
| renderer 单测 | lucide doc stroke-width 组合（2.0→2.5；1.5→2.0），回归锚 `plan619_lucide_doc_renders_geometric_ink` 保持绿 | G2 |
| 金样 | 新增 state 语料双端 golden（a2vue + vm corpus）；**既有金样零 diff 反断言** | G3 |
| docs_gen | schema 重生成后 `cargo test -p auto-lang --test docs_gen` | schema 一致 |
| 双端 e2e | 030 smoke 增一断言：state 绑定翻转 → 双端 class/着色同步（autoui-verifier 双端跑） | G1 端到端 |
| 全局回归 | `cargo t`（与基线红逐项对拍）+ 触 VM 语料后 `cargo tv` | 零新增红 |

## 验收标准

- **AC-01 state 双端一致**：`icon (name: "bell", state: "on")` 双端渲染 primary 色、
  off 渲染 muted-foreground dim；深/浅主题下各自正确（VM 双盘 token、Web CSS 变量）。
  验证：新金样 + e2e 截图对拍。
- **AC-02 优先级与零扰动**：显式 `text-*` 类在场时 state 不覆盖颜色；未声明 state 的
  `icon` 输出与改动前逐字节一致。验证：单测反断言 + 既有金样零 diff。
- **AC-03 绑定响应式**：`state: .liked` 翻转模型 bool → 双端着色/描边同步变化。
  验证：单测 + e2e。
- **AC-04 描边加重**：on 态描边 = 基档 + 0.5（2.0→2.5；≥48px 档 1.5→2.0），双端同值；
  PLAN-619 ink 回归锚保持绿。验证：renderer 单测 + 金样。
- **AC-05 零新增回归**：`cargo t` 相对基线零新增红；`docs_gen` 绿。验证：对拍表。
- **AC-06 策略成文**：设计注记落地并登记 `00-intro.md`；SD-01/SD-02 写入
  ui/overview.md；`python scripts/spec-index.py` 无漂移。验证：文件存在 + 索引检查。

## 执行步骤

- [ ] **T-01 schema**：`schema/aura.at` icon element 增 `state`（`one_of:on,off` +
      binding 说明）；修 `backends.iced: native`；size default 注记改 20px 对齐
      P-2。重生成 `docs/components/core.md`。验证：`cargo test -p auto-lang
      --test docs_gen` 绿。（→AC-05/06 基础）
- [ ] **T-02 VM 着色臂**：`style/class.rs` 增 `StrokeWidth`；`aura_view_builder.rs`
      增 `with_state_tint`（镜像 `with_icon_size` 结构：解析类集判定优先级、
      `extract_bool_expr` 支持绑定、注入 `Text(Primary/OnSurface)` + `StrokeWidth`）。
      单测覆盖三态 + 优先级 + 绑定。验证：`cargo check -p auto-lang` +
      `cargo t with_state_tint`。（→AC-01/02/03）
- [ ] **T-03 VM 描边**：`renderer.rs` lucide 路径读 `StrokeWidth` 类替代写死基档；
      `plan619_lucide_doc_renders_geometric_ink` 回归锚 + 新增组合单测。
      验证：`cargo t plan619_lucide` 绿。（→AC-04）
- [ ] **T-04 Web 臂**：`ui_gen/vue.rs` icon 臂支持 state（字面量 → class +
      `stroke-width` attr；绑定 → 三元）；先核实 Web 侧 ≥48px 细线档落点保证
      组合规则两端同式。验证：`cargo check` + 新增生成器单测。（→AC-01/04）
- [ ] **T-05 语料与金样**：新增 state 语料（字面量 + 绑定各一）；生成双端金样；
      跑既有金样零 diff 反断言。验证：`cargo tv` 新旧全绿。（→AC-01/02/03/04）
- [ ] **T-06 设计注记**：新增 `docs/design/autoui/icon-state-and-library-policy.md`
      （调研数据表、裁定理由、补位 playbook），登记 `00-intro.md`。（→AC-06）
- [ ] **T-07 规范增量**：SD-01/SD-02 落 `docs/specs/auto-lang/ui/overview.md`；
      `python scripts/spec-index.py`；ledger `ui/plans.md` 加 621 行。（→AC-06）
- [ ] **T-08 双端 e2e + 复审**：030 smoke 增 state 翻转断言，autoui-verifier 双端
      跑；`cargo t` 对拍基线；按 `/auto-plan:review` 逐条对拍 AC。（→AC-01/03/05）

## 分支与提交归属

- 常规：`git worktree add D:/autostack/.wt/lang-621/auto-lang -b plan-621-dev`
  （Plan 529 布局；创建前过 `wt-guard.sh`）。
- 与 PLAN-620 的顺序依赖：两计划的 `ui/overview.md` 规范增量条目**不相交**
  （620：尺寸口径/数据源；621：state 契约/图库策略），但同文件先后写避免冲突
  ——默认 620 T-05 先落、621 后落；若 620 长期挂起，621 可先行（条目互不触碰
  对方段落）。复审时确认。

## 复审记录

- 2026-09-14 draft handoff（/auto-plan:new）：`stage: new`，`outcome: pass`——
  任务覆盖全部 AC 与 SD；路径/命令均经 master 实勘（§4.2）；无阻塞决策，
  待用户确认后 `next: work`。

## 待澄清事项

1. ~~**on 态主色选 `primary` 还是 `accent`**~~ **已裁定（2026-09-14，用户问询触发
   实勘）**：用 `primary`。实勘定案：`Color::Primary` 运行时由 accent 预设驱动
  （`resolve_semantic_rgb` Primary 专臂 → `registry::accent_hsl`），即「accent 预设
   的主色」；token `accent` 是 shadcn hover 高亮面（scaffold light 近白 96.1%
   亮度），用于 state:on 会不可辨。详表见 §2.1。
2. **off 态 token**：`muted-foreground`（裁定倾向）vs `opacity-40`。前者双盘
   token 化更系统；若作者侧已有 text 色继承场景的兼容问题，T-02 实勘后回退到
   opacity 方案（需两端同值）。
3. **描边加重默认开还是关**：现按默认开（+0.5 是 lucide 描边技法的免费线索）；
   若金样观感过重，可降为一档常量或砍掉（T-03 独立成任务就是为了可单独否决）。
4. **与 620 T-05 的落地顺序**：见「分支与提交归属」默认裁定，复审时确认。
