# AV缓存、资源与验收

> **PLAN-858 开发期周转口径**：ACE 有效样本按升序取第 `N-floor(N/10)` 项（从 1 开始）为 P90，硬门为 P90≤2000ms；正式样本数不减少，10 项最多 1 项超过 2000ms，等值通过。缺失、非法或错误结果不得计入绿；保留各路径原有零值准入、真实窗口、结构与资源断言。P95/P99 作为诊断与历史数据保留；架构基本成形后以实测另定长期目标。详见 [TestCase v2 周转口径](../auto-test-case/profile-v2-native.md)。


来源：PLAN-798；实际代码 `experimental/ac-av-probe/probe/src/{cache,driver,worker_restore,observations,bench,native_admission}.rs` 及正式 `crates/auto-lang/src/test_cases/ac_av.rs`。

## 持久化与授信

复用783 ManagedStore及796语义身份，函数逻辑身份与body/interface/content轴分开。A08保存已实现MIR unit，A09保存ABC unit，manifest引用逻辑/签名/内容/producer/profile/版本/当前授权、payload fingerprint及来源。keys包含适用codec/语义/能力轴，跨工程/新进程恢复仍真实执行，不缓存PASS或AV结果。

缓存信封与实际载荷核owner/类别/版本/签名/producer/内容；A08、A09、闭包与恢复消费都复核当前能力授权。unit信封不能承载完整module；合法raw类别正控保留。损坏/身份冲突与陈旧版本miss分态，旧歧义v1不按v2消费；同指纹重发布Conflict可按既定规则容忍，其他写失败可观察。

恢复manifest的JSON格式为2，必含diagnostics_snapshot，查询键带manifest-format=2，指纹绑定此快照；位置编辑复用函数unit身份，但生成独立恢复闭包。旧缺字段/未知格式/请求快照不匹配在执行前受控拒绝。manifest有unit/sidecar required-data边；消费窗口有lease，发布的project/root和依赖边参与GC，恢复期间逐单元保活。这里只保证持久产物生命周期，不表示已管理活帧/在线代码代际。source sidecar适用性失败会重推导并重写，不能静默发布无来源trap。

## 真实粒度与后继

| 阶段 | 798实际行为 |
|---|---|
| 前端 | 每请求整模块parse/resolve/typecheck、Atom render/bind和HIR验证 |
| MIR | miss时整模块lower/verify/split；仅miss单元持久化 |
| ABC缓存 | body编辑通常1 emit、114 hit；hit仍读盘decode/verify |
| 装载/执行 | 完整closure组装、编解码/准入，再执行入口；Driver不保留上次代码 |

因此函数产物命中≠全链路函数增量。828 已交付 source→函数代码安装完整切片：常驻会话内未改函数 HIR/MIR/代码 identity/
代际不变、未改生产阶段/产物 IO 为 0；结构与时间双门（正式战役=常驻 worker+IPC+父侧
judge，计时窗=编辑入队→父侧全部校验完成为终点，n≥30，P90≤2000ms 且全对且结构绿——
结构含修订进度/确切安装目标/目录完整（functions 交叉+entry 锚）/未改函数派发载荷身份稳定（地址+代际+内容，观测由执行器采进 trace）/驻留对象绑定（resident 活地址对比，run 内真实副本执行必红）/执行序配对（record seq==current seq，冗余副本执行必红）；逐样本阶段 wall（真实生产接缝 µs）与真实 IO 字节（无 store=结构零盘 IO）随观测上报；完整观测与逐样本 raw 经 write_evidence **先落盘后断言**留存，成功失败同留）。
资源按 798 §5.9 双线预算硬门：A 线前端编译瞬态隔离测量 ≤224MiB；B 线探针残差
（总峰值−A 归因）≤128MiB，缺归因整峰计 B。存储轴：manifest format=2 双指纹键
（body+规范化 required 闭包）；单元 ABC/sidecar store-internal 挂 required 边，
superseded 根发布点退役；A07 HIR soft root 有界（每函数至多一个当前 cv root，
supersession 退役/回访重挂/restore 按行存在置归属追踪器——与 untrusted HIR 可采纳性
分轴）；512 真实修订+周期 GC 存储水位有界（不随编辑数增长），恢复重验、旧 pin
保活/释放回收、用户哨兵不动。详见 [resident-function-install.md](resident-function-install.md)。798不交付在线热替换、CTE、AJ、DLL loader/状态迁移；native_admission仅真实COFF引用/内容/target/ABI/版本准入的有限合同，不执行新native/JIT。

## 资源与计量

受管32MiB（hot8/staging16/execution8），冷产物压力在同一store上回收，活root/lease保活，用户哨兵不动。冻结负载的worker内存测量硬门沿r5双线（不是全局分配拦截器）：A前端峰值≤224MiB，B整体采样峰值扣同口径前端归因的残差≤128MiB；整体/子进程/归因峰值原样报告，端点增量仅信息项，不把A+B分区或磁盘bytes混为同一指标。常驻新状态不得隐去，新能力扩展必须复测。

六场景各≥30：独立冷store、同进程hot、每样本新进程磁盘hot/跨工程、30不同body修订、目标签名冲突诊断、真实并发GC。固定源273575B/115函数，独立expected/实际值或准确诊断校验；raw/summary同轮绑定输入与二进制，不删失败样本。sig是前端参数不匹配早退，不能代表合法签名更新。

798正式probe/run仍按原技术门记录从请求准备到父侧判定/发布的连续窗口，含逐请求启动/打开store/身份校验；它不是用户定义的驻留VM编辑周转。独立warm Driver bench不能替代formal，正式技术门未过不能授798完成。worker身份每次完整重读文件并计算SHA-256，使用宿主已依赖的加速实现保持相同哈希/替换检测，不缓存身份或跳准入。

稳定工程首个结果之后的完整编辑→正确结果/诊断的当前 P90≤2000ms 开发期时间门及全链路增量结构门沿用828实际路径。后继817/818、820–827新增work-owned兼容项，须重放其source基线并验证新增能力局部更新、精确失效与强制全量退化必红；不能借cold/版本迁移例外使正常body更新整模块重编/重载。


798 r7用户交付裁定：六场景warm结果及所有失败如实计量保存，已有正式PASS证据持久保存；整模块重查的完整增量优化归优先828，不再以warm测量超1秒单独阻断798。798当时冻结的30样本两路径 P95<1000ms 是历史技术门；PLAN-858 将当前 ACE 开发期门改为 P90≤2000ms，原正确性/双线内存/K14和生命周期门继续必需。原bench及Scope turnaround仍可作为严格诊断工具返回目标未达，不能把诊断退出码单独冒充r7合入结论，也不能将资源/正确性失败归为时延例外。

## av.data/1 持久与恢复（PLAN-820）

新 producer `plan820/av-data` + A20 层（schema `ac820/a20-data-abc/v1`，键轴
producer/owner/entry/profile/types-fp/codec/三语义版本/source-snapshot）。恢复
重走 envelope→decode→深验（含布局重算）后执行：好 hash 坏载荷拒（校验和+
内层 MIR 深验）、类型表变化 clean miss、跨 profile 信封身份拒；同 store 新
进程 worker `data-restore-run` 恢复重验执行。V-08/09（cache_compat 5/5 +
store_restore_new_process；好 hash 坏内层独立反例见
[review-r10](../../plans/attachments/820/review-r10/review.md)）。


## PLAN-827 源码程序消费（source-file 身份/恢复/反控）

PLAN-827 消费既有内容寻址 source/cache 合同。fresh、同 store hit 与真实新
worker restore 都执行 AV 并获得新的父侧比较；不使用缓存 PASS/actual 或
序列化活帧。Source 内容/位置、依赖、profile/工具/producer/entry/当前授权
参与既有身份与消费门。授权撤销在 dispatch 前拒绝；位置编辑把诊断 span 绑定
到当前坐标。相同内容可跨工程标签复用；发布 root 与必需数据 lease 仍拥有 GC
可达性。错误 owner/body 身份、损坏帧与自洽坏内层载荷被拒绝。冲突发布保留
已提交的好工件。执行槽位所有权与持久 store lease 是分离的资源轴；失败清理
证据必须覆盖每条所声称的轴。


## PLAN-821：owned A21 层与新进程恢复

PLAN-821 交付 owned A21 工件层（producer `plan821/av-owned`，AVAD v2，
键轴含 types-fp 与 source-snapshot）。worker `owned-restore-run` 为真新进程
恢复：信封校验→decode→深授信→执行，跨进程 canonical 面同结果；自洽坏内层
（正确 checksum 重封）在深验证拒且 dispatched=0（exit 4）。types-fp 变化为
clean miss；producer/schema/profile 身份三向隔离。恢复永不信任磁盘；活帧/
borrow/宿主地址不入缓存。证据：av_owned_821_cache_compat 7 例。


## PLAN-822：A22 collections 层与恢复身份绑定

新 producer `plan822/av-collections` + A22 层（AVAD v3，键轴 producer/owner/
entry/profile/types-fp/codec×3 语义/source-snapshot）。恢复零信任盘面：信封
（producer/owner/snapshot/语义外层核对）→ decode → 深验后，**载荷实际身份四
绑定**先于 dispatch（V-08）——

- **实际 profile**：解码 profile id 必须 = collections（id 4；合法 core-i32
  wire 于 collections 身份下拒——通用门按载荷自身 profile 选版本元组，不代
  请求方强制，R17）；
- **实际 types-fp**：解码模块类型表指纹与键轴同式比对（异型表拒，R14）；
- **实际入口名**（R15；封套从不携带 entry——builder 忽略该参数）与**入口
  函数 source_snapshot**（R15）逐项等于请求身份。

owner 轴由准入门**重盖章**（载荷无自证权，不发明 wire 自证字段）。世代隔离
v1/v2/v3 互不可读；好信封坏内层、内容寻址同键冲突（防篡改）拒；确定性重放
步数一致。证据：av_collections_822_cache_compat 8 例 + review-r14..r17。

## PLAN-824：动态代码与 portable 值分域

`plan824/av-dynamic` 的 `av.dynamic/1`（profile 5、AVAD v4）沿现有受检代码缓存恢复链，portable 值以 `auto.portable.av-dynamic/1` 分域；恢复重验当前类型/字段/分支及授权，不从保存的载荷恢复信任。动态与旧 profile 的 schema/producer/语义身份隔离，坏内层或改变依赖受控拒绝。资源和 typed 支持边界见 [dynamic-portable-values](dynamic-portable-values.md)。

## PLAN-823：A23 closures 层与恢复身份

新 producer `plan823/av-closures` + A23 层（AVAD v5、profile id 6，键轴
producer/owner/entry/profile/types-fp/codec×3 语义/source-snapshot）。缓存只
持久化代码与可持久依赖，活 env/borrow/宿主地址不入缓存；新 worker 恢复零信任
盘面：信封→decode→深验（含 env 声明逐位一致）→载荷身份绑定（profile id 必须
=closures 6、types-fp、entry 名与 source_snapshot 逐项等于请求身份）后执行。
好 hash 坏内层拒；types-fp/entry/snapshot 轴 clean miss；与 dynamic/collections/
owned/data 各 profile 的 schema-producer-语义身份互斥，交叉 owner 工件在准入门
拒绝（av_closures_823_dynamic_824_compat 常驻正反控）。闭包层与 824 的直接/
嵌套 pack/freeze 拒绝（任意深度）为两 profile 共同边界。恢复与增量会话合同见
[resident-function-install](resident-function-install.md)。

## PLAN-825：LoadedProgram 恢复与索引重建

恢复的 portable 工件先进既有 envelope/decode/verifier 门再重建执行索引：默认恢复路径重建 `LoadedProgram`，从不反序列化执行索引、宿主地址或恢复 trusted 标记。cache 命中不恢复已序列化索引、不直接 PASS——重验后仍真实执行。装载索引的编辑会话计数按函数归因（`loaded_index_build:<fn>`），全模块门只看 `axis:function` 键增长。执行表示与预算合同见 [loaded-program-runtime](loaded-program-runtime.md)。

## PLAN-826：紧凑 wire 世代与恢复路由

- A09 单元键新增 `abc-codec` 维度（`abc_unit_key_with_codec`，取值 = wire schema）；AVAB 与 AVAC/1 两世代键空间不相交，跨世代查找为确定性 miss/rebuild（O(1) magic+version 世代闸门按策略判）。
- `ModuleManifest` 新增 `abc_codec` 字段（serde 默认 = 旧 AVAB schema，旧 manifest 恢复行为不变；不入 fingerprint——codec 差异已由 payload_fp 区分）。restore 按 manifest codec 路由解码/准入，未知 codec 拒绝；恢复不继承授权，current grants 必检。
- 新旧 wire 均不恢复 trusted；每一层 envelope→decode→verify 重验。cache 命中不恢复序列化执行索引或 PASS。

## PLAN-866：在线重载的预算与计数口径

- 在线换版不新增任何资源上限：沿用既有 AV 预算（执行分区 slot 8MiB/受管
  32MiB/累计步数 1M/wall 按段 2s）与 825 账本扣除（新 resolution 的驻留索引
  字节计入段预算）；旧代回收上界 = 828 内容史 HISTORY_CAP=32/函数 + max_frames
  在途代码对象面，retired snapshot Weak 释放为 oracle。
- 周转口径沿用 TestCase v2（P90≤2000ms 硬门，样本不缩）；常驻在线的生效点
  计量 = 编辑入队 → 安全点生效（SafetyPointReport.visibility 样本）。结构
  计数（未改函数生产=0、whole-module=0、chain_entries=1、安装集合、代际）
  由父侧 judge 独立判定，不采信 worker 值言。
