---
plan_id: PLAN-531
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-debt-clearance-batch
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-017]     # 自举（债务清欠,塔顶验证链前置）

affects: [aavm, auto-lang/trans]
current_step: 0
total_steps: 11
---

# [PLAN-531] aavm 债务清偿批：tt 真缺陷 / a2r 模式两洞 / pac target / May 发射

## 变更摘要

打包清偿 523/525 收口挂账的存量债务（P523-1/2/3 + P525-4，及可选项
P525-3 `??`），**为塔顶自举（Plan 532）清障**——其中 P523-2② 转译版
struct 字段表洞直接位于塔顶验证链上：

- **P523-3（核心,含两件真缺陷）**：tt 档（a2r test-trans 金样）28 件
  存量过期——批量 bless + **两件真发射缺陷修复**（pointer/004：cast 后
  方法调用；arc_dyn_spec/008：str as usize——文本金样长期掩盖）+
  tt 纳入复审清单（防再烂）；
- **P523-2（塔顶路径障碍）**：aavm.at a2r 模式入口两洞——①宿主
  `argv.get(1)` 发射 `Vec::get` Option 形态缺自动解包（E0308）；②转译
  版 VM 运行期 struct 字段表（`c.field_idx` 动态查表在转译宿主的语料内
  struct 形态未覆盖,b34 报 `no field x in Point`;VM 解释路径不受影响）；
- **P523-1**：pac 最小工程 rust target 生成缺口（CargoBuilder "no rust
  target found"）——pac.rs 语义补齐（api-example 前端存量红清偿视成本
  裁定归属,见待澄清③）;
- **P525-4**：宿主 May 裸值 return 发射修复（`fn f() ?int { return n*10 }`
  主 a2r 发裸值于 Option<T> fn——E0308;修后补裸值语料）;
- **可选**：P525-3 `??` NullCoalesce 实现（已入 Pratt 表,缺 codegen 臂;
  便宜则顺做）。

**不纳入**（裁定留档）：P525-1 主 a2r merge 后处理 AST 级根治（lib 已
规避,超小计划范围,塔顶后有需要再立）；P525-2 闭包 m4 对拍（已裁定迁位
corpus_a2r,判据面=发射闸+四路）；P525-5 塔顶稳定性（归 Plan 532 W0）。

**与 532 的关系**：532（塔顶自举）**硬前置 = 本计划 archived**——
P523-2② 洞不清,塔顶的转译版验证链不干净。

## 目标

1. tt 档 28 件 bless 后全绿；两件真缺陷修复有回归锚；tt 入复审清单
   （计划复审节+tf 可见性裁定）。
2. `auto/aavm.at` a2r 模式两洞清偿：位置参数形态经主 a2r 转译构建后
   **b07→55 实测**；转译版 struct 语料（b34）运行绿。
3. pac 最小工程 rust target 生成链补齐（冒烟脚本走 pac 正路而非旁路）。
4. 宿主 May 裸值 return 发射修复 + g34 补裸值语料。
5. （可选）`??` 操作符 aavm 四层实现 + 语料。
6. 全程 525 终态保护网不破绿。

### 非目标（Out of Scope）

- 塔顶自举本体（Plan 532）；api-example 全管线前端 strict 红清偿
  （视待澄清③裁定,缺省独立处置）;生成器 yield（P525-3 裁定延后不动）；
  P525-1/P525-2（上文裁定留档）。

## 架构方案

```text
W0 考古                 W1 tt 档清偿            W2 a2r 两洞            W3 pac+May+收尾
──────────            ──────────────          ─────────────          ───────────────
两真缺陷根因 →   批量 bless+修复锚定 →   argv 解包+字段表 →   pac target 链 →
tt 复审可见性 →   tt 入清单            →   塔顶障碍清除      →   May 裸值+(??) → 复审
bless 流程                                        aavm.at 全链实测        归档
```

## 需求分析与背景调查

（取材：KNOWN-DEBT P523-1/2/3、P525-3/4 行、divergences.md
D-a2r-mode-entry（:485）、`scripts/aavm_build_smoke.sh`、tt 档现状）

### 基线（开工时复测留档）

525 reviewed/归档中时点：四路 30/30、tv 3559、tt 28（存量红）、tf 2 红
（schema_drift=523 存量注记+docs_gen=528 面）。

### 逐债要点

| 债 | 现状证据 | 修复位预估 |
|---|---|---|
| P523-3 真缺陷①（004 pointer） | cast 后方法调用发射错（文本金样掩盖） | trans/rust.rs cast 接收者链 |
| P523-3 真缺陷②（008 arc_dyn_spec） | str as usize 缺 | trans/rust.rs 后处理强转位 |
| P523-2① argv.get 解包 | E0308,产品位文本垫片实测通过（临时） | trans/rust.rs Vec::get→索引/解包发射 |
| P523-2② 转译版 struct 字段表 | b34 RUNTIME-ERROR no field x in Point（转译宿主语料内 struct 未入字段表;VM 解释路径绿） | 转译侧 typeinfo 对齐（主 a2r 语料内 type 的字段注册缺失） |
| P523-1 pac target | pac.rs targets 无 rust App/Lib 项 | pac.rs/pac 最小工程 target 生成链 |
| P525-4 May 裸值 return | E0308（语料规避中） | trans/rust.rs return 位 Option 包裹 |
| P525-3 `??` | Pratt 表已有,codegen 缺臂 | codegen.at + 语料 |

### 风险与对策

| 风险 | 对策 |
|---|---|
| 两真缺陷修复波及面（merge 后处理敏感 pass） | 逐缺陷独立提交+金样 diff 评审;525-1 同族敏感性注记参照 |
| bless 掩盖真差异（bless 本身无判别力） | 先修真缺陷再 bless;bless diff 逐件评审（28 件清单留档） |
| 转译版字段表洞根因在宿主深层（525-1 同片区） | W0 根因定位先行;若超小修量级→登记转 532 W0 处置（逃生舱,待澄清②） |
| api-example 前端红范围失控 | 缺省只做 pac target 链,前端红独立处置（待澄清③） |

## 详细设计

### W0 考古（master 纯文档+探针）

1. 两真缺陷最小复现+根因定位（rustc 报文+发射文本对照）;tt 复审可见性
   方案（复审清单条目+tf 别名可见性——`.cargo/config.toml` tt 不在 tf
   档,裁定是否入）;bless 流程（复用 523 `--bless` 基建）。
2. P523-2 两洞根因定位（①：宿主 argv.get 发射对照;②：转译侧 typeinfo
   字段注册路径——以 b34 转译构建运行复现,定位在主 a2r type 发射或
   运行时表桥接）。

### W1 tt 档清偿（worktree）

3. 真缺陷①②修复（各自独立提交+新金样锚）。
4. 28 件批量 bless（diff 逐件清单评审留档）+ tt 入复审清单。

### W2 a2r 模式两洞（worktree 续）

5. argv.get 解包发射修复 → aavm.at 位置参数形态转译构建全链实测
   （b07→55）;垫片撤除。
6. 转译版 struct 字段表修复 → b34 转译构建运行绿（10/20）;若根因超
   量级按待澄清②转 532。

### W3 pac+May+收尾（worktree 续）

7. pac 最小工程 rust target 生成链补齐 → 冒烟脚本切正路（`auto build`
   路径）实测。
8. May 裸值 return 发射修复+g34 补裸值语料;（可选,待澄清①）`??` 四层
   实现+语料。
9. 全量回归（tf+四路+矩阵）+ 折叠合入。
10. 文档回写（KNOWN-DEBT 五债核销/divergences D-a2r-mode-entry 清偿
    注记）+ 复审 → reviewed。
11. merge 沉淀归档。

## 测试设计（TDD）

- **保护网**：525 终态全绿面（四路 30/30+tv 3559+矩阵）+tf 基线红
  （归属注记）不新增。
- **红先行**：两真缺陷（修前 rustc 红/金样 diff）→ 修后绿;P523-2 两洞
  （b07/b34 转译构建运行红→绿）;May 裸值（g34 裸值件红→绿）;`??`
  （新语料红→绿）。
- **命令**：`cargo tt`（tt 档）/ `bash scripts/aavm_build_smoke.sh` /
  四路 runner / 矩阵（P517-2 纪律）。

## 验收标准

1. `cargo tt` 全绿（28 件 bless 后+新锚）;两真缺陷修复有回归金样;tt
   入复审清单（条目留档）。
2. aavm.at a2r 模式：转译构建后位置参数 b07→55 实测;b34→10/20 实测
   （或按待澄清②显式转 532 登记）。
3. pac 最小工程 target 生成链补齐,冒烟脚本走 `auto build` 正路。
4. May 裸值语料绿;（可选项按裁定）。
5. tf/四路/矩阵零新增红;KNOWN-DEBT 核销;无静默丢弃。

## 执行步骤
（原子任务;W0 在 master,实现 in worktree `.worktrees/plan-531-dev`;
单折叠点=步骤 9）

1. [ ] W0 考古：两真缺陷根因+tt 可见性方案+bless 流程;两洞根因定位。
   验证：考古注记+复现留档。
2. [ ] W0b 红先行：两真缺陷最小件/两洞复现/May 裸值件落盘（红证）。
3. [ ] 真缺陷①修复（独立提交+锚）。验证：新金样绿。
4. [ ] 真缺陷②修复（独立提交+锚）。验证：同上。
5. [ ] tt 28 件批量 bless（逐件 diff 清单评审）+入复审清单。
   验证：`cargo tt` 全绿。
6. [ ] argv.get 解包修复+aavm.at 全链实测（b07→55,垫片撤除）。
7. [ ] 转译版 struct 字段表修复（b34→10/20 实测;超量级转 532 登记）。
8. [ ] pac target 链补齐（冒烟切正路）+May 裸值修复+（可选）`??`。
9. [ ] 全量回归+折叠合入 master。
10. [ ] 文档回写+复审 → reviewed。
11. [ ] merge 沉淀归档。

## 复审记录

## 待澄清事项

1. **`??` 可选项**（阻塞步骤 8）：缺省顺做（Pratt 表已有,四层臂+语料
   半天级）;若 W0 发现与 May 语义耦合超预期则显式延后（P525-3 维持）。
2. **P523-2② 逃生舱**（阻塞步骤 7）：根因若在主 a2r type 发射深层
   （525-1 同片区敏感性）且超小修量级 → 登记转 532 W0 处置,本计划
   其余照收。
3. **api-example 前端红归属**（阻塞步骤 8）：缺省独立处置不入本计划
   （UI 线/独立小件）;若 pac target 链修复顺带揭示同根因再裁定。
