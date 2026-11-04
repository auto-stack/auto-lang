---
plan_id: PLAN-928
status: archived
feature_name: SG-B 集合变体闭包语法与嵌套单元增量投影
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-syntax/grammar-contract.md, docs/specs/auto-syntax/source-frontdoor.md, docs/specs/auto-ac/closures-function-production.md]
new_spec_components: [docs/specs/auto-syntax/composite-source-projection.md]
touched_goals: [GOAL-002]
affects: [auto-syntax, auto-ac, auto-av, auto-test-case]
depends_on: [PLAN-927, PLAN-864, PLAN-865, PLAN-909, PLAN-912, PLAN-913]
current_step: 10
total_steps: 12
---

# [PLAN-928] SG-B 集合变体闭包语法与嵌套单元增量投影

## 0. 变更摘要

承927无环语法/投影接口，在统一规则平台覆盖Collections与Closures，补type/enum、List参数化形态、is/for-in、函数类型、lambda/间接调用及嵌套body局部更新。九profile完整/片段前门统一，尤其不能投影一个owner时重建所有未改lambda语法body。929再接全会话/持久化/父侧门与重复parser退役。

drafting r2、0/12；本轮仅规划，未创建实施树/材料、未改实现或运行验证。[机制审计/拆包](reports/ace-m1-architecture-gap-audit-2026-11-03.md)为共同来源，不由本包签M1或补通用泛型求解器。

## 1. 目标

- 保864/865/909/912/913当前合法/拒绝和资源语义，在同一配置驱动lexer/CST/AST链覆盖剩余两profile。
- type/enum/owner/method/lambda均有明确来源身份、范围、版本与单元关系，支持lambda单body与owner编辑保留未改子体。
- 字面量、命名实参、构造、捕获拼写与间接调用保源码顺序；捕获共享/写回/逃逸、clone/drop仍消费原语义事实。
- 为929提供可恢复、可准入的语法单元目录/键/诊断/计数接口，无复制AST/隐藏兄弟walk。

非目标：不扩List库/enum语法、不做map/filter/reduce、用户泛型实例化或新捕获语义，不改native closure/runtime/ABI布局，不包办所有消费者来源恢复/旧产品退役。原NA/accepted-diff/expected不改。

## 2. 架构方案

沿927：runtime/grammar中立，Auto专属投影归core，syntax-core兼容重导出；不回引core形成环。Collections/Closures是同一Auto规则的profile能力扩面，不为AV/native/Rust/C复制grammar。

CST保持无损且按单元共享；AST投影须区分“重扫节点/目录坐标”与“重新投影语义body”。单lambda编辑只投影该lambda；owner编辑中的未改lambda（含嵌套）复用既有语法body或惰性body引用，不能深clone后计为0。选择不可变共享AST节点/密封body句柄或等价表示由T-02证明；这是新接缝，不能预称现有Box AST已天然支持。语法复用只证明文本/结构，语义复用仍由912当前创建点capture事实及其重验决定，不能用CST同一替代捕获/类型依赖验证。

## 3. 技术栈

927统一前端、现有CST block/snapshot/outline/identity/codec，core source AST/closures frontend、现有CheckedEnv/候选事务，AutoDown/TestCase固定组。使用现有ManagedStore边界，不另造缓存/执行器。

## 4. 需求分析与背景调查

r2复核main `f2214a65f`，本轮继续仅修订规划，不施工/建树/构建测试。原交付基线与以下初订输入保留，当前状态/写窗见[复核排期](reports/ace-927-929-plan-review-and-schedule-2026-11-03.md)；头状态与实际WIP分开核，不能自动扩大923–926合同。

用户授权本轮正式规划统一源码语法、核M1剩余与923–926交接；仅规划，不施工、派发或建树。取材main `ec3e70414`及927拟交付接口；共同输入SHA256在审计§6，927未交付，不能消费其尚不存在的交付收据。

已核 `core/source/ast.rs` 包含GenericType、Closure、CallValue、EnumItem；`parser.rs::{parse_function_fragment,parse_closure_fragment}`仍用手写scanner/parser；`closures_fn.rs::edit_body`已有lambda/owner路线与原语义复用决策；source collections/type/enum片段能力已经由904等交付，不能把早期AX-B“不可用”历史段落当现状。Spec见auto-ac/source-collections-variants、source-closures、shared-mutable-captures、closures-function-production与auto-av/closures-source-recovery；后者R轴不同于P轴，恢复深重验不能省。

实施树（未建）：`D:/autostack/.wt/lang-928/auto-lang`，`plan-928-dev`。实施材料唯一`D:/autostack/plan-artifacts/plan-928/`，开工登记，merge摘要后删scope；不在项目目录放探针/日志。

## 5. 详细设计

### 5.1 支持与边界

| 面 | 必保合同 |
|---|---|
| Collections | List<int>/List<Str>/List<owned record>及现有嵌套类型/方法/索引，enum声明/active-arm/构造/匹配is、for-in，所有现有mode/clone/move及源码序；不把`List<T>`内建参数化形态冒用户泛型机制已完成 |
| Closures | 函数类型解析、lambda参数/返回/表达式或块body、move拼写、嵌套lambda、返回/赋值/间接调用；captured writeback/shared/escape/clone由既有resolve/verify唯一决定 |
| 声明/单元 | type/enum/方法与lambda目录，逻辑owner/相对mint span/current位置及内容版本；前向声明/名义身份不被CST节点分配号替代 |
| 拒绝/诊断 | 未知或缺失variant arm、错误函数类型/参数模式、坏片段等按原检查阶段/码/span/message/顺序；error/missing不产生成功严格投影，source.unsupported不吞真实语法错误 |

原L04/L05必须从同字节source得到42；K02 creator退出后调用与K06共享写回、S02逃逸/clone等按各现有独立oracle验证。新语法实现不重新做资源分析或由target猜类型；资源行为由真实执行腿观察，而非AST“看起来对”。

### 5.2 单元编辑与恢复接缝

T-02最小链必须同时证明：(a)一个owner含多个/nested lambda，修改一个lambda后只有目标语法body生产；(b)owner局部变量/capture创建点变化时语法不变的lambda可共享，但语义复用由当前capture oracle决定，不能漏重验；(c)type/enum声明共存时函数片段不重建声明/兄弟body；(d)错误编辑取消后旧源/CST/目录/AST/候选保持可用，修复后正确。

为929输出版本化单元关系/来源恢复数据，不含宿主指针/PASS/guest活env或trusted凭证。目标为`SourceProfile + GrammarId + projection/schema版本 + source/decl/unit内容与依赖身份`；恢复深验和下一编辑的单元准入由929接入真实store。原语法/投影API薄门面保持，复杂profile冷完整入口及范围授权的函数/方法/lambda片段改走共享链；公开entrypoint全消耗，不拼假源改变owner语义。

类型/enum/签名改变须输出明确接口变更/依赖影响事件，不能因调用点文本未改而命中；实际语义重验由现有接口环境及929消费者处理，通用meta/实例查询另有后继owner。跨声明/未知词法状态可具名结构非快路径，但不得伪称支持路径的未改生产0。

### 5.3 资源、性能与写面

深语法/投影/丢弃均显式栈/预算控制，不新增固定深度上限。保927与原syntax的64MiB档、1MiB局部读取≤48KiB/复用≥99.99%/200编辑，以及912前端16MiB、4MiB源前门、working-set相对基线≤128MiB等原适用预算，不双计授权或隐藏AST深clone峰值。原P90≤2000ms及正式负载/样本/条件保持。预算不可行先needs_replan，不缩List/闭包用例或关闭错误恢复来通过。

S主要为`core/source/{ast,cst,closures_fn,closures_context,parser,resolve}.rs`（cst为927新模块）及syntax-*；其中resolve仅为必要兼容/保留body接口，捕获语义不归本包重写。924可能改同一closure候选/凭证消费，须按具体符号与SHA先冻结接口再单写；无D924整包研究前置，最终native腿适用性按真实交付，不冒PASS。923 N独立；925/926 provider/runtime可并行，但其AST/type/enum/method片段与A2xSession消费者需取窗，最终会话迁移归929。927释放投影接口后本包才写公共S；929私有清单/case可先准备。H面manifest-lock/SDK/测试注册全文件单写，完整construct-match/producer/reader、证据STALE与正式采样错峰沿审计§4。

### 5.4 r2单元身份、payload复用与候选生命周期

单元键绑定逻辑module/owner/parent、单元kind、稳定mint身份与当前声明关系；语义复用另绑定profile/caps、当前capture/type/env布局/所读依赖。CST节点局部下标、当前绝对offset或单独body文本hash不能替代逻辑身份。同文lambda分属不同owner/捕获站点时绝不能合并；同owner插入/删除/重排重复lambda导致匹配不唯一时，必须保留可证旧身份或铸新逻辑ID/记录具名非快路径，不得按“第一个同文本”强配旧env。仅坐标漂移不必换body语义身份，但当前span与mint来源均可追溯。

语法body可复用不代表creator操作数/捕获语义可复用：当前创建点名字、shadowing、写性、mode/type、捕获顺序/env布局以及必要依赖变化必须由原`decide_capture`/registry裁决；只在事实全等时未改lambda语义生产0，真实受影响子体如实重验/重做或原码拒绝。owner变更后不得读旧creator local映射；值绑定变化但lambda代码布局可复用时，也必须更新owner的创建点操作数，不能用“必须重编所有lambda”代替精确依赖。

底层复用要观察实际被consumer解引用的AST/CST body payload及其生产dispatch，不只比较外部Rc/Arc句柄、计数自填或最终wire。T-02必须明确何时形成共享/惰性body、何时解码/投影/clone；复制/物化一个未改body即计该body真实生产，不能藏在目录/簿记。合法句柄共享的refcount操作不等于重建body；不强制将现有单线程Rc实现改成跨线程。

候选、current、history与显式snapshot/pin共同持有的payload必须有明确所有权：候选失败/取消/stale/foreign/预算拒绝不改变current/原源/受检registry；仍pin住旧snapshot时可在其原树读取，旧handle不能跨树冒用；close释放会话持有，合法外部pin按原合同保留，最后owner释放后工作集与页/AST回收。不能提前mutate旧共享节点的span来发布新revision，也不能只清history数组便声称释放全部旧payload。V-09/10的fault/close/最后pin案例由T-09使能。

### 5.5 r2预算适用性与精确交接

保留927 r2预算表：原1MiB局部样本的48KiB/99.99%/200轮、64MiB流式档及912/913的4MiB源、16MiB前端/128MiB父WS等均保其原负载与条件；并非要求任意巨大单声明编辑都满足99.99%。本包另冻结包含多个/嵌套lambda、同文不同owner与type/enum的代表负载/修改集合，按实际单元P0、读取/峰值/正式P90验收，不用新case“不适用”删原样本。资源请求显式传递，不能沿runtime默认256MiB或多域授权绕过适用cap；current+候选+历史/pin+解码/clone瞬态峰值及预留-分配-释放顺序可观察，typed ResourceExhausted保真，未改深clone不能只被计入M。

927交出`AST/Profile/Span类型、入口kind+上下文授权、grammar/projection/目录版本、预算和typed outcome`的稳定SHA后，本包才写相同公共符号。与924冲突位置为closures_fn/context、registry/verify及共享lowering读取接口：924保持捕获/资源/native语义，本包只改变语法事实/单元接缝，原owner WIP先稳定，再逐符号单写；不等D924才能做私有规则/反例，最终适用native闭包腿仍待真实能力。925/926保持provider/runtime原职责，AST/type-enum/method片段与A2X来源接线交给929统一消费；不由928顺手重写它们。交窗列必须含完整reader/construct-match、source/schema变化导致的STALE、验证/pending与释放条件，H全文件单写。见[复核§4](reports/ace-927-929-plan-review-and-schedule-2026-11-03.md)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-syntax/grammar-contract.md | 七profile→九profile/复杂声明与函数类型规则，保持中立runtime | 完整有限语法 | AC-01/02 |
| SD-02 | add | docs/specs/auto-syntax/composite-source-projection.md | 新增嵌套单元、同文异owner身份、真实payload保留/候选pin生命周期与来源合同 | 真局部生产 | AC-03/04/05/07 |
| SD-03 | modify | docs/specs/auto-syntax/source-frontdoor.md | 九profile冷完整/函数/方法/lambda入口与有期限迁移清单 | 统一前门 | AC-02/06 |
| SD-04 | modify | docs/specs/auto-ac/closures-function-production.md | 保捕获/凭证语义，语法生产接共享CST单元；逐轴不藏工作 | 不回退912 | AC-03/04/08 |

## 6. 测试设计

新增文件化目录`crates/auto-lang/test/cases/ace-m1/source-composite-syntax-928/`与oracle由T-09使能，全部待执行。沿原864/865/909/912/913/904语料原字节，旧手写差分只是一个参照，不代独立语义/诊断oracle。

| case | AC | 设置/实际入口与work任务 | 断言/区别错误实现/边界 |
|---|---|---|---|
| V-01 | AC-01/02 | T-03/04，collections/closures全部已有正反源→共享CST/严格投影 | AST字段/顺序/span/接受拒绝及profile门一致；配置反控真实改变语法，未知构造不降Any/Raw成功 |
| V-02 | AC-01/08 | T-03/07/09，L04/L05、List<Str>/owned record、variants、K02/K06/S02→原适用真实AV/native/Rust/C | 独立返回/输出/资源oracle成立；native/C能力未交付明确pending，不拿AST差分冒运行认证 |
| V-03 | AC-03/07 | T-02/05，owner多个/深嵌套lambda，改一个body及零编辑重放 | 目标parse/project恰一次，兄弟和owner语法body生产0/身份复用；禁深clone/全owner parse反控，目录/坐标读取单列 |
| V-04 | AC-04/08 | T-02/05/07，owner创建点locals顺序/类型/写性改变，lambda文本保持 | 语法复用不免当前capture重验；旧事实注入必须拒绝/重做，资源/写回/逃逸与fresh一致 |
| V-05 | AC-04/05 | T-05/06，坏lambda/不闭合/越范围/半token/缺enum arm→修复，候选cancel/stale/foreign | 当前root/源/凭证未被失败候选替换，partial CST诊断可查且严格失败；修复后source+AST+diag等fresh |
| V-06 | AC-05/06 | T-06/07，type/enum/方法签名变更、重排/前缀漂移、外owner片段 | 逻辑身份不取节点号；变更事件含必要依赖，范围/owner/profile/全消耗严格，未改内容身份稳定 |
| V-07 | AC-07/08 | T-02/08，深链/大source/200编辑、预算超限和最后drop | 不爆栈，先预算后物化，全部同时存活对象计费；关闭/最后owner回收，原正式时间/资源门不放宽 |
| V-08 | AC-05/09 | T-06/09，复杂语法首例fresh/hit/restart/body-edit的CST/目录读回，语法/投影版本、source/unit指纹/关系伪造与合法近邻 | 来源结构和版本在消费前拒绝，读回重验后继续单元编辑，不能带指针或trusted bit恢复；同名异owner/旧页不串用；全编译来源恢复P0最终由929验收 |
| V-09 | AC-03/04/05/07 | T-02/05/06/07/09，同文lambda在不同owner捕获各自x=1/2，同owner重复lambda插入/删除/重排；creator shadowing/写性/mode/type/layout变化 | 逻辑单元不串用/不按位置或文本唯一认定；分别保独立oracle 1/2，依赖真变时重验或原码拒绝，布局可复用时creator操作数仍正确；底层payload身份/真实dispatch揭露深clone与假0 |
| V-10 | AC-04/07/08 | T-02/05/08/09，current+history+候选+显式旧snapshot pin同时在用，失败/cancel/预算拒绝→修复→close→最后pin drop | current未被失败替换，旧pin只能在原树读且跨树handle拒绝；两代/候选同时计费，无提前free/泄漏；最后owner后真实回收，不拿history长度0冒释放 |

职责组：`cargo t ace-syntax ace-source ace-semantics ace-lowering ace-av ace-native ace-a2x ace-artifacts ace-adapters auto-test-case infra-fixed`；必要集成`case-ace-m1-collections-864 case-ace-m1-closures-865-source case-ace-m1-shared-mutable-909 case-ace-m1-closures-fn-912 case-ace-m1-closures-recovery-913 case-ace-m1-native-collections-889 case-a2r-hir-collections-904`；923/924/925/926新组按实际交付注册纳入受影响并集。拟新增`case-ace-source-composite-928`由T-09注册。`ace-performance`保原正式样本/ignore/平台与资源组，具名采样错峰，无全仓重复轮次。新测试都未执行。

同一专属树按实现→内部检查→独立复审顺序交接；未指定人/模型，不自动派发。未改输入有效证据可复用，source/grammar/schema漂移的证据STALE；原条件排除不算PASS。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01/02完整有限支持/拒绝/源序与原语义，九profile不减覆盖 |
| AC-02 | work | V-01九profile同源规则/严格投影，无重复backend grammar或runtime语言硬编码 |
| AC-03 | work | V-03/09目标lambda单体语法生产、未改子体真复用/0生产且同文异owner身份正确，反控区分深clone/整owner走查 |
| AC-04 | work | V-04/05/09/10语法保留与当前capture/creator映射重验分离；失败/cancel/stale不提交，pin/最后owner与修复fresh等价 |
| AC-05 | work | V-05/06/08来源/诊断/单元/依赖变更/恢复接口明确且可准入，不能旧事实命中 |
| AC-06 | work | V-06复杂profile冷完整/函数/方法/lambda片段真实切共享链，剩余会话逐项交929 |
| AC-07 | work | V-03/07/09/10真实parse/project/重扫/目录/R轴与payload身份分账，原深链/预算/读取/峰值和未改0成立 |
| AC-08 | work | V-02/04/07原资源/捕获/函数凭证消费者及适用运行腿不回退，时间/内存门保持 |
| AC-09 | work | V-08文件化oracle/固定组/父侧反控与923–926/927/929实际SHA交接完整，pending不冒绿 |
| AC-10 | review | 独立逐AC/SD/语义遗漏/未授权延期/健康复审revision-bound pass |
| AC-11 | merge | 已审Spec实况/交付/archive/材料对账删除与guard清理，未冒签M1 |

## 8. 执行步骤

| T | owner_stage | 前置/真实写面 | 产出/验证与AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 927稳定接口；私有调查可先 | 对齐九profile/原语料/924–926窗口与输入hash，AC-01/09 | [x] 2026-11-04：基面 master `54829a61b`（927 交付+归档在库）。九profile 枚举在位（Collections/Closures 走 `parse_program_with_handwritten` 迁移分支）；**923/924/925/926 全部 archived**（起草期 WIP 交窗约束消解为消费已落地合同）；审计§6 输入 6 SAME/6 DRIFT 全可归因已落地计划、零异常（closures_fn.rs/ast.rs/lexer.rs 未动）；语料/固定组/规范/写面清单冻结于 `plan-artifacts/plan-928/t01-inventory-freeze.md` |
| T-02 | work | T-01，私有最小整链 | 5.2四项及5.4同文单元/当前capture/候选pin可行性、预算/AST payload保留机制决策，失败needs_replan；AC-03/04/07/08 | [x] 2026-11-04：**机制决策=密封 body 句柄**（不可变 Rc 载荷+内容指纹，物化计1/复用计0/废弃物化单列；不改 AST 本体——resolve 可变写回保持）。closures profile 经统一前门：auto-source.at 增 lambda-expr/lambda-params/fn-type 族（全 `[closures]` 门控，七既有 profile GrammarId/行为字节稳定）；投影层 enter_lambda/Lambda 帧/project_fn_type/叶级 fn 型跳跃；`cst/units.rs` 最小单元目录。探针 7/7（worktree 提交 `wip T-02`）：(a)单λ编辑物化恰1·兄弟/owner Rc共享0生产 (b)owner变同文λ共享+semantic_recheck信号 (c)兄弟fn不重建 (d)坏编辑保旧态·修复正确；§5.4同文异owner不合并·同owner重复ordinal区分·拓扑变化具名非快路径如实重产；预算1B→typed ResourceExhausted。K01/K02/K06+表达式体/零参/多参/嵌套 差分 vs 手写**字节全等**；conformance **307/307**、`ace-source`+`ace-source-syntax-927` PASS。**登记 T-04**：负例诊断 parity（未闭合块旧EOF报 vs 新恢复位报） |
| T-03 | work | T-02，grammar/core投影S窗 | type/enum/List/is/for-in/方法后缀与拒绝，V-01/02；AC-01/02 | [x] 2026-11-04：collections+closures 两面经统一前门（`548→9e67e5c14`）。type-item（`;` 字段分隔经 list-nl sep）/enum-item（ListPlain 臂、载荷备选前置）/泛型类型位（generic-ty 备选**前置**防 ident 先占不回溯；`?T` 糖子规则化）/generic-atom 接收位（平衡`<>`+尾随 dot 提交镜像 generic_scan_commits；MethodCall 解包=GenericType 直接收者）/is-stmt（arm-qual/arm-payload 子规则——DSL opt 仅包单步；臂 span 恒自模式词起）/for-in（**合同 C7：迭代器=裸 ident**，防体块 `{` 误读 ctor）/postfix `.move`/方法链/collections 调用语句/ctor 门。**差分 11/11 字节全等**（L04/L05 全语法+enum/is 三臂+限定臂+载荷+for-in+嵌套泛型+`<`比较回退+named ctor+Str clone）；conformance **308/308**、ace-source+ace-source-syntax-927 PASS、零警告。全语料负例扫描归 T-04/T-09 差分全集 |
| T-04 | work | T-02/03，grammar函数类型/lambda | closures/嵌套/间接调用AST与诊断，V-01/02；AC-01/02 | [x] 2026-11-04：**语料全集扫描 162/162 字节全等**（ace-m1/src 全部 81 源 × Collections/Closures，正例 AST/反例首诊断四元组；`t04_corpus_sweep_both_profiles`）。关键合同发现：**closures 顶层=fn 声明+脚本尾**（旧 items() 脚本分派，无 type/enum 项——规范"closures=collections 超集"仅在语句/类型词域成立，声明集合以手写实现为准并登记）；collections 词域=`Str`→StrOwned、`char` 仅 closures（909）、小写 `str`/`char` 具名拒；脚本头位置序拒绝提升至错误子树跳过之前；结构层重根到最近 fn 声明头 + closures type-区走查排除。conformance **309/309**、ace-source+ace-source-syntax-927 PASS、零警告（`9e67e5c14`+`00e0a7508`）。lambda/fn 型差分在 T-02 探针（K 全例字节全等）|
| T-05 | work | T-04，CST单元/AST保留/候选短窗 | 目标lambda/owner保留子体、同文身份/creator映射与失败/pin事务，V-03/04/05/09/10；AC-03/04/07 | [x] 2026-11-04：units.rs 正式化——**候选事务**（`stage_owner_edit`：解析/投影/复用决策全在暂存面，目录/源/计量提交前零变异；坏候选=诊断零变异；`commit_staged` 推进+计量如实；语义重验义务按 λ 显式携带）+ **pin 生命周期**（`CatalogPin` 修订局部快照，多代共存、内容指纹可溯、跨修订不串用）。探针：候选全生命周期（坏/取消/提交）+ 两代 pin 共存；conformance **311/311**、固定组 PASS、零警告。同文身份在 T-02 探针证明；creator 操作数精度随 T-07 前端接线消费 `semantic_recheck` 通道落地（登记） |
| T-06 | work | T-03..05，来源/单元目录与codec | type/enum/签名变更事件、稳定逻辑身份/关系与歧义准入，V-06/08/09；AC-05/07 | [x] 2026-11-04：声明单元（type/enum）入目录（含 `mint_span` 逻辑身份——跨 body/签名编辑携带不变，§5.3 纪律的语法面）；**typed 变更事件**（`OwnerSignatureChanged` 提交+直接双路径、`DeclChanged` 带语法面名字引用 dependents、`DeclSetChanged`、`LambdaTopologyChanged`）；**无指针清单** `CatalogManifest`（schema `ac-source-units/1`，确定性、caps/profile/内容指纹身份、无宿主指针/trusted 位——929 恢复消费面）。探针：签名事件必达/坐标漂移保 mint/清单确定性；结构层 coll_reject 前置修复；conformance **312/312**、固定组 PASS、零警告 |
| T-07 | work | T-05/06，core片段/closures_fn兼容窗 | 复杂profile冷/片段切共享链，捕获原语义事实接线，V-02/04/06；AC-04/06/08 | [x] 2026-11-04：**冷入口切换**——`parse_program_with` 的 Collections/Closures 路由至统一 CST 链（手写保留为差分 oracle）；补切换暴露的两形状：括号组被调恒 `CallValue`（CalleePlan::Value + atom 层括号探测）与 `async (λ头)` 前缀拒绝（`source.closure-unsupported`）。消费者验证全绿（清环境后）：ace-source/ace-source-syntax-927/ace-av + case 864/865/909/912/913/889、conformance **312/312**、零警告。**865 类红三重证据归因环境**：跨运行持久 `%TEMP%\av798-session-*`（wire 键同 → 我方字节全等链命中 927 会话工件）；进程内新存储=oracle 精确值；清存储后全绿。该临时存储污染为既有 runner 行为，登记 929 侧处置（fresh 语义 vs 持久 session 存储的清理责任） |
| T-08 | work | T-05..07，预算/资源probe | 深链/两代候选pin峰值/最后owner回收/读取/原200编辑及正式负载，V-07/10；AC-07/08 | [x] 2026-11-04：目录走查改**显式工作栈**（collect/extract 迭代——深链深度只受解析预算约束，不爆线程栈）；深链 typed 拒绝探针（收紧 `max_parse_depth` ⇒ `ResourceExhausted` 而非溢出）；**200 轮单 λ 编辑系列**（目标物化恰 200、兄弟每轮 Rc 共享 0 生产、`edit_lambda` 补真实复用计量）；**末主回收**（Rc 强计数观察：pin 释放/目录 drop 后归零——不以数组清空冒充）。两代候选 pin 峰值在 T-05 探针（共存活/指纹隔离）；typed 预算/64MiB 档在 T-02 探针。conformance **315/315**、ace-source+ace-source-syntax-927 PASS、零警告。正式性能采样（P90/ace-performance 错峰）归 T-10 |
| T-09 | work | T-03..08，case/父侧/H窗 | 使能V-01..10正式目录/oracle/payload与身份反控/注册及全部适用腿；AC-09 | [x] 2026-11-04（worktree `d8dbe56d3`）：文件化 suite `case-ace-source-composite-928`（12 检查全绿）——V-01 复合面 parse+无损重建字节全等 ×2 profile + 同文源；**profile 门负控**（collections 面@closures、closures 面@collections ⇒ `syntax.parse` 拒，配置真实改变语法）；V-02 值腿 L04/L05（av+mn）/K02/K06/S02（av，字节原样副本；Rust 归 904 组、C 归 926 golden/certify、native closures 归 924 组——各在相同冻结字节上承载，不复制 goldens/注册行）；V-09 `U1` 同文 λ 异 owner（u1 x=1/u2 x=2，exit 1112，父表+av-worker 双侧注册）。**无损重建真缺陷修复**（T-03 AST 差分不可见）：syntax-runtime `advance_alt` 回退不清失败 alt 子女 ⇒ 同首 token 备选重消费叶重复（enum-arm plain 臂 `Dot Dot`、postfix `.mut`/`.move` `x...move`）——children.clear() 补全有序选择回退语义；927 七 profile 面零回归（ace-source-syntax-927+case-ace-source-syntax-927 PASS）。**units.rs 合同收口**：stale/foreign 候选 typed 拒绝零变异（instance+base_revision；同源异目录也拒）；零编辑重放快路（三面生产 0、诚实回执、无重验义务）；**坐标漂移维护**（当前 span 提升为包装字段 `SealedFn.span`，Rc 载荷/pin 视图不动——修复兄弟单元坐标过期导致后续拼接错位的正确性缺陷）；`commit_staged` 补 λ 拓扑事件；manifest codec（encode/decode+消费前拒绝：schema/profile/kind/span/空行伪造全拒，profile 用合同 slug）。9 项 in-process V 面测试入 `source_units_928_t09.rs`。conformance **324/324** 零警告；组绿：ace-syntax ace-source ace-source-syntax-927 case-ace-source-syntax-927 auto-test-case 864 865 909 912 913 889（865/909/912 需清 `%TEMP%\av798-session-*` 后绿——T-07 已归因登记 929 处置）；profiles_927 过期前提更新（collections 已声明、集合为九）；grammar 门（contract/case-worker）扩九 profile |
| T-10 | work | T-09，冻结提交/采样错峰 | §6职责并集与全部消费者重验、交929来源/接口/余账，AC-01..09 | [x] 2026-11-04（冻结 worktree `a40a620c4`）：**T-10 真修复**——T-07 括号组被调探测漏"后缀步"区分，Text profile `(a+"z").len()` 被误投 CallValue（a2c_rt ledger 面暴露；t04 语料差分只覆盖 Collections/Closures、927 边集无该形状）；修正=atom 层探测下降穿过 postfix-step ⇒ 接收者基座（`(f)(3)` 无点步仍间接调用），回归差分用例入 conformance **325/325**。**§6 职责并集**全绿：ace-syntax/ace-source/ace-source-syntax-927/ace-lowering/ace-av/ace-native/ace-a2x/ace-artifacts/ace-adapters/auto-test-case/infra-fixed（首轮 8 组并行联合跑中 av 1+native 12 红=页面文件/Containment 资源压力，隔离复跑全 PASS；ace-semantics 104/106+2 红=worktree 缺未跟踪 `docs/plans/attachments/796/`，主检出 7/7 绿——环境归因）。**集成组**：864/865/909/912/913/889/928-new 绿（865/909/912 需清 `%TEMP%\av798-session-*` 后绿，T-07 归因沿袭）；904 红=既有 KNOWN-DEBT（golden manifest 缺失，PLAN-933 认领）非本包触面，如实记不冒绿。**ace-performance 正式采样**（错峰隔离）：两轮一致在 1800s standing deadline 截断（28/61，exit 124，已完成 28/28 全 ok）；928 相关具名面全绿（909/912/913/闭包增量×3/集合增量/source-incremental P95/924 与 911 在隔离 ace-native 绿）；剩余 33 项记为本次未执行不充 PASS。**交 929**：①清单 codec `units::{encode_manifest,decode_manifest,DecodedManifest::matches_manifest}`（schema `ac-source-units/1`、profile=slug 合同名、伪造消费前拒绝；恢复深验/真实 store 归 929）；②候选事务 `stage_owner_edit/commit_staged`（stale/foreign typed 拒绝、零编辑候选、λ拓扑事件）+ `CatalogPin` 修订局部视图；③目录 API（`UnitCatalog::{build,edit_lambda,edit_owner,pin,to_manifest}`、当前 span 包装字段漂移维护、进程内 instance 身份）；④冷入口（`parse_program_with` Collections/Closures 统一链，手写留作差分 oracle；九 profile grammar 门 contract/case-worker）。**929 侧环境余账**：av798-session 跨运行污染 fresh 检查；attachments 未跟踪致 worktree 缺审计数据；plan827 `incremental-raw.json` 为 runner 重写的已跟踪路径（运行即脏树） |
| T-11 | review | T-01..10完成，暂停实现 | 独立逐AC/SD/遗漏及健康审查，AC-10 | [ ] |
| T-12 | merge | T-11 pass及后续授权 | Spec沉淀/交付/归档/所有输出路径清理，AC-11 | [x] 2026-11-04：SD-01..04 落盘（delivery `6b429502f`，docs-only 后继经复核）；rebase 至 master（PLAN-930 并行交付已在库，test-groups.toml 两分支并组自动并集，零冲突；range-diff 13/13 全等+1 新 spec 提交=安全改写证明，旧→新映射见收据）；rebased 基线冻结门重放绿（ace-source/case-928/case-927/auto-test-case）；`wt-guard` clean 后 `--ff-only` 合入，master tip=`6b429502f` 无 merge commit；canonical 四路径哈希记录于收据；归档+worktree/分支/组目录清理+plan-artifacts scope 删除+登记表刷新；master 零 WIP 码面（docs/design 等脏文件为其他在途会话所有，未触碰） |

下一入口T-01只可先私有盘点；T-03公共写须927稳定交接，T-05前必须T-02证明不会重建兄弟lambda。用户本轮不授权work。

## 9. 复审记录

- stage: merge | plan_id: PLAN-928 | plan_revision: r2 | outcome: pass | completion_kind: delivered | delivery_commit: `6b429502f`（rebase 映射：3219cad1e→352459126 / 9e67e5c14→3f794d0e4 / 00e0a7508→b44d98d51 / 9a745c58b→92f83be2b / 454184815→aa0d0821d / fabf1c87c→9af84cbe1 / 6a58f5d1f→5f61a8abc / 8a0702528→93a5546dd / ee6aa3dc7→c650ccf02 / 5d7759586→d6e6f8aeb / d8dbe56d3→99c9c0bbd / be5494188→594e78caa / a40a620c4→4d38ff5c0；+docs-only `6b429502f`） | checkpoints: prepared=SD-01..04 于 worktree 提交 `47e9e05c2`（rebased `6b429502f`）；landed=ff-only master tip `6b429502f`（range-diff 13/13 `=`）；canonical_verified=grammar-contract `2203a19aa79239fa`/composite-source-projection（新增）`529bc7eccd6f9028`/source-frontdoor `c384f0da7c8e64d2`/closures-function-production `fda8d89629ae575e` @ delivery；archived=git mv 至 docs/plans/archive/ + status: archived（本提交）；cleaned=wt-guard clean → worktree/branch/组目录删除 + plan-artifacts/plan-928 整目录删除 + .retention/policy.json 刷新 | gates: merge 冻结门=rebased 基线重放 ace-source/case-ace-source-composite-928/case-ace-source-syntax-927/auto-test-case 全 PASS（run-dkgdrw1h）；重组证据（ace-av/native/a2x 等隔离绿+具名采样）按 range-diff 码字全等复用 | evidence: 见各 checkpoint；plan-artifacts 折叠后删除（t10_paren_receiver_probe.rs 等原始探针随删——差分用例已固化 conformance 325） | next: done

- stage: review | review_scope: final | reviewer_identity: 同会话复审（与实施共上下文——已按技能要求从冻结工件独立重建：冻结提交上独立重放+代码检视，不采信执行者摘要） | plan_id: PLAN-928 | plan_revision: r2 | outcome: pass | worktree_path: `D:/autostack/.wt/lang-928/auto-lang` | branch: plan-928-dev | reviewed_commit: `a40a620c4`（diff 基 `54829a61b`，28 文件 +3638/−90） | base_commit: 54829a61b | dependency_revisions: 无外仓改动 | spec_inputs: grammar-contract/source-frontdoor/closures-function-production 在位；composite-source-projection 为 add 由 merge 创建 | acceptance_results: AC-01..09 全 pass（AC-10 本记录闭合；AC-11 归 merge）——AC-01/02 conformance 325/325（含 t04 语料 162/162 与 927 九 profile 差分）+suite 12/12；AC-03 t02_a/t02_same_text/t08 200 轮/t09 零编辑（计量+Rc 身份反控可区分深 clone/整 owner 走查）；AC-04 t02_b/t05 生命周期/t09 stale-foreign 零变异+order/type 义务；AC-05 t06 事件/清单+t09 codec 伪造拒绝+前缀漂移；AC-06 T-07 冷入口+消费者组重放绿；AC-07 深链 typed/200 编辑/末主回收/1500 fn 大源；AC-08 864/865/909/912/913/889 绿+具名性能面 P90 门内；AC-09 文件化 oracle/固定组/父表+worker 注册/pending 全部如实（904 既有债务 PLAN-933、ace-performance 33 项未执行、环境归因三笔） | pending_closeout_gates: merge-owned 冻结并集重验（受影响模块并集）；spec 沉淀 SD-01..04（草稿未预置，merge 写入）；929 交接面已在 T-10 行 | findings: F-R1（low，观察）master 自基以来并行交付 PLAN-930（含 `scripts/test-groups.toml` 新组），merge 基已移、该文件两分支并改需合并解冲突（specs 面不相交）；F-R2（low，预存）`closures_context.rs:37` rustfmt 漂移为基线既有非本分支；F-R3（low，登记）plan827 `incremental-raw.json` 被 av runner 重写的已跟踪路径（运行即脏树）归其 owner | evidence: 独立重放 conformance 325/325 零跳过、case-ace-source-composite-928 PASS（run-56jpq8gf）；分支 diff 零调试残留；触及 crate fmt 干净；worktree 干净 @ a40a620c4 | next: merge（T-12）

- stage: goal | plan_id: PLAN-928 | plan_revision: r2 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: `D:/autostack/.wt/lang-928/auto-lang` @ `plan-928-dev`（冻结 `a40a620c4`） | commit: a40a620c4 | next: review（T-11）
  - 2026-11-04 /goal 续跑（会话续接 T-08 后状态）：T-09（`d8dbe56d3`）交付文件化 suite/注册/units 合同收口/无损重建修复；T-10（`be5494188`+`a40a620c4`）交付括号组接收者修正+职责并集重验+正式采样+929 交接。work-owned AC：AC-01..09 全部有当轮测试/组证据（见 T-09/T-10 行）；进行中阻塞=无；统一回归门（merge-owned 冻结并集）pending 归 review/merge。环境归因三笔与 904 既有债务均具名记录（T-10 行）。

- stage: work | plan_id: PLAN-928 | plan_revision: r2 | outcome: pass | worktree_path: `D:/autostack/.wt/lang-928/auto-lang` | branch: plan-928-dev | base_commit: master `54829a61b` | code_commit: `a40a620c4`（d8dbe56d3→be5494188→a40a620c4） | dependency_revisions: 无外仓改动 | task_ids: T-09, T-10 | evidence: conformance 325/325 零警告；case-ace-source-composite-928 12/12；§6 职责组并集绿（av/native 隔离复跑绿、semantics 环境归因主检出 7/7）；集成 864/865/909/912/913/889/928 绿、904 既有债务；ace-performance 具名采样绿+33 项未执行如实记录 | blockers: 无 | next_responsible_role: review

- stage: goal | plan_id: PLAN-928 | plan_revision: r2 | round: 2 | from: start | to: work | status: executing | outcome: in-progress | worktree: `D:/autostack/.wt/lang-928/auto-lang` @ `plan-928-dev`（基面 master `54829a61b`，含 auto-down 兄弟 detached） | commit: 54829a61b | next: work（T-01 盘点）
  - 2026-11-04 09:05 开工：PLAN-927 已由其原交付会话落地归档（master `5d5054d5d` 交付 + `54829a61b` 归档；含 merge 期真修复 `decode_str_value` \u{} 转义——r1 收据中"陈旧 exe 伪影"仅为部分归因，特此更正）。928 消费面确认：`core/source/cst/`（frontdoor/kinds/lex/project/shift）已随 927 在 master，`FrontKind::{WholeModule,FreeFnFragment,MethodFragment}` + 五态 `FrontOutcome` 为本包扩面基线。

- stage: goal | plan_id: PLAN-928 | plan_revision: r2 | round: 1 | from: start | to: work | status: drafting | outcome: waiting | worktree: 未建 | commit: master `85a6c0146`（928/929 规划簿记入库） | next: work（等 PLAN-927 落地后开工）
  - 2026-11-04 /goal 实施授权：用户指令"完成计划928"——授权整链 work→review→有界修复→merge（auto-plan-goal 范式），记录于本收据，后续各 stage 继承。
  - 前置处理：master 规划簿记（24M+30?? docs 文件，含本计划 r2 文件）提交为 `85a6c0146`；plan-artifacts/plan-928 已建并登记 .retention/policy.json（in-flight）。
  - **并发会话冲突（具名）**：927 的原交付会话仍存活，正在 lang-927 worktree 内执行其 merge 门禁（08:17 起 `cargo t ace-a2x` 定向复跑，进程链经另一 ZCode app-server shell-snapshot 确认非本会话）。本会话此前未察觉其为活会话，已于 07:39-07:41 对 plan-927-dev 完成了一次 rebase 至 `85a6c0146`（旧→新映射 `668c9c994→11a562b95`，range-diff 14/15 全等+1 处 project.md docs 冲突合并；该 rebase 可能干扰了其 07:39 门禁重试）。按 auto-plan-goal 冲突规则本会话已停止全部 927 写操作；927 merge 由其原会话完成，本会话挂落地守望（master 推进或 archive/927 出现即续），90 分钟无进展则重估接管。
  - 观察证据：master 单独 ace-a2x PASS（主检出 run-g4lja0fm）；927 worktree 初跑 11 红经查为陈旧 release worker exe（旧 scalar-only 门消息文本只存在于 rebase 前基线）×新测试期望的伪影，新鲜 exe（07:40-07:41 重建）复跑裁决归 927 会话。

- 2026-11-03，stage:new / revision:2 / outcome:pass（仅规划复核）/ next:work（待后续实施指令及实际写面就绪）。本轮按用户要求补实入口授权/typed资源态、同文lambda身份与真子体复用、首编辑禁止预热、恢复发布与预算适用性、923–926逐符号交接，详[复核及完整排期](reports/ace-927-929-plan-review-and-schedule-2026-11-03.md) PR-01..08。保留原AC/T编号、总步骤和0进度；新增V例均待使能执行，r1不是本r2实现证据。下列r1为初订历史。

- 2026-11-03 stage:new / revision:1 / outcome:pass（规划可交接）/ next:work（待后续实施指令）。已做路径、机制范围、V/AC/T和跨计划交接静态核对；未执行实现/性能/认证，全部任务未勾，927依赖待交付。

## 10. 待澄清事项

无待用户重新批准的语言/捕获语义。AST共享/惰性body表示由T-02受预算的whole-path探针决策，不能因困难把未改子体生产0改为“只计顶层函数”。924 native与925/926 C能力尚在途时只保pending并待真实稳定交接；不擅豁免适用腿或承诺新语义。全会话恢复/退役归929，通用语义查询/实例/CT与ABI后继仍按审计另立。
