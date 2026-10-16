# package-resolver — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 800 | man-dependency-model-extraction（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-800 建立：已解析依赖视图与模块路径解析核心（std-only 零第三方依赖，闭包由 test/fixtures/modularity/package-resolver-consumer 外部探针 + verify-deps.sh 钉住并登记 module-boundaries man-ecosystem 域）——DependencyEntry<M> 不透明模式标签 + PackageResolverCore<M> 单一权威 map；find_package_path/find_module_in_dep/resolve/exists/module_search_paths 规则与错误文本自 auto-man resolver.rs 逐字冻结搬迁（resolve 无盘臂/exists 无 package.at 臂/歧义不得成功；b0d6d4aab 探针冻结面 docs/plans/attachments/800/t01-freeze.md）；auto-man 转语言 adapter（ExecutionMode/pac.at 行扫描/ModuleResolver trait/Pac-Super-None 委托留此，Dependency=核心泛型条目别名单一 map，路径算法净删）；R1 修复：错误映射描述对齐真实边界（legacy 调用点 if let Ok 静默忽略、无 adapter 映射）+ 新文件 rustfmt |
