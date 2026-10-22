# PLAN-815 T09 R2 已审规范增量（merge输入）

final pass，Plan r3；reviewed HEAD ca9e89b44fea32c923297d943473191f435e3a37，code0f2858e35cb220964d8c23c8d77afce6839db491。规范正文基准为本目录frozen-handoff.md的SD01..07，以下审定其当前目标和约束解释；来源版本hash见baseline，终核七目标无漂移。未直接写canonical/live ledger。

| ID | 目标 / 已审持久规则（before→after） |
|---|---|
| SD01 | **docs/specs/auto-ui-tools/project.md**。由A窄元数据端点/资源登记→B完整UI工程工具owner：生成/build/run和资产在auto-ui-tools，normal/build/dev无auto-man或旧auto-lang大库反向边；旧门面只转发。所有新协议工作入口消费单源已求值请求，结果/失败/真实布局诚实；公开shell再生器也按canonical双直依赖生成，可被独立消费。 |
| SD02 | **docs/specs/tool-protocol/project.md**。A的v1、4字节大端framing、8MiB帧预算、wire120/信封122深度及同源数据owner保留；B补五操作、单飞、progress/日志控制分离、有界输入、input/cancel回执与run/build终态双通道、owned树/reader清理及真实退出码。取消回执不等于工作已结束，不把捕获内部限界当所有构建/运行SLA。 |
| SD03 | **docs/specs/auto-man/project.md**。UI生产算法/资源退出Man，薄门面保旧公开签名和legacy结果语义；通用api-gen/sidecar效果/共享workspace算法单源归auto-api-gen（lang-service-adapter），不复制另一份算法。协议严格失败不被旧提示性Ok吞掉，兼容无请求路径保持明确边界。 |
| SD04 | **docs/specs/project-model/project.md**。生产UI请求消费809唯一类型/codec；动态求值在显式上层adapter，静态准入不扩；主声明快照在盘文件改变/移除后仍权威，字段/相对根/载荷以已审field-profile-matrix为依据。arkts在边界归一为ark，raw具名载荷不静默丢失；必要跨文件源读取与主声明二次求值区分。 |
| SD05 | **docs/specs/auto-lang/ui/overview.md**。工具/资源消费814已发布compiler/core/runtime SDK，真实外部命令在托管形态使用owned设施；canonical生成组件和shell库采用auto_ui_runtime/auto_lang_core，生成清单双直依赖。既有app壳层有限compat限定路径若仍保auto-lang直依赖按已披露兼容边界表达，不将仅shell已零legacy外推为所有模板零legacy。 |
| SD06 | **docs/specs/repository-boundaries/generated-ui-distribution.md**。原792受管bundle合同扩至tool/protocol/source/embed/producer闭包26项及根输入，fresh/locked/重定位、资源/失败保留恢复按已审范围。源码包由包内locked构建工具；源提交+manifest目录digest/producer/锁等绑定source，实际二进制SHA256是有provenance的参考artifact，不承诺重建相同字节。当前code0f2858e35/docs ca9、新参考a73b117f…；T10按最终实际冻结/合入身份发布收据。逐文件blake3是目录digest计算输入，manifest不是逐源文件hash展开表。 |
| SD07 | **docs/specs/auto-ui-runtime/project.md**。新增request_host_exit/host_exit_requested及host_exit_window_begin/end公开退出窗口规则：代计数、窗口内新请求、初始化期请求不丢、跨会话旧请求不误杀；旧CLI形态保持边界。VM可重入的真实运行边界为每会话独立子进程，主题/外部后端上下文精确按请求替换，必需后端装载失败具名失败。 |

SD01/02最初标add，但A阶段文件已经存在，B合入须对现存规范受管增补并保留A交付来源，不能覆盖或重造已发布历史。其他五项modify，目标及GOAL-003/GOAL-007和主Plan元数据一致；修复未改变合同，不增revision。

冻结证据：shell源/Cargo原生逐字等于入库、原生消费及locked均0、freshness1/1；新release四组消费4/4；R1未变AC/SD与原33/33/377资源/生命周期/字段闭合记录按明确scope复用。canonical沉淀与ledger派生归T11，不在review写入。

最终审定：AC01..05通过，T09完成。AC06及T10/T11 merge-owned门仍pending，reviewed不代表已合入/交付。门禁修复改变实现/依赖/输入时按影响重审；禁止用本次pass覆盖未来源改动。
