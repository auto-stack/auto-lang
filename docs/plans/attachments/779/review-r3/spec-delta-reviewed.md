# PLAN-779 R3 规范增量冻结

review_scope=final；plan_revision=1；reviewed_commit=45203b5cad61680097bc9f39590868bac03c816a；outcome=pass；delta_status=reviewed。

源版本：docs/specs/auto-lang/vm/overview.md SHA256=0a892771a4e9d4902641a78a5911741498264238254e17ea126a866f24e8e144；docs/specs/auto-lang/ui/overview.md SHA256=cde75f5c73817c7c8e68108f9200be852ed600912286c13a08a56e2c65c0caa5。

本文件冻结待merge沉淀的实际规则，替代原Plan仅列三个宿主注册点的早期提案；不发布canonical Spec或ledger。

## SD-01 modify docs/specs/auto-lang/vm/overview.md

frame native 9918/9919为标量宿主能力pilot。core shim通过`vm::host_capabilities::FrameProvider`读取begin_ms/present_ms，不直接引用UI的frame_bench实现。canonical名称、固定ID、int单槽编码、i32上界饱和及未捕获0语义保持。

NativeInterface持provider槽，AutoVM构造快照当前进程默认，实例`set_frame_provider`可覆盖；之后进程默认变化不影响已建实例。NativeInterface深克隆保留provider，VM merge保留目标实例provider；provider替换不更改shim绑定身份、不bump binding_revision，register/merge原有contract/proof失效规则保持。

装配分三层：

1. 显式`set_process_default_frame_provider`是替换接口，后显式注册者胜。
2. `set_process_default_frame_provider_if_absent`只在空槽补缺，检查及写入同一写锁临界区。兼容安装不得覆盖调用方选择。
3. 实例provider独立于之后的默认变化，可在该VM上显式覆盖。

无provider的原始Core/no-ui路径读0；高层入口是否安装兼容默认由组合宿主负责。FrameProvider的Send+Sync不改变VM线程亲和，不承诺VM跨线程迁移。此次不外推heap handle、media或async寿命已解决。

## SD-02 modify docs/specs/auto-lang/ui/overview.md

`ui::frame_provider::FrameBenchProvider`从既有frame_bench通道读取；捕获开关、时源、renderer note语义不变。`install_process_default`统一调用if-absent，兼容默认只补空槽。

自动装配位置包括VmBridge两构造、run_file_dynamic_ui_inner、CLI ui-iced入口，以及crate根组合助手`install_ui_host_capabilities`。根助手覆盖run家族执行漏斗、test/debug、create_vm_from_source/abt、配置求值、REPL、解释器、back_proxy的宿主创建路径；ui构建提供兼容默认，no-ui构建助手为no-op。不能再沿用“仅前三点注入、其余所有库程序轨须自行注册”的早期不完整描述。

调用方在高层入口前设置显式默认，UI开/关两种构建均保持真实供值；无人注册时UI宿主恢复frame_bench默认，门开可读真值。原始AutoVM仍只按构造时默认快照，不把UI实现接回vm core。

本pilot仅迁VM frame两函数：a2r_std及生成handler的frame直接臂保留；clipboard、scroll、editor、terminal等未迁族按Plan清单后续迁移，不宣称整个runtime或UI仓已解耦。真实显示性能/呈现吞吐不在本次标量注入验收内。

## 验证边界与元数据

外部no-ui独立manifest6/6、同源外部测试链接workspace锁定的UI库6/6；另有独立默认优先关系/实例/有限并发控制与门开程序轨/bridge真值控制。此证据不等于独立fixture全新锁的ui-iced依赖解析已通过：work记录windows-core版本解析问题未在本次重放，作为后续fixture/发行依赖身份检查事项保留；不得隐去或宣称已解决。

supersedes_spec_components=[docs/specs/auto-lang/vm/overview.md,docs/specs/auto-lang/ui/overview.md]；new_spec_components=[]；touched_goals=[GOAL-007,GOAL-003]。SD01对应AC01/02，SD02对应AC03；无合同revision变化。

merge-owned：冻结提交裸cargo t+tv，通过后沉淀canonical/ledger、合流归档、批到期判定与guard-clean。代码/依赖/验证输入变化后应核对受影响复审证据，review pass不能跳过合入门禁。
