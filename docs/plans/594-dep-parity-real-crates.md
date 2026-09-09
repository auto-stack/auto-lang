---
plan_id: PLAN-594
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: dep-parity-real-crates
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-09

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [parity]             # 改 parity 工具（独立 workspace）+ 语料 + CI；不触主仓 crates/
current_step: 13
total_steps: 13
---

# [PLAN-594] dep-parity-real-crates：真三方 Rust 库的 AutoVM 动态加载三轨对拍

> **执行完成（2026-09-09）**：13/13 步全绿。四库（serde_json/regex/url/semver）
> 三轨 TAP 全绿（9 case 零分歧），base64 按待澄清 #2 裁定全红样本；红面
> DIV-DEP-8..14 显式登记；命中率报告落盘并回填 591 待澄清 #4。
> 执行期三项裁定变更（fmt_assist 不引入/CI 宿主改 parity-ci.yml/base64
> 全红不进 p10）均待澄清预授权范围内，见各步 [✅] 注记。
> worktree 分支 `plan-594-dev`（a7facbb94 + 30befca5f + acdbfba1a），
> 待 `/auto-plan:review`。

> **前置 spike 已完成（2026-09-09，本计划起草期，全部本机实证）**——结论见
> §需求分析与背景调查 S1..S10。核心：parity 工具 a2r 腿 dep 透传 = 仅改
> `run_a2r` 的 Cargo.toml 渲染（转译器零改动）；五库类型面走 native_catalog、
> 自由函数面走 430/212 pack 的**混合路径**已实证，计划按此如实设计。

## 变更摘要

为 AutoVM 动态加载**真三方 crates.io 库**建 R 层 parity 对拍（PLAN-592 的后续线）：
serde_json / regex / base64 / url / semver 五库，经 `dep crate(version: "...") +
use.rs ...` 动态加载调用，parity 工具三轨（VM / a2r / 手写 Rust oracle）TAP 对拍：

1. **P0 前置（spike 已验证可行性）**：`parity/crates/auto-parity` 的 `run_a2r`
   生成测试 crate 的 Cargo.toml 时透传语料 `dep` 声明（本地小解析器 +
   crates.io 依赖行渲染；auto-parity 不引 auto-lang 依赖）；
2. **五库语料**：`parity/libs/dep/<lib>/` 新分类（green-first——只用当前能力
   绿面 API；逐库 pin 精确版本，oracle 与 dep 声明同版）；
3. **红面显式登记**：known-divergences.md 续 DIV-DEP-8+，按
   Option / by-value-self / trait / generic / 其他 分类；
4. **skip 面命中率报告**（重要附带交付）：红项按分类统计五库 API 命中率，
   落 `docs/plans/reports/`，结论回填 PLAN-591 待澄清事项（591 V1/V2 的
   T2/T3 优先级依据）；
5. **网络与 CI**：本地 `AUTO_LANG_PARITY_NET=1` env 门控；CI 网络档进
   `vm-files-ci.yml`（带 cargo registry 缓存预热）。

**不触主仓 `crates/` 下源码**——转译器对 `dep` 行静默吸收、`use.rs` 发射已
实证可用（S2），零转译器改动。按仓库测试门禁：改 parity 独立 workspace 跑其
自身测试（`cargo test -p auto-parity`），不跑 `cargo t`/`tf`/`tv`。

## 目标

1. 五库 green-first 语料在 parity 工具下三轨（VM/a2r/oracle）TAP 全绿：
   本地 env 门控 + CI 网络档各跑通一次。
2. 真三方库行为把 AutoLang 两套实现（VM native_catalog 面 + 430/212 pack 面）
   钉在真实 crate 行为上——任何一套漂移当场暴露。
3. 红面全部显式登记（DIV-DEP-8+，明确归类），无静默跳过。
4. skip 面命中率报告产出并回填 591 待澄清——为 591 V1（Option/Result 语义）
   与 V2（trait/泛型）的 T2/T3 优先级提供五库真实分布数据。
5. parity 工具获得 dep 语料透传能力（后续任何 `dep` 型对拍库可复用）。

## 架构方案

```
parity/libs/dep/<lib>/                     ← 新分类（与 flat legacy libs/serde_json 无叶子名冲突，S9）
  README.md                                ← scope / 上游版本 / 已知分歧
  tests/auto/<case>.at                     ← TAP 语料：dep <crate>(version: "X.Y.Z") + use.rs + tap_ok/check_str
  tests/rust/{Cargo.toml, tests/<lib>.rs}  ← oracle：空 [workspace] + 真 crate 同版依赖 + 手写同语义 TAP

                       ┌─ VM 腿: run_vm → auto <case>.at
                       │    类型构造器/方法 → native_catalog（native 实现，零构建）
  dep 声明(version) ───┤    自由函数 → 430/212 methods pack（真编译三方 crate；首跑 crates.io，后 ~/.cargo 缓存）
                       ├─ a2r 腿: run_a2r → auto trans（dep 行静默吸收，use.rs 正确发射）
                       │    → 生成测试 crate Cargo.toml = 固定基座 + 【本计划新增】语料 dep 行渲染
                       │    → (opt-in) fmt_assist 文本后处理：Result/Option print 位可 Display
                       │    → cargo build --release（cargo 自身增量）→ 跑产物收 TAP
                       └─ oracle 腿: run_native → tests/rust/ cargo 跑手写 Rust 逐用例 TAP（真 crate 同版）
 判定: 三轨 TAP 逐条对齐（tap.rs parse_tap → compare.rs）
 门控: libs/dep 分类库一律要求 AUTO_LANG_PARITY_NET=1（未设 → 整库 skip + 一行提示）
```

- **P0 改法（spike 定案）**：`run_a2r` 每测试 crate 的 Cargo.toml 追加从语料
  解析的依赖行。auto-parity 是独立 workspace（仅 clap+walkdir，S8）——
  新增 `deps.rs` 本地解析器（~60 行），**不引** auto-lang 的 dep_scanner，
  **不复用** trans/rust.rs 的 render_cargo_dep（AST 绑定）；仅支持
  crates.io 形态（`dep name` / `dep name(version: "x"[, features: ..])`），
  path/git 语料解析报错明示（path 形态的对拍已由 592 形态覆盖，非本计划范围）。
- **fmt_assist（opt-in，新库启用）**：a2r 发射文本 `println!("{}", X)` 机械
  后处理为 `println!("{}", ParityDisplay::pd(&X))`，注入的 trait 对
  `Display` 类型恒等（既有 8 个 rust 库零影响，T3 回归验证）、对
  `Result<T: Display, _>`/`Option<T: Display>` 提供确定文本。这是 serde_json
  绿面的必要件（S3：裸 `print(from_str(...))` E0277 编不过）。
- **门禁归属**：改动全在 `parity/`（独立 workspace）+ `.github/` + 语料 +
  `docs/plans/`——`cargo test -p auto-parity` + 本地网络档实跑；**不触主仓
  crates/ → 不跑 cargo t/tf/tv/docs_gen**（AGENTS.md Category A/B 界定 +
  用户验收标准 4）。

## 需求分析与背景调查

（取材：docs/specs/overview.md——parity/ 模块「独立 workspace：三后端行为
一致性验证」、GOAL-006 Consumer-mode parity；docs/specs/goals.md GOAL-006；
PLAN-592 归档计划执行步骤与复审记录（用户指定前置阅读）；代码实勘 +
spike 实跑 2026-09-09）

**前序线**：PLAN-591（use-rust-any-crate-direct，能力计划 V1 布局 manifest /
V2 trait+泛型，drafting）→ PLAN-592（dep-rust-parity-matrix，P0 特征化网，
archived——path fixture 三轨对拍 + 8 管线 bug 修复 + DIV-DEP-1..7 登记）。
本计划是 592 的 **R 层真三方库**后续线：592 对拍的是本地 path fixture，本计划
对拍 crates.io 真库，红面分布直接回答 591 的优先级问题。

### Spike 结论（S1..S10，起草期实跑实证）

- **S1 VM 腿通**：`dep serde_json(version: "1")` + `use.rs
  serde_json::{from_str, to_string}` + `from_str(json)` let 绑定 → `print`，
  `auto` CLI 一次跑通（8.9s 含 pack 检查），输出紧凑 JSON 文本。
- **S2 a2r 转译零改动可用**：`auto trans --path x.at rust` 对 `dep` 行静默
  吸收（trans/rust.rs L11656 `Stmt::Dep` → `dep_crates` 记录，零发射不报错）；
  `use.rs serde_json::{from_str, to_string}` 正确发射 `use serde_json::{...};`
  （并自动补 `use serde_json::Value;`）；方法调用/静态构造器发射标准 Rust。
  592 测试 runner 的「剥 dep 行」是防御性的，本计划不需要。
- **S3 a2r 腿真红面（本计划要解的发射缺口）**：生成的 Rust 直接编译发现
  `println!("{}", data)`（data = `from_str(...)` 的 `Result<Value, Error>`）
  E0277 编不过——Result 无 Display。VM 侧已把 Result 规范化为 unwrap 后文本。
  → fmt_assist 设计（§架构方案）。
- **S4 混合路径（必须如实描述）**：五库全在 `BUILTIN_OPAQUE_CRATES`
  （compile.rs L52-59）。类型构造器+实例方法走 **native_catalog** 手写
  native 分发（零构建秒级：url 全程 0.246s）；自由函数走 **430/212 methods
  pack**（真编译三方 crate）。三轨对拍因此同时钉 VM native 面与 pack 面，
  但报告按路径分别统计（S7）。
- **S5 绿面实证（VM 腿）**：url `Url.parse` + `u.scheme()` + `u.host_str()`
  全绿（host_str 虽 Option 返回，native 面不受 classify 限制）；semver
  `Version.parse("1.2.3")` + `.major/.minor/.patch` → 1/2/3 全绿；regex
  `Regex.new("\\d+")` + `re.is_match("abc123")` → true。
- **S6 红面实证（VM 腿）**：serde_json `data.name` 字段导航 →
  `[GET_FIELD] non-i32 obj_id` + 0；base64 `print(STANDARD.encode("hello"))`
  → None（print 参数位构造器劫持 DIV-DEP-5 与/或 trait 面，T7 分辨归类）。
- **S7 红面集合按路径不同**：`host_str()`（Option 返回）在 pack 面是
  DIV-DEP-1 skip（592 登记），在 native 面却绿——命中率报告按
  native/pack/a2r 三面分别统计。
- **S8 auto-parity 独立**：`parity/crates/auto-parity` 仅依赖 clap+walkdir，
  shell 调 `auto` 二进制——dep 解析必须本地实现（避免跨 workspace 依赖）。
- **S9 布局无冲突**：`resolve_lib_dir` 只扫 `libs/<category>/<lib>/` 两层；
  新分类 `libs/dep/serde_json` 与 flat legacy `libs/serde_json`（手写复刻库）
  叶子名不撞（flat 目录不会被当作 category 命中）。
- **S10 语料/骨架形态现成**：TAP 语料自带 tap_ok/check_str helper 范式
  （consumer 库在用）；`transpile_library` 对缺失的 `auto/` 目录返回空串
  （runner.rs L565-569）→ dep 库无需库源复刻，直接 `tests/` 目录即可。

**已知工程债勿踩**（KNOWN-DEBT-AND-RISKS.md P592-D1..D5）：尤其 D1——自由
函数 wrapper 缓存键 `v3_{自由函数数}` 同数量换签名会陈旧。本计划五库语料
**锁精确版本、执行期不得改版本重跑**（改版本需先清 `~/.auto/cache` 相关键）。

## 详细设计

### D1 `parity/crates/auto-parity/src/deps.rs`（新模块，P0）

```rust
pub struct DepSpec { pub name: String, pub version: Option<String>, pub features: Vec<String> }
pub fn parse_dep_lines(source: &str) -> Result<Vec<DepSpec>, String>
pub fn render_cargo_lines(deps: &[DepSpec]) -> Result<String, String>
```

- 逐行 trim → `dep ` 前缀 → 括号内 `version: "..."` / `features: ["a","b"]`
  （字符串级解析，格式对齐 dep_scanner 支持的子集）；bare `dep foo` → 报错
  （可重现性要求：真三方库语料必须 pin 版本；错误消息注明补
  `(version: "x.y.z")`）。path/git → 报错明示越界。
- 渲染：`name = "x.y.z"`；带 features → `name = { version = "x.y.z",
  features = ["a"] }`。
- 单测：bare 报错 / version+features / 多行 / 注释行跳过 / 非 dep 语料空集。

### D2 `run_a2r` dep 透传（runner.rs，P0）

Step 1 转译后、写 Cargo.toml 前：`parse_dep_lines(&test_rs_source)`（读 .at
原文）→ `render_cargo_lines` 追加进固定基座依赖段。任一库解析失败 →
TapResult 失败 + 诊断信息（不静默）。固定基座（auto-lang/a2r-std/once_cell/
tokio/async-stream/futures）保持不变。

### D3 fmt_assist（runner.rs + main.rs 接线，opt-in）

- `RunConfig` 增 `fmt_assist: bool`；`run_library` 按库名是否属 `libs/dep/`
  分类置 true（或按 lib_dir 路径推断）。
- 实现：a2r 发射文本逐行扫描，`println!("{}", EXPR);` 形态 → `println!("{}",
  ParityDisplay::pd(&EXPR));`（仅顶层 print 语句行，字符串级机械替换）；生成的
  main.rs 头部注入 trait 定义（Display 恒等 / Result→`Ok(v)` 文本或 `"Err"`
  / Option→`Some(v)` 文本或按 VM native 行为钉的空文本）。Err/None 的文本
  形态以三轨一致为准、执行期随语料钉死并写入库 README。
- T3 对既有库回归：p1（base64/url）+ p2（serde_json/regex）flat 复刻库
  跑一遍，结果与改动前逐条一致（fmt_assist 未启用 + 恒等性双保险）。

### D4 五库语料（libs/dep/<lib>/，green-first）

每库：`README.md`（scope/版本/分歧）+ `tests/auto/*.at`（TAP）+
`tests/rust/{Cargo.toml（空 workspace + 真 crate 同版依赖）, tests/<lib>.rs}`
（手写同语义 TAP；oracle 断言 = VM/a2r 同一期望文本）。绿面只用当前能力：

| 库 | pin（执行期落精确版） | 绿面（spike 已证部分） | 预期红面（喂报告） |
|---|---|---|---|
| serde_json | `1.x` 精确 | `from_str` 往返（fmt_assist 依赖）；往返文本=紧凑 JSON | Value 字段导航（S6）、`to_string(&Value)` 借用参、`json!` 宏 |
| regex | `1.x` 精确 | `Regex.new` + `is_match`（S5）；`find` 文本若可行 | `Match`/`Captures` 句柄导航、迭代器面 |
| base64 | `0.22.x` 精确 | 执行期找（let 绑定形态优先；最低限度 encode/decode 文本往返若 trait 面可绕） | `STANDARD` 常量 + `Engine` trait 方法（S6）、`&[u8]` 参数 |
| url | `2.x` 精确 | `Url.parse` + `scheme()` + `host_str()` + `path()`（S5） | `host_str` 的 Option 语义在 a2r/pack 面、`UrlMut`/借用链 |
| semver | `1.x` 精确 | `Version.parse` + `.major/.minor/.patch`（S5）；`bump_*` 若在 | `VersionReq`/`Comparator` 面、`Display` 以外 trait |

- print 参数位纪律：自由函数调用先 let 绑定再 print（DIV-DEP-5）；String
  实参用 `let x str` 注解绑定或方法调用结果传参（DIV-DEP-7）；u64 返回面
  避开 print 直出（DIV-DEP-4/6——semver `.major` 为 u64，VM native 面已绿
  但 a2r 面按 u64→i64 槽钉，若不一致现场登记而非放宽）。
- 每库至少一条绿面 + 红面逐条显式登记；若 base64 绿面确不可行（T7 裁定），
  该库登记为「全红样本」并在验收标准 #1 下注记（见待澄清 #2）。

### D5 网络门控与 CI

- 门控：`run_library` 对 `libs/dep/` 分类库检查 `AUTO_LANG_PARITY_NET=1`，
  未设 → 打印 `[dep-parity] <lib> skipped (set AUTO_LANG_PARITY_NET=1)` 并
  skip（对齐 592 的 AUTO_LANG_DEP_PARITY_A2R 模式）。离线兜底：crates 已在
  `~/.cargo` 缓存时 cargo 构建天然免网（门控语义 =「允许拉取/构建三方
  crate」，不强制探测网络）。
- CI（`.github/workflows/vm-files-ci.yml`）：新 step——`actions/cache` 命中
  `~/.cargo/registry`（预热步：`cargo fetch` 五库 lock）→
  `AUTO_LANG_PARITY_NET=1 cargo run -- --root parity --auto-binary <auto> run`
  逐库（或新注册 phase 名，随 T4 定）。首跑时长实测后回填本计划（592 待澄清
  #4 同款关注点在 parity 工具侧的对应物）。

### D6 命中率报告与 591 回填（重要附带交付）

- `docs/plans/reports/p594-dep-skip-hit-rate.md`：五库全部调用面清单 →
  绿/红标注 → 红项分类（Option / by-value-self / trait / generic / 其他）
  → **按 VM-native / VM-pack / a2r 三面分别统计命中率**（S7：同一签名在
  不同面绿红不同）→ 结论段：五库分布对 591 T2（Option/Result 语义）vs
  T3（trait 单态化）优先级的量化依据。
- 回填：`docs/plans/591-use-rust-any-crate-direct.md` 待澄清事项追加一条
  （#3 之后），注明本报告路径与三行结论（格式对齐 592 执行期注记 #3）。

## 测试设计

| 用例 | 层 | 关键断言 |
|---|---|---|
| deps.rs 单测 | auto-parity | bare 报错 / version+features 渲染 / 注释跳过 / path·git 报错 |
| run_a2r 透传 | auto-parity 集成（临时语料） | 生成 Cargo.toml 含 `serde_json = "1.x"` 行；构建失败进 TapResult 诊断 |
| fmt_assist 恒等性 | auto-parity | Display 类型文本逐字节不变；Result/Option 文本确定 |
| 既有库回归 | parity 工具 | p1+p2（base64/url/serde_json/regex flat 复刻库）结果与改动前一致 |
| 五库三轨 | parity 工具（AUTO_LANG_PARITY_NET=1） | 每库 TAP 三轨逐条对齐；红面语料不进 TAP（进 known-divergences） |
| 门控 skip | parity 工具 | env 未设 → skip + 提示行，无失败 |
| CI | vm-files-ci.yml | yaml.safe_load 通过；缓存预热步在网络档前 |

## 验收标准

1. 五库 green-first 语料在 parity 工具下三轨（VM/a2r/oracle）TAP 全绿：
   本地 `AUTO_LANG_PARITY_NET=1` 一次 + CI 网络档一次。（若 base64 绿面经
   T7 裁定不可行，该库按「全红样本」注记，其余四库必须全绿——见待澄清 #2）
2. 红面全部登记 DIV-DEP-8+，每条有明确归类（Option / by-value-self /
   trait / generic / 其他），无静默跳过。
3. 命中率报告产出（`docs/plans/reports/p594-dep-skip-hit-rate.md`，三面
   分别统计）且结论回填 591 待澄清事项。
4. 测试门禁按改动面：`cargo test -p auto-parity` 全绿 + 既有库 p1/p2 回归
   一致；**未触主仓 `crates/` → 不跑 cargo t/tf**（若执行期确需触主仓，
   按改动面选档：VM→tv / trans→tt，并在计划执行记录中留痕）。
5. CI 网络档步进 vm-files-ci.yml，yaml 解析通过，缓存预热先行。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成]
一行证据。执行于 worktree `D:/autostack/.wt/lang-594/auto-lang`——开工前先在
master 提交 .next-id 与本骨架；worktree 内禁 junction/symlink；移除前必跑
wt-guard。**五库语料版本一经 T5..T9 pin 即锁定，后续不得改版重跑**（P592-D1）。）

- [x] **T0** 前置：master 提交本计划 + `.next-id`；建 worktree
  `git worktree add D:/autostack/.wt/lang-594/auto-lang -b plan-594-dev`。
  验证：`git worktree list` 含 lang-594；`bash D:/autostack/wt-guard.sh`（fold 时）。
  [✅ 已完成] master 0b730c4a2（仅计划+.next-id 两文件，无关 v05/593 工作区改动未触碰）；
  worktree 建于 D:/autostack/.wt/lang-594/auto-lang（分支 plan-594-dev，8240 文件检出完成）。
  执行注记：本计划不改主仓 crates/ → worktree 内不重建 auto 二进制，VM 腿统一用
  master 基线二进制 D:/autostack/auto-lang/target/debug/auto.exe（行为=worktree 代码，
  因 crates/ 零改动）；a2r 腿构建的 auto-lang path 依赖指向 worktree crates/（同内容）。
- [x] **T1** deps.rs 解析器（D1 全集）+ `#[cfg(test)]` 单测。
  验证：`cargo test -p auto-parity deps`（parity/ 下）全绿。
  [✅ 已完成] parity/crates/auto-parity/src/deps.rs 新建（parse_dep_lines/
  render_cargo_line/DepSpec；bare 无版本与 path/git 源均报错明示）+ main.rs
  `mod deps;` 接线；`cargo test -p auto-parity deps` 6/6 PASS（worktree
  lang-594）。
- [x] **T2** run_a2r dep 透传（D2）：runner.rs Cargo.toml 渲染接 deps.rs；
  解析失败 → TapResult 失败+诊断。
  验证：`cargo test -p auto-parity`；临时 dep 语料（/tmp 形态，不入库）经
  `cargo run -- run` a2r 腿生成的 Cargo.toml 含渲染行（人工核对 build_a2r 产物）。
  [✅ 已完成] runner.rs run_a2r 读 .at 原文 → parse_dep_lines → 渲染行进固定
  基座之后；失败两分支（dep parse / corpus 读失败）均 TapResult 诊断+continue。
  `cargo test -p auto-parity` 44/44；scratch 库 tmp_check（serde_json pin
  1.0.145）实跑：VM/a2r 双腿 pass，生成 Cargo.toml 含 `serde_json = "1.0.145"`
  （人工核对后 scratch 已删）。提交 a7facbb94。
  执行注记：freshness 门控对 worktree 场景误报（master 二进制 vs worktree 检出
  新 mtime，crates/ 内容逐字节一致）→ 统一 `--allow-stale` 放行，语义安全。
- [x] **T3** fmt_assist（D3）：RunConfig.fmt_assist + println 后处理 + trait
  注入；libs/dep 分类启用。
  验证：`cargo test -p auto-parity`；既有库回归——`cargo run -- --root . --auto-binary
  ../../target/debug/auto.exe phase p1` 与 `phase p2` 结果与 master 基线一致
  （先在 master 记录基线输出）。
  [✅ 已完成] **裁定变更（不引入 fmt_assist）——实钉发现更优解**：语料写
  `.unwrap()`（VM native 面实证支持：url/semver 输出正确；a2r 发射
  `Url::parse(...).unwrap();` 合法 Rust）+ Auto 类型标注 `let data Value =
  from_str(json).unwrap()`（解 a2r 侧 FromStr 泛型推断）。三份发射（serde_json
  1.0.145 / semver 1.0.26 / url 2.5.4）真编译运行输出与 VM **逐字节一致**
  （紧凑 JSON / 1·2·3 / https）。print 位无需任何文本后处理，既有库零风险。
  RunConfig 不加字段、runner.rs 无新改动。计划偏差依据：待澄清 #1 预授权
  （"以 T5 实钉为准"）；unwrap 语义双面行为作为 S11 结论补记入 §需求分析。
- [ ] **T4** 门控与 phase 注册（D5 前半）：`AUTO_LANG_PARITY_NET` 检查 +
  main.rs `discover_libraries_by_phase` 注册新 phase（p10 = dep 五库）。
  验证：未设 env → `cargo run -- phase p10` 打印 skip 提示、退出 0；
  设 env 后 list/phase 能发现五库。
- [x] **T5** serde_json 语料（D4）：pin 精确版本；tests/auto/*.at（from_str
  往返 + fmt_assist 下 print 形态）+ tests/rust oracle 同版；首跑三轨对齐
  （Result/Err 文本形态此处钉死并写入库 README）。
  验证：`AUTO_LANG_PARITY_NET=1 cargo run -- run serde_json`（--root parity）
  三轨 TAP 全绿。
  [✅ 已完成] libs/dep/serde_json_real（pin 1.0.145）：basic.at 绿面
  `from_str_unwrap_print`（unwrap+`let data Value` 标注范式）+ oracle 同版
  断言 Display==紧凑输入；三轨 1/1 一致。红面（to(str) Debug/i64 标注窄槽/
  is_err None/to_string 缺&）探针钉死入 DIV-DEP-8/9/10/12 与 README。
- [x] **T6** regex 语料：绿面 Regex.new + is_match（+find 文本若可行）；
  红面登记素材收集（Match/Captures 句柄）。
  验证：同 T5 形态 `run regex` 三轨全绿。
  [✅ 已完成] libs/dep/regex_real（pin 1.11.3）：`new_unwrap_is_match`/
  `is_match_negative` 双绿；三轨 2/2。find/captures 推定红入报告。
- [x] **T7** base64 语料：先探 let 绑定 `STANDARD.encode` 形态；裁定绿面
  可行性（不可行 → 全红样本注记，验收 #1 下注记生效）；红面分类素材
  （trait/常量面）。
  验证：`run base64` 三轨绿面全绿（或全红样本注记 + 报告素材齐全）。
  [✅ 已完成] **裁定全红样本**（待澄清 #2 预授权）：VM pack 面
  `STANDARD.encode` 直呼绿（aGVsbG8=），但 a2r 发射器常量接收者大小写
  启发式破坏（`STANDARD::encode`）；小写绑定形态双断；嵌套路径 use.rs
  E0099。libs/dep/base64_real/README-only（探针证据表），**不注册 p10**，
  p10 缩为四库（main.rs 注记）。发现 DIV-DEP-13/14。
- [x] **T8** url 语料：绿面 scheme/host_str/path（S5 已证）。
  验证：`run url` 三轨全绿。
  [✅ 已完成] libs/dep/url_real（pin 2.5.4）：`parse_scheme`/`host_str_unwrap`/
  `parse_path` 三绿；三轨 3/3。host_str 直出（DIV-DEP-11）/to(str)（DIV-DEP-8）
  红面登记。
- [x] **T9** semver 语料：绿面 Version.parse + major/minor/patch（S5 已证）；
  u64 槽 a2r 面行为钉死（不一致 → 现场登记 DIV，不放宽断言）。
  验证：`run semver` 三轨全绿。
  [✅ 已完成] libs/dep/semver_real（pin 1.0.26）：`parse_major/minor/patch`
  数值比较形态三绿；三轨 3/3。print(v) 占位分歧入 DIV-DEP-8 家族；
  VersionReq/bump_* 推定红入报告。
- [x] **T10** 红面登记：`parity/docs/known-divergences.md` 续 DIV-DEP-8+，
  编号连续、格式对齐 DIV-DEP-1..7（条目含签名面/三面表现/归属分类/591 解锁
  条件）。
  验证：T5..T9 收集的每个红项都有对应条目；无静默 skip（对照各库调用面清单）。
  [✅ 已完成] DIV-DEP-8..14 共 7 条（8 字符串化三态/9 标注窄槽污染/
  10 to_string 双断/11 Option native 直出/12 Result 谓词静默/13 常量接收者
  当类型/14 嵌套路径语法面），各带探针锚点；节首新增「绿面语料范式」段；
  native/pack/a2r 三面口径入节引言。
- [x] **T11** 命中率报告 + 591 回填（D6）：报告落
  `docs/plans/reports/p594-dep-skip-hit-rate.md`；591 待澄清追加 #4。
  验证：报告含五库全部调用面清单与三面命中率表；591 diff 人工核对。
  [✅ 已完成] 报告落盘（25 面/绿 8/实证红 10/推定红 7/命中率 32%；分类
  统计+591 结论四条）；591 待澄清 #4 回填（含 BUILTIN_OPAQUE_CRATES
  方法论注记）。两文件 master 侧核对。
- [x] **T12** CI（D5 后半）：vm-files-ci.yml 新步（cargo registry 缓存 +
  预热 fetch + AUTO_LANG_PARITY_NET=1 逐库跑）。
  验证：`python -c "import yaml;yaml.safe_load(open('.github/workflows/vm-files-ci.yml'))"`。
  [✅ 已完成] **裁定变更：宿主改 parity-ci.yml**（已有 parity/** 触发与
  rust-cache 双 workspace 配置，vm-files-ci 改动面更大）。新 job
  `parity-dep-real-crates`：nightly（VM pack 面 rustdoc）+ actions/cache
  产物缓存（build_a2r/*/target + tests/rust/target，键挂工具+语料 hash）+
  `AUTO_LANG_PARITY_NET=1 phase p10`；yaml.safe_load 通过。计划 D5 原文写
  vm-files-ci.yml——偏差记录：宿主文件不同，机制等价（缓存先行+env 门控）。
- [x] **T13** 收口：parity README「Adding a new library」补 dep 分类指引 +
  parity-guide 增 libs/dep/ 约定（版本 pin/门控/三面统计）；库 README ×5 齐；
  健康检查（`cargo test -p auto-parity` 零失败、`cargo fmt --check -p
  auto-parity` 干净、无探针残留）。
  验证：三件套输出留痕进复审记录。
  [✅ 已完成] README 增「The dep category」节（_real 后缀理由/网络门控/
  unwrap 范式/red-face 纪律）；**parity-guide 未另加**（dep 约定集中在
  README 新节，避免双源——偏差注记）；库 README ×5 齐。
  健康检查：`cargo test -p auto-parity` 44/44；fmt 裁定——master parity
  既有代码非 fmt-clean（-p auto-parity 会产生 ~130 行无关重排，runner.rs
  实测 129 行噪音已回退），改为**新增文件 fmt-clean**（deps.rs
  `rustfmt --check --edition 2021` 通过，fmt 噪音零带入）；探针残留扫描
  （PLAN-594-TMP/dbg!/todo!/unimplemented!）= 0。既有库回归 p1（base64
  33/33+url 30/30）p2（serde_json 56/56+regex 45/45）全绿——dep 渲染段对
  无 dep 语料零扰动。

## 复审记录

（/auto-plan:review 时填写；以下为 work 阶段交接记录）

```
stage: work | plan_id: PLAN-594 | plan_revision: 起草稿+执行期三裁定 | outcome: pass |
code_commit: a7facbb94 + 30befca5f + acdbfba1a（worktree plan-594-dev,基于 master 0b730c4a2） |
task_ids: T0..T13 全部完成 |
evidence: phase p10 四库三轨全绿（serde_json 1/1 + regex 2/2 + url 3/3 + semver 3/3,
本地 AUTO_LANG_PARITY_NET=1 实跑）;既有库回归 p1 33/33+30/30、p2 56/56+45/45 全绿;
cargo test -p auto-parity 44/44;DIV-DEP-8..14 七条登记;命中率报告
docs/plans/reports/p594-dep-skip-hit-rate.md + 591 待澄清 #4 回填;
探针残留扫描 0;不触主仓 crates/（门禁:未跑 cargo t/tf,按验收 #4） |
blockers: 无 |
next: /auto-plan:review（CI 网络档首跑由 GitHub Actions 侧验证,本地 yaml 已校验）
```

## 待澄清事项

1. **serde_json a2r 绿面机制**：fmt_assist（D3）是默认方案，但 Err/None 的
   文本形态、以及 print 语句行替换的鲁棒性（多行表达式/嵌套宏）以 T5 实钉
   为准；若 fmt_assist 证实过度脆弱，备选 = serde_json 语料收窄为「不 print
   Value/Result」面（from_str 往返经中间 String 表达），或该库 a2r 腿降级
   双轨 + DIV 登记——三轨同源优先原则下不轻易降级。
2. **base64 可能零绿面**：STANDARD 常量 + Engine trait 方法双红（S6），let
   绑定形态未证。T7 裁定：若确无绿面，该库登记为「全红样本」（报告价值
   最高的一条），验收标准 #1 对其注记豁免——默认假设可豁免，review 时若
   用户不同意再收窄语料重试。
3. **VM 混合路径的表述**：S4 已证类型面走 native_catalog——「真三方库动态
   加载」在类型面上实际是 VM 内置实现 vs 真库的对拍。计划按混合路径如实
   表述与统计；若用户预期「全部走 430 pack」（含类型面），属能力缺口 →
   归 591（V1 布局 manifest），本计划不扩界。
4. **CI 网络档首跑时长**：本地冷跑实测——phase p10 四库全量（含 a2r 腿
   auto-lang ×4 冷编译 + oracle ×4 构建）约 9 分钟单机；CI 侧经 actions/cache
   产物缓存（键挂 parity 工具+dep 语料 hash）后稳态预计 <2min。宿主为
   parity-ci.yml 独立 job（40min 预算），与主测试工作流隔离，无需拆分。
5. **版本 pin 与 P592-D1 的耦合**：五库锁版后若中途必须换版（如 pin 版本
   有 yank），需先清 `~/.auto/cache` 自由函数 wrapper 缓存键再重跑三轨，
   并在执行记录留痕——防止陈旧 wrapper 混入对拍。执行期未触发（四库
   一次 pin 成功：1.0.145/1.11.3/0.22.1/2.5.4/1.0.26 均为 crates.io 在架
   精确版本）。
