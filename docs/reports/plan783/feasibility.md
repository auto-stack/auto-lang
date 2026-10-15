# PLAN-783 可行性收据(T-01 冻结决策 + 实测;R1 修复版)

日期:2026-10-15(work 阶段,基面 `ec934b000`,分支 `plan-783-dev`;R1 复审修复后更新)。
探针:`experimental/ac-artifact-probe/`(独立 Cargo workspace,`[workspace]` 空表)。
本报告为 T-01 冻结决策的落地收据;实测数字见 [perf-report.md](perf-report.md)。

## 1. 冻结决策(T-01)

| 决策点 | 冻结结果 | 依据/实测 |
|---|---|---|
| 依赖 | `auto-ac-prototype`(path ../ac-core)、`auto-cache`(path ../../crates/auto-cache,仅隔离适配 `BlobStore`)、`blake3 1.5`、`rusqlite 0.30 bundled`、`serde/serde_json 1`、`windows-sys 0.59` | 全部仓内已用版本,离线解析一次通过;独立 `Cargo.lock` 已生成并提交 |
| 块层 | `auto-cache::storage::BlobStore` 经隔离适配复用:同 chunk 跨进程写由 `locks/chunk-<id>.lock`(LockFileEx,**有界等待** 10s)串行——固定 tmp 名风险的存在面是结构性的(同哈希写共享同一 `*.tmp` 路径);探针以锁串行化规避,**未证明生产 BlobStore 本身并发安全**(其修复/上移是 SD-04 差距项);竞态面在探针内 0 复现(串行化后) | V-07 + R1 F-08 反例回归 |
| 元数据层 | SQLite(WAL + busy_timeout 8s);artifacts/chunks/edges/roots/leases/txns 六表;发布=**共享 liveness 锁内**单事务;GC/恢复=**独占 liveness 锁**全阶段 | 三故障点恢复幂等(V-08)+ lease/发布与 GC 互锁(R1 F-01) |
| HIR codec | 二进制定长标签格式(v1):core+scalar 受限可执行子集;解码全索引边界校验;chunk 框架(48B 头+blake3)自带校验;**读取时 CAS 地址复验**(框架字节哈希必须等于请求地址) | 14 项 lib 单测 + V-02 + R1 F-04 |
| 布局 | A=32B 定长记录表+payload 独立页流(算术索引);B=变长记录+10B 目录项分页;页 64KiB 内容寻址,版本=页清单,未改页逐块共享;**读取路径完整准入**(框架+地址+字段界) | V-03(≥128MiB/布局实测写入)+ R1 F-05 |
| 发布协议 | **单临界区**(R2-01/03/R3-01):共享 liveness 覆盖 staging 分配→**跨进程独占预算锁内的原子预算预留(measure→check→write,8MiB,含 48B 框架;PEAK_STAGING 记锁内真实并发合计)**→chunk 锁(有界等待)→CAS 投放 →[F2]→SQLite 单事务(chunk 状态 upsert 回 live;required 边目标存活校验);GC tombstone →[F3]→独占门锁→trash→删除+级联删行;GC/恢复全程独占 liveness 且入口共拿 namespace 根 reparse 门(R3-03) | `AC_PROBE_FAULT` 注入;R1/R2/R3 反例回归 |
| 子进程 | 60s 默认截止(测试经 `AC_PROBE_RUN_DEADLINE_MS` 收紧);Job Object 树杀;管道并发排干 | `subproc.rs`;挂死 exe 用例 3s 内受控失败 |
| 目标面 | 仅 `x86_64-pc-windows-msvc`(宿主);其余受控拒绝且零产物 | V-05 |

## 2. 最小纵线可行性结论(T-01 门槛)

四条 core oracle fixture(p01=14/p04=3/p12=0/p13=70)全部通过:

1. **fresh**:真源码 → `compile_parts_with` → 真 verify → 真 `lower_object` → 真 `link_object_staged` → 运行,exit 逐条相符,stdout/stderr 空;**诊断绑定当前工程源路径**(R1 F-11)。
2. **对象持久化 + 新进程只链接执行**:对象 chunk 落盘后,新进程(且**源文件已被移除**——禁调失败注入)仅 link+run 即复现 exit;parse/bind/verify/lower 计数全 0(V-04);被替换的对象块在链接前被 CAS 准入拒绝(R1 F-04)。
3. **二进制 HIR 恢复 → 重 verify → native**:purge 对象层后跨进程恢复二进制 Bundle(`hir_restore=1`),真 verify 通过后重新 lower/link/run,payload 无 `hir(`/`module(` 文本标记——非文本 roundtrip(V-04 + lib 单测)。
4. **scalar 兼容控制**:同一 core∩scalar 面源码在 scalar profile 下全层 miss 重建、语义一致(V-05)。

结论:**二进制 HIR 恢复/公开 API 可行性成立**;块层并发前提在探针的串行化协调下成立(生产 BlobStore 的原生并发安全未证明,见 SD-04)。T-01 有界可行性门槛通过,无反例需要回规划。

## 3. 关键实测发现(work 阶段 + R1 修复)

- **liveness 互锁是必需品(非 TTL 可替代)**:R1 F-01 实证——快照后取得的 lease/hard root 会被 GC 删除;修复为发布/注册(共享锁)与 GC/恢复(独占锁)全阶段互斥 + 注册时最终存活校验(`TargetMissing` 受控拒绝)。
- **CAS 准入校验必须在读取路径**:自洽但异址的合法块可整体替换目标块(R1 F-04);修复为读取时复验框架字节身份。
- **重发布必须恢复 chunk 状态**:`INSERT OR IGNORE` 让 GC 删除后的同内容重发布永久 miss(R1 F-03);修复为 upsert 回 live + 级联删行。
- **恢复要覆盖"已提交孤儿行"与 trash**:冲突路径提交的无主 chunk 行不被 artifact 驱动的 GC 看见;阻塞删除的 trash 残留需恢复趟重试(R1 F-06)。
- **GC 背富地板不可计入瞬态 WAL**:否则写活跃期虚假退让;地板=主 DB 文件+锁+日志+活块,WAL/SHM 只入账不计地板(F-07 修复)。
- **A10 回执内容必须是链接输入的纯函数**:provenance 走图边(metadata),回执=对象 chunk+target+backend+entry+producer;回执内 chunk id 必须与存储 CAS 同源(框架字节哈希)。
- **摘要内容身份不得含 span**;**hit 消费方必须注册当前版本根**;**LockFileEx 竞争 33/32 都映射为忙**;**对象键以 HIR 查询键哈希为成分**(缺源命中的可计算前提)。

## 4. 限制与后续(不做即声明的边界)

- reader 安全性证明止于**文件/锁级**;不冒称 DLL/callback 运行代际安全(A12 Future)。
- 无 pack/compaction;SQLite 主文件经 auto_vacuum=INCREMENTAL+checkpoint 在回收后**真实收缩**(R2/R3 轮修复),原 64/48MiB 总量合同在 512-churn 实测达成;NTFS 新写块可见性滞后以 settled_space 沉降测量应对(R3-02)。
- 布局人口为**确定性造数**,证明分页/预算/共享机制,不冒充通用 CST。
- 亚秒周转未宣称;实测距离见 perf 报告(P95 意义达标,链接支配收益结构未变)。
- 生产差距与迁移路线(现有两条 GC 入口、fingerprint 粗粒度、BlobStore tmp 名修复上移、无 single-flight 服务化)见 spec-delta(SD-04)。
