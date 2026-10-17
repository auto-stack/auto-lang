# auto-toolchain-core — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 805 | toolchain-rules-core-extraction（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-805 建立：Man 工具链规则单一来源——config（CompilerKind 八类/ExecutableType/FlagFormat/FlagMappings/CompilerLocation/CompilerConfig 及八类默认表）/mapper（FlagMapper，map_output 尾缀语义）/templates（四命令内联模板+长度降序一轮顺序渲染）自 auto-man builder/ninja 三模块迁入（迁移提交与 base 282e50053 Git blob 逐字节相等，R1 修复轮 rustfmt 后以 16 件 golden 行为对拍为证；闭包=auto-val+std，由 test/fixtures/modularity/toolchain-core-consumer 外部探针+verify-deps.sh 钉住并登记 module-boundaries man-ecosystem 域）；auto-man 旧三模块薄 re-export 门面（同类型身份，CompilerStore am.at 加载/Resolver PATH 磁盘进程观测/NinjaBuilder 执行留上层；CompilerStore 预存空载缺口登记 P805-D1）；R1 修复：迁入 fmt+新测试零告警+render 语义表述改准（后 key 展开先插值/不回扫/等长 key 继承 HashMap）+SD 真实模块表；不进 792 交付闭包（反向依赖仅 auto-man） |
