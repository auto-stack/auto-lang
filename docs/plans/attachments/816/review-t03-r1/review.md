# PLAN-816 T-13 / T-03 受影响复审 R1 — needs_fix

2026-10-21（客户端日期），revision 2。review_scope=phase，限T-03已提交改动；不是
M816-B最终审查，也不重审或撤销已发布M816-A。本聊天Codex /root独立于T-03实施
会话，从源码、旧公开合同和独立consumer重建结论；具体模型/会话ID未暴露，未创建agent。

- 原树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`，入场/收尾clean。
- 受审 `1629aa60f0bc890c06a6a403aad5e6c07697c35c`，实现c73a9fe48a、样式df1b36216。
- 代码基线 `7a2323cc364000dca8777d06c8d16629d1b8456f`；含已发布/恢复的814能力、
  M816-A及815窄tool。T-03代码尚非master祖先，未当成已合入交付。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- [85项输入指纹及6个旧版本blob](inputs.json)，Plan hash在本轮簿记前采集。
  [冻结既有SD合同](spec-delta-frozen.md)，SHA256
  `3c9a1fc2fcc19f32a230d15169e82ef69e5f9c34c9ebe4a00612e89dcf7f63bc`。

**五项P2未闭。** 静态闭包与Pac/Target/Automan类型单源通过，不能替代旧库默认
行为、旧模块路径和服务可用性。下面均为已有T-03/AC-01及§5.4兼容合同内缺口，
不是要求提前完成薄CLI、完整UI工具协议或M815-B。

## F-816-T03-R1-01：旧auto-man库入口新增未批准的安装前提

旧 `auto_man::Automan::create_jet(path)` 直接执行原生成器，无需先安装UI回调。
现在 `auto-man/src/lib.rs:14` 直接再导出runtime类型，调用落到
`auto-man-runtime/src/automan.rs:68–70` 的ui_ops::current；默认provider是
NotInstalledUiOps。只有CLI main和一条测试主动安装compat，旧库调用方没有这个步骤。

独立consumer直接调用**同一历史入口**：

```text
no-install: Err("UI ops not installed ... install_default_ui_ops ..."), output_exists=false
with-install: Ok(()), output_exists=true
```

[最终整组回放](replay-current-output.txt)；[consumer源码](compat-probe.rs)。两个模式
分别启动新进程，with-install实际生成Jetpack工程，不以fake或仅编译当正控。
旧函数体与当前CompatUiOps里的生成逻辑可在基线diff核对，默认失败发生在调用算法之前。

这是**旧compat入口的默认行为**问题，不要求纯runtime给UI操作提供隐式实现。
应保持旧公开调用的可用性，同时保住runtime零UI闭包；若要改变旧库初始化合同，须
正式修订并获批准，不能用“库消费者注入”说明替代既有兼容要求。
闭合oracle：新进程直接调用旧入口成功、产物真实，显式注入正控继续有效。

## F-816-T03-R1-02：非UI独立服务依赖UI provider

服务模块声明VM轨不启动UI，Plan将语言执行归语言adapter；但
`auto-man-runtime/src/http_service.rs:135–136` 调UiOps.run_language_file。
NotInstalled版本直接报UI缺失。Rust服务的共享workspace/member命名也经该UI seam，
未安装时 `ui_ops.rs:264–268` 返回空PathBuf/空String，未给出错误。
实际共享workspace实现已单源在auto-api-gen::rust_workspace。

[runtime独立consumer](runtime-probe.rs) 使用有效API：

```auto
#[api(method = "GET", path = "/api/ping")]
pub fn ping() str { return "ok" }
```

无UI包/未安装provider时，public serve_project(...,"vm",config)实际返回
`VM service run failed: UI ops not installed: run_language_file`，在执行语言前失败。
workspace helper实际返回path=""、name=""。

同一API/服务调用的已安装compat对照，测试自行占用OS分配的端口：真实语言引擎
登记1条route、进入HTTP bind，然后以预期WSAEADDRINUSE失败；
`SERVICE_RESERVED_PORT_CONTROL=true`，[输出](service-positive-output.txt)。
测试不kill端口持有者，不启动GUI，也没有把预期bind失败写成完整HTTP运行pass。
因此输入确实能到达既有生产路径，未安装分支被UI seam提前截断。

早期print-only诊断探针的原始记录保留在replay-output.txt；本finding依据后来
有效API和bind负控重放，见[当前回放](replay-current-output.txt)及
[有效API失败输出](runtime-valid-api-output.txt)，没有以无serve脚本要求ready。

须把独立服务所需语言执行/通用workspace能力放在其真实非UI owner接线上，
不能为运行无UI服务而要求安装静态UI compat，也不能用空路径伪装可用能力。
闭合oracle：runtime-only服务到达真实VM/正确bind错误；Rust服务使用真实命名/
workspace，不依赖UI provider；闭包仍无UI实现。

## F-816-T03-R1-03：旧util模块路径断裂

基线的auto_man::util公开再导出六个函数。现在glob再导出的util是runtime::util，
这六项被放进 `auto-man/src/lib.rs:38–42` 的新ui_port_util，根路径虽仍导出，
**旧模块路径**没有保持。

[legacy-paths.rs](legacy-paths.rs) 原样导入
`auto_man::util::{hash_string,http_base_url,http_port,kill_process_on_port,parse_tasklist_name,should_skip_kill}`，
实际E0432六项均不存在，见当前回放末尾。这是消费者编译回归，不是链接闭包问题。
同fixture中Pac/Target/Automan跨门面的类型赋值编译通过，因此没有把“迁移必然
改变类型”当作原因。恢复旧路径转发时须保留函数算法的既有单一owner，不要求复制。

## F-816-T03-R1-04：Git detached判定反转

`auto-man-runtime/src/git.rs:48` 在 `git branch --show-current` **非空**时报detached；
基线是为空时报detached。本轮从output()迁到执行设施时改反了条件，属于已建立原因。

独立consumer在仅由测试创建的仓库执行真实Git：init main→空提交→检查→checkout
--detach→再检查，实际：

- attached main：Err(detached)，应Ok。
- detached HEAD：Ok，应Err(detached)。
- 同一仓clean status为Ok、写入dirty.txt后为Err，正反控保持。

[完整输入/命令](runtime-probe.rs)，当前回放含两实际值。没有对用户仓做checkout或
写Git配置。应恢复原判定，保留四状态控制；只改函数名或错误文案不能关闭本项。

## F-816-T03-R1-05：迁移删掉四条回归并产生重复登记

基线generation_request_adapter的下列测试，在当前整个crates树均未找到同名迁移：

1. request_from_pac_carries_all_fields
2. input_sources_follow_request_front
3. sidecar_source_is_request_not_disk
4. sidecar_obj_and_nested_prop_forms_survive_request

前两条是实际Pac/请求投影与front根/递归/干扰排除，不依赖UI实现。
本轮从基线保留其断言，只有auto_lang→auto_lang_core命名空间和外部consumer
import适配：[两原回归源码](preserved-adapter-tests.rs)，在runtime-only consumer
**2/2通过**，[结果](preserved-adapter-output.txt)。首轮fixture漏tempfile依赖的
编译错误另存preserved-adapter-fixture-error.txt，补依赖后通过，不判产品缺陷。
后两条确需UI owner的PacSource，但应迁到相应compat/消费测试，不能为闭包纯净删除oracle。

删除还留下 `generation_request_adapter.rs:391/:394` 两个#[test]都挂到
apply_evaluated_root_b_group。实际cargo list为119个描述符、118个唯一ID；该ID出现2次，
并报duplicated attribute告警。[列表汇总](test-list-summary.json)。
本轮115次通过不是115个不同测试：实际114个唯一通过、4个唯一ignored。

须恢复四条原意义控制并建立明确归属，去掉重复登记；用测试ID和断言逐项核对，
不能以改名/删条后的总数证明旧覆盖。此项也包含本次新增duplicate属性健康问题。

## 通过项、验证边界与Spec

| 检查 | 独立结果 |
|---|---|
| 三包cargo check | exit0；[诊断](diagnostics.json)，不把继承的旧warning判新增 |
| runtime定向包测 | [115通过执行、4ignored](runtime-tests.txt)，去重后114通过；不是全仓档 |
| management真实集测 | [8/8](management-tests.txt)，含显式安装后真实npm/node build及失败传播 |
| 两条被删纯adapter原回归 | 独立consumer 2/2通过，说明保持它们不需要引入UI闭包 |
| 单源类型身份 | Pac/Target/Automan跨auto-man与runtime赋值编译通过 |
| 静态闭包 | runtime normal/build图420包，禁项0；独立runtime consumer另起workspace实际构建，非通过compat统一特性证明 |
| 777 strict | [pass，132边/0违规/3grandfathered](modularity-output.txt) |
| fmt与diff空白 | 两包fmt --check exit0、diff --check通过 |
| clippy --all-targets --no-deps | exit101；30个唯一unused_io_amount错误（lib/lib-test重复报告60），均在两份逐字保持的旧builder/cmake模块。见[来源证明](clippy-baseline-proof.json)，未伪称全绿或将其列新增finding |

静态闭包的[原始树](runtime-tree.txt)/[汇总](closure-summary.json)不含auto-lang兼容
包、auto-ui-tools/compiler/runtime或iced。该证明有效；F02说明“没有静态依赖”仍
可能通过回调在运行时要求UI，不否认已成立的类型/依赖分层证据。
本轮未运行整档t/tv/tt/tb/tu/taa/tf、GUI或全服务部署测试。

T-03对应AC-01服务/compat面partial，受影响的原API/服务/Git行为未通过；T-13本次
检查needs_fix。M816-A的AC-07/08历史pass与AC-09发布事实保留，不因T03未合入改动撤销。
完整M816-B AC-01..04及最终T09审查尚未就绪，未以本次定向审查替代。

SD-03/04当前合同仍要求单源兼容、非UI服务和旧行为保持；本轮不批准将默认手工注入、
丢旧路径或空workspace写成新规范。B的完整as-built delta在T08/最终审查完成；没有
发布canonical Specs或修改live ledger。纯语言TS包装的算法仍委托core公开实现，
本轮未发现其复制业务算法；git剩余已知旧pull错误处理不冒报为本次新增回归。

next：原树work一次修齐上述五项、补合法/失败/无安装/已安装/旧路径及测试迁移控制，
提交后受影响复审。T03及复用作当前审查的T13重开，current_step=4/14，仍executing；
这是当前检查计数，历史A交付收据保留、T14仍已完成，不撤销A发布。

T04必须先满足T03及本次复审；窄tool已发布，**窄能力发现/版本/缺能力负控不需等
M815-B**，完整UI正控与完整profile/bootstrap仍按既有门等M815-B。未提前实施T04。
最终B合入前的冻结日常/实际触面门、完整集成、Spec/ledger/来源收据仍属merge-owned
pending；本轮不做合入、归档或清树。
