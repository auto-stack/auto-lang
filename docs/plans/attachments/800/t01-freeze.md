# PLAN-800 T-01 责任表与冻结面

基面：master `b0d6d4aab`（2026-10-16）。取材指纹 commit `30d3e1d2a` 与 HEAD 间
`crates/auto-man/src/resolver.rs`、`crates/auto-lang/src/{resolver,mode}.rs`、
`ast/module_path.rs` 零漂移（`git diff --stat` 空），仅根 Cargo.toml +5 行（他线合入）；
本表行号对当前 master 有效。

## 1. 责任表（源声明 → 模式适配 → 已解析依赖视图 → 路径候选 → 语言 trait）

| 责任 | 旧位置（auto-man/resolver.rs） | 新位置 | 依据 |
|---|---|---|---|
| pac.at legacy 行扫描（mode:/use/dep:/("pkg", mode:)） | `load_pac_at` L123-227, `prepare_env` L100-113 | **adapter 保留原样** | 计划 §5：扫描算法与警告/缺文件行为留 adapter；不是 780 静态模型 |
| 模式解析与默认/alias 语义 | `ExecutionMode::from_str` 调用（adapter 内） | **adapter**（语言 `auto_lang::mode`） | 模式标签对核心不透明 |
| 已解析依赖视图（name→path→mode 单一 map） | `dependencies: HashMap<String, Dependency>` L58 | **core**：`PackageResolverCore<M>` 内 `HashMap<String, DependencyEntry<M>>`（唯一权威 map） | §5 单一 map；adapter 不留副本 |
| std 前缀/相对路径/已声明依赖命名规则 | `ModuleResolver::resolve` L414-451 | **core** `resolve(&str)`（trait impl 留 adapter 薄委托） | 语言无关字符串/路径规则 |
| 包路径候选 `find_package_path` | L235-258 | **core**（错误改 core 自有类型，adapter 映射回 `AutoManError::FileNotFound` 保持外部错误面） | 语言无关 |
| 依赖内模块判定 `find_module_in_dep`（file.at/mod.at/歧义） | L361-410 | **core** | 语言无关（segments 以 `&[&str]` 进核心，AutoStr 在 adapter 转换） |
| dep 未声明错误文本（含 Declared dependencies 枚举） | L322-343 | **core** `resolve_in_dependency`（查 map+错误+判定一体） | 纯字符串投影 |
| `exists` 存在性规则 | L465-488 | **core** | 语言无关；**与 resolve 的差异冻结**（见 §3） |
| `search_paths()` 组合（search+std+dep 路径） | L457-463 | **core** | 语言无关 |
| Pac/Super/None 前缀解析 | L350-354（委托 `FilesystemResolver::with_package_root(project_root)`） | **adapter**（继续委托语言 FilesystemResolver） | §5：不复制其语法/根定位 |
| `ModuleResolver` trait impl / `ModulePath`/`PathPrefix` | L413-489, L316-356 | **adapter** | 语言类型不下沉 |
| `AutoManResolver` 公开构造/getter/add_* | L66-301 | **adapter 薄委托**（字段入 core，`default_mode` 留 adapter） | 公开 API 保持 |

## 2. DTO / 单一 map 方案（T-01 论证）

- core 泛型 `DependencyEntry<M> { pub name: String, pub path: PathBuf, pub mode: M }`，
  `M` 对 core 完全不透明（core 代码不 match、不读取 mode）；
  `new(name, path, mode)` 任意 `M`，`with_default_mode(name, path)` 要求 `M: Default`。
- adapter：`pub type Dependency = DependencyEntry<ExecutionMode>;` —— 类型形状
  （字段名/类型/derive Debug+Clone）与旧 struct 一致，`auto_man::resolver::Dependency`
  路径与 `new`/`with_default_mode` 调用面保持；ExecutionMode 为
  `Copy+Eq+Default(AutoVM)`（mode.rs L10/L100），满足泛型约束。
- `AutoManResolver { core: PackageResolverCore<ExecutionMode>, default_mode: ExecutionMode }`。
  `get_dependencies() -> &HashMap<String, Dependency>` 直接借用 core 内唯一 map
  ——**无第二份权威 map**；`add_dependency` 同名替换后
  `resolve`/`resolve_with_prefix`/`get_dependency_mode` 立即看到新路径/新 mode
  （V03 反控用例）。
- core 依赖仅 std（无 auto-val——segments 由 adapter `AutoStr::as_str` 转换；
  无 serde/log——core 不做 IO 日志，日志留 adapter 调用侧）。
- 错误类型：core `find_package_path` 返回自有 `PackagePathNotFound { searched_local: PathBuf }`
  （Display "File not found: {path}"），adapter `map_err` 成
  `crate::AutoManError::FileNotFound`（旧调用点 `load_pac_at` 仅 `if let Ok` 吞错，
  外部不可观察，映射保持一致性）。resolve/resolve_in_dependency 错误维持 `String`。

## 3. 三入口冻结（V02 旧臂；行为=旧代码逐条，改动即行为变更）

### 3.1 `resolve(module_name: &str) -> Result<PathBuf, String>`（trait 臂）
顺序与存在性检查（**resolve 可返回尚未存在的路径**——std/相对/依赖三臂不查盘）：
1. `std.` 前缀：`Ok(std_root.join(format!("{}.at", rest)))`，无存在检查。
2. `./`/`../` 前缀：`Ok(project_root.join(module_name))`，无存在检查，不规范化。
3. 已声明依赖（map 键相等）：`Ok(dep.path.join("package.at"))`，无存在检查。
4. search_paths 按插入序：先 `sp.join(name).with_extension("at")` 存在即返回；
   再 `sp.join(name)/package.at` 存在即返回。
   （注：`with_extension` 会替换已有扩展——如 "a.b" → "a.at"；冻结原语义。）
5. Err：`Module '{name}' not found. Available dependencies: {:?}`
   （`{:?}`=HashMap keys `Vec<&String>` Debug 形态 `["a", "b"]`；
   **HashMap 枚举无序——测试只做集合比较，不把该无序归一到搜索优先序**）。

### 3.2 `resolve_with_prefix(module_path: &ModulePath, current_file)`（inherent）
- `PathPrefix::Dep(name)`：map 查 name；缺失 → 错误文本
  `Dependency '{name}' not declared in pac.at.\n\nAdd it to your pac.at file:\ndep {name}(path: "path/to/{name}")\n\nDeclared dependencies: {keys.join(", ") | "(none)"}`
  （declared 枚举同 3.1 无序处理）；命中 → `find_module_in_dep(dep.path, segments)`。
- `Pac | Super(_) | None`：`FilesystemResolver::with_package_root(project_root.clone()).resolve_with_prefix(...)`，错误文本由语言侧产生，不改写。
- `find_module_in_dep(dep_root, segments)`：拼 `dep_root/seg1/seg2/...`；
  ① `X.at` 存在：若 `X/mod.at` 也存在 → Err `Ambiguous module '{segs.join(".")}' in dependency - both '{file}' and '{dir}' exist`（**歧义不得成功**）；否则 Ok(file)。
  ② `X/mod.at` 存在 → Ok(dir)。
  ③ Err `Module '{segs}' not found in dependency.\n\nSearched locations:\n- {file} (file module)\n- {dir} (directory module)\n\nDependency root: {dep_root}`。

### 3.3 `exists(module_name: &str) -> bool`（trait 臂；与 resolve 差异不抹平）
1. `std.` 前缀：`std_root/{rest}.at` 真存在才 true。
2. map 含键即 true（**不查 dep.path 存在性**）。
3. search_paths：仅查 `sp.join(name).with_extension("at")`（**无 package.at 臂**——
   resolve 第 4 步有、exists 没有，此不对称冻结）。
4. 相对路径不入 exists 检查（resolve 有、exists 无，冻结）。

### 3.4 辅助面
- `search_paths()`（trait）：`[search_paths..., std_root, ...dep paths]` 顺序组合（dep 路径为 map 值序=无序）。
- `find_package_path(name)`：① `project_root/packages/{name}` 目录存在→Ok；
  ② `.../{name}/package.at` 存在→Ok（同值）；③ `std.` 前缀→`std_root/{rest}.at` 存在→Ok；
  ④ Err(FileNotFound(local_pkg))。仅供 `load_pac_at`，私有语义。
- `get_std_root()`：返回 std_root 克隆。
- `get_default_mode`/`get_dependency_mode`：map/default_mode 读取；missing→None。
- `load_pac_at`：读失败→`AutoManError::Io`；pac.at 缺失→`prepare_env` 仅 log::warn；
  坏 mode 行→warn 并保持 default；未找到包→静默不插入依赖。全部留 adapter 原样。

## 4. 共享窗口登记（并行合同短窗口面）

| 文件 | 本计划改动 | 合同规则 |
|---|---|---|
| 根 `Cargo.toml` | members 加 `crates/auto-package-resolver`（带 PLAN-800 注释行）；exclude 加 `test/fixtures/modularity/package-resolver-consumer` | 仅加自己的 member/精确 exclude，不重排不覆盖（合同 §小共享面） |
| `crates/auto-man/Cargo.toml` | dependencies 加 `auto-package-resolver = { path = "../auto-package-resolver" }` | 800 owns resolver core 接线 |
| `crates/auto-man/src/lib.rs` | `pub use resolver::{AutoManResolver, Dependency}` 形状保持（Dependency 需 re-export 类型别名） | 同上 |
| `.autoos/module-boundaries.toml` | domains.members 加 `"auto-package-resolver" = "man-ecosystem"`（含闭包注释） | 777 精确域映射，无宽豁免；`man-ecosystem-no-lang-monolith` 禁向自动适用 |

work 时点 792/794/799 均未合入 master（worktree list：lang-792/794 无——794 无树、
792 有树未并；799 无树无计划文件）。V05 按合同"后合入者责任"登记，不预写 792 清单。

## 5. 分支命名偏差记录

计划 §6 拟 branch `codex/plan-800-dev`；本仓 AGENTS.md L1 规范及全部现存树为
`plan-<NNN>-dev`，采用 **`plan-800-dev`**（组目录 `D:/autostack/.wt/lang-800/auto-lang`
按计划）。纯命名偏差，无语义影响。
