# SD-01..06 核定

冻结 r3/a030427fe。已读主检出现行 overview、HIR stage-contract、syntax snapshot-cst、semantics query-cache、AV cache-and-conformance、test-case architecture、Design40/43，并对照实际 producer/consumer；版本哈希见 baseline.json。SD01/03/04 三个 add 及 SD02/05/06 四个 modify 路径与 frontmatter 一致，GOAL-002 真实存在。无 retire，无跨仓共享语义改写。

规范增量在本次审核范围通过，尚未沉淀。精确待沉淀文本是 spec-delta-reviewed.md，原冻结文本另保留；审校仅明确冷编与 resolve_env 的组合、区分单 body 投影和允许的相邻词法重扫、分清 session/环境两层基面守卫，并补一个 Markdown 链接，不改验收或语义。原 delta 中若保留“恰一重扫”应理解为冻结测试输入，不能升级为任意编辑只扫描一声明的公共承诺。

SD01 真单 body/候选/上下文/内容身份与产物依赖；SD02 完整输入缺 body 拒绝与单函数脚手架边界、严格 CST 投影；SD03 单函数 lower 与局部表/稳定 imports；SD04 无 merged 单元派发和三入口/原子安装/生命周期；SD05 双门、双线预算、manifest 实际 required 集合与有界 HIR 根；SD06 正式 persistent worker/judge、完整报告与 one-shot 兼容，均与当前代码及测试断言一致。

merge 必须保留 798 既有六场景、原域支持面及有限 profile 边界，追加 828 规则；不得用 828 单一战役替换 798 历史合同。successor 817/818、820–827 是真实接缝交接，不把后继能力实现计作 828 已完成，也不反向等待其实现。canonical/ledger 本轮未改。
