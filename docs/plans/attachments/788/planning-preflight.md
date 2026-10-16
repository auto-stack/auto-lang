# PLAN-788 r1 起草预检

日期2026-10-16。stage:new；outcome:pass（规划契约）；next:work，T-06前核D-787。当前只交付计划，未创建实施树、未运行实现测试。

## 来源与授权

用户要求基于此前讨论规划787的下一计划并解释内容。采用auto-plan-new及verification-contract，读取Spec/实际decoder/runner、ac-core前端/执行器、783 driver与787状态。主检出调查基面67b44e0409fb11ef45b8d01b0ed17e2cc0ec3c05；787树当时8b1920c33、executing r1/4步。来源SHA-256见planning-inputs.tsv，work重新核实际基面/设计/依赖。

按主检出scripts/new-plan.sh取得788，.next-id→789。调用处同时持Local\AutoLangPlanAllocation mutex和docs/plans/.allocation.lock排他句柄，重核active/archive最大号、分配后唯一性。首次沙箱内Git Bash初始化因系统对象权限失败，未写计划；自动权限复核允许后成功运行原脚本，未绕过取号或碰Git工作树。

本轮仅规划/调查与保存文档，不实施、不联系其他聊天、不创建代理/树。资源预算、1s性能门与样本是拟议契约，非已有测试结果；不记录不存在的用户数值预算/模型分工。其它计划/债务WIP保留。

## 定界与预检

- Design35 B为下一紧邻工作包；Design39–42的cache、meta/CT、ABI和周转要求作为接缝与观测依据，不把整套语言实现塞进测试计划。
- 原生支持core/scalar，syntax支持787两grammar；不冒称text/record/native全导入、完整新前端→native或完整meta/AVM/FFI交付。
- 787不是协议/原native任务的硬前置，syntax实际接入必须等独立review/门禁/合入/Spec后绑定；不复制在途实现，不跳过T-06。
- v2新支持表明确，v1接受/拒绝集合保持；不伪装当前协议已支持native/metrics。source HIR输入独立保留，负例不只从正常source生成。
- 协议核心不依赖backend/UI，worker独立，auto-lang只编排/比较，ac-core不反向依赖产品crate。
- 已核实Diagnostic与compile_parts、vertical及真实executor；其String错误/有损采集/无输出cap是需弥补的实际缺口，不假定现有接口足够。T-01 whole-path先验证、T-03窄扩展同一生命周期，禁止新复制编译器/执行器。
- 新旧负例区分程序exit70、typed语言诊断、工具/timeout/panic等Infrastructure；坏actual+好expected必须红，金样FAIL不阻断有效actual产品交接。
- 原始bytes、selector零副作用、路径/reparse/nonce/manifest、截止/输出上限/全树回收均有强反控。
- fresh/reuse/read-only不缓存PASS；toolchain以真实输入而非调用方常量证明新鲜，原生对象命中仍真link/run，HIR命中重verify。
- 同进程热点用有限会话，新进程重启真spawn；会话与每请求预算并存。参考性能含materialize/IPC/等待/发布，不偷扣启动或丢掉超时样本。
- 每阶段trace/计数/资源、原生hot/body编辑与syntax局部edit的P95<1000ms为可检查验收。六组30样本及持续50revision分场景，功能/性能状态分列；超标不删AC或扩大预算。
- ≥24语义case是导入最低规模，不代替V-01–15补充矩阵，旧test/gate不退役；未来类型/CT阶段标未实现，不编造0ms。
- 0–10章节、10任务/0完成、AC-01–09、SD-01–06与V-01–15均有owner/入口/预期与边界。未来入口明确由T-01–08物化。
- work定向验收，review独立同树绑定commit，merge统一根t/tv/tt及独立触面closeout；tf按批量到期单实例。代码变化后重验受影响证据。
- 本轮不编辑canonical Spec；SD由T-08准备/T-10沉淀；唯一预定lang-788树尚未创建。

起草验证仅含编号唯一、章节/任务计数、链接、来源存在、范围/AC追踪及diff空白检查；未运行cargo或性能测试，不宣称功能/亚秒已达标。
