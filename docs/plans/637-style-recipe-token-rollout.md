---
plan_id: PLAN-637
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: style-recipe-token-rollout（examples/ui 全量配方化 + token 化）
author: [zhaopuming]
created_at: 2026-09-17
updated_at: 2026-09-17
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/overview.md（示例配方化规范条目：stylekit 共享库 + 禁新增裸调色板色门禁）
touched_goals: ["GOAL-007: AutoUI 跨端视觉一致（样式配方/令牌抽象）"]

affects: [auto-lang/ui, autoui-examples]
current_step: 0
total_steps: 9
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

## 1. 目标

1. **G1 全量覆盖**：examples/ui/ 全部 35 个编号 demo（除 045 自身）完成
   Phase A 去重；重复 ≥2 次的 class 串不再以字面量形态出现。
2. **G2 token 化**：全部 demo 完成硬编码调色板色 → 语义 token 迁移
   （映射表在案），迁移后 demo 内 `bg|text|border|ring-<palette>-<n>` 零
   残留（有意的品牌色除外，逐 demo 显式豁免表）。
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

**D2 波次划分（从简到重，每波一个任务）**：

| 波 | demo | 特征 |
|---|---|---|
| W1 | 001 002 003 004 005 | 最简（≤5 屏，部分或无样式面） |
| W2 | 006 007 008 009 010 012 014 | 中小（style 面 <40 处） |
| W3 | 016 017 018 021 022 024 026 027 029 030 031-image-viewer 031-paint 041 043 044 | 中型 |
| W4 | 011 019 020 023 | 重型（裸调色板色 ≥100） |

013/015/045：Phase A 已达标，仅纳入 Phase B token 化与门禁矩阵
（013 自身仍有 27 处裸调色板色）。

**D3 双轨验收纪律**：
- Phase A 改写机械可验：脚本断言「改写后每个被替换位点的展开串 ≡ 原串」
  （本地小工具或测试内 desugar 对拍）+ 双端截图零漂移；
- Phase B 映射表先行：每个 demo 一张 `palette → token | 定性` 表
  （neutralize/brand 化/destructive/豁免），截图 before/after 存档，
  差异逐条可归因；**不追求跨 demo 同色一律同映射**（020 的品牌蓝 ≠
  011 的计算器蓝，逐 demo 定）。

**D4 守卫固化**：`scripts/style_palette_guard.py`——读已完成 demo 清单，
`grep` 该 demo 下 `.at` 的裸调色板色，非豁免命中即退出 1；矩阵登记进
本档（执行期回填）。豁免仅限显式品牌色（映射表标注）。

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

- **AC-01 覆盖**：35 demo 全部完成 Phase A；矩阵（执行期回填本档）逐
  demo 勾记，重复 ≥2 次长串字面量清零（grep 可验证）。
- **AC-02 token 化**：全部 demo 完成Phase B 或显式豁免登记；已完成 demo
  裸调色板色残留 = 豁免表条目。
- **AC-03 零漂移（Phase A）**：全部 demo 截图 before/after 逐像素一致，
  凭据在案。
- **AC-04 受控变更（Phase B）**：每 demo 映射表 + 截图 before/after 在案，
  差异逐条归因。
- **AC-05 机制消费**：≥10 demo 的 pac.at 含 `dep stylekit` 且 `use`
  导入真实配方；双端 run 物化消费通过。
- **AC-06 门禁**：guard 脚本落地并入 matrix 工作流；正反例自测绿。
- **AC-07 门禁**：`cargo tv` 作用域重基线逐条定性零漂移；`cargo t`
  对拍 master 零新增红。

## 8. 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T-00 stylekit 扩容**：styles.at 落 D1 首批 recipe（pub），045 回归
  （既有消费不破）；`cargo test -p auto-man --lib vue` + 045 双端 run。
- **T-01 守卫脚本先行**：`scripts/style_palette_guard.py` + 调用约定
  （矩阵清单文件）；正反例自测。
- **T-02 W1 波**（001-005）：逐 demo「Phase A 去重 → 双端零漂移 →
  Phase B 映射表 + 迁移 → 截图 → 矩阵登记」；T-00 后 stylekit 消费第一批。
- **T-03 W2 波**（006/007/008/009/010/012/014）：同模板；按需增补
  stylekit recipe（版本随波推进）。
- **T-04 W3 波**（15 个中型 demo）：同模板；允许按 demo 分子提交。
- **T-05 W4 波**（011/019/020/023）：重型 token 化；020 的 264 处逐簇
  （同类串整簇替换）而非逐处。
- **T-06 存量三 demo 收尾**（013/015/045 Phase B + 门禁纳入）。
- **T-07 矩阵终验**：35/35 勾记 + grep 全量复扫 + 豁免表复核。
- **T-08 回归与收口**：`cargo tv` + `cargo t` 对拍；spec delta 回填；
  `execution_done`。
- （波次内发现的新重复模式回流 stylekit 时，走 demo 内 style 先行、
  升库与消费方切换同波完成，避免半态。）

## 9. 复审记录

- （draft 起草 handoff 2026-09-17：`stage: new | PLAN-637 | plan_revision: 1 |
  outcome: pass | next: work`——Phase B 视觉变更授权与豁免边界见待澄清#1，
  开工前需裁定。）

## 10. 待澄清事项

1. **Phase B 视觉变更授权（阻塞 W1 Phase B，不阻塞 T-00/T-01/W1 Phase A）**：
   token 化是有意的视觉变化（如 gray-400→muted-foreground 在 dark 模式下
   提亮、blue-500→primary 换品牌色）。两个子裁定：
   a) 逐 demo 映射表是否需要用户过目确认，还是按「语义就近 + 截图留档
   事后抽查」授权执行（推荐后者，波次验收时集中过目）；
   b) 豁免边界：纯装饰性插画色/语法高亮色等非语义色是否允许保留字面量
   （推荐允许，进豁免表）。
2. **Phase A/B 同 demo 内的顺序**：推荐 A 完成验收后再做 B（漂移归因
   单变量化）；若 demo 样式面很小可 A+B 合并一次过（W1 多数如此）。
3. **stylekit 定位**：它是 examples 内的示范包（随仓分发）还是未来抽出为
   独立发行组件库的雏形（影响命名空间与版本纪律——本计划按示范包执行，
   发行化另行立项）。
