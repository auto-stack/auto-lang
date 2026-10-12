# PLAN-738 独立复审 R1（2026-10-09）

结论：**needs_fix**。原 AC-01..08 保持不变，plan_revision=1；本轮只复审、记录和回退进度，不修实现、不合入、不发布 canonical/ledger。
这是实施会话之外的新复审上下文。执行者的勾选、work/pass 与报告只作为待验证声明；结论来自代码、实际入口反例和本轮门禁。

## 基线与证据身份

- reviewed_commit：`e4425b673eec46fabc25b4534084c19f2738a38f`，入口实施 worktree clean。
- worktree：`D:/autostack/.wt/lang-738/auto-lang`，branch=`plan-738-dev`。
- 原 work 记录基线：`4262f761c5f7a0608fbfc5061ad6afa1b31cc574`；re-sync=`0424673e2`。本轮主检出=`9bdd04ffeb1a4178ed7ce754c3ddff54fd60eba2`，两树 merge-base=`6d69dbdc78d99f23ba90a37c9559f7fb54dd7be3`。因此差异同时参考直接两树对比和计划提交链，不能把全部 merge 差异归为 738。
- auto-down：`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（lang-738 兄弟树）；其余依赖/Spec 输入 hash 见 [738-review-baseline.json](738-review-baseline.json)。
- CLI 反例实际使用 `lang-738/auto-lang/target/debug/auto.exe`，SHA256=`c41f5427ae68f052357aa76966e38f627f19079b11ab4eefbc96f39d5e668508`。后续本轮 CLI 单测另列，不能把测试二进制 hash 当作 CLI 文件 hash。
- 临时探针在同一 worktree 运行，未改变被审代码；完整源码冻结在 [738-review-probe.rs](738-review-probe.rs)。探针是观测程序，不是宽松的验收测试。复现方法：复制到 `crates/auto-lang/examples/review738_probe.rs`，在本计划树运行 `cargo run -p auto-lang --example review738_probe`；`-- --quick` 跳过耗时转译两腿。运行后删临时副本。
- 精简原始输出见 [738-review-evidence.txt](738-review-evidence.txt)。临时日志不是唯一证据，以下保留命令、结果和可复现来源。

## 必要修复（都在既定验收面内）

| finding | 级别 | AC / tasks | 实际证据 | 最小修复与复验要求 |
|---|---|---|---|---|
| P738-R1 | P1 | AC-03/05/08；T-03/04 | `validate_core_vm_bindings`/`core_status_diagnostics`/ID 检测只由测试与 inspect 消费；生产 compile、VM codegen、persistent、Rust/C emit 无 strict 校验调用。两活 body（公共 val=1、VM val=2）经真实 `resolve_uses("use proto")` 返回 `Ok(1)`。既有 duplicate 测试的 `Ok` 臂直接放行，因此测试绿不能证明冲突被拒。 | 在生产装载/链接/emit 前消费引用或导入项闭包，拒 missing/conflict/unverified/被引用 unsupported/feature off，未用 unsupported 不毒化整个模块。区分 bodyless+合法补全和两个活实现。native 注册时证明 alias 同 callee，防 ID 冲突先覆盖 shim；P738-D1 的“check 报非零后延期”不能代替生产门。补真入口“不执行业务”的负测。 |
| P738-R2 | P1 | AC-01/03/05；T-02/04 | `SymbolEntry` 只有名字、kind、arity 等，无参数/返回类型、self/static/async/generic/feature/body 及源位置；校验跳过所有非 `#[vm]` 公共声明。`tcp_listener_close` 的 inventory arity 从 1 改成 999，仍 `Supported + SignatureChecked`。`ret_known` 只查注册表是否有返回类别，该类别还可能来自同一磁盘声明，未与独立生产者签名比较。 | 建立六核心公共 identity → 实际 native/VmModule/opaque/host/Auto-body 的映射；真实生产者提供独立逻辑签名、能力、别名与 stub 信息，校验参数/返回/self/static/async/generic 约束及适配。签名未核验不能升 SignatureChecked。接通 io 第四方法绑定面、json canonical 面等既有支持能力，不能把 P738-D2 全推给“公共面全量重写”。不要求本期实现深层资源 ABI parity。 |
| P738-R3 | P1 | AC-04/06；T-05 | 同一 CompileSession 装载后将 old_layer 替换为 new_layer，再 `resolve_uses` 返回 Ok，模块仍只有 old_layer。`load_module` 在任何指纹检查之前按模块名早退。克隆会话缓存 A 的绝对路径，在 CWD B（同名 proto 优先）解析仍选 A。`get_valid` 先于 resolver，key 不含本次 resolved source identity；依赖检查只核直接依赖源段，不递归核其依赖。 | 按编译 epoch 区分“本轮已加载”和“上轮缓存”，先解析实际来源再查缓存；key 纳入规范化来源/root、target/env、provider/features。依赖闭包递归校验并对循环有界。变更须产生新类型/bytecode/manifest，或活 VM 明确要求重建，不能成功返回陈旧内容。补同一会话、CWD/解析根切换和 A→B→C 深依赖反例，不只测 clone 后改层。 |
| P738-R4 | P1 | AC-06/07；T-05/06 | 在同一模块下把 `m.vm.at` 重命名为 `m.rs.at`、内容不变，VM assembly fingerprint 前后均 `b6e5ba81f6cbbcc4`，但实际 VM 选层已消失。指纹只吸收 module+content，不吸收文件/层身份；provider 只取 schema_version，没有目录内容、实际 host/native 实现版本、环境/features/真实依赖闭包。生成收据定位失败 `.ok()`→null，新鲜度 `(None,None)=>true`。 | 指纹绑定实际选择的来源/层/缺层存在性、provider 内容与实现版本、target/env/features 和依赖闭包；schema 版本不是内容版本。receipt 与启动消费者用同一来源策略，不能找不到标准库就当已核验。若合法无 Auto 依赖则以显式“空闭包”记录证明；有依赖但不可核验须错误。补改 host/provider（不 bump schema）、层重命名/增删、环境/feature 和缺根反例。 |
| P738-R5 | P1 | AC-02/04/07；T-03/06/07 | 真实 `trans_rust_with_session` 对 `use auto.net` 返回 Ok，产物路由不存在的 `a2r_std::net`，`session.layer_selections.len()==0`；C 同样 Ok 且 manifest=0。两个入口只是 set target，没有生产 AssemblyPlan/provider 消费；`trans/rust.rs`、`trans/c.rs` 在本计划没有接线改动。persistent 仍单独硬编码 stdlib 根与 `.vm.at`，只扫描注册声明，未共用不可变 manifest。普通 loader、native 扫描、inspect/receipt 的根策略仍不同。 | 在既有 resolver 后生成共同装配计划；VM 常规/persistent、Rust/C emitter 和 API 生成器实际消费并回报选中 provider，不只设置 enum 或输出 candidate。允许 host-mapped 无 sidecar，但必须验证真实 callee。统一来源策略并保留用户模块遮蔽/循环语义；环境/目标改变由受控 API 拒绝活会话热换。不得为了通过而给 C HTTP/Rust net 新造未计划 backend。 |
| P738-R6 | P1 | AC-03/05/07；T-06 | `auto stdlib inspect --module auto.net --target rust --check` exit=0、status=pass；实际目录明确 Rust net unsupported。`auto.async --target vm --check` 也 pass，附带约 40 个全库诊断，其中公共 async 不可解析；check 未把相关解析失败计入结果。`--actual examples/stdlib/assembly/witness.at --check` 只依赖 jsonx，却检查六核心全部并报 72 违规。`check_core` 无 target 参数；actual 的模块过滤把 `sel.module` 与剥 auto 前缀的值直接比较。 | CLI 根据真实 target/environment、resolved identity 和实际请求闭包调用同一 strict 验证器；相关 parse/read 失败必须非零，无关未引用缺口留 inventory。校验 module 参数的存在与归一形（未知模块不得 vacuous pass），实际 JSON 报目标、provider、symbol/source 双位置。actual 不得混入全部目录 claims 冒称实际消费；便携清单映相对 source ID。补上述假绿/假红和 auto 前缀负测。 |
| P738-R7 | P2 | AC-01/05/08；T-02/07 | inventory 不记录公开字段；TypeDecl 强行 `is_pub=true`；非法 UTF-8 的 bad.at 得到 files_total=1、modules=0，仅诊断，未保留层条目。WalkDir 错误被 `filter_map(e.ok())` 丢弃。矩阵 VM 分母用目标层 `#[vm]`，async 公共符号数=0却标 VM 10/10 supported；Rust/C 为模块 claim，缺其 native/browser 两格。Browser 只禁 io/net 与名称含 server/listen 的 http，其余直接复用 native shim Supported，未证明 browser adapter。 | inventory 保留所有文件/失败层、真实公开 fn/method/type/field 和源位置，目录遍历失败也有 diagnostic/partial；区分 isolated-parse 与真实依赖装配结果。六核心以公共身份逐符号 × 三目标 × 两环境分类，不用 VM helper 名或模块级存在性代替。Browser 的 supported 必须有对应适配/能力证据，未知应 Unverified；Vue 前端+Native 后端保持 Native。 |
| P738-R8 | P2 | AC-04；T-03 | 源映射只是 `context_byte_boundary`；错误时再 parse 公共段、用 `AutoError::Msg` 加目标路径。探针确有目标文件名，但只有“unexpected token/Expected term”文字，无该文件的 span/行列。没有合并 span→真实文件偏移映射；fallback parse 还写共享 TypeStore。现有测试只查 message contains 文件名。 | parse/conflict/native diagnostic 用原始 source segment 映射 file+span/line/column，给双源位置；错误归因不能通过可变二次 parse 猜测。补公共前缀长度变化、目标首行/多行错误、公共错误与双实现/签名冲突位置断言。 |
| P738-R9 | P1（验证）/P2（健康） | AC-02/07/08；T-07/08 | Rust witness 只证普通 `Json.parse` 经真实 a2r-std 编译运行，未走 generated API→service→ready。AC-07 的真实生成启动被执行报告“归后续”；C 支持样例实编被拿“无 C HTTP provider”替代，二者不同要求。SD 仍只有起草表、无已核验的最终规范正文。新增 model.rs rustfmt 检查也有 3 处 diff，原“改动文件 fmt 干净”不成立。 | 修复前述项后补同源 VM/Rust/C 支持样例实编执行，保留缺 backend 的负测；generated API→Rust 编译→736 startup/ready→修改 assembly 不改 api.at→拒旧/再生，包含729/730/734兼容。收紧双 body/签名测试，禁止宽松成功分支；完成相关格式检查、警告分诊、最终 SD 正文/版本与逐 AC 证据。阻塞应保留未完成，不能债务化后勾全。 |

执行顺序建议：先 T-02/03 的公共 identity、来源与共同计划，再 T-04 生产 strict 门，随后 T-05/06 缓存指纹及 CLI，最后 T-07/08 真入口证据与规范增量复审。这是原计划内修复，不需改变验收语义或新建“全库重写”计划。若执行时证明设计无法闭合，再按证据 needs_replan，不预先削弱 AC。

P738-D1/D2 的历史记录保留，但涉及六核心的部分由 R1/R2 承接为阻塞项；未承诺的深层 ABI、全库非核心签名/行为 parity、新 backend、完整 actor/TLS/WS 仍属后续。执行报告所记 as_int 下沉与内嵌 runtime 漂移，至少必须在实际选中 provider 的签名/版本核验中暴露，不能让 json 整模块 supported 掩盖坏 callee。

## 验收逐项判断

| ID | 本轮结果 | 已有有效证据 | 未完成/失败与关联 findings |
|---|---|---|---|
| AC-01 | partial | inventory 总文件计数、排序和 parse diagnostics 可用 | 公开字段/源位置/失败层/真实公共分母未闭合：R2/R7 |
| AC-02 | fail | VM 选层运行 witness、persistent context 路径修复、普通 Rust JSON 实编运行 | 共同生产计划/manifest 和 C 实编缺失：R5/R9 |
| AC-03 | fail | resolved+bound 两面检查、目录 ID 冲突诊断可观测 | 独立签名/公共映射/生产门/重复实现仍失败：R1/R2/R6 |
| AC-04 | partial | use 回归及用户同名模块测试在本轮日常门通过 | 双源 span/行列缺失、同名异 root 缓存失效：R3/R8 |
| AC-05 | fail | target/env enum 分开；io/net Browser 负分类存在 | 真实六格公共符号矩阵/适配证明/严格拒绝缺失：R1/R6/R7 |
| AC-06 | fail | clone 后目标内容变更与增删层、内容 hash 检查的现有用例通过 | 同会话、resolver identity、深依赖、层身份/provider/host、fail-open：R3/R4 |
| AC-07 | fail | CLI 可用、生成 receipt 字段与纯 freshness 单测存在 | target/闭包假绿假红、真实 manifest、新鲜度身份、生成启动反例：R4/R5/R6/R9 |
| AC-08 | partial | tv 及既有兼容面回归，未承诺 backend 的拒绝原因有记录 | 等级假升、验收延期未批准、最终 SD 与健康门未闭合：R2/R7/R9 |

## 本轮门禁与分诊

- `cargo t --no-fail-fast`：**5146 run / 5132 pass / 14 fail / 1537 skipped，62.362s**。14 红逐名都在仓内 `.last-batch-regression.json` 的 known_reds_seen（p053×4、p054×2、plan606、projector、desktop_bus、desktop_surface、ash、schema×2、docs_gen）。这是与已有回归收据对照，**未在本轮新建 baseline worktree 重跑同命令**，不冒称新做了双树证明。原 work 报告 fail-fast 截断不足以证明完整门禁。
- `cargo tv`：**162/162，1.863s**。
- `cargo tt --no-fail-fast`：**5517 run / 5503 pass / 13 fail / 1 timeout / 1785 skipped，160.430s**。11 红在已有 known_reds（6 musk、plan606、3 desktop、ash）；plan502 在 P733-R1 已有负载 flake；back_provision 新观察与 dep_fields timeout 分别 scoped 重跑 **1/1 pass（0.096s）**、**1/1 pass（0.914s）**。这只能说明本次非稳定复现，未充分证明原因；不据此增加 738 的确定性修复范围。
- `cargo th --test-threads=1 --no-fail-fast`：**102 run / 100 pass / 2 fail / 6668 skipped，190.847s**。两红为 `http_e2e_back_proxy_real_031_native_ns_session` 和 `http_e2e_back_proxy_real_routes_corpora_data_face`，执行报告已有 master 两红基线；本轮未新做双树复跑，保留环境/基线归因的证据限制。
- `cargo test -p auto-lang --lib rust_host_provider_real_compile_witness -- --ignored`：**1/1 pass，2.41s**；此为普通宿主 JSON witness，不是 generated API/serve witness。
- `cargo test -p auto --bin auto stdlib -- --test-threads=1`：**8/8 pass，0.38s**；`cargo test -p auto-man --lib assembly_freshness_truth_table`：**1/1 pass**。CLI 的既有测试通过仍不能抵消本轮新增反例。
- `git diff --check 4262f761c..HEAD`：pass。`rustfmt --check --edition 2021 --config skip_children=true` 对新增 Rust 文件：**fail，model.rs:106/196/203 三处 diff**。对全部被改文件亦有大量旧格式差异，不要求为了本计划格式化全部巨型旧文件。
- Cargo 构建/门禁编译成功但存在预存警告（auto-lang lib 400、test-trans lib-test 553 等）；不声称全仓零警告。新增装配文件无本轮编译 warning 定位，尚需 repair 阶段完成触面警告分诊。
- 未跑 tf/taa/tu；本计划不触 aavm/UI generator，tf 为合入到期主检出批量档。

## 规范增量冻结与路由

原 `### 规范增量` 的 SD-01..07 表冻结于 [738-review-baseline.json](738-review-baseline.json)（UTF-8 文本+SHA256），输入 current Spec 版本/hash 同文件。路径真实、GOAL-003 存在；新增 `docs/specs/stdlib/design/assembly-manifest.md` 尚不存在，属提案。

全部 SD 本轮 **not approved**：SD-01/02/03/04/06/07 依赖 R1..R8 的实际合同；SD-05 依赖 R4 与 generated 启动证据。保留现有影响路径和 touched_goals，不能用起草 proposal 冒充已核验持久规则。修复后写出具体 add/modify 正文、当前目标版本、对应 AC 及消费证据，再冻结新的 reviewed delta。任何语义契约变化才 bump plan_revision；本轮状态/checkbox/复审记录变化不 bump。

next：`/auto-plan:work PLAN-738` 按 P738-R1..R9 修复并补证 → 新 commit 的 `/auto-plan:review`。合入与归档保持禁止，非核心后续项不替代当前失败验收。
