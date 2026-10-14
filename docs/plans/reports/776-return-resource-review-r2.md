# PLAN-776 最终独立复审 R2（2026-10-14）

**结论：needs_fix。** 部分R1原例已经关闭，完整不变量仍有缺口，并引入setter返回值的UAF回归。
本轮8项finding：5 P1、3 P2；退回 `executing`，原合同revision=1不变。
这已经是连续第二次未通过的最终复审；下一次交接须先给出诊断和路径/状态覆盖调整，不能仅补旧例。

## 1. 基线与证据

| 项 | 值 |
|---|---|
| review_scope | final |
| reviewer_identity / limitations | 当前Codex会话独立于修复实施会话；延续R1审查上下文，无子agent/额外模型独立性声明 |
| reviewed_commit | `6494092de6c33875b5e99f81491bb25815852b38` |
| repair diff base | R1审查提交 `b4183e761199642aa341cfecd6491c1821c9e2e4` |
| original base / merge-base | `5e21561d2c303a02319ff7a29e4607933e09a38f` |
| worktree / branch | `D:/autostack/.wt/lang-776/auto-lang` / `plan-776-dev` |
| 主检出 | `0a0950bc025d1a7d601e3f89f5d4166f091e35c5`；读取此处共享Plan及当前Specs |
| dependency | 组内auto-down detached `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean |
| 输入Plan blob | `3e5c51424e853ddb12b7f76a7cb24bb538c26eed`（本轮写结论前） |
| reviewed SD blob | `debc28e4c80f5e73765846eeb16c495c1e0885a4` |
| Plan/SD/七份canonical hash | [input-hashes.json](../attachments/776/review-r2/input-hashes.json) |
| 实现 cleanliness | 开始、探针/门禁后均clean，HEAD未漂移；复审不写实施代码 |

冻结SD在 [reviewed-spec-delta.md](../attachments/776/review-r2/reviewed-spec-delta.md)。
原始source、命令及输出在 [review-r2](../attachments/776/review-r2/)；
`reproduce.ps1` 编译独立Rust探针，链接实施树库，逐进程执行TCP与受控状态检查。
本轮库 `libauto_lang-65ca4d7fb52714a0.rlib`，SHA256
`F3F5386677E494C1FF65F8869D69F969BE40113DD022C64504235EAF75BA44A6`。
脚本退出成功只表示反例采集完成，expected/actual和wire才是oracle。

R1源码原件未修改。R2重放保留iterator、legacy构造、Internal/Err的原始.at行为；
unit对新producer签名/新allocator字段作必要适配，不把ABI修改后的编译失败当行为通过。
Clone已移除，通过源码/编译与专项确认；无伪造新Clone运行例。

## 2. R1 finding闭合状态

| R1 ID | 旧例结果 | 完整不变量 / repair regression |
|---|---|---|
| F01 数值资源猜测 | 原未声明流的0→JSON、Response构造后的1→JSON通过 | 声明流时仍猜数值；client egress被降成数据，R2-01/05 |
| F02 普通数值误release | 新dispatch is_object门、永久wire跨窗重放通过；专项38/38内含 | 该旧例关闭；setter新ownership错误单独列R2-03，不重新冒称旧例失败 |
| F03 owner同号 | 两相邻owner互拒通过 | 只编码7位，A和A+128同号误取，R2-02 |
| F04 终态泄漏 | 原直接Internal11次、直接除零11次均跨窗live=0 | record内持凭据异常仍live=11，R2-04 |
| F05 用户alias/空声明 | Response=int和空Response均Json通过 | stdlib alias/限定名仍Json，R2-07 |
| F06 访问器 | status读取新快照专项通过 | setter返回份额丢失、bytes未迁移，R2-03/06 |
| F07 载荷耗尽 | local=2^24-1成功、下一次Err且无新增表项，独立控制通过 | 该local边界旧例关闭；owner回绕单独列R2-02 |
| F08 Clone计数 | derive Clone移除；无生产clone调用；编译/专项通过 | 关闭这一旧项；不把计数归零当作所有ownership证明 |
| F09 矩阵/健康/SD | 新增若干真实wire、旧例永久测试；check成功 | External仍未await、生成wire未成功、format仍红、SD存在未批准缩减，R2-08 |

## 3. 当前findings

### F776-R2-01 [P1] 声明不能给普通整数授予iterator身份

- 类型：原合同缺口未闭合 / 未批准workaround。关联R1-01；AC-01/02/03；T-01/02/03/07。
- `http_server.rs:5814`：`handler_declares_stream_return && iterators.contains_key` 仍把普通整数提升为SSE。
  原Plan §2明确“静态返回声明或表命中”都不能证明真实来源；不能把AC缩成“未声明流的数字安全”。
- 正控制 `iterator.log` 的 `fn p() int` 返回0为JSON0。
  只把p声明改成 `~Iter<int>`，保留 `let live=stream(); return 0`，
  `declared-iterator.log`实际得到 `200 text/event-stream`、`data: 42`。
  两输入都真实编译/运行，首个generator活跃id=0；该普通值没有按值资源来源。
- 修复：generator/iterator真实producer在返回边界建立身份，数字不得经声明或表取得身份；
  保留真实generator及旧int源码wire，增加同handler条件返回/普通数字负例。

### F776-R2-02 [P1] 7位owner每128次创建重复，二线owner字段不能救同号查询

- 类型：原合同缺口未闭合。关联R1-03；AC-05；T-01/02/07。
- `http_return_resource.rs:107` 取 `instance_id & 0x7f`，只编码7位。
  解析先按id取B对象再校验该对象owner==B，未校验输入原属A。
- `units.log`：相邻A=1/B=2互拒；保留A活跃，顺序创建并丢弃中间VM，最后VM=129。
  A和129各有一个资源，编码相同；在129解析A值，actual取到129的203/owner-a-plus-128。
  expected跨owner拒绝。无需128并发VM，测试时只保留必要的少数VM。
- 修复：owner/session身份不能复用为仍可到达旧凭据；有限表示耗尽受控拒绝。
  在owner重复边界覆盖旧owner存活/失效和nv/VmRef双载体，不以“现实不可达”豁免AC。

### F776-R2-03 [P1] setter丢返回值的owned stake，跨窗后UAF并终止owner

- 类型：修复引入回归。关联R1-06；AC-02/05/06；T-02/05/06/07。
- status/header/text/html setter都 `drop(stake); task.ram.push_nv(res_nv)`，
  释放receiver份额后裸推返回值，没有建立或转移返回槽份额。
- `units.log`：真实201 status setter返回栈槽rc=0、stake=0；snapshot暂可读，跨窗reap后返回值对象消失。
  expected returned Response具有有效持有份额。
- `setter-life.log`：`let r=http.response_status(http.response(),201)`，
  执行9000次循环后 `return r`。expected201；actual RC canary UAF
  `heap object 2164260864 was freed`，owner线程panic，wire503，server result为execution thread panicked。
- 修复：按Plan419 owned slot协议正确转移/重建份额，覆盖串联、丢弃返回值、保存返回值、
  共享/逃逸及跨回收窗，不靠宽限窗维持可用。

### F776-R2-04 [P1] 只扫顶层凭据，没有清理任务拥有容器内的凭据

- 类型：原合同完整路径未闭合。关联R1-04；AC-02/04/06；T-02/04/05/07。
- `release_task_credentials`（442起）只匹配stack槽本身为HttpReturnResource，
  不终结task拥有的record/list/closure等容器路径；父对象留在heap，子凭据保留引用。
- `record.log` 正控制：Box.response的真实redirect取出，wire302+/box，证明该record传播形态存在。
- `nested-error-life.log`：仅持有 `Box { response: http.response_redirect(...) }`，随后1/0；
  11请求全500，9000步drain后live=11，expected回基线0。
  原直接凭据的 `error-life.log` / `internal-life.log` 都回0，不能替代这一容器出口。
- 修复：按实际task所有权终结其必要容器/捕获子路径，保留已逃逸/shared值；
  对成功/异常/确证取消/关闭分别验证root和nested持有者，不扩成随意扫描清空VM。

### F776-R2-05 [P1] client Response进入HTTP egress被改成JSON裸编号

- 类型：原合同缺口 / 未批准范围缩减。关联R1-01；AC-02/03/06/08；T-01/02/03/07。
- `stdlib.rs::push_handle_result`（8623起）仍返回client int+表，
  本轮删除HTTP出口查表后让真实资源也作为数据返回；修复报告称“client不在边界”与Plan §1冲突。
  原文已明确“client HTTP Response若通过同一生产者/表进入服务返回，也在身份边界内”。
- `client.log`：真实本地上游返回 `201 Created`、body `upstream`；
  handler `return http.get(upstream_url)`。expected保留真实Response status/body；
  actual `200 application/json`、body `2`（裸client句柄）。
- 修复：client Response producer/必要访问器桥也保留按值来源；
  同值普通数字仍为数据，真实client响应保持wire行为和owner引用生命周期。

### F776-R2-06 [P2] response_bytes还在旧int表路径，setter全集没有迁移

- 类型：遗漏子项未关闭。关联R1-06；AC-02/07；T-01/02/06/07。
- `stdlib.rs:6140` 仍 `pop_i64`、改HTTP_RESPONSES、`push_i64`；其他构造已返回object。
- `units.log`使用真实201凭据和有效ListData<i32> [65,66]，给bytes列表保留独立份额以排除列表寿命干扰。
  actual shim返回Ok，但returned_resource=None，original snapshot body仍空；expected返回相同资源、body AB。
- `bytes.log`直接语言输入还遇到Invalid list ID，单独保留为未分诊控制；不把该错误归因于bytes迁移。
  本finding依据有效载荷的shim控制和实际源码，不以失效列表证明修复回归。
- 修复：完成bytes载荷/owner/ownership适配，真实语言字节参数控制与既有二进制访问器roundtrip必须成立。

### F776-R2-07 [P2] alias只是加入用户名集，仍未解析真实来源

- 类型：原合同完整身份解析未闭合。关联R1-05；AC-07/08；T-01/06/07。
- `ApiExtractor`把所有TypeAlias作为空ApiType，但不保留target；分类仍按返回字符串和名集。
  用户Response=int的旧例修了，不代表stdlib别名/限定名按来源解析。
- `units.log`：`type Download = FileResponse; fn p() Download { return http.file_response(...) }`
  endpoint分类Json；`http.FileResponse`也Json。expected在实际绑定stdlib FileResponse后分类File。
- 修复：共享实际声明/导入/alias target解析身份，区分用户同名与stdlib别名，
  三消费者保持能力拒绝矩阵；补stdlib别名/限定名正例、用户同名和导入组合负例。

### F776-R2-08 [P2] 再次声称矩阵关闭，但必需oracle、生成wire与Spec delta仍未通过

- 类型：验收/closeout缺口未关闭。关联R1-09；AC-03/04/07/08/09；T-01/04/06/07/08。
- `http_e2e_plan776_r1_three_future_shapes_same_wire` 的External端点仍
  `delay_async(60)`而非 `.await`，无完成gate/park状态证据；quick()直接return1也未证明Ready Future。
  四资源×三Future×named/closure与JSON完整值域、Failed/缺结果、错kind、重复移交、
  迟到资源/确证取消矩阵没有从38项测试或收据得到完整证明。
- harness改成临时端口，但仍只有stderr打印run错误、sleep端口探活、三次启动重试；
  没有755启动结果/panic传回测试的机制。变更后不能继续声称已复用755有界结果机制。
- 修复交接承认生成File真TCP编译失败，把它归给742；本合同AC-08仍不能因此判pass。
  不要求本计划扩面接管742，需可运行现行支持fixture的实编/wire证据，或如实记其阻塞/未验证面。
- 本轮fmt在 `vm/ffi/mod.rs`仍失败；SD修订宣称跨VM同号不可能、将普通数据保证缩为未声明专属类型、
  改client egress行为、宣称setter全集迁移，均与原合同/实证失配；旧decision未按新表示重写。
- `future-cache.log`的组合调查初次请求即500，重放0；没有成功首回控制，暂不宣称
  是本轮新增“已完成Future重用回归”。保留输入/结果，须在完整传播/async矩阵中分诊。
- 修复：先完整关闭实现不变量，补状态可观察的oracle和生成wire，再更新当前task/AC/SD表。
  既有非目标或预存失败不能替代合同内未证明项，也不允许悄悄改合同revision。

## 4. AC与门禁

| AC | verdict | 当前证据/限制 |
|---|---|---|
| AC-01 | fail | 未声明普通数值旧例绿；声明流普通0仍取活跃iterator（R2-01） |
| AC-02 | fail | record取出/直接常量/legacy专项有正例；setter UAF/client/bytes兼容失败（R2-03/05/06） |
| AC-03 | partial | 真Internal直接Response正例已绿；完整四资源/Future/closure值域仍待证明（R2-01/08） |
| AC-04 | fail | 原直接Failed/Err有绿；nested异常资源滞留，迟到/确证取消完整资源oracle未补（R2-04/08） |
| AC-05 | fail | local耗尽边界过；owner重复误取、setter误释放/UAF（R2-02/03） |
| AC-06 | fail | 1000直接返回/SSE断连绿；nested异常11资源滞留，setter返回值失活（R2-03/04） |
| AC-07 | partial | 755四守卫绿；空Response和用户alias旧例绿；stdlib alias/限定名来源与bytes适配未闭合（R2-06/07） |
| AC-08 | fail | 生成字符串测试46绿；client响应泄漏编号、来源能力失配、生成真TCP未证明（R2-05/07/08） |
| AC-09 | fail | Spec/矩阵声明失配；fmt未清；完整门禁原始结果见下表 |

| 本轮命令/检查 | 实际结果 | 原始证据 |
|---|---|---|
| cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 -j1 | 38/38绿，19.878s | plan776.log |
| cargo t --no-fail-fast | 5299/5300绿，94.596s；donut_sector_code_hit红 | cargo-t.log |
| cargo t plan499_donut_sector_code_hit | 单独1/1绿；不拼成整档pass | t-red-replay.log |
| cargo tv | 162/162绿 | cargo-tv.log |
| cargo th -j1 --no-fail-fast | 116passed、corpora1failed、builder_multipart_wire1timeout，353.332s | cargo-th.log |
| nextest -E test(http_e2e_plan727_builder_multipart_wire) -j1 | 隔离重放1/1绿，18.803s；不撤销整档timeout记录 | th-timeout-replay.log |
| cargo check -p auto-lang -p auto-man | 成功；现有warnings不当作零warnings | cargo-check.log |
| cargo t plan755 | 4/4绿 | plan755.log |
| cargo test -p auto-man --lib api_gen -- --test-threads=1 | 46passed、1ignored | auto-man-api-gen.log |
| rustfmt --check --edition2021 --config skip_children=true 全触面Rust | fail，ffi/mod.rs | rustfmt.log |
| git diff --check original-base..HEAD | 通过 | 工具回执 |

日常红项的本轮actual为donut pointer命中区断言未满足；scoped绿只能说明本轮非稳定重现，
不把它宣称为已证新增回归，也不伪造未跑的base结果。HTTP在册corpora仍失败；
builder_multipart_wire本轮120.036s超时，隔离重放18.803s通过，不能按执行者
“唯一预存红/无超时”口径抹掉，也不把隔离通过拼成整档pass；当前为本基线间歇超时待归因。
不跑tf/taa/tt/tu：无相应触面，tf为主检出单实例批量档；不因needs_fix启动合入/批量回归。
生成真TCP失败的交接记录如实列为未验证，不花新整轮冷编译仅再确认已承认的未完成oracle。

## 5. 第二次失败后的诊断与R3入场要求

| 剩余不变量 | 上轮修复/检查为什么漏掉 | 必须调整的路径/状态与oracle |
|---|---|---|
| 逐值来源不依赖声明/表 | 只重放未声明int，加入声明门重新猜测 | 同handler普通数字/资源分支；declared/undeclared、sync/Future；真实iterator生产点 |
| owner身份不复用 | 只测试相邻两个VM，以128“并发”误述顺序创建边界 | 少量live VM+历史创建；owner循环/失效；两载体同号控制与耗尽拒绝 |
| owned setter返回值 | 只验证即时读取status和普通构造wire | receiver stake→返回stake全过程；串联/持久化/丢弃/跨8192步；所有setter含bytes |
| 终态释放所有资源持有路径 | 扫顶层凭据后只重放旧直接root | record/list/closure等parent-owned路径；成功/Err/取消/关闭，保留escaped/shared对照 |
| HTTP支持面没有缩减 | 把client表面改列非目标，把生成失败归他案后仍声称完成 | client真实响应+同值数据反例；现行生成fixture实编TCP、alias能力矩阵 |

work需在原树先完成并记录上述诊断的实现/测试调整、当前结果和finding closure，
R3审查先核实这些证据确已覆盖完整不变量，再执行最终门禁；不得通过改名、internal pass或revision重置轮次。
Plan revision=1仍绑定原合同；本轮拒绝未经批准的范围缩减，不将其沉淀为canonical规则。
SD-01..07及goal/path metadata维持目标映射，但SD整体不通过；七个修改目标/一个新增目标存在性与GOAL-003/016核验通过。
review不发布canonical、不改ledger、不merge/archive/clean worktree。
