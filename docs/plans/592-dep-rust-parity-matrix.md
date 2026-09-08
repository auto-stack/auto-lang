---
plan_id: PLAN-592
status: execution_done         # drafting → executing → execution_done → reviewed → archived
feature_name: dep-rust-parity-matrix
author: [ZCode]
created_at: 2026-09-08
updated_at: 2026-09-09

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # engine.rs GET_FIELD / ffi.rs / 测试基建
current_step: 12
total_steps: 12
---

# [PLAN-592] dep-rust-parity-matrix：AutoVM 调三方 Rust 库的 P0 特征化 parity 网

> **执行完成（2026-09-09）**：12/12 步全绿；执行期额外抓出并修复 8 个管线真 bug
> （见执行步骤"执行期追加修复"）。三轨对拍（VM/a2r/oracle）在 016/017 全语料
> 上 stdout 精确相等。worktree 分支 `plan-592-dev`（690ed3422 + 5c28b2ce2），
> 待 `/auto-plan:review`。

## 变更摘要

为 430 dep 管线（`dep crate(path)` + `use.rs` → cdylib shim 包 → `DepOpaqueObject`）建立
**P0 层特征化测试网**，不依赖 PLAN-591（能力计划），先行钉住现有系统行为：

1. **两枚 fixture**：扩展 `autolang_counter`（drop 计数 + skip 面负面方法）+ 新建
   `autolang_abi_matrix`（marshalling 全矩阵：宽整型收窄/bool 掩码/i64 边界/字符串/
   arity 0-3/自由函数签名类）；
2. **两个 ffi_dual 用例**：`016_dep_abi_matrix`、`017_dep_lifecycle`（VM 轨 golden，
   沿 013 模式：nightly 缺席 skip、指纹缓存二次秒级）；
3. **三轨行为对拍 runner**：`ffi_dep_parity_tests.rs`——同一 `.at` 用例的
   VM 腿 / a2r 转译编译腿 / Rust oracle 腿 stdout 精确相等（a2r 腿 env 门控，
   oracle/VM 腿进日常档，cargo build 产物内容 hash 缓存）；
4. **两处静默 0 收口**（生产代码改动，仅此两处）：
   - GET_FIELD 对 dep 对象未命中 → 从静默压 0 改为**路由合成 getter 或报错**
     （`p.x` 字段语法桥接，canonical 字段访问面统一为 `p.x`）；
   - 自由函数 wrapper 未覆盖签名 → 从 warn+静默返 0 改为 **VMError**；
5. **已知分歧登记**：Option 返回 / 按值 self / >3 ABI 参数三条进
   `parity/docs/known-divergences.md`（VM 期望错误 golden），591 T2/后续翻绿。

## 目标

1. 430 管线 marshalling 面（i/l/f/b/s 全码 + 宽槽收窄 + 边界值 + BorrowStr/TakeStr +
   arity 上限）有确定性 golden 网，任何回归当场暴露。
2. 生命周期语义（chain 同句柄、clone_reset 独立性、skip 面不静默）有断言。
3. **VM ≡ a2r ≡ 手写 Rust** 三轨同库（同 path fixture）行为对拍跑通，为 591 V1/V2
   的验收提供现成 runner。
4. 消灭两处静默错误面：字段访问/未覆盖签名从"拿回 0"变为"拿到值或拿到错误"。
5. 591 的计划编号撞号问题（014/015 已被占用）留痕回改。

## 架构方案

```
                    ┌─ VM 腿: run_with_capture(dep+use.rs → methods pack → dispatch 3000)
input.at (语料) ────┼─ a2r 腿: transpile → 生成 crate(路径依赖由 dep_scanner 渲染)
 {{FIXTURE_DIR}}    │          → cargo build --release(内容hash缓存) → 跑产物   [env 门控]
 同一 fixture crate └─ oracle 腿: oracle/{Cargo.toml(空ws),main.rs} + ../fixture 路径依赖
                               → cargo build --release(内容hash缓存) → 跑产物
 判定: 三腿 stdout trim 后互等 且 == expected_output.txt
```

- **canonical 字段访问面 = `p.x` 属性语法**（a2r 天然发射；VM 侧经本计划桥接到合成
  getter——`METHODS` 表里 `"Point.x"` 即合成 getter 的方法 key）。getter 面 `p.x()`
  保留兼容但不进 parity 语料。
- **两收口点均为最小切口**：engine.rs GET_FIELD 末段路由 + ffi.rs 未覆盖签名分支
  改 Err；不触 shim-metadata/classify（那是 591 的地界）。
- **门禁归属**（AGENTS.md Category B）：改动触及 vm/engine.rs → `cargo tv` 兜底；
  日常档 `cargo t` 只含 VM 腿 + oracle 腿（缓存命中秒级）；a2r 腿挂
  `AUTO_LANG_DEP_PARITY_A2R=1`（本地手动/CI 全量档），避免日常门禁被 cargo build 拖慢。
- 技术栈：Rust（auto-lang 测试基建 + engine/ffi 两处收口）、libloading/manifest 机制
  照旧、无新依赖。

## 需求分析与背景调查

（取材：docs/specs/overview.md——auto-lang/vm 模块、shim-metadata/430 管线、
GOAL-006 消费者 parity 背景；代码实勘 2026-09-08）

**现有机制（430 档，VM 调三方库的唯一动态加载路径）**：

- 装载链：`dep` 声明（`compile.rs resolve_deps` L676 起，文本级
  `dep_scanner.rs scan_dep_statements` L89-109）→ auto-cache `methods_pack.rs`
  （nightly rustdoc → shim-metadata 生成 cdylib + manifest，C3 指纹缓存）→
  `lib.rs init_rust_ffi` L520-667 → `vm/ffi/dep_methods.rs register_pack`。
- marshaller：参数码 `s/i/l/f/b/p`、返回码 `v/i/l/f/b/s/p`；整型统一 i64 宽槽、
  wrapper 内显式 `as u8` 收窄（emit_cdylib.rs L521-547）；返回 bool 仅 al 有效
  （VM 侧 `& 0xFF` 掩码，dep_methods.rs L363-369）；字符串 `*const c_char` +
  `auto__free_cstring` 手动释放；**v1 ABI 参数上限 3**（L586-591 RuntimeError）。
- 结构体返回 = `p` 码 `Box::into_raw` → `DepOpaqueObject`（ptr + drop 符号 +
  lib 保活）；字段访问唯一现役面 = **合成 getter 方法**（rustdoc.rs L179-236，仅
  pub + 标量/Str/Opaque 白名单字段；METHODS 表 key `"短类型名.字段名"`）。
- classify skip 面（classify.rs）：Option 返回（三方路径 L70-79）、按值 self
  （L102-104）、按值外来参数（L125）、借用返回 `&T`（L156，`&Self` ChainInPlace
  例外）、泛型无 mono、i128/u128、闭包参数。
- 自由函数走 plan-212 wrapper cdylib：`ffi.rs create_rust_shim_lazy` L715-819，
  手写 match **仅 4 类签名**（()→基元、(String)→String/Long、(Long,Long)→Long），
  未覆盖签名 **warn + 静默 push 0**（L802-812）。

**两处静默 0（本计划收口对象）**：

1. GET_FIELD（engine.rs L5530-5838）：RustStdlibObject 臂有特例（Output.stdout、
   native_catalog 字段表 regex/url/semver/Instant/OnceCell/File），未命中
   **push_i32(0) 不报错**（L5800-5802）；`DepOpaqueObject` 无专属臂，落到末臂同样
   静默 0——拼错字段名拿回 0。
2. ffi.rs 未覆盖自由函数签名 warn+0（同上）。

**现有测试空白**：

- `ffi_dual_013_dep_method`（ffi_dual_tests.rs L97-158，VM 单轨）是唯一 dep e2e；
  f32/f64、宽整型收窄、u64 边界、arity、move、Option skip 后行为、字段语法均零覆盖。
- `parity/` 工具（VM/a2r/Rust oracle 三方 TAP 对拍，runner.rs）**无任何 dep/use.rs
  语料**；8 个 rust 库全是手写复刻。
- `test/vm/20_rust_ffi/001_serde_json` `#[ignore]` 且 golden 是期望报错。
- a2r 轨 dep 只有转译文本 golden（`16_interop/021_path_dep`），从不编译运行。

**与 PLAN-591 的关系**：591 是能力计划（V1 布局 manifest / V2 trait+泛型+回调），
status: drafting、未启动。本计划是其**验证前置层**：P0 网先钉现状，591 各层落地后
在同一 corpus 上翻绿（591 计划的 014/015 用例编号与现存 014/015 撞号，需回改为
018/019——见执行步骤 T10）。本计划不改 shim-metadata/classify（591 地界）。

## 详细设计

### D1 fixture：autolang_counter 扩展（013 目录内）

`crates/auto-lang/test/ffi_dual/013_dep_method/fixture/autolang_counter/src/lib.rs`：

- 加 `static DROPS: AtomicUsize` + 各类型 `impl Drop { DROPS.fetch_add(1) }` +
  自由函数 `pub fn drop_count() -> u64`（顺带成为自由函数路径 ()→u64 冒烟）。
  现有 013 golden 不受影响（不加 Drop 副作用到断言路径的输出）。
- `maybe()`（Option 返回）与 `bump()`（按值 self）**保留不动**——它们是 skip 面
  负面用例（017 断言调用报错而非静默）。
- 限制说明：RC/GC 时机使"作用域结束即 drop"不可中途断言；move 消耗型 drop 计数
  被 classify skip 挡住（by-value self 不进包）——move/drop 真覆盖推迟到 591/后续，
  登记待澄清 #2。

### D2 fixture：autolang_abi_matrix（新，016 目录内）

`crates/auto-lang/test/ffi_dual/016_dep_abi_matrix/fixture/autolang_abi_matrix/`
（独立 `[workspace]`，同 013 fixture 布局）。`src/lib.rs` 内容：

- `Num` struct（挂方法面）：`echo_i8/i16/i32/i64/u8/u16/u32/u64/usize/f32/f64/bool`
  （各 `&self, x: T) -> T` 成对），`add3(&self, i64, i64, i64) -> i64`（arity 3 上限内），
  `quad(&self, i64,i64,i64,i64) -> i64`（arity 4，预期 RuntimeError 负面）。
- `Words` struct：`greet(&self, name: &str) -> String`（BorrowStr）、
  `shout(&self, s: String) -> String`（TakeStr）、`unicode(&self) -> String`
  （emoji/CJK/空串往返）。
- `Pt` struct：**pub 乱序字段** `a: u8, b: String, c: i64`（字段语法桥接用，
  合成 getter `Pt.a/Pt.b/Pt.c`）。
- 边界自由函数/常量方法：`i64_min/i64_max()`（i64 槽边界）、`u64_max() -> u64`
  （宽槽行为钉死）。
- 自由函数签名类：`free_noop() -> i64`（()→基元，已覆盖类）、
  `free_s(s: String) -> String`（(String)→String，已覆盖类）、
  `free_ll(a: i64, b: i64) -> i64`（(Long,Long)→Long，已覆盖类）、
  `free_uncovered(a: f64, s: String) -> i64`（未覆盖类，T8 后断言报错）。

### D3 语料与 VM 腿（ffi_dual 扩展）

- 目录约定：`test/ffi_dual/016_dep_abi_matrix/{input.at, expected_output.txt,
  fixture/autolang_abi_matrix, oracle/}`；`017_dep_lifecycle/{input.at,
  expected_output.txt, oracle/}`（fixture 复用 013 扩展件）。
- `input.at` 内 dep 路径写占位符 `{{FIXTURE_DIR}}`，测试运行时替换为绝对路径
  （保持语料文件化，三腿共享；通用骨架 `test_ffi_dual` 增加占位替换逻辑）。
- VM 腿测试照 013 模式：`auto_cache::methods_pack::nightly_available()` 为假 →
  skip；`run_with_capture` → stdout trim 对比 `expected_output.txt`。
- golden 生成惯例：先跑一次把 stdout 写入期望文件（框架落 `.wrong.out` 供核对）。

### D4 三轨 runner：`crates/auto-lang/src/tests/ffi_dep_parity_tests.rs`

经 `src/tests.rs` 注册。语料发现：扫 `test/ffi_dual/{016,017}`（白名单起步，后续
语料递增）。三腿：

1. **VM 腿**：同 D3（占位替换 + run_with_capture；nightly 门同 013）。
2. **oracle 腿**：`oracle/` 目录（提交 `Cargo.toml`：空 `[workspace]` + 对
   `../fixture/<crate>` 的相对路径依赖；`src/main.rs` 手写 Rust 逐用例打印与
   input.at **同格式** 行）。`cargo build --release` 产物按内容 hash 缓存到
   `~/.auto/cache/dep_parity/<fnv1a64>/`（输入=fixture+oracle 源字节），二次秒级。
3. **a2r 腿**（`AUTO_LANG_DEP_PARITY_A2R=1` 才启用）：库内调用 trans API 转译
   input.at（参考 `a2r_tests.rs` 的 `transpile_rust` 用法；转译前如单文件管线对
   `Stmt::Dep` 报错/告警，则预处理剥离 dep 行——依赖由本腿自行渲染）；生成 crate
   的 Cargo.toml = 固定依赖集（a2r-std/auto-lang path、once_cell 等，参考
   `parity/crates/auto-parity/src/runner.rs run_a2r` L251/L492-539 的 wrap 模式）
   + `dep_scanner::scan_dep_statements(input.at)` 解析出的 path 依赖行；build
   产物同内容 hash 缓存；跑产物捕获 stdout。
- 断言：三腿（启用集）stdout trim 互等且 == golden；任一腿构建失败 = 测试失败并
  保留现场目录（`.wrong.out` 同款惯例）。
- 日常档默认：VM 腿 + oracle 腿；a2r 腿关闭时打印一行 `[dep-parity] a2r leg
  skipped (set AUTO_LANG_DEP_PARITY_A2R=1)`。

### D5 收口 A：GET_FIELD 桥接（engine.rs）

GET_FIELD 臂（L5530-5838）对堆对象 `type_tag()` 为 `RustStdlib(full)` 且
`full.contains("::")`（即 dep 对象）时的次序：

1. 既有特例保持：`.length`、Output.stdout/stderr、native_catalog 字段表命中者；
2. **新增**：重推接收者 + 注入 `method=字段名/type_name`（复用 dispatch 3000 的
   栈形约定，参考 engine.rs L7559-7603 CALL_SPEC 的 dep 路由）→ 命中
   `dep_methods::dispatch("短类型名.字段名")` 即合成 getter 取值（`p.a` ==
   `p.a()` 语义）；
3. 未命中 → `Err(VMError)`："unknown field '<f>' on dep object <full_type>（该字段
   无合成 getter：非 pub / 类型不在 标量·Str·Opaque 白名单）"——**替代静默
   push_i32(0)**。
4. Auto 本地 struct/Node/ObjectData 路径与本改动无交集（分臂在前）；全部 VM golden
   由 `cargo tv` 兜底回归。

### D6 收口 B：自由函数未覆盖签名（ffi.rs）

`create_rust_shim_lazy`（L715-819）的未命中签名分支：warn+push 0 →
`return Err(VMError)`，消息含符号名 + sig_code + "unsupported free-function
signature (plan-212 wrapper v1)"。已覆盖 4 类与 known_signature 回退路径不变。

### D7 登记与编号回改

- `parity/docs/known-divergences.md` 新增：DIV-DEP-1（Option 返回不进包→VM 报
  Unknown、a2r 正常；591 T2 解锁）、DIV-DEP-2（按值 self 同上；move marshaller
  dep_methods.rs L596-607 因此无 fixture 覆盖）、DIV-DEP-3（>3 ABI 参数 VM
  RuntimeError v1 上限、a2r 正常；430 后续放宽）。
- `docs/plans/591-use-rust-any-crate-direct.md` 待澄清事项追加：计划内 014/015
  用例编号与现存 014_std_generated_segment / 015_musk_backend_wave1 撞号，落地时
  改用 018/019（016/017 已被本计划占用）。

## 测试设计

| 用例 | 腿 | 关键断言 |
|---|---|---|
| 016 echo 全类型（Num） | VM+oracle+a2r | `echo_u8(1000)`→232（宽槽收窄语义=oracle `as u8`）、f32 经 f64 槽往返、bool true/false、i64::MIN/MAX、u64::MAX 行为钉死（golden 定型） |
| 016 arity | 三腿 | add3 正常；quad → 错误消息含 "v1 supports ≤3"（负面 golden） |
| 016 字符串 | 三腿 | &str/String 参数、emoji/CJK/空串往返一致 |
| 016 字段语法 | 三腿 | `p.a`/`p.c` == oracle 直读字段值（依赖 D5）；`p.nope` 报错（非 0） |
| 016 自由函数 | 三腿 | 3 类已覆盖签名正常；free_uncovered 报错含 "unsupported free-function signature"（依赖 D6） |
| 017 chain | 三腿 | `cfg.verbose(true)` 返回句柄后，原句柄与别名读值一致（同对象） |
| 017 clone 独立性 | 三腿 | clone_reset 新句柄 mutate 不影响原对象 |
| 017 skip 面负面 | VM | `maybe()`/`bump()` → "Unknown Rust stdlib call"（**不静默返 0**） |
| 017 drop 计数基线 | 三腿 | `drop_count()`==0 起始（自由函数 ()→u64 冒烟）；teardown 计数断言不可行→待澄清 #2 |

门禁映射：改动触及 `vm/engine.rs` → `cargo tv`；`ffi.rs` → `cargo t ffi`；日常
`cargo t` 含新 ffi_dual 用例与 dep_parity（VM+oracle 腿）；fold 前 `cargo tf`
（review 阶段执行）。不触 aavm 面（`auto/lib`、aavm2 语料）→ 不跑 `taa`。

## 验收标准

1. `cargo t ffi_dual` 全绿（016/017 在列；无 nightly 环境自动 skip 而非失败）。
2. `AUTO_LANG_DEP_PARITY_A2R=1 cargo t ffi_dep_parity` 全绿：016/017 每用例
   VM/a2r/oracle stdout 与 golden 精确相等。
3. 静默 0 清零：dep 对象未知字段访问报错；`p.a` 命中合成 getter 取到与 oracle
   一致的值；未覆盖自由函数签名报错。
4. `cargo tv` 零回归（GET_FIELD 改动的全量 golden 兜底）；`cargo check -p auto-lang`
   零新警告。
5. known-divergences 新增 3 条、591 编号回改留痕。
6. CI（vm-files-ci.yml）纳入 016/017 与 a2r 腿全量档。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据。
执行于 worktree `D:/autostack/.wt/lang-592/auto-lang`——开工前先在 master 提交
.next-id 与本骨架，创建 worktree 时遵守 wt-guard 红线：worktree 内禁 junction/symlink。
**执行纪实（2026-09-09）**：除下列 12 步外，特征化网抓出 8 个管线真 bug 并就地修复
（详见各步证据与复审记录）；另因 nextest `--all-features` 需解析 auto-lang 的跨仓
path 依赖 `autodown-core`，在分组目录补建了 auto-down 的 detached 兄弟 worktree
（`D:/autostack/.wt/lang-592/auto-down`，merge 时随组清理）。）

- [x] **T1** 扩展 fixture autolang_counter：
  `crates/auto-lang/test/ffi_dual/013_dep_method/fixture/autolang_counter/src/lib.rs`
  加 DROPS 原子计数 + 各类型 `impl Drop` + `pub fn drop_count() -> i64`。
  [✅ 已完成] cargo check 通过；`cargo t ffi_dual_013` PASS（shim 包带新 fixture 重建）。
- [x] **T2** 新建 fixture autolang_abi_matrix（D2 全集，独立 workspace）。
  [✅ 已完成] cargo check 通过；27 方法 + 4 自由函数进 manifest（v1.2 生成器）。
- [x] **T3** 语料 016：input.at（`{{FFI_DUAL_DIR}}` 占位，fn main 包裹）+ oracle/ + 首跑 golden。
  [✅ 已完成] oracle `cargo run --release` 输出 = golden（27 行含空串空行）。
  偏差注记：占位符实现为 `{{FFI_DUAL_DIR}}`（= test/ffi_dual 绝对根，017 复用 013
  fixture 也能表达）；语料改 fn main 包裹形态（双轨安全）。
- [x] **T4** 注册 VM 腿 `ffi_dual_016_dep_abi_matrix`（含 VM 特征化钉死段与负面断言）。
  [✅ 已完成] `cargo t ffi_dual_016` PASS——含宽槽收窄 232/u64_max -1 钉死、quad
  "≤3" 错误、free_uncovered 报错、unknown field 报错、p.a/p.b/p.c 字段值正确。
- [x] **T5** 语料+注册 017（复用 013 扩展 fixture）。
  [✅ 已完成] `cargo t ffi_dual_017` PASS（chain 可见性/clone 独立性/drop_count 基线/
  maybe/bump 负面）。
- [x] **T6** 三轨 runner：`ffi_dep_parity_tests.rs` + `src/tests.rs` 注册 + test/ffi_dual/.gitignore。
  [✅ 已完成] VM+oracle 双腿日常档 PASS（缓存命中秒级）；
  `AUTO_LANG_DEP_PARITY_A2R=1 cargo t dep_parity` 三腿全量 2/2 PASS（a2r 腿冷构建
  ~2.5min，内容 hash marker 缓存后秒级）。偏差注记：负面断言按 013 err-assert 形态
  落在测试体内（错误输出无法进 stdout golden）。
- [x] **T7** 收口 A（D5）：engine.rs GET_FIELD 的 DepOpaqueObject 臂。
  [✅ 已完成] `p.a/p.b/p.c` 命中合成 getter 取到与 oracle 一致值；`p.nope` 报
  "unknown field"；`cargo tv` 2780/2781（唯一余红 charts_gallery 为 master 同款预存）。
- [x] **T8** 收口 B（D6）：ffi.rs 未覆盖签名 → VMError。
  [✅ 已完成] free_uncovered 断言错误消息含 "unsupported free-function signature"；
  `cargo t ffi_tests` 35/35 零回归。
- [x] **T9** negative golden 定稿。
  [✅ 已完成] 全部负面断言绿（quad ≤3 / maybe / bump / free_uncovered / unknown field），
  以测试体内 expect_err+消息断言形态定稿（见 T6 偏差注记）。
- [x] **T10** 登记：known-divergences.md 增 DIV-DEP-1..7；591 待澄清事项追加编号
  回改注记（014/015→018/019 + PLAN-592 可复用资产清单）。
  [✅ 已完成] 两文件 diff 人工核对（DIV-DEP-1 Option/2 by-value self/3 arity/4 u64槽/
  5 print参数劫持/6 wrapper u64转换/7 a2r String形参假定&str）。
- [x] **T11** CI：vm-files-ci.yml 增 dep 三轨步（AUTO_LANG_DEP_PARITY_A2R=1）。
  [✅ 已完成] python yaml.safe_load 校验通过；ffi_dual 既有步骤的名称过滤器天然
  覆盖 016/017。
- [x] **T12** 收口健康检查。
  [✅ 已完成] `cargo check -p auto-lang` 警告数 173=master 173（零新增）；
  `cargo t ffi_dual` 20/20；`cargo t ffi_tests` 35/35；`cargo tv` 唯一余红=charts_gallery
  预存（master 复现确认）；`cargo tt` 唯一余红=同一预存。fold 前 `cargo tf` 由复审
  阶段执行。

### 执行期追加修复（特征化网战果，均在 worktree 提交 690ed3422/5c28b2ce2）

1. **emit_cdylib i8/i16 参数漏发收窄 cast**（wrapper 编译失败→VM 侧静默降级
   "Unknown"）——补 `Ty::I8/I16` 两臂，GENERATOR 升 v1.2（指纹失效重建）。
2. **dep_methods pop_int 不解堆编码**（>2^48 i64 字面量参数弹栈失败）——
   接 convert::decode_i64_full 堆感知兜底。
3. **'i' 槽返回零扩展**（echo_i8(-128) 呈现 4294967168）——ret_call 补符号扩展。
4. **自由函数 wrapper 装载链断裂**（compile 键 v3_{sig串长} vs init 键
   v3_{fn数} 从未对齐 + init 列表含类型名/重复项）——两侧统一 v3_{自由函数数}，
   init 过滤+去重；根因即 20_rust_ffi_001 expected.error。
5. **auto-cache wrapper 生成器三缺陷**——&str/String 参数折叠（新增
   ShimType::CStringOwned，线格式仍 's'）、path 依赖不走 syn 扫描
   （scan_path_dep_signatures）、ffi header 条件发射（全数值签名 wrapper 编不过，
   改无条件）。
6. **codegen 自由函数路由**——裸名查不到 `rust.{fn}` 限定注册，补限定名回退
   （否则 dispatch 3000 "Unknown"）。
7. **trans/rust.rs rust 型绑定缺 `let mut`**（a2r 对 dep 对象方法调用 E0596）——
   scan_mutated_bindings 保守标记 + golden 校准豁免表（std/chrono/io/regex/url/
   semver 家族）。
8. **a2r String 形参假定 &str**（`free_s(boxed)` 发射 `.as_str()` E0308）——
   登记 DIV-DEP-7，语料以方法调用结果传参规避（根治待 extern-sigs/元数据接入）。

## 复审记录

## 待澄清事项

1. **a2r 腿发射支持面**：`Type.new()` 静态调用、`p.a` 字段语法、`use.rs` 导入的
   发射是否在单文件转译管线全部可用（playground examples 声称 chrono 双轨可跑，
   预计可行）。若个别形态缺失：T6 内修 a2r 发射，或收窄 016/017 语料面并把该形态
   登记为 known-divergence——以"三腿同源"优先。
2. **drop/teardown 计数断言不可中途观测**（RC/GC 时机非确定；move 消耗型 drop 被
   classify skip 挡住）——017 仅断言基线 0；move marshaller（dep_methods.rs
   L596-607）的真覆盖推迟到 591 V2 / classify 例外表接线，与 DIV-DEP-2 一并登记。
3. **u64::MAX 经 i64 槽的呈现**（-1 或大整数装箱）以 golden 定型为准，三腿一致即
   pass；若 a2r 与 VM 呈现不一致，属真 parity bug，现场修复而非放宽断言。
4. oracle/a2r 腿 cargo build 在 CI 的首次冷构建耗时（预计 1-2min/腿）是否可接受，
   或需预热缓存步骤——T11 时实测决定。
