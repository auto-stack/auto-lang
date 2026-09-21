---
plan_id: PLAN-678
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: rq-default-centering
author: [zcode]
created_at: 2026-09-21
updated_at: 2026-09-21

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 桌面轨窗口根缺省居中放置契约（RqProjector 根入口 + iced 窗口根容器双轨同律）]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/specs/auto-lang/ui/overview.md]
current_step: 0
total_steps: 5
---

# [PLAN-678] rq-default-centering——小 app 内容缺省居中放置（双轨同律）

> 来源：2026-09-21 RQ 实机走查用户裁定。001-helloworld `auto run -r vm -q`
> 首次实机渲染成功后用户观察到内容左上角，裁定"对于小 app，窗口应该让
> 他们默认居中显示"。同会话已落地前置修复 e0e2d6b52（fix-rq-items-center，
> L0）：RqProjector 列臂 items-center 交叉轴居中两遍法——**声明的**居中
> 现在 `-q` 轨生效；本计划补的是**缺省**居中（app 零声明时的宿主行为）。

## 0. 变更摘要

内容尺寸小于视口的轴，宿主缺省居中放置。双轨同律：

1. **RqProjector（`-q` 轨，vm+native 共用）**：`project()` 根布局完成后
   量内容实际尺寸，宽/高任一轴严格小于视口对应轴（扣除 MARGIN）即对该轴
   整体平移 ops/hits/overlays——**纯放置平移，零测量语义改动**（首轮布局
   仍以全视口宽为 avail_w）。
2. **iced 独立轨窗口根**：Plan 504 根容器（`renderer.rs` "常态 Fill×Fill"）
   改造——Shrink 化 + center 包装，使 Shrink 根内容居中、Fill 根行为不变。
   此臂触碰测量语义全局缺省（class.rs:13"窗口根解析为 Fill"），是本计划
   唯一高危面，全 gallery 双轨视觉走查背书。

## 1. 目标

- **G1（-q 轨缺省居中）**：RqProjector 根入口平移通道落地；001-helloworld
  `-q` 实机双轴居中（水平由 e0e2d6b52 items-center 通道与本通道叠加一致）；
  单测钉根入口几何。
- **G2（iced 轨同律）**：窗口根 Shrink+center 改造；002-counter 级小 app
  iced 轨居中；**Fill/h-screen 族零回归**（018/043/画廊 Shell 全走查）。
- **G3（豁免与逃生门）**：嵌入面（AutoFrame/gallery demo-mount/虚拟桌面
  子面）不抢居中——判定面勘定 + 豁免落地；`AUTO_NO_AUTOCENTER` env 逃生
  门（与 AUTO_VM_WINDOW 同型）。
- **G4（保真债不回退）**：两轨居中行为 parity 记入 autoui-verifier 对比
  面；桌面 shell/launcher 等"面"入口天然不经本通道（不同 entry）的边界
  成文入 specs。

**非目标**：不改 Plan 512 裁定（.at 层 `center{}` 退役不复活）；不做
app 级 `center` 关键字恢复；不动 P020-D1 双投影器归一主线。

## 2. 架构方案

### 2.1 RqProjector 根平移通道（低危，先做）

`native_projector.rs` `project()`：root block 布局后，ctx.ops/hits 逐 op
量包围盒（现有 ops 携带坐标）→ `content_w/content_h` → 对满足
`content < viewport - MARGIN*2` 的轴计算 `delta = (avail - content)/2 -
MARGIN`，非零则全部 op/hit 坐标平移。要点：

- **零测量改动**：布局期子级拿到的 avail_w 不变（全视口宽），平移发生在
  产物侧——Plan 512"Fill 把内容测量钉回窗口尺寸"塌缩机制在此不可达。
- hits 必须与 ops 同 delta（命中区一致性）；开态 select overlays 同批。
- 位图引用（bitmap://）/scissor 矩形同步平移。
- **内容满宽/满高时零平移**（`max(0)` 自然豁免 Fill 形态）。

### 2.2 iced 窗口根改造（T-01 勘定后方案坍缩：α/β 二分消解）

**T-01 静态勘定结论（examples/ui 全量根扫描）**：满幅根**全部**携带显式
尺寸类（h-screen×7 / min-h-screen×9 / h-full·w-full×4）；无尺寸类根恰好
= 小 app 目标集（001/005-012/038/条件根 009/019）；max-w-md+mx-auto 自居
中族（003/031-paint）在 Shrink 化后首次获得正确语义（隐式 Fill 下 mx-auto
本就无效）。**因此无需"哪些类算满幅"的判定枚举**：

- 根缺省 Fill → **Shrink** 翻转（iced_adapter 根缺省解析点 + class.rs:13
  注记同步）；显式尺寸类解析**零改动**（h-screen/min-h-screen/h-full/
  w-full 照旧 → Fill 满铺等效现状）。
- Plan 504 根容器（renderer.rs "常态 Fill×Fill"）外层套 **center 包装**：
  Shrink 子自然居中，Fill 子满铺不变（iced container align 语义天然
  分派）。
- 条件根（009/019 if.dark_mode）自动成立——缺省翻转只作用于无类情形。
- 嵌入豁免：embed pre-pass 判定位已存在（iced_adapter.rs:1412"嵌入边界
  内已在 pre-pass 重写为 Fixed"）——嵌入态根不翻转，复用判定。

~~原 α（缺省翻转+满幅类枚举）/ β（条件包装）二分~~——静态扫描证明满幅
集与显式类集重合，两案收敛为上述单方案；T-04 gallery 全量走查仍是硬门。

### 2.3 豁免面勘定（G3）

- 桌面 shell/launcher/虚拟桌面子面：独立 entry（非窗口根入口），天然
  不经两通道——spec 成文即可。
- gallery demo-mount / routes-in-embed：嵌入内容居中会破坏满幅假象——
  iced 臂走 embed 判定豁免；RqProjector 臂勘定嵌入态入口（663 视口
  重锚定位点）后同豁免。
- env 逃生门 `AUTO_NO_AUTOCENTER=1`：双轨同认，排障用。

## 3. 需求分析与背景调查

- 用户裁定原文（2026-09-21）："对于小 app，窗口应该让他们默认居中显示。"
- 现状：两轨窗口根均无缺省居中。`-q` 轨根入口 `NodeStyle::default()` +
  `(MARGIN, MARGIN)` 起排（native_projector.rs:499-508）；iced 轨根容器
  Fill×Fill + Shrink 根左贴。001-helloworld 修复 e0e2d6b52 后仅水平居中
  （声明驱动），垂直仍顶贴。
- Plan 512 先例：.at 层 center 外壳退役，理由"Fill 会把内容测量钉回窗口
  尺寸"——本计划放置平移方案正面规避该机制，属其精神延续而非回退。
- 关联债：P020-D1（双投影器归一——本计划两通道同律落地即缩短差距面）；
  画廊塌缩债（gallery-vm-embed-layout-collapse，663 D 系）——iced 臂
  设计直接对峙此坑，T-02 探针为硬前置。
- spec 面：`docs/specs/auto-lang/ui/overview.md`（渲染缺省行为）、
  desktop-protocol-v1.md（宿主窗口放置语义 v1.x 增补）。

## 4. 详细设计

### 4.1 RqProjector 根平移通道（T-01a 勘定定案）

- **触发点**：`RqProjector::render_frame`（native_projector.rs）。根块
  `layout_view_block` 返回值（现 `let _ =` 丢弃）取 `Laid.size` =
  内容自然尺寸——**零 op 遍历测量**（overlays/select 弹层锚定内容坐标，
  随同平移天然一致）。
- **delta 计算**：逐轴 `avail = viewport − MARGIN×2`；`content < avail`
  时 `delta = (avail − content)/2`（内容起点 MARGIN 平移到居中位）；
  `content ≥ avail` 轴 delta=0。`AUTO_NO_AUTOCENTER=1` 整通道旁路。
- **平移集**（choke 点 = `self.hits = ctx.hits` 赋值前一次性完成）：
  `ctx.ops`（Quad/Scissor/Image 的 rect、Text/TextStyled 的 x/y）、
  `ctx.hits`（HitEntry 全变体携 rect：Msg/Input/Slider/SelectBox/
  SelectOption/Scroll/PopoverDismiss/…）、`ctx.right_hits`。
  **零坐标面**：BitmapUpload（id/w/h/stride/rgba——位置在 Image op 的
  rect 里）、frame.clear（全窗 BG）。
- **视口锚定豁免**：`rect.w ≥ width && rect.h ≥ height` 的 WRect 不平移
  （popover modal 全窗 scrim + PopoverDismiss 全窗命中——平移会撕开
  覆盖）。逐 op 判定，混合帧（全窗 scrim + 内容面板）语义正确。
- **Scroll 元组**：offset/viewport/content 是尺寸非坐标——只平移 rect。
- **输入槽位**：focused_input/select_open 为槽序身份，与坐标无关；
  Input 命中 rect 随 hits 平移，聚焦点击定位自然一致。

### 4.2 iced 窗口根（见 §2.2 坍缩后单方案）

改点两处：根缺省解析（Fill→Shrink）+ Plan 504 根容器 center 包装；
显式类零改动；embed pre-pass 判定位豁免嵌入态。

### 4.3 逃生门与边界

- `AUTO_NO_AUTOCENTER=1`：双轨同认（RqProjector 通道旁路 + iced 包装
  不套）。
- 桌面 shell/launcher/虚拟桌面子面：独立 entry 不经窗口根，天然豁免
  （spec 成文）。

## 5. 测试设计

- 单测（RqProjector）：小内容双轴平移几何钉（中心=视口中心 ±1px）；
  满宽内容零平移；`AUTO_NO_AUTOCENTER` 关闭通道；hits 与 ops delta 一致
  （点选命中坐标平移后仍中）。
- 单测（iced）：根容器形态断言（Shrink 探针 + center 包装存在位）。
- 视觉走查：gallery 全量双轨截图对比矩阵（001/002/012/018/020/043 +
  Shell 三画廊框架），Fill/h-screen 族零漂移为硬门。
- e2e：`auto run -r vm -q` 001 实机居中走查（autoui-verifier 双轨）。

## 6. 验收标准

- [ ] AC-01 001-helloworld `-q` 轨内容双轴居中（实机截图 + 几何断言）。
- [ ] AC-02 002-counter iced 轨内容居中，且 `-q` 轨同形态（两轨 parity）。
- [ ] AC-03 gallery 矩阵：Fill/h-screen/嵌入面零回归（截图对比在档）。
- [ ] AC-04 `AUTO_NO_AUTOCENTER=1` 双轨逃生门生效。
- [ ] AC-05 覆盖门/成熟度门零回退（desktop_protocol 作用域 + 裸
      `cargo t` 零新增红）。

## 7. 执行步骤

- [ ] T-01 勘定：根平移通道实现点 + iced 方案探针，产出裁决档回填 §4。
      [✅ 已完成] 2026-09-21——§4.1 通道定案（Laid.size 测量/hit 全变体
      rect 平移/视口锚定豁免/BitmapUpload 零坐标实证）+ §2.2 方案坍缩
      （examples 根全扫：满幅集=显式类集，α/β 收敛单方案；embed pre-pass
      判定位 iced_adapter.rs:1412 复用）。
- [ ] T-02 RqProjector 根平移通道 + 单测四件（§5）。
- [ ] T-03 iced 窗口根改造（按 §2.2 单方案）+ embed 豁免 + 单测。
- [ ] T-04 env 逃生门双轨接线 + gallery 全量走查矩阵（AC-03）。
- [ ] T-05 复审收口：specs 沉淀（ui/overview + desktop-protocol-v1 增补）、
      债册对账（P020-D1 差距面缩短注记）。

## 8. 复审记录

（复审时填写）

## 9. 待澄清事项

1. ~~方案 α/β 裁决~~——T-01 静态勘定消解（满幅集=显式类集，收敛单方案，
   见 §2.2）；gallery 实测走查（T-04）仍是回归硬门。
2. `-q` 轨平移是否覆盖 launcher/shell 之外的"面"类宿主入口——勘定结论：
   shell/launcher 等走独立 entry 不经 `render_frame` 窗口根通道，天然
   豁免（T-05 spec 成文确认）。
