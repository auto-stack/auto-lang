# PLAN-798:r7 合入准备与冻结门禁收据

stage: merge；checkpoint: prepared。完整最终审查R19绑定83086e76f，R20告警清理与R21实际主检出依赖审查绑定216d139ab。评审者为实施会话，独立上下文限制已公开，不冒称另一位评审人。

本收据确认准备与验证，不预先声称已合入/归档/清理；实际落地与清理另见landing-receipt.json.txt、archival-check.json.txt及cleanup-receipt.json.txt。交付承诺按r7：798有限MIR/ABC/AV、受管持久恢复与真实计量；完整函数增量归优先828。

## 当前冻结门禁

216d139aba888760963d12a5f2cbe693489b06c4上重新完成：无过滤cargo t --no-fail-fast --test-threads 8 5320/5320、cargo tv 163/163、独立workspace 105/105（包含真实native PE、cache20及资源17）、共享auto-test-case合同30/30，均exit0。每项原始日志/退出码/冻结提交/依赖锁见final-frozen-*与gates-audit.json.txt。默认日常筛选未临时缩窄，仓库配置/断言/阈值不改；8仅是本轮运行并行度。正式性能门仍依既有reviewed配置独立执行。

前两轮root失败原件保留：首次旧native公共worker未就绪导致10例超时；显式正常构建该worker后家族13/13通过。第二轮旧state_file锁在degraded分支丢一次计数；该文件相对主分支未改，且批量收据已有同名观察，isolated三轮与之后无过滤5320全绿。没有改测试断言、超时、过滤规则或给旧失败改写PASS。之后仅清理新增unused import，R20复审后在新冻结提交再次执行本节四档。

216d前两轮完整5320中均5319通过，唯一失败为既有plan484_024_charts_streaming_recompute；保存在final-frozen-t-attempt1/2原件。该测试与依赖代码未被本Plan修改，批量收据有同名记录；isolated三轮均通过，而完整池中都是parked Init未完成即断言t29失败。第三轮改变执行条件为并行度8，仍完整5320、无过滤/阈值/配置变更。最终PASS不抹去旧失败，也不据此宣称旧flake已修好。

旧E970根锁第三轮5319/5320，charts与锁用例已通过，唯一失败是vm::tests_clipboard_natives::clipboard_files_and_image_via_vm_shims（heap ref NV）。原件attempt3保留，剪贴板源码未被本Plan修改。发现根锁不一致后没有继续在该锁上刷绿，改为实际主检出F1锁；本报告四档的root完整轮在F1锁上5320/5320通过，覆盖该用例。不把“换锁后本轮通过”写成依赖升级/回退修复剪贴板的因果证明。

旧正式性能与180行原始测量保留其R18原始身份：formal编辑P95 872.5ms/签名387.7ms；release warm编辑946.69ms，双线内存/正确性通过。旧01dc正式687.4/234.9ms保留，分别绑定原提交，不能称这些数值在216d重新测得。R20仅移除unused import，不改变任何执行函数、合同、锁、输入或阈值；重放映射与Spec增量检查一并保留。R21又发现原工作树根锁与主检出根锁不同，未改主检出；改为在工作树使用实际主检出锁F1F9855F...，重新类型检查与正式7例/30样本，再最终复审、执行四档。旧E970...根锁结果不冒领实际主检出依赖的合入门禁；不据依赖差异直接推导先前flake的因果。最新正式数值见main-lock-formal-audit.json.txt。独立实验workspace自己的入库锁始终未变。

## 规范与派生账本

11份MIR/AV canonical新增、3份AC/HIR/TestCase合同修改及索引已在唯一工作树准备。拟Spec不宣称完成函数前端增量、驻留代码替换、CTE/JIT或默认生产接线。新增恢复manifest format2与诊断快照轴不混同ABC wire2/schema2/semantic2。

外部Codex会话Regime B；本机8080 store在非沙箱探测下主动拒绝连接。按技能采用guarded offline projection，不声称store写入成功。先验证已单独提交的有效v60恢复基线，逐字节备份，再对designs/tests/reviews/reports追加5项且版本+1；其他字段/旧项/字节保持不动。实际投影与owner Rust Serde读回证据单列。current知识条目指向canonical Specs，review/report指向不可变历史原件，不复制Plan章节。

## 构建产物观察与后续

本计划为独立实验与测试接缝，未接入默认VM/生产应用。主检出target/release/auto.exe、auto-lang.exe、实验av_probe_worker.exe以及gen/front/vue/dist/index.html在准备时均不存在；没有生产进程被部署或重启。工作树实验release曾用于冻结性能验证，删除前保留源码、锁和原始结果；主检出后续使用实验入口须正常构建，不以源码落地声称已有生产二进制。auto-down源码未修改，依赖身份保持c39ac7f9d。

批量tf到期规则在最终清理后再次检查；当前2026-10-20T09:20Z已覆盖817，798非5的倍数且未满48h，不触发。828为798实际交付后的下一优先实施计划；其余ACE扩展仍受D828和增量保持控制约束。
