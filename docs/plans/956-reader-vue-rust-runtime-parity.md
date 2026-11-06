---
plan_id: PLAN-956
status: reviewed
feature_name: Reader Vue Rust backend parity
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 4
current_step: 5
total_steps: 6
supersedes_spec_components: [docs/specs/auto-lang/trans/rust-semantic-admission.md, docs/specs/stdlib/design/primitive-type-of.md, docs/specs/stdlib/design/primitive-str-byte-len.md, docs/specs/auto-vm/zip-natives.md]
new_spec_components: []
touched_goals: []
---
# PLAN-956 Reader Vue + Rust 后端能力与 VM 对拍
## 0. 变更摘要
用户明确要求继续补齐默认Rust后端，不能只换VM。归档955仅诊断/可用入口，本计划承接真正的Rust执行能力。
## 1. 目标
AC01(work): Reader同一back源码实际A2R/API生成、Rust编译成功，默认Vue+Rust启动，42条路由保留；禁止VM执行桥/脚手架替代业务。
AC02(work): VM原生type_of及Reader用到的动态JSON、跨模块签名/类型、标准库能力在Rust侧语义正确；普通未知调用仍拒绝。
AC03(work): 隔离同源夹具、同序API操作，对拍Rust/VM响应与持久化；覆盖导入/重复/中文/章节/进度/标注/搜索/导出/元数据/组/移除恢复/坏输入/重启恢复。非确定ID/时间须具名规范化，不抹业务差异；route清单42匹配。
AC04(work): 有永久测试入口，关键原语正反例，原VM oracle与数据不被弱化；Vue前端生产构建与Rust后端均真实成功。
AC05(review): 冻结代码与规范、差异清单和全部AC独立重建复核；merge负责实际受影响固定职责组并集和收尾。
## 2. 架构方案
保持共享Rust转译→a2r-std运行时→Web生成器→原生服务路径。先 bounded inventory，输出真实转译/编译错误族，再修复各owner的共享机制。新增能力以既有VM行为及Reader接口契约为基准，遇到VM缺陷不复制数据损坏，具名核对双方与独立oracle。
## 3. 技术栈
Rust auto-lang-core/a2r-std/auto-generation-core；auto-web API generation；auto-ui工具consumer；auto-reader行为门。
## 4. 需求分析与背景调查
授权：用户2026-11-06明确继续实现Rust后端、构建运行及VM一致性，可执行必要跨仓修复与验证，不再以VM切换算交付。未要求额外审批/模型/预算。
依据：docs/specs/overview.md、docs/specs/auto-lang/trans/overview.md、rust-semantic-admission.md；reader docs/research/20261005-t00-runtime-capability.md §12，src/back全部模块、tests/spec与tests/basic的现有oracle。
955确认 first error type_of；现有代码已有JSON Value跟踪和a2r标准库，历史缺口清单仅线索，不当作今天事实。先跑现行源码生产入口，使用准确symbol/module诊断。
主检出既有他方docs WIP、reader两个未跟踪assembly文件保留。本计划只提交自身文件。唯一实施组 D:/autostack/.wt/lang-956/{auto-lang,auto-web,auto-ui,auto-reader,auto-down}，各repo唯一worktree；branch codex/plan-956-dev或repo后缀，依赖只读。材料 D:/autostack/plan-artifacts/plan-956/。
## 5. 详细设计
T01 bounded probe：真实全部back转译/完整Web生成，收集错误族与闭包；选择等价实现并登记，若需要改变用户可见语义先依据独立oracle解决。
T02 语言/stdlib能力：裸type_of、动态JSON类型和值消费、必要调用/类型解析，保留typed静态准入；正反例从真实语法构造。
T03 API组装：完整back模块声明、引用/依赖闭包与Rust构建，禁止fallback/default空业务/VM委托。
T04 双轨永久对拍与真实默认Vue+Rust运行/生产构建。
### 规范增量
| ID | 操作 | target | before/after | 理由 | AC |
| SD01 | modify | docs/specs/auto-lang/trans/rust-semantic-admission.md | type_of/Reader原先VM-only边界按实际完成能力更新；严格未知符号不变 | Rust真实支持 | AC01/02 |
| SD02 | modify | docs/specs/stdlib/design/primitive-type-of.md, primitive-str-byte-len.md; docs/specs/auto-vm/zip-natives.md | type_of 原生Rust/宽整数；字节索引迁移；共享纯Rust ZIP | 双轨合同 | AC02/04 |
| SD03 | modify | auto-web/docs/specs/http-contracts.md | 多路径捕获按路由顺序/精确名称 | 实际文件端点 | AC01/03 |
| SD04 | modify | auto-ui/docs/specs/runtime/document-surface-boundary.md; docs/specs/build/desktop-api-surfaces.md | strict schema 与既有宿主 leaf；Cargo 构建先于 ready、原生发布镜像从应用根运行 | 默认前端构建/冷启动 | AC01/04 |
| SD05 | modify | auto-reader/docs/specs/reader/real-library.md | 默认Rust、显式类型/修改约定、保留旧ph1的i32回绕定义、永久HTTP门 | 已有数据兼容 | AC01/03 |
## 6. 测试设计
V01：真实Reader生成物cargo check/build及原生server ready+42routes，HTTP不是VM进程；无AUTO_A2R_BODY=0/scaffold绕过。
V02：type_of的int/float含0.0/string/bool/null/array/object，独立期望与VM对拍；JSON字段/数组/null/缺失/嵌套值及消费型转换永久正反例。
V03：Reader现有HTTP规格与独立VM/Rust序列对拍，响应状态/JSON业务值/文件字节/重启后值作为oracle，覆盖AC03。资源夹具只写隔离数据；非确定值规范化清单可审查。
V04：必要完整固定组至少a2r、a2r-std、auto-generation-core与实际受影响http/api组；Web/UI使用各owner相应测试。按触面明确并集，不把scope测试称全仓回归，不降低原样本/门禁。
实施和最终复核顺序在同一树；不创建未授权agent，当前上下文复核须明示限制。
## 7. 验收标准
AC01~05均须真实通过；只生成Rust或只过compile不满足行为一致。新旧tool/源码身份绑定，失败不记PASS。
## 8. 执行步骤
- [x] T01(work) 真实缺口清单与机制决策。
- [x] T02(work) 语言/标准库能力修复与永久原语反例。
- [x] T03(work) RustAPI实际生成/构建/启动。
- [x] T04(work) Reader双轨对拍/重启/前端构建。
- [x] T05(review) 冻结最终验收与规范复核。
- [ ] T06(merge) 完整职责门、合入与安全收尾。
## 9. 复审记录
stage:new | r1 | pass | 用户已明确授权当前范围；next:work，不再请求同范围审批。
## 10. 待澄清事项
无；具体错误族由T01调查，不预设通过删除业务/改用VM完成。

## 11. 执行记录（r2，2026-11-06）
T01：11 个 Reader back companion 中仅 library 在 type_of 首次准入失败；补齐后真实 Rust 初编 559 错误。错误族为动态 JSON 推断/字段/条件/null、i64 窄化、缺失标准库映射、列表参数共享修改约定、跨函数文本后处理污染、多路径捕获漏参及嵌套 try return。03 降到 21，04 降到 2（List<int>.new 结果类型推断缺失）。证据 scope 下 generate-rust-NN/rust-check-NN.log。
机制：ZIP 预算读取算法抽到 a2r-std，VM 仅保请求根解析适配；Rust 无解释器调用。字符串 find/substr/slice 统一 VM 字节索引/UTF8 内夹，char_at 仍字符索引，需同步旧 PLAN-885 规范与断言。Reader 动态值形参显式 JsonValue、共享修改参数显式 mut，保留算法而非删除 inflatex。
r2 有界扩展：实际严格前端构建遇到既有 doc-surface 未登记（955 已留首次红）。在唯一 auto-ui sibling 建 codex/plan-956-ui，登记既有宿主 leaf 的 schema 与 web div 映射/责任表，Windows 原生仍需 feature，不声称提供引擎。满足 AC04 默认严格构建的必要修复；不降低 schema 门。
开发 std-full-01：115/115 执行通过，但执行期间代码有变导致身份门判 false，不能用作最终收据。最终固定代码重跑。

r3 运行实证：generate-rust-06 + rust-build-04 已真实构建完整 Rust API。parity-01 发现可选 null 拼接被误写为文本 null；按 VM 的 null 拼接可捕获 TypeError 修复。parity-02 发现 ph1 依赖 VM int 32 位回绕，不能改已有身份；Reader 纯 .at 安全分解运算保留该定义，独立 Python 数学 oracle 与旧 VM 三夹具黄金值完全一致。parity-03：40 HTTP case + 42 route 清单 + 冷重启 + 23 个持久化文件字节一致，仅规范化 ID/时钟/隔离根/导出文件名。该轮为开发实证，格式化/最终提交后重跑。
类型谓词保留原规范 JSON 数组 object 标签；补齐原先未分类的 VM i64/u64/BigInt 整数标签到 int，直接纳入双轨值域测试。types-parity-01 首次暴露旧byte_len测试i32/i64比较未扩大，已无损修正；02单测执行绿但期间继续修订源，身份门无效，待冻结完整组。
前端严格校验已越过 doc-surface；默认完整构建先因 JsonValue 参数被误选 as_int_str 触发签名证据门，已修正 Value 分派，保留门而不降级。CLI 自动创建 deps 链接，不再在 worktree 继续做前端包安装；使用 scope 下经69文件逐SHA核对的 default-consumer 快照做真实默认入口复验，产物仍归同一scope。已有工作树自动生成deps链接需具名保留/安全处理，禁止递归穿透。

### r4 冻结门与默认冷启动收口
- Lang 功能冻结 58a1dcb9206a9b747cd18225002557a5b8e60fd8（92320ae4 后仅补新模块指纹登记与三行借用金样）；Web 0550aadd8be9362c89487ffcc0864842c13b85d5。
- frozen-groups-01：首次红四项。F01 dynamic_json.rs 漏显式 Vm provider 登记（BUILD_INPUTS 虽已有闭包，域目录完整性门仍要求具名登记），已补；F02 三行文字金样变化分别为 String loop owned coercion、&String comparison as_str、Version::parse 显式借用，逐行审查后只更新三行，不改测试源/oracle意图。
- frozen-groups-02：cargo t a2r a2r-std stdlib vm api http auto-generation-core；gate_passed=true、source/config/inputs 未漂移；1508 + 115 + 32 = 1655 实际执行全绿，原 ignored 保留。279.82s 整命令；准备/执行明细随最终收据折叠。额外 core-value-helper-frozen-02：1/1 PASS，gate_passed=true。
- Web api_gen 单元53 PASS、1原ignored。UI schema_loader 7 PASS；扩大 auto-ui-compiler lib 为1144 PASS/3 FAIL/2 ignored。F03 三红为原生Rust UI前端（非Rust API）：outlet_multi_route_loud_reject / test_bare_popover_point_anchor_codegen / test_display_family_codegen_arm_fixture。用 UI eff9ece3 源归档 + main Lang 基线（a786/523之间仅941簿记）重放 rust子集77 PASS/同3 FAIL/2 ignored；三份生成码 SHA 完全相同（0a34bee7、9bc95722、9b13aa98），判既有外围问题，作为auto-ui候选债，不弱化断言。
- 严格真实 auto build -r vue 已成功，vite生产打包8.77s；无lenient，无server覆盖。副本69输入逐文件SHA与产品源一致。
- F04 默认 CLI 冷启动首次红：start_api_server 把Cargo冷构建算入120秒ready，编译中被杀，不能以热缓存重跑当作关闭。F05 同查发现服务cwd为生成工作区，Reader相对引擎资产应按应用根。r4必要运行时修复：复用既有Cargo JSON artifact与发布设施，先构建（既有build/cancel预算）再启动成员自有原生镜像、再开始原120s ready；构建cwd保持workspace，运行cwd改应用根；保留人类compiler diagnostics，启动错误直接传播。UI提交59b725b754c6c35ae3862092a1891e4b61777ed0。
- 新增UI实际Cargo build/publish/错误旧镜像负控1/1 PASS；Reader永久冷启动/相对资产/原生监听/真实Vue页面驱动 vue_rust_default.py（9a347c3）；全新目标根复验待完成。双轨门补段落锚、emoji+CRLF/UTF16锚、ID形状和时钟类型约束，不能靠规范化掩盖无效ID/恒零新书时钟。
当前仍 executing：T04 的冷启动/浏览器与最终46-case对拍尚未完结，不签发review pass。


r4补证：UI最终22a925c7（只删除重复清端口）；启动器60项串行58PASS/1FAIL/1ignored，余一delivery闭包在eff9ece3原样源码+主线44份manifest副本具名重放同断言，五个并发红串行转绿，D-956-1登记。旧基线归档缺兄弟布局的8环境红不作语义证明。
冷门02首次失败是driver未创建显式target根，已修脚本3e69594；03在业务源码变更后主动终止自身进程，不计PASS。expanded-01对拍正向检查发现新书created_at仍由历史业务代码写0；TXT/MD与EPUB导入已改time.now_sec（旧记录时间保持），最终Reader d10225b，所有新鲜夹具created_at>0强断言保留。全新04目标根正在跑，未签最终通过。


### T04最终完成（2026-11-06）
- Cold04实际默认auto run（无server覆盖、全新目标根）：126秒完整Rust冷构建后发布/启动原生app-2348c2db-back.exe，服务ready通过；总564.70秒到Vue代理、引擎资源原字节、中文导入/章节与原生监听身份。完整driver因浏览器测试依赖缺失exit1，不冒报整驱动PASS；cold HTTP/native阶段live.json有效。补齐scope内Playwright/Chromium后，原发布镜像与原生成Vue、原导入数据的browser-replay-03通过真实书架和阅读页，pageerror=[]，截图实查中文正文可见。早期browser-replay-02因脚本未按createWebHashHistory访问路由而超时，已修永久脚本；未改产品路由/正文。永久driver现启动前预检浏览器依赖，可用--browser-module-root指定工作树外安装。
- Cold04 Reader业务快照d10225b；其后仅时间native实参改先绑定局部，避免VM多参错位（D-956-2）。Rust启动机制/前端/SDK输入未变，冷启动及生产Vue证据按触面复用；最终业务代码已从新源码重新正式生成并Rust构建（135秒），不改生成物。最终Reader64ff06503d359de0b730f4cfbddaf5affe3d0bf8。
- parity-final：同一最终Reader src/back，Rust实际新exe与VM解释器各46项HTTP操作，42条路由清单、冷重启、28份持久化字节一致；新鲜created_at>0、UUID形状与时钟类型检查成立。仅规范化具名ID/时钟/数据根/导出basename。保留旧ph1黄金值、独立oracle、状态码/错误和原文件内容，未用VM桥/替代后端。VM-expanded-preflight首红定位嵌套native实参错位；02原46操作及严格时钟通过。
- 工作树链接清理：最初计算路径+Force移除被自动审批策略阻止；未穿透/未递归。改为逐一字面量路径、校验Junction与精确target、无Force无递归的Remove-Item，两个956 deps链接安全移除；vendor两目标均存在，最终Reader guard clean。955旧持有未触碰。五仓guard均clean，清理收尾尚未执行。


### 最终复审 r4（2026-11-06，final / pass）
复审上下文=同一实施会话，未获授权创建独立agent，明确不称独立会话复审；从实际diff、原始结果、行为断言与source manifest重新核对。所有实现已提交，四仓工作树无WIP（Web最后提交bf72baa只收录测试实际已使用的lock内容：sha2/zip与944语义依赖，不改变运行输入）。
基线：Lang58a1dcb9206a9b747cd18225002557a5b8e60fd8/base a7863181fe213a548da4be9a43185d9c0f2f9ad0；Web bf72baa（父0550aadd8be9362c89487ffcc0864842c13b85d5）/base37d32c22d1a4e7744710bf8f02f6c74d5a96c897；UI22a925c72d7620b929c947ff2bbddead13fda783/baseeff9ece3883adb4bd958135ca485a31caf7efdf1；Reader64ff06503d359de0b730f4cfbddaf5affe3d0bf8/base4a2c748e51d754ecd700c97beffcc56522a7f5e9。完整变更文件SHA清单40323029a8808f625c38c38b97dda9cc716ca5f3723da7040d96b7194e191740（scope/frozen-source-manifest.json）。

| AC | 结论与证据 |
|---|---|
| AC01 | PASS：完整API原生build，42route静态核验；默认CLI冷build126s、原生发布镜像监听/应用cwd资产；Vue书架/阅读页正文实际可见。generated Rust业务直接调用，无VM执行桥或scaffold。 |
| AC02 | PASS：静态未知symbol仍拒并打印符号/模块；type_of shadow/arity保持，JSON null/值域/i64-u64/数组object按VM；ZIP共享纯Rust，预算/UTF8/无覆盖失败保持；字节索引迁移已文档化。最终原语与职责门均有永久断言。 |
| AC03 | PASS：最终源实际46case/重启/28文件字节对拍；原错误/状态/UTF16锚/旧ph1独立黄金值不归一；UUID形状、整数正创建时钟先检查。Rust与VM normalized文件各SHA d8418ca331bf16b5364a2169e0cb21499043f4a0dde56e6d51099ef10d98f16d，parity-final.log SHA c127ef5ac76c14b0f6762d707f75b8d73b0323a3dcc90a7d4ffabf49e0d9241d。 |
| AC04 | PASS：永久HTTP/cold/browser入口，实际strict auto build -r vue成功；首次红及补依赖/路由脚本修正完整保留。启动器Cargo失败不复用旧exe负控PASS，浏览器pageerror空。 |
| AC05 | PASS：SD01..05与四仓canonical diff逐项一致，无legacy ledger写入，无生成Rust补丁/lenient/VM替代。复审限度及既有红D-956-1/VM边界D-956-2已具名，不将外围红算绿。 |

复审查验：T01完整11模块缺口不漏/不删inflatex；JSON词法null槽与try嵌套返回、同名参数跨函数、两路径捕获顺序、编译失败旧镜像、cold>120s、相对资产cwd、时钟正整数/双轨原文件均有对照。finally/外层break-continue、不透明Future/native UI引擎非本次新增承诺，Spec明确边界，无未批准功能延期。
门禁复用：frozen-groups-02实际a2r/a2r-std/stdlib/vm/api/http/auto-generation-core完整组并集1655PASS、462原排除，not_run=0/无identity漂移。选择对应实际转译、运行库、VM ABI、API/HTTP和跨模块组装触面；未改AST/ACE内部实现，不额外将实验组混入。本次UI启动/Reader时间绑定修复未改该并集输入，可复用。发现/准备18.35+1.10+17.08s，执行215.56+7.82+2.11s，整命令279.82s；额外core null helper1PASS。结果SHA a946742deb1782f0c73d3ca2b68d5ca28d13593f1be44af322665862e03bf258；core收据4c2cb1fc092c166f0a9da685c30cd3e2c210243602a792542105fb3410f9a417。
Web53PASS/1ignored；UI schema7PASS，compiler1144PASS/3基线同红/2ignored，launcher串行58PASS/1基线同红/1ignored。UI启动器最后删除重复清端口不改变其受测构建/发布/运行机制；cold04实测最终UI22a925c7，关闭F04/F05。编译保留既有warning不隐藏，无新增未处理warning类。源码diff --check四仓全绿。
关键证据hash：cold04 live e193bccff4ad85b66dc37aaf889884f762372fd3c5ba9f1d2e1d1a2359a483dc，69输入manifest27ab3323efd4d8e0e0ba342ef604b59d1a7755ae3222c2c9a4eab5fa758dbb13；browser03 e4802e54073ef898dbc2ab1614da0b63a7b3c1a9a48b92d781061be69367f853，截图d131908d8366d6ac1f36eea58612df12471cfd51662f78f53cf8cfa456c08553；Rust最终build01810ebaaf7ed9ac663b904e8cbe1af02775022edcf750df71408da56aa0e949；productionVue日志d0737712a66b39006aa8fbefa4b53303b0ba3555b554f2e14c5d4d9bed7c810d。
next=merge：允许按身份等价复用上述merge-owned职责并集，仍须实际ff-only、canonical读回、刷新默认工具、归档与guard清理；本段不冒报已合入/已清理。
