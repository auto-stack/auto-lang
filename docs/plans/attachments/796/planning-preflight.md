# PLAN-796 r1 起草预检

2026-10-16；stage:new；outcome:pass（规划契约）；next:work，T-06前核D-788。非实施/独立验收通过。

用户授权两计划草拟及并行/cache/AutoDown/亚秒要求，本轮只读调查、排他取号与写计划附件，不创建树/代理、不联系其他聊天。主检出调查9c773601abfdb25b80c08d06922d0ad50edc653a；来源SHA-256见planning-inputs.tsv，实际在途进展继续变化，work须重新绑定。

## 预检结论

- 0–10章节，frontmatter drafting r1/0步/9任务，AC-01–08、V-01–09和SD-01–08分别有工作/审查/收尾owner。
- 共同写面与依赖见两计划parallel-contract；796是语义Schema/probe公共接缝唯一owner，不需要797完成才交付，无循环依赖。
- T-01–05独立可做；正式AutoDown T-06需要788真正合入。源/oracle从首例存在，早期Rust真实执行与runner pending分列，不伪称已支持未知adapter。
- probe/run会改变v2支持表，SD-08明确decoder/报告能力版本与既有行为兼容；不修改788在途文件，不写第三套.ad parser或任意shell runner。
- 只建新独立workspace；不修改旧TypeStore/Codegen/infer、787/783底层或793–795生产热点。公共注册/manifest/Spec短窗口串行。
- 完整meta覆盖矩阵与实际有限i32/bool推导/描述codec分开；给797的u32/u64/f64等只是元信息表示，不假称新语言已能编译；泛型/CT/runtime接缝模型不假报AVM执行。
- 首例用真实统一store，不另造meta cache；producer自有版本化Schema与内层校验，不假称783 Future层自己完成；bad address/owner/ref与负依赖有反控。
- 32MiB分页、16MiB受管、64MiB私有增量目标、4096深、512churn、查询步数/时间/IPC/输出预算为拟议实验负载/策略，不是用户指定或语言永久上限。
- 六场景30样本、局部请求→当前正确推导/诊断P95<1s，阶段trace从首例保留；正式adapter接入后重测，不能丢失败/提高阈值或把功能pass当达标。
- T-01有whole-path门槛，尚待Schema/求解/页容器/注册细节只限既定契约内选择，不可行回规划；后续任务不能跳过。
- 最终review同树独立、绑定commit，work定向测试，merge-owned根/独立门禁明确；本轮不跑cargo/docs_gen，也不提前写canonical Spec。

取号遵守scripts/new-plan.sh；排他mutex+allocation.lock覆盖两次取号，扫描active/archive及.next-id、一号一文件；796/797前已有793–795保留。起草仅核来源、相对链接、编号、任务/AC/SD覆盖和diff空白，所有新命令都是后续物化入口。
