# trans 相关 plan 索引

> 状态以各 plan 文件自身为准；归档列为当前所在目录（`plans/` 活跃、`archive/` 归档；2026-09-07 复核 328/355/364/400 已归档）。
> 主题概览见 docs/plan-indices/06-transpilers.md。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 007 | implement-a2r-transpiler | partial（Phase 1 完成） | archive/ | a2r 初建，沿用 a2c 架构模式 |
| 022 | python-transpiler | ✅（2025-01-14） | archive/ | a2p 十阶段实现，f-string/match 直映 |
| 023 | javascript-transpiler | ✅（2025-01-14） | archive/ | a2j 十一阶段，9/9 测试 |
| 062 | c-transpiler-generics | ✅ | archive/ | a2c 泛型单态化 |
| 066 | incremental-transpilation | ✅（2025-02-01） | archive/ | a2c/a2r 接入 Database，Universe 双轨过渡（ADR-04） |
| 067 | strengthen-rust-transpiler | planning | archive/ | a2r 对齐 a2c 的差距分析 |
| 083 | a2r-transpiler-with-rsat | partial | archive/ | `.rs.at` 平台专属实现文件与 `#[rs]` 注解 |
| 100 | a2js-to-a2ts | partial（Phase 2/3 完成 2026-03-01） | archive/ | 默认生成 TS，javascript.rs 保留（ADR-02） |
| 152 | a2ts-typescript-transpilation-design | ✅ | archive/ | a2ts 功能对齐 + 四模块拆分（ADR-03） |
| 161 | a2r-list-implementation | ✅ | archive/ | a2r `List<T>` 与 `.as(Type)` cast |
| 162 | method-keyword-to | ⏳ | archive/ | `.to(Type)` 显式转换关键字（未实施） |
| 163 | a2r-core-struct-support | ✅ | archive/ | 静态方法/嵌套字段/枚举 tag 值/Option-Result/自定义属性 |
| 164 | a2r-ext-for-external-trait | ⏳ | archive/ | `ext Type for Trait` 外部 trait 实现（未实施） |
| 165 | a2r-struct-destructuring | ⏳ | archive/ | is 分支内结构解构（未实施） |
| 166 | a2r-generic-constraints | ⏳ | archive/ | `#[with(T as Trait)]` → `<T: Trait>`（未实施） |
| 167 | module-system | ✅ | archive/ | 模块系统；MultiSink 多文件输出 |
| 168 | shared-variable | ✅ | archive/ | shared 变量 + pub 迁移；escape_str 转义统一 |
| 170 | a2r-test-reorganization | ✅ | archive/ | a2r 测试分类化目录结构 |
| 171 | a2c-test-reorganization | ✅ | archive/ | a2c 测试分类化 |
| 172 | a2ts-test-reorganization | ✅ | archive/ | a2ts 测试分类化 |
| 173 | r2a-rust-to-auto-transpiler | ✅ | archive/ | 基于 syn 的逆翻译（ADR-08） |
| 204 | a2r-transpiler-completeness | ✅ | archive/ | Result/spec/struct/enum/stdlib 映射/安全输出六阶段 |
| 213 | a2py-maturation | ✅ | archive/ | a2p 覆盖率 18%→80%+ |
| 215 | a2ts-maturation | ✅ | archive/ | a2ts 扩至 80+ 测试 |
| 219 | playground-source-map | ✅ | archive/ | Sink source map 全目标打通 |
| 220 | a2r-transpiler-improvement | ✅ | archive/ | 类型映射/枚举/stdlib 覆盖改进 |
| 223 | a2r-step00-transpiler-fixes | ✅ | archive/ | lexer pos drift、多参枚举变体、is 表达式修复 |
| 229 | self-hosting-via-a2r | ✅（Phase 4） | archive/ | a2r 作为自举落地路径 |
| 232 | a2r-lexer-compilation | ✅ | archive/ | `.sub()`/`.slice()` 映射 + post_process() 类型修正 |
| 240 | rust-cookbook-a2r-tests | ✅ | archive/ | 163 个 cookbook .at 全 assert 化，124/124 通过 |
| 241 | a2r-string-type-cleanup | ✅ | archive/ | get_or/insert 的 .to_string() 启发式修正 |
| 242 | a2r-feature-gap-tracker | active（living doc） | plans/ | a2r 遗留缺口清单（活文档，持续更新） |
| 263 | transpiler-tests | ✅ | archive/ | 约定式测试发现 tests/a2*_tests.at（ADR-05） |
| 264 | a2r-dot-to-double-colon | ✅ | archive/ | 模块路径 `.` → `::` 映射 |
| 266 | vm-a2r-conformance | partial（Phase 1） | archive/ | AutoVM 与 a2r 语义一致性 |
| 271 | remove-a2r-examples | ✅ | archive/ | 清理 a2r 预期文件中的 example 声明 |
| 283 | a2py-maturation-plan | ✅ | archive/ | a2p import 系统/类型跟踪/PyDep 依赖收集 |
| 290 | a2gd-transpiler | ✅ | archive/ | GDScript 后端初建，9/9 测试 |
| 305 | a2gd-maturation | ✅ | archive/ | a2gd 对齐 a2py 功能面 |
| 310 | auto-ownership-escape-analysis | ✅（2026-06-16） | archive/ | 逃逸分析 own-by-default + 保守回退（ADR-06） |
| 328 | a2r-http-server-architecture | 已归档（原"设计完成待实施"） | archive/ | `#[api]` → Axum 原生 server 转译设计 |
| 355 | a2r-async-await-transpilation | 已归档（原"设计文档/TODO"） | archive/ | a2r async/await 转译设计（从 Plan 344 拆出） |
| 364 | a2r-cosmic-replication-readiness | 已归档（原 ⏳） | archive/ | 为 COSMIC 桌面复制补齐 a2r 缺口 |

备注：

- **355 重编号冲突**：`plans/355-a2r-async-await-transpilation.md`（本表所引）与
  `plans/archive/355-fix-persistent-session-fn-body-recursion.md`（auto-shell session 修复，与 trans 无关）
  同号并存；引用 355 必须带 slug。
- 242 是活文档，其状态由自身维护；其指向的子项完成度以 plan 文件内表格为准。

## 2026-08 增补（Plan 471）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 400 | api-gen-a2r-body-transpilation | 已归档（原 🟡 Phase 1+2） | archive/ | api_gen .at body 直转 Rust（is_thin_delegation/try_transpile_body + AUTO_A2R_BODY） |
| 415 | a2r-remaining-big-items | 📋/🟡 | plans/ | a2r 剩余大项（242 tracker 收口线） |
| 417 | a2r-parity-debt 持续清偿线 | ✅（E1-E4 系列） | archive/ | string_utils/serde_json 等 parity 断裂根修（D2 导入签名/E3 if 尾值/427 str_param_borrow） |
| 442 | cross-platform-closure（转译侧） | 已归档（原 🟡，2026-09-07 复核） | archive/ | musk 五域接线；vue.rs SVG 静态字面量直通 |
| 555 | script-mode-w1-dispatch-foundation（trans 侧） | ✅（reviewed→archived） | archive/555-script-mode-w1-dispatch-foundation.md | s2s 改写器骨架 auto_s2s.rs：LoweringRule{id:A1..F4} 规则表 W1 空置/token 粒度单遍首中/identity 逐字节发射/语法验证兜底，三单测（幂等+注入即生效+非法源拒绝）；CLI trans auto 子命令+--dump-lowered；W2 链式语义与 AST 发射器待裁定（P555-D2） |
| 567 | script-mode-w25-tail-w3-oracle（trans 侧） | ✅（reviewed→archived） | archive/567-script-mode-w25-tail-w3-oracle.md | s2s E1 隐式传播规则（may 化+.?·无 use.py 零改写·幂等）+W3 注解预言机（get_type_hints 注册期内省→PY_RETURN_ANNOTATIONS 表→py_return_types 灌注+W0010 lint）+emit if-true 块包装（E0007/尾块歧义免疫）+a2py may 家族/with-as 回译/py_int 反映射+parity runner PYTHONPATH 注入；账本 P567-1..6；债 P567-R1 |
| 560 | script-mode-w2-sugar-batch（trans 侧） | ✅（reviewed→archived） | archive/560-script-mode-w2-sugar-batch.md | s2s AST 帧形+链式规则落地（P555-D2 销号）：emit.rs 源发射器（Op/Type 符号表+Str 再转义+幂等纪律）+py_known 静态分析+s2s_rules A/B/C 族（py-known 门控设计修正·步长嵌套 Range·词法双星惯例教训）；管线激活 .as=lower→compile；19 py 套件 .as 载体；契约 design/s2s-lowering.md；账本 P560-1..6 |
| 569 | py-ret-dynamic-dispatch（trans 侧） | ✅（reviewed→archived） | archive/569-py-ret-dynamic-dispatch.md | s2s A1 `.len()` 特判——py-known 接收者方法糖 .len() 改发 obj_len（原 py_call(recv,"len") 在 py 侧恒 AttributeError，py 无 .len 方法）；与 codegen 侧表路径（.at 直跑）殊途同归，.as/.at 双模式同值；账本 P569-2/P569-4 |
| 577 | emitter-gaps-batch | ✅（reviewed→archived） | archive/577-emitter-gaps-batch.md | 发射器缺口批清偿 auto-down DEBTS 016/022/转介 052⑤：a2r Phase 0 五小修（空 map {}→HashMap::new/map 下标 recv_is_map 容器分派 m[&k]+.insert/enum derive Hash（eq_safe 同门）/​.length 属性形态 (x.len() as i64) 对齐方法形态（expr_is_len_call 扩字段形态防 .to(int) 双 cast）/保留字表补 Rust 2024 reserved 12 词含 final——台账红利 known-broken 026 销行）+a2ts T1-T4（多 payload enum 工厂 rest-tuple 参数+裸单位变体引用补调用/is 的 else 臂前缀按分支类型后写/OptionPattern 真实判型 Some(v)→!==null/struct_names 全位 new+const enum→普通 enum）；R1/R4 台账形态经 433A1/447/019 过渡清偿（探针降级回归锁，代码为准）；directed 探针双门（26_plan577 语料四例+rustc 实编门 / a2ts_directed_probes+tsc 驱动）；auto-down 侧双 gen.mjs B1/B2 断言式后修退役（TS 产品字节中性）；Phase 4 crate 试点硬前置就绪；账本 P577-1..7 |
| 597 | cffi-engine-face（trans 侧） | ✅（reviewed→archived） | archive/597-cffi-engine-face.md | a2c 引擎面首证：未初始化固定数组裸声明发射（原 `= NULL` 非法 C，真 MSVC 抓出文本快照盲区）+12 符号链接驱动先例（MSVC 拒 DLL 直接输入 LNK1107→链 cdylib 副产 .dll.lib+DLL 同目录分发，build-engine-face-a2c.cmd）；快照 18_c_interop/004_engine_face（12 符号 fn.c+驱动流，engine_abi.h 语料内承接） |
| 610 | a2r-c-abi-two-faces | ✅（reviewed→archived） | archive/610-a2r-c-abi-two-faces.md | C ABI 双面（004 §5⑤⑥ 清账）：⑤#[export]/#[export(system)] 兄弟包装模块导出发射（no_mangle extern，符号=fn 名调用方零改动；int i64↔i32 边界 cast/cstr CString 边界/句柄空安全解引用 &mut+缓冲双形参规则；auto_cabi_kit 生成模块收口全部 unsafe）+⑥use.c 双形态下降（manifest 共享 IR 增 link 字段：S 静态 #[link] extern/S 变 libloading env→exe同目录→PATH；source_dir 解析管线三入口）；语料 27_c_abi 五件+三道 #[ignore] 实编门（AC-02 dlopen/AC-03 12 符号 Auto 版 cdylib×597 驱动器 dumpbin 12/12 恒等+CFACE_OK/AC-04/05/06 三腿——AC-05 同驱动产物改链 ⑤ 产物=Auto↔Auto 零手写胶水闭环）；trampoline 选型 A 证伪/A' 具名导出 fn 选定 defer；账本 P610-1..6 |
