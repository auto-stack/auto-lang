---
plan_id: PLAN-637
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: style-recipe-token-rollout（examples/ui 全量配方化 + token 化）
author: [zhaopuming]
created_at: 2026-09-17
updated_at: 2026-09-17
plan_revision: 3

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/overview.md（示例配方化规范条目：stylekit 共享库 + 禁新增裸调色板色门禁）
touched_goals: ["GOAL-007: AutoUI 跨端视觉一致（样式配方/令牌抽象）"]

affects: [auto-lang/ui, autoui-examples]
current_step: 5
total_steps: 10
---

# [PLAN-637] style-recipe-token-rollout——examples/ui 全量 Style Recipe / Design Token 应用

## 0. 变更摘要

PLAN-635 交付跨包配方引用机制后，消费侧成为唯一缺口：examples/ui/ 36 个
demo 中仅 3 个配方化（013/015 为 607 试点、045 为 635 示范），裸 `style: "…"`
属性点 **2069 处**，重复长串严重（`text-[10px] text-muted-foreground` ×76、
`text-xs text-muted-foreground` ×58、`w-4 h-4 shrink-0 text-muted-foreground`
×56），硬编码调色板色集中在 020-music-player（264）/023-realworld（119）/
011-calculator（112）/019-video-app（107）。

本计划把 Style Recipe + Design Token 能力铺满全部 demo：

1. **T-00 stylekit 共享配方库扩容**：把跨 demo 重复模式沉淀为 canonical
   recipes（`hint_text`/`caption_text`/`icon_base`/`input_field`/`card_*`/
   `pill` 族——635 机制的真实消费），demo 经 `dep stylekit` + `use` 导入。
2. **Phase A 逐 demo 去重（零视觉漂移）**：重复 class 串提取为配方（共享
   归 stylekit、demo 专属留本地），展开后 class 串逐字节等价，双端截图
   before/after 零漂移。
3. **Phase B 逐 demo token 化（有意的视觉变更）**：硬编码调色板色 → 语义
   token（如 `text-gray-400 → text-muted-foreground`、`bg-blue-500 →
   bg-primary`），映射表逐 demo 定案，接受受控的视觉变化并留 before/after
   凭据；完成一个 demo 即纳入「禁新增裸调色板色」grep 门禁。

排序从最简单 demo 开始（001-005 起步），重型（020/023/011/019）最后。

> **r2 修订（2026-09-17 计划再评审，8 项）**：①025/038 无 `.at` 源码，
> 矩阵登记 N/A（G1 计数勘正 35→33+045）；②无色号字面量 `text-white`/`bg-white`
> 入 Phase B 作用域，`bg-black` 遮罩与渐变 `from-/to-` 入豁免表；③D4 豁免
> 口径对齐待澄清#1 裁定（三类）；④AC-05 增 W1 不过度抽象注记；⑤D1 增
> 变体纪律（Phase A 不跨串合并）；⑥tree_icon widget 四胞胎合并另立小计划；
> ⑦新增 T-01b——006/010/016 settings 消费面移除（用户裁定：demo 跟随系统）；
> ⑧T-08 增 worktree junction 拆除步骤（wt-guard 红线）。
>
> **r3 增补（2026-09-17）**：执行批次定为 5 个 fold 点（B1-B5，见 D2b，
> 按 33 demo 全量实测工作量配平，总计 ~3070 位点）；D2 特征列勘正
> （007/008 实测远超原描述）；T-06 提前并入 B1。

## 1. 目标

1. **G1 全量覆盖**：examples/ui/ 全部 34 个有源码 demo 中除 045 外的 33 个
   完成 Phase A 去重（r2 勘正：原「35 个编号 demo」计数含 025-dashboard/
   038-minesweeper——实测两者已无 `.at` 源码，仅余 `.am` 簿记，矩阵登记
   N/A）；重复 ≥2 次的 class 串不再以字面量形态出现。
2. **G2 token 化**：全部有源码 demo 完成硬编码颜色 → 语义 token 迁移
   （映射表在案），迁移后 demo 内 `bg|text|border|ring-<palette>-<n>` 零
   残留（有意的品牌色除外，逐 demo 显式豁免表）。**r2 边界补全**：无色号
   字面量同入作用域——`text-white`（全仓 ×101，011×42/020×26）、`bg-white`
   （×40+）按宿主角色映射（主按钮 `text-white` → `text-primary-foreground`、
   卡片 `bg-white` → `bg-card`）；`bg-black/40-50` 遮罩与渐变
   `from-/to-/via-`（~20 处）无对应 token，进豁免表登记。
3. **G3 零漂移纪律**：Phase A 各 demo 双端截图 before/after 逐像素一致；
   Phase B 各 demo before/after 截图在案且差异可归因于映射表逐条解释。
4. **G4 门禁固化**：rollout 完成的 demo 登记进矩阵；新增「demo 禁新增裸
   调色板色」grep 守卫（脚本 + 已完成 demo 清单驱动）。
5. **G5 机制实证放大**：stylekit 经 635 通道被 ≥10 个 demo 真实消费
   （dep 声明 + use 导入），跨包配方机制从示范转为常态。

## 2. 架构方案

```
examples/ui/stylekit/（PLAN-635 共享包，本次扩容为 canonical recipe 库）
├── src/front/styles.at     # 既有 pill/card_base + 新增 hint_text/caption_text/
│                           # icon_base/input_field/pill_ghost/badge_* …
└── pac.at                  # pub style 全导出面

每个 demo（Phase A）:
  pac.at:  dep stylekit { path: "../stylekit" }        # 635 声明门控形态
  *.at:    use stylekit.styles: hint_text, icon_base   # 符号导入
           style: hint_text                             # 消费（同本地配方）
  本地重复串 → demo 内 style 声明（不进共享库，避免过度抽象）

Phase B（token 化）: 调色板色 → registry 31 键语义 token（593 单源），
  映射表 = demo 内逐色定性（neutralize→muted-foreground / brand→primary /
  destructive→destructive / 显式豁免保留）。
```

- **展开时机不变**：全部走 607/635 编译期 desugar 单点，运行时零成本。
- **依赖物化**：demo 的 `auto run` 自动 junction 物化 stylekit（Plan 475
  既有通道），635 声明门控保证未声明不可达。

## 3. 技术栈

- Auto 语言面：examples/ui/**/*.at（style 声明/use 导入/语义 token 改写）；
- 共享库：examples/ui/stylekit；
- 守卫脚本：scripts/（grep 门禁 + demo 矩阵）；
- 验证：autoui-verifier 双端（VM MCP snapshot + 截图 / Vue Playwright），
  `cargo tv` 语料回归（demo 源码变动牵连 golden 时按作用域重基线，逐条
  定性）。

## 4. 需求分析与背景调查

- **消费面荒芜实证（2026-09-17 勘察，master 07463f54b）**：36 demo 仅
  013/015/045 有 style 声明；`style: "` 属性点 2069 处；重复串 TOP：
  `text-[10px] text-muted-foreground`×76、`text-xs text-muted-foreground`×58、
  `w-4 h-4 shrink-0 text-muted-foreground`×56、`font-semibold text-gray-900
  text-sm`×21；裸调色板色 TOP：020×264、023×119、011×112、019×107、
  007×86、008×79。
- **基建就绪**：593 值单源（31 键）/601 主题热切换/607 配方语言层/635 跨包
  引用 + stylekit/045 范式，全部 archived delivered。
- **607 试点经验**：013 收敛 8 处、015 收敛药丸串后双端零漂移；lint 对
  recipe 体内裸调色板色已有 warning。
- **022 通道先例**：demo 重构以双端对拍为验收（Plan 448 style 数组、619
  parity 探针）。
- **r2 勘察补记（2026-09-17 计划再评审会话实勘）**：
  - ①无色号字面量体量：`text-white` 全仓 101 处（011×42、020×26、029×8）、
    `bg-white` 44 处（含 `/10` 变体）、`bg-black` 系 10 处（遮罩 `/40`~`/50`
    +实黑）、渐变 `from-/to-` ~20 处——均落在原门禁模式 `<palette>-<n>` 之外；
  - ②`icon_base` 收编对象 ×56 全部集中于 018/026/027/041 四份 **md5 逐字节
    相同** 的 `components/tree_icon.at`（各 14 处）——组件级四胞胎，widget
    本体合并另立小计划（见待澄清#3/#5）；
  - ③006/010/016 pac.at `dep settings { path: "../common/settings" }` 目标
    目录已删（settings 迁址 auto-os/apps/common/settings，auto-os 侧
    ui-gallery 自有可解析 dep），本仓 `deps/settings` junction 悬空、006
    无本地兜底模块（app.at:10 `use settings: SettingsPopover`），standalone
    解析必失败——r2 裁定：demo 跟随系统，删消费面自足化（T-01b），gallery
    宿主自带切换器（ui-gallery/src/front/app.at 有 dark_mode/accent_color
    + 五色板，SettingsPopover 引用数为 0）承接主题控制。

## 5. 详细设计

**D1 stylekit canonical recipe 集（首批，执行期按波次增补）**：

| recipe | 形态 | 收编对象 |
|---|---|---|
| `hint_text` | `text-[10px] text-muted-foreground`（×76） | 全域提示行 |
| `caption_text` | `text-xs text-muted-foreground`（×58） | 说明文字 |
| `icon_base` | `w-4 h-4 shrink-0 text-muted-foreground`（×56） | 图标默认盒 |
| `input_field` | `w-full px-3 py-2 border border-gray-300 rounded text-sm`（×13→Phase B 后 border-border） | 表单输入 |
| `pill`/`pill_ghost` | 既有/次级药丸 | 013/015/045 既有收敛 |
| `card_base` | 既有 | 卡片基座 |
| `section_title` | `font-semibold text-foreground text-*` 族（×9+） | 分节标题 |

命名纪律：避开保留字（`shared` 事件在案）；参数化只对真变体（色/尺寸），
不做过头抽象。

**变体纪律（r2）**：Phase A 字节等价约束下**不跨串合并**——形态相近的串
（如 005-login 的 `w-full px-3 py-2 border rounded-lg mt-2 text-gray-900`
vs D1 `input_field` 的 `w-full px-3 py-2 border border-gray-300 rounded
text-sm`）各自本地配方，Phase B 归一（边框色→token、圆角/边距族裁定）后
才升库合并。

**icon_base 注记（r2）**：56 处全在四份逐字相同的 tree_icon.at——recipe
收编在本计划内完成；widget 本体四胞胎合并走 475 组件级 use 通道（机制已
交付）另立小计划，与待澄清#3 stylekit 定位联动。

**D2 波次划分（从简到重，每波一个任务）**：

| 波 | demo | 特征 |
|---|---|---|
| W1 | 001 002 003 004 005 | 最简（≤5 屏，部分或无样式面） |
| W2 | 006 007 008 009 010 012 014 | 中小为主，含两个准重型（r3 勘正：007=121 style/216 综合位点、008=133 位点，原「style 面 <40 处」描述失实） |
| W3 | 016 017 018 021 022 024 026 027 029 030 031-image-viewer 031-paint 041 043 044 | 中型；内部两极——Phase A 巨型件（024=191、026=149、018=147、027=125 style 位点）与零/低调色板纯 A 件（017/021/031×2/043/044） |
| W4 | 011 019 020 023 | 重型（裸调色板色 ≥100） |
| N/A | 025 038 | 无 `.at` 源码（仅 `.am` 簿记残留），矩阵登记 N/A 不入波次（r2） |

013/015/045：Phase A 已达标，仅纳入 Phase B token 化与门禁矩阵
（013 自身仍有 27 处裸调色板色）。

**D2b 执行批次划分（r3，fold 点=5）**——工作量代理 = style 位点 +
调色板色 + white/black 计数（2026-09-17 全量实测，33 demo 总计 ~3070）：

| 批次 | 内容（任务 → demo） | 工作量 | 验证目标 |
|---|---|---|---|
| B1 奠基+试点 | T-00 stylekit 扩容 + T-01 守卫 + T-01b settings 自足化 + T-02（W1 五 demo）+ T-06（存量三 demo，r3 提前——Phase A 已达标故便宜，且提前兑现 AC-05 消费实证） | ~310（含基建） | 全流程模板定型（A/B/截图/矩阵/guard 纳管）；stylekit 首批 recipe 形态定案 |
| B2 W2 原班 | T-03（006/007/008/009/010/012/014） | ~615 | 首个硬点 007（216 位点）；中型 demo 映射表节奏 |
| B3 W3 轻半 | T-04a（016/017/021/022/029/030/031×2/043/044，10 demo） | ~570 | 批量流水化（多数零/低调色板，Phase A 为主） |
| B4 Phase A 巨型+tree_icon | T-04b（018/024/026/027/041，5 demo） | ~750 | 024-charts 191 位点为全计划最大单件；tree_icon 四胞胎同批机械联动（icon_base 一次落库、四份相同文件同步改） |
| B5 重型+收口 | T-05（011/019/020/023）+ T-07 矩阵终验 + T-08 回归收口 | ~950 | 视觉变更集中过目（待澄清#1 裁定的波次验收点）；全量门禁 |

批次机制（AGENTS.md「multi-phase plans fold per phase and re-sync」形态）：
每批 worktree 内完成 → 批内快验证（双端截图 + `cargo t` 作用域）→ fold
回 master（该批矩阵行 + 抽查复验）→ master 反向 merge 进 `plan-637-dev`
re-sync → 下一批。完整 `/auto-plan:review` + `cargo tv`/`tf` + 归档仅在
B5 终态执行一次，中间批次不重复付全量门禁成本。备选：007 可挪 B4（与
024/018 同为 style 巨型件）——实测配平后 B2≈615 即设计工作量，不挪亦成立。

**D3 双轨验收纪律**：
- Phase A 改写机械可验：脚本断言「改写后每个被替换位点的展开串 ≡ 原串」
  （本地小工具或测试内 desugar 对拍）+ 双端截图零漂移；
- Phase B 映射表先行：每个 demo 一张 `palette → token | 定性` 表
  （neutralize/brand 化/destructive/豁免），截图 before/after 存档，
  差异逐条可归因；**不追求跨 demo 同色一律同映射**（020 的品牌蓝 ≠
  011 的计算器蓝，逐 demo 定）；同一字面量按宿主角色分流映射（如 011 的
  `text-white`×42：橙/靛蓝键上 → `text-primary-foreground`、深色键上 →
  `text-card-foreground`），整簇替换不逐处。

**D4 守卫固化**：`scripts/style_palette_guard.py`——读已完成 demo 清单，
`grep` 该 demo 下 `.at` 的裸颜色字面量，非豁免命中即退出 1；矩阵登记进
本档（执行期回填）。**r2 门禁模式扩容**：`<bg|text|border|ring>-<palette>-<n>`
之外增扫无色号 `text-white`/`bg-white`（`bg-black` 系与渐变在豁免表内静态
登记，不入动态扫描）。豁免三类（对齐待澄清#1 裁定口径）：显式品牌色、
装饰性插画/渐变色、语法高亮色——映射表逐条标注。

### 规范增量

| delta_id | op | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/overview.md | 新增示例配方化规范：stylekit 为 canonical 共享配方库、demo 经 635 通道消费、已完成 demo 禁新增裸调色板色 | 示例层纪律成文 | AC-01/04 |
| SD-02 | modify | docs/specs/auto-man/project.md | stylekit 记为标准多包消费样例（dep path + workspace 双形态） | 635 机制常态化的参照 | AC-05 |

## 6. 测试设计

1. **零漂移对拍**：每 demo Phase A 前后双端截图（VM MCP screenshot +
   Playwright dark 1280×800），文件级 diff 零差异；结构快照 style 行
   before/after 全等（autoui_snapshot）。
2. **展开等价断言**：Phase A 用 Rust 侧小测（或脚本）对抽查位点做
   desugar 前后串等价校验。
3. **Phase B 归因**：映射表 + before/after 截图入库（plan 附件区）。
4. **门禁自测**：guard 脚本对已完成 demo 正例通过、人为注入裸色负例失败。
5. **回归**：`cargo tv`（demo 源码若被语料 golden 引用，作用域重基线并
   逐条定性）；`cargo t` 对拍基线零新增。

## 7. 验收标准

- **AC-01 覆盖**：33 demo（有源码、除 045）全部完成 Phase A；矩阵（执行期
  回填本档，含 025/038 N/A 行）逐 demo 勾记，重复 ≥2 次长串字面量清零
  （grep 可验证）。
- **AC-02 token 化**：全部 demo 完成Phase B 或显式豁免登记；已完成 demo
  裸调色板色残留 = 豁免表条目。
- **AC-03 零漂移（Phase A）**：全部 demo 截图 before/after 逐像素一致，
  凭据在案。
- **AC-04 受控变更（Phase B）**：每 demo 映射表 + 截图 before/after 在案，
  差异逐条归因。
- **AC-05 机制消费**：≥10 demo 的 pac.at 含 `dep stylekit` 且 `use`
  导入真实配方；双端 run 物化消费通过。（r2 注记：W1 极简 demo——001 仅
  2 处 style、002 仅 1 处、003 零调色板——仅在真实命中 recipe 时接入，
  不为指标强行挂 dep；≥10 目标由 W2/W3 兜底，007 单 demo 121 处 style
  面已足。）
- **AC-06 门禁**：guard 脚本落地并入 matrix 工作流；正反例自测绿。
- **AC-07 门禁**：`cargo tv` 作用域重基线逐条定性零漂移；`cargo t`
  对拍 master 零新增红。

## 8. 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] **T-00 stylekit 扩容**：styles.at 落 D1 首批 recipe（pub），045 回归
  （既有消费不破）；`cargo test -p auto-man --lib vue` + 045 双端 run。
  （待澄清#1 已裁定，Phase B 无阻塞。）
  [✅ 已完成 2026-09-17 B1] 8 recipe 落库（pill_ghost/hint_text/caption_text/
  icon_base/input_field/section_title 新增；input_field 直接落 Phase B 定案形
  border-border，消费方 W4/023）。045 双端 run 像素 IDENTICAL
  （evidence/637/045-{vue,vm}-after.png 对 before）。验证命令勘正：auto-man 在
  auto-os 仓不在本仓，本仓侧以 045 双端 run+guard 为准（Category A 免 cargo t）。
- [x] **T-01 守卫脚本先行**：`scripts/style_palette_guard.py` + 调用约定
  （矩阵清单文件）；正反例自测。
  [✅ 已完成 2026-09-17 B1] guard + style_palette_manifest.json 落地；自测
  4/4 绿（正例零命中/反例 4 命中/from-bg-black 不扫/hover 变体命中）；已完成集
  正例 8/8 PASS（001-005/013/015/045）。注释行不扫（映射表头注引用旧字面量）。
- [x] **T-01b settings 消费面移除（r2 前置，W2 开工前完成）**：006/010/016
  删 `use settings: SettingsPopover`、`SettingsPopover(...)` 调用点、
  `settings_open` 状态与 ⚙ Theme 触发按钮；**保留** `dark_mode`/
  `accent_color` 两变量（双端运行时契约——VM 渲染器每帧回读、Vue gen
  据此生成 `.dark` 绑定与 applyAccent 注入，pac `theme:`/`accent:` 启动
  默认的落点，删则 demo 失去主题敏感性）；悬空 `deps/settings` junction
  逐枚 `rmdir` 拆除。验证：三 demo 双端 `auto run` 可起（standalone
  自足化，无悬空 use）；auto-os 侧 gallery registry 再生时 demo 面
  popover 自动消失（registry 从 examples/ui 全量覆写，单源不漂移），
  不入本计划验证面。
  [✅ 已完成 2026-09-17 B1] 006/010 use+调用点+settings_open+⚙按钮+ToggleSettings
  删除；016 仅删 pac.at 悬空 dep（其内置面板为 store 本地实现，保留）；
  SetTheme/SetAccent msg 面保留（宿主→demo 主题传播路线通道，本 demo 内无发射方）。
  主检出悬空 junction ×5 拆除（006/010/015/016+计划外 019；011/deps/common 留 W4）。
  三 demo VM 冒烟绿 + Vue/VM run-proof 在案（016 双端像素 IDENTICAL）。偏差：r2
  「standalone 必失败」失实（worktree 无 junction 亦能起，疑 index/缓存兜底），
  移除按用户裁定照常执行。
- [x] **T-02 W1 波**（001-005）：逐 demo「Phase A 去重 → 双端零漂移 →
  Phase B 映射表 + 迁移 → 截图 → 矩阵登记」；T-00 后 stylekit 消费第一批。
  （r3：与 T-00/T-01/T-01b/T-06 同属 **B1** 批次。）
  [✅ 已完成 2026-09-17 B1] 001/002 no-op（无重复串/无调色板色，矩阵登记）；
  003 Phase A 本地配方 ×4 位点，vue+vm 像素 IDENTICAL 零漂移；004 Phase B
  8 簇（vue 11.4%/vm 10.5% 差异逐条归因，状态点+渐变豁免登记）；005 A+B 合并
  （3 本地配方 ×6 位点 + 9 簇，vue 16.2%；vm 81.4%=bg-white→bg-card 暗主题整卡
  翻转，目标行为）。W1 无全等 recipe 命中，依 r2 AC-05 注记不强行挂 dep。
- **T-03 W2 波**（006/007/008/009/010/012/014）：同模板；按需增补
  stylekit recipe（版本随波推进）。（r3：= **B2** 批次。）
- **T-04 W3 波**（15 个中型 demo）：同模板；允许按 demo 分子提交。
  （r3：拆 **B3** 轻半 10 demo / **B4** 巨型 5 demo 两次 fold；
  tree_icon 四胞胎（018/026/027/041）聚 B4 机械联动。）
- **T-05 W4 波**（011/019/020/023）：重型 token 化；020 的 264 处逐簇
  （同类串整簇替换）而非逐处。（r3：= **B5**，与 T-07/T-08 收口同批。）
- [x] **T-06 存量三 demo 收尾**（013/015/045 Phase B + 门禁纳入；r3：提前
  并入 **B1** 执行——Phase A 已达标故便宜，且提前兑现 AC-05 stylekit
  消费实证）。
  [✅ 已完成 2026-09-17 B1] 013 Phase B 16 簇（vue 95.4%/vm 98.3%，主因
  bg-gray-100→bg-background 全屏翻转+映射表逐条）；015 caption_text ×5 真实消费
  （pac dep stylekit + editor/sidebar use 导入；vue 像素 IDENTICAL 零漂移；vm
  22.8%=有状态 db 内容/时间戳漂移，差值可视化在案非样式）+ accent 五色票豁免
  登记；045 零改动纳入门禁（双端 IDENTICAL 回归）。guard completed 集 8 demo。
- **T-07 矩阵终验**：35/35 勾记 + grep 全量复扫 + 豁免表复核。
- **T-08 回归与收口**：`cargo tv` + `cargo t` 对拍；spec delta 回填；
  `execution_done`。**worktree 链接拆除（r2，wt-guard 红线）**：各 demo
  `auto run` 按 475 通道物化的 `deps/stylekit` 等 junction（预计 30+ 枚）
  收尾时逐枚 `rmdir` 拆除（严禁递归删除——会穿透链接删除目标内容），
  拆净后跑 `bash D:/autostack/wt-guard.sh` 确认 clean 再移除 worktree
  （635 先例：合并时手工拆除 360 枚链接，见其合并回执 cleaned 节）。
- （波次内发现的新重复模式回流 stylekit 时，走 demo 内 style 先行、
  升库与消费方切换同波完成，避免半态。）

### 8.1 执行矩阵（执行期回填，随批次追加）

像素对拍证据：`docs/plans/evidence/637/`（b1-evidence-summary.md 含归因总表）。

| demo | Phase A | Phase B | B1 对拍（vue/vm） | 门禁 | 消费 stylekit |
|---|---|---|---|---|---|
| 001 | no-op | no-op | —（未改未拍） | ✅ | — |
| 002 | no-op | no-op | —（未改未拍） | ✅ | — |
| 003 | ✅ 本地配方 ×4 | 无色可换 | IDENTICAL / IDENTICAL | ✅ | —（无全等命中） |
| 004 | 无重复串 | ✅ 8 簇+豁免 2 | 11.4% / 10.5%（归因表） | ✅ | — |
| 005 | ✅ 本地配方 ×6 | ✅ 9 簇 | 16.2% / 81.4%（归因表） | ✅ | — |
| 006 | （T-01b 先行移除） | B2 | 3.9% / 3.2%（移除归因） | B2 | — |
| 010 | （T-01b 先行移除） | B2 | 25.8% / IDENTICAL（vm 本不可见） | B2 | — |
| 013 | 607 达标 | ✅ 16 簇 | 95.4% / 98.3%（归因表） | ✅ | — |
| 015 | 607 达标 | ✅ 豁免 5 | IDENTICAL / 22.8%（状态漂移在案） | ✅ | ✅ caption_text ×5 |
| 016 | B2 | B2 | IDENTICAL / IDENTICAL（T-01b 仅 pac） | B2 | — |
| 045 | 635 达标 | 无色可换 | IDENTICAL / IDENTICAL（回归） | ✅ | ✅（既有） |
| 025/038 | N/A（无 .at 源码，r2） | N/A | — | — | — |

## 9. 复审记录

- （draft 起草 handoff 2026-09-17：`stage: new | PLAN-637 | plan_revision: 1 |
  outcome: pass | next: work`——Phase B 视觉变更授权与豁免边界见待澄清#1，
  开工前需裁定。）
- （r2 修订 2026-09-17 计划再评审：8 项落档——①025/038 无源码 N/A（G1
  计数勘正）；②white/black 入 Phase B、遮罩/渐变入豁免；③D4 豁免三类
  口径对齐#1 裁定；④AC-05 W1 不过度抽象注记；⑤D1 变体纪律；⑥tree_icon
  widget 四胞胎另立小计划；⑦T-01b settings 消费面移除（用户裁定：demo
  跟随系统，宿主切换器承接）；⑧T-08 worktree junction 拆除。评审依据=
  同会话实勘带锚点：tree_icon md5 四份一致、006/010/016 悬空 junction、
  text-white/bg-white 计量、025/038 目录只剩 .am。）
- （r3 修订 2026-09-17 用户确认：执行批次定 5 个 fold 点（B1-B5，D2b 表，
  工作量实测配平）；D2 特征列勘正（007=216/008=133 位点，原 W2「<40 处」
  失实）；T-06 提前并入 B1；T-04 拆 B3/B4 两次 fold。工作量基线=同会话
  33 demo 全量实测（style/调色板/white 三列计量，总计 ~3070 位点）。）
- （B1 批次 work handoff 2026-09-17：`stage: work | PLAN-637 | plan_revision: 3 |
  outcome: pass（批次级，整体保持 executing） | code_commit: plan-637-dev B1 提交 |
  task_ids: T-00/T-01/T-01b/T-02/T-06 | evidence: docs/plans/evidence/637/
  （b1-evidence-summary.md + 截图对 + 015-vm-diff-vis.png） | blockers: 无 |
  next: fold 回 master 后 B2（T-03 W2 波）。批次机制：完整 review+cargo tv/tf
  仅在 B5 终态执行一次；批次级偏差 7 项见 b1-evidence-summary.md §3
  （r2 standalone 失实勘正、006/010 SetTheme/SetAccent 保留、016 本地面板保留、
  004 状态点豁免、T-00 验证命令勘正、主检出副产物还原、截图脚本陷阱）。）

## 10. 待澄清事项

1. **Phase B 视觉变更授权**：✅ **已裁定（2026-09-17 用户）**——按「语义
   就近 + 截图留档、波次验收时集中过目」执行。裁定的实质依据（用户同日
   给出终局目标）：token 化颜色表最终要做**可选可配置的主题系统**（VSCode
   式切换/预览，每主题对全部 token 独立设色），demo 不拥有颜色、主题拥有
   颜色——因此 Phase B 映射 = 纯角色分类（机械），原字面量值按设计退休，
   最终观感由默认主题（registry 既有 zinc/scaffold/stella）决定；截图 =
   归一化文档而非漂移审查。第一版默认一套（或几套）即可——registry 三套
   内置已满足。豁免边界（装饰性插画色/语法高亮色保留字面量）一并授权，
   进豁免表。**后续路线注记（另立项，不入本计划）**：①双轨默认色板归一
   （VM stella vs Vue scaffold zinc 的 593 分叉，VSCode 式体验要求 canonical
   默认）；②主题选择器/预览组件 + 用户自定义主题配置集成（601 热切换
   之上的 UI 面）。
2. **Phase A/B 同 demo 内的顺序**：推荐 A 完成验收后再做 B（漂移归因
   单变量化）；若 demo 样式面很小可 A+B 合并一次过（W1 多数如此）。
3. **stylekit 定位**：它是 examples 内的示范包（随仓分发）还是未来抽出为
   独立发行组件库的雏形（影响命名空间与版本纪律——本计划按示范包执行，
   发行化另行立项）。
4. **settings 跟随系统（r2）**：✅ **已裁定（2026-09-17 用户）**——006/
   010/016 demo 跟随系统，不保留应用级 settings：删消费面（T-01b），保留
   `dark_mode`/`accent_color` 契约变量；主题控制由 auto-os 侧 gallery 宿主
   切换器承接（已就位）。**宿主 → 内嵌 demo 的运行时主题传播**不在本计划
   ——归后续主题系统路线（待澄清#1 注记②立项时列为首个设计点）。
5. **tree_icon widget 级合并归属（r2）**：✅ **已裁定（2026-09-17 用户
   确认）**——本计划仅做 `icon_base` recipe 收编；widget 本体四胞胎合并
   （018/026/027/041 的 tree_icon.at，md5 逐字节相同，走 475 组件级
   use 通道）另立小计划，与待澄清#3 stylekit 定位裁定联动。
