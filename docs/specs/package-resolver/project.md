# auto-package-resolver

> **Status**: active
> 路径：`crates/auto-package-resolver`  | 技术栈：Rust（std-only，零第三方依赖）

已解析依赖视图与模块路径解析核心（PLAN-800 从 auto-man `resolver.rs` 拆出）：
依赖条目模型、包路径候选、依赖内模块判定、std/相对/命名依赖/search 路径命名
规则与错误文本投影的单一定义。可被无 auto-lang/auto-man/auto-val/auto-ast/
auto-gen/auto-vm/UI 闭包的外部 consumer 独立消费（外部消费方探针
`test/fixtures/modularity/package-resolver-consumer`，闭包断言 `verify-deps.sh`）。

## 目标与范围

- 模型：`DependencyEntry<M>{name,path,mode}`——`M` 为**不透明模式标签**，
  核心从不读取其内容；`new` 任意 `M`，`with_default_mode` 要求 `M: Default`
  （auto-man 侧实例化为 `DependencyEntry<ExecutionMode>` 即旧公开 `Dependency`）。
- 核心：`PackageResolverCore<M>` 持**单一权威 map**（`HashMap<String, DependencyEntry<M>>`）
  + std/project roots + 插入序 search paths；同名 `add_dependency` 替换后
  resolve/resolve_in_dependency/依赖查询立即看到新路径与新 mode（无第二份 map）。
- 规则：`find_package_path`（project packages/ 目录 → package.at → `std.` 前缀
  std_root/`.at`）、`find_module_in_dep`（`<segs>.at` file 模块优先于
  `<segs>/mod.at` directory 模块；并存=歧义错误）、`resolve`（std./相对/已声明
  依赖/search 路径命名规则）、`exists`（存在性规则）、`module_search_paths`
  （search+std+dep 路径组合）。
- 不做：pac.at 解析（legacy 行扫描留 auto-man adapter；780 静态源模型不在此）、
  ExecutionMode 解析/默认/alias 语义（语言适配层）、`ModulePath`/`PathPrefix`/
  `ModuleResolver` trait（语言类型）、Pac/Super/None 前缀解析（auto-man 继续委托
  语言 `FilesystemResolver`）、依赖下载/物化/版本求解、锁格式
  （auto-package-lock 所有权）。

## 依赖边界（红线）

- 依赖仅 std（连 auto-val 也不依赖——segments 以 `&[&str]` 进入核心，
  `AutoStr` 转换在 adapter）。
- **禁止**依赖 auto-lang / auto-ast / auto-man / auto-val / auto-gen / auto-vm /
  任何 renderer/UI/解释器栈。闭包由 package-resolver-consumer fixture +
  module-boundaries 基线钉住（man-ecosystem 域，`man-ecosystem-no-lang-monolith`
  禁向规则适用）。
- 核心无 IO 日志、无网络、无物化、无执行动态配置（调用侧日志留 adapter）。

## 冻结语义（PLAN-800 T-01 旧臂，b0d6d4aab 探针冻结；改动=行为变更须显式决策）

1. **resolve 可返回尚未存在的路径**：std/相对/已声明依赖三臂构造即返回、
   不查盘；只有 search 路径臂探测磁盘存在（file `.at` → 目录 `package.at` 两候选，
   插入序）。相对路径不做规范化（`./`、`../` 字面 join）。
2. **exists 与 resolve 差异不抹平**：exists 的 std 臂要求磁盘真存在；map 含键
   即 true（不查 dep.path）；search 臂只查 `.at` file 候选（**无 package.at 臂**）；
   相对路径不入 exists。默认 trait `exists`（resolve+查盘）不适用此实现语义。
3. **歧义不得成功**：依赖内 file 模块与 directory 模块并存 →
   `Ambiguous module '<segs>' in dependency - both '<file>' and '<dir>' exist`。
4. **错误文本逐字冻结**：dep 未声明
   （`Dependency '<n>' not declared in pac.at.` + `Add it to your pac.at file:`
   + `dep <n>(path: "path/to/<n>")` + `Declared dependencies: <keys.join(", ")|(none)>`）、
   依赖内未找到（`Searched locations:` 双候选 + `Dependency root:`）、
   resolve 未找到（`Module '<n>' not found. Available dependencies: [<keys>]`——
   HashMap keys 的 Debug 形态）。
5. **无序枚举只归一无序比较**：错误中的 declared/available 依赖枚举来自
   HashMap 迭代（无序）；测试按集合比较，不把该无序归一到搜索优先序
   （search 路径插入序、file→dir 候选序、std/相对/依赖/search 臂序全部保持）。
6. **`with_extension` 语义保持**：候选构造用 `with_extension("at")`（替换已有
   扩展，如 `a.b`→`a.at`），不改为 append。
7. **包路径候选错误**：`PackagePathNotFound{searched_local}`（Display
   `File not found: <path>`）——adapter 映射回 `AutoManError::FileNotFound`；
   旧调用点（load_pac_at）对 Err 静默不插入依赖，外部不可观察。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| lib | DependencyEntry/PackagePathNotFound/PackageResolverCore + 全部路径规则 | active |

## 验证

- 核心单测（`crates/auto-package-resolver/src/lib.rs` tests）：三臂无盘构造、
  相对路径不规范化、search 两候选、exists 三差异（std 真存在/键即真/无
  package.at 臂）、组合序、包路径候选优先级与错误、依赖内 file/dir/歧义/缺失
  四态、未声明文本、同名替换立即可见、泛型 mode 不透明（`&'static str`/`u8`
  实例化）。
- 外部消费方探针：`cargo test --manifest-path
  test/fixtures/modularity/package-resolver-consumer/Cargo.toml`（7 例，consumer
  自定义 `ConsumerMode` 证模式不透明）+ `bash
  test/fixtures/modularity/package-resolver-consumer/verify-deps.sh`
  （normal/build 闭包 = 仅 auto-package-resolver）。
- auto-man adapter 端到端（pac.at 行扫描、ExecutionMode、ModuleResolver trait、
  Dep/Pac/Super 前缀、旧 API 形状）由 `crates/auto-man/src/resolver.rs` 测试钉住。

## 已知债与未迁项（诚实登记）

- pac.at legacy 行扫描（mode:/use/dep:/("pkg", mode:)）与 ExecutionMode 语义留
  auto-man adapter；`use`/`dep:` 行获得硬 AutoVM 默认而非工程 default_mode——
  历史行为，冻结不改（如需语义修复另立显式合同）。
- Pac/Super/None 前缀与 `FilesystemResolver` 的语言侧规则不在本核心；语言
  resolver trait 不删除。
- Target/build/Git/动态配置/CLI 未迁（与 PLAN-795 未迁清单一致）；
  `auto-man→auto-lang` 边在 module-boundaries 基线 grandfather（exit R5）。
