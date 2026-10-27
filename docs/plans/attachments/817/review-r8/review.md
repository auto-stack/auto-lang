# PLAN-817 R8 final review — needs_fix

2026-10-24；Plan r3；review_scope=final；当前独立于实施的Codex审查上下文。同树D:/autostack/.wt/lang-817/auto-lang，plan-817-dev，冻结实现2552597ff1cc2d5b33edcd5f2dac8fd5b73c939e，base d46e03e8a4986f3d5fed5bd09665c2aeebf0340d，本轮修复基面d640fbce4。开始/结束HEAD未变，clean。按新裁定全部编译/测试通过pwsh执行，只跑恢复改动的受影响面；未跑整档或重新执行GC/规模/周转全档。

## R7-01闭环状态

原“元数据正确的2MiB carrier”例已关闭：probe读SQL大小，在载入前拒绝；新增永久回归和普通restore预留顺序测试均通过。加载暂存按framed+payload预留、decode Hot由1×改2×是实质改善。

但完整不变量未关闭：原关闭oracle明确包括异质/损坏payload及长度漂移时的真实分配上界。本轮probe只读取数据库记录长度，底层仍fs::read完整文件；文件损坏/外部写入导致文件长度与SQL元数据不一致时，校验发生在实际分配之后。见R8-01。没有新增支持面或提高规模要求。

## F-817-R8-01 / P1 / AC-05 / T-05,T-08：数据库长度不能限制真实blob读取

production序列：store::probe_frozen_size读取chunks.size → driver按framed+payload预留 → load_frozen_payload → ManagedStore::get_artifact → read_chunk_gated里的fs::read(path)完整分配实际文件 → hash/frame校验。driver的payload/probe一致性检查位于返回后。内容寻址提供完整性校验，不提供“校验前读取不超过已预留额度”。

**独立、真实分配反例**：corrupt-read-bound.rs只链接冻结release rlib，使用公开store API建立隔离review store，put一个512B载荷（帧560B），probe保持该长度；把此自建store中的CAS文件扩为2MiB，保持SQLite原元数据。按production预留1072B后调用实际load_frozen_payload，全局allocator仅在这次调用期间记录单次最大allocation。

结果corrupt-read-output.txt：

    metadata_framed=560 metadata_payload=512 reserved=1072
    max_single_real_allocation=2097152
    load_result=Err(... CAS address mismatch ...)

即2MiB先真实分配，之后才正确报哈希错误。不是只用事件日志猜测，也没有把损坏载荷当正常CT结果接受。good-read-output.txt同源无文件损坏正控：probe=560/512，reserve1072，max allocation768，Ok(Some512)。只修改本轮review新建的隔离store，未碰用户store或实现树。

关闭oracle：实际文件读取/重组须受已获额度硬约束（含文件与元数据不符及probe/read之间漂移），拒绝在超额读入/分配前发生；保留这个allocator反例、正确元数据2MiB旧例和界内正控，文件增长/截断/错误frame/hash也要受控。仅追加fs::metadata检查但随后无界fs::read仍有漂移窗口，不能证明该不变量。decode的2×依据还需与允许的嵌入结构一起核对；不能以任意倍数替代最坏结构内存依据，本轮未声称另有实测decode OOM。

## F-817-R8-02 / P2 / AC-03,04 / T-05,T-08：probe把所有SQL错误吞成clean miss

新增probe_frozen_size以query_row(...).map_err(...).ok()得到Option<u64>，导致构造的CteAdmissionError被直接丢弃。除合法无记录外，数据库busy、I/O、类型转换、schema损坏也全部Ok(None)，driver会走真实重算/发布路径；这不是规定的infra受控拒绝。

独立反控：在本轮另一个新建隔离store中先存/读正控，然后仅为测试把chunks表rename为chunks_saved；调用同一probe，应返回“no such table chunks”的Err(Infra)，实际sql-probe-output.txt记录 `sql_error_probe=Ok(None)`。只测试错误传播，不要求产品支持损坏schema；无需修改任何产品store。关闭oracle：只有“确实无行”返回None；SQLite执行/解码等错误保留Err及原因，driver不得把错误当miss开始AV。无行正控、正常命中、代表性DB错误分别断言。

## 本轮定向验证

均在pwsh：
- cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_resources_tests cte_resources_restore_reserve_before_alloc_order -- --exact：1/1通过。
- 同test，cte_resources_oversized_restore_rejected_before_load --exact：1/1通过。
- cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_cache_tests --test cte_freeze_tests -- --test-threads=1：6+7=13通过。
- 本轮两个文件driver.rs/store.rs rustfmt --check通过；独立allocator/SQL探针rustc链接冻结rlib并运行成功，原始输出持久化。

15项正向/已有反控通过不覆盖本轮新失败输入。只运行受影响恢复/缓存/冻结面；没有全量测试，也没有使用其他会话编译结果代替当前冻结代码。新probe源码保存在本目录；编译时用release/deps rlib及对应build/native lib搜索路径，native lib来自既有cargo缓存。所有操作pwsh。

Spec delta冻结SHA256 9992F13182B3E450B360495A208FE92D53D4BF84D7EC993BC5E4E12AA716705F，canonical四目标只读；只读依赖为冻结树内783/796/798/828与lock。没有改实现/canonical Spec/ledger，未合入/归档。

## AC和路由

R6累计deadline与其他已关闭finding沿此前匹配修订证据维持；本轮受影响缓存/坏块普通正控已复验。AC-05恢复完整资源不变量fail/partial；AC-03/04的probe错误传播新增局部缺陷，AC-08/09不能最终pass；其他未变领域证据保留，不把本轮15测试宣称为整体交付通过。merge-owned裸cargo t+tv+消费面门禁pending，最终review后执行。

维持executing6/11，T-05/T-08/T-11开放（T-03已关闭的deadline本轮不重开）；同树work修R8-01及R8-02，下一交接必须包含实际物理读/allocator证据，而非仅SQL size和上层Alloc事件排序。