---
plan_id: PLAN-589
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B P-7——shell pack 路径化+权威翻转+hash-lock
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [reports/P589-1, reviews/P589-2]
touched_goals: [GOAL-010]             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径
current_step: 5
total_steps: 5
---

# [PLAN-589] Stage B P-7——shell pack 路径化 + 权威翻转 + hash-lock 同步契约

## 变更摘要

Design 01 §4-P7（用户二次裁定 shell 四件随迁）。三步：

1. **加载器**：`ui/shell.rs` 四 const 保留为内嵌 pin 快照（include_str! 不动，
   无 pack 环境逐字节一致）；新增 `resolve_shell_pack_dir()`（host override →
   `AUTO_SHELL_PACK` env 设置即权威 → 兄弟 `../auto-os/shell` → 主检出兜底 →
   None）+ `shell_source(name)`（pack 命中读文件，缺件逐件回退+警告）；四个
   `build_*_component` 改走 loader——外部消费面天然已收敛于此四漏斗
   （直用常量仅 icon 契约测试与 vue 金样测试，钉快照不动）。`DesktopOptions`
   增 `shell_pack: Option<PathBuf>`（沿 apps_dir 先例，run_session 注入）。
2. **物理迁移+权威翻转**：四件复制入 auto-os `shell/`（权威源）；auto-lang
   `assets/` 内嵌副本降级 pin 快照。
3. **hash-lock 契约**：auto-os `scripts/shell-pack-sync.py`（四件 hash 比对，
   单向 auto-os → auto-lang 同步+pin 版本注记，漂移红灯）；auto-lang 侧
   parity 测试（pack 可解析时四件 hash == 内嵌快照，solo 跳过）。

门档：Category B（ui 局部）——`cargo t iced` + tf。worktree=lang-589。

## 目标

1. 无 pack 环境行为与现状逐字节一致（回退零变化，既有 3000+ 测试零重锚）。
2. pack 命中（env/兄弟/主检出）加载成功且实机桌面不回归（V9）。
3. 双源无静默漂移：hash-lock 脚本红灯 + parity 测试。
4. Design 01 §7 P-7 行回填；§1-A4 权威翻转落位。

## 架构方案

| 件 | 落点 | 说明 |
|---|---|---|
| D1 加载器 | ui/shell.rs | resolve+source+四 build 漏斗改线；override OnceLock |
| D2 DesktopOptions | iced/renderer.rs | shell_pack 字段 + run_session 注入 |
| D3 迁移 | auto-os shell/ 四件 | 权威源（本批创建=内容与 pin 快照全等） |
| D4 hash-lock | auto-os scripts/shell-pack-sync.py + auto-lang parity 测试 | 单向同步契约 |

## 详细设计

- 解析序（与 P-3/P-2 解析序家族同律）：`DesktopOptions.shell_pack`（宿主
  显式，最特定）→ `AUTO_SHELL_PACK` env（设置即权威，兼关断：指空目录=关）
  → 兄弟（repo root 相对，worktree 组布局）→ `D:/autostack/auto-os/shell`
  （主检出兜底）→ None（内嵌）。pack 目录存在但缺某件 → 该件回退内嵌+
  eprintln 警告（不炸）。
- 低频装载（boot/召唤期）不做缓存，每次直读（语义最简）。
- parity 测试（auto-lang）：resolve Some 时逐件 sha256 对比内嵌快照；
  None 时 pass（solo 语义）。

## 测试设计

`cargo t iced` 局部 + `cargo tv`；折叠前 `cargo tf`。单测：env 权威 fixture
pack（临时目录+假件）；缺件回退；无 pack 字节一致（getter==const）；
parity（真布局）。V9 实机：ui_desktop 双轮 boot（无 pack/AUTO_SHELL_PACK
命中）surface 装载成功证据。

## 验收标准

- [x] 无 pack：getter==内嵌 const 字节一致（单测断言）；iced 169/169
      零重锚（lucide manifest 红=预存 master 同败，P589-D1 在案）。
- [x] pack 命中：四件从 pack 读到（fixture 断言）；缺件回退+警告；
      实机双轮 boot 零警告零失败。
- [x] auto-os shell/ 四件就位（sha256 全等）；sync 脚本三态实测
      （0/1/0）。
- [x] parity 测试绿（D3 后本机转活，双源全等）。
- [x] tv/tf 唯红 charts 预存基线一致；Design 01 §1-A4/§7 已回填。

## 执行步骤

1. [✅ 已完成] **D1+D2 加载器+override 接线+单测**：shell.rs
   resolve_shell_pack_dir（override[OnceLock 首值胜]→env 设置即权威[指非
   目录=关断]→兄弟→主检出→内嵌）+ shell_source（逐件回退+警告）+ 四 build
   漏斗改线 + DesktopOptions.shell_pack + run_session 注入。
   [✅ 已完成] p7_loader 测试 2/2（env 权威 fixture/缺件回退/未知件防御/
   关断语义/hash parity）。
2. [✅ 已完成] **D3 迁移**：auto-os shell/ 四件落位（sha256 与 pin 快照
   全等：shell e7df723ab5/desktop 7b698bd0ae/switcher 8e54978c8f/
   notification_center 2ad182c90f）。
3. [✅ 已完成] **D4 hash-lock**：auto-os scripts/shell-pack-sync.py（校验
   红灯/--sync 单向同步+pin 注记；全等/漂移/还原三态实测 exit 0/1/0）+
   auto-lang parity 测试（pack 可解析时逐件 sha256 对比快照，solo 跳过——
   D3 后本机已转活）。
4. [✅ 已完成] **V9 实机+门档**：ui_desktop 双轮 boot（AUTO_SHELL_PACK=
   auto-os/shell 命中 / 指非目录=关断回退）均零 "[shell-pack] fallback"
   警告零 surface 失败、registry 37/21 两轮一致；iced 169/169 绿（唯
   lucide manifest 预存红——master 同败实证，登记 P589-D1）；tv/tf 唯红
   charts 预存基线一致。
5. [✅ 已完成] **收口**：Design 01 §1-A4/§7 P-7 行回填（auto-os 随行提交）
   + P589-1/2 沉淀 + ui/plans.md 589 行 + KNOWN-DEBT P589-D1 + INDEX 重生
   + 归档。

## 复审记录

（2026-09-07 复审，verify-don't-trust。）

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| C1 | 无 pack 逐字节一致 | PASS | 单测 env 关断臂 getter==const 断言；include_str! 编译期不动 |
| C2 | pack 命中+回退 | PASS | fixture 单测（命中读文件/缺件回退/未知件空串）；V9 双轮 boot 零警告 |
| C3 | 迁移全等+脚本 | PASS | sha256 四对全等；sync 三态 exit 0/1/0 |
| C4 | parity | PASS | D3 后本机转活全等；solo 跳过语义 |
| C5 | 门档+簿记 | PASS | iced 169/169（lucide 预存 master 同败）；tv/tf charts 预存基线；§1-A4/§7 回填 |

遗漏/延后扫描：①lucide manifest 缺口（025-sys-monitor:activity/030-
video-player:film）= 541 合并期预存、master 同败实证——登记 P589-D1
（lucide_svg 补臂另案，非本批）；②override OnceLock 首值胜为进程内幂等
防御（多 session 进程首宿主声明为准——语义注记在案）；③四 const 直用面
（icon 契约测试/vue 金样）钉快照不动=设计内。健康：零新增警告；eprintln
两处为运维警告（[shell-pack] 前缀）。

## 待澄清事项

（无——§4-P7 三步方案即定案；解析序沿 P-2/P-3 家族惯例。）

## 变更摘要

## 目标

## 架构方案

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

## 详细设计

## 测试设计

## 验收标准

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

## 复审记录

## 待澄清事项
