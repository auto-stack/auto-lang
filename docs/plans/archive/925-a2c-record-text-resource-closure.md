---
plan_id: PLAN-925
status: archived
feature_name: CA-B HIR到C的record/text参数模式与资源闭环
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-a2x/checked-resource-facts.md, docs/specs/auto-a2x/incremental-and-cache.md, docs/specs/auto-a2x/architecture.md]
new_spec_components: [docs/specs/auto-a2c/record-text-hir-backend.md, docs/specs/auto-test-case/a2c-hir-record-text-certification.md]
touched_goals: [GOAL-002]
affects: [auto-a2c, auto-a2x, auto-test-case]
depends_on: [PLAN-881, PLAN-894, PLAN-902, PLAN-904, PLAN-921]
current_step: 11
total_steps: 14
---

# [PLAN-925] CA-B HIR到C的record/text参数模式与资源闭环

## 0. 变更摘要

CA-B正式立项：消费受检HIR/验证器资源与参数事实，在921共享TargetProvider/session上增加C record/text/text-record；带static/view方法、view/mut/move、显式clone/深clone/逐出口清理、真函数/方法暖编辑、来源恢复后下一编辑及独立认证。依据[Design51 §14](../design/51-shared-a2x-and-hir-to-rust.md#14-ca-bc-recordtext资源与后继abi接缝2026-11-03target-state)、[Design41 §11](../design/41-interface-metadata-and-foreign-abi.md#11-a2c统一abi与ffi的近期结构2026-11-03target-state)与[C路线](../design/strategy/shared-a2x-and-hir-to-c-roadmap.md)。

drafting r1、0/14；本轮只规划，不施工/建树/测试/current Spec或原source/expected/批准/登记变更。921实际在途，分支代码不是D921；最终集成必须含其正式交付基面，私有设计/fixture/reference准备可提前，不改其合同/进度。

## 1. 目标

G1：原902冻结源及全部相关模式/方法/资源正反例经新C provider生成完整C11文件集，actual/reference各真实编译运行对独立oracle。G2：唯一消费CheckedFunction/BodyResourcePlan/排他/借用事实，不重做ownership分析；源序、模式、writeback/返回与清理精确一致。G3：支持自由函数及static/view方法body暖编辑，未改真实P/IO0/对象代际保持；纯来源恢复P0/R>0且下一编辑不隐藏整源重生产。G4：目标身份/独立认证/原资源时间门完备，给CA-D稳定边界，不把私有C结构当公开ABI。

非目标：CA-C List/Variant、闭包/HOF/K06、mut实例方法/ext/trait/任意重载、Unit/script入口、宽源码、模块/泛型/meta/stdlib扩面、host/console/FFI/use.c、797公开ABI/loader。CA-D有限桥/CA-E总签章仍后继；本包不签M1/M2，不改旧NA/accepted或用户批准hash。

## 2. 架构方案

现source profile→共享受检HIR/单函数凭证→SemanticView/ParamFacts/FnKind/BodyResourcePlan→C私有类型/模式/临时与清理合法化→确定性完整文件集→显式受控cl双侧认证/日常bytes+证书父判。公共session/store/key/恢复复用921，不复制cache/runner或借Rust代码转C；资源/借用/排他在原验证器单源，后端不从原AST/MIR/AV结果猜事实。

C内部Str/view/record/runtime与公开AutoABI/C边界分开；CA-D将来显式adapter，不让内部helper布局永久变成公开ABI。原Rust provider与C scalar字节/recipe/证书兼容，真实公共事实版本变化按依赖失效，不拿“共享”静默改旧golden。

## 3. 技术栈

Rust实现ACE，C11与真实MSVC cl，Windows x86_64-pc-windows-msvc；921 a2x-c及共享TargetProvider/AXP来源服务、现有ManagedStore与受控CREATE_SUSPENDED+Job/reader清理。C资源runtime为provider内版本化源资产，不另开workspace/静态Rust宿主编译腿。标准C分配/检查与工具/CRT身份由实际文件集/认证绑定，缺工具是infra。

## 4. 需求分析与背景调查

授权：2026-11-03用户确认923实施/924等待，要求检查其他工作并继续考虑A2C及统一ABI/FFI结构；继承已批准C作为M1必需、CA-B范围。同日 `/goal 继续完成计划925` 授权整链交付：本树 work（含续 T-08..T-11）、review、有界修复、merge（SD 沉淀/archive/清树/落点核查），各阶段继承本授权不再重复确认；不改目标/验收/范围/预算。本轮只设计/正式立约，未授权施工、跨仓修改/联系其他会话或新agent/chat。main读取ca9bcafb9；922已交付D=d66fdef04并归档9/9。923/924头滞后遵用户实际状态，不代改；921 executing1/14但分支已推进至fff3c124d，所有WIP保留。

| 已读来源 | 事实/内容SHA256或限制 |
|---|---|
| docs/specs/auto-a2x/checked-resource-facts.md | 11bff6549a73140d71fd83c3373d1a56bb8c3de6f2e212ed21495c59ad34cdd4；验证器BodyResourcePlan/排他fixpoint为唯一权威 |
| docs/specs/auto-a2r/record-text-hir-backend.md | 5867db025399f723c62585f246494b723d0d4bc2b40cb310f2cf53afedb66d44；902三域/方法/模式/资源，904只解除含type自由函数暖限制，方法仍冷 |
| experimental/ace/crates/a2x-core/src/semantic_view.rs | eaac652ddf650206242a403cf3a5212dbba1a785bbfd23c49e35c026762b08c3；FnFacts/ParamFacts/ExternalFn modes/body_plan出口 |
| experimental/ace/crates/core/src/text_resources.rs | 34c4e07146e81c90ad0d8050f97418993bca8241212ef0a7ed19b568d89c5fb6；escape_reach拒owned/mut参数/局部/临时的borrow返回 |
| crates/auto-lang/test/cases/a2r-hir-record-text-902/ | 原R07/R08/T03/T05四源与oracle42/42/23/42、独立Rust参考/金样；新C reference必须独立，不能翻译生成物自证 |
| experimental/ace/crates/a2x-worker/tests/cases/{record_text_902,record_text_modes,record_text_t04,store_rt_902,cert_rt_902}.rs | 全模式/资源/方法/预算/认证基线，四源通过不代表此全集已覆盖 |
| 921分支fff3c124d：a2x-core/src/target.rs | TargetDescriptor/Family/FilesetRequest/TargetProvider、prelude seed、C scalar provider；RecordText具名拒绝。尚未D921 |
| 同分支a2x-store::ensure_env | restore后parse_program_with+resolve_env重建，注释M-class不等于真实生产0；T-01重核最终D921，不能直接继承为恢复后暖门通过 |
| docs/specs/auto-abi/windows-x64-v01.md、abi/src/descriptor.rs | 797 candidate/descriptor3、pure/fallible/view/handle/per-call callback；不是A2C内部布局，CA-B不改其公开格式 |
| 922报告/Spec | M2-EXEC/HOST/SESSION/TEST、TS provider/应用迁移等仍后继；本包不替换产品默认或关闭这些缺口 |

取材含在途分支是只读能力调查，不消费未提交WIP/不宣称分支已发布；T-01登记实际source/内容hash/稳定接口、所有consumer、已验/未验与写窗。两份旧批准/原binding/applicability/source-manifest不改。

## 5. 详细设计

### 5.1 支持域与C值/借用表示

沿902三源域：i32/bool/Unicode char、str/Str、非递归具名record（纯值嵌套/含Str资源字段）；位置/命名构造、字段路径读写、static/view方法、view/mut/move自由函数参数及现支持返回。Copy值在求值点快照，资源字段读借用而非隐式clone；名义同形异owner不能混用。方法owner限定且receiver=params[0]，先于实参各恰一次求值；调用/构造所有有副作用表达式按源序物化临时再bindings落位，短路RHS保持惰性。

C表示可用私有struct/指针/长度，具体接口由T-02 whole-path冻结；int不能用平台long，char不是C char。str明确UTF-8 byte length，len按Unicode scalar、len_bytes按bytes、eq/ne按完整字节无归一化，NUL不能strlen截断；char_at/slice负值/越界/非法码点在访问/指针算术前具名trap。长度加法/size_t转换/索引步进先checked，signed overflow/别名/未对齐访问不得依赖UB。

view record只读、mut record真实writeback、move传责任由callee清理；纯Copy读取快照不可因后续mut追溯改变。mut文本沿现域只有排他读/转发，不新增可变字符串API；视图/独占由验证器模式/同根排他事实证明，C端不能因编译器没借用检查而放松。Borrow return只允许已验view参数/static literal，不来自owned/mut/临时；资源/借用事实缺出口从原验证器补有界只读事实，不在C重新推导。不未经证明随意加restrict或把内部函数当不可信C外部入口。

### 5.2 资源计划、失败原子性与退出

逐BodyResourcePlan消费exit_frees/buffer_frees/owned_returns/replace_frees及参数排他信息。显式clone/deep资源record clone产生独立backing，move源责任转移、callee入参清理、owned返回移交不提前free；借用本身不free。循环break/continue、分支/临时/early return/短路/trap/cancel各出口按计划清理，不能依赖C块结束或OS回收。

RHS/深clone所有fallible准备成功后才commit目标、释放旧值/移动源；后字段分配失败时先前新字段回收、源/旧目标仍有效，覆盖自clone替换/目标原本非空。只承诺每个操作的precommit原子性，不回滚此前已合法commit的guest writeback，不能改变原语言语义。cleanup不因已耗尽guest预算/取消再次中断；借用/pin活时允许对应backing保留，最后owner后live归基线。

runtime具名区分TextIndex/InvalidScalar/Resource/取消等现有域与编译/工具/宿主crash infra，C exit非零/panic/sanitizer崩溃不能充Auto trap。默认关闭的fault/事件旋钮不改生成文件bytes；alloc/free/owner事件纯数据、先限额，避免观测保活或隐式clone。逐真实allocation域1..N失败/N+1及无注入正控，不搬Rust N；每故障点检查旧逻辑值/责任、部分输出回收与清fault同进程重试，不只重跑程序推断原子性。

### 5.3 文件集、目标身份与共享层

新增C record-text recipe拟ac-a2c/hir-record-text-emit-v1、fileset auto.a2x.c.record-text.fileset/1、cert ace.a2c-hir.record-text.cert/1；fragment/runtime/打印族与C scalar/Rust严格互拒，层号T-01在当前registry核后分配，不预占全局A层。AXS2等中立语义层复用，真正target/layout/runtime相关数据不塞中立key；模式/名义类型/callee接口/资源事实及实际deps更新精确失效。

完整header/runtime/helper/generated入口/source map/manifest按必要闭包装配，prelude在纯恢复无SemanticView时仍可验证重建，不依main外的隐式include/绝对路径/本机缓存。路径规范、重复/穿越/额外文件/超量按原前门拒，输出与已授信source根不相交。CA-D只消费其受控描述/私有转换边界，**本包不生成公开AutoABI wrapper/import或宣称任意foreign语法**。

### 5.4 真函数/方法暖编辑与恢复续接

含type自由函数沿904真片段；static/view方法body必须补窄方法片段/owner上下文/单体凭证接线。至少3个未改函数/同owner兄弟方法：真实parse/resolve/typecheck/plan/bind/HIR verify/prepare/C lower/print/每函数工件IO0、resident真实payload/代际保持；不能整type/整源重编后对比文本冒0。callee仅改未被读取body不重编普通caller；签名/mode/名义字段/borrow/实际body读取依赖精确失效。声明/layout变更按914 D-05冷声明前端+依赖精确rebuild，不能把例外扩大到body方法。

纯恢复P0/R>0及恢复后下一函数/方法编辑分别验：caller源/上下文/target/工具/实际依赖锚绑定；有界decode/结构/语义/资源再验，索引/manifest/hash仍未授信，不deserialize Checked/PASS/宿主地址/活owner。需要的声明/方法/catalog/资源事实重建以数据投影/凭证验证完成；whole parse_program_with/resolve_env是P，即使代码注释称M也不能隐藏。T-02须先证恢复→下一method edit真链；D921若已修则复用，否则必要有界出口/重建修复归本包，缺大平台需明确修订/拆包，不cold凑门或延期到CA-E。

fresh/hit/restart/body/method-edit、A→B→A、bad edit→修复→下一restore、readonly hit/miss、foreign/stale/closed分别覆盖。失败/取消/迟到candidate保旧source/env/资源事实/resident/fileset/root/代际；cold修复单列、修复后下一restore真复用。当前/历史/reader lease保活到最后owner、随后物理GC，不删除别工程哨兵；P/R/M分账且阶段来自真实生产点，whole/只保hash换对象反控必须红。

### 5.5 独立认证、首例登记与预算

长期新cases拟crates/auto-lang/test/cases/a2c-hir-record-text-925/；原902四源与所有modes/method/borrow/resource正反例、881/894适用源保持字节与oracle，单独手写C reference不从生成C翻译。每例actual完整原文件集/独立reference分别真实cl编译运行、各自对固定值/具名trap/资源oracle；故障时actual allocation序与reference可不同，父表按实际阶段独立断言，不能用相同N自洽冒独立oracle。

证书严格绑source/profile/body/entry/target/runtime/printer/backend/完整文件集路径bytes与所有实际header/库/flags/SDK/cl identity、wrapper/reference/oracle/受控执行收据；未知字段/缺文件/裁main/工具错配/只编不跑/只跑reference/旧operation/伪造资源流全部非绿。新C认证不能套旧A2R cert-gap豁免。日常仅现场emit+bytes+有效证书身份核查，cl/link/目标程序/rustc/cargo实际spawn0，漂移只proposal/STALE不自动覆盖金样或隐式认证。旧M1批准/required313不改，首例新C登记/负控自本包进入，CA-E只组合不补本包遗漏测试。

A2X合计工作集≤48MiB（共享+Rust/C resident+暂存一起计），同owner store≤128MiB、cert scratch≤512MiB、guest值8MiB/managed backing32MiB、source/metadata4MiB、每流1MiB/父private增量128MiB和原更严cap；观察≤4MiB/操作但stderr仍受1MiB流门，先限额后物化，满则具名拒不截断PASS。原结构深度400保持其a2x.too-deep分类，不另加永久语法深度。compile≤120s/run≤30s/认证wall≤360s、Job/子孙/reader清理完整；不能拿本门放宽797桥原60/5/70s等限制。

正式基面保902资源record+方法+24caller/type-free变长负载，增加含type自由函数与static/view方法各编辑，至少3个未改单位，fresh/hit/restart/body/method各≥30样本、512真实不同修订，暖P90≤2000ms。每修订独立认证准备及cold/fresh/编译运行单列；warm计现场emit/恢复/父bytes+证书准入，不以缓存TestCase PASS或旧guest值替现场判决。C工具可能重编大于单函数的目标文件，生成P0不冒cl单函数增量，稳定日常Auto编辑不自动目标编译。

### 5.6 与921、923/924、922及ABI/FFI后继的交接

921主A2X共享TargetProvider/session/worker/AXP1，本包最终基面含D921；可先私有resource runtime/参考/fixture与T-02接口研究，不能抢其store/target写面或消费未提交WIP。以实际提交SHA/接口版本/字段/全部construct-match/消费者/已验/pending/释放交接，接口再变STALE受影响证据；不是等整波结束才释放具体符号。

923/924主native runtime/Place/闭包N，本包不写N或碰其faultABI。公共S仅方法片段/凭证/BodyResourcePlan/借用排他出口与shared core/schema：优先复用，确需变更逐符号取窗，AV/native/Rust/C全部真实consumer核齐，不因只加字段认为兼容。H=worker/ac_protocol/ac_a2r_hir/新C父adapter/mod、auto-test-case/SDK/fixed/workspace manifest-lock/来源和新target登记，全文件单写，正式认证/性能错峰，不持H等编译。

922已交付，消费其当前M2分母/后继排序，不改历史审计收据或自动裁减TS/host/产品支持。共同策略文档也按单写窗维护。本包不以M2-EXEC/HOST/通用FFI全部完成为前置，CA-D消费本包稳定资源/mode/符号与来源出口再正式立项；CA-C/D/E不预占号。CA-D仍M1必需，统一metadata/ABI/FFI结构按Design41 §11，但本包不能以未来桥为由泄露内部布局或漏资源/认证/方法暖门。

### 5.7 冻结接口（T-02 2026-11-03，whole-path 可行性已证）

**C 值/借用表示**：`str` view=`a2x_str_view{const uint8_t* data; size_t len}`（UTF-8 字节长，NUL 安全不 strlen）；`Str` owned=`a2x_str{uint8_t* data; size_t len}` 仅显式 free；`char`=uint32_t；record=私有 `struct a2x_rec_<Name>`（Str 字段按值拥有）；view record 参数=`const struct*`、mut=`struct*` 真实 writeback、move=按值转移由 callee 按计划清理；i32/bool 沿 921 checked int64 中间量。探针（plan-artifacts/t02-probe/probe.c）经冻结旗标 cl -std:c11 -W4 -WX -O2 -MT 零警告编译、运行 value:233 正确（UTF-8 标量索引）。

**层/身份**：新片段层 AXC2 `a2925/axc-c-rt-fn/v1`；新锚层 AXP2 `a2925/axp-rt-source-anchor/v1`（携带 C prelude seed 字段；AXP1 v1 字节不动）；recipe `ac-a2c/hir-record-text-emit-v1`、fileset `auto.a2x.c.record-text.fileset/1`、cert `ace.a2c-hir.record-text.cert/1`（T-01 核零冲突）。C prelude seed=provider-opaque 行文法（record 表：名/资源/字段+类型），冷时 capture_prelude_seed(view,RecordText) 捕获、恢复 view=None 装配传入。

**文件集**：4 文件——main.c（921 字节协议包装）+ a2x_c_rt_runtime.h（资源运行时：trap 增 InvalidScalar/TextIndex/Resource、UTF-8 标量运算、分配预算+fault/事件 env 旋钮、默认关闭不改生成 bytes）+ generated.c（seed 派生 struct 定义+前向声明块+片段定义）+ manifest.json。

**方法暖门链**（各环静态核实存在/有界缺口）：parser 新增 parse_method_fragment（Receiver::Static|View，free 保持 parse_function_fragment）；嫁接 recheck_swap 已 881 方法感知（owner/recv 槽位权威）；单体文档含 owner record 脚手架（adapter.rs:3340）；verify_text_function 按 def_idx 可用；from_parts 经声明上下文指纹（含 owner）+body digest 绑定；fingerprint 已有 `sig mt` 标记（prepare.rs:367）。缺口：env 方法限定名槽位查找（env_fn_names 为裸名）、def 命名分叉（方法 def=`d_f{fi}_{name}` 非 `d_{name}`，a2x-store def_index_of/prepare_function_candidate 需按 kind 分叉）。

**恢复链**：冷/ranged 编辑在 RecordText+C 后端发布 AXP2 锚（携带 seed+fileset 收据+必活边）；warm_edit_ranged 放宽收 RecordText(C)；锚 origin 重绑复用 scalar 模式；restore_source_bound 放宽至 RecordText+C：AXP2 查找→切片/语义/片段/文件集逐项重验→view=None+seed 重装配→装 residents、env=None；下一编辑 ensure_env（M 计量）后走方法/函数暖路（P0 未改单位）。Rust record-text 目标保持现状（不 widen，具名拒绝）。

**有界事实缺口清单（全在本包，无大平台缺口，D-02 正向关闭）**：①parser 方法片段入口；②def 命名/槽位分叉；③AXC2/AXP2 LayerSpec+dispatch；④C provider 4 处具名拒绝→RecordText 支持放宽；⑤a2x-c 资源 runtime+record/str/char/modes/出口 lowering 新模块；⑥store 锚发布/重绑/恢复 RecordText+C 放宽+seed 传递；⑦worker emit_c/certify-c profile 放宽+recipe/适配/contract 登记+新固定组。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-a2c/record-text-hir-backend.md | scalar后继→C三域/模式/方法/资源与拒绝 | 真实C域，不冒FFI | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-a2x/checked-resource-facts.md | 补C实际需要的原验证器只读事实/消费边界 | 不复制资源分析 | AC-04 |
| SD-03 | modify | docs/specs/auto-a2x/incremental-and-cache.md | C资源片段/纯恢复/续编辑/P/R/M/事务/配额 | 不把whole记管理 | AC-05/06/08 |
| SD-04 | modify | docs/specs/auto-a2x/architecture.md | C资源provider/类型prelude/所有consumer与后继边界 | 原Rust/C scalar兼容 | AC-04/09 |
| SD-05 | add | docs/specs/auto-test-case/a2c-hir-record-text-certification.md | 首例/独立actual-reference/严格cert与日常零编译 | 新C强认证不豁免 | AC-07 |
| SD-06 | modify | docs/specs/auto-a2c/record-text-hir-backend.md | 资源事件/fault/恢复/正式预算与实际支持边界 | 真实内存/时间 | AC-02/06/08 |
| SD-07 | modify | docs/specs/auto-a2x/incremental-and-cache.md | 方法/body实际依赖/上下文和CA-D交接身份 | 不缓存PASS/旧源盲命中 | AC-05/09 |

## 6. 测试设计

现有固定职责组ace-a2x、ace-source、ace-semantics、ace-adapters、auto-test-case、infra-fixed、infra-ace-m1、case-a2r-hir-record-text-902、case-a2r-hir-collections-904、case-ace-m1-method-881、case-ace-m1-text-source-894；touch ManagedStore补ace-artifacts，公共IR/凭证真变补ace-lowering/ace-av/ace-native等必要consumer。921新增scalar组须D921后按真实注册确认；本包拟新case-a2c-hir-record-text-925由T-08注册，不当前当现成命令。正式性能原ace-performance条件不缩；无因不叠全experimental/all。

| Case ID | AC | 输入/真实入口/使能任务 | 独立结果与区分性控制 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/07 | 原R07/R08/T03/T05及全部902冻结正例→C实际文件集，T-03..05/09 | actual/reference分别42/42/23/42及各自固定oracle；原源/expected不改 | 三域非只四微例 |
| V-02 | AC-03 | view/mut/move、Copy快照、返回借用/拥有、receiver单求值、同根冲突，T-04 | 真实writeback/源责任/合法borrow；交换mode/隐式clone/同根放行反控红 | mut文本仅原排他读域 |
| V-03 | AC-01/03 | Unicode/NUL/负索引/非法char、构造与命名实参trace/短路，T-03/04 | 正确长度/bytes、TextIndex/InvalidScalar、源序一次；strlen/顺序反控红 | 不依C UB |
| V-04 | AC-02 | clone/deep record/旧目标替换、多字段后段fault各1..N/N+1，T-05 | 故障源/旧值保留、部分清理、同进程retry；早free/只重跑/漏seam红 | 只回滚未commit操作 |
| V-05 | AC-02 | return/break/continue/分支/短路/trap/cancel与活view，T-05 | 逐owner一次清理、借用不free、owned返回保活；省drop/提前free红 | 验证器计划，不靠OS |
| V-06 | AC-04 | 正式/单体凭证、mode/owner/decl/borrow事实漂移，T-02/04/11 | 异owner/旧plan/无排他种子拒，合法近邻可用；不把C编译当Auto验证 | 原单源facts |
| V-07 | AC-05 | 含type自由函数＋static/view方法body、≥3未改兄弟，T-06/07 | 真实各P/IO0、resident恒等；whole type/source后过滤/换对象红 | 方法暖门不可cold豁免 |
| V-08 | AC-05/09 | callee body vs签名/mode/layout/依赖、A→B→A，T-06/07 | 普通caller复用，真读者精确失效/旧态安全，调用点不变也核deps | 声明沿D-05分账 |
| V-09 | AC-06 | 纯来源restore→下一function/method edit，坏件/readonly/修复→再restore，T-02/07 | P0/R>0、next未改P0、坏来源拒；ensure_env whole parse记M反控红 | 不deserialize Checked |
| V-10 | AC-07 | 新cert全轴/全fileset/actual-reference/故障伪证，T-08/09 | 坏字段/旧域/裁main/只编/只参考/工具错配非绿；强制绕过实际执行反控红 | 新C cert独立 |
| V-11 | AC-07/08 | 日常snapshot/漂移/STALE/旧cert复用，T-08..10 | 实际cl/link/target/rustc/cargo spawn0，只proposal不改golden | 显式认证另列 |
| V-12 | AC-08 | 原24caller负载＋方法编辑/30样本/512修订/深clone与预算边界，T-10 | 原cap/流与P90≤2000ms、先限额后物化；超量不截断PASS | cold/认证单列 |
| V-13 | AC-09 | Rust原scalar/rt/collections＋C scalar、全reader/CA-D边界，T-11 | bytes/recipe/有效证书/真实命中不无因变、跨target互拒；内部struct不冒AutoABI | ABI桥仍待CA-D |

实现/自查同一唯一树，final review独立上下文从冻结SHA重核每AC/必需补例/SD；无用户指定review模型/外部agent，不自动创建聊天或第二树。review暂停实施写，修复回原树；merge按真实diff冻结完整职责/必要consumer并集并处理漂移，身份未变证据可复用。周期cargo t all仅due主检出单实例。所有用例/命令本轮未执行。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01/03三域原语义/完整C文件集/Unicode/源序与明确拒绝通过，不依UB |
| AC-02 | work | V-04/05真实资源计划/clone/move/返回/各出口/故障原子性及清理通过 |
| AC-03 | work | V-02/03参数模式/writeback/Copy快照/借用来源/receiver与别名边界通过 |
| AC-04 | work | V-06 C只消费正式受检单源事实，外来/过期/缺种子拒，所有补出口有真实owner |
| AC-05 | work | V-07/08方法与自由函数暖未改各P/IO0/对象代际/实际依赖失效通过，cold不能冒暖 |
| AC-06 | work | V-09纯来源恢复/下一方法编辑/失败修复再恢复/只读完整，P/R/M真实分账 |
| AC-07 | work | V-01/10/11首例AutoDown/独立双侧真认证/strict身份/日常零目标编译通过 |
| AC-08 | work | V-11/12原资源/规模/正式时间/深clone/流/Job清理与边界控制通过 |
| AC-09 | work | V-08/13 Rust/C scalar兼容/全consumer、D921基面与CA-D稳定出口清楚，无越权公开ABI |
| AC-10 | review | 独立冻结review逐AC/V/SD通过，无必需method/恢复/资源/认证延期到CA-E |
| AC-11 | merge | 含D921实际主线交付/SD读回/archive、全部输出对账/guard清树分支组/scope登记收据 |

## 8. 执行步骤

| T | owner_stage | 前置/真实写面 | 产出与验证入口 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 当前main/921稳定SHA/923924窗口 | 核全基线/方法冷路/restore续编辑、接口/所有consumer与工具/新域身份 | AC-01..09 | [x] 2026-11-03：main=ec3e70414（D921=3c760865c 在线：TargetProvider接缝/AXC1/AXP1恢复/C scalar后端+cert均已交付）；方法冷路实证=a2x-store lib.rs:971-974注释+env_fn_names仅自由函数(resolve.rs:2014)；scalar恢复续编辑=restore_source_bound P0(无lower/print/persist)+ensure_env计数env_rebuilds、下一编辑仅编辑函数生产(a2c_restore.rs:82-102)，本树实测5测试3.18s绿(含c_restore_pure_p0_then_edit_then_restore_again)；RecordText恢复全缺(连Rust也scalar-only门lib.rs:1230)；consumer清单=a2x-store(lib.rs layer_fragment/static_layer/restore门)、a2x-c 4处具名拒绝、worker emit_c门/certify-c、auto-lang-core ac_a2c_hir适配、auto-test-case contract.rs 4处、boundaries.rs钉、test-groups(模板case-a2c-hir-scalar-921)；三拟用id(ac-a2c/hir-record-text-emit-v1/auto.a2x.c.record-text.fileset\/1/ace.a2c-hir.record-text.cert/1)全仓零冲突；cl工具链经测试实spawn编译运行验证；923 executing 2/12、924 drafting窗确认 |
| T-02 | work | T-01，私有runtime及S短窗 | source→C资源+参数/借用返回＋method fragment＋restore→next method whole-path feasibility，冻结最小接口/事实缺口 | AC-02/03/04/05/06 | [x] 2026-11-03：§5.7 冻结（C 表示/AXC2+AXP2/4文件集/方法暖门各环/恢复链/7项有界缺口）；探针零警告+value:233（cl VS2022 BuildTools 真实编译运行，t02-probe/build.bat）；方法链静态核实：recheck_swap:2126 方法感知、单体文档 owner 脚手架 adapter.rs:3340、from_parts:265 上下文指纹绑定、prepare.rs:367 mt 标记；恢复链 scalar 各环（publish_anchor:901/ranged rebind:1124/restore:1214/ensure_env:1503）可平移 RecordText+C，缺口=env 裸名槽位+def=`d_f{fi}_{name}` 分叉（见§5.7） |
| T-03 | work | T-02，a2x-c P | C私有类型/UTF8/char/checked runtime/具名错误与完整文件集，V-01/03；cargo t ace-a2x | AC-01 | [x] 2026-11-03：a2x-c新增rt_runtime/seed/lower_rt+provider拓宽、store AXC2层；worker新a2c_rt腿6测全绿（冻结四源R07/R08/T03/T05经真cl→42/42/23/42含R07 mut回写+T03 move转移；V-03文本/字符面Unicode/NUL/全字节eq/标量索引TextIndex/InvalidScalar；CLEANUP 6事件序钉计划清理（replace在RHS物化后free旧、出口LIFO）；fault第n次分配→Resource；文件集字节确定+schema；seed严格解码6反例）；`cargo t ace-a2x` PASS；a2x-c clippy/fmt零警告（a2x-store 9警告为921存量anchor.rs）；commit 33chars前缀feat(a2c)已入库worktree |
| T-04 | work | T-02/03，provider/core必要窗 | 模式/名义record/静态view方法/receiver/source-order/facts消费，V-02/06；cargo t ace-source ace-semantics ace-a2x | AC-03/04 | [x] 2026-11-03：a2c_rt 新增4测（commit 794a75e89）：modes_matrix 8例经真cl（Copy快照221/双mut 12/view record 42/mut文本转发4/借参slice返回2/owned返回22/命名反序构造12/同名自由函数+双owner方法137）；receiver_once marks [mark_b,mark_a]（C trace，receiver先于实参恰一次）；同根mut+view经C session入口具名verify.mut-conflict；warm片段编辑lower/print=1、persist=2、真cl value:63、跨名片段具名拒。R08 static/view方法已在required_four。门：ace-source 307/307、ace-a2x 119/119 PASS；ace-semantics 104/106——2红为存量D-925-1（sema_boundary_audit读git忽略attachments/796文件，新鲜worktree必红，与925 diff无耦合，已登记KNOWN-DEBT），不充PASS |
| T-05 | work | T-03/04，C runtime/lower | 所有计划出口/深clone/replace原子/fault/事件与同进程retry，V-04/05；cargo t ace-a2x | AC-02 | [x] 2026-11-03：a2c_rt_resource 双测绿+门PASS：①探针TU同仓#include generated.c共享runtime账本——逐分配fault 1..N（typed Resource、部分回收平衡free==attempt-1、源record逐故障点完整）、N+1不动、clone独立backing（free一侧另一侧可读）、清fault同进程retry成功、mut参数replace原子（fault时旧buffer存活0 free、retry提交value:302、恰好1 free）；②出口路径程序（for循环continue/break、分支return、短路侧owned temp）值oracle+账本alloc==free平衡。**修复两个被oracle抓到的真实缺陷**：(a)验证器S窗——reown后root无条件入exit live集，mut资源record参数被callee排出口free（902语料无此形状的存量缺口，Rust侧被幂等take-free掩蔽；三处reown点加plan.owned守卫，replace_frees经借用照旧）；(b)C lowering——pending view-consumer free在消费者pin之前drain（use-after-free，改消费者后drain）。门 cargo t ace-a2x ace-source PASS 428/428零回归 |
| T-06 | work | T-02..05，source方法片段S窗 | 窄method/含type片段凭证与真阶段计量/依赖，V-07/08；cargo t ace-source ace-semantics ace-a2x及实际consumer | AC-05 | [x] 2026-11-03：S窗新增parse_method_fragment/env_method_index/env_free_fn_index、prepare方法def `d_f{slot}_{name}`、脚手架方法span inert（904规则补齐，原retained item_span越界片段）；store warm_edit_method+WarmTarget分叉。a2c_rt_method_warm 5测绿：view/static方法body暖编辑真阶段计量parse=1（无整源parse）/prepare=verify=1/lower=print=1/persist=2/whole_fallback=0，未改兄弟方法/同名自由函数/caller resident字节恒等，真cl 4217/2327/2219；签名变更/未知owner/receiver形状变更具名拒；双owner同名方法不串线(17)；A→B→A store hit且lower=0；904含type自由函数暖片段在C通道成立。门 ace-source/syntax/semantics/a2x/av 1657 ok，红=D-925-1两例(环境)+a2c_cert一例(release worker过期，重建后隔离重放3/3绿；final gate前须重建release worker) |
| T-07 | work | T-02/06，921交接store/anchor | 资源prelude/文件集/来源恢复next编辑/事务/历史/readonly/GC，V-08/09；cargo t ace-a2x及实际ace-artifacts | AC-05/06 | [x] 2026-11-03（commit 199c23a40）：anchor v2 schema a2925/axp-rt-source-anchor/v1+seed行（严格配对，v1字节不动）；AXP2层+anchors_family门（scalar全域/RT仅C，Rust rt保持无锚冻结行为）；冷发布/ranged重绑带schema+seed、按owner限定decl重绑（裸名曾致方法条目漏绑+错位切片sha自洽——已修）；恢复按family显式层/schema配对证明/seed透传盲装配；warm_edit_ranged_method；失败编辑回滚源绑定（manifest仍绑next：预设+失败还原，修collections回归）。a2c_rt_restore 4测绿：P0/R>0新会话恢复字节恒等、恢复后方法编辑（env_rebuilds+1/仅编辑法/锚重绑/真cl 23）再恢复P0、no-anchor/context拒、坏编辑保绑定+修复重绑、readonly零发布、lease+GC存活、Rust-rt/跨家族隔离。**另修C oracle发现的scoped drain缺陷**（多实参view消费temp被后参pin提前free→mk("ab","cde")算出3；改mark/drain_since作用域free；Rust同形登记D-925-2）。门 cargo t ace-a2x ace-artifacts PASS（曾2次discovery失败=store单测Anchor字面量缺新字段+并行负载偶发，均已处置） |
| T-08 | work | T-03..07，H全文件窗 | worker/父adapter/新recipe/AutoDown/strict新cert/固定组/首例新ledger，V-10/11；cargo t ace-adapters auto-test-case infra-fixed infra-ace-m1及新组 | AC-07 | [x] 2026-11-03（commit 6b1e18b07）：worker emit_c/certify-c 放宽 record/text/text-record（Family::RecordText+seed 捕获+hir_domain 真实绑定）；cert.rs 双 schema 严格配对（scalar/RT 交叉伪装 REJECTED）；父 adapter A2C_RT_HIR_RECIPE 分派+profile 门；contract.rs recipe 注册（op=run 单步+a2x_hir profile 集）；首例 ledger a2c-hir-record-text-925（R07/R08/T03/T05/C04 五源+独立手写 reference-*.c+真实 emission 冻结 golden 4 文件集）；固定组 case-a2c-hir-record-text-925 注册（**补 all=false**——infra-fixed 抓到缺省 all=true 违反 PLAN-898 语言侧-only 裁定）；a2c_rt_cert 3 测绿（两腿真 cl PASS+cert 字节稳定/错 oracle 拒/跨通道+裁证据+非 PASS 负控）。门：新组 5/5 bytes 恒等、ace-adapters/auto-test-case/infra-fixed/infra-ace-m1 PASS、ace-a2x PASS、conformance a2c_rt 24/24。健康：a2x-c 7 条新警告清零（seed.rs doc 列表+provider.rs useless_format，golden 字节复核不变）；a2c_rt_restore.rs &PathBuf 修；cargo fmt 全包应用（含 902-921 存量 formal bins 纯格式）；存量警告登记：a2x-worker 15 条=902/904/921 文件（a2x-formal/a2x-coll-formal/a2c-formal/a2x-rt-formal/a2c_restore/store_cache 等，非 925 diff）。清理上次会话副作用：plan827 incremental-raw 指针还原+pid 文件删（av_source_incremental 每跑刷新，与 925 无关）、experimental/crates 镜像删（一次性残留，复跑不复现已证） |
| T-09 | work | T-08，认证错峰 | 全冻结源/modes/方法补例actual/reference真实cl、独立金样审阅/反控；V-01/10 | AC-01/07 | [x] 2026-11-03（commit 25e97f9f9）：a2c_rt_cert 扩至 7 测——rt_certify_all_suite_cases（五源 R07/R08/T03/T05/C04 actual+独立 reference 双腿真 cl，oracle 42/42/23/42/42，cert 绑 a2c-hir-925/<case>）；rt_certify_modes_matrix（8 模式/方法例各配手写内联 reference：Copy快照221/双mut12/view record42/mut文本4/借返2/owned返22/命名构造12/双owner同名137）；rt_certify_trap_axis（char_at(-1)/slice(0,6)=TextIndex、surrogate=InvalidScalar kind=trap 认证+错 trap 名拒）；rt_certify_bad_references_refused（错值 reference 拒于 oracle 腿+坏语法 reference 拒于编译腿）。**独立金样审阅抓到 reference-T03.c 两个真实缺陷并修复**：①字面 `\n` 落入 C 源（语法错误，从未被编译过）②c.len() 误算字节长 25≠标量数 23（oracle 腿独立抓出）——修为诚实的 UTF-8 lead-byte 标量计数实现。门：conformance a2c_rt_cert 7/7、ace-a2x PASS、case-a2c-hir-record-text-925 PASS |
| T-10 | work | T-01..09，正式采样错峰 | V-11/12原负载/方法暖编辑/30样本/512/资源门与实际spawn测量 | AC-08 | [x] 2026-11-03（commit 7728ec4f2，收据 plan-artifacts/a2c-rt-formal-receipt.json）：新采样器 a2c-rt-formal（C RT 通道，单实例错峰窗）——**514 修订全部真 cl 认证**（1 record 基线+256 method 修订+1 text 基线+256 body 修订=512 真实不同修订，全部对独立闭式 oracle，认证墙 680s 不入计时样本）；六状态×30 样本：fresh p90=440ms（单列）、hit 7/restart 11/body-edit 22/**method-edit 34**/restore 13（p90 ms），全暖门 P90≤2000ms PASS（余量>58 倍）；未改单位账本：body 态 a1/a2/main、method 态 24caller+W.make+main，逐态断言 lower/print=30、persist=60（仅被编辑单位生产）；预算：a2x 工作集合计 10,479,228B≈10MiB/48MiB、store 配额 128MiB 未触、compile≤120s/run≤30s 沿 crun 冻结期限；子进程证人：跨进程纯恢复 P0（lower=0、8ms）→ ranged 方法编辑 env_rebuilds+1（M 计量）→ 双 cl 认证。**跑中修两错**：①我方闭式 oracle 误算 "anchor-two!!"=11 字节（实为 12）被 actual 独立抓出；②restore→method 编辑须 ranged 授权（非 ranged 需活 env，store 合同）——均按独立 oracle 纪律修正 |
| T-11 | work | T-01..10 | V-13冻结完整职责自查/SD/CA-D接口与未支持域；无旧Rust/C scalar无因漂移 | AC-09 | [x] 2026-11-03（fmt 收尾 commit b10dd1d4b）：冻结职责组并集 14 组——12 绿（ace-a2x/source/adapters/auto-test-case/infra-fixed/infra-ace-m1/ace-artifacts/case-925/case-921/case-902/case-881/case-894）；ace-semantics 单跑=仅 2 已知 D-925-1 环境红（git-ignored 附件缺失，T-04 起登记），并跑时另 5 sema_cache 红为并行负载偶发（隔离重跑绿，T-07 先例）；**case-a2r-hir-collections-904 为 master 存量红非 925 造成**：全部 20 例 golden 自 904 交付提交 6049a3da6 起即无 manifest.json（git --all 全历史无该文件记录）而 case .ad 全部期待之，runner 自 Plan 773 起硬性解析——归 904 债，本包不越权修，待回归分诊/独立修复任务。漂移：基面 ec3e70414 diff 无 abi//docs/specs/902/904/921 触碰，61 文件全在声明写面；recipe/fileset/cert/layer（AXC2/AXP2）与全部既有身份零冲突（跨通道伪装双向拒）；内部 C struct 未冒公开 AutoABI。健康：a2x-c/a2x-worker/a2x-store fmt+clippy 自有代码零警告；存量登记 a2x-worker 15 条（902/904/921 文件）+ ace-core 2 条 + collections_facts.rs fmt 存量（非 925 diff）。CA-D 交接：稳定资源/mode/符号/来源出口在本包；List/Variant（CA-C）/闭包/mut 实例方法/公开 ABI wrapper/FFI 为显式未支持域 |
| T-12 | review | work完成，冻结SHA | 独立所有AC/必需V/SD复审，修复回同树，不先勾merge门 | AC-10 | [x] 2026-11-03：final pass @ b10dd1d4b（复审=同会话独立上下文受限，已按技能以工件重建全部结论：新鲜重跑+源码重读+oracle 独立重算，未引用 work 阶段总结）。AC-01..09 全 pass（证据映射见 §9 复审记录）；五源 oracle 从 .at 源文独立重算吻合（42/42/23/42/42）；S 窗三处 plan.owned 守卫（text_resources.rs:1822/1951/2028）与 mark/drain_since 作用域释放（lower_rt.rs:661-917）代码核实；V-06 覆盖=seed 严格解码 6 反例+未知 owner/签名/receiver 具名拒+同根冲突拒；V-11 对等=925 7 测严格超集 921 3 测先例；SD-01..07 目标/规则/验收映射核（auto-a2c/record-text-hir-backend.md 新文件沿 921 scalar 命名先例）；frontmatter spec-impact 元素一致；D-925-1/2 已登记 KNOWN-DEBT。**复审中一次 ace-a2x 并行红分诊**：a2c_rt_cert 3 例 `cl -? spawn: Deadline`（abi/ctool DISCOVERY_DEADLINE=10s，12 路 nextest+每 certify 新 worker 进程各探测一次，负载风暴超时）——隔离重跑 7/7 绿+整组重跑 PASS（冻结提交上），判并行负载瞬态（T-07 先例同族），非契约缺陷；非阻塞建议：后续可跨 worker 缓存工具链身份或探测重试一次。观察项：case-a2r-hir-collections-904 master 存量红归 PLAN-904（git 证据）；a2x-worker 15 条存量警告（902/904/921 文件）。merge-owned pending：b10dd1d4b 上 14 组冻结并集重跑、SD-01..07 沉淀读回、archive、清树 |
| T-13 | merge | review pass且含D921 | 主线漂移/冻结职责与consumer并集验证，合入/SD-01..07读回/archive | AC-11 | [x] 2026-11-03：冻结 14 组并集 @ b10dd1d4b=12 绿+2 已知红（ace-semantics 2=D-925-1 环境；904=master 存量，git 归因 PLAN-904）。**merge 抓出打包缺陷**：925 套件 5 个 golden manifest.json 被 `.gitignore:162 *.json` 母规则吞掉未跟踪（落地即复现 904 型 inventory 红）——沿 892/902/921 豁免先例补 `!.../a2c-hir-record-text-925/golden-*/manifest.json` 并跟踪，删盘→git 恢复→组绿验证自足。SD-01..07 沉淀（auto-a2c/record-text-hir-backend.md 新、auto-test-case/a2c-hir-record-text-certification.md 新、a2x 三件 modify、auto-a2c/project.md 导航更新，docs-only commit a4a0777dd）。rebase master（基面增量=docs+1 无关 UI fixture，证据免刷新）range-diff 全 `=`（旧→新：179653ef3→c714fc55c…b10dd1d4b→2e190e8c9、a4a0777dd→a37245cd6），ff-only 合入 master=a37245cd6；canonical 验证 6 spec 哈希（ad5417d66544/86f5049f6d69/b31e8c7fce88/5c06c640218f/d977cb1bff2c/ab66d3635eb5）；主线冒烟 case-925 PASS+ace-a2x PASS（首轮 receiver-once 空 marks=冷构建负载瞬态，隔离 10/10+整组重跑绿） |
| T-14 | merge | T-13 | 全实际输出对账/摘要折计划、guard后清树分支组与整个scope/刷新登记 | AC-11 | [ ] |

拟唯一树D:/autostack/.wt/lang-925/auto-lang、分支plan-925-dev，本轮未建。实施材料唯一D:/autostack/plan-artifacts/plan-925/，开工登记owner/用途/在途；探针/生成C/对象/exe/日志/认证/SDK副本/外置target全入scope，常规target在专属树。禁attachments/项目证据目录/散落副本/链接；merge整删，仅具名未解决hold可留，不清他方材料。

## 9. 复审记录

2026-11-03 | stage:merge | PLAN-925 r1 | checkpoints: prepared=reviewed b10dd1d4b+docs-only 后代 a4a0777dd（SD 沉淀+golden manifest 跟踪）；landed=master `git merge --ff-only plan-925-dev` → tip a37245cd6（rebase 于 master 2f558fc93，range-diff 全 `=` 补丁等价，映射 179653ef3→c714fc55c/794a75e89→9c9854984/92ac084fc→759ee8221/0a066c89a→f81d14cb2/199c23a40→09d62e377/6b1e18b07→b0394b2d1/25e97f9f9→(r7)/7728ec4f2→8d9ebddc3/b10dd1d4b→2e190e8c9/a4a0777dd→a37245cd6；基面增量 docs+1 无关 UI fixture，冻结证据免刷新）；canonical_verified=6 spec 于 master（哈希见 §8 T-13 行）；merge 门=冻结 14 组并集 12 绿+2 已知红（显式记账）；主线冒烟=case-925 PASS+ace-a2x PASS（1 瞬态已分诊） | next: archive+cleanup（T-14）

stage: goal | plan_id: PLAN-925 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-925/auto-lang | commit: b10dd1d4b | next: merge

2026-11-03 | stage:review | review_scope:final | reviewer: 本会话（与实现同会话，独立性受限——按计划 §6 无指定外部 reviewer/不自动创建会话；以工件重建结论缓解：新鲜重跑+源码重读+oracle 独立重算，未引用执行者总结） | PLAN-925 r1 | outcome:pass | worktree: D:/autostack/.wt/lang-925/auto-lang | branch: plan-925-dev | reviewed_commit: b10dd1d4b（树 clean） | base_commit: ec3e70414 | dependency_revisions: 无跨仓 | spec_inputs: auto-a2x/{architecture,checked-resource-facts,incremental-and-cache}.md 现版+auto-a2c/scalar-hir-backend.md（921）+auto-test-case/a2c-hir-scalar-certification.md（921） | acceptance_results: AC-01 pass（suite 5/5 bytes 新鲜重跑@b10dd1d4b+cert 五源双腿+五源 oracle 独立重算吻合）；AC-02 pass（fault 1..N/N+1 矩阵+replace 原子+出口账本，ace-a2x 绿@冻结）；AC-03 pass（modes 8+receiver-once marks+同根拒+cert modes 7/7 隔离重跑）；AC-04 pass（S 窗单源：三处 plan.owned 守卫代码核实；seed 6 反例；跨通道配对双向拒）；AC-05 pass（方法暖 parse/lower/print=1、persist=2、whole=0+A→B→A+T-10 未改单位账本）；AC-06 pass（恢复 P0/R>0+下一方法编辑 ranged+坏件修复+readonly+lease/GC+子进程证人）；AC-07 pass（首例 AutoDown+7/7 cert 测超集 921 先例+日常 spawn-zero bytes）；AC-08 pass（P90≤34ms/10MiB÷48MiB/514 cl 认证收据+crun 冻结期限）；AC-09 pass（基面 diff 无 abi/spec/旧套件触碰+12/14 并集绿[2=D-925-1 环境红，5 并行 flake 隔离绿；1=904 存量红已归因]+跨 target 互拒）；AC-10 本记录 | pending_closeout_gates: merge-owned——b10dd1d4b 上 14 组冻结并集重跑、SD-01..07 沉淀+读回、archive、guard 清树分支组、plan-artifacts scope 整删 | findings: 无契约内缺陷；非阻塞观察 3 笔（①a2c_rt_cert 工具链探测并行 flake：DISCOVERY_DEADLINE=10s 于 12 路 nextest+50 进程探测风暴下超时，隔离/整组重跑绿，建议后续跨 worker 缓存或单次重试；②case-a2r-hir-collections-904 master 存量红归 PLAN-904；③a2x-worker 15 条存量警告=902/904/921 文件） | evidence: 见 §8 T-12 行+本轮新鲜命令（cargo t case-a2c-hir-record-text-925/ace-a2x、conformance a2c_rt_cert 7/7） | next: merge

stage: goal | plan_id: PLAN-925 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-925/auto-lang | commit: b10dd1d4b | next: review

2026-11-03 | stage:work | PLAN-925 r1 | outcome:pass | worktree: D:/autostack/.wt/lang-925/auto-lang | branch: plan-925-dev | base_commit: ec3e70414（含 D921=3c760865c） | code_commit: b10dd1d4b（链 179653ef3→794a75e89→92ac084fc→0a066c89a→199c23a40→6b1e18b07→25e97f9f9→7728ec4f2→b10dd1d4b） | dependency_revisions: 无跨仓依赖 | task_ids: T-08/T-09/T-10/T-11（T-01..07 前次会话完成，本轮对账核认后沿用） | evidence: T-08 首例 suite 5/5 bytes 恒等+cert 3/3+四门禁绿（infra-fixed 抓出并修复新组缺 all=false 违反 PLAN-898）；T-09 认证扫全 7/7（五源+8 模式+3 trap 双腿真 cl、错值/坏语法/跨通道/错 trap 反控全拒；独立审阅抓出 reference-T03 两真缺陷：字面 \n 语法错+c.len 字节/标量混算）；T-10 采样 514 修订全 cl 认证/六状态全暖门 P90≤2000ms（实测≤34ms）/未改单位账本/子进程 P0 证人（收据 plan-artifacts/plan-925/a2c-rt-formal-receipt.json，10,479,228B/48MiB）；T-11 冻结并集 12/14 绿+2 已知环境红+1 master 存量红归因（904 golden 缺 manifest.json 自交付即缺，git 证据）；健康 fmt/clippy 自有代码零警告 | blockers: 无阻塞性未决；备注两笔非阻塞登记——D-925-1 环境红（存量）、case-a2r-hir-collections-904 master 存量红归 PLAN-904（建议独立修复任务/回归分诊） | next_responsible_role: review（T-12 独立复审）

2026-11-03 | stage:new | PLAN-925 r1 | outcome:pass（规划静态准入，非C资源实现/运行认证/final review） | 0..10节、T14/AC11/V13/SD7定义与引用、16处新计划/导航本地链接锚点、现有固定组/拟新组、唯一ID/next926/全任务未勾已核。next:work T-01与私有T-02研究，公共A2X取921稳定SHA，最终含D921；S与923/924单写，统一ABI/FFI本轮仅结构设计、CA-D未取号。两份批准及binding/applicability/source-manifest哈希未变，无代码/current Spec/在途计划变更，未建925树/scope或执行语言/性能/认证测试。方法body与恢复后next edit真实P0是必需门，不能以M-class整源重建绕过。

2026-11-03 | stage:new | PLAN-925 r1 | 起草：先Design51 §14/Design41 §11补设计，排他new-plan.sh取925，next926。921分支已有C scalar/provider/restore但未交付，922已交付；923实施/924等待沿用户状态。本包承诺方法暖门/恢复后下一编辑，不把cold或M-class whole parse藏生产；统一ABI/FFI只设计、CA-D仍M1必需但未取号。本轮仅规划，未实施/建树/构建/测试/current Spec/原批准登记或在途计划变更。

## 10. 待澄清事项

| D | owner / 有界关闭 |
|---|---|
| D-01 921实际出口 | T-01核正式稳定SHA/全部construct-match/consumer与D921最终来源恢复行为；私有准备不消费未提交WIP |
| D-02 方法/borrow事实 | T-02先证method fragment+凭证/返回借用与source恢复next edit；有界补出口在本包，需新大平台则正式修订/拆包，不降暖门 |
| D-03 工具与资源表示 | T-02/03冻结C11 runtime/分配/typed trap/完整文件集，缺工具报infra，不换编译器遮合法C错误 |
| D-04 CA-D边界 | T-11给参数/资源/符号/来源与工具依赖实际版本；foreign语法/publicABI在CA-D另立，不泄内部layout充完成 |
| D-05 公共S/H窗口 | T-01与921/923/924逐文件/符号短窗，正式采样错峰；922历史审计只读，不改其收据/义务分母 |
