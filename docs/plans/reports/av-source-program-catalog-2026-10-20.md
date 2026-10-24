# AV真实源码程序目录与迁移验收种子

2026-10-20。衔接[能力矩阵](../../design/strategy/av-language-capability-matrix.md)、[路线](../../design/strategy/av-source-programs-and-avm-replacement-roadmap.md)、[827](../827-av-source-program-baseline.md)。

本轮只读取现存源文件及相邻expected，**没有运行旧AVM、新AV或native/a2r**；此表是23个固定种子，不是“23例已通过”，更不是完整替换分母。source SHA/原有oracle文本见[输入收据](../attachments/av-language-roadmap-20261020/program-inputs.json.txt)。AP-01/02/03/22/23属于现有AC源码域，其余为旧VM语料/应用；两者都要真实source入口，不能混成同一个已支持集合。

| ID | 原始程序 | 目标阶段/owner | 现有或拟强oracle | 必需依赖/边界 |
|---|---|---|---|---|
| AP-01 | [experimental/ac-core/fixtures/scalar-source/valid/p-gcd-euclid.at](../../../experimental/ac-core/fixtures/scalar-source/valid/p-gcd-euclid.at) | G1 / 827 | entry i32=0；gcd各组期望由源码显式检查 | AC scalar源码，非标准库脚本；本轮仅手审 |
| AP-02 | [experimental/ac-core/fixtures/record-source/valid/p01-records.at](../../../experimental/ac-core/fixtures/record-source/valid/p01-records.at) | G2a / R2-D | entry i32=0；p/q独立、mut/嵌套字段检查 | 旧native/AC域参照，不假定旧AVM所有语法适用 |
| AP-03 | [experimental/ac-core/fixtures/text-record-source/valid/p02-overlap.at](../../../experimental/ac-core/fixtures/text-record-source/valid/p02-overlap.at) | G2a / R2-D | entry i32=9（kind2+rebuilt长度7），另审owner/cleanup | 重叠clone/move/replace，资源事件由新AV独立oracle补齐 |
| AP-04 | [crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.at](../../../crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.at) | G2b / R2-L | stdout 3\n（现存expected） | 需要顶层、lambda、捕获、List.map、print |
| AP-05 | [crates/auto-lang/test/vm/09_functions/015_list_chain_pipeline/list_chain_pipeline.at](../../../crates/auto-lang/test/vm/09_functions/015_list_chain_pipeline/list_chain_pipeline.at) | G2b / R2-L | stdout 60\n5\n5\n（现存expected） | 不能只用MIR调用closure代替源码管线 |
| AP-06 | [crates/auto-lang/test/vm/16_option_result/022_result_multi_error/result_multi_error.at](../../../crates/auto-lang/test/vm/16_option_result/022_result_multi_error/result_multi_error.at) | G2b / R2-L,R2-X | stdout 404\ntimeout\n200\n（现存expected） | enum/Result/多层is与文本载荷 |
| AP-07 | [crates/auto-lang/test/vm/17_modules/001_use_fn/use_fn.at](../../../crates/auto-lang/test/vm/17_modules/001_use_fn/use_fn.at) | G3a / R3-M | stdout world\n3\n（现存expected） | 必须打包auto.greet_mod等真实依赖；没带库不能算语言失败 |
| AP-08 | [crates/auto-lang/test/vm/24_generics/001_list_struct/list_struct.at](../../../crates/auto-lang/test/vm/24_generics/001_list_struct/list_struct.at) | G2b / R2-L，SEM-03先决 | stdout 2\n（现存expected） | 内建List<用户record>不是完整用户泛型；str字段需决策 |
| AP-09 | [crates/auto-lang/test/vm/18_ffi/038_json_encode_parse/json_encode_parse.at](../../../crates/auto-lang/test/vm/18_ffi/038_json_encode_parse/json_encode_parse.at) | G3b / R3-C,R3-H | stdout \"hello\"\nobject\n（现存expected） | actual JSON provider；824 freeze不替代Json.parse |
| AP-10 | [crates/auto-lang/test/vm/18_ffi/051_file_write_read/file_write_read.at](../../../crates/auto-lang/test/vm/18_ffi/051_file_write_read/file_write_read.at) | G3b/G4 / R3-H,R4-P | stdout hello auto\n0\n（现存expected）；文件内容及删除状态 | 只在专属scratch cwd运行；0/bool显示规则另核 |
| AP-11 | [crates/auto-lang/test/vm/18_ffi/055_process_args/process_args.at](../../../crates/auto-lang/test/vm/18_ffi/055_process_args/process_args.at) | G4 / R4-P | 无相邻expected；必须按固定argv/argv0与显示合同补强 | 不能把随当前测试进程变化的stdout默认为正确 |
| AP-12 | [crates/auto-lang/test/vm/23_actor/006_state_increment/state_increment.at](../../../crates/auto-lang/test/vm/23_actor/006_state_increment/state_increment.at) | G5 / R5-A | stdout reached two\nreached three\n（现存expected） | 消息到达/状态/stop与实际调度生命周期需补观测 |
| AP-13 | [crates/auto-lang/test/vm/22_generator/001_sum/sum.at](../../../crates/auto-lang/test/vm/22_generator/001_sum/sum.at) | G5 / R5-A | stdout 6\n（现存expected） | yield suspend/resume不是普通for-in List |
| AP-14 | [examples/script-to-ship-demos/async-demo/main.at](../../../examples/script-to-ship-demos/async-demo/main.at) | G5 / R5-A | 手审stdout compute(21, 0) = 42\ncompute(5, 1) = 11\n，待冻强oracle | serial await目标；并发完成顺序不任意排序 |
| AP-15 | [examples/http_server/server.at](../../../examples/http_server/server.at) | G5 / R5-S | 无相邻expected；真实loopback GET应含Hello from Auto!；补协议/退出oracle | 硬编码8080及无限循环需专属可用端口环境/有界contained harness；占用时不杀用户进程；改端口另记变体 |
| AP-16 | [examples/ui/015-notes/src/front/app.at](../../../examples/ui/015-notes/src/front/app.at) | G6 / R6-U | 无文本oracle；需真实新增/编辑/搜索/选择笔记、state与截图/退出断言 | 完整pac+modules/assets/provider闭包；首帧不代表交互通过 |
| AP-17 | [examples/script-to-ship-demos/trait-demo/main.at](../../../examples/script-to-ship-demos/trait-demo/main.at) | G3a / R3-T | 手审stdout max(up): 9\nmax(down): 3\n，待冻结 | spec/泛型/双实现者分派，不能只证明一种receiver |
| AP-18 | [crates/auto-lang/test/vm/99_script_err/05_obj_call_auto_catch/obj_call_auto_catch.as](../../../crates/auto-lang/test/vm/99_script_err/05_obj_call_auto_catch/obj_call_auto_catch.as) | G6 / R6-F，SEM-08先决 | 现存expected caught: TypeError: object is not callable\nend\n | 已知旧保守误报见证；需决定保留/修正语义，不能把该错误当然作新规范 |
| AP-19 | [crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.at](../../../crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.at) | G3b / R3-C | result=600（现存expected.result） | 匿名对象+动态键写入，不能用固定record fixture冒领 |
| AP-20 | [crates/auto-lang/test/vm/18_ffi/056_math_pow/math_pow.at](../../../crates/auto-lang/test/vm/18_ffi/056_math_pow/math_pow.at) | G2/G3 / R2-N,R3-H | result=true（现存expected.result） | 宽浮点算术/Math装配不是位型往返 |
| AP-21 | [crates/auto-lang/test/vm/99_py_dispatch/01_dual_len/dual_len.as](../../../crates/auto-lang/test/vm/99_py_dispatch/01_dual_len/dual_len.as) | G6 / R6-F | stdout 6\n3\n（现存expected），环境需固定Python/numpy | 缺依赖为infra/gap；有.py脚本不等Python SKU已验证 |
| AP-22 | [experimental/ac-core/fixtures/scalar-source/valid/p-binop-args.at](../../../experimental/ac-core/fixtures/scalar-source/valid/p-binop-args.at) | G1 / 827 | entry i32=0，命名/位置/嵌套/多行表达式各组手审 | 原有scalar-source文件不改写，OS exit与entry result分开 |
| AP-23 | [experimental/ac-core/fixtures/scalar-source/valid/p-short-circuit-trap.at](../../../experimental/ac-core/fixtures/scalar-source/valid/p-short-circuit-trap.at) | G1 / 827 | guest integer divide-by-zero trap，无正常entry_result；native测试exit70仅独立通道 | 第一条false RHS不执行；第二条true RHS必须trap，不能任何错误都算通过 |

## 目录使用合同

- G0/827 T-01全量扫描真实VM语料、CLI/SDK入口、stdlib调用/模板/服务/UI及可分发feature，附seed以外的清单和缺项；不能只挑这23例宣布覆盖全语言。
- 原源码/资源/模块闭包冻结输入hash；归一EOL/数值显示/argv0等只按已批准比较合同进行，不能删import/语句/分支或改语言来提高成功率。导出的最小变体、finite服务fixture、故障注入都有独立ID和source hash，原AP状态不被其通过替代。
- 前端失败、unsupported、invalid MIR/ABC、guest trap、resource、cancel、infra分别给stage/code；无oracle/工具/依赖时是明确gap，0例/skip/任意Err不算语言负例。
- stdout/result/exit/trace/文件或网络副作用/Drop与进程退出是独立通道。AP-23的native exit70不等于AV guest trap的结构化结果；AP-10的旧0显示与bool语义需SEM决策。AP-15外部强杀不证明guest Drop/优雅停止，后者用支持真实cancel/stop的另一个程序专门验收。
- 强oracle优先手审数学/状态/协议结果和规范；旧expected是待核的历史输入，AP-18已知误报尤其不能冻结成“新引擎必须有同样bug”。当两引擎语义profile不同，先列差异/决策，比较共同域或明确compat profile，不偷偷统一结果。
- future harness只修改本次scratch/自有进程/配置端点；库/资源从固定闭包复制或只读加载。禁止在源目录写临时文件、杀共享端口占用者或使用任意shell/exe作为case载荷。
