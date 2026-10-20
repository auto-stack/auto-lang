# AV缓存、资源与验收

来源：PLAN-798；实际代码 `experimental/ac-av-probe/probe/src/{cache,driver,worker_restore,observations,bench,native_admission}.rs` 及正式 `crates/auto-lang/src/test_cases/ac_av.rs`。

## 持久化与授信

复用783 ManagedStore及796语义身份，函数逻辑身份与body/interface/content轴分开。A08保存已实现MIR unit，A09保存ABC unit，manifest引用逻辑/签名/内容/producer/profile/版本/当前授权、payload fingerprint及来源。keys包含适用codec/语义/能力轴，跨工程/新进程恢复仍真实执行，不缓存PASS或AV结果。

缓存信封与实际载荷核owner/类别/版本/签名/producer/内容；A08、A09、闭包与恢复消费都复核当前能力授权。unit信封不能承载完整module；合法raw类别正控保留。损坏/身份冲突与陈旧版本miss分态，旧歧义v1不按v2消费；同指纹重发布Conflict可按既定规则容忍，其他写失败可观察。

manifest有unit/sidecar required-data边；消费窗口有lease，发布的project/root和依赖边参与GC，恢复期间逐单元保活。这里只保证持久产物生命周期，不表示已管理活帧/在线代码代际。source sidecar适用性失败会重推导并重写，不能静默发布无来源trap。

## 真实粒度与后继

| 阶段 | 798实际行为 |
|---|---|
| 前端 | 每请求整模块parse/resolve/typecheck、Atom render/bind和HIR验证 |
| MIR | miss时整模块lower/verify/split；仅miss单元持久化 |
| ABC缓存 | body编辑通常1 emit、114 hit；hit仍读盘decode/verify |
| 装载/执行 | 完整closure组装、编解码/准入，再执行入口；Driver不保留上次代码 |

因此函数产物命中≠全链路函数增量。828负责source到函数代码安装的完整切片；未改HIR/MIR/代码identity/代际不变，未改生产阶段/产物IO为0，结构与时间双门。798不交付在线热替换、CTE、AJ、DLL loader/状态迁移；native_admission仅真实COFF引用/内容/target/ABI/版本准入的有限合同，不执行新native/JIT。

## 资源与计量

受管32MiB（hot8/staging16/execution8），冷产物压力在同一store上回收，活root/lease保活，用户哨兵不动。冻结负载的worker内存测量硬门沿r5双线（不是全局分配拦截器）：A前端峰值≤224MiB，B整体采样峰值扣同口径前端归因的残差≤128MiB；整体/子进程/归因峰值原样报告，端点增量仅信息项，不把A+B分区或磁盘bytes混为同一指标。常驻新状态不得隐去，新能力扩展必须复测。

六场景各≥30：独立冷store、同进程hot、每样本新进程磁盘hot/跨工程、30不同body修订、目标签名冲突诊断、真实并发GC。固定源273575B/115函数，独立expected/实际值或准确诊断校验；raw/summary同轮绑定输入与二进制，不删失败样本。sig是前端参数不匹配早退，不能代表合法签名更新。

798正式probe/run仍按原技术门记录从请求准备到父侧判定/发布的连续窗口，含逐请求启动/打开store/身份校验；它不是用户定义的驻留VM编辑周转。独立warm Driver bench不能替代formal，正式技术门未过不能授798完成。worker身份每次完整重读文件并计算SHA-256，使用宿主已依赖的加速实现保持相同哈希/替换检测，不缓存身份或跳准入。

稳定工程首个结果之后的完整编辑→正确结果/诊断P95<1s及全链路增量结构门由828交付。后继817/818、820–827新增work-owned兼容项，须重放其source基线并验证新增能力局部更新、精确失效与强制全量退化必红；不能借cold/版本迁移例外使正常body更新整模块重编/重载。
