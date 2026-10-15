# PLAN-783 Spec 增量草案(T-07,merge 拟沉淀;本轮不编辑 canonical Specs)

对应计划 §5"规范增量"表;SD-01..05 逐条落到拟写入内容。**全部为
experimental 探针事实**,不扩大语言面,不改变任何生产入口。

## SD-01 add `docs/specs/auto-ac/artifact-contract.md`

来源:`docs/reports/plan783/contract.md` + `schema.rs` 登记表。要点:
A00–A12 全层登记(schema/键成分/引用类别/驻留/再生);实际 codec 面
=A00/A04/A07/A09/A10(core+scalar 受限可执行子集);身份三分
(QueryKey/ArtifactId/ChunkId)+ 域分离 blake3 + 路径不入内容键;
恢复 HIR 不带信任、一律重过真 verifier;对象键以 HIR 查询键哈希为
成分;provenance 与 required-data 分离(GC 只沿 required 保活);
同键异内容=不确定性报告。

## SD-02 add `docs/specs/auto-ac/artifact-probe.md`

来源:README + feasibility.md + perf-report.md。要点:探针位置/依赖/
运行面(`verify-ac-artifact-probe.ps1` 两档,closeout 恒等校验+新临时
perf 产物校验成功后发布并做发布后哈希验证,IO 失败传播为门禁失败);冻结
预算(64KiB 页、≥128MiB/布局、8MiB 热、2 页暂存、512KiB 查询、GC
64/48MiB、**8MiB 临时空间=staging+CAS `.tmp` 双副本之和——跨进程独占
预算锁横跨 staging 写入至 CAS 放置的原子预留(R3-01/R4-02),并发发布
在配额窗口内串行,PEAK_TRANSIENT 只记成功预留**);发布协议=**单临界区**
(共享 liveness 覆盖 staging 分配→预算→CAS 投放→元数据提交;GC/恢复
全程独占 liveness);GC/恢复候选=**完整状态矩阵**(无行/deleted-tombstoned
行重现块/无 artifact 的 live 行,R4-01);支持面=core/scalar 四 oracle +
scalar 兼容控制;限制清单(文件/锁级 reader 证明≠DLL 安全、无 pack、
造数布局≠通用 CST、无亚秒承诺、链接支配对象命中路径 ~150-200ms、
NTFS 新写块可见性滞后以 settled_space 沉降测量应对)。

## SD-03 modify `docs/specs/auto-ac/project.md`

新增一节"独立产物契约探针导航":指向 SD-01/02 两份新 spec 与
`experimental/ac-artifact-probe/`;显式声明探针不改变 ac-core 语言面、
不新增生产入口(ac-core 零产品依赖由 dep_audit 永久断言)。

## SD-04 modify `docs/specs/auto-cache/project.md`

"现状差距与实验/迁移边界"节,实测差距清单:
1. `BlobStore::put` 固定 tmp 名:同哈希并发写共享 tmp 路径——结构性风险
   (R1 F-08 反例:无外层协调时竞争失败);探针以 per-chunk LockFileEx
   有界等待串行化规避并实测**重叠双进程**同键(同内容/异内容)行为。
   **探针证据不构成生产 BlobStore 原生并发安全的证明**——生产化需修复
   或上移该串行化。
2. 元数据删除 vs 物理释放:现有 GC 两条入口按记录大小计,无 lease/图保活/
   崩溃恢复/物理释放收据;探针实现了 tombstone→门锁→trash→删除 +
   级联删行 + 幂等恢复(含已提交孤儿行与 trash 残留的恢复趟,R1 F-06),
   作为迁移目标形态的参考实现(非生产代码)。探针 store 以
   `auto_vacuum=INCREMENTAL`+checkpoint 使 SQLite 主文件在回收后**真实
   收缩**(R2/R3 轮修复),GC 收敛决策用**沉降后的物理总量**
   (settled_space:orphan/tmp/trash 全计,R3-02)——原 64/48MiB 水位
   总量合同在 512-churn 上实测达成。
3. `fingerprint::compute_content_hash` 直哈原源码——各阶段细粒度键
   (profile/caps/target/backend 成分化)在探针 identity.rs 有完整参照。
4. 迁移边界:探针不并入任何生产入口;退役/迁移由后续计划决策,防第三套
   生产缓存(计划 §2 红线)。

## SD-05 modify `docs/specs/auto-hir/stage-contract.md`

补三条:A07 持久 payload 不携带信任(磁盘 trusted 标志不能构造
CheckedModule——探针恢复路径重过 verify 的实测依据);恢复重验义务
(任何持久 HIR 消费前必须经本域 verify 入口);provenance 非强 pin
(对象与其 HIR 输入哈希的关联是溯源边,不保活上游)。
