# plan-430 C1/C2/C3: 三方 crate 方法 shim 包管线

- 日期：2026-08-23；worktree `429-rust-cleanup` @ `430-shim-metadata`
- 验证环境：Windows x64，stable 1.97 + nightly（双工具链），aliyun crates 镜像

## 交付内容

Phase C 全部三项落地，D2 一并接线：

1. **C1 生成器**（crates/shim-metadata，lib+bin 重构）：
   - `rustdoc.rs` 修复三处 v53 实测陷阱（见下），新增自由函数枚举、按值 self（Move）、
     借用返回标记（`Opaque("&T")`）、`StrOwned`（按值 String 参数所有权区分）；
   - `emit_cdylib.rs` 新模块：MarshalPlan → cdylib shim 包
     （`Cargo.toml`/`src/lib.rs`/`manifest.json`/`signatures.json`/`rules.json` 五件套，
     430-a1 shimpack v1 格式）。wrapper 形如
     `auto_<Type>_<method>_<params>_<ret>`（字母表沿用 plan-212 sig_code，p=裸指针）；
     另导出 `auto__drop_<Type>`（每类型析构）、`auto__free_cstring`、`auto__shim_manifest`。
   - CLI 新命令：`shim-plan <json>`（分类报告）、`shim-emit-pack <json> --crate X --version V --dep-line '...'`（离线产出完整包）。
2. **C2 管线**（auto-cache `methods_pack.rs` + auto-lang 接线）：
   - `Sandbox::compile_dep_methods`：dep 声明的 crate 在自由函数 wrapper 之外自动
     "nightly rustdoc 提取 → 分类 → 生成 → stable cargo 编译 cdylib → 指纹缓存"；
     nightly 缺失自动降级（仅自由函数路径，现状不变）；
   - auto-lang `vm/ffi/dep_methods.rs`：`DepOpaqueObject`（cdylib 对象的 VM 堆形态，
     Drop 回调 `auto__drop_<Type>`，Arc<Library> 保活）+ 方法注册表
     （挂 dispatch 3000 兜底段最后一段：生成段 → 手写臂 → native_catalog → **dep 注册表** → 报错）；
   - marshaller 按 manifest 的 ABI 参数码（含接收者前导 'p'）右到左弹参、按返回码压栈；
     整型统一 i64 槽（规则 6），调用点按真实宽度收窄（u8/u16/u32/u64/usize/f32）。
3. **C3 指纹**：fnv1a64(工具链 × crate × 生成器版本 × 分类器版本 × 排序签名集)；
   签名行记录 `(Ty 名, ABI 码)` 对——宽度变化（u64↔i64）即变指纹即重建。
   缓存名 `{crate}_methods_wrapper-fp{指纹前12}.{dll|so|dylib}`。
4. **D2**：`ffi::resolve_signature` = 方法包元数据优先 → `known_signature` 回退；
   替换了 compile.rs/lib.rs/codegen.rs 全部三处调用点。自由函数签名来自包 manifest
   （rustdoc 提取，仅元数据不生成代码，代码生成仍走 plan-212 syn 路径避免符号冲突）。

## rustdoc JSON v53 三个实测陷阱（解析器修复记录）

1. 固有 impl 归属类型字段是 **`for`**（不是 `for_`）；
2. `resolved_path` 的名字字段是 **`path`**（不是 `name`）；
3. `"trait": null` 在 serde_json 里是 `Some(Null)` 不是 `None`——固有 impl 判定须
   `map_or(true, |t| t.is_null())`。
三处任一写错的表现都是"方法全部掉进自由函数桶/类型变 Unknown"，静默且难查。

## 端到端验证

1. **ffi_dual_013_dep_method**（checked-in 测试，nightly 缺失自动跳过）：
   path-dep fixture `autolang_counter`，覆盖静态构造（String 参数）、&mut void、
   &self i64、&self String、&mut String 参数、&mut i64→i64、&self→不透明新对象、
   静态→String、**ChainInPlace 链式 builder**（bool/i64 参数，压回原句柄）、
   链式别名语义、opaque 参数传参。二次运行走指纹缓存快路径（~0.3s）。
2. **semver（crates.io 真实 crate，网络拉取）**：`Version.new(1,2,3)` 构造成功；
   v1 可用面仅此一项——`major/minor/patch` 是**字段**非方法，parse 族全被
   Option/Result 策略挡下（见边界）。
3. **csv（crates.io）**：44 方法分类成功（含 ReaderBuilder 链式 7 项），但运行时
   撞遗留层：`ReaderBuilder.new` 先命中手写臂造出 RustStdlibObject，后续方法落入
   dep marshaller 报"not a dep crate object"——BUILTIN_OPAQUE 类型新旧层共存碰撞
   的实录（F 阶段逐 crate 迁移的对象）。

## 分类策略新增/修订（v1 边界，全部记入 skips 带原因）

| 场景 | v1 策略 | 依据 |
|---|---|---|
| Option/Result 返回 | 跳过（unwrap policy pending） | 错误转换属例外层第 4 类 |
| 借用返回 &T | 跳过（&str 除外，投影为 Str） | 引用无法装箱 |
| **&self/&mut self 返回 &Self** | **ChainInPlace**：原地改、压回原句柄 | builder 模式（csv 已验证） |
| **按值 self（Move）** | **跳过**（handle invalidation pending） | 见下"崩溃复盘" |
| 拥有的外来类型参数（按值 T） | 跳过 | VM 侧无法构造 |
| 泛型接收者 impl（`impl<R> Reader<R>`） | 方法标记 generic → 走无 mono 提示跳过 | wrapper 无法引用裸泛型 |
| u8/u16/u32/u64/usize/f32 参数 | i64/i32/f64 宽槽 + 调用点收窄 | semver `new(u64×3)` 验证 |

### 崩溃复盘（Move × chain 别名）

fixture 曾含 `bump(self) -> Self`（按值消耗）。ChainInPlace 使 cfg/c2/c3 三个 VM
句柄**别名同一对象**；`c3.bump()` 的 wrapper `Box::from_raw` 消耗对象后，别名全部
悬垂（首版表现为 ACCESS_VIOLATION）。这正是计划"句柄别名/失效语义"例外类（出错
最隐蔽的一类）的实锤。v1 裁定：Move 一律跳过，VM 侧保留接收者句柄置空防御逻辑，
解锁条件 = 例外层给出失效策略（如引用计数或禁别名规则）。

## 遗留（进 F 阶段或例外层）

1. **Option/Result unwrap 策略**——真实 crate 可用面的最大闸门（parse 族构造器全军覆没）；
2. BUILTIN_OPAQUE 类型逐 crate 迁移（csv 碰撞实录；迁一个删一个）；
3. Move 语义解锁（句柄失效策略）；泛型接收者 mono 提示；
4. marshaller ABI 参数上限 3 个（arity≤3 全类组合；超出报清晰错误）；
5. 生成的 wrapper 不含 catch_unwind——panic 跨 `extern "C"` 会 abort（与 plan-212 现状一致）；
6. 方法 key 按短类型名（`Type.method`），跨 crate 同名类型后注册者覆盖（warn 日志）。
