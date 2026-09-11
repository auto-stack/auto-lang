---
plan_id: PLAN-610
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: a2r-c-abi-two-faces
author: [ZCode]
created_at: 2026-09-10
updated_at: 2026-09-10
plan_revision: 1
current_step: 0
total_steps: 12

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/trans rust(a2r), auto-lang parser(attrs), auto-bindgen(消费面)]
---

# [PLAN-610] a2r-c-abi-two-faces：C ABI 双面——cdylib 导出发射（004 §5⑤）+ use.c 的 a2r 后端（004 §5⑥）

> **来源**：auto-term `docs/designs/004-engine-autoization-roadmap.md` §5⑤⑥
> 与 §3.5（一套语义三个 lowering，manifest=共享 IR）。前置全落地：
> 595（manifest 类型层 FnPtr/abi + kernel32 数据集）、597（引擎面链接/
> 驱动先例 + load_manifest_file）、599（外来形态四能力 + 实编门范式）。
> **用户裁定（2026-09-10）**：两题合并一个计划——共享验收场（引擎
> 12 符号面），且 ⑥ 的最佳语料恰是 ⑤ 的产物（"Auto 调 Auto 编译物"
> 闭环）。
> **边界**：与 Q2 轨 PLAN-596（430 dep 运行期管线/反向回调重入 VM）
> 码路无交；596 T-05 跳板选型（捕获闭包→令牌→宿主跳板+catch_unwind）
> 是本计划 T-10 trampoline spike 的养分，结论可引用不依赖。

## 0. 变更摘要

a2r 补齐 C ABI 的两个方向，终局以引擎面闭环：

1. **⑤ 导出面**：Auto 声明的函数可发射为 cdylib 导出（`#[unsafe
   (no_mangle)] pub extern "C"/"system" fn` + 安全内部体）；
2. **⑥ 调用面**：`use.c` 在 a2r 下双形态下降——S 静态（`#[link]`
   extern 块 + 安全包装，链接 import lib）与 D 动态（libloading 生成
   模块，路径解析序沿用引擎先例）；
3. **capstone 双闭环**：引擎 12 符号面的 **Auto 版替身**（⑤ 产物，
   dep autoterm-core）被 597 的 a2c 驱动器链接跑出 CFACE_OK；同一
   引擎驱动逻辑以 ⑥（a2r）书写、链接真 DLL 跑出 CFACE_OK；终局
   **⑥ 驱动改链 ⑤ 产物——Auto 源写的驱动经 a2r 调 Auto 源写的引擎
   面**，两端零手写 C/Rust 胶水；
4. **trampoline spike**（bounded，可降级）：Auto 闭包 → `extern "C" fn`
   回调的选型记录（吸收 596 T-05）。

## 1. 目标

- **G1（⑤ 发射）**：`#[export]`（或 T-01 裁定形态）函数经 a2r 产物为
  cdylib 导出面，类型保真（i32/u32/usize/ptr/cstr ABI 宽度正确）；
- **G2（⑥ 下降）**：`use.c <header>`（builtin/JSON manifest）在 a2r
  下生成 Rust FFI 消费面，S/D 双形态可用；
- **G3（闭环）**：AC-03/04/05 三级 CFACE_OK（真 DLL×a2c 驱动、真
  DLL×a2r 驱动、Auto-cdylib×a2r 驱动）；
- **G4（选型）**：trampoline 机制有成文选型（实作可另立）。

### 非目标

- 不动 Q2 轨（596 领地：shim-metadata/430 wrapper/VM 重入）；
- 不做 at-gen engine.rs 的实际替换与 auto-term 仓任何改动（只读消费
  autoterm-core 构建产物与 597 脚本先例）；
- trampoline 实作不在本计划硬性范围（spike 产出选型即达标，实作按
  成本自裁降级为 follow-up 并记录）；
- 不做 VM 轨 use.c 增强（597 已定型）。

## 2. 架构方案

**⑤ 导出面**：语法拟 `#[export]` 属性（fn 级 attrs 透传已有先例，
T-01 核位）；a2r 发射层在 fn_decl 处理 `#[export]` → 包装为
`#[unsafe(no_mangle)] pub extern "C" fn`（abi 按 T-01 裁定可注记
`"system"`），体内按 ABI 类型收窄/转换后调用 Auto 逻辑体。类型保真
策略二选一（T-01 spike）：A=导出包装层边界 cast（i64↔i32 等显式
as）；B=parser 增窄整型标注位。倾向 A（零语法面新增）。

**⑥ 调用面**：manifest=共享 IR（595/597 在库）——a2r 的 use.c 处理
走 `load_builtin_manifest`→`load_manifest_file` 同源数据，生成：
- S 形态：`#[link(name = "<library>")] mod ffi { extern "<abi>" {…} }`
  + 每函数安全包装（标量直过/cstr 转换），产物 Cargo 需链接 import
  lib（.dll.lib，rustcdylib/系统库均有，597 T-01 实证）；
- D 形态：libloading 模块（Lazy 静态 + Library::new 解析序：
  env 覆盖 → exe 同目录 → PATH，引擎先例）。

**构建 harness**：a2r 产物 → cdylib/exe 需要 cargo 小工程（`#[ignore]`
实编门内模板生成，先例=594 parity libs rust/ 目录 + 599 三道门）。

### T-01 spike 裁定（2026-09-11，结论落 §2）

- **语法形态＝`#[export]`**（fn 级，可选 abi 实参 `#[export(system)]`→
  `extern "system"`，默认 `"C"`）。与 GDScript `export*` var 注解零冲突：
  store_attrs 仅被 var 路径消费（parser.rs:5139），fn 前的 `#[export]`
  今日被静默忽略——复用该名，臂内增设 `ann.export_abi: Option<AutoStr>`，
  经 `pending_api_attrs` 同款 pending 流转（parser.rs:5061→9391），AST
  `Fn.export_abi` 新字段（None=非导出）。**不进 impl_attrs**（防 Rust
  产物出现非法 `#[export]` attr、防其他后端误发）。
- **类型保真＝策略 A**（包装层边界 cast，零语法面新增）。发射"原名安全
  体 + 兄弟包装模块"双件：`mod <name>_c_export { #[unsafe(no_mangle)]
  pub extern "<abi>" fn <name>(ABI 形参) { super::<name>(conv…) as … } }`
  ——符号名=fn 名（no_mangle 不含模块路径），**调用方零改动**（无需
  全程序改名映射）。
- **ABI 类型映射**：Int→i32（内体 i64，边界 `as` 双向）；i64/u64/usize/
  byte/bool/char/float/double 直过（uint→u32）；cstr→`*const c_char`
  （空指针容错→""）；返回 cstr→`CString::new(…).unwrap_or_default()
  .into_raw()`（所有权让渡 C，内嵌 NUL 降级空串不 panic——"无 panic
  穿越 FFI"契约）；`*T`→原样 + 新修 `*void`→`*mut std::ffi::c_void`
  （今日 `*mut void` 系坏输出，无语料依赖）；复合类型→发射期报错
  （MVP 边界）。拒绝面：方法/泛型/async/test 与 #[export] 组合报错。
- **⑥ 形态选择＝manifest `"link": "static"|"dynamic"`**（serde 默认
  static）——零 Auto 语法新增；S=生成模块内 `#[link(name=<library>)]
  extern "<abi>"` 块 + 安全包装 + `pub use <module>::*`；D=libloading
  OnceLock，解析序 env `<LIBRARY>_DLL` → exe 同目录 → 裸名(PATH)。
  **缓冲出参**（take_dirty_rows/row_text/row_style/cursor）由生成模块
  内置安全助手消化（buf/ptr 读写族）——004 §3.5"所有 unsafe 居于生成
  模块内"的落地形态；Auto 指针原语（`x.@`/`p.*`，02_types/004_pointer
  在案）+ `*void` 修复后 ⑤⑥ 语料全程安全面可书。
- **manifest 路径解析**：a2r 增可选 source_dir 入口
  （`transpile_rust_in_dir`；run_a2r_file_test 传 case.dir，CLI 传文件
  父目录），相对路径先 source_dir 再 CWD。

## 3. 技术栈

trans/rust.rs（⑤发射+⑥生成器）、parser attrs、auto-bindgen manifest
（只读消费）、rustc/cargo（实编门）、MSVC（链接，597 知识）、
auto-term 构建产物（只读）。

## 4. 需求分析与背景调查

- **授权**：用户 2026-09-10 指示"一起立项…合并成一个项目（同一计划
  文件）"——本计划为立项起草（drafting），**执行待 /auto-plan:work
  指令**；
- ⑤⑥ 领地盘点在案（2026-09-10）：596-609 无重叠（596=Q2 运行期，
  方向相反）；
- 引擎面事实：12 符号签名全量核位（597 计划 §5）；无回调——⑥ 的
  MVP 语料天然免 trampoline；597 的 engine-face-a2c 驱动器（链接
  import lib 跑 CFACE_OK）直接复用为 AC-03 验收器；
- `engine_abi.h`（597 语料内）是 ⑤ capstone 的签名契约源；
- a2r 快照门=tests/a2r_tests.at（目录发现）；实编门范式=599 三道
  #[ignore] 门 + 语料级 a2r_rustc_real_compile_gate 兼容性（外来
  名须 use.rs 导入，F-1 教训）。

## 5. 详细设计

### 语料蓝图（test/a2r/27_c_abi/，编号执行时按实际目录空位定）

- `001_export_basic`：`#[export] fn add(a int, b int) int` → 快照断言
  no_mangle/extern "C" 形态；
- `002_export_cstr`：cstr 参数/返回导出（边界转换体）；
- `003_use_c_static`：`use.c <engine_face.json>`（597 的 manifest 复
  用）→ S 形态生成快照；
- `004_use_c_dynamic`：D 形态生成快照；
- `005_engine_face_auto`（capstone ⑤）：dep autoterm-core + use.rs
  PtySession/TermSession → 12 符号 Auto 版（含 snapshot 状态与
  kind_color 标量编码语义对齐 ffi.rs）；
- 实编门语料（非快照）：MVP face / ⑥ 驱动源 / 闭环驱动源。

### T-01（bounded）⑤ 语法与类型保真 spike

核位 attrs 透传链（parser→a2r）与 cast 策略 vs 窄整型标注；产出=
§2 落定 + 必要语法裁定记录。

### T-10（bounded）trampoline 选型 spike

候选：A=零捕获闭包直接 `extern "C" fn` 化（595 ctrlc 先例）；B=
捕获闭包 Box::into_raw+静态注册表+壳（004 §3.5 原案）；C=596 宿主
跳板式。判据：产物零运行时依赖优先。产出=选型文档（本计划 §5 附
录），实作授权自裁降级。

### 规范增量

| delta_id | 操作 | docs/specs/... target | before/after rule | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/a2r-transpiler-guide.md | Implementation Status 增 610 条：⑤导出发射（语法/类型保真策略）+⑥use.c 双形态下降+trampoline 选型 | 004 §5⑤⑥ 清偿 | AC-01..07 |
| SD-02 | modify | docs/specs/auto-lang/trans/overview.md | a2r 能力面补：C ABI 双面条目 | 当前态记录 | AC-01..06 |
| SD-03 | modify | docs/specs/auto-bindgen/project.md | 消费面三后端补 a2r（§3.5 落地：VM/a2c/a2r 同 IR） | 共享 IR 收口 | AC-03/04 |

## 6. 测试设计

- 快照：001-005 五件（文本门，含 real-gate 兼容性——外来名 use.rs
  纪律）；
- 实编门（#[ignore]，599 范式）：
  `a2r_cabi_export_gate`（MVP cdylib + Rust 消费者 dlopen 断言）、
  `a2r_cabi_engine_face_gate`（⑤ cdylib 符号清单 dumpbin 对照 +
  597 驱动器链接跑 CFACE_OK）、`a2r_cabi_use_c_gate`（⑥ S 形态链真
  DLL 跑 CFACE_OK + D 形态同场景 + 闭环：改链 ⑤ 产物）；
- 护栏：a2r 套件基线三连不变；tt/tf/tv 唯基线（含 596 在途 020）；
  bindgen/a2c 套件不变。

## 7. 验收标准

- **AC-01**：`#[export]`（或 T-01 裁定）发射形态入快照（no_mangle +
  extern "C"/"system"，类型 ABI 保真）；
- **AC-02**：MVP 导出面编译为 cdylib，Rust 消费者 dlopen 调用断言
  通过（实编门绿）；
- **AC-03**：引擎 12 符号 Auto 版 cdylib——导出符号清单与 Rust 版
  一致（对照工具在案）且 **597 a2c 驱动器链接它跑出 CFACE_OK/exit 0**；
- **AC-04**：⑥ S 形态——Auto 驱动源（use.c 引擎面）经 a2r 链真
  autoterm_core.dll.lib 实跑 **CFACE_OK**；
- **AC-05**：闭环——同一 ⑥ 驱动源改链 ⑤ 产物（Auto 写的引擎面
  cdylib）同样 **CFACE_OK**（Auto↔Auto 全链零手写胶水）；
- **AC-06**：⑥ D 形态运行期加载同场景绿（或 bounded 降级记录有据）；
- **AC-07**：trampoline 选型文档在案（实作降级须显式记录）；
- **AC-08**：套件零回归 + SD-01/02/03 落稿 + 004 §5⑤⑥ 回执。

## 8. 执行步骤

- [x] **T-01**（bounded）⑤ 语法与类型保真 spike（attrs 链/cast 策略），
  结论回填 §2；[✅ 已完成：`#[export]`（复用 GDScript export 名，fn 路径
  零冲突实证 parser.rs:5139 store_attrs 仅 var 消费）；策略 A（兄弟模块
  wrapper，零调用点改动）；⑥ 形态=manifest link 字段；缓冲出参=生成模块
  内置安全助手；manifest 路径=source_dir 入口。裁定全文 §2 T-01 节]
- [x] **T-02** ⑤ 发射实现 + 快照 001/002；[✅ 已完成：parser `#[export]`
  /`#[export(system)]`（FnAnnotations.export_abi + pending 流转 + Fn.export_abi
  AST 字段）+ rust.rs 兄弟包装模块发射（unsafe 收窄于 CStr::from_ptr）+
  `*void`→`*mut std::ffi::c_void` 修复；语料 27_c_abi/001_export_basic、
  /002_export_cstr 快照绿（逐例 2/2 + a2r_tests 模块 369/369 +
  a2r_rustc_real_compile_gate 绿）；执行期两修：cstr 形参系 &str（借
  用需 `_conv` 局部持有者）、param+param 字符串拼接系 a2r 既有空白
  （语料改 f-string）；worktree commit 27c63c6a1]
- [x] **T-03** cdylib/exe 构建 harness（实编门内 cargo 模板）；
  [✅ a2r_c_abi_gates.rs 模板 harness：cargo cdylib/exe 工程 staging 于
  target/plan610（gitignore 暖缓存）+ 标准库零依赖消费端
  （LoadLibraryA/GetProcAddress，注册表/网络免依赖）；执行期一修：
  Sink.done() 返回产物字节（非 body 残留）——gate 首版产物空文件实证]
- [x] **T-04** MVP 导出面实编门（AC-02）；[✅ a2r_cabi_export_gate 绿：
  #[export] cdylib 真编 + dlopen 调用断言（add(2,3)==5 宽度桥 +
  dup("ab")=="abab" cstr 边界）；worktree commit 5713421b7]
- [x] **T-05** capstone ⑤：引擎 12 符号 Auto 版（语料 005 + dep
  autoterm-core 只读消费）；[✅ 已完成：005_engine_face_auto 语料快照绿
  +全量 rustc 门过；执行期五修——TermColor 系 ffi.rs 私有别名（公开路径
  autoterm_core::term::Color）、外来非 Clone 字段压制 blanket derive
  （显式 #[allow(dead_code)] 替换默认 derive 集）、spawn 空参须类型化
  List<Str> 局部（AsRef 推断歧义）、free 走 kit cabi_drop_boxed
  （from_raw unsafe 居 kit）、wrapper 模块补 use super::*；**AC-03 手动
  实证**：build-engine-face-a2c.cmd 以 AUTOTERM_ENGINE_DLL 改链
  engine_face_auto.dll（⑤ 产物）→ 驱动器 CFACE_OK/exit 0 两连；
  worktree commit 3991584f1]
- [ ] **T-06** ⑥ S 形态生成器 + 快照 003；
- [ ] **T-07** ⑥ D 形态生成器 + 快照 004（解析序 env→同目录→PATH）；
- [ ] **T-08** ⑥ 实编门：S 链真 DLL CFACE_OK（AC-04）；D 同场景
  （AC-06）；
- [ ] **T-09** 闭环门：AC-03（597 驱动器×⑤ 产物）+ AC-05（⑥ 驱动×
  ⑤ 产物）；
- [ ] **T-10**（bounded）trampoline 选型 spike（AC-07；可并行/可后置）；
- [ ] **T-11** SD 落稿 + 004 §5⑤⑥ 回执 + DEBTS #10 增 610 条；
- [ ] **T-12** 收口门禁：a2r 套件 + 三道实编门 + tf/tt/tv +
  bindgen/a2c（基线不变）。

依赖：T-02←T-01；T-04←T-02/03；T-05←T-02/03；T-06/07 独立；T-08←
T-06(07)；T-09←T-05+T-08；T-10 独立；T-12 收口。

## 9. 复审记录

stage: new | PLAN-610 | rev 1 | outcome: pass | next: work
（起草即绪：前置/领地/验收 oracle 均在案；执行授权待用户指令。）

## 10. 待澄清事项

- T-01 语法形态（#[export] vs 备选）授权执行时按 §5 判据自裁并记录；
- T-10 trampoline 实作降级（只交选型）授权自裁，记录于 §5 附录；
- 外部前置：auto-term `cargo build -p autoterm-core` 可产出 rlib/
  cdylib（现役绿色路径）+ 597 驱动器脚本在 master（已合）。
