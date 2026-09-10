---
plan_id: PLAN-597
status: archived                      # drafting → executing → execution_done → reviewed → archived
feature_name: cffi-engine-face
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-10
plan_revision: 1
current_step: 7
total_steps: 7

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/auto-bindgen/project.md, docs/specs/auto-lang/vm/design/ffi.md, docs/specs/auto-lang/trans/overview.md]
new_spec_components: []
touched_goals: [GOAL-006, GOAL-013]   # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-bindgen, auto-lang/vm ffi, auto-lang/trans c]   # 详见 §5 规范增量
---

# [PLAN-597] cffi-engine-face：C 通道驱动 autoterm_core.dll 12 符号面 + engine 胶水 Auto 化路径裁定

> **来源**：auto-term `docs/designs/004-engine-autoization-roadmap.md` §5③
> （P-C 相位第二批；首批 = PLAN-595 已归档 2026-09-09）。
> **前提在库**：manifest 类型层 FnPtr/abi（595）、a2c 闭包原型入头（595）、
> MSVC 构建先例 `scripts/build-ctrlc-a2c.cmd`（595）。

## 0. 变更摘要

用 C 通道驱动引擎 12 符号 FFI 面，交付三件事：

1. **a2c 全量驱动**：Auto 源（`fn.c` 声明 12 符号）经 a2c → MSVC 链接
   autoterm_core.dll → 真 PTY 会话端到端（spawn cmd → write_input echo →
   feed_ready → take_dirty_rows → row_text 见锚点 → kill/free）；
2. **VM 标量子集**：12 符号中 8 个标量/CStr 友好符号经 c_ffi 冒烟
   （可行则落 manifest+用例，不可行则 gap 记录）；4 个缓冲/出参型符号
   （take_dirty_rows/row_text/row_style/cursor）为 VM 现行封送不可达边界，
   出分析记录（归 004⑥ a2r 生成器轨道）；
3. **路径裁定书**：engine 胶水 Auto 化三径对照（a2c 链入 Rust 宿主 /
   a2r+侧车签名 / VM 运行期），产出推荐裁定，回填 004 §4 P-C。

## 1. 目标

- **G1**：a2c 产物 exe 能独立驱动真实引擎会话（echo 锚点经
  take_dirty_rows + row_text 可见），全程零手写 C；
- **G2**：VM c_ffi 对引擎面的可达子集有行为证据，不可达边界有据记录；
- **G3**：engine 胶水 Auto 化路径有可复核裁定（判据：junction 成本/
  降级能力/运行时依赖/与 004⑥ 的衔接）。

### 非目标

- 不改 auto-term 仓任何代码（引擎与 12 符号契约冻结，只读消费）；
- 不做 004⑥（a2r 生成 Rust FFI 模块）——本计划为其产出口径输入；
- 不做 at-gen engine.rs 的实际替换（裁定书先行，替换属后续计划）；
- 与 PLAN-596（430 dep 管线 trait/泛型/回调）无涉——不同轨。

## 2. 架构方案

一套语义三个 lowering（004 §3.5）：本计划验证引擎面在 **a2c 直发
（S 静态链接形态）** 与 **VM 运行期（标量子集）** 两径的可达性。
a2c 侧链接机制二选一（T-01 裁定）：MSVC linker 直接以 DLL 为输入
（C 导出可直接链）vs 先产 import lib；缓冲出参在 C 侧为母语
（Auto 数组 + `.@` 取址先例：`02_types/004_pointer`）。

**引擎 DLL 解析序**（沿用引擎先例）：`AUTOTERM_ENGINE_DLL` env →
组内 ../auto-term 产物 → D:/autostack/auto-term 主检出 target 产物；
构建脚本内建解析，不引入 junction（红线）。

## 3. 技术栈

a2c（trans/c.rs）、auto-bindgen manifest（595 类型层）、MSVC 2022
（vcvars64 自举，同 595 脚本范式）、auto-term cdylib（只读）。

## 4. 需求分析与背景调查

- **授权**：用户 2026-09-09 指示就 004 §5③④「各自做出计划文件」——
  本计划为立项起草（drafting），**执行待后续 /auto-plan:work 指令**；
- 12 符号签名全量核对在案（auto-term ffi.rs:47-243）：标量友好 8 个
  （spawn/write_input(CStr 近似,行内无 NUL 即等价)/feed_ready/resize/
  interrupt/is_exited/kill/free），缓冲出参 4 个（上列）；
- 595 已证：a2c `fn.c` 声明不发射（原型来自真头/链接面）、MSVC 警告
  面（C4113 等）不阻塞、`auto test -d tests/a2c_tests.at` 为 a2c 快照
  门禁、`.wrong.c/.h` 反哺法；
- VM c_ffi：manifest 仅 builtin 字典（JSON 文件无运行期加载面）——
  engine 专属 manifest 不宜入内置字典，加载面缺失是 T-05 的已知前置。

## 5. 详细设计

### 12 符号 fn.c 声明形态（T-02 蓝本）

```auto
use.c <stdint.h>
fn.c autoterm_engine_spawn(cols int, rows int, program cstr) ptr
fn.c autoterm_engine_write_input(h ptr, bytes ptr, len int)
fn.c autoterm_engine_feed_ready(h ptr) int
fn.c autoterm_engine_take_dirty_rows(h ptr, out_rows *int, cap int) int
fn.c autoterm_engine_row_text(h ptr, row int, out_buf ptr, cap int) int
fn.c autoterm_engine_row_style(h ptr, row int, out *uint, cap int) int
fn.c autoterm_engine_cursor(h ptr, out_row *int, out_col *int) int
fn.c autoterm_engine_resize(h ptr, cols int, rows int)
fn.c autoterm_engine_interrupt(h ptr) int
fn.c autoterm_engine_is_exited(h ptr) int
fn.c autoterm_engine_kill(h ptr)
fn.c autoterm_engine_free(h ptr)
```

驱动流（快照语料 18_c_interop/004_engine_face）：spawn → 循环
feed_ready → take_dirty_rows → row_text 全行扫锚点 `CFACE_OK` →
printf 见证行 → kill → free → exit 0/1。

### T-01 链接机制（结论，2026-09-09 实证）

- **直接 DLL 输入：不可行**——MSVC linker 报 LNK1107（文件无效/损坏，
  0x2C0 处不可读；直接 DLL 输入是 GNU ld 特性）；
- **import lib：零成本可行**——rust cdylib 构建已副产
  `autoterm_core.dll.lib`（与 DLL 同目录），cl 直接链它；
- 运行期 DLL 解析：PATH 或 exe 同目录（构建脚本拷贝 DLL 至产物目录 =
  同目录分发契约）；
- 证据：最小手写 C（spawn/is_exited/kill/free）经 .dll.lib 链接运行
  `spawn=1 exited=0 LINK_OK`（tmp/p597/mini_lib.exe）。

### T-05 VM 子集口径

可行路径：c_ffi 增 JSON manifest 文件加载面（`load_manifest_file`，
非 builtin），engine 子集 JSON 放语料旁；VM 冒烟语料走 spawn(feed 无
UI)→write_input→feed_ready→is_exited→kill/free。若加载面成本超小改
边界，降级为 gap 记录（缓冲型 4 符号无论何径都不可达，一并入记录）。

### 路径裁定书（T-06 产物，回填 004 §4）

| 维度 | a2c 链入 | a2r+侧车 | VM 运行期 |
|---|---|---|---|
| junction | C 对象链入 Rust 宿主（extern 声明 ~30 行手写） | 手写体驻侧车文件 | 零（原生） |
| 降级能力 | 无（链接期定死） | 可 | 可（try-load） |
| 缓冲出参 | ✅ C 母语（本计划实证：row_text/take_dirty_rows 全过） | ✅ | ❌（4 符号封送不可达，实证） |
| 运行时依赖 | 零 | a2r-std 视用法 | VM 全量 |
| 适配场景 | 独立驱动程序/工具 | at-gen 宿主内胶水 | VM 应用 |

**裁定（T-06 产物，2026-09-09）**：
- **独立驱动/工具（引擎会话外）** → **a2c 链入**（本计划主证：12 符号
  全量、缓冲出参天然、零运行时依赖；唯一前置 = 引擎 ABI 头，长期应
  由 auto-term 侧随产物分发 `autoterm_engine.h`——本计划以语料内
  engine_abi.h 承接）；
- **at-gen 宿主内胶水（UI 应用进程）** → **a2r+侧车**（进程内共存
  iced/auto-lang 运行时，C 对象链入需额外 extern 面，侧车模式已有
  A2R_EXTERN_SIGS 既有机制）；
- **VM 应用** → 标量子集可用（本计划打通 use_scanner 潜伏缺口 +
  3 分派臂 + JSON manifest 加载面；VFACE_OK 三连绿）；缓冲型 4 符号
  维持不可达（封送边界，归 004⑥ a2r 生成器轨道消化）；
- 附带产出（VM 轨存量缺陷两枚，均已实测定位+规避口径）：① if 条件位
  内联 C-FFI 调用 + while 循环 = VM 挂起；② 循环计数器在嵌套 if 内
  赋值 = 控制流静默断裂。两者均已写入语料头注，留 VM 轨道修。

### 规范增量

| delta_id | 操作 | docs/specs/... target | before/after rule | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | auto-bindgen/project.md | before：manifest 消费面=VM 注册期拒绝 FnPtr+a2c 原生；after：+JSON 文件加载面（若 T-05 落）与 engine 面口径（builtin 不收 engine 专属） | engine manifest 归属语料侧 | AC-03 |
| SD-02 | modify | auto-lang/vm/design/ffi.md | before：FnPtr 注册期拒绝；after：+缓冲/出参型签名（数组出参）为 VM 封送不可达边界（4 符号实例） | 267 边界实证扩展 | AC-03 |
| SD-03 | modify | auto-lang/trans/overview.md | before：a2c c 互操作=fn.c 声明不发射；after：+12 符号面链接驱动先例（链接输入形态按 T-01 结论） | 引擎面首证 | AC-01/02 |

## 6. 测试设计

- a2c 快照：`18_c_interop/004_engine_face/`（声明+驱动流）；
- bindgen 单测：JSON 加载面 round-trip（若落）；
- 真机验收：`cargo build -p autoterm-core`（auto-term 侧，产出 dll）→
  脚本构建 a2c exe → 运行断言锚点与 exit 码；VM 冒烟（若落）；
- 基线护栏：a2c 套件 8 例存量失败不变；cargo tf/tt 两例基线固有失败不变。

## 7. 验收标准

- **AC-01**：a2c exe 独立驱动真引擎会话——stdout 含锚点见证行
  （row_text 读出的 `CFACE_OK`），exit 0；复跑可重复；
- **AC-02**：12 符号 fn.c 声明形态 + 驱动流入 a2c 快照（套件绿，
  零回归）；
- **AC-03**：VM 子集二选一有据——冒烟绿（含 manifest 加载面单测）或
  gap 记录（含缓冲型 4 符号封送边界分析）落入 SD-02 目标文档；
- **AC-04**：路径裁定书落稿（§5 表格实例化+推荐+依据）并回填
  auto-term 004 §5③；
- **AC-05**：套件零回归（a2c 8 例存量 + tf/tt 两例基线固有，数目与
  名单不变）。

## 8. 执行步骤

- [x] **T-01**（bounded investigation）链接机制验证：最小手写 C 样例
  链 autoterm_core.dll（直接 DLL vs import lib），结论回填 §5；
- [x] **T-02** Auto 驱动源 + 12 符号 fn.c 声明快照语料（.wrong 反哺法）；
- [x] **T-03** 构建脚本 `scripts/build-engine-face-a2c.cmd`（DLL 解析
  序 + MSVC + 布局冒烟）；
- [x] **T-04** 真机驱动验收（AC-01 运行取证）；
- [x] **T-05** VM 标量子集：加载面/冒烟 或 gap 记录（AC-03 二选一）；
- [x] **T-06** 路径裁定书 + 004 §5③ 回执 + DEBTS #10 增 597 条；
- [x] **T-07** 收口门禁：a2c 套件 + bindgen 单测 + cargo tf/tt 档
  （基线两例不变）。

依赖：T-02←T-01；T-03←T-01/02；T-04←T-03；T-05 独立；T-06←T-04/05。

## 9. 复审记录

stage: work | PLAN-597 | rev 1 | **pass** | code_commit=ddcf9ffb1 |
task_ids=T-01..T-07 | evidence=见下 | blockers=无 | next=review

### 执行证据(2026-09-09)

- **T-01**:直接 DLL 输入 = LNK1107(实证,GNU ld 特性不适用 MSVC);
  rustcdylib 副产 `autoterm_core.dll.lib` = 零成本 import lib;最小手写
  C 链接运行 `spawn=1 exited=0 LINK_OK`。结论回填 §5。
- **T-02**:快照 `18_c_interop/004_engine_face`(12 符号 fn.c 声明 +
  驱动流:spawn→write_input(usize len)→feed_ready/take_dirty_rows 轮询
  →row_text 扫锚点→kill/free,exit 0/1/2)。执行期两修:①a2c 未初始化
  固定数组发射 `= NULL`(非法 C)→裸声明;②引擎无 C 头→隐式声明段
  错误(spawn 假设返 int 截断句柄)→语料内 `engine_abi.h`(12 原型,
  write_input len 对齐 size_t)。
- **T-03/04**:`scripts/build-engine-face-a2c.cmd`(import lib 链接 +
  DLL 同目录分发 + vcvars 自举);真机 **CFACE_OK / exit 0**(echo 经
  take_dirty_rows+row_text 见证;仅良性警告 C4702)。
- **T-05(全量冒烟支线,未降级 gap)**:①修 `use_scanner` 潜伏缺口
  ——任何 VM 模式 `use.c`(含 `<math.h>`)都在文本扫描层被当模块
  解析报 Module not found(Plan 216 存量断裂,实测坐实);补 .c 点式
  分支(单测 dot_form_c_import);②`load_manifest_file` JSON 加载面 +
  codegen `.json` 回退;③三支分派臂(([Int,Int,CStr],Ptr)/
  ([Ptr,CStr,Size],Void)/([Ptr,Int,Int],Void));④engine_face.json
  (8 标量符号,单测 load_manifest_file);⑤冒烟 **VFACE_OK 三连绿**
  (spawn→write_input→feed 观察到字节批→resize→kill/free)。
  **VM 轨两枚存量缺陷实测定位**(均已在语料头注规避):
  D1 = if 条件位内联 C-FFI 调用 + while 循环 → VM 挂起(单发条件位
  正常);D2 = 循环计数器在嵌套 if 内赋值 → 控制流静默断裂(后续
  print 全失)。缓冲出参 4 符号(row_text/row_style/take_dirty_rows/
  cursor)VM 封送不可达,实证归 004⑥。
- **T-06**:裁定书落 §5(独立工具→a2c 链入/宿主内胶水→a2r+侧车/
  VM 应用→标量子集);auto-term 004 §5③ 回执 + DEBTS #10 增 597 条
  (主检出未提交批次)。
- **T-07**:a2c 套件 **110 ok / 基线 9 失败行不变**(engine_face 绿,
  净 +1);bindgen 6/6;cargo tf **3484/3485**(唯一失败=基线
  vue::test_charts_gallery_compiles,595 复审已验基线固有);cargo tt
  **3000/3001**(同上);新增单测 2 件全绿。执行中期曾现"门禁挂起"
  ——定位为冒烟调查期孤儿 auto.exe 进程干扰(资源占用),清场后
  22s/36s 干净收官,非代码回归。


stage: new | PLAN-597 | rev 1 | outcome: pass | next: work
（起草即绪：路径/命令/签名均经仓库核位；执行授权待用户指令。）

stage: review | PLAN-597 | rev 1 | **pass** | reviewed_commit=b400a2821 |
base_commit=e056d1d86（merge-base master） | dependency_revisions=auto-term@ec623572
（主检出；004/DEBTS/specs.json 回执批次未提交，work 记录已披露） |
spec_inputs=plan §5 规范增量 rev1（SD-01/02/03，三目标文档现行文本核位：bindgen
project.md:27 消费面行 / ffi.md:34 FnPtr 注册期拒绝行 / trans overview.md:53 a2c 行） |
acceptance_results=AC-01..05 全 pass | findings=F-1/F-2 均记录性非阻塞 |
evidence=见下 | next=merge

### 复审证据(2026-09-10，独立复现，非采信执行摘要)

- **基线**：worktree `D:/autostack/.wt/lang-597/auto-lang` 干净无脏改；
  分支恰一提交 b400a2821（work 记录引用的 ddcf9ffb1 为前身，amend 差异
  仅补 `test/vm_engine_face/engine_face.json` 67 行——即 T-05 声称的
  manifest 产物，最终提交已含）。
- **AC-01 ✅ 复现**：`scripts/build-engine-face-a2c.cmd` 重跑构建成功
  （import lib 链接 + DLL 同目录分发；仅基线良性警告 C4702）；产物
  engine-face-a2c.exe 两连跑均 `CFACE_OK` / exit 0。exe 由快照
  expected.c 编译（快照被 a2c 套件字节锁）——「零手写驱动 C」口径成立；
  engine_abi.h 为 fixture 本地 ABI 头（计划 §5 已声明承接方式）。
- **AC-02 ✅ 复现**：套件 110 ok / 9 失败；另在合并基点 e056d1d86
  detached 重建同跑得 **109 ok / 9 失败，失败名单与 597 分支逐行
  IDENTICAL**（diff 空）——净 +1 = 004_engine_face 绿，零回归实证。
- **AC-03 ✅ 复现**：VM 冒烟 `engine_face_vm.at`（engine DLL 上 PATH，
  worktree auto.exe）打印 `VFACE_OK` exit 0；新增单测
  load_manifest_file_reads_engine_subset / dot_form_c_import_scans_as_
  c_import 两件绿。engine_abi.h 与 auto-term ffi.rs:48-237 十二签名
  逐符核对一致（write_input len=size_t 对齐正确）。缓冲出参 4 符号
  不可达口径在语料头注+manifest 单测断言（row_text 缺席）双落。
- **AC-04 ✅ 核位**：auto-term 主检出 004 §5③ 回执（CFACE_OK/LNK1107/
  裁定书/两枚 VM 缺陷全要点在案）+ DEBTS「③ 已落地清账(auto-lang
  PLAN-597,2026-09-09)」条目均在。
- **AC-05 ✅ 复现**（均 --no-fail-fast 全量）：cargo tv **3628/3629**、
  cargo tt **3842/3843**、cargo tf **3484/3485**——三档唯一失败均为
  基线固有 `ui_gen::vue::test_charts_gallery_compiles`（595/599 复审
  在案），名单与 work 记录一致，零新增红。
- **健康检查**：改动四 Rust 文件零新增警告（codegen.rs 命中的 3 处警告
  位置 7/2611/7813 均远离本提交 hunk ~5005 行，基线存量）；diff 无
  调试残留（仅 codegen 回退路径 log::warn!，符合既有模式）。
- **Findings（非阻塞）**：
  - F-1（记账）：两枚 VM 存量缺陷（D1 if 条件位内联 C-FFI+while 挂起 /
    D2 循环计数器嵌套 if 赋值静默断流）现仅在语料头注+计划 §5 在案，
    KNOWN-DEBT-AND-RISKS.md 尚无条目——merge 时登记债务候选。
  - F-2（观察）：`auto <script>` 尾行打印脚本尾值（冒烟尾部 "false" =
    run_file_with_args 尾值语义，runner 代码本提交未触及，基线行为，
    不构成失败标记）。
- **规范增量定稿**：supersedes=[auto-bindgen/project.md,
  auto-lang/vm/design/ffi.md, auto-lang/trans/overview.md]（SD-01/02/03
  三 modify）；new=[]；touched_goals=[GOAL-006（consumer-mode C 轨：
  三方引擎 DLL 只读消费实证）, GOAL-013（C 生态 a2c 互操作先例）]。
  plan_revision 维持 1（复审未改语义契约）。

stage: merge | PLAN-597:r1 | outcome: **pass** | delivery_commit=b1fa1b7a3 |
canonical_specs=docs/specs/auto-bindgen/project.md +
docs/specs/auto-lang/vm/design/ffi.md + docs/specs/auto-lang/trans/overview.md |
ledger_targets=.autoos/specs.json P597-1..6（运行时台账，gitignored）+
vm/trans plans.md 597 行 + docs/specs/INDEX.md | archive_path=docs/plans/archive/
597-cffi-engine-face.md | cleanup=见 checkpoints

### Merge checkpoints(2026-09-10)

- `prepared` ✅：reviewed 基线 b400a2821（rev1 pass 在案）；worktree 折入
  master（598 codegen 闭包区改动，与 597 C-FFI 区不同 hunk，合并干净
  1e1c950c6）→ delivery_commit 候选 b1fa1b7a3（纯 docs 后代：SD-01/02/03
  三规范落稿 + vm/trans plans.md 台账行，代码/依赖与 reviewed 提交零差异）。
- 折 master 后刷新复现全绿：单测 2/2、cargo tv 3633/3634、cargo tf
  3489/3490（唯一红=charts 基线预存）、a2c 套件 110 ok/基线 9 名单不变。
- `landed` ✅：`git merge plan-597-dev` 落 master（fast-forward 至
  b1fa1b7a3）；b400a2821 在 master 祖先链核位；master 树与
  plan-597-dev 树 diff 为空（=门禁已验字节）；spec 三文档+两 plans.md
  行内容在 master 核位。wt-guard clean（合并前置扫描）。
- `ledger_refreshed` ✅：.autoos/specs.json P597-1..6 六节发布（运行时
  台账，回填归档路径）；docs/specs/INDEX.md 经 scripts/spec-index.py
  再生成并提交。
- `archived` ✅：git mv → docs/plans/archive/597-cffi-engine-face.md +
  status: archived 终态；KNOWN-DEBT-AND-RISKS.md 增 P597 债务节
  （两枚 VM 存量缺陷 D1/D2 复审 F-1 登记）。
- `cleaned` （清场后回填）

## 10. 待澄清事项

- T-05 的 VM 加载面落/不落，授权执行时按成本边界自裁并记录（无需
  预先裁定）；
- 唯一外部前置：auto-term 侧 `cargo build -p autoterm-core` 可产出
  cdylib（现役绿色路径，风险低）。
