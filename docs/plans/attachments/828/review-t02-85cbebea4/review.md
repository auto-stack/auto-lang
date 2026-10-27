# PLAN-828 r3 / T-02 生产 session 专项核验

**T-02 受影响内容专项核验 PASS，无新增阻塞 finding。** C01..C05/F05b 已测闭合保持；可以继续 T-03。Plan 保持 executing0/11，不自动勾任务、不授整份 reviewed/merge。

本轮按用户显式要求检查已冻结 T02。检查期间主计划§8新增“三批复审”：第一批T02+T03+T04、第二批T05+T06、第三批T07+T08+T09。本收据是 **T02 专项证据，不是第一批包级复审通过**；保留最新安排，接下来T03→T04完成后再在新冻结根做第一批复审，不在两任务之间再要求一次包级复审。初始 [Plan](plan-input.md) 与新增节奏后的 [回写前Plan](plan-before-receipt.md) 均保留，未改原AC/V/预算/profile语义。

受审 `plan-828-dev@85cbebea47dd8c62baac38879fc386ca67fe0b22`，基面 `27aaa34d565b11facab37008d42c6e509cc3e13d`，累计基面 `47894c2e6747c8c06032be7a3d40bf00b90a204d`。实施树 `D:/autostack/.wt/lang-828/auto-lang` 入口/收尾clean。review_scope=final角色显式定向检查，非批级/整份最终验收；reviewer=Codex /root当前审查上下文，未实施本提交及前序T01，无额外agent。只读实施/定向验证及主检出簿记，未修实施/启动其他任务/合入/清树。

[接口决策输入](interface-decision-input.md)、[SD输入](spec-delta-input.md)、[冻结指纹](baseline.json)、[最终复核](final-state.json)。auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean，Cargo未消费该兄弟包；实际依赖manifest/锁文件/配置/库/Spec身份见baseline。保留原始测试选择和退出码，重复集合不累计为唯一总数。

## 生产路径核对

| 面 / 原合同 | 源码及真实消费 | 本轮结论 |
|---|---|---|
| C01-01/05、AC01/02：真单body前端 | warmup保留CheckedEnv并核声明顺序；[prepare_edit:1297](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-av-probe/probe/src/session.rs:1297) 从增量CST仅投影改函，调用 [prepare_candidate_document:455](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:455)。synth Program/stub_item/stubify及synth_stub_walks生产管线删除；临时graft后恢复当前，候选自持新条目 | PASS，真实s.edit窗口恰changed +1 resolve/typecheck/plan/bind，其余函数0；同窗再执行实际旧全stub管线，门红 |
| C01-04、AC02/05：校验不弱化 | 文档prepare明确不授验证凭证；session miss在真实verify_function后lower/admit，hit只复用已受检body/code对，并核接口/上下文及内容版本 | PASS受测子集，undefined/type错误/无返回的真实拒绝、冷编诊断类别及拒绝后继续编辑通过；完整授信旧反控保持 |
| D03/C05、AC04/06：候选隔离与发布 | PreparedEdit自持FnCandidate；closed/session-id/revision先拒绝；[candidate.commit:1592](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-av-probe/probe/src/session.rs:1592) 在link成功后更新env并发布当前根/源/修订；[close:807](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-av-probe/probe/src/session.rs:807) 显式close新环境 | PASS本轮成功/前端拒绝/取消/迟到/foreign/closed子集；完整链接失败及资源出口仍按下述T05/06原owner验收 |
| 来源与未改对象 | scoped发射仅把非目标函数的接口目录/无body脚手架惰性span置0；目标body仍用新source坐标，整编路径仍用真span | 长度/换行变化、跨函数后续编辑、history命中及完整trap func/start/end与冷编一致；旧pin保持旧坐标；未改main HIR/code Arc保持 |
| 旧语言/profile支持 | 沿原source解析/检查/HIR构造；共享Emitter整编分支及Scalar投影 | r1/主session、6个source兼容目标及精确scalar_projection_目标通过；trace结果/events和无授权拒绝通过 |

没有把文档候选视为CheckedFunction，没有从缓存恢复trusted标记；新doc API只让生产消费者在命中与miss之间选择既有准入。冷初始化新增parse/resolve_env是冷路径实际工作，保留环境/候选/工件的真实驻留仍计F09，不据本核验声称满足资源/性能上限。

## 六个独立生产组合控制

[session-controls.rs](session-controls.rs) **6/6**，见 [raw](session-controls-output.txt)。均调用公开生产session接口，不手改内部字段：

1. 真实不同body的A/B同时prepare，root/source/revision/结果仍旧；取消A、提交B恰安装B；之后迟到候选拒绝、同名foreign拒绝，两个会话保持；换另一函数再改helper可继续。未改main的HIR/code同一Arc。
2. 三种合法可投影的坏body（类型错、缺return、undefined local）分别在真实resolve/typecheck/verify阶段拒绝，诊断与独立冷编同类别；旧root/source/revision/代码身份及结果不变，修复后可继续。
3. helper增长/换行/缩短到历史内容，真实reuse_hit；随后main/tail编辑，完整trap span与冷编逐次相等，旧snapshot pin仍给旧trap。
4. 真实session改函生产窗口只一body；另一次在**同一窗口**执行session编辑+旧全stub实际compile，helper=2/2/2/2、其余=1/1/1/1，同门必红。不是只比两个外层自报counter。
5. trace授权对照，分别编辑带intrinsic函数/其caller/另一函数，值与实际trace events逐步同冷编，无授权仍拒绝。
6. 真正改变body的未提交候选遇close：close即撤销无外部owner的旧根，commit受控拒绝，revision不动、run Infra、close幂等。补足新永久prepare-close样本使用no-op编辑的观察边界。

原T01八事务、三publisher、完整准入及r23a九关闭控制在新接线版本全部原样重放通过，不以旧版本PASS覆盖新代码。

## 实测选择和范围

| 定向目标 | 结果 / raw |
|---|---|
| contract / t01r | 21/21、4/4，[输出](contract-output.txt)，hotpath3与transaction8是21中的子集 |
| 独立session组合 | 6/6，[输出](session-controls-output.txt) |
| 原boundary / admission / publisher / transaction / close | 11/11、2/2、3/3、8/8、9/9，[boundary](boundary-output.txt)、[admission](admission-output.txt)、[publisher](publisher-output.txt)、[transaction](transaction-output.txt)、[close](close-output.txt) |
| 主session / r1 / AV core | 17/17、86/86、22/22，[session](session-output.txt)、[core](core-output.txt) |
| ac-core共享Emitter/source兼容 | record_source22/scalar_source7/source_frontend14/source_hir12/text_record_source21/text_source10，全过，[输出](ac-core-source-output.txt)；本轮没有声称跑全crate所有native/CLI目标 |
| Scalar增量投影 | 精确 `--test scalar_projection_` 6/6，[输出](scalar-projection-output.txt) |
| 五修改Rust文件 / diff | rustfmt check skip_children 0、diff-check 0，[格式](fmt-output.txt)、[diff](diff-check-output.txt) |

本轮无净新增warning；原session未用SpanInfo/new_len/context_revision/history_entry与shim旧doc warning保持。未跑整档t/tv/tt/tb/tu/taa/tf、docs_gen、正式性能资源或旧62/798兼容全集，不把工作“全crate”摘要冒充本轮复现。

**探针校正记录**：初版六控制3pass/3fail，[原源码](session-controls-initial.rs)、[原输出](session-controls-initial-output.txt)保留。两例在末声明长度编辑遇合同允许的NoConvergence而错误断言应安装；加入未改sentinel提供收敛点后检查实际快路径。另一例把`var x int`当成已证实可投影的未初始化样本，实际走声明集合非快路径拒绝，不能据此指控语义检查缺失；改成真实可投影undefined-local并同冷编诊断对照。未记实现finding、未降低有效断言。Scalar投影最初用名称filter只选5项（另有0项目标），保留[选择输出](scalar-projection-filter-output.txt)，改用精确目标实际跑满6项；不以空选择计PASS。

复跑cwd=实施树，先以Cargo元数据绑定精确库：[runner](run-controls.ps1)、[库哈希](linked-libraries.json)。

    cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_contract --test function_incremental_828_contract_t01r -- --test-threads=1 --nocapture
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t02-85cbebea4/run-controls.ps1 -Source session-controls.rs -Filter t02_
    cargo test --locked --manifest-path experimental/ac-syntax/Cargo.toml -p ac-syntax-core-adapter --test scalar_projection_ -- --test-threads=1
    cargo test --locked --manifest-path experimental/ac-core/Cargo.toml --test source_frontend --test source_hir --test scalar_source --test text_source --test record_source --test text_record_source -- --test-threads=1

## 保留边界与下一步

本PASS覆盖T02前端生产接线及受影响的成功/拒绝/来源/寿命子集。**不授完整事务所有失败出口PASS**：当前commit仍有origin/history-map在link前更新的既有顺序，实际link失败/安装失败的元数据不变及恢复必须在原C02-04/T05/06真实失败分支完成，不能用本次guard/前端拒绝通过替代。该面此前已登记由T05/06交付，本轮未触发或重新分类为已完成，也不授权延期/削弱；后续新改动须按影响重验。新环境body的内存实测和store/lease/GC仍F09/T07等，关闭Weak仅证明其已测根/HIR/code拥有关系，不代表全部内存预算通过。

SD01/02/05的本次生产前端子轴与SD04受影响寿命/候选子集通过，SD01..06仍待最终整体实施/规范审核；冻结SD SHA256 `371a9298c285eb8a0d7309b77298ce9bdadb932e626cc82b7df3af7be3759623`，canonical/ledger、r3语义/owner不改。

按[当前GLM指示](glm-next-round.md)继续T03→T04；第一批包级复审待两者完成后新冻结提交，不能拿本收据当第一批已过并推进第二批。F01后续真实查询/A07轴、F02、F09/F10剩余/F11剩余/F12/F14仍OPEN。整份AC09最终验收、merge-owned裸t+tv/必要consumer/触面门及Spec/ledger/归档/guard仍pending；未启动实施/发送GLM消息/合入/清树。
