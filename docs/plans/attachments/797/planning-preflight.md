# PLAN-797 r1 起草预检

2026-10-16；stage:new；outcome:pass（规划契约）；next:work，T-06前核D-796/D-788。非ABI发布/实施验收。

本轮授权/取号/基面与来源同796：用户明确要求两计划草拟；调查9c773601abfdb25b80c08d06922d0ad50edc653a，逐文件hash见planning-inputs.tsv。未改其它计划WIP、未创建树/代理、未安装工具/执行native。

## 预检结论

- 0–10章节，drafting r1/0步/9任务，AC-01–08/V-01–11/SD-01–05均有owner和入口/强oracle。
- 首版是Windows x64有限候选profile；版本/布局/status/out/context/ownership/错误/callback均有范围，未知形态拒绝。最终review/merge前不发布“ABI v0.1已冻结”。
- 独立AutoABI声明→Cranelift模块与独立C前端三腿，明确不经Auto源码parser；当前内部ABI-A–E不改、不冒称C importer/Rust shim/loader交付。
- C工具实际可用性归T-01；缺失是infra blocker，不能Rust repr C替代或自动安装。C header/layout和非对称语义oracle独立审定，不由同一错误生成器自产预期。
- 资源要求是真分配/账本/原方释放/失败旧out不变/同步callback，不是光改计数模型；raw指针仍有caller有效性合同，不宣称沙箱。
- 796是语义ref/公共probe adapter唯一owner，797早期只用有hash快照，T-06后切真实库并退出临时模型；不复制持久TypeStore。
- 前五项可与787/788/796及拆分路线并行；最终T-06要等796/788交付，不能用早期.ad存在代替实际runner执行。796不等797，无循环依赖。
- 不改787/783底层、788在途协议或793–795生产热点；recipe/manifest/Spec单写窗口，公共支持表只追加本recipe不重写decoder。
- cache首例进入layout/对象，完整type/layout/body/工具链/include依赖入key，禁止复用PASS；命中producer计数/禁调反控和真实link/run并存。
- 阶段trace与六场景30样本从首例落实；两条硬路径P95<1s，阶段/端到端不混、raw失败不删；全冷C准备单列，用户实际等待不扣。
- 请求时间/输出/IPC/产物预算、cache16MiB受管及512版本是拟议实验策略；T-01细节冻结不能升预算换绿，深图资源拒绝与不兼容分开。
- 最终独立review同树绑定commit/工具链/796依赖，merge冻结提交门禁/Spec/guard收尾；本轮只文档结构/链接/覆盖/空白检查，不执行测试或写canonical Spec。

两份合同及parallel-contract需要共同阅读；所谓可并行是独立阶段和明确gate，不表示两个分支可同时修改公共协议、也不表示全计划能在依赖未满足时提前验收。
