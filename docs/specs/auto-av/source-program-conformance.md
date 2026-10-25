# AV 源码程序符合性合同（source-program conformance）

> PLAN-827 r3 交付（R10 冻结文本）。合同正文如下，描述经独立复审检视的实际
> 行为。

## 套件与信任边界

源码程序套件为 `crates/auto-lang/test/cases/av-source-827/`，复用
`ac-av/core-run-v1` 既有有界 worker 与父侧判定。不新增
`av/source-run-v1`、`run-source` 命令或 workspace。可信宿主一次性解析有界
source bytes/root 上下文；准入后的字节在其后磁盘变更下保持冻结。原始
fixture 字节只读；私有 overlay `return 0` → `return 9` 同时改变源身份与
结果。

执行路径为 source → Checked HIR → 共享 MIR → ABC decode/verify → AV →
typed actual。旧 AVM/native 不执行新路径；native 仅作独立 oracle。
Source/worker/nonce/request/当前授权与工件身份在既有各门核验。
parse/admission/IO、infra/resource/cancel 与精确 guest trap 通道保持区分。
非法准入永不 dispatch；运行期失败保留合法 trace 前缀。

## 资源与证据

既有 cap 与内存记账不变：输入 4MiB、适用表/嵌套/验证限额、执行
fuel/frame/wall/8MiB 槽位 cap、hot8/staging16/execution8 MiB 分区、前端
瞬态 A ≤224MiB、worker 残差 B ≤128MiB。既有原始装载结构与时间门保持必需；
小文件延迟不能替代它们。帧槽位分配在预算检查后可失败；失败返回既有
SlotBytes 资源结果；成功分配的更早缓冲由帧拥有，并在完成/出错/取消/drop
时释放。这不承诺所有进程分配可恢复 OOM。`FrameAllocationProbe` 是真实
根/被调帧槽位 reserve 边界上的可选线程局部验证 API，不是 guest 输入、授权
能力或全局 allocator 替换。

bench 零阈值反控要求实际零门与两条冻结硬门路径都按该阈值拒绝；无关失败
不能满足预期反控。留存的调用文本必须逐字；历史溯源更正与原始测量分开
保存。

Scope 成功要求每个精确必需成员恰一次、无必需 skip/缺失工具/零选择、子失败
传播。acceptance 含 resources。bench 证据必须绑定实际当前
source/依赖/配置/输入/二进制身份；历史证据仅在其未变范围内可复用。成功与
失败 raw 样本都保留。
