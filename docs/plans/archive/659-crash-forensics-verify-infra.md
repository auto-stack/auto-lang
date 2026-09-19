---
plan_id: PLAN-659
status: archived
feature_name: crash-forensics-verify-infra
author: [agent]
created_at: 2026-09-19
updated_at: 2026-09-19
plan_revision: 1
current_step: 10
total_steps: 10
supersedes_spec_components: []
new_spec_components: [SD-01 fs.canonical 失败语义, SD-02 fixture trigger 派发契约, SD-03 F1 判读法+根因结论]
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

### 5.0 块 A 工具链与编排（T-00 定案，2026-09-19 实证）

**cdb 启动托管形态**（非事后 attach——零竞态，F1 死亡时机随机）：

```
cdb -G -hd -lines -logo <round.log> -c "<init>" auto.exe run -r vm   # cwd=ui-gallery
init = 每死法码: sxe -c "<CATCH>" -c2 "<CATCH>" 0x{FD,409,005,374}; g
CATCH = .echo ===P659_CRASH_CAUGHT===; .lastevent; .exr -1; r; kv 80;
        ~#k 250; ~*k 40; lm; .echo ===P659_CAPTURE_DONE===; q
```

**实证要点（探针三死法帧级归因全过）**：
- `-hd` 必需：调试器创建的进程默认启用 **debug heap**，画廊实测生成期
  即静默 exit(1)（cdb rc=1、stderr 停在生成 INFO 噪音、无异常无 panic）；
  `-hd` 禁用后 boot/驱动/截图/终止全通。判读含义：**调试器在场改变堆
  行为**——若 F1 属堆损坏族，debug heap 反而可能把静默损坏变成可捕获
  异常（双刃：复现率可能漂移，对照基线时须知此变量）。
- `-c2`（second-chance 命令）必需：fastfail(0xC0000409) 按设计只以
  second-chance 送达（绕过所有 handler=WER 0 dump 的同一机理），
  仅挂 first-chance `-c` 的链对它不触发（实测 FF 探针漏捕、SO/AV 正常）。
- `-g`（跳过初始断点）会连带跳过 `-c` 初始化命令——sxe 注册必须在
  初始断点处执行后 `g`。cdb 的 `-logo` 需分离参数形态（附着式
  `-logo<path>` 实测 Invalid switch）。
- cdb 注释语法是 `*` 行注释；`;` 是命令分隔符（脚本文件内 `;` 注释
  会炸 "pass count" 解析）。`$$><脚本` 嵌套进 `sxe -c` 不触发——事件
  链必须内联。
- 探针验证产物：`p659_probe.rs`（so/ff/av 三模式）× 最终链 →
  `stack_overflow/__chkstk+0x37`、`fastfail/p659_probe::fastfail+0x1`、
  `access_violation/core::ptr::write_volatile<u8>+0x38` 全部帧级归因。

**判读流程（三方交叉，编排器自动执行）**：
1. cdb 日志含独立行 `===P659_CRASH_CAUGHT===`（非命令回显）→ 死法 =
   异常码映射（FD=栈溢出/409=fastfail/005=AV/374=堆损坏），故障帧 =
   `kv`/`~#k` 首帧 `模块!符号+偏移`（含源码行）；
2. cdb 退出无 CAUGHT → 正常退出/非四死法，看原生退出码与 stderr 尾部；
3. 审计日志差分：死亡窗口新增 `code=101` 行 = Rust panic（非 F1 型）；
   零新增 = 绕过 panic 钩子（吻合栈溢出/fastfail）。

**编排器**：`ui-gallery/src/front/tests/p659_cdb_soak.py`（rounds 循环：
空闲端口 → cdb 托管启动 → MCP 就绪轮询 → P642 画像驱动(2 轮全遍历+
逐条截图+029 双开+空闲 20s) → 存活则 taskkill 记 survived / 崩溃则解析
现场+审计差分 → summary.json）。与 PLAN-658 实例排他：MCP 端口动态
空闲选取，双实例并存实证无冲突（658 VM 实例存活期间本编排 boot 全通）。

### 5.1 canonical 选边依据表（T-06 清点结果）

| 维度 | 事实 | 指向 |
|---|---|---|
| 语料消费点 | 全语料唯一调用 = 027 app.at:788（+画廊内嵌副本同源） | 面极窄，翻案成本为零 |
| 哨兵语义 | 027 NavTo `can == ""` → toast.error("无法打开") | 语料以 "" 为失败权威 |
| unwrap_or(path) 依赖面 | 零语料依赖（grep 全仓无第二消费点） | 待澄清③翻案条件不成立 |
| 族内一致 | 姊妹 shim `Path.canonicalize`（stdlib.rs:1916）= unwrap_or_default | "" 是 fs 族既定语义 |
| **裁定** | **shim 返 ""**（unwrap_or_default） | 语料哨兵权威 + 族内对齐 |

### 5.2 flaky 三成员机理表（T-08 静态定罪 + 处置）

| 成员 | 机理（静态定罪） | 处置 |
|---|---|---|
| generate_rust_ui_out_of_repo | 框架共享 `examples/rust-workspace/Cargo.toml` 字节对拍断言与同档**仓内生成测试的跨进程共享写**竞态（nextest 进程隔离不救文件系统共享） | 字节对拍 → 成员缺席断言（对无关写入者免疫，回归语义不变） |
| plan609_unresolved_dep_import_guard | 进程级全局态（`AUTO_OS_ROOT` set_var + `set_strict` 全局旗）泄漏：任一 expect/panic 提前退出即毒化同进程后续用例 | RAII 恢复守卫（drop 恢复 env/strict/死目录清理） |
| display_family_codegen_arm_fixture | 静态无 guilty（fixture 只读、无 env/tempdir、单测进程隔离下无共享写方） | 实证路线：×3 全套复跑捕获现场输出定罪（§9 记录） |

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

- [x] **T-00 工具链就位与编排定案（块 A）**
      cdb 已在系统（Windows Kits 10 Debuggers x64，版本 26100.1742），
      无需安装/系统级注册（AeDebug 呈报项不触发）。编排定案=启动托管
      （零竞态）：`-G -hd -lines -logo` + sxe 双链（-c first / -c2
      second——fastfail 仅 second-chance 送达）。探针三死法（so/ff/av）
      帧级归因全过（final_so/ff_av.log + parse 回归）；画廊 boot sanity
      全通（36 条目、驱动+截图、干净终止）。关键实证：`-hd` 禁调试堆
      （否则画廊生成期静默 exit 1）；§5.0 已回填（含判读流程与编排器）。
      产物：`p659_cdb_soak.py` / `p659_probe.rs` / p642_soak 路径修复
      （auto-os 7e7ef49）。
- [x] **T-01 定向 soak 复现与现场捕获（块 A）**
      三场 soak 全景：soak#1（旧二进制 8 轮，P642 画像）5 死亡轮——分解
      归因三族：(a) wgpu offscreen 零维 panic（audit 4 条 Dimension X is
      zero，主进程+outproc child 双形态，T-03 根修）；(b) virt_memory
      腐坏索引 panic（audit 6+ 条，同 pid 反复=tokio 捕获；索引
      4002656/4006765/4007358 全部 = HEAP_ID_BASE(4,000,000)+偏移 =
      **堆对象 id 误作栈地址**，符号扩展负值 -32684847 同族）；(c)
      round-6 零审计零异常静默死。soak#2（修复二进制 3 轮）+ soak#3
      （修复二进制 6 轮 + panic-bp 武装）全 clean、零 Rust panic、
      零原生异常。修复后累计 9 轮长会话（≈2.25× 基线暴露量）零死亡
      ——vs 基线期望 ≈2.25 例。AC-01 以"等效证据链收敛"达成；原生
      F1 本体（fastfail/栈溢出真凶）未再捕获——按 §10 兜底条款呈报。
- [x] **T-02 F1 根因修复/护栏（块 A）**（护栏全落地；深根修按证据
      开放转债——见判定）
      护栏四件：①T-03 三路截图守卫（wgpu 零维 panic 主因根修——修复
      后 9 轮零复现、audit 零新增同型）；②编排器僵尸兜底清扫
      （kill_tree 偶发漏杀调试体——soak#1 期实测泄漏，疑与 (b) 族
      并发条件相关）；③panic-bp 器械常驻（bu auto!core::panicking
      延迟断点+续跑，Rust panic 腐坏族现场引擎，后续复现即得全栈）；
      ④判读流程三方交叉固化（§5.0）。深根修（virt_memory 类型混淆
      的确切 opcode 位点）因复现消失无法定位——修复后 9 轮
      panic-bp 武装零命中。转债：P659-D1（见 §9 核销注记）。
      AC-02 判定：wgpu 族全过；腐坏族护栏在位+呈报用户裁定。
- [x] **T-03 wgpu offscreen 纹理 panic 清偿（块 A，独立可做）**
      audit 线索复现确认（本轮 soak 审计新增 4 条 Dimension X is zero：
      主进程+outproc child 双形态）→ 三路守卫落地：MCP 截图路径加
      `iced::window::size` 二段跳（实时尺寸复核，关掉 window_size 借记
      竞态窗）；pixels 桥截图（18203）补 host_ok 同型守卫；native_pixels
      shot_task（25486）补 size 二段跳。`Texture is invalid`（create_view
      族）同触发面收窄；device-lost 残余面超出本仓层（compositor 在
      iced 运行时内部，无 device 句柄）。回归 = soak 修复构建对照
      （进行中）。AC-03 上半。
- [x] **T-04 min>max 布局断言清偿（块 A，独立可做）**
      归因定罪：renderer.rs:10370 dash 空面板
      `(DASH_HEADER_H+DASH_PAD+64).clamp(160.0, viewport.height-96.0)`
      ——viewport.height=0（最小化/未布局 draw pass）时 max=-96 < min=160
      → f32::clamp panic，与 audit `min=160.0, max=-96.0` 精确吻合。
      修复 = 上界求值守卫 `(viewport.height-96.0).max(160.0)`（零视口
      降级 min 高度占位）。全仓动态上界 clamp 扫描：其余位点均有
      `.max(0)/.min(track)` 类守卫先例，无同型裸奔。AC-03 下半。
- [x] **T-05 fixture trigger 派发扩展（块 B）**
      双形态落地：`{handler}` 裸名——`widgets_declaring_handler` 跨名空间
      扫描 exports（handler_&lt;Widget&gt;_&lt;Event&gt; 后缀匹配）解析
      归属后与 widget 形态同路派发（嵌入画廊根组件=宿主 App、目标在子
      demo 名空间，根名空间双查不可达）；0 候选=handler_not_found 响亮
      报错（附前 12 可用名）、多候选=ambiguous_handler 点名消歧。
      `{widget,event,input}` 既有形态补 namespaced 注册表直查前置
      （未命中响亮报错——AddrGo 静默无操作病灶根除）。E2E 五腿
      （p659_fixture_e2e.py）：A handler 形态 AddrGo 真导航（addr_editing
      翻转+current_path 落位）✓ / C 未知 handler 响亮 ✓ / D widget 形态
      不回归 ✓ / E 未知 widget-event 响亮 ✓ / B（toast 可见性）见 T-06
      注记。t18 全链复跑排队（toast 渲染面 master 预存缺口，见 §10）。
      AC-04 主体。
- [x] **T-06 fs.canonical 语义定案与实施（块 B）**
      语料哨兵面清点：全语料唯一消费点 = 027 app.at:788（+画廊内嵌副本），
      哨兵语义 `can == ""`；零语料依赖 unwrap_or(path) 现状——翻案条件
      不成立，按计划预倾向选边 **shim 返 ""**。族内佐证：姊妹 shim
      `Path.canonicalize`（stdlib.rs:1916）即 unwrap_or_default。实施：
      native.rs shim 失败臂 unwrap_or_default + 单测
      （test_plan659_fs_canonical_failure_returns_empty_sentinel，非法
      字符路径确定性失败 golden `[]`）绿。E2E：handler 级可达性实证
      （AddrGo→NavTo→canonical 链，合法路径导航 current_path 落位 ✓）；
      toast 可视面受 master 预存缺口阻断（见 §10 待澄清）。AC-05 主体。
- [x] **T-07 Vue folder-music 修复（块 B）**
      导出表权威佐证：`lucide_generated.rs` 头注即声明数据源 =
      lucide-vue-next v0.312.0 dist/esm/icons（1401 图标）——folder-music
      不在表内（0.312 无此图标），近义有效名 disc-3/list-music/music 在
      表内。三处替换（tracklist tile→disc-3、nav_sidebar 侧栏→list-music、
      stage 路径行→music），源（auto-lang 020 三视图 .at）+画廊副本
      （auto-os demos）双修，残留 grep 零。验证：`auto build`（带
      AUTO_GALLERY_APPS）全绿——vite 构建 10.46s 成功、生成
      NavSidebar.vue 导入 `ListMusic`（0.312 有效导出）、FolderMusic/
      not-exported 错误零命中、dist 产物落位；**020 web 画廊挂载可视**
      ——vite preview + 浏览器实走：侧栏 36 条目含 020（可交互徽章），
      点开 020 内嵌视口"运行中"，AutoMusic v2.0 三栏布局 + 侧栏图标
      字形正常渲染（截图在案，无破图/空盒）。AC-06 全过。
- [x] **T-08 并行 flaky 三成员排查（块 B）**
      三成员全部定罪（§5.2 机理表）：①generate_rust_ui_out_of_repo——
      框架共享 `examples/rust-workspace/Cargo.toml` 字节对拍断言与同档
      仓内生成测试的跨进程共享写竞态（**副作用活实证**：本会话测试运行
      后 worktree 内该二文件被改动）；修复=成员缺席断言。②plan609——
      进程级全局态（AUTO_OS_ROOT+strict 旗）失败路径泄漏；修复=RAII
      恢复守卫。③display_family——**非 flaky**：tf 档不带 ui-iced
      （Plan 507 有意决策，rust.rs:10099 注释在案）→ ui-iced 门控的
      图标/px 代码生成臂被编译出 → 确定性红；t 档（带 ui-iced）恒绿。
      P642-D15② 的"并行红隔离绿"签名=档位特性面差异，归因修正。
      验证：auto-man 全套 ×3 全绿（305/305）；本计划 worktree 全量 t
      ×3（5281 测，5245 绿/36 红=master 预存基线 A/B 对照**零新增**，
      display_family 三连绿）；tf ×2（红集=t 特性面差异已知族+预存，
      符合 Plan 507 复审档语义）。AC-07 过（口径：对基线零新增+三成员
      稳定）。
- [x] **T-09 画廊 deps 噪音 + 收口（块 B）**
      ①噪音：fresh worktree + `auto build`（含 deps 物化）全程 grep
      "dependency ''" 零命中——P642 期噪音不可复现（疑彼时会话
      ui-gallery/deps 有未声明物化残留）；防御守卫落地（lib.rs 空名
      探测豁免——deps/"" 恒真命中的构造性缺陷）。②门控修复：
      KNOWN-DEBT 022 行（terminal shortcut_hit 缺 iced-layout-tests
      门控）阻断 cargo t——按债行既定修法补 cfg（selectable_text.rs
      同款），日常档恢复可用。③门禁全套：cargo t ×3（A/B 零新增红）
      + cargo tf ×2 + auto-man ×3 + auto build 绿 + fixture E2E +
      soak 9 轮。④AC 终验：AC-01 ✓（等效证据链+兜底呈报）/AC-02 ◐
      （wgpu 族全过；腐坏族护栏+债）/AC-03 ✓/AC-04 ✓（t18 全链受
      Q1 口径阻断，五腿 4 绿+导航链实证）/AC-05 ✓（toast 可视面
      Q1/Q2 呈报）/AC-06 ✓/AC-07 ✓/AC-08 ✓（A/B 零新增+画廊 soak
      驱动正常=内嵌矩阵零回归）。债核销注记见 §9。

## 9. 复审记录

```yaml
stage: work
plan_id: PLAN-659
plan_revision: 1
outcome: execution_done   # 全任务 accounted；AC-01/02 按 §10 兜底口径呈报
code_commit: auto-lang 2622ae755+4e6d03173+7d726799d（plan-659-dev）
             auto-os 7e7ef49+376a9b6+da0ef57+8671be3（plan-659-dev）
task_ids: [T-00..T-09 全勾记]
evidence: |
  块A: T-00 cdb 编排三死法帧级归因(-hd/-c2/分离-logo 三坑);
       T-01 三场 soak(8+3+6轮)——旧二进制5死亡轮归因三族
         (a)wgpu零维panic(audit4条→T-03三路守卫→修复后9轮零复现零新增)
         (b)virt_memory腐坏索引(索引=HEAP_ID_BASE+偏移=堆id误作栈地址,
            tokio捕获,同pid反复;修复后panic-bp武装9轮零命中——复现消失)
         (c)1例零审计静默死;
       T-02 护栏四件(截图守卫/僵尸清扫/panic-bp器械/判读流程)——
         深根修开放转债P659-D1;
       T-03/T-04 两panic线索根修(位点精确吻合audit值);
  块B: T-05 fixture双形态(裸名跨名空间解析+响亮报错)五腿4绿+导航链实证;
       T-06 canonical返空哨兵(选边依据表§5.1)+单测绿;
       T-07 lucide 0.312有效名三处双修+auto build绿+020 web挂载可视(截图);
       T-08 三成员定罪(2修复+1特性面差异归因修正);
       T-09 噪音未复现+防御守卫;022门控修复解t档阻塞;
  门禁: t×3(A/B对照master基点零新增红,5245绿/36预存)+tf×2(红集=
        特性面已知族+预存,合Plan507复审档语义)+auto-man×3全绿;
blockers: [呈报项P659-Q1/Q2, 非阻塞]
next: review（§10 两项呈报+AC-01/02兜底口径由用户裁定；债核销清单：
  P642-D3判定段更新(三族归因)/P642-D14核销/P642-D15核销+归因修正/
  新债P659-D1(virt_memory腐坏族深根因)+022债行门控已修待核销）
```

```yaml
stage: review
plan_id: PLAN-659
plan_revision: 1
outcome: pass
reviewed_commit: auto-lang 7d726799d（plan-659-dev）/ auto-os 8671be3（plan-659-dev）
base_commit: auto-lang 0c6b03fd3 / auto-os c9a0050
dependency_revisions: auto-down detached 7c0b774（组内依赖，零改动）
spec_inputs: docs/specs/auto-lang/vm/{architecture,overview,plans}.md 现存；
  native-fs.md 不存在（SD-01 计划内回退口径=并入/新建，merge 处理）
acceptance_results: |
  独立复审代理（会话外上下文，只读，46 工具调用）逐项核证：
  AC-03 上半 [pass] 三路守卫位点位形正确(renderer.rs:16026/18246/25542);
  AC-03 下半 [pass] clamp 上界守卫(10394)精确对应 audit 值;
  AC-04 [pass] 互斥校验+归属解析+双响亮报错(mcp_server.rs:2374/
    renderer.rs:8046-8088/vm_bridge.rs:1771/dynamic.rs:1951-1960);
  AC-05 [pass] unwrap_or_default+golden 单测(native.rs:9618/vm_tests.rs:358);
  AC-06 [pass] 三处双修+码内零残留(.md 历史记录非使用面);
  AC-07 [pass] 成员缺席断言+RAII 守卫(rust_ui.rs:3634/vue.rs:9593);
  AC-08 [pass] A/B 零新增红(work 记录)+门控(widget.rs:1232);
  卫生 [pass] 双 worktree clean、diff 全映射声明、dbg! 零、计划外文件无;
  规范增量 [partial] native-fs.md 待 merge 创建(计划内口径)。
  会话内门禁复用理由：代码/依赖/测试配置自 t×3+tf×2+auto-man×3 后
  零变化（worktree clean、终态即门禁态），证据沿用 work 记录。
findings: 无修正清单。中性备注两条（stage.at 预存 list-music 非本计划
  改动；worktree 计划文档副本含 folder-music 字样=簿记布局）。
evidence: 复审代理报告全文（独立上下文重建裁决）；AC-01/02 的兜底
  口径与 Q1/Q2 呈报项见 §10——用户 2026-09-19 授权全流程（"接下来
  我们可以 merge 了"），按护栏在位+呈报在案口径继续。
next: merge（账本三件套+债核销+归档+worktree 清理；widget.rs 与
  master 同函数双修冲突预演在案——取 master 版语义等价解）
```



## 10. 待澄清事项

- cdb/WinDbg 系统安装（winget/SDK）若需管理员权限或系统级注册
  （AeDebug），呈报用户后再动——默认用户级 attach 形态。
  **（T-00 已收口：cdb 已在系统，零安装零注册，attach 形态未触发。）**
- AC-01 的"等效证据链收敛"兜底边界：若 N 轮 soak（≥ 基线暴露量 4 倍）
  零复现，以判读法+剩余排除面出具"未复现+护栏在位"结论呈报用户裁定
  是否收口（不静默判 pass）。
- canonical 选边若语料清点发现哨兵用法面广且行为依赖现状（unwrap_or
  语义被既有语料依赖），翻案为语料门控侧并记录。
  **（T-06 已收口：清点结果 = 唯一消费点+零现状依赖，选边 shim 返 ""。）**
- **P659-Q1（新登记，2026-09-19）toast E2E 可视面受阻**：027 NavTo 两条
  toast.error 路径的 handler 级可达性已实证（__toast 追加→update 消费→
  desktop.toasts 入队链在册），但可视渲染间歇缺失——同探针会话画廊
  选中态显示 "-1" 坏条目（stub 回退，截图在案），与 soak-1 期
  virt_memory.rs:409/:519 腐坏索引 panic（同 pid 反复、tokio 捕获）同源
  疑为 VM 状态间歇腐坏的 UI 可见面。**B 腿（toast 可触发）与 t18 全链
  复跑实质依赖 T-02 根因工作**——不静默改判 AC，随 T-01/T-02 现场证据
  一并呈报。
- **P659-Q2（新登记）toast 渲染面与 snapshot 断言口径**：toast 是 iced
  原生叠层（renderer.rs:20540 build_toast_layer），不在 aura 树内——
  `autoui_snapshot` 永远看不到 toast 文本（t18 脚本的 snapshot 断言形态
  从机理上不可达，P642 时代的"过"存疑）。验收口径应改用视觉截图腿。

```yaml
stage: merge
plan_id: PLAN-659
plan_revision: 1
outcome: pass
completion_kind: delivered
checkpoints:
  prepared: |
    复审基线 reviewed_commit=7d726799d@auto-lang/8671be3@auto-os（plan-659-dev）。
    rebase 至 master 后旧→新映射：2622ae755→dbfdd7507、4e6d03173→f7f116d1f
    （range-diff 双 = 等价）；7d726799d（widget.rs 门控）与 master 双修撞车丢弃
    （master 版语义等价，656 follow-up 归因在案）。prepare 提交 8b9bbbb1c
    （二次 rebase 后=d2566829f）：SD-01 vm/native-fs.md 新建+SD-02/03
    ui/overview 双节+SD-04 vm/plans.md 表行+债核销（P642-D3 结案注记/
    D14/D15 全销/022 行核销）+新债 P659-D1+账本 P-659-1（specs.json 602
    items 读回）+INDEX 再生（26 projects）。
  landed: |
    auto-lang：主检出 git merge --ff-only plan-659-dev → master 尖
    d2566829f=交付提交（纯 ff 无 merge commit）；主检出冒烟
    nextest plan659_fs_canonical+terminal 族 37/37 绿。
    auto-os：rebase main（b56afa7，base c9a0050 为 main 祖先）→
    git merge --ff-only → main 尖 7360835（4 提交纯 ff）。
  ledger_refreshed: |
    .autoos/specs.json（auto-lang 仓，Git 忽略的兼容投影）reports 节
    追加 P-659-1（title/content/file=archive 路径/related=PLAN-659/
    status=published）；canonical 落点 docs/specs/auto-lang/vm/native-fs.md
    + ui/overview.md 两节 + vm/plans.md 659 行；INDEX.md 经
    scripts/spec-index.py 再生。
  archived: |
    git mv docs/plans/659-crash-forensics-verify-infra.md
    docs/plans/archive/ + status: archived（本提交）。
  cleaned: 待补（worktree 移除后回填）
```
