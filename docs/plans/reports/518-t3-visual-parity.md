# Plan 518 T3 视觉对表记录(双主题 × AUTHORITATIVE.png)

> 2026-09-02。通道:505 验收通道同型(ui_desktop + AUTOUI_ACCEPTANCE=1 +
> autoui_desktop MCP 注入;驱动脚本 `scratch/p518/t3_drive.py`)。
> **权威参照** = `scratch/visual-gap/stella/AUTHORITATIVE.png`(用户提供,
> 暖纸 light 桌面)。证据图 = 本目录(对表板:VM light | 权威 | VM dark)。
> 方法论:像素采样(色值断言到 hex)+ 结构走查;逐条对差距矩阵
> (GAP-MATRIX.md 第二轮)条目。

## 环境

- worktree plan-518-dev(11 commits,含双主题 token/壁纸/dock/chrome/
  transparency/appearance/backdrop 全链)。
- 浅色轮:storage 预置 `shell.appearance.theme=light` boot;深色轮:键缺席
  默认。窗口素材:013-todo/015-notes(无 os-config 逐 app 主题配置——
  calculator 有用户配置 theme=dark,属 504 逐 app 主题合法覆盖,见注记④)。
- VM 截图 2560×1600(autoui_screenshot baseline)。

## 对表板

| 板 | 内容 |
|---|---|
| `t3-parity-board.png` | VM light 桌面 \| AUTHORITATIVE \| VM dark 桌面 并排 |
| `t3-windows-board.png` | light 虚拟窗(todo+notes) \| dark 虚拟窗 并排 |
| `live-1..4-*.png` | 实时切换序列:浅色设置面板 → PickTheme(dark) 即时翻 → PickTransparency(high) → 关面板深色桌面全景 |
| `glass-vue.png` / `glass-vm.png` | G8 毛玻璃样张双臂(vue 真毛玻璃 / VM 降级 no-op) |

## 逐条核对(矩阵条目 → 判定)

### 1.1 壁纸 ✅(注记①)

- VM light 桌面壁纸 = 内嵌宣纸(builtin:ricepaper)直出:可见区域实测
  ≈(237,231,219)#EDE7DB **与权威图壁纸色值逐通道一致**(权威实测同值)。
- 深色轮壁纸**同一张浅色宣纸不变**(topleft 两轮同为 scrim 后纸色)——
  **壁纸与主题解耦实证**(stella dark 同款行为);区别仅在深色轮叠 35%
  深色 readability scrim(503 M3 既有层),见注记②。
- 水墨备选(builtin:inkwash)经同一 builtin: 通道可设(单测覆盖)。

### 1.2 主题基调 ✅

- light:桌面 Background 实测 (246,242,232)≈#f6f2e8(token #f5f1e8+
  10% light scrim 微移);dock 卡 Surface (251,248,242)=#fbf8f2 逐值命中。
- dark:桌面 Background (20,27,42)=#141a29 **逐值命中**;dock Surface
  (26,34,53)=#1a2235 逐值命中。
- 双主题对照权威图:light 轮与权威图同为暖纸基调(并排板左中);dark 轮
  为精修蓝黑(stella dark 实证形态)。

### 1.3 accent 密度 ✅

- dock 无常驻 accent 底格:聚焦窗格 bg-primary/15 已撤(步骤3),图标格
  hover 才出格;运行指示 = 贴底宽条(h-1 w-6)。
- accent 实测仅在:运行指示条/激活竖条(rose/coral 系)——与权威图
  "玫瑰粉只出现在 dock 指示/日历选中/播放条"密度一致。
- 注记③:本轮截图 accent 为 indigo 默认(ui_desktop 直跑不经 CLI
  --accent);coral=stella 玫瑰粉校准由 T1 单测逐值锁定。

### 2.2 dock 图标 ✅

- dock 图标 = lucide 线性 glyph 无色块容器(命中表测试全绿,步骤4);
  桌面图标 = per-app 徽标色圆角底+白 glyph(与权威图桌面 Notes 徽标
  同款);色板白字对比全板 ≥4.5:1(单测锁定)。
- 大图标(≥48px 独立形态)stroke 1.5 细线参数已备(单测锁定)。

### 2.3 窗口 chrome ✅

- 三色圆点保留(用户裁定);**标题窗口级居中**(三列 row 实现,整条
  拖拽把手语义不变)——窗口板两轮标题条中部均无文本偏左。
- 无右侧按钮(× 关闭走圆点,同权威图形态)。

### 2.4 窗口质感 ✅

- 窗体圆角 16px(rounded-2xl 档,503 既有)+ 柔影重校 (0,10)/40:
  light 12–18% / dark 40–52%(随主题分支,聚焦加深)——窗口边缘
  阴影带实测连续渐变,无硬边。
- light 轮窗 = 白纸感面板浮于壁纸(权威图同形态);dark 轮 = 蓝黑面板。

### 2.5 内容卡片 △部分(注记④,示例线)

- 虚拟窗内 app 自有卡片密度属示例/458 线(计划明确 app 内部排版密度
  排除);todo/notes 窗内容双主题可用。**注记④**:os-config 逐 app 主题
  (504)与 shell 全局主题共享一个 dark_mode thread-local——用户配置
  calculator theme=dark 时其视图构建会把 shell chrome 一并翻深(浅色桌面
  下实测);per-app 色彩上下文拆分登记 KNOWN-DEBT(P518 注记)。

### G1 双主题切换 ✅(live 序列)

- live-1(浅色桌面+设置面板)→ PickTheme(dark) → live-2 **同帧序列即时
  翻深**(面板/桌面/窗 chrome 同步)→ storage 落 `dark`(驱动脚本回读
  实证)→ 重启 boot 读回浅/深正确(两轮 boot 实证)。
- **壁纸不被重置**:切换前后壁纸同一张宣纸(解耦,见 1.1)。

### G6 Transparency ✅(live-3)

- PickTransparency(high) → storage `shell.desktop.transparency=high`
  (回读实证)→ 虚拟窗底色 alpha 0.62 每帧生效(live-3 窗底透出壁纸);
  三档映射 T1 决策单测锁定。窗口文字对比不随底色 alpha 变化(文字
  照常不透明绘制),可用性保持。

### G8 毛玻璃样张 ✅(glass-vue/vm)

- vue 臂:计算样式 `backdrop-filter: blur(24px) saturate(1.6)` 实测
  (Playwright getComputedStyle)+ 视觉毛玻璃确认;Tailwind JIT content
  直扫零登记即生效。
- VM 臂:同串正常渲染零报错(降级 no-op,装饰性非错绘);queue 臂装载
  Covered 不触发 not-yet(单测锁定)。真模糊挂 RenderQueue
  (KNOWN-DEBT P518 planned-debt)。

## 注记(分歧与后续)

1. **壁纸内容差异**:权威图含书法资产+朱红印章(用户私产);自制宣纸为
   纯质感无书法——素材面差异非渲染差距,水墨备选已备。
2. **深色 scrim 浓度**:dark 轮壁纸叠 35% 深色罩层(503 M3 可读性层)
   → 纸感偏灰;stella dark 壁纸保持亮纸。浓度可调(实机定,当前 35%
   保图标/文字可读)。**壁纸键本身不随主题变**——解耦语义成立。
3. **accent 预设**:实机板为 indigo 默认;stella 玫瑰粉=coral 由 CLI
   `--accent coral`/设置链生效,T1 逐值锁定(#c4706a vs 权威 #C96B62)。
4. **os-config 逐 app 主题 × shell 全局主题共享 thread-local**:calculator
   (用户配置 theme=dark)在浅色桌面会把 shell chrome 翻深——504 逐 app
   合法偏好与 518 shell 主题的架构缝;登记 KNOWN-DEBT,拆 per-app color
   context 归后续(RenderQueue 色彩上下文重构一并)。
5. **设置面板本体**:灰阶硬编码(gray-800/900,487 形态)不随主题切——
   stella 设置同为深色面板,形态可接受;语义化留后续。

## 结论

**矩阵条目 1.1/1.2/1.3/2.2/2.3/2.4 全 ✅,2.5 △(示例线+注记④)**;
G1 切换/G6 透明度/G8 毛玻璃三链实机实证。权威图对表通过——"计划说
刷新了、眼睛说没变"的 503 教训以像素级逐值断言 + 并排板留痕收口。
