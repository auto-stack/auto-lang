# trans 相关 plan 索引

> 状态以各 plan 文件自身为准；归档列为当前所在目录（`plans/`、`plans/archive/`、`plans/old/`）。
> 主题概览见 docs/plan-indices/06-transpilers.md。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 007 | implement-a2r-transpiler | partial（Phase 1 完成） | old/ | a2r 初建，沿用 a2c 架构模式 |
| 022 | python-transpiler | ✅（2025-01-14） | old/ | a2p 十阶段实现，f-string/match 直映 |
| 023 | javascript-transpiler | ✅（2025-01-14） | old/ | a2j 十一阶段，9/9 测试 |
| 062 | c-transpiler-generics | ✅ | old/ | a2c 泛型单态化 |
| 066 | incremental-transpilation | ✅（2025-02-01） | old/ | a2c/a2r 接入 Database，Universe 双轨过渡（ADR-04） |
| 067 | strengthen-rust-transpiler | planning | old/ | a2r 对齐 a2c 的差距分析 |
| 083 | a2r-transpiler-with-rsat | partial | old/ | `.rs.at` 平台专属实现文件与 `#[rs]` 注解 |
| 100 | a2js-to-a2ts | partial（Phase 2/3 完成 2026-03-01） | old/ | 默认生成 TS，javascript.rs 保留（ADR-02） |
| 152 | a2ts-typescript-transpilation-design | ✅ | old/ | a2ts 功能对齐 + 四模块拆分（ADR-03） |
| 161 | a2r-list-implementation | ✅ | old/ | a2r `List<T>` 与 `.as(Type)` cast |
| 162 | method-keyword-to | ⏳ | old/ | `.to(Type)` 显式转换关键字（未实施） |
| 163 | a2r-core-struct-support | ✅ | old/ | 静态方法/嵌套字段/枚举 tag 值/Option-Result/自定义属性 |
| 164 | a2r-ext-for-external-trait | ⏳ | old/ | `ext Type for Trait` 外部 trait 实现（未实施） |
| 165 | a2r-struct-destructuring | ⏳ | old/ | is 分支内结构解构（未实施） |
| 166 | a2r-generic-constraints | ⏳ | old/ | `#[with(T as Trait)]` → `<T: Trait>`（未实施） |
| 167 | module-system | ✅ | old/ | 模块系统；MultiSink 多文件输出 |
| 168 | shared-variable | ✅ | old/ | shared 变量 + pub 迁移；escape_str 转义统一 |
| 170 | a2r-test-reorganization | ✅ | old/ | a2r 测试分类化目录结构 |
| 171 | a2c-test-reorganization | ✅ | old/ | a2c 测试分类化 |
| 172 | a2ts-test-reorganization | ✅ | old/ | a2ts 测试分类化 |
| 173 | r2a-rust-to-auto-transpiler | ✅ | old/ | 基于 syn 的逆翻译（ADR-08） |
| 204 | a2r-transpiler-completeness | ✅ | old/ | Result/spec/struct/enum/stdlib 映射/安全输出六阶段 |
| 213 | a2py-maturation | ✅ | old/ | a2p 覆盖率 18%→80%+ |
| 215 | a2ts-maturation | ✅ | old/ | a2ts 扩至 80+ 测试 |
| 219 | playground-source-map | ✅ | old/ | Sink source map 全目标打通 |
| 220 | a2r-transpiler-improvement | ✅ | old/ | 类型映射/枚举/stdlib 覆盖改进 |
| 223 | a2r-step00-transpiler-fixes | ✅ | old/ | lexer pos drift、多参枚举变体、is 表达式修复 |
| 229 | self-hosting-via-a2r | ✅（Phase 4） | old/ | a2r 作为自举落地路径 |
| 232 | a2r-lexer-compilation | ✅ | old/ | `.sub()`/`.slice()` 映射 + post_process() 类型修正 |
| 240 | rust-cookbook-a2r-tests | ✅ | archive/ | 163 个 cookbook .at 全 assert 化，124/124 通过 |
| 241 | a2r-string-type-cleanup | ✅ | old/ | get_or/insert 的 .to_string() 启发式修正 |
| 242 | a2r-feature-gap-tracker | active（living doc） | plans/ | a2r 遗留缺口清单（活文档，持续更新） |
| 263 | transpiler-tests | ✅ | old/ | 约定式测试发现 tests/a2*_tests.at（ADR-05） |
| 264 | a2r-dot-to-double-colon | ✅ | old/ | 模块路径 `.` → `::` 映射 |
| 266 | vm-a2r-conformance | partial（Phase 1） | old/ | AutoVM 与 a2r 语义一致性 |
| 271 | remove-a2r-examples | ✅ | old/ | 清理 a2r 预期文件中的 example 声明 |
| 283 | a2py-maturation-plan | ✅ | old/ | a2p import 系统/类型跟踪/PyDep 依赖收集 |
| 290 | a2gd-transpiler | ✅ | old/ | GDScript 后端初建，9/9 测试 |
| 305 | a2gd-maturation | ✅ | archive/ | a2gd 对齐 a2py 功能面 |
| 310 | auto-ownership-escape-analysis | ✅（2026-06-16） | archive/ | 逃逸分析 own-by-default + 保守回退（ADR-06） |
| 328 | a2r-http-server-architecture | 设计完成待实施 | plans/ | `#[api]` → Axum 原生 server 转译设计 |
| 355 | a2r-async-await-transpilation | 设计文档/TODO | plans/ | a2r async/await 转译设计（从 Plan 344 拆出） |
| 364 | a2r-cosmic-replication-readiness | ⏳ | plans/ | 为 COSMIC 桌面复制补齐 a2r 缺口 |

备注：

- **355 重编号冲突**：`plans/355-a2r-async-await-transpilation.md`（本表所引）与
  `plans/archive/355-fix-persistent-session-fn-body-recursion.md`（auto-shell session 修复，与 trans 无关）
  同号并存；引用 355 必须带 slug。
- 242 是活文档，其状态由自身维护；其指向的子项完成度以 plan 文件内表格为准。
