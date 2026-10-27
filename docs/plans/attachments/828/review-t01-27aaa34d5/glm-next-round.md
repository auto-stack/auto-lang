# PLAN-828：T-02 生产 session 前端热路径换接

以 `plan-828-dev@27aaa34d565b11facab37008d42c6e509cc3e13d` 为已复审基面，实施树仍 `D:/autostack/.wt/lang-828/auto-lang`。先核path/branch/HEAD/clean，读取主检出 [Plan](D:/autostack/auto-lang/docs/plans/828-av-function-incremental-session.md) §§5.6–8和最新§9、[本轮PASS](review.md)及当前interface-decision。C01..C05与F05b已测范围闭合；无需再修已过模型。进入原T02，不另开Plan/树，不merge。

1. 把实际session的body编辑前端换接到既有受检环境/单声明投影与 `prepare_function_candidate`，移除热路径 synth 全Program/stubify→compile_program_parts。冷初始化与显式非快路径单列；热编辑只有changed body真实resolve/typecheck/plan/bind，接口目录/来源元数据读取单列。复用既有source规则及私有函数凭证入口，不手写HIR或绕过校验。
2. 按既有D03接线候选所有权：PreparedEdit拥有候选及其新源/来源/绑定元数据，prepare不改当前；可失败的验证/构造/链接先完成，再在成功提交点一致发布env/root/source/revision/来源。核owner/基面/closed，拒绝或丢弃不污染当前；session.close显式撤销新环境拥有引用。完整T06矩阵仍按原owner交付，但T02实际新增/替换的入口先做真实成功/拒绝首例，不能复用旧PASS跳过新路径。
3. 保留现有Scalar支持和已登记C01控制：合法非首位函数、递归/稳定caller、bool/短路/条件for/命名实参与intrinsic支持依原Plan及实际旧输入对照；真实source/CST增量投影接到生产消费。以冷编HIR/结果/诊断对照及producer窗口证明未改body工作为0、改函真正执行；同一窗强制旧全stub实际管线仍必须使结构门红。检查主session入口，不以仅直接调用接缝的测试替代接线证明。
4. T02保持前端范围；完整无merged派发、三入口单元绑定仍T05/06，A07生产/恢复仍T03/T07，资源/正式性能仍原任务，不把它们写成已完成或擅自延期。新环境/候选和旧snapshot等真实驻留仍计F09预算。按实际变更回放关闭9、publisher3、事务8、admission/contract以及相关scalar_projection和session定向控制；只运行定向检查与格式，不整档t/tv/tt/tb/tu/taa/tf/docs_gen。
5. 交接生产调用链/对象拥有关系、真实前后计数/身份、冷编对照、明确选择数及raw，提交完整SHA和clean/依赖后做T02受影响面复审。主检出写Plan簿记，实施和测试留原worktree；本轮起点仍executing0/11，不把本次局部PASS当整Plan reviewed/可merge。
