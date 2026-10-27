# PLAN-828 第三批 R2 → work：仅C03/C04残项，一批修齐

基线f4d7a9da6fb6b58a15a3149ee86583a7555daa86；同树D:/autostack/.wt/lang-828/auto-lang。先读本目录review.md及原r3合同。C01/C02/C05原具体错误已关闭，root机制、候选模型、单元派发不要重做。第三批仍needs_fix，暂不T-10。

**C03/P1：先校正诊断，再补真正消费证据。** 上轮copy例现在红，是“新内容必须换对象”间接抓到旧记录，不代表实际消费绑定已完成。审核将run里唯一AV调用真实换成副本执行，新观测未动，正式gate仍绿。源与重放在generate-consumer-copy.py / consumer-copy-controls.rs。另两个兄弟出口：ready.functions=3但身份清单漏掉main仍绿；原正式worker对合法`7→7 `内容命中反而红（真实计数reuse_hit:helper=1，judge错读reuse_hit全局键）。

先写一行数据流：真实AV payload获取→本次执行观测（对象/代际/内容）→本次请求/快照/结果publication→父侧判决。选择合适接缝，不把“先记录current、再执行”当不可脱钩证据，不额外执行guest来刷新记录，不发明恒真布尔值/缺项默认通过。修正当前内容hit的合法语义，完整目录/必要观测缺失应拒PASS。不要只修reuse键；全部三个出口同批闭合。已有普通/step/resume共享观测若改变，补代表性兄弟控制；不是重做核心派发。

**C04/P2：失败路径与阶段观测都要补齐。** 当前成功report/raw确已落盘、计时端点已后置，保留；但是正式战役在write_evidence前assert成功，值错时30轮已跑完却没有report/raw。evidence-failure-controls保留原测试函数体及实际wrongvalue worker输入。调整为先留本次成功/失败证据，再判总门/退出；异常中断/回包也应保留已有样本与失败上下文，别误用上次成功文件。两个总时长加阶段次数仍不是原§5.5各阶段wall/IO；按正式session真实生产入口补所需观测、说明零IO口径。可复用现有Obs或等价轻量机制，不需通用平台/漂亮报告。

将本目录原输入/故障/oracle收编为永久或稳定可重放控制；API变动只做机械适配并记录，不能降低oracle。先完成两组所有修复，再跑定向：原6judge/2root保持、consumer-copy应红/原run绿、官方当前内容hit绿、缺main观测红、历史回访绿、失败report/raw实存且内容正确。正式最终代码跑30轮一次（含身份/全部父侧核验与证据/同798双线预算），保留raw及输入/构建身份；512证据仅在被测代码路径/依赖/输入未变时可明确复用，否则最终冻结上定向补跑一次，不反复跑旧两个512或整档。

修正相关interface/coverage/SD05/06的完成措辞，coverage笔误dispatched_identities应指实际符号。给C03/C04各一行“根因→受影响出口→机制→正反控→余界”，提交含所有实际修改的新冻结根，再整批受影响面复审。维持r3/executing0/11、T-10及merge-owned门pending；不代发消息/新建chat/合入。
