# CA-D正式立项：基线、拆包与独立实施顺序（2026-11-05）

本轮用户“OK，那么做CA-D的计划”，承前只规划。已读AGENTS、Spec总览、auto-plan-new/verification-contract、auto-abi三件、A2X资源/增量合同、A2C scalar/record-text/collections现状及真实代码。未建实施树/材料scope、未构建/测试/认证、未启动goal/chat/agent，未接管929。以下code/receipt核查是只读事实，不是新调用链PASS。

## 1. 决定：三包，顺序937→938→939

| 正式计划 | 状态/任务数 | 独立交付与关闭标准 | 实际前置 |
|---|---|---|---|
| [937 CA-D1](../937-shared-foreign-interface-and-scalar-abi-bridge.md) | drafting r1，0/12 | backend-free接口事实/物理投影、797真实native consumer + 新A2C标量source exporter，独立C caller/peer与原三腿、桥artifact增量/恢复 | 797/921已交付；可独立于929/934主体推进，H和正式采样短窗仍单写 |
| [938 CA-D2](../938-checked-source-foreign-scalar-calls.md) | drafting r1，0/13 | 显式typed foreign声明真正驱动普通source编译，native直接call/C HIR薄call、精确依赖、全函数增量/无预热source恢复首编辑 | D937；公共source取929稳定接口，最终交付须D929；934若已接管则同facts权威/required页接口 |
| [939 CA-D3](../939-foreign-resource-handle-and-callback-bridge.md) | drafting r1，0/14 | buffer/record/multiout、原方handle、同步callback+ctx、单源资源/失败清理/lease、source增量恢复/独立认证 | D937+D938；925/924资源/闭包基线已交付；私有oracle可先准备 |

不是一个大包：937能在不改source/HIR/lowering/session恢复的前提交真正可用桥，938有独立高风险语义/恢复接线，939有独立所有权/多out/回调状态机；三者各有独立AC/V/T/SD与review/merge。也不进一步拆成“先无人使用schema、以后再找consumer”。937完成不代表源import，938完成不代表资源桥；三个D齐后才可核销CA-D。CA-E/ARCH最终组合门仍未立项，不预占号，不能让它接漏掉的功能验收。

**可并行结论**：937是当前可独立开发的新方向，可与929收尾/934查询主体并行；需要共同workspace/lock/注册/父协议的步骤按文件SHA短窗，正式慢测错峰。938/939现阶段可准备独立header/oracle/边界调查，整包主链有上述D门。938不无条件等待935/936整波，但若其实际使用实例/CT事实就消费真实checked版本/对应接口，不绕过；source-query、core、shared lowering、H的写面不并写。

## 2. 实际状态/收据核对

只读main `122725e9a`，本轮Git ancestry命令返回值0=当前祖先、1=非祖先；归档不以current_step残留重新打开。

| 基线 | 当前查得 | 对CA-D的含义 |
|---|---|---|
| 797 | archived9/9；历史D1799f86ca ancestry=1；当前abi已纳ACE单workspace | 按现Spec/源码hash重验，旧PASS不能复制；candidate/descriptor3继续原支持面 |
| 921 | archived13/14簿记残留，merge/cleaned已交付；D3c760865c ancestry=0 | TargetProvider/C scalar/完整文件集是真底座，不能当未交付 |
| 925 | archived14/14，Da37245cd6 ancestry=0 | C record/text/modes/last-consumer/owned cleanup，供939；原S缺口修复读当前Spec |
| 926 | archived14/14，D3fce61b9a ancestry=0 | C collections共享资源事实已交，不等整包；其内部List/Variant不变publicABI |
| 924 | archived，D48c2d8066 ancestry=0，计数残留不重开 | native closure底座可复用，不推断任意持久callback |
| 928/931/932/933 | D6b429502f/84c894b92/da4c98b51/44d64c4ed均ancestry=0并已归档交付 | 旧“等待这些包/缺904独立manifest/sema附件”已过时；Rust print/2及929首编辑余账保留 |
| 929 | executing r2，10/14；唯一lang-929树bf8c189c0 | 934会话已有用户授权收尾T-11..14（当前提交收据）；本会话不跑竞争门/改其代码 |
| 934 | header drafting r2，1/13，但已有goal执行授权/私有准备，当前为929收尾 | 不能把header误读为未执行；不要求再做已耗尽的私有准备 |
| 935/936 | drafting r2，0/12、0/14 | 查询端口稳定后有自己的D门/窗口；CA-D不改其合同或取消真实AV CT禁FFI |

797当前独立C工具为cl.exe（env→vswhere→最高可用版本），缺工具infra；`abi/ctool.rs`原flags为`-c -nologo -GS- -Zl -W4 -WX -O2`且只产obj，后续复用core链接/受管执行。A2C已有CRT运行时认证flags/生命周期与freestanding ABI probe不同：新桥需按完整生成物选实际工具/CRT服务闭包并严格入key/证书，不能抄一个flags字段就认为兼容；不为匹配旧probe剪掉生成业务或忽略CRT。937 T-02有界冻结真组合；无工具/原60/5/70预算不合不得假绿或默认放宽。

## 3. 真实机制差额和边界

| 当前代码观察 | 新包负责 | 不能冒认 |
|---|---|---|
| abi/decl.rs::BodyKind为固定probe业务，emit真实CLIF机器码 | 937保真实consumer/兼容模型投影；938把ordinary source绑定到foreign事实 | 声明IR三腿等于source FFI |
| core/hir.rs::DefKind只有Function/Method/Intrinsic | 938有界设计checked foreign DefRef/facts/调用表达 | 用假本地body或绕过verify后端插裸call |
| a2x-core::ExternalFn为兄弟signature-only脚手架 | 938独立区分foreign来源/物理契约/权限/实际依赖 | signature-only=foreign安全绑定 |
| native/lower.rs拒MirInstr::HostCall；emit私有bool I32/Str(ptr,I32len) | 938共享MIR新受检foreign调用与native直call；939资源转换 | console HostCall=通用FFI；私有ABI=AutoABI |
| 891宽typed浮点用整数位载体，公开ABI f64是浮点lane | 937typed物理转换，f32公开仍拒 | 原宽入口=公开ABI支持，任意rawbit直接复用 |
| a2x-c只有a2x-core/core依赖；函数返回私有a2x_result | 937中立窄contract/薄export；938/939受检import/resource | 为借descriptor引Cranelift；私有result/struct即公开wire |
| 925/931资源最后消费者、core BodyResourcePlan是权威 | 939借用/owned/opaque/multiout/callback映射消费单源事实 | 新后端再实现分析、幂等free掩盖双destroy |

统一架构已更新[Design41 §12](../../design/41-interface-metadata-and-foreign-abi.md#12-ca-d正式拆包与可独立交付边界2026-11-05target-state)。native编译source继续Checked HIR→共享MIR→CLIF；A2X从HIR消费；797有限probe发射不冒第二套生产source lowering。普通源i32/bool与typed i64/u64/f64/char/数组视图分别标身份；publicf32、按值聚合/varargs/union/保存跨线程callback/unwind/loader不扩。

四端普通业务各对独立oracle；新增有限foreign必须native+A2C真方向/真source证据，AV/A2R是否支持分别陈述，不自动豁免既定required行。M1完整small-language架构的既定门保持；M2当前应用host/全部A2X目标接管与旧引擎disable仍是v0.6发布硬门，不等新通用生态铺完。

## 4. 写面/交接顺序

| 写面 | 当前owner/新增触面 | 释放/接收与准入 |
|---|---|---|
| 独立ABI底座 | 937 foreign-contract、abi纯模型/bridge、独立C参考及C新薄adapter模块 | 不改source、MIR、a2x session/store恢复；797所有reader/re-export/canonical回归，937可先启动专属实现（需用户后续指令） |
| source/HIR/semantic/restore | 929收尾→934权威query事实；938/939后接foreign/资源事实 | 稳定SHA+文件/符号/required页producer-reader清单+兼容门释放，不能只以能解冲突并写；935/936同样受该窗 |
| shared MIR/native | 938受检foreign-call新schema/所有穷尽consumer；939资源/closure生命周期 | 一次版本迁移全读者，旧AV/console不回归，新未支持op提前拒；不让native经C转译 |
| C provider/A2X store | 937只新薄模块/lib注册短窗；938/939按HIR事实与函数key/恢复扩展 | C backend无process/store，host组合根做binding/cert；普通Rust/C emit不受隐式FFI影响 |
| H | workspace/lock、worker/protocol/decoder/SDK、TestCase registry、fixed组、fixture/cert reader | 929/934/937..939以实际文件SHA交接；全文件单写、释放即让窗，不能占窗等待D门；新recipe/严格读者全核 |
| 正式采样/重内存/独立C认证 | 929当前T-11优先，后续按owner排程 | 原规模/平台/ignored/限额不减，单实例错峰；不因可并行编码就同机争样本 |

## 5. 验收共同底线与首例

- 各包都有首例文件化AutoDown/TestCase、独立手写C header/物理原型表/provider/caller、手算期望、真实callee计数；生成器与worker不读expect，actual/reference分别等oracle。
- wrong domain/target/packing/signature/semref/权限/runtime/provider在call前拒且计数0；不调用已知不兼容函数制造UB；合法非零ctx/混合栈参/len0控制不能被全拒。
- provider实现变化与接口/布局变化分层：不读取body的caller/adapter P0，实际binding/link更新新实现；完整认证身份含真实库/头/工具/搜索适用域，兼容声明hash不替本次准入。
- 937验桥输入层artifact恢复，938/939必须验真实source R深验→无预热首编辑/首错修复→再恢复，parse/solve/resource/lower/emit/tool都装真实P探针，不隐藏全源生产到R/M。
- 原797 six scenarios×30、512修订/GC、build60s/run5s/total70s、IPC4MiB/stream1MiB、hot8MiB+暂存索引8MiB、单产物64MiB/请求128MiB；source/A2X各自原更严门AND，P90≤2000ms，父完整判定落盘窗口。daily不spawn，正式certify真实编译/链接/运行。
- 文件注册/模块编译不等于真实认证；缺工具/坏协议/资源/普通策略拒分别如实终态；NA/accepted-diff/原分母/expected与批准hash不为新门凑绿而更改。

## 6. 本轮只读取材的内容指纹

SHA256记录的是当前源内容，不是本轮执行PASS；实现T-01若漂移重核实际内容与consumer，不按历史hash强行倒回。

| 路径 | SHA256 |
|---|---|
| `experimental/ace/crates/abi/src/descriptor.rs` | `721dcf16b2b56d50787591f6d5d39f26a5499fd7665e647c7e7a2a1259767c71` |
| `experimental/ace/crates/abi/src/decl.rs` | `35ab0a5139b22dd091f7864c5f2850183ebf89bed845de94347c2fa9c61e176f` |
| `experimental/ace/crates/abi/src/emit.rs` | `fce0593133936da7de0ceb433a6af4abcb75cad41294799ebe8493e6bfd67eb2` |
| `experimental/ace/crates/abi/src/cache.rs` | `1abe1057fd7b17118c8c440b7fd8caf6febf49a843123007d7389f0f66a34be0` |
| `experimental/ace/crates/abi/src/ctool.rs` | `441f4a469455acdc451ef52c55824a814385a16db99d535b5e3bc52f337cbbfa` |
| `experimental/ace/crates/core/src/hir.rs` | `07df1d6114b3a3efbf79692338075ecc4f3a96fa326d6b7823ddf73ed16eda3e` |
| `experimental/ace/crates/native/src/emit.rs` | `e59c0a1304c9ea1c40cd1e2bd7544bdde762b968fc86f71767676e6c97c8a1cf` |
| `experimental/ace/crates/a2x-core/src/target.rs` | `f697979db01af0cb47213322e1bee4cbc980d2c30686e858e472460ba055d9b9` |
| `experimental/ace/crates/a2x-c/src/provider.rs` | `25360ae899bb74d836bfd4361e1ac78babdb3156970a649f052b46334b2ad182` |
| `docs/specs/auto-abi/windows-x64-v01.md` | `97fa28d356759e1aeb6eaed631ff3896dc4dc5affd4ea829d3dde21dd8588c5f` |

## 7. 静态规划验收记录

2026-11-05 stage:new / outcome:pass（**仅静态规划就绪**）：三包各0..10节齐，39项T/27项AC/30项V/21项SD编号唯一且交叉引用有效；任务owner_stage及review/merge边界、全部待执行标记一致。正式命令列出的现存固定组逐项对当前test-groups.toml核实，新组与新Cargo test注册均明确为未来H任务；所有新增计划/报告本地链接可解析，10项代码/Spec SHA256与读时内容一致。937/938/939在active+archive+历史archived目录唯一，中央next-id=940；独占取号锁已释放。git diff --check通过，tracked改动仅design/strategy/plans，current Specs/实现零修改，原三项无关untracked未处置。没有新建worktree/材料scope或执行产品构建/ABI/性能/认证/独立实施review；静态通过不替代这些未来证据。
