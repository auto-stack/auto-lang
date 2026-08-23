---
plan: 430
title: aavm-shim-metadata-tool（shim 映射流程重构：rustdoc 元信息工具）
affects: [docs/specs/auto-lang/runtime/design/ffi-bridges.md, auto-lang/vm]
status: draft
---

# Plan 430: shim 映射流程重构——rustdoc 元信息工具

> **For Claude:** 执行上下文：worktree 名 `plan-430/shim-metadata`。
> 构建/测试：`cargo test -p auto-lang --lib -- a2r_tests` + ffi_dual 测试（`test/ffi_dual/`）+
> golden `17_rust_std`/`18_pure_rust` 相关用例。
> 前置：Plan 429 的 B1 盘点报告（shim 需求清单是本工具首个裁剪输入）。
> **迁移全程允许新旧共存，无大爆炸切换。**

## Goal / 目标

把 `use.rust` 在 VM 路径下的映射机制从"手写白名单"重构为"元数据驱动 + 代码生成"：

1. 做一个**离线元信息工具**：扫描目标 crate（含 std）的 rustdoc，结合规律规则层，
   产出 **shim 包**（签名元数据 + 规律/例外 + 生成的 shim 源码 + 编译产物）；
2. 编译器启动时加载 shim 包的元数据与预编译入口，**运行期零代码生成**；
3. dep 管线接入：导入新三方 crate 时自动多做一步"元信息提取 + shim 包编译"；
4. 渐进迁移现有手写层（dispatch 3000 的 ~111 臂 + `known_signature` 表 + 40 个
   `BUILTIN_OPAQUE_CRATES`），迁一个删一个；
5. 首个交付物裁剪为 **String/Vec/HashMap 子集**——同时满足 AAVM（计划 432）的全部 shim 需求。

完成后："未来添加新的默认库 = 改工具配置 + 跑一遍工具"，手动层归零。

## 背景 / 已确认的决策

（2026-08-23 与用户讨论定稿的方案，论证过程记录于此，执行时不必重推）

### 三层现状

| 层 | 覆盖 | 现状 |
|---|---|---|
| 手动层① | std + ~40 白名单 crate（chrono/rand/regex/serde_json…） | 手写 dispatch 3000 match 臂（`vm/ffi/stdlib.rs:6851`，~111 臂）+ 手写 `ffi.rs:1047 known_signature` |
| 半自动层② | 白名单外经 `dep` 声明的 crate | 沙箱自动生成 `auto_<fn>_<sig_code>` cdylib wrapper（Plan 212，syn 扫描），但只支持自由函数 + 标量（sig_code 字母表 v/i/l/f/b/s/p），**无方法调用** |
| 全自动层③ | 一切 | a2r 直译（无 shim 概念），不动 |

### 关键设计结论

1. **元信息 ≠ 执行**：rustdoc 签名只替代"信息层"（known_signature、校验、推断、通配展开）；
   进程内调用仍需已编译的胶水代码。生成物分两个编译目的地：
   - **std → 构建期**：生成的 shim 源码编入 auto-lang 二进制（加默认库需重编 auto-lang，可接受）；
   - **三方 crate → cdylib shim 包**：wrapper 是完整 Rust 代码，可链接真实 crate 并包装方法调用
     （不透明对象留 VM 堆，跨 ABI 传裸指针：`extern "C" fn auto_Duration_as_secs(h: *const c_void) -> u64
     { unsafe { &*(h as *const Duration) }.as_secs() }`）。**方法调用可以过 C ABI，此前"永远解不了"
     的判断已修正**——现有沙箱只是没做。
2. **规律层**（签名分类器，纯 rustdoc 驱动，覆盖 ~80-90%）：

   | 决策 | 规则 |
   |---|---|
   | 返回值标量 vs 不透明装箱 | 返回类型在 marshalling 字母表内 → 压标量；外来类型 → `RustStdlibObject` 压句柄 |
   | self 读写 | `&self` → 读锁 + `downcast_ref`；`&mut self` → 写锁 + `downcast_mut` |
   | 链式方法 | 返回 `Self`/`&mut Self` → 原地修改、压回接收者句柄 |
   | 静态构造器 | 无 self 关联函数 → 弹参数、按返回值规则装箱/压标量 |
   | String/&str 参数 | 默认借用（`&str` 从池弹借用）；`String` 弹后克隆转移 |
   | 整数宽度 | i32/u32→i32 槽，i64/u64→i64 槽（顺带修正手写臂中 u64→i32 的有损截断） |

3. **例外层**（真正需逐函数人工的仅四类，存例外表）：
   - 泛型单态化选择（`HashMap::get<K>` 常见默认键=String；元素=不透明）；
   - 跳过清单（闭包/函数指针参数、`impl Trait`、裸生命周期）——判定机械，无需思考；
   - 句柄别名/失效语义（`Vec::push` 是否使迭代器句柄悬空等）——量少、出错最隐蔽；
   - panic/错误转换的个别文档语义。
   预估每 crate 个位数到十几个条目；首版 String/Vec/HashMap 子集 ≤20 条。
4. **shim 包结构**（per crate）：
   `① signatures.json（rustdoc 提取）② rules+exceptions ③ 生成的 shim Rust 源码
   ④ 编译产物（std 编入 / cdylib）`。①② 供编译期校验/推断层复用（顺带解锁 Plan 190 推迟的
   四项：type stubs、rustdoc 集成、通配展开、trait 方法解析），③④ 供 VM 执行层。
5. **格式**：JSON 为事实源（rustdoc 直接产 JSON，serde_json 已在树内）；转存 Atom 格式为后续美化，非首版必需。

## 任务（按阶段）

### Phase A：可行性验证（1-2 天）

- [ ] A1 在当前工具链验证 `cargo doc --output-format json`（历史上需 nightly，确认现状）；
  若不稳定，备选方案：`cargo metadata` + syn 扫描 crate 源（对 std 用 rustup component 源码）。
- [ ] A2 确定 rustdoc JSON 顶层结构的解析子集（我们只要：类型/方法/关联函数/签名/self 修饰/
  可见性/泛型参数标记），写最小解析器原型，对 `std::collections` 跑通。
- [ ] A3 元信息格式 v1 定稿（schema 放 `docs/specs/auto-lang/runtime/design/` 或工具目录 README）。

### Phase B：分类器与例外表（2-3 天）

- [ ] B1 实现 6 条签名分类规则（上表），输出"每方法的 marshalling 计划"（中间表示）。
- [ ] B2 例外表格式与加载（mono 提示 / 跳过 / 句柄语义注记 三类条目）；
  分类器默认值 + 例外覆盖，类似 a2r 的 companion trait 模式。
- [ ] B3 对照验证：用分类器跑现有 40 crate 的手写臂覆盖面，输出 diff 报告
  （哪些臂是纯规则可生成的、哪些靠例外、哪些手写臂本身可疑——如 u64→i32 截断）。

### Phase C：shim 包生成器（3-5 天）

- [x] C1 代码生成器：marshalling 计划 → shim Rust 源码。
  std 目标：生成 `match (type, method)` 臂的机器版（结构与现 stdlib.rs 臂一致，可并存的追加段）；
  三方目标：生成 cdylib shim 包 crate（`auto_*` wrapper + manifest，复用 Plan 212 的
  sig_code/manifest 机制并扩展方法 wrapper）。
  （三方路径 2026-08-23 落地：`shim-metadata::emit_cdylib` 五件套 + CLI `shim-emit-pack`，
  报告 430-c1）
- [x] C2 编译产物管线：三方 crate 在 dep 流程中加"提取元信息 + 编 shim 包"一步
  （与现有 cargo 下载/编译 cdylib/加载 同构，用户感知仅为首次导入稍慢，之后缓存）。
  （`Sandbox::compile_dep_methods` + dispatch 3000 兜底段接线；nightly 缺失自动降级）
- [x] C3 元信息版本指纹（工具链版本 × crate 版本的 hash 校验），防签名漂移。
  （fnv1a64(工具链 × crate × 生成器 × 分类器 × 签名集)，签名行含 Ty 宽度；缓存名 fp 前缀）

### Phase D：dispatch 改造与渐进迁移（持续）

- [x] D1 dispatch 3000 改为"先查 shim 包映射表，未命中再走手写 match 臂"的混合查找
  （与 NativeInterface 现有静态/动态混合查找模式对齐）。
- [x] D2 `known_signature` 改为"先查元信息，未命中回退手写表"。
  （`ffi::resolve_signature`：方法包 manifest 自由函数签名优先 → known_signature 回退；
  compile.rs/lib.rs/codegen.rs 三处调用点已替换）
- [ ] D3 迁移一个最简 crate（建议 `Box`/`RefCell` 这类臂极少的）端到端走通：生成包 →
  加载 → 查表调用 → 删除对应手写臂 → 测试全绿。

### Phase E：首个正式交付——String/Vec/HashMap 子集（3-5 天）

- [ ] E1 按 Plan 429 B1 盘点报告裁剪配置，生成三者的 shim 包（例外条目预计 ≤20）。
- [ ] E2 用例验证：现有 golden `17_rust_std`/`18_pure_rust` + `test/ffi_dual/` 全绿；
  补 String/Vec/HashMap 方法级用例（目标：覆盖 AAVM 移植所需全集，清单引用 429 B1 报告）。
- [ ] E3 删除三者对应的手写臂与 known_signature 行。
- **此阶段完成即解锁 Plan 432 的 shim 依赖**（432 可提前并行启动其 lexer 切片）。

### Phase F：40 crate 全量迁移（E 完成后排期，可跨版本）

- [ ] F1 按"臂数量少→多"排序逐 crate 迁移（每 crate：生成 → 验证 → 删臂 → commit）。
  （2026-08-23 F 轮裁定：**builtin crate 整体迁移被两类能力缺口挡住**——①字段访问
  （semver major/minor/patch 是字段非方法，legacy 静态表面向字段）；②Display/trait
  方法（to_string 等，v1 只取固有 impl）。D3 以 std Duration 迁移替代完成（见执行结果）；
  builtin 迁移的前置 = dep 对象字段访问通道 + trait 方法解析（Plan 190 挂账项）。
  std 手写臂侧：Duration 5 臂已迁生成段，剩余臂按同法逐个迁。）
- [ ] F2 全部迁完后：`BUILTIN_OPAQUE_CRATES` 白名单语义改为"预生成 shim 包的默认配置清单"；
  `shim_rust_stdlib_dispatch` 手写臂清零退役。
- [x] F3 演示项：往默认配置加一个新 crate（如 `semver`），跑工具 → 新库立即可用，零手写代码。
  （2026-08-23 以 **uuid**（不在 builtin 清单）达成：`dep uuid(features: ["v4"])` +
  `use.rust uuid::{Uuid}` → new_v4/get_version_num/parse_str(unwrap_ok)/is_nil 全通，
  零手写代码；附带落地 unwrap_ok 策略使 parse 族构造器可用。报告 430-f1。）

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| rustdoc JSON 不稳定/需 nightly | A1 首日验证；备选 syn 源码扫描（三方本来就有源码；std 走 rustup component） |
| 生成 shim 与手写臂行为微妙不一致 | 迁移期内每个 crate 的验证以现有 ffi_dual/golden 为准；可疑手写臂（有损截断类）在 B3 diff 报告中单独标记并人工裁决 |
| cdylib 方法 wrapper 的裸指针安全 | 句柄来自 VM 堆（`RustStdlibObject` 生命周期由 heap_objects 管理）；例外表"句柄语义"字段强制标注失效规则 |
| 首次导入延迟增加 | shim 包按 (crate, version) 缓存；CI 预生成 40 默认 crate |
| 与 Plan 417 后仍在演进的 native id 体系冲突 | 统一走既有 NativeInterface 分段契约（100-199 Rust FFI），不新开 id 段 |

## Out of Scope

- 运行时 JIT / 进程内动态代码生成（明确不做）
- rustdoc 元数据驱动的 IDE 全功能（补全等只做"顺带可用"，不追求完整）
- C 目标（`trans/c.rs` 对 use.rust 报错的现状不变）
- a2r 路径（无 shim 概念，不动）

## Verification

1. Phase E 出口：String/Vec/HashMap 的 shim 全部由工具生成，对应手写臂删除，
   `17_rust_std`/`18_pure_rust` golden + ffi_dual 全绿；
2. 新三方 crate 端到端：`dep foo` → 自动出 shim 包 → VM 调用成功（含一个方法调用用例）；
3. B3 的 diff 报告归档；发现的手写臂可疑行为（有损截断等）有逐条裁决记录；
4. 40 crate 迁移进度表（F1）持续回填本文件。

## 执行结果（进行中,2026-08-23）

- **Phase A**（c0c8f50d）：nightly rustdoc JSON v53 解析打通（签名/self 可变性/泛型齐备）；
  stable 1.97 不支持；std 本体配方暂未打通 → v1 决策：std 走手编目录，三方走 rustdoc。
  元信息 shim 包格式 v1 定稿（报告 430-a1）。
- **Phase B**（566680b8）：crates/shim-metadata 工具落地——rustdoc v53 解析器、6 条规律分类器、
  mono/skip/note 例外表、std 目录 v1、std 追加段代码生成器；std-plan 冒烟 61 plans/0 skips。
- **Phase D+E 主体**（本轮）：
  - D1 接线：generated_std.rs（生成段，~890 行）编入 auto-lang，dispatch 3000 手写臂之前优先调用；
    push_rust_obj 提为 pub(crate)。
  - 生成器迭代（借 rustc 当检查器）：turbofish、Result<Option<()>> 签名、借用块作用域内
    .to_string()/.copied()/.map、usize 参数、owned-key 方法、ferr 错误包装。
  - **Vec 全链路打通**：10 万次 push + len + is_empty 输出全部正确（429-B2 的 len=0 bug 场景修复）。
  - codegen 修复：泛型构造器路径（Plan 087）与 use.rust 导入类型撞名时 rust 导入优先
    （Vec.new 此前被劫持为 GenericInstanceData 占位——429-B2 bug 的第二根因）。
  - **E 阶段设计裁定**（实测驱动，修订原计划）：
    1. String 是标量池值非堆对象——方法走引擎 str 原生路径（len/contains/to_uppercase/replace/trim
       实测全对），生成段只留 String.from/new 静态构造器；
    2. HashMap/HashSet 由 Auto 原生 Map 路径拥有（auto.hashmap natives + a2r 真 HashMap 双路径
       golden 已证），生成段不覆盖，避免表示分裂；AAVM 规范：Map 用带类型注解的 Auto 语法；
    3. Vec 用生成 shim（opaque Vec<i64> 堆对象）。
  - 已知长尾（后续）：`use.rust std::collections::HashMap` 导入语句会使构造器推断路径劣化
    （带类型注解但不 import 可用）；`String.from(x) + y` 拼接场景返回句柄——均登记为
    codegen 推断缺口，进本计划 F 阶段或 242 tracker。
- **Phase C/D2**（本轮，报告 430-c1）：
  - C1 三方 shim 包生成器：shim-metadata 拆 lib+bin（lib 供 dep 管线进程内调用），
    `emit_cdylib` 产出五件套（Cargo.toml/src/lib.rs/manifest/signatures/rules）；
    rustdoc v53 解析器修三处实测陷阱（`for` 字段、`path` 字段、`trait:null≠缺失`），
    新增自由函数枚举与 Move/借用返回/StrOwned 投影；
  - C2 管线接线：`Sandbox::compile_dep_methods`（nightly rustdoc → 分类 → 生成 →
    stable cargo 编译 cdylib → 指纹缓存，nightly 缺失降级）；auto-lang 新增
    `vm/ffi/dep_methods`（DepOpaqueObject 堆对象 + 方法注册表，挂 dispatch 3000
    兜底段最后一段）；类型导入（大写 use.rust 项）不再进自由函数 wrapper；
  - C3 指纹：签名行记 (Ty 名, ABI 码) 对，宽度变化即重建；
  - D2：`resolve_signature` 元数据优先回退 known_signature；
  - 端到端：ffi_dual_013（path-dep fixture，覆盖静态/&mut/&self/void/字符串/i64/
    不透明返回/ChainInPlace 链式/opaque 传参）+ semver/csv 真实 crate 实录；
  - v1 边界裁定（详见报告）：Option/Result 返回跳过（unwrap 策略待例外层，真实 crate
    最大闸门）；按值 self 跳过（chain 别名 × 消耗语义冲突，实测 ACCESS_VIOLATION 复盘）；
    泛型接收者 impl 跳过；BUILTIN_OPAQUE 类型新旧层碰撞（csv 实录，F 阶段逐 crate 迁移）。
- **Phase F 首轮**（本轮，报告 430-f1）：
  - **unwrap_ok 策略落地**（F 遗留首优先级）：rustdoc 投影把 `Result<T,E>` 解包为 T +
    fallible 标记；wrapper 解 Ok，Err 写入 cdylib 线程局部错误通道
    （`auto__last_error`/`auto__clear_error`），VM 侧 fallible 方法压栈前查通道转
    VMError——**错误消息完整穿透三层**（semver 实测：
    `Version::parse failed: unexpected character 'n' while parsing major version number`）。
    Option 仍跳过（None 语义待例外层）；std/三方分类双轨：std 保 Option 装箱语义
    （防 Vec.get 等丢臂），三方跳过。
  - **rustc 检查器剔除环**：包构建失败时从报错提取肇事符号，剔除对应方法重试——
    个别不可编译的 wrapper（u128 参数/跨 crate opaque 实参）不再弄死整包。
  - **D3（std Duration 变体）**：from_secs/from_millis/from_secs_f64/as_secs/as_secs_f64
    五臂迁 std 目录 → 生成段，手写臂删除，全量 3128 绿；**顺手修正遗留 u64→i32 有损
    截断**（5e9 秒不再变 705032704）。发现 "Duration" 标签下混用 chrono/std 两种具体
    类型（days/hours/seconds 是 chrono），chrono 系与 u128 返回的 as_millis 族暂留手写。
  - **F3（uuid）**：真实 crates.io 新 crate 零手写即插即用（见上）。
  - ** marshaller 修复**：bool 返回只保证 al 有效——读 i64 槽必须掩码（实测 uuid
    is_nil 曾把垃圾高位带回触发 48 位越界 panic）；整型压栈改 heap-aware push_i64_vm。
  - 分类器新增边界：128 位参数跳过；rustdoc `crate::` 前缀路径剥离（uuid 根重导出实测）。
