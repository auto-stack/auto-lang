---
plan_id: PLAN-659
status: drafting
feature_name: crash-forensics-verify-infra
author: [agent]
created_at: 2026-09-19
updated_at: 2026-09-19
plan_revision: 1
current_step: 0
total_steps: 10
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-010]
affects: [auto-lang/vm, auto-lang/ui, auto-man, auto-os/ui-gallery]
---

# [PLAN-659] crash-forensics-verify-infra

## 0. 变更摘要

PLAN-642 终审收口遗留的**稳定性债与验证基建债**合成专项，两块：

1. **块 A：F1 崩溃族根因（P642-D3 承接，用户 2026-09-19 裁定 a）**——
   画廊 VM 实例间歇性 exit 127 静默死亡（≈1/4 长会话，死亡时机随机，
   P625 时代预存）。前序收窄弹药全部就绪待承接：bash 127 = fastfail
  （0xC0000409）/ 栈溢出（0xC00000FD）/ 高位截断三源（crashprobe 控制
   实验；AV→139、panic-unwind→101 均排除）；fastfail 按设计绕过 WER
   （LocalDumps 0 dump 机理得解）、两死法绕过 panic 审计钩子（死亡零
   审计行吻合）；审计日志判读法（死亡时 101=panic 有痕 / 127 无痕=栈
   溢出或 fastfail）；主线程 32MB 栈在档（tokio/wgpu 小栈线程与 P574
   型自设栈为剩余候选面）；audit log 两条画廊相关 panic 线索（wgpu
   offscreen source_texture invalid=截图路径、min>max 布局断言）；soak
   复现脚本常驻（ui-gallery tests/p642_soak.py）。
2. **块 B：验证基建四小件（P642-D14/D15 承接）**——①MCP fixture 派发
   不可达任意 handler（AddrGo 实证静默无操作）→ trigger 扩展直查
   handler 注册表；②`fs.canonical` 失败返原路径（unwrap_or(path)，
   native.rs:9397）vs 语料 `can == ""` 哨兵语义漂移；③Vue 臂构建预存红
   （folder-music × lucide-vue-next 0.312.0 无导出——020 上 web 臂画廊
   的前置）；④并行测试 flaky 三成员（plan609 / generate_rust_ui /
   display_family——并行红隔离绿）+ 画廊 boot deps 噪音。

## 1. 目标

### 块 A（崩溃专项）
- **G-A1 现场捕获**：F1 死亡被工具链（cdb/WinDbg attach 或等效）捕获
  至少一次并给出线程/模块/死法定罪（栈溢出 vs fastfail + 故障帧）。
- **G-A2 根因修复或护栏**：按捕获证据修复（深递归迭代化/stacker、
  CRT/堆 fastfail 定位）或落地防护（线程栈参数矫正、失败降级），修复
  后定向 soak 显著低于基线复现率。
- **G-A3 两条 panic 线索清偿**：wgpu offscreen 纹理失效（截图路径）与
  min>max 布局断言（audit log 在案）各自归因修复——独立于 A1 复现
  成败（它们是确定性 panic，修复面自洽）。

### 块 B（验证基建）
- **G-B1 fixture 派发全量可达**：`autoui_fixture` trigger 支持任意
  msg handler 派发（namespaced_handler_fn_name 注册表直查）。
- **G-B2 canonical 语义定案**：`fs.canonical` 失败语义与语料哨兵
  契约对齐（选边实施：shim 返 "" 或语料 fs.exists 门控），027 NavTo
  两条 toast.error 路径可达。
- **G-B3 Vue 臂复绿**：folder-music 图标名修正（lucide 0.312 有效集）
  或 lock 升级，`auto build` 全绿，020 可上 web 臂画廊。
- **G-B4 flaky 与噪音清偿**：并行 flaky 三成员根因定案（tempdir/资源
  竞争）+ 处置（隔离/串行化）；画廊 boot deps 噪音消除。
- **非目标**：F1 的跨仓通用崩溃框架（仅画廊 VM 形态）；auto serve/
  daemon 崩溃面（PLAN-658 域）；改 lucide 上游。
- **受影响仓库**：auto-lang（vm native / ui mcp_server / renderer /
  auto-man 测试基建）+ auto-os（ui-gallery 语料与 vue 构建）。
- **成功判据**：AC-01..08 全过；既有画廊内嵌矩阵零回归。

## 2. 架构方案

- **块 A**：工具链优先 cdb attach（LocalDumps 对 fastfail 已证无效；
  WER 例外注册 `HKLM ... AeDebug` 仅在用户授权系统级改动时考虑，默认
  attach 形态）。复现编排 = p642_soak.py 长会话画像（2+ 轮全遍历+截图+
  029 双开）× 多实例，attach 下等待死亡。判读优先级：死亡退出码（bash
  127 原生码确认）→ cdb 现场（线程栈、故障模块）→ 与 audit log 零行
  交叉验证。G-A3 两条 panic 以 audit log 复现路径直修（wgpu 纹理失效
  → 截图 offscreen 纹理生命周期/重建护；min>max → 布局 clamp 调用点
  求值守卫）。
- **块 B**：①mcp_server tool_fixture 的 trigger 臂由 on_with_input_for
  扩展为 handler 注册表直查（namespaced_handler_fn_name 同键，命中即
  call_fn 派发——与渲染路径同构）；②canonical 语料面清点（全语料
  `can == ""` 哨兵 grep）后选边——倾向 shim 返 ""（语料语义权威，
  unwrap_or(path) 是 shim 实现侧漂移）；③语料 folder-music → lucide
  0.312 有效名（FolderOpen+Music 组合或 folder-cd 等近义，以安装版
  导出表为准）；④flaky 三成员逐个隔离成因（tempdir 唯一性/共享资源/
  cwd 依赖），修复或 nextest 层面串行化兜底。
- **执行布局**：worktree 组 `D:/autostack/.wt/lang-659/{auto-lang,
  auto-os}`（Plan 529 布局）。块 A 的 soak 复现与 PLAN-658 的画廊实例
  排他（避免双实例端口/单实例冲突——两计划并进时错峰跑长会话）。

## 3. 技术栈

- Rust：`crates/auto-lang/src/vm/native.rs`（fs.canonical shim）、
  `crates/auto-lang/src/ui/mcp_server.rs`（fixture trigger）、
  `crates/auto-lang/src/ui/iced/renderer.rs` + `snapshot/offscreen`
  （wgpu 纹理生命周期）、layout 断言点、`crates/auto-man/src/vue.rs`
  测试基建（flaky）。
- 工具：cdb/WinDbg（Windows SDK 调试器，winget 安装）、audit log
  （PLAN-575 exit-audit 基建）、p642_soak.py。
- 语料：examples/ui/027（NavTo 哨兵）、020（web 臂画廊图标面）、
  auto-os ui-gallery（deps 噪音）。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户 2026-09-19（PLAN-642 F1 处置裁定，方案 a）："以预存间歇性原生
  不稳定立足本条目——PLAN-642 修复面照常收口，崩溃根因另立专项；先行
  动作 = exit(127) 枚举 + 追踪"→ 枚举已完成（P642-D3 判定段），本计划
  承接根因。
- 用户 2026-09-19（本计划立项指示）："崩溃专项（P642-D3）……小件：
  P642-D15（Vue folder-music 预存红）、D14（fixture 派发扩展 +
  fs.canonical 语义）——这两个问题合成一个新计划处理。"
- 前序弹药全部在册：KNOWN-DEBT-AND-RISKS.md P642-D3（裁定+枚举结果+
  判读法+工具建议）、P642-D14、P642-D15；PLAN-642 归档 §9 终审记录。

### 4.2 关键实证（前序成果索引）

| 事实 | 证据位置 |
|---|---|
| bash 127 = fastfail/栈溢出/截断；AV→139、panic→101 | P642-D3 枚举小步结果（crashprobe 控制实验） |
| fastfail 绕过 WER；两死法绕过 panic 钩子 | 同上（0 dump + 零审计行机理） |
| audit log 两条画廊 panic 线索 | `%LOCALAPPDATA%/auto-desktop/exit-audit.log` 扫描（wgpu offscreen source_texture / min=160 max=-96） |
| 主线程 32MB 栈；tokio/wgpu 小栈候选 | .cargo/config.toml /STACK:33554432 |
| fixture 派发：SetAddr 可达 / AddrGo 静默无操作 | P642-D14①（addr_editing 翻转判别） |
| canonical unwrap_or(path) | native.rs:9397-9402 |
| folder-music 预存红 | P642-D15①（NavSidebar.vue × lucide-vue-next 0.312.0） |
| flaky 三成员并行红隔离绿 | P642-D15②（master 同特征） |

### 4.3 背景

- P625：F1 首观测（inv1 master 二进制）；P574：自设 4MB 执行线程栈
  在案；PLAN-642 §8.3：复现画像（≈1/4 长会话，死亡页随机）。
- PLAN-575：exit-audit 基建（panic hook → code=101 审计行）。
- PLAN-646：Alt+drag 截图/offscreen 纹理面（wgpu 线索的邻近域）。

## 5. 详细设计

（T-00 工具链就位后回填：attach 编排形态、cdb 命令脚本、判读流程图；
canonical 选边依据表——语料哨兵使用面清点结果；flaky 三成员各自成因
与处置表。）

### 规范增量

| delta_id | op | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/vm/native-fs.md（或并入既有 vm spec） | 无 → `fs.canonical` 失败语义契约（返 ""，语料哨兵权威） | 语料/VM 语义对齐为 enduring 契约 | AC-05 |
| SD-02 | add | docs/specs/auto-lang/ui/overview.md 测试面 | 无 → MCP fixture trigger 派发契约（注册表直查，msg handler 全量可达） | E2E 驱动能力面 | AC-04 |
| SD-03 | add | docs/specs/auto-lang/ui/overview.md 稳定性节 | 无 → F1 崩溃族判读法（退出码映射/审计判读/工具链流程）+ 根因结论 | 稳定性知识沉淀 | AC-01/02 |
| SD-04 | modify | docs/specs/auto-lang/vm/plans.md | 沉淀行追加 PLAN-659 | 账目 | — |

## 6. 测试设计

- **单测**：canonical 失败返 ""（不存在的路径/非法盘符）；fixture
  trigger 派发 msg handler（AddrGo 类，addr_editing 翻转断言）；
  min>max 布局断言修复点回归；wgpu offscreen 纹理守卫回归（headless
  可测面）。
- **E2E**：027 地址栏非法路径 → toast.error 可达（B2 验收面）；020
  web 臂画廊构建+挂载；fixture 派发驱动 toast 生命周期全链（复用
  PLAN-642 t18 脚本）。
- **soak（块 A 主验收）**：p642_soak.py 多实例×多轮（attach 或
  判读法监控），死亡/存活记账；修复后对照基线（≈1/4）。
- **门禁**：`cargo t` 日常档 + `cargo tf`（收口）；flaky 处置后并行
  全套稳定绿 ×3 连跑。

## 7. 验收标准

- **AC-01**：F1 死亡至少一次被工具链捕获并归因（线程/模块/死法），或
  等效证据链收敛（排除法+强化探针定罪到可修根因）。（G-A1）
- **AC-02**：根因修复/护栏落地，修复后定向 soak（≥基线 3 倍暴露量）
  零死亡或复现率显著下降并有判读法背书。（G-A2）
- **AC-03**：wgpu offscreen 与 min>max 两条 panic 线索清偿（修复+回归
  测试；audit log 不再新增同型）。（G-A3）
- **AC-04**：fixture trigger 可派发任意 msg handler（AddrGo 实证 +
  单测）；t18 驱动脚本全链复跑通。（G-B1）
- **AC-05**：canonical 失败语义对齐语料哨兵（选边实施+E2E：027 NavTo
  两条 toast.error 路径可触发）。（G-B2）
- **AC-06**：Vue 臂 `auto build` 复绿；020 web 画廊挂载可见。（G-B3）
- **AC-07**：flaky 三成员并行全套 ×3 连跑全绿（或根因定案+显式处置
  在案）；画廊 boot deps 噪音消除。（G-B4）
- **AC-08**：门禁零新增红；画廊既有内嵌矩阵零回归。

## 8. 执行步骤

（原子任务；每步完成后追加 [✅] 证据行）

- [ ] **T-00 工具链就位与编排定案（块 A）**
      cdb 安装验证（winget/SDK）+ attach 编排（soak 画像下实时 attach/
  自动重启循环）+ 判读流程固化（退出码→原生码→现场交叉）；产出回填
  §5。若环境不可安装，产出等效替代（gflags/Application Verifier/
  vectored handler 探针族）并记录限制。
- [ ] **T-01 定向 soak 复现与现场捕获（块 A）**
      p642_soak.py 多实例长会话 ×N（排他 PLAN-658 实例），死亡现场
  cdb 捕获 → 线程栈/模块/死法定罪。AC-01。
- [ ] **T-02 F1 根因修复/护栏（块 A）**
      按 T-01 证据实施（深递归迭代化/stacker、线程栈参数、CRT fastfail
  定位或防护降级）；修复后 soak 对照。AC-02。
- [ ] **T-03 wgpu offscreen 纹理 panic 清偿（块 A，独立可做）**
      audit log 线索复现（截图路径 offscreen source_texture invalid）→
      生命周期/重建守卫修复 + 回归。AC-03 上半。
- [ ] **T-04 min>max 布局断言清偿（块 A，独立可做）**
      min=160/max=-96 形态归因（布局求值产生倒序界）→ clamp 守卫或
  根修 + 回归。AC-03 下半。
- [ ] **T-05 fixture trigger 派发扩展（块 B）**
      mcp_server tool_fixture trigger 臂直查 handler 注册表；AddrGo
  实证 + t18 脚本全链。AC-04。
- [ ] **T-06 fs.canonical 语义定案与实施（块 B）**
      语料哨兵面清点（全语料 `fs.canonical` 调用+判空用法 grep）→ 选边
  （预期 shim 返 ""）实施 + 单测 + 027 E2E。AC-05。
- [ ] **T-07 Vue folder-music 修复（块 B）**
      lucide 0.312 导出表对照 → 语料图标名修正 → vue build 复绿 →
      020 web 画廊验证。AC-06。
- [ ] **T-08 并行 flaky 三成员排查（块 B）**
      逐成员隔离成因（tempdir 唯一性/共享资源/cwd 依赖）→ 修复或 nextest
  层串行化处置；并行全套 ×3 稳定绿。AC-07 上半。
- [ ] **T-09 画廊 deps 噪音 + 收口（块 B）**
      deps 物化 vs pac.at 声明的生成器侧修正（不物化或自动声明）；
  AC-01..08 终验 + spec 增量落稿 + 门禁（cargo t / tf）+ 债核销
  （D3 判定段更新/D14/D15）。

## 9. 复审记录

```yaml
stage: new
plan_id: PLAN-659
plan_revision: 1
outcome: pass        # 契约就绪,待用户确认后进入 work
code_commit: N/A
task_ids: []         # T-00..T-09 待执行
evidence: 前序弹药索引（§4.2 八项事实表,全部可追溯 P642-D3/D14/D15 与
  终审记录）;授权链（用户 2026-09-19 两笔裁定/指示）
next: work（用户确认契约后:建 worktree 组 D:/autostack/.wt/lang-659/
  {auto-lang,auto-os} → T-00 工具链就位）
```

## 10. 待澄清事项

- cdb/WinDbg 系统安装（winget/SDK）若需管理员权限或系统级注册
  （AeDebug），呈报用户后再动——默认用户级 attach 形态。
- AC-01 的"等效证据链收敛"兜底边界：若 N 轮 soak（≥ 基线暴露量 4 倍）
  零复现，以判读法+剩余排除面出具"未复现+护栏在位"结论呈报用户裁定
  是否收口（不静默判 pass）。
- canonical 选边若语料清点发现哨兵用法面广且行为依赖现状（unwrap_or
  语义被既有语料依赖），翻案为语料门控侧并记录。
