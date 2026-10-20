# ABC与AV标量执行合同

来源：PLAN-798。实现 `core/src/{abc,abc_verify,av,contract}.rs`，相对 `experimental/ac-av-probe/`。

## 身份与准入

| 轴 | 已实现值 |
|---|---|
| MAGIC / ABC_VERSION | AVAB / 2 |
| ABC_SCHEMA / ABC语义 | auto.abc.av-probe/2 / 2 |
| AV_PROFILE / AV语义 | av.core-i32/1 / 1 |
| MIR_SCHEMA / MIR语义 | auto.mir.av-probe/1 / 1 |
| producer | plan798/av-probe |

ABC是显式小端、版本化typed wire，包含模块/函数/类型/常量/imports、槽/块/指令/terminator、kind/entry/能力和来源sidecar；不是Rust enum/地址/Arc的dump。页≤256KiB，长度/offset算术checked；未知版本/opcode/保留位、非法bool、越界ref、截断或校验错误受控拒绝。wire trailer FNV不是授信凭证，结构解码之后仍走语义验证。

FLAG_IS_UNIT明确类别，不从函数数推断。单函数完整模块和unit分别合法；符合对应规则的kind声明改变，不因bit变化本身自动非法。可信producer/unit信封、当前owner/profile/语义和载荷类别不一致则拒绝。v1缺kind的旧载荷不能作为v2读取；公开字节输入拒绝旧版本，缓存按当前版本key隔离/确定性陈旧miss重建，不恢复旧信任。

只有verify_abc产生LoadedModule。它可承载不可执行unit凭证；实际AV只执行类别为完整模块、imports闭合、零参i32入口的可执行闭包。unit不能因翻trusted/executable标志而执行。手工DecodedAbc也复核版本/profile/owner、能力、类型/CFG/初始化/调用和资源，不能绕decoder/信任门。

## 执行与来源

AV显式frame/slot栈，不以guest递归消耗宿主递归栈。默认≤100万逻辑指令、1024帧、8MiB活槽、2s墙钟；取消至多每1024条逻辑指令检查，测试trace≤64KiB。分配前检查根/子帧预算，成功/trap/fuel/cancel均报告实际峰值。成功值、语言trap、resource/cancel和infra分态；先前合法效果保留，不把未走分支trap提前为编译错误。

来源sidecar与当前snapshot/函数适用性绑定；空或不覆盖可trap指令的sidecar不能抹掉诊断来源，必要时从当前source重推导。热编辑尚不复用完整LoadedModule：当前会重新组装、编码/解码/验证闭包，再执行入口。真实驻留函数装载/安装/旧调用保活属于828，尚未由798交付。
