# PLAN-815 T-07子批1：规范影响核验

2026-10-22，Plan r3，review_scope=phase（子批1三块验收，非完整T-07/T-09）。HEAD f6fd0ce7c86fd3c4f70ddddec296f2d021642031，code 661c0573c0e16630cd9c2c8e8da91c7736c40afd，base c203d9642a58a3bceeb2075a7ee116e79a416e45。当前canonical/依赖/配置/锁hash见baseline.json.txt，冻结交接见frozen-batch1.md及frozen-handoff.md。

SD-01..07不增加产品规则、支持profile、API或序列化版本。本轮源diff仅proc_host的cfg(test)模块及独立协议fixture；实际生产源码/二进制/依赖边不变。bin SHA256 f21db18136403d1124ecbc1338f61c5603544401c434e7d12fad20a60c37dfb5与前轮一致，生产来源仍为e351cb2a，不要求仅测试提交产生新工具身份。

- SD-02/tool-protocol：四种正式回执的完整类型载荷往返及根为1的MAX_FRAME_JSON_DEPTH=122边界有证；原fixture新增负控验证写入口拒绝，审查额外以原始帧验证read_frame/parse_frame两个读入口在122接受、123具名拒绝。不是修改协议上限，也不把类型构造当成真实tool执行四种工作操作的新增证明。
- SD-05/07：已披露real_tree的Windows命令改为cfg分支，Windows原行为保留，Unix采用真实grep和sh进程组用例；Linux相关模块与捕获截止独立实跑通过。收口的是测试可移植缺口，不能外推完整Linux平台/资源/生命周期矩阵已完成。
- SD-06：纯协议fixture正常闭包门通过；本轮未跑777 master并集或792发行，也没有为零依赖边变化重采样基线。完整发行/资源审计仍在子批2。

先前已关闭finding按未变且适用输入继承，详evidence-reuse.json.txt。完整字段/兼容矩阵、资源许可和发行真实结果未齐，T-08仍需最后据实定稿；SD-01/02最终增补要保留已发布A内容。Plan的supersedes_spec_components/new_spec_components/touched_goals沿r3保持，未写canonical或live ledger。T-09及其后merge-owned冻结门、实际合入/发布/归档均pending。
