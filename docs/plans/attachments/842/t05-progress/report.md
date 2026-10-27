# PLAN-842 T-05 验证

T-05 的版本迁移及指定真实升级/恢复控制完成；后续 T-06..12、最终源码的 T-07 复验和独立 review/merge 仍未完成。

- lang `aeb39d4a13fb6ad39c0454ff2ed764d2f2e246f1` 将 auto-lang 升至 0.2.0；UI `1ef875b574bad75b4de07232c0103cc63e213bd7` 将 auto-ui-dialect/runtime 升至 0.2.0；语言/UI/Man 锁已更新。可复跑外部消费者逐一证明三个 `=0.1.0` 约束拒绝 0.2.0 源、三个 `=0.2.0` 均实际编译成功；原旧源保持在历史固定 Git 提交。七份本计划生成的消费者锁已离线刷新并仍未跟踪保留。
- Man `484d9d2fe0904f13c9aff29ebbebea564bf8eac0` 的安装器在构建/准备前校验旧安装：真实旧 Core 的未知 schema、未知 profile、坏内容分别具名拒绝，原字节不变；合法旧安装可进入失败注入并恢复。首次真实升级因 Core 锁缺新增 sha2 失败，旧安装未替换；补锁后真正升级到当前版本，旧备份保留，升级版和从备份复制恢复的旧版均 verify 并实际运行 `probe.at` 成功。
- 交付 verifier 与再交付拒绝未来 schema 并保护原产物；未来版本的带事务标记备份不得被旧版 GC 删除。真实生成/未知版本/恢复、外来目录/未来备份和未完成备份控制均通过。UI codec 9/9（含旧/未来 schema、domain/tag）、旧缓存失效、重建和 reload 1/1 通过。
- 真实旧 Full 安装升级已通过，旧备份保留；从备份恢复的旧安装与新安装均 verify，旧版实际生成交付包后由新版原位再交付，三个 crate 从 0.1.0 升至 0.2.0 并 verify。未来 schema 的 verifier/生成器拒绝且清单与 main 字节保留；恢复后再次 verify。真实升级交付包 fresh lock 和 locked build 通过（2m26s），实际生成宿主 default/显式覆盖 probe 1/1；Full 换装中断注入后 manifest、两个 executable、SDK lock 四文件 SHA 不变且完整 verify 通过。后续最终版本 T-07 仍需完整 Core/Full/consumer 复测，review 与 merge 门 pending。

- 核查 Core 正常、构建、dev 依赖闭包无 UI、auto-lang facade 或 shell-pack。T-05 结束提交 lang `aeb39d4a13fb6ad39c0454ff2ed764d2f2e246f1`、UI `1ef875b574bad75b4de07232c0103cc63e213bd7`、Man `484d9d2fe0904f13c9aff29ebbebea564bf8eac0`；历史备份及恢复副本仍保留在同组工作目录。
