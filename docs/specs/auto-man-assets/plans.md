# auto-man-assets — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 803 | man-template-assets-extraction（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-803 建立：Man 模板/构建器资源单一来源 + 纯脚手架路径规划（rust-embed+std，闭包由 test/fixtures/modularity/man-assets-consumer 外部探针 + verify-deps.sh 钉住并登记 module-boundaries man-ecosystem 域/资产根 man-template-assets）——templates 五族/builders-ghs/builders-iar 三组 20 资源自 crates/auto-man/assets git mv 逐字迁入（SHA-256 冻结=基线 1dd123361a 旧臂实测）；plan_template_family 余段字面 name 替换/未知族空/零输出 IO（调试态 rust-embed 动态读源码资源目录、发行态内嵌，反控 mode-boundary-probe.py）；auto-man asset.rs 兼容门面真委托（copy 旧臂逐文件求值/物化/失败阶段逐字保持，RustEmbed trait impl 具体返回 Filenames 保跨 crate 类型面，inherent+trait 双形态 get/iter/compressed 转发）；R1 修复：API 具体类型恢复 + 编译正控 + rustfmt + 构建模式边界措辞；R2 修复：方法级 refining_impl_trait 局部 allow |
