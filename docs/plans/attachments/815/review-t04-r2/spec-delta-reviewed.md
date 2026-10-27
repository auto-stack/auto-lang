# T-04 R2 规范增量核验

Plan 815 r3，受审 dbd295f050c05ef37450adbd0f8f835a150e077f。SD-01..06 原提案、canonical 输入及合同 SHA256 见 baseline.json.txt；继承 R1 对十项偏差的范围裁量，不扩大阶段或重写已发布 A/L 事实。

- SD-01/03/05：旧 Man 六个运行入口和 auto-lang 根三个 store-deps/注释函数路径已恢复，纯公开转发，无算法复制；编译器新 owner 的测试依赖已补。运行主体仍按 M814-B 门在 T-05 继续。
- SD-06：真实源码闭包为 25 项；auto-project-model/auto-toolchain-core 已补，按实际投影剥除 auto-test-case/autodown-core。独立 A/B metadata、漏项红与恢复绿通过；工作侧 792 全程 25/25 仅按明确内容身份复用，不将本轮 metadata 说成再次 cold build。
- SD-02/04：没有改变协议/能力/请求准入规则；新 canonical B 仍未发布。本轮重放相关窄工具面，完整交互/取消留在原阶段。
- 资产生产唯一来源、旧 oracle 与锁文件消费路径已接线；金样改为非编译资源并继续逐字比较真实生成器。

本轮仍 needs_fix，原因仅为 F-815-T04-R1-06 格式不变量未闭合；其余五项独立关闭。最终完整 B 的 T-08/T-09 规范与验收依旧 pending，需准确涵盖 auto-ui-compiler/auto-api-gen owner/路径。既有 spec-impact 元数据及 GOAL-003/007 保留，不写 canonical/live ledger。
