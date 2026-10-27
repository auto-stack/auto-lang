# PLAN-817 R9 final review — needs_fix

2026-10-24；Plan r3；review_scope=final；当前独立于实施的Codex审查上下文。同树D:/autostack/.wt/lang-817/auto-lang，plan-817-dev；冻结提交cb0ba5231b0962e9b371007a5d90f3ba470268be，整体base d46e03e8a4986f3d5fed5bd09665c2aeebf0340d，本轮修复基面2552597ff。起止HEAD未变、树clean。编译/测试均pwsh，只跑恢复/缓存/冻结受影响面；未跑整档或重新执行GC/规模/周转全族。

## R8两项核查

- **F-817-R8-02 SQL错误传播关闭。** OptionalExtension只将无行变None，底层错误保留Err；驱动正控/错误反控通过。独立旧探针适配当前bounded API后重放，sql-output.txt明确Err(Infra)“no such table: chunks”，不是Ok(None)。
- **F-817-R8-01旧2MiB损坏blob反例关闭，但完整预算不变量仍partial。** 新opened-handle metadata+take读取确实拒绝该大blob，独立allocator重放corrupt-output.txt：reserved1072、max allocation768，ResourceExhausted。既有未修改783接口保持原路径，新增两方法为追加式适配。新问题是take限制读取长度，未限制Vec capacity及共存副本，详见R9-01。不是重提SQL/大文件元数据预检缺陷。

## F-817-R9-01 / P1 / AC-05 / T-05,T-08：Read::take没有把实际分配限制在预留内

违反原恢复关闭oracle：真实分配上界须由先行额度覆盖（包括实际瞬态framed buffer与payload copy），不是仅Vec.len<=额度。

新增read_chunk_gated_bounded先Vec::new()，take(max_bytes).read_to_end(&mut bytes)。Read::take只限制读取字节数量；read_to_end的Vec扩容可能大于max_bytes。driver仍只reserve framed_len+payload_len，又将这份**总共存预算**作为reader单个buffer的max_bytes；loader随后还会payload.to_vec。长度上限与容量/共存预算混为一谈。

### 独立真实反例A：近界损坏文件（无需并发竞态）

near-bound-capacity.rs链接冻结release rlib；自建隔离store通过公共put存512B载荷，元数据framed=560/payload=512，production同式reserve=1072。将这个自建CAS文件长度改为1025（小于1072，因此通过新增metadata预检），调用实际load_frozen_payload_bounded(...,1072)，全局allocator只在该loader期间记录最大单次allocation。

near-bound-output.txt：

    metadata_framed=560 metadata_payload=512 reserved=1072
    max_single_real_allocation=2048
    load_result=Err(...CAS address mismatch...)

即后续哈希错误正确，但在之前单次已分配2048B，超过1072B额度。此例不依赖修改SQLite记录、手工Alloc日志或声明被拒之前成功消费了损坏值。

### 正常载荷也少算共存容量

good-output.txt同路径正常512B载荷：实际bounded frame Vec len560/capacity1024；get_artifact_bounded在frame仍存活时创建payload.to_vec512B，因此仅内容缓冲区实际同时占1536B，先行reserve=1072B。probe实际读取的frame.capacity()可独立观测；不是拿Vec.len当真实分配大小。

### 已修正的旧例/对照

bounded-read-replay.rs（从R8原独立探针改为当前bounded API）重放：2MiB损坏文件被读取前拒绝、max allocation768；正常载荷Ok(Some512)；SQL schema错误变Err(Infra)。原R8修复成果保留，不因R9新反例全部重开。

关闭oracle：把read buffer实际capacity上界、临时扩容重叠及payload副本的共存额度分开证明；硬读取cap和同时存活副本不能复用同一份总额度。可采用不会隐式超预算扩容的固定/预定容量有界读，或有可靠依据的保守容量预留；任何分配之前必须有覆盖额度。保留512B正控、1025B近界损坏、2MiB旧损坏、各2^n附近合法/损坏载荷与漂移控制，用allocator峰值/实际capacity证明不超过已授权额度，不能仅检查读入长度或“最大分配<512KiB”。不提高原输入或恢复限制，不把受控哈希错误等同资源安全。

## 验证记录

所有命令pwsh，在既有plan树：
- resources定向四项（corrupt_blob_read_bounded、probe_sql_error_not_miss、restore_reserve_before_alloc_order、oversized_restore_rejected_before_load）各1/1通过。
- cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_cache_tests --test cte_freeze_tests -- --test-threads=1：6+7=13通过。
- 合计17项受影响面通过，不覆盖本轮容量反例。没有整档测试或资源重族重跑。
- driver/store定向rustfmt --check通过。编译资源测试有unused_mut warning：cte_resources_tests.rs:1102的d2不需mut（本次四次定向均报告）；健康项待清，不是R9-01根因。
- 独立rustc probe经pwsh编译，链接本轮生成release rlib，配cargo已用native库目录；实际输出均持久化。本轮没有改实现/依赖；只在审查新建的隔离store变异文件和表，未碰产品store。

## Baseline/Spec/路由

spec-delta-frozen.md SHA256 9992F13182B3E450B360495A208FE92D53D4BF84D7EC993BC5E4E12AA716705F；近界探针源码SHA256 530A3489A3456ADFF4AC03F9E45522E4BE9FDB069D48931E66D2F1A49698812D。读取主检出Plan r3；依赖为冻结树783/796/798/828/lock，本轮783变更仅新增两个bounded API、原接口实现零diff。

AC-05恢复资源不变量仍fail/partial，AC-08/09不可最终pass；此前deadline、SQL分类、大文件物理边界具体修复证据维持。其他未变验收与性能证据只在原修订/输入等价范围保留；本轮不因17测试通过宣称整体交付。merge-owned裸cargo t+tv+probe消费面门禁pending，最终review通过后执行。

维持executing6/11，T-05/T-08/T-11开放；同树work修复R9-01并清unused_mut，再冻结送审。未改实现/canonical Spec/ledger，未合入/归档。