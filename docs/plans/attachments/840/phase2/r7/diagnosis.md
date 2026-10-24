### r7 / Phase 2 资源分档绑定补齐（原计划内继续修复）

r6完整冻结门已过并以ede7f5a90线性落master，ledger80已核；原图形8b62交付保持。主检出batch-r2 tf完整6288项：6287过/1红，未取消尾项；红为ffi_dual_019的wide布局值683604832768≠5000000000。匹配REG-2既有共享fixture竞争观察。查证daily已有ffi_dual→ffi-dual串行绑定；full/t3只有max-threads=1组声明，缺该绑定，明确配置缺口。尚不声称生产ABI根因已修。

同一Phase2的T-13/14/15重开：仅补.config/nextest-full.toml与nextest-t3.toml的既有整族绑定，保持输入/布局/features/expected值/选择面；SD-P2-02补充“声明组须实际绑定、config-file整体替换”的规则，不改生产pack/cache/ABI。保持Phase3Init/RQHost修复与全部支持面；不新建Plan。

V14追加full/t3实际test-group成员核验、删除绑定负控与整族ffi_dual定向执行；当前batch-r2 TT/TB继续完成，期间不另启构建/测试。修复提交/受影响review pass后，bare t及已完成TT/TB仅在其实际代码/依赖/默认配置/输入逐项未变时复用（新改的full/t3非其输入），不机械重跑无关全档。受影响tf必须在新冻结落地主检出完整重跑；t3只按本次FFI资源scope定向验证，不运行aavm/塔全档（无触发文件改动）。

当前executing17/22，T-10/T-16最终批量收据/归档清理未关闭。完整旧红及其余legs原样保留，禁止以“已知flake”标签当作新tf通过。工作树为原lang-840，仍保留以完成同计划修复。
