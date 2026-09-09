---
plan_id: PLAN-597
status: drafting                # drafting → executing → execution_done → reviewed → archived
feature_name: cffi-engine-face
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-09
plan_revision: 1
current_step: 0
total_steps: 7

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

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

### T-01 链接机制（bounded，先手写最小 C 样例）

验证 MSVC 对 cdylib C 导出的链接输入形态（直接 DLL vs import lib），
结论写回本节；构建脚本据此落 `scripts/build-engine-face-a2c.cmd`
（DLL 解析序 + vcvars 自举，纯 ASCII）。

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
| 缓冲出参 | ✅ C 母语 | ✅ | ❌（本计划证实） |
| 运行时依赖 | 零 | a2r-std 视用法 | VM 全量 |
| 适配场景 | 独立驱动程序/工具 | at-gen 宿主内胶水 | VM 应用 |

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

- [ ] **T-01**（bounded investigation）链接机制验证：最小手写 C 样例
  链 autoterm_core.dll（直接 DLL vs import lib），结论回填 §5；
- [ ] **T-02** Auto 驱动源 + 12 符号 fn.c 声明快照语料（.wrong 反哺法）；
- [ ] **T-03** 构建脚本 `scripts/build-engine-face-a2c.cmd`（DLL 解析
  序 + MSVC + 布局冒烟）；
- [ ] **T-04** 真机驱动验收（AC-01 运行取证）；
- [ ] **T-05** VM 标量子集：加载面/冒烟 或 gap 记录（AC-03 二选一）；
- [ ] **T-06** 路径裁定书 + 004 §5③ 回执 + DEBTS #10 增 597 条；
- [ ] **T-07** 收口门禁：a2c 套件 + bindgen 单测 + cargo tf/tt 档
  （基线两例不变）。

依赖：T-02←T-01；T-03←T-01/02；T-04←T-03；T-05 独立；T-06←T-04/05。

## 9. 复审记录

stage: new | PLAN-597 | rev 1 | outcome: pass | next: work
（起草即绪：路径/命令/签名均经仓库核位；执行授权待用户指令。）

## 10. 待澄清事项

- T-05 的 VM 加载面落/不落，授权执行时按成本边界自裁并记录（无需
  预先裁定）；
- 唯一外部前置：auto-term 侧 `cargo build -p autoterm-core` 可产出
  cdylib（现役绿色路径，风险低）。
