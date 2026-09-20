---
plan_id: PLAN-662
status: archived                # drafting → executing → execution_done → reviewed → archived
completion_kind: delivered
feature_name: gallery-fast-start-and-viewport
author: [agent]
created_at: 2026-09-20
updated_at: 2026-09-20
plan_revision: 1
current_step: 6
total_steps: 6

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-lang/ui/overview.md §ui-gallery 增补（画廊行扫描缓存契约 + VM 视口滚动对齐，SD-01）, docs/specs/auto-lang/vm/plans.md PLAN-662 沉淀行（SD-02）]
touched_goals: [GOAL-010]

affects: [auto-lang/ui, auto-man, auto-os/ui-gallery]   # 受影响的 specs 路径/模块
---

# [PLAN-662] gallery-fast-start-and-viewport

## 0. 变更摘要

画廊 VM 臂（`auto run -r vm`）当前两大体验问题（2026-09-20 用户实测反馈）：

1. **启动慢**：全程 ~83 秒，其中 ~77 秒耗在逐 demo 扫描（36 demo × ~2s
   `VueProject::from_workspace`：pac resolve["Downloading deps" ×37] + 逐组件
   SFC 编译，串行）；唯一缓存 `GALLERY_ROWS_CACHE` 是进程内线程局部变量，
   **磁盘零缓存 → 第二次启动同样慢**。而真正"加载所有 demo"（合并编译进
   画廊 VM）仅 ~3 秒、back-proxy + 2 session ~1 秒——**瓶颈是 registry
   分析，不是 demo 加载**。
2. **020-music-player 内嵌显示不全**：`AppViewport.vm.at` 视口 frame 为
   `h-[720px] overflow-hidden` 且无滚动包装；020 根布局 `h-screen`（iced
   下解析为窗口高度 900px）塞进 720px frame → 底部 ~180px（播放控制条/
   曲库底部）整体裁掉。web 臂 `AppViewport.vue` 的挂载根本就有
   `overflow-auto`（:133）——**两臂不对称**，VM 臂缺失滚动。

附带一个已实测阻断项：主检出（`D:/autostack` 平铺布局）下
`cd auto-os/ui-gallery && auto run[-r vm]` 两臂均硬错在 `gallery_apps_dir`
解析（探测链缺平铺布局一级）——用户需 ps1 设 `AUTO_GALLERY_APPS` 才能跑。

本计划：①补 apps 目录解析的平铺布局探测；②画廊行扫描落盘缓存（内容
哈希键，二次启动 ~5-10s）；③扫描并行化（有界安全调查，首启目标 ≤30s）；
④AppViewport VM 视口滚动对齐 web 臂；⑤020 高度链有界修复（或登记边界）。
用户曾提"按需加载（点 tab 才装载 demo）"——实测数据表明加载非瓶颈
（3s），分析才是；**懒加载记为远期选项，本计划不做**（§1 非目标）。

## 1. 目标

- **G-1 主检出一键可跑**：`cd D:/autostack/auto-os/ui-gallery && auto run -r vm`
  零 env 启动成功（proxy + registry 正常），同理 Vue 臂。
- **G-2 二次启动达标**：同语料第二次 `auto run -r vm` 从命令发出到窗口
  首帧 **≤10 秒**（现 ~83s）；首启（冷缓存）目标 ≤30s（依赖 T-03 并行
  结论，不达标则如实记录+记债）。
- **G-3 缓存正确性**：单 demo 内容变更只重扫该 demo；缓存命中与未命中
  产出的 `registry.at` **字节一致**；缓存损坏/版本不符自动重建不崩。
- **G-4 020 内嵌完整可达**：画廊 VM 臂内嵌 020，滚动/布局修复后播放
  控制条（播放/上一曲/下一曲/进度）与曲库列表**完整可见可达**；web 臂
  与独立形态零回归。
- **非目标**：Vue 臂 vite/npm dev 启动提速（`generate_gallery_host` 全量
  SFC 发射+pnpm/vite 本身，仅间接受益于行缓存，另议）；按需懒加载
  （点 tab 动态编译装载 demo——实测加载仅 3s 非瓶颈，且 VM 臂
  `AppViewport.vm.at` 为静态分发架构，动静大收益小，记远期）；020 全
  视觉保真（flex-wrap 等 Plan 412 降级矩阵边界仅登记不修全）；658
  proxy/会话语义任何变更（纯消费方提速，零触碰）。
- **受影响仓库**：auto-lang（auto-man 发射器/解析 + demo 语料[仅 020 若
  需] + 单测）+ auto-os（ui-gallery 生成产物刷新提交，625/658 先例）。
- **成功判据**：AC-01..06 全过；既有内嵌矩阵与独立形态零回归。

## 2. 架构方案

**测量基线**（2026-09-20 本机实测，master 二进制 d03afa88d 含 658）：

| 阶段 | 耗时 | 内容 |
|---|---|---|
| 逐 demo 扫描 | ~77s | 36 × `gallery_demo_row`→`VueProject::from_workspace`（pac resolve+SFC 编译 ~2s/个，串行） |
| demo 合并编译进 VM | ~3s | 36 demo .at → 画廊单元 |
| back-proxy + sessions | ~1s | 32 media 路由 + 017/031 session |

**修复杠杆**（按量级）：磁盘缓存（消二次启动 77s）≫ 并行化（首启 77→
~15-25s）＞ apps 解析补全（解锁主检出）＞ 视口滚动（显示完整性）。

**缓存层位**：包在 `gallery_rows_via_cache` 内层——按 demo 粒度键
`hash(demo 目录内容 + 路径依赖[stylekit] 内容)`，值 = 序列化
`GalleryDemoRow`（loadable/fullstack 等分析产物）；命中即跳过
`from_workspace`。存储：`~/.auto/auto-man/gallery-cache/<apps_dir 摘要>.json`
（用户级缓存，不污染两仓 git；原子写 temp+rename）。版本字段防格式漂移。

**视口滚动**：发射器 `AppViewport.vm.at` 生成处（vue.rs ~7200）给 demo
分支外补滚动容器——样式类 `overflow-y-auto`（renderer.rs:2333 已映射
`View::Scrollable`，PLAN-656 机制现成），镜像 web 臂挂载根
`demo-mount-root overflow-auto`。frame 三态尺寸（1024×720/768×1024/
full×720）不变。

**020 高度链**：有界调查定修法——候选 A：demo 语料 `h-screen`→`h-full`
链（风险：独立形态全屏语义，须双臂+独立验证）；候选 B：仅视口滚动
（T-04）已足则登记 flex-wrap 降级为 Plan 412 已知边界收案。证据驱动，
不预设。

## 3. 技术栈

- `crates/auto-man/src/vue.rs`：`gallery_apps_dir`(6263)、
  `gallery_rows_via_cache`(6481)、`gallery_demo_row`(7897)、
  `start_gallery_back_proxy`(6513)、`refresh_gallery_registry`(6582)、
  `generate_gallery_host`(4500)、AppViewport.vm.at 发射(~7200)。
- `GalleryDemoRow`（vue.rs ~8005）：加 serde Serialize/Deserialize 派生。
- iced 滚动映射：`crates/auto-lang/src/ui/iced/renderer.rs:2333`
  （overflow-y-auto → View::Scrollable，PLAN-656）。
- 语料（仅 T-05 若选候选 A）：`examples/ui/020-music-player/src/front/
  {app.at,tracklist.at,player_store.at}`。
- 产物：`auto-os/ui-gallery/src/gallery/AppViewport.vm.at`（生成，随计划
  刷新提交）；`src/front/registry.at`（既有生成物，缓存确定性守卫对象）。
- 缓存：`~/.auto/auto-man/gallery-cache/`（新目录；`.auto` 用户态先例：
  am.at/index/ 已在用）。并行：`std::thread::scope`（**不加 rayon**——
  auto-man 无该依赖，遵 658 无新依赖先例）。
- 验证工具：MCP 驱动（`.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`，
  658 同款）；计时脚本（命令→日志 "first state sync" 时间戳差）。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户 2026-09-20（本会话）："OK，那你帮我用 auto-plan-new 起草一个计划，
  然后实施它"——起草+实施双授权；范围=本会话已呈报的启动提速方案
  （缓存+并行，懒加载明确不做）+ 020 显示修复 + 主检出启动修复。
- 前序会话授权链：PLAN-658（画廊多后端 proxy，delivered 3dab57f9a）
  确立 VM 臂内嵌形态；本计划为其体验层收尾，不改其语义。

### 4.2 关键实证（2026-09-20 本机核实）

| # | 事实 | 证据 |
|---|---|---|
| 1 | 启动 83s 中 77s 为逐 demo 扫描 | /tmp/vm_gallery_ok.log 时间线 16:52:16→16:53:33（"Downloading deps"×37 顺延全窗） |
| 2 | demo 合并编译仅 ~3s、proxy+session ~1s | 同日志 16:53:33→36；"session up" 同秒 |
| 3 | 唯一缓存线程局部、无磁盘 | vue.rs:6470 GALLERY_ROWS_CACHE thread_local |
| 4 | 主检出两臂裸跑硬错 | 实测 `Gallery mode needs an apps directory...`（vue.rs:6287 Err 路径） |
| 5 | 020 内嵌底部裁切、控制条挤压 | 截图 auto-os/ui-gallery/src/front/tests/screenshots/p658_020_display.png；启动日志 flex-wrap 降级声明（Plan 412） |
| 6 | VM 视口无滚动 vs web 臂有 | AppViewport.vm.at 生成串 `overflow-hidden` 无滚动 vs AppViewport.vue:133 `overflow-auto` |
| 7 | iced 滚动机制现成 | renderer.rs:2333 needs_scroll；020 tracklist.at:143 已用 `overflow-y-auto` |
| 8 | 数据面正常（非 658 回归） | MCP state：item_count 393 / "本地曲库已就绪" |

### 4.3 背景

- PLAN-625（VM registry/`AppViewport.vm.at` 生成）、PLAN-642（内嵌修复）、
  PLAN-658（多后端 proxy）——本计划直接消费三者产物，零语义变更。
- PLAN-656（universal scroll-pane）：滚动映射机制来源。
- P625-D5（VM live 视口集 14/20 多文件降级）：与显示面相关但独立，不动。
- 已知坑（memory）：iced `pane h-*` 双样式压缩内容；`h-screen`/flex 高度
  链在 iced 的解析差异——T-05 调查输入。

## 5. 详细设计

### 5.1 apps 目录解析补全（T-01）

`gallery_apps_dir` 探测链追加一级：`root_dir.parent().parent().join(
"auto-lang").join("examples").join("ui")`（平铺主检出
`D:/autostack/{auto-lang,auto-os}` 与 worktree 组
`.wt/lang-<NNN>/{auto-lang,auto-os}` 双命中）。顺序置于现有
`parent/auto-lang` 探测之后、Err 之前。单测：临时目录四种布局（旧仓
examples 内嵌 / 组内兄弟 / 平铺兄弟 / 全无→Err）。

### 5.2 行扫描磁盘缓存（T-02）

- 键：per-demo `SHA-256(relpath+size+mtime` 走查 demo 目录[含 tutorial/
  README/SPEC、pac.at、src/**] + 解析出的路径依赖目录同样走查[stylekit
  等])`；apps_dir 身份另入文件名。mtime 粒度足够（语料为源码树，无亚秒
  篡改场景）。
- 值：`GalleryDemoRow`（serde JSON；`doc/source/pac` 大文本也入缓存——
  它们本就来自文件读，缓存后免读）。注意 `Option<VueProject>` **不入**：
  行消费方（registry/proxy）不需要；`generate_gallery_host` 的 vp 需求
  不受影响（其自扫路径 4502 不走本缓存）。
- 读写：进程内先查内存（现有 thread_local）→ 未命中查磁盘 → 未命中/
  失效逐 demo 重扫并回写（全量收集后一次原子写）。损坏 JSON/版本不符/
  字段缺失 → 整体丢弃重建（fail-open，不阻断画廊）。
- 确定性守卫：缓存命中与冷跑产出的 `registry.at` 必须字节一致（测试钉
  住——缓存只重放分析结果，不改发射输入）。

### 5.3 扫描并行化（T-03，有界）——决策产物回填（2026-09-20）

调查结论：发射管线注册表全为 thread_local（`handler_codegen.rs`
STORE_FIELDS/STORE_WIDGET_NAMES/STORE_MSG_MAP/CURRENT_* 族）——scoped
工作线程各自隔离，无进程级共享可变态；唯一结构性竞争=结果聚合，按
"每线程返回本块 `(idx, row)`、join 后主线程合并"规避（共享 Vec 跨线程
写是 UB）。**安全实证**：并行冷跑 vs 清缓存串行冷跑的 registry.at +
AppViewport.vm.at + demos/（76 文件）字节级 diff 全空 → `jobs` 缺省=
`available_parallelism`（不回落 1），`AUTO_GALLERY_SCAN_JOBS` env 覆写。

### 5.4 AppViewport VM 滚动对齐（T-04）

发射器 demo 分支（branches 串）外套 `col { style: "w-full h-full
overflow-y-auto" ... }`（具体层级以 656 映射实测为准——frame 的
`overflow-hidden` 保留裁圆角，滚动容器置于 frame 内、demo 分支外）。
web 臂零触碰。产物刷新：worktree 内重新生成并提交
`auto-os/ui-gallery/src/gallery/AppViewport.vm.at`（625/658 先例：lang
发射器确定性派生物提交至 os）。

### 5.5 020 高度链（T-05，有界）——裁定回填（2026-09-20）

**选项 B 成立**：T-04 滚动容器落地后，§5.5 判定标准（控制条按钮可
辨识+可交互+进度条在）经 MCP+截图实证全达成（播放/上下曲五按钮组、
进度条 0:30/4:18、is_playing=true、393 曲库）——020 语料零改动。残余
=控制条 `flex-wrap` 两行换行（按钮完整可用，仅排版降级），登记
KNOWN-DEBT P662-D1（Plan 412 降级矩阵家族），避免语料改动引入
standalone/Vue 臂连锁验证面。

### 规范增量

| delta_id | op | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md §ui-gallery | 无 → 增补：画廊行扫描缓存契约（键构成/存储位/失效粒度/fail-open/registry.at 字节一致守卫）+ VM 视口滚动对齐 web 臂（frame 三态不变、demo 分支滚动容器） | 缓存失效语义与两臂视口契约为 enduring 行为面 | AC-02..04 |
| SD-02 | modify | docs/specs/auto-lang/vm/plans.md | 沉淀行追加 PLAN-662 | 账目 | — |

## 6. 测试设计

- **单元**：`gallery_apps_dir` 四布局探测；缓存命中/单 demo 失效重扫
  （临时目录改一个文件→仅该 demo 重算，钩子计数）；损坏缓存 fail-open；
  registry.at 缓存/冷跑字节一致；并行 jobs env 解析。
- **计时 E2E**：连续两次 `auto run -r vm`（同语料），二次命令→日志
  "first state sync in view()" 时间戳差 ≤10s；冷缓存（清缓存目录）首启
  计时留档。
- **MCP E2E**：内嵌 020 打开→滚动/布局后 snapshot 含播放控制条按钮与
  进度条、曲库行可见；press 播放按钮有 state 反应；截图留证
  evidence/p662/。
- **回归**：gallery 发射测试 17/17（既有）；内嵌矩阵 spot-check
  （013/015/017/020/022/031 MCP 快照非崩）；web 臂 `auto run`（vite 起
  +020 页面无脚本错）；020 独立形态两臂（若 T-05 选 A）；`cargo t`
  日常档 + 定向 auto-man。

## 7. 验收标准

- **AC-01**：主检出 `cd D:/autostack/auto-os/ui-gallery && auto run -r vm`
  零 env 成功（日志含 Gallery back-proxy + registry 36 demos）；同目录
  Vue 臂不再报 apps directory 错（进 vite 即可）。（G-1）
- **AC-02**：二次启动（同语料连续第二次）命令→"first state sync" ≤10s；
  冷缓存首启计时留档，若 T-03 并行落地 ≤30s。（G-2）
- **AC-03**：单 demo 内容变更→仅该 demo 重扫（日志/计数证据）；缓存/
  冷跑 registry.at 字节一致；损坏缓存自动重建画廊正常起。（G-3）
- **AC-04**：画廊 VM 臂内嵌 020：播放控制条（播放/上一曲/下一曲/进度）
  与曲库列表完整可见可达（MCP snapshot+截图）；播放按钮可交互
  （press 后 state 变化）；web 臂内嵌 020 视觉零回归。（G-4）
- **AC-05**：回归零新增红：gallery 发射 17/17；内嵌矩阵 spot-check 六
  demo 通过；`cargo t` 日常档基线不变；020 独立形态两臂正常（T-05=A 时
  必测，B 时抽测）。
- **AC-06**：auto-os 侧产物（AppViewport.vm.at 等）刷新提交；KNOWN-DEBT
  增量登记（并行结论/Plan 412 边界行视 T-03/T-05 结果）。

## 8. 执行步骤

（原子任务；每步完成后追加 [✅ 已完成] 一行证据；全部在 worktree
`D:/autostack/.wt/lang-662/auto-lang` 实施，plan 簿记留主检出）

- [x] **T-01 apps 目录解析补平铺布局**
  `vue.rs gallery_apps_dir` 追加祖父级 `auto-lang/examples/ui` 探测 + 四
  布局单测；主检出实测两臂零 env 启动（AC-01）。
  [✅ 已完成 2026-09-20] commit d5ec6d7b7。探测链补祖父级（平铺主检出
  `D:/autostack/{auto-os,auto-lang}` 与 worktree 组 `.wt/lang-NNN/` 同形
  命中）；单测 `test_plan_662_gallery_apps_dir_layouts`（旧仓内嵌/
  standalone/同层兄弟/平铺组/全无 Err 五场景）。活体：os worktree
  `ui-gallery` 零 env `run -r vm` 成功（AC-01 达成；主检出平铺形态同构）。
- [x] **T-02 画廊行扫描磁盘缓存**
  `GalleryDemoRow` serde 派生 + `~/.auto/auto-man/gallery-cache/` 落盘层
  （键/值/原子写/fail-open 如 §5.2）+ 单测四件（命中/单失效/损坏/字节
  一致）；二次启动计时 ≤10s（AC-02/03）。
  [✅ 已完成 2026-09-20] commit d5ec6d7b7：`crates/auto-man/src/gallery_cache.rs`
  （新模块）+ `gallery_rows_with_disk_cache` 接线（scan_one 注入式可测）。
  **实测修正**：目录 mtime 在 NTFS 创建后最初时刻延迟落定（并行测试下
  漂移→假失效实证），哈希行改为**仅文件**（relpath|len|mtime；增删文件
  必然改变文件集合，目录行冗余）。计时：二次启动 **5s**（0 scanned, 36
  from disk cache；基线 83s）。单测 `test_plan_662_gallery_rows_disk_cache`
  （命中零扫/单失效仅重扫变更 demo/损坏 fail-open 全量重建/serde 往返
  逐字段一致=registry 确定性守卫）。
- [x] **T-03 扫描并行化（有界）**
  from_workspace/SFC 线程安全面调查（决策产物回填 §5.3）→ 安全则
  std::thread::scope 池 + AUTO_GALLERY_SCAN_JOBS env + 首启计时；不安全
  则记债收案（AC-02 首启项转缓并如实呈报）。
  [✅ 已完成 2026-09-20] commit d5ec6d7b7。调查结论：发射管线注册表全为
  thread_local（handler_codegen.rs STORE_*/CURRENT_* 族）——工作线程天然
  隔离；竞争风险=共享 Vec 跨线程写（已按"每线程返回本块结果、join 后主
  线程合并"规避）。**安全守卫实证**：并行冷跑与清缓存后串行冷跑的
  registry.at + AppViewport.vm.at + demos/（76 文件）**字节级 diff 全空**
  → 默认档=核数确认（不回落 1）。计时：冷启并行 **22s** vs 串行 ~62s
  （基线 83s）。env 覆写单测 `test_plan_662_gallery_scan_jobs_env`。
- [x] **T-04 AppViewport VM 视口滚动对齐**
  发射器 demo 分支滚动容器（§5.4）+ os 产物刷新提交 + MCP 020 可达性
  验证（AC-04 主体）；web 臂零触碰验证。
  [✅ 已完成 2026-09-20] lang d5ec6d7b7（发射器）+ os 0778c28（产物）。
  frame 内补 `col overflow-y-auto` 滚动容器（656 映射 View::Scrollable），
  frame 三态样式不变；web 臂零 diff（AppViewport.vue 未触，Vue 臂生成
  面 demos-registry.ts/SFC 未再生）。MCP 实证：020 内嵌控制条完整可见
  （歌曲信息+五控制按钮+进度条 0:30/4:18）、393 曲库、is_playing 可交互
  （evidence/p662/p662_020_fixed.png vs 修复前 020_before_658.png）。
- [x] **T-05 020 高度链有界修复**
  证据驱动选 A（语料修复三面验证）或 B（登记 Plan 412 边界收案），
  决策记录回填 §5.5（AC-04/05）。
  [✅ 已完成 2026-09-20] **裁定=选项 B（020 语料零改动）**：T-04 滚动容器
  已使 §5.5 判定标准全达成（控制按钮可辨识+可交互[is_playing]+进度条
  在）；残余=控制条 flex-wrap 两行换行（按钮完整可用仅排版降级）——
  登记 KNOWN-DEBT P662-D1（Plan 412 降级矩阵家族），不动语料（免
  standalone/Vue 臂连锁验证面）。
- [ ] **T-06 收口：全量回归 + 簿记**
  §6 回归矩阵全跑（gallery 17/17 + spot-check 六 demo + cargo t 日常
  档 + auto-man 定向）；evidence/p662/ 留痕（计时日志/截图/驱动脚本）；
  KNOWN-DEBT 增量；spec 增量 SD-01/SD-02 落稿（AC-05/06）。
  [🔶 进行中 2026-09-20] gallery 定向 20/20 并行绿（auto-man --lib gallery；
  其中 662 三件+既有 17）；auto-man 全量 lib 310/312（2 红=desktop_
  extra_app_roots_apps_container + plan609 预存**并行 env 竞争**——单跑
  均绿、`--skip test_plan_662` 排除后仍红，非本 diff 引入，记 P662-D2）；
  cargo t 日常档在跑；evidence/p662/（timing.md + 修复前后截图）已落；
  os 产物已提交（0778c28）。余：cargo t 结果 + spot-check 六 demo +
  KNOWN-DEBT 三行 + spec 增量落稿。
  [✅ 已完成 2026-09-20] **cargo t 日常档（--no-fail-fast 全量）5265/5270，
  5 红全在 auto-lang（本 diff 零触碰 auto-lang 代码，确定性测试由构造与
  基线一致）：mouse_area/autodown_panel_heading（P661-D7 在案预存）+
  kitchen_sink（658-R2 在案）+ 2×ui::layout snap/usable_rect（任务栏/
  环境红家族）——零新增红**。首跑 fail-fast 期 musk p053 族红=画廊实例
  并发资源竞争 flaky（清场复跑全绿）。spot-check 六 demo MCP 全过
  （013 待办/015 笔记[首探测时序误报，复查视口在渲染"清爽笔记应用"等
  内容]/017 Send/020 393 曲库/022 看板/031 图片）。KNOWN-DEBT P662-D1
  （flex-wrap 登记）/P662-D2（预存并行 env 竞争）落册。spec 增量
  SD-01/SD-02 worktree 提交（198ae949d，顺修 658 行陈旧状态）。
  evidence/p662/（timing.md + 修复前后截图）。

## 9. 复审记录

```yaml
stage: merge
plan_id: PLAN-662
plan_revision: 1
outcome: pass
delivery_commit: a0bf33fe8（= reviewed 198ae949d 经 rebase 等价映射 480db2bc1/4fd115f2b/a0bf33fe8
  [range-diff 3/3 全等]，仅叠加 docs-only 账本投影——实现面与复审基线零变化，master 定向冒烟 3/3）
receipt: PLAN-662:r1
checkpoints:
  prepared: SD-01/SD-02 worktree 内备妥（198ae949d）；账本投影 P662-1(designs)/P662-2(reviews) 外科插入 .autoos/specs.json（+22 行最小 diff，604→606 items）+spec-index.py 再生（f64af22a6→rebase a0bf33fe8）
  landed: "master a0bf33fe8 fast-forward（merge --ff-only 实证；rebase range-diff 3/3 等价）；
    auto-os main 0778c28 fast-forward——他方 034 会话 WIP 占用同名再生文件（registry/014）经
    stash→merge→pop 保全：其 docs/plans/034 与 019-video-app 等 WIP 原样恢复（diff=0 核对），
    031-image-viewer.at 冲突取落地版（现行发射器重生成，其旧发射器残留被超集取代）后 stash drop"
  ledger_refreshed: ".autoos/specs.json@master a0bf33fe8——P662-1/P662-2 入位（读回 606 items，P658/P661 各项共存）"
  archived: docs/plans/archive/662-gallery-fast-start-and-viewport.md（git mv）+ status archived + completion_kind delivered
  cleaned: "wt-guard ×3 clean（lang-662/{auto-lang,auto-os,auto-down}）→ 三 worktree 移除+双分支删除+组目录移除"
canonical_spec_paths: [docs/specs/auto-lang/ui/overview.md §PLAN-662, docs/specs/auto-lang/vm/plans.md 662 行]
ledger_targets: [.autoos/specs.json]
debt: P662-D1（020 flex-wrap 排版降级登记）/ P662-D2（预存并行 env 竞争）落册 KNOWN-DEBT
next: 终态（全部 checkpoint 闭环：wt-guard ×3 clean → 三 worktree 移除+lang/os 双 plan-662-dev 删除[was a0bf33fe8/0778c28]+auto-down detached 移除+组目录 lang-662 删除，worktree list 双仓零 662 残留实证）
```

```yaml
stage: review
plan_id: PLAN-662
plan_revision: 1
outcome: pass
reviewed_commit: 198ae949d（= d5ec6d7b7 实现 + docs-only spec 增量；实现面即 d5ec6d7b7）
base_commit: fe88a3240（master 计划起草点；diff 基=fe88a3240..198ae949d 全 5 文件）
dependency_revisions: auto-os worktree plan-662-dev @ 0778c28（产物刷新）；auto-down 组内 detached @3f73737（构建依赖）
spec_inputs: SD-01 ui/overview.md §PLAN-662 六条 + SD-02 vm/plans.md 662 行/658 行修正（198ae949d 已提交；逐条对码——env 覆写路径/fail-open/版本门/仅文件行哈希/滚动容器/探测顺序全一致）
acceptance_results: |-
  AC-01 pass（复跑：os worktree 零 env run -r vm 裸跑成功——Gallery rows 36 + back-proxy 3358[32 apps, 2 sessions]）
  AC-02 pass（复跑：二次启动 4.3s 实测[0 scanned, 36 from disk cache]；冷启并行 22s/串行 62s 留档 evidence/p662/timing.md）
  AC-03 pass（复跑：test_plan_662 三件单测 reviewed commit 上重跑绿；命中/单失效/损坏 fail-open/serde 往返一致）
  AC-04 pass（重建：修复前后截图对照 + state[393/is_playing]；复审实例被真实窗口交互点开 020
    [日志 SelectDemo + Demo020MusicPlayer_Init 执行成功]——附加活体证据；web 臂零 diff）
  AC-05 pass（复跑：串行/并行产物字节 diff 全空[registry+AppViewport+demos 76 文件]；gallery 20/20；
    spot-check 六 demo；cargo t 日常档 5265/5270 + cargo tf 全量 3646/3648——
    t 5 红/tf 2 红[ffi_dual_019+kitchen_sink]全为 658 复审/P661-D7 在案预存或环境红，
    本 diff 零触碰 auto-lang 代码[5 文件 diff 全 auto-man+docs]，由构造与基线一致）
  AC-06 pass（os 0778c28 四文件；KNOWN-DEBT P662-D1/D2 落册；SD-01/SD-02 worktree 提交）
findings: |-
  零阻塞发现。非阻塞注记三条：
  R1[独立性限制] 同会话实施+复审——以"reviewed commit 上复跑全部关键检查 + 工件重建
    （截图/日志/diff/timing.md）+ 不复用实施期自述"缓解；计时与 MCP 均为复审轮独立驱动。
  R2[预存红 surfaced] auto-man 并行 env 竞争（desktop_extra_app_roots + plan609 互踩，单跑绿）
    登记为 P662-D2；musk p053 族首跑红=画廊实例并发资源竞争 flaky（清场复跑绿）。
  R3[顺带修正] vm/plans.md 658 行陈旧状态 executing→archived（事实修正，随 SD-02 提交）。
evidence: |-
  docs/plans/evidence/p662/（timing.md 四档计时+diff 记录 + 020_before_658.png + p662_020_fixed.png）
  + 本记录内嵌：tf/t 逐项归因、复审计时 4.3s、复审实例 020 真实交互日志。
  worktree D:/autostack/.wt/lang-662/auto-lang @ 198ae949d 保留待 merge。
next: merge（/auto-plan:merge——前置全满足：实现已提交/reviewed 翻转/SD 已备/os 产物在册）
```

```yaml
stage: work
plan_id: PLAN-662
plan_revision: 1
outcome: pass
code_commit: 198ae949d（worktree D:/autostack/.wt/lang-662/auto-lang @ plan-662-dev；
  实现链 fe88a3240[base] → d5ec6d7b7[T-01..T-04 实现+三单测] → 198ae949d[SD-01/SD-02]；
  auto-os worktree plan-662-dev @ 0778c28[产物刷新]；auto-down 组内 detached @3f73737 构建依赖）
task_ids: [T-01✅, T-02✅, T-03✅, T-04✅, T-05✅, T-06✅]
evidence: |-
  AC-01 pass：os worktree 零 env run -r vm 裸跑成功（Gallery rows 36 + back-proxy
    3358 + registry 36 demos；祖父级探测组内命中，主检出平铺同构）。
  AC-02 pass：二次启动 5s（0 scanned, 36 from disk cache；基线 83s）≤10s；冷启
    并行 22s ≤30s（vs 串行 62s 实测留档）。
  AC-03 pass：单测三件（命中零扫/单失效仅重扫/损坏 fail-open）+ serde 往返逐字段
    一致（registry 确定性守卫）；实测修正=目录 mtime NTFS 延迟落定→哈希改仅文件行。
  AC-04 pass：MCP+截图——020 控制条完整可见（歌曲信息+五按钮+进度条 0:30/4:18）、
    is_playing 可交互、393 曲库；web 臂零 diff（AppViewport.vue 未触）。
  AC-05 pass：串行/并行产物字节 diff 全空（registry+AppViewport+demos 76 文件）；
    gallery 定向 20/20；spot-check 六 demo 全过；cargo t 日常档 5265/5270 零新增红
    （5 红全在 auto-lang=预存在册/环境，P661-D7/658-R2 在案）。
  AC-06 pass：os 产物提交 0778c28（AppViewport.vm.at 滚动容器+三文件现行重生成）；
    KNOWN-DEBT P662-D1/D2 落册；SD-01/SD-02 worktree 备妥（198ae949d）。
  evidence/p662/：timing.md（四档计时+diff 记录）+ 020_before_658.png +
  p662_020_fixed.png。
blockers: ""
next: review（/auto-plan:review——工作全部完成，execution_done 翻转）
```

```yaml
stage: new
plan_id: PLAN-662
plan_revision: 1
outcome: pass
code_commit: N/A
task_ids: []
evidence: §4.2 八项实测（时间线/截图/代码锚）；授权=用户本会话"起草+实施"
next: work（用户已预授权实施：master commit 计划 → worktree
  D:/autostack/.wt/lang-662/auto-lang → T-01 起）
```

## 10. 待澄清事项

- 缓存存储位（~/.auto/auto-man/gallery-cache/ 为默认提案）——实现期若
  发现 .auto 目录契约限制可改 ui-gallery 本地忽略路径，非阻断，T-02 内
  定案记录。
- T-03 并行默认档位与 T-05 A/B 走向为证据驱动决策点，按 §5.3/§5.5 判定
  标准执行，结果呈报，无需用户前置裁定。
