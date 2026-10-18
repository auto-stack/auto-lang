# PLAN-804 R3 — readiness与定向复审：needs_fix

2026-10-17，Asia/Shanghai。review_scope=final，检查深度止于readiness与受影响的定向复核。前两轮final失败历史不重置；本轮未在已知必验缺口之上再展开一轮完整终审。

当前Codex `/root` 为独立于实施的review上下文，沿用本chat复审历史；模型/会话ID未暴露。没有创建agent/新工作树、修改实施、合入、发布canonical Specs或ledger。

## 基线与就绪

- 工作树 `D:/autostack/.wt/lang-804/auto-lang`，branch `plan-804-dev`，提交 `21d5f9c018cf64b1c6e7439075819871db7df17a`，对上一受审 `12e8936281620f660883a30e90beaa846e61af0e` 为3文件+373/-283。
- 总diff base=`b7ecea5e36ddf41d9effc49f0aba53cca47b99ad`，auto-down兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。提交、Plan、实际锁/配置、Spec与技能hash见 [baseline](baseline.json.txt)。根锁与R2同为`afcd298e…065fd`。
- 已核对§9两轮失败诊断表：承认“同函数不等于同路径”、需要退出残留反控、warning须按changed-file审计，诊断方向正确。
- 但交接同时明确最终形态“未能当场取得第二次绿”，仅引用较早形态18.35s成功叙述。没有绑定当前代码/驱动/收尾形态的完整真实宿主通过结果。不能将早期通过覆盖后来的触发、重试及清理修改。

## Findings

### F804-R3-01 / P2 — 当前真实宿主验收尚未闭合，不能execution_done

影响：AC03、V05、T05/T06；延续R2-F01/R1-F02就绪缺口。

新example确实通过生产`run_dynamic_desktop`，测试删除了手工补锚/选目标/注入，方向已修。新oracle要求started=true、finish=copy|none、非发起方marker缺席及宿主正常退出，这些是可区分的生产结果。

当前缺的是**这份最终实现的实跑通过证据**，不是要求再跑一次无意义的历史绿。计划work记录自己承认该缺口，而work-r3目录仅有输入反控脚本和README，没有最终成功收据。此前一次成功的文字说明未附可绑定当前源/二进制的原始日志。

本轮已重新构建当前example及lib test，普通全链启动前，独立脚本读到预存Return按下，故未启动会注入Tab/Enter的流程、未改该初始键态（[precondition](dnd-all.json.txt)）。这不是当前代码正常路径失败，也不能据此将必验转成pass或批准降级。需原树修复收尾后，在明确的清洁输入条件下完成当前提交全链与退出反控，保留命令、HEAD/二进制/marker、实际结果；没有该证据就保留executing。

### F804-R3-02 / P2 — 新宿主测试失败时留下子进程

影响：AC03资源恢复，V05，T05/T06；本轮修复引入的子进程所有权缺口。

位置：`crates/auto-lang/src/tests/plan804_os_e2e.rs:1312–1315`创建Child，`:1338`窗口超时panic，`:1358`前台断言及`:1394–1397`finish断言均早于正常child等待。Child没有启动即生效的终止/等待守卫；唯一显式kill位于已经走到成功marker之后的退出超时分支（:1408）。

独立 [os-probe.py](os-probe.py) 通过当前测试二进制启动该单测，只挂起本次创建的宿主、在窗口创建之前模拟初始化停滞，不发任何键鼠输入。测试按既定25s超时exit101；**测试退出后宿主仍存活**。同一失败路径两次复现（PID31672、21292）；第二次完整原始结果见 [JSON](host-no-window-replay.json.txt) 和 [测试输出](host-no-window-replay.log.txt)。窗口集合为空、marker为空；输入前后保持不变。复审脚本在记录之后核验创建时间和精确exe路径，仅终止并等待自己跟踪的两个遗留宿主，未清理其他运行实例。

预期不是“测试在初始化停滞时也应通过”，而是**它失败以后必须回收自己启动的宿主**。外层复审脚本的回收不能算测试自身已修。相同所有权漏洞也覆盖前台取得失败和finish缺失。修复应在spawn成功时立即建立子进程守卫，所有成功/断言失败/timeout/unwind出口都终止（必要时）并wait，输入/窗口/光标收尾也须完成；新增启动超时和交付失败反控，逐一断言无遗留宿主及无输入残留。

## 旧finding闭合与范围

| Finding | 本轮判断 |
|---|---|
| R1-F01 父窗/取消 | 保持关闭；对话框实现、测试与依赖自R2未变，本轮不重复已通过V04 |
| R1-F03 792 | R2独立25/25证据保持历史结论；R3不冒称重新生成了当前producer包。当前readiness未满足，不执行昂贵全链交付重放 |
| R2-F01 真实事件链 | 源码方向已修，完整证据待R3-01；未授关闭 |
| R2-F02 输入残留 | 五会话正常收尾子路径关闭：本轮真实五会话成功，退出立即/+250ms left/right/escape皆false；宿主正常/失败整体收尾仍由R3-01/02阻断，不将单腿pass扩成全链pass |
| R2-F03 新增warning | CLOSED：原renderer手工接线273行已移除，新example/e2e触面无新增warning，新增段格式正常 |

## 本轮验证

| 操作 | 实际结果 |
|---|---|
| `cargo build -p auto-lang --example plan804_real_host --features native-dnd` | exit0，38.21s；[build](build-real-host.log.txt)，禁用故障sccache wrapper，未改代码 |
| `cargo test -p auto-lang --features native-dnd --lib v05_real_host --no-run` | exit0；[compile](build-os-tests.log.txt)，当前test binary=`auto_lang-44e2db1f10ad4e73.exe` |
| `python os-probe.py dnd-five v05_real_dodragdrop_sessions_via_vm_shim` | 1/1五会话，测试9.75s；[log](dnd-five.log.txt)、[键态/退出反控](dnd-five.json.txt)，立即与+250ms均无left/right/ESC残留 |
| 普通双测全链启动前检查 | 输入三键left/right/ESC及Tab松开，Return持续按下。未开始注入输入的普通双测，不把预条件缺口记作正常路径代码失败 |
| `python os-probe.py host-no-window-replay v05_real_host_dnd_finished_delivery --suspend-host` | 受控初始化失败时单测exit101、25.02s，宿主遗留；外层清理后无本次宿主。无键鼠注入 |
| fmt / diff | example/e2e exit0；renderer仅R2已知预存9处格式差异；[fmt](fmt.json.txt)，git diff --check exit0 |

五会话不使用Tab/Return，故其反控只要求并采样left/right/ESC，未修改预存Return态；JSON仍保留完整前后键态。这避免把既存键态当成当前测试注入或擅自清理它。受控宿主失败反控全程不发输入。

两个已编译二进制hash见各JSON，当前test=`3748adb9da4f1d034aea2030ef7174903b0ab13265047c88a28a21cb63d2012d`，host=`a7cec59ea69f692836e5cd4e7b55dfc7a134d50144fbd5cd6cbfbc4ead477b94`；未以旧exe作当前证据。

## AC / SD / 路由

AC01/02：本轮生产shim/接口/adapter/装配、对话框语义及依赖配置未变，保留R2已核结论；本轮仅定向检查，未重新授完整final pass。AC03 partial（R3-01/02）；AC04部分证据保持、当前完整结论pending；AC05 fail；AC06 merge pending。

规范增量及现有元数据路径/GOAL003/007维持，冻结见 [spec-delta](spec-delta.md) 和hash。新example是验收设施，不是生产API；当前未闭合不能写成规范验收已完成。canonical Specs/ledger未修改。

outcome=needs_fix，Plan回executing，保留T01..T04，重开T05/T06，4/8；T07/T08未完成。next=原804树work补当前全链证据及子进程全出口收尾，复跑两类失败反控与输入退出反控后提交。已完成的诊断表需增补“child启动即归属/所有失败出口”的覆盖；不以非env档12/12（OS测试env-off早退）或历史一次成功替代。

未运行整档t/tv/tt/tb/tu/taa/tf，也未重复未受影响的no-ui、V04或完整792。最终pass后的merge-owned冻结裸t/tv、master/802并集、777重绑、受影响consumer、Spec/ledger/索引/归档及wt-guard清理继续pending。未合入。
