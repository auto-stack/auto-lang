# Design 56：ACE M2 完整 FFI 与新版标准库

2026-11-19，W1 按 1071 原子面和 AutoUI 104–108 剩余 AV 分母验齐，正式 W2 由 [1097](../plans/1097-ace-ac-autoui-w2-and-age-host.md) 交付，1074 消费 W2。AGE 领域来源见[AGE 交付表](strategy/age-m2-three-route-strategy.md#后继交付与消费)。1068 接收正式证据并执行最终三端验收。 普通 AC 修复按 [AAC 与现 AC 实施归属](59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属) 进入实际任务。

> 2026-11-16 · proposed r13。完整 M2 范围仍在推进，947有限合同/console已交付，完整原生库未交；补充 [Design55](55-ace-m2-complete-language-and-product-replacement.md)，执行入口为 [M2路线](strategy/ace-m2-delivery-roadmap.md)。r2把945–948的调研/准备/最小接线统一移至M1正式tag之后，门见Design55 §1.1。本轮仅修改设计与 drafting 计划，不改 M1/941/999、canonical Specs 或产品代码。

2026-11-13 r6补充：用户明确Rust库+shim共同构建及目标C ABI独立分发，编译器版本/目标/接口兼容与失效规则落§3.3、967 r10；保全部原FFI/LIB/F0与阶段义务。本轮仅文档，无构建/运行/探针、无接口已实现声明。

## 0. 两阶段归属（2026-11-09，当前）

用户正式把ACE M2拆为A语言/新版标准库完善、B产品替换；依据[Design55 §16](55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)。本设计的完整binding/ABI/stdlib、四端真实调用、平台选择、候选装配与MIX-07能力腿属于A，不能等全App迁移才首次验证。原安装/升级/默认/最终退役后重建腿属于B，保985→986→987职责；B-pending不冒A失败或NA，A-required缺项仍阻A。

A阶段完整性采用旧义务守恒迁移＋按规范补足新case＋真实RG-A验证的并集。每操作/目标provider、数值/文本单位、错误/授权/ownership/close/future取消分别有独立oracle；复用TestCase/ACE worker和固定Cargo组，新增跟踪源与必要adapter由能力owner及985/A负责，不重新造测试框架。旧943的17项、M1 NA及旧bug均不是新版全库的覆盖上限或豁免。

945–948的M1-TAG进入门保持；本轮只修改文档，未执行调研前探或产品验证。A的正式出口由985/T-A01..A05收M2A-01..08及RG-A，B最后消费全产品与最终安装/退役证据；AC自举不在本设计/M2硬门。

## 0.1 首个ACE UI Hello：最小FFI架构先行（2026-11-11，当前）

用户要求按AV/Rust FFI方向规划首个UI Hello，并核对既有ACE M2计划；本轮仍只规划，不构建/运行/生成binding/素材/SDK或建实施树。这里的“架构先行”是先正式交一个可复用、受检的Rust互操作纵向切片，不要求完整Rust/C importer、所有泛型/async/标准库或整个ACE M2先完。

Iced仍是Rust编译的宿主库；Auto代码经共同前门/领域投影与受检foreign事实，由AV stub调用自动生成的C兼容Rust shim。Rust宿主内部正常调用Iced，不能把iced::Element/trait对象/Rust私有布局直接当AV值。UI文本必须在guest借用失效前由session拥有者复制；签名/权限/错误/线程/析构规则明确，不能以UI专用native数字ID或手写manifest绕开通用FFI。

| 原Plan / 早期切片 | 最小交付、前置与边界 |
|---|---|
| [967/F0](../plans/archive/967-ace-m2-host.md) r7，0/19 | 已交938/939与现有text/AV基础→普通use.rs/真实Rust公共元数据→自动binding/shim→共享verify→AV真实非UI Rust函数；首批借用UTF-8/i32-bool/同步同线程/准入资源错误。原完整T-09..15继续扩大C/Rust、资源、四端/真实库，F0不等962/963整包 |
| [979/H0](../plans/archive/979-ace-m2-schema.md) r8，0/12 | 现有grammar/CST/普通函数/text/verify基础→一个AGE node/view/text（Schema驱动解析、位置/上下文名称）与普通函数文本表达式→受检领域描述/CPU消费者；不等完整963/966，保原完整Schema/Node及双后端任务 |
| [969/E0](../plans/archive/969-ace-m2-facade.md) r5，0/12 | F0正式来源→公开显式ACE AV同步执行/授权/结果/关闭接口，非UIconsumer自证；不等Dynamic/async/热更，不让UI另造解释器 |
| [978/B0](../plans/archive/978-ace-m2-build.md) r7，0/14 | 已交942/944/960＋F0/E0正式来源→预编译宿主/FFI闭包、运行构建分账、改Auto不重建不变provider；不等全部新库/CT/完整SDK，非UIconsumer闭合 |
| [980/HELLO](../plans/archive/980-ace-m2-autoui.md) r7，0/35 | F0/H0/E0/B0正式来源→AGE Node独立最小Hello、单text和函数文本、真实AV→FFI→既有Iced宿主、错误/恢复/退出；不等STATE全部async、File/WEB/PARITY，不精简原001 |

每个早期切片新增I/V/R/D四个明确任务（work实现、work验证、独立review、merge阶段fold正式交接），共20任务；原任务/AC/用例及完整M2分母全保留，旧Plan号复用、没有新取号。只交早期能力不勾整项旧任务/整包完成，不归档/删唯一树；下游只消费已审已落主接口，不把冻结分支tip作交付。

AV首窗与A2R分开推进：967原T-12的AXR0简单签名腿、978原T-03/09构建腿、980原B-RUST的同源Hello腿可并行独立复核/phase fold；AV首窗不等它们，完整980/RG-A03及M2-A仍须真实A2R与原模式全覆盖。相同源和资源、签名/ABI/owner事实共用，A2R禁AV/旧VM解释provider仍运行；直接crate adapter须独立证明与AV shim合同等价。

最小Hello只证声明/呈现，函数文本再证实际AV求值与FFI dispatch。按钮→双实例→原001 themes/model/design/布局/选择复制→其它Demo逐步消费新增能力，Demo未覆盖的语法/资源/错态另补。完整FFI不能由简单借用文本、完整UI不能由Hello替代，945–948 M1-TAG及A/B总出口不变。

2026-11-13当前纠正：首Hello直接以AGE `node Hello { view { text "Hello" {} } }`运行，979/H0由版本Schema驱动parser和结构校验；`widget/model/view/on`仅在AGE激活且Schema声明的NodeName槽/父规则内识别，普通表达式/无AGE环境不保留这些词。旧widget仅作同一Schema的显式兼容声明头别名，原源兼容与只改头的Node迁移分别验，不另造传统widget管线。967仍先做非UI函数F0，969/978保持非UI自证、避免反依UI；其接口供同一Node Hello宿主消费，无新增对完整963/966/981的早期等待。完整FFI/标准库/旧UI全部模式、AV/A2R及M2A-08义务不减。详Design55 §7.1和979/980/981当前合同；本段只修订规划。


## 0.1a MIN-A UI-FFI与B三波（2026-11-16当前）

A只交980 MIN-A-UI、981最小真实AV领域场景和OS064最小宿主见证，完整图形在B按AV→AC→A2R走整条爬梯。已交Hello的`text_byte_len`只证字节接收，Rust组件直接绘制不代实际UI FFI效果；A仍要真UI submit/typed事件回AV与借用/owned/代际/权限/主线程/关闭恢复。复用967自动binding/shim、预编译Rust/Iced宿主，guest不解释整套Iced，不跨ABI暴露Rust私有布局。

W1由980 B-AV/981域owner完整跑示例与Widgets，UI-W1-AV正式后1097补AC机器业务/回调并独立交UI-W2-AC，随后1074 W3生成/真实rustc/运行全部同源UI。每波只当前后端实现/真实oracle，AV波不逐计划跑A2R图形；现有A2R与非图形语言/库/FFI基础证据保留，源/ABI变更按实际触面重验。W2不等1068最终三端，W3后OS全量，再985 B→986→987。

详细分母、阶段与收据沿[980](../plans/archive/980-ace-m2-autoui.md)、[981](../plans/archive/981-ace-m2-age.md)、[1068](../plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md)、[985](../plans/985-ace-m2-certification.md)和OS064；不新增Plan或执行器，当前只规划不改Spec事实。原语言/库的AC真实基础、非图形A2R及最终完整三端不后移为可选。

## 0.2 默认Auto系统库、可选FFI与字符串边界（2026-11-12，当前）

用户明确默认标准库由Auto承担类似C标准库的基础职责，而不是包一层libc；Rust/C FFI用于显式外部能力与可选实现。这里收紧原§2“libc由provider决定”的默认选择：默认core/alloc/基础系统库不得把libc作为语义/API实现provider，也不得经Rust std或第三方shim间接绕回libc来实现同一基础操作。不是声称当前已实现无libc默认库，更不以旧a2r-std或有限F0支持充整库完成。

| 层 | 规则与owner |
|---|---|
| Auto公共API与算法 | 默认普通Auto源码实现数学/文本/bytes/集合/parse/format及已批准基础职责，沿943/959分母；不是逐函数复刻完整C/POSIX API或继承C字符串/errno模型。947定分层，972实现；Storage/allocator具体API仍按原M2讨论冻结，不借本条预定签名。 |
| Auto内部调用 | 使用版本化AutoABI、Auto值/ownership/错误合同；AV、AC、新A2R/A2C保同一逻辑语义。实现语言或C ABI本身不决定某函数是不是libc。 |
| OS边界 | 973提供薄的目标平台适配，遵守OS实际公开入口/系统调用的调用约定、布局、编码与资源规则；AutoABI不替代内核/OS ABI。不要求所有平台发同一裸syscall指令，也不以通过C调用约定访问系统API为“依赖C标准库”。 |
| 显式Rust/C外部能力 | 967自动import/binding/shim、资源/回调/future/双向调用继续扩面；SQLite、第三方解析/图形等按模块/provider manifest显式声明依赖。也可显式选libc兼容provider，但它不冒默认自有实现，不被缺provider时自动回落选中。 |
| 工具/宿主/产物依赖 | 978分别列出构建工具、AV/UI宿主、标准库provider、目标运行时与第三方库的实际依赖。Rust宿主或工具链含C运行支持，不等于Auto标准库基础操作可委托libc；也不能据此宣称整个宿主/最终二进制完全无libc。默认库调用链与实际目标导入/链接闭包须单独实证。 |

**字符串**：Auto Str/str/bytes保显式长度和相应编码/ownership语义，不采用NUL（`\0`）结尾作为通用长度协议；逻辑表示不冻结某个Rust String布局或未批准的AutoABI字段。空串、中文/多字节、含`ab\0cd`的五字节载荷须完整保真。C ABI并不意味着所有buffer都NUL终止：ptr+len接口按长度传递；只有外部API明确要求CStr时才在边界建立终止符、编码转换与临时owner。内嵌NUL不适用于CStr时明确拒绝或按该API已声明的有损策略处理，不静默截断；C端返回字符串按已证长度/容量或有上界扫描读取，不能无界strlen。Rust字符串经明确shim/handle/借用缓冲适配，不暴露私有布局；所有复制/借用/销毁及allocator跨界责任由967/972/973验。

新义务纳入M2-A能力/候选验收，B继续安装/升级/退役后重建验证。原FFI-M2-01..08不因默认自有库而缩减；F0仍只交借用UTF-8同步早期切片，不能拿F0代完整CStr、owned、泛型或async；相关能力按原完整任务扩面，首个UI Hello不被完整C importer反向阻塞。

| 新义务 | 完成条件 | 原Plan/任务及新增AC |
|---|---|---|
| LIB-M2-07 默认自有实现 | 每个基础操作有自有Auto/原语/OS适配来源与默认选择；无隐式libc或Rust std→libc委托；可选外库显式分账，缺默认provider具名拒绝 | 947 T-09/10、948 T-09、972 T-09/10、973 T-09/10、978 T-09/10、985 A/T-09、986/987 T-09；各AC-NATIVE |
| LIB-M2-08 字符串/ABI边界 | Auto长度/编码/owner语义独立于CStr；C/Rust入出参、内嵌NUL、借用/复制/释放、平台编码与错误有四端独立oracle | 967 T-10..13/15、972 T-09/10、973 T-09/10、985 A/T-09；各AC-NATIVE |

验收分两腿：一腿移除所有可选Rust/C/libc provider，用默认库在真实OS执行文本/格式化/分配/文件等原已批准功能，核源码/符号/链接/运行调用与错误/清理；另一腿显式导入C/Rust测试库，分别验证ptr+len与CStr/Rust适配。默认腿不得偷用外部桥过关，外部腿也不得因“默认不依赖libc”被删。注入错误默认选择/隐式libc委托、错ABI、NUL截断、缺长度/销毁、缺工具/缺provider的控制必须红，合法空串/多字节/含NUL长度载荷及正确原生provider近邻须绿。四端相等只是补充，不替独立expected。

本条只补设计与计划；未新增或生成binding/fixture/runtime/SDK，不运行探针/系统调用/产品，不改canonical Specs、不改M1或M1-TAG，不冻结新的allocator接口。

## 0.3 Windows/Linux原生I/O具体方案（2026-11-13当前）

[原生平台I/O Draft 0.1](stdlib/native-platform-io.md)实读Zig0.15.1/Go go1.25.0一手源码后确定：Auto公共算法→受检平台原语→Linux直接内核ABI / Windows公开Win32 API。Zig Windows部分走NtCreateFile，不据此声称两语言全部相同；Auto首版保公开Win32，必要Nt能力另具名论证。明确AV/AC buffer/授权/资源、Unicode控制台与重定向、部分完成、Path/关闭/持久化和三层无libc声明。

已归档943/944/947不重开；新增1039受检平台I/O/内存区域原语、1040 Linux x64 ELF/入口/syscall/匿名内存、1041跨平台两流与Unicode补充；当前1039/1040为r2、各drafting 0/9，1041仍0/8；973继续完整File/系统服务及原A2R/A2C义务。948/972/973/978/985同步具体AC/V/SD；早期Hello/B0/F0前置不变。当前AC发射冻结COFF/PE，Linux目标选择不等ELF运行；新四格Windows/Linux × AV/AC由各owner按正式来源收证。宿主/工具/runtime与默认IO调用链分账，不宣称整个Rust AV已无libc或自举。本轮仅设计/计划，无实现/运行/canonical Spec更新。

## 0.4 自有原生库的强制底座依赖（2026-11-13当前）

用户指出平台机制是Auto自有标准库基础，不能只写为可选并行线。现修正[平台设计Draft 0.2](stdlib/native-platform-io.md)§9.1/10.1：1039/1040补OS页/区域来源（Windows VirtualAlloc/VirtualFree，Linux直接mmap/munmap），不依赖malloc/Rust全局allocator提供guest默认区域；Storage算法与公共API仍归946有限切片/972完整库及内存专题。

946整包正式前置补1039/1040；972补1039/1040/1041，原生实施/验收必须消费真实实现/OS运行/独立复审/落主身份，不能拿接口、provider声明、旧Rust no_std runtime绿代替。纯core算法可提前写测，原生owned/alloc接线与整包完成必须等底座；不为无需OS的每个函数新增运行期syscall。1041只等945最小文本，不反依972；1039/1040不反等946/972，防止分配器自循环。973原已有三包前置，974/975及完整978/985沿现图传递依赖，不重复所有边；985新增FOUNDATION验收核实际来源。

修正后的原生库主链为1039→1040→946/1041→948交接与972→973→974/975→完整SDK/认证。948前期调查、语言完善、967显式FFI及早期F0/H0/E0/B0仍按各自最小前置推进；其通过不冒默认原生库完成。943/944/945/947等归档不重开，旧交付按实际范围引用。本轮仅修订设计/计划，未实施代码/探针或修改canonical Specs。

## 0.5 标准库最低后端与 AV 三来源（2026-11-13当前）

用户明确：所有新增公共标准库能力至少支持AV/AC/A2R/A2PY，AV还需native、C、Rust三种实现来源。当前覆盖设计为[单AV/多provider](stdlib/backend-provider-coverage.md)。此前“四端”AV/AC/A2R/A2C仍保留，新增A2PY，不把旧A2C删掉。

最少六条主路径为AV/native、AV/C、AV/Rust、AC、A2R、A2PY，另保原A2C。不是所有后端各建三版本；一个AV共用语法/字节码/类型/错误/授权/资源规则，装配时选原语provider。普通Auto算法单源，外部调用/资源边界有各自adapter，Rust经稳定C兼容shim；资源后续操作和析构始终由原provider承担，不静默fallback或跨来源释放。默认native不隐式用C/Rust标准库，显式外库和Python/Rust目标运行时依赖另列。

1042新建最小Python producer/运行端口前置，977/PY消费后继续完整语言，不让库反等其GD/SCENE/REVERSE整包。946、972–975、1041逐公共操作关闭六路径/原A2C；948管分母，967管AV来源和A2PY受检FFI适配，978管装配/依赖，985核A完整性。1039/1040是内部平台底座，保AV/native+AC原生格，但不以基础设施完成签新增公共能力已全路径支持。F0/H0/E0/B0/Hello早期切片范围与前置不扩。

本节是当前新增能力覆盖合同，优先于历史“四端”作为完整分母的表述。逐操作有独立expected与真实执行/资源证据；纯算法可按相同输入/身份/观察复用，不复制三套算法或盲跑全笛卡尔积。缺A2PY或任一AV来源的公共新能力不能标完成；旧归档计划不重开，由新计划/后继补差。本次设计/计划修订不是支持声明。

## 0.6 新加载机制、#in与首个实施包（2026-11-13当前）

[加载设计§8–10](stdlib/backend-provider-coverage.md)已按用户决定更新：公共io.at＋io.at.at/io.c.at/io.rs.at/io.py.at四类实现来源，provider后缀与执行后端正交；平台不扩后缀，使用`#in sys is current { Windows -> use ... }`。#in打开局部namespace并使内部is/if/for继承编译期阶段，不重复#is。旧in namespace“仅候选”的建议由此取代，裸in运行时扩展仍不在本轮。

新增[1043](../plans/archive/1043-ace-stdlib-context-assembly.md)先交最小source装配/#in/条件导入与增量恢复，复用936/958/944/947/960，Python最终验证消费1042；1043不依完整966/967/库。966 r8消费本包继续完整CTE/宏/metadata与原目标条件职责，967及新增公共库能力最终消费加载合同，不重写loader；944/947已归档，不重开。1043的纯选择fixture不冒真实C/Rust provider dispatch，也不冒OS I/O完成。

本套新加载机制下一实施计划为1043；1039 OS/内存原语可以并行、1040继其补Linux，1042同步交Python最小producer，完整输出/集合/系统库最后汇合这些底座与967的实际FFI。原946等M1-TAG、早期F0/B0/Hello、A/B正式门不减，不以新增计划绕过正式来源核验。当前仅设计/计划变更，无代码、worktree、产品运行或支持完成声明。

## 0.7 Linux延后验证、Windows DLL与普通库归属（2026-11-15当前）

本节按用户最新裁定优先于前文/历史快照中Linux必须当期实跑、WSL渠道及待执行底座状态：1039–1043、946/948、967现已归档。当前主序972→973→974/975，973另需968；本轮不实施代码或测试。

Linux分支继续写代码、保完整用例/oracle，但当前不执行Linux测试；建设包以Windows required门＋Linux实现/移交审查完成。Linux格保deferred-required/NOT_RUN交[1063](../plans/1063-ace-stdlib-linux-deferred-validation.md)，环境完善后承[1061](../plans/1061-ace-linux-suite-isolation.md)原生Linux VM显式入口实跑。1063不反向卡能力包/Windows认证，未验Linux不升verified、不删分母；1040/1041已验历史保留，不重开。

Windows系统DLL既可PE导入由loader启动时解析，也可LoadLibraryExW/GetProcAddress按需解析并缓存；AC固定OS入口优先PE导入，AV复用宿主薄入口/既有受检loader。系统DLL/C ABI不是C标准库，详[平台设计§4](stdlib/native-platform-io.md)。

math/io/net及文本、集合、JSON/编码等普通标准能力默认必须由Auto实现算法/协议，再接OS薄原语；不能拿libm、C stdio/Rust std::io、reqwest/hyper/libcurl代默认自有实现。高级数据库/TLS等引擎可显式FFI直到Auto替换，依赖单列。AV可选C/Rust来源与各转译运行时adapter仍保留，六主路径/原A2C的当前Windows验证不减；网络/数据责任分别落974/975，Web产品宿主仍归983。

## 0.8 最新阶段与三端一致性

### M2-A基础可用、M2-B三端对齐与AAC实证（2026-11-16当前优先）

用户最新裁定：**M2最终AV/AC/A2R公共能力一致；A2C完整对齐为增强项。** M2-A仍须完整AV/A2R语言/库及原required真实验证，AC至少有能承载AAC启动的真实多模块/类型与owned数据、泛型/Option/Result基本链、所需CTE/FFI、文本/分配/同步文件等基础。AC其余具名差额允许移M2-B：原行ID/输入/oracle/错误及资源义务、实际来源、B-owner/关闭条件必须保留，不能记PASS/NA或从分母删除。仅“原native不支持/旧profile外”不再是M2最终豁免。

[1067](../plans/archive/1067-ace-ac-native-async-and-task-parity.md)补原生异步/Future/Task；[1068](../plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md)收全部三端语言/库/FFI/领域差额，并以AAC实际模块持续实证。985/A不等待两包，985/B及最终986/987必须收三端完整闭合；A2C现有代码/用例/支持记录保留、缺完整对齐不阻M2。A2PY既有标准库义务与AV三provider、Linux1063延期规则不变。

AAC在A出口后、M2-B期间推进，以真实Auto编译器模块检验AC语言/库；某任务所需能力未交只阻该任务/切片，缺口回语言/库真实owner补齐。AAC未使用的语言/领域/异步特性仍补独立三端用例。完整Gen1→Gen2→Gen3自举签章保持独立，不强加为M2通过前置；其所暴露的M2三端能力缺口必须在M2最终前关闭。此规则优先于旧“AC所有义务必须A期全齐”“M2-B与AAC完全无能力交接”“原A2C全量同等硬门”等表述。

## 1. 决定与现状

M2必须交付自动生成 binding 与 ABI adapter 的 C/Rust FFI，以及适配 ACE 的新版标准库。用户从 `use.c ...`、`use.rs ...` 公共入口使用库；不能只提供手填 Foreign IR、手工 extern 声明或一组受限演示。新版标准库保持一套公共 Auto API，优先用普通 `.at` 实现可移植算法，再通过平台实现、宿主服务和 FFI 提供外部能力。M2终验增加下文 FFI-M2/LIB-M2 义务，原 REP/V06、语言与产品分母不减少。

M1已交的是 Windows x64 有限 AutoABI/Foreign IR、手写 typed manifest、资源/同步 callback 与部分目标发射，不是完整 importer。旧引擎已有部分 `use.c`/`use.rs` 或 bindgen，并不证明新 ACE 公共入口接通；A2R生成 Rust 源也不证明 Rust库 FFI。M1残余正确性继续原 owner，本设计不把新范围塞进999。

输入依据：当前 Specs 优先；[Design41](41-interface-metadata-and-foreign-abi.md)给出 provider/ABI 分离和 Rust shim 方向；[Design54](54-stdlib-semantics-and-ace-integration.md)给出统一语义/装配合同。本设计扩展其早期有限范围，不把这些 proposed 文档当现有能力证书。保留 Design46 完整 CAP 原进入门，显式平台选择与装配无需等待完整 CAP 求解。

已使用 read_thread 读取用户引用的标准库讨论（local / `01a1abb2-ae3b-7033-894c-11bff0ff948c`），包括 2026-11-09 已完成的 `#if`/`#is` 平台选择答复；该讨论仅为设计输入，原文中的完成/支持主张仍须对当前源码和规范核定。本轮不编辑其 Design41/54，不向该聊天发消息。

## 2. 一条编译链与两类 ABI

```mermaid
flowchart TD
    A[Auto 公共源码 / 标准库 / use.c / use.rs] --> B[模块与目标条件选择]
    B --> C[共享绑定 / Checked HIR / Foreign IR]
    D[C 头文件及预处理 / Rust 公共元数据及具体实例] --> E[自动 binding / shim / ABI adapter]
    E --> C
    C --> F[共享 MIR → AV / AC 原生机器码]
    C --> G[共享 A2X → 新 A2R / 新 A2C]
    F --> H[Auto runtime / 宿主端口 / 平台 ABI]
    G --> H
```

Auto模块之间使用受检 AutoABI（内部可内联或优化）；系统接口与 C-compatible shim 使用明确的 C/system/目标平台 ABI。AutoABI 不代替操作系统 ABI，也不承诺直接调用任意 Rust 原生 ABI。AC直接产生机器码调用适用平台 API，无需把业务 Auto 先转译成 C 或 Rust。系统入口遵目标OS ABI；默认基础标准库的自有实现/非libc provider与显式C/Rust外库分账，选择及字符串规则以§0.2为准。不把“原生”解释为所有平台都发裸syscall，也不允许以provider可选为由默认委托libc。

四个独立维度进入装配：执行后端 AV/AC/A2R/A2C、宿主 CLI/browser/UI/service、语言/能力 profile、目标 OS/arch/env/ABI。provider 的实现语言是另一属性；不能从 `.rs.at` 推断“ACE后端”，也不新增 `.ace.at` 或第二套 `ace.std` 公共 API。

## 3. 完整 FFI 合同

| 义务 ID | 必须交付的能力 | 明确边界 | 实施与验收映射 |
|---|---|---|---|
| FFI-M2-01 | 公共 `use.c` / `use.rs` 从工程输入到共享绑定、类型诊断和实际执行；自动产生可追溯 binding | 可显式配置库/工具链和所有权注解，用户不逐函数手抄签名/布局或手写 extern wrapper | 967 T-09 / AC-07 / V-09 |
| FFI-M2-02 | C importer：真实预处理、include/宏/typedef、目标布局、指针/record/enum/callback/opaque handle、常量和 inline 所需 shim | 逐行继承现用 C 义务；varargs/union/bitfield 等具名支持或保留缺口，不能偷偷删分母；不自动扩大为任意 C++ | 967 T-10 / AC-08 / V-10 |
| FFI-M2-03 | Rust 公共 API 元数据、具体泛型实例到自动 C-compatible shim/handle adapter | 不依赖私有 Rust ABI；显式 String/Vec/Result/borrow/析构、panic、Send/Sync与 future 约束；无法证明的接口具名诊断 | 967 T-11 / AC-09 / V-11 |
| FFI-M2-04 | AV、AC、新A2R、新A2C消费同一签名/资源/效果事实并实际调用 | A2R可用经证明等价的直接 crate adapter；不能以生成 Rust 或 emit-only 算通过；不回落旧VM/AST emitter | 967 T-12 / AC-10 / V-12 |
| FFI-M2-05 | 双向 import/export、适用静态/动态链接、allocator 与销毁、callback 重入、版本/签名检查 | generation/lease保活，在途callback/析构/future禁止卸库；跨ABI不放任panic/unwind，错误签名在调用前拒绝 | 967 T-13 / AC-11 / V-13；原T-08/AC-06仍保留 |
| FFI-M2-06 | 异步注册/完成/取消端口可承接Rust future及实际现用桥 | 967只交端口/adapter与受控provider；语言await/调度归968，不能让967反向等待968 | 967 T-13 / AC-11 / V-13；968原T-02..04/AC-02..03消费 |
| FFI-M2-07 | 可分发 binding/shim/provider 闭包、精确缓存与清晰工具依赖 | 缓存无进程地址/授权；导入编译工具与安装后运行依赖分别列出 | 967 T-14 / AC-12 / V-14；978 T-09 / AC-07 / V-09 |
| FFI-M2-08 | 用户级真实库工程与独立ABI oracle，失败/取消/升级均闭合 | 自有fixture不能替代真实库；缺环境仍是缺证，不改PASS/NA | 967 T-15 / AC-13 / V-15；985–987 T-09 / AC-07 / V-09 |

### 3.1 导入输入与安全边界

C导入身份至少包含头文件的实际传递include闭包、预处理选项/宏、include搜索序、target/sysroot、解析器与ABI工具链版本、库链接身份。不得靠字符串正则抽签名、按宿主布局猜目标，或把生成文件存在当binding正确。

Rust导入身份至少包含crate版本/lock、features/cfg、target/toolchain、导出元数据版本、具体类型/泛型实例、shim源码/生成器和链接身份。编译辅助工具缺失应报告构建缺依赖；无元数据不猜公开布局。原生态的宏/trait/泛型等通过真实公共元数据与具体实例承接，不能宣称任意 Rust 源都可直接变为稳定 ABI。

自动binding不等于能自动猜全部所有权。来源注解、API约定与校验必须足够；不确定的raw指针/生命周期保留显式低层边界，不能包装成虚假的安全标准库函数。受检标准库facade必须关闭其所有权、错误、线程、取消义务。

T-09冻结可运行源码、依赖/工具链和required特征表，原有C/Rust/Python消费者保持；本次“完整”是基础设施和已批准支持集完整闭合，不承诺世界上所有第三方API零配置兼容。冻结不能用缩成scalar demo代替完整义务；新增语义争议需正式修订。

### 3.2 独立验证路径

C用独立C编译器构建caller/callee核布局、调用约定、返回值、写回、释放及错误签名；Rust用独立Rust测试核同一公开API，再核自动shim和四端结果。至少覆盖一个标准C头输入、真实SQLite header/library工程和一个使用Rust `serde_json` 公开API的工程，及具有callback/owned/泛型实例/future的专门fixture。具体版本由T-09钉住并记录许可证/分发依赖，sqlite和serde_json是最低真实路径，不作为全旧用法的上限。

分别断言值、效果、资源销毁次数、越权dispatch=0和拒绝阶段；四端彼此相等不作唯一oracle。删除手工manifest、开发树与旧桥后仍能从原工程生成并执行，才算自动体验闭合。

### 3.3 Rust provider的二进制兼容与独立分发

架构单源为[Design41 §5.1](41-interface-metadata-and-foreign-abi.md#51-rust-provider的构建与兼容合同2026-11-13目标态)，执行合同为[PLAN-967 r10 ABI-COMPAT](../plans/archive/967-ace-m2-host.md)。这里把“C兼容shim”明确为以下M2验收义务，不修改原四端/真实库/资源分母。

- 源码模式：Rust库＋生成shim在兼容工具链/依赖闭包内共同构建，真实rustc/target/crate lock/features/cfg/flags和生成器进入BuildIdentity。自动适配不使任意旧rlib/Rust ABI动态库可跨rustc直接链接；缺源码/工具/版本不满足须具名失败。
- 分发模式：默认用封装Rust私有依赖的cdylib及其接口/工件/运行依赖清单，通过目标C ABI供AV/AC/适用A2X消费；已有C API无需强制再包。staticlib是原生链接分发，更新可能要重新链接，不能和独立动态替换混称。A2R等价直接crate adapter仍属同Rust构建闭包，不带跨rustc二进制稳定承诺。
- 运行兼容：consumer与provider可用不同rustc，但目标ABI/OS-arch-env/位宽/端序/必要ISA、ABI域/descriptor版本、required符号/布局/调用、接口错误/owner/线程及运行依赖必须兼容。C ABI不等于跨平台、不含自动CPython/CRT兼容，也不自动批准旧handle交新代释放。
- 失效分账：源码构建换rustc或真实输入→失效受影响BuildIdentity并重建库+shim；consumer工具链改变而已分发provider字节/接口/运行条件未变→不因rustc标签不同强制重建provider。目标或破坏性接口变化→重建适配/证据并在不兼容时业务dispatch前拒绝；显式规则允许的可选符号增量可按required子集重新准入。工件精确pin不因ABI兼容自动解除，新provider证据不得复用旧代码身份。

967按原T-09/11/12/13/14/15及T-04交付B-01..06、AC-ABI-COMPAT和七个V-ABI-*；T-14向978输出双身份/版本化清单，978复用现有loader/store处理装配和分发，986核实际隔离安装/升级。967可先交独立消费夹具，不反等978/986整包，也不拿夹具代最终安装。

独立oracle必须覆盖：两个实际兼容toolchain构建同合同provider及跨rustc消费；错target/布局/owner/接口/依赖拒绝；真实构建输入变化失效而无关变化不重建；ABI兼容新旧provider仍保旧owner/pin/释放；移除开发源码与rustc/Cargo可访问性后的隔离运行。缺第二工具链、缺目标runner或运行依赖均具名pending，不能以文档/emit/同rustc示例关闭；F0仅原最小Rust AV切片，不被本节扩大。

## 4. 新版标准库合同

| 义务 ID | 必须交付的能力 | 实施与验收映射 |
|---|---|---|
| LIB-M2-01 | 一套公共API与稳定操作ID；core/alloc/host/domain分层；默认可移植Auto实现，平台provider可替换 | 947 T-09 / AC-08 / V-09；972 T-09 / AC-07 / V-09 |
| LIB-M2-02 | 组织/装配/加载清楚；公共声明、Auto算法、宿主端口、平台实现、foreign声明、生成产物有明确owner及版本 | 947 T-10 / AC-09 / V-10；978 T-09 / AC-07 / V-09 |
| LIB-M2-03 | `#if`/`#is`选择目标平台代码与导入，统一模块作用域，目标条件进入依赖图 | 947 T-09冻结合同；966 T-09..10 / AC-07..08 / V-09..10；973 T-09 / AC-07 / V-09 |
| LIB-M2-04 | 完整基础/集合/文本等公共算法四端语义一致，core无隐式heap/宿主；可选alloc层单列 | 972 T-09..10 / AC-07..08 / V-09..10；945/946原有限义务保持 |
| LIB-M2-05 | 文件/系统/宿主库用统一语义与平台ABI，真实读写、错误/EOF/close/取消/授权四端闭合 | 973 T-09..10 / AC-07..08 / V-09..10；网络/数据沿974/975既有完整责任 |
| LIB-M2-06 | 兼容分母、安装/增量/恢复/最终退役门均包含新库，旧runtime复用须有合同证明 | 948 T-09 / AC-08 / V-09；978 T-09..10 / AC-07..08 / V-09..10；985–987 T-09 / AC-07 / V-09 |

### 4.1 组织与加载

优先保留公共 `auto.*` 源路径和现有943操作实册/944manifest事实；初始组织如下，物理目录迁移必须有原路径兼容和构建验证，不要求本轮创建目录。

| 层/材料 | 逻辑职责 | 执行owner | 加载与边界 |
|---|---|---|---|
| 公共声明与Auto默认实现 | 稳定操作ID、签名/错误/效果、可移植算法 | 947合同、972基础、973–975各领域 | 普通`.at`参与统一模块绑定；不复制四套算法 |
| core / alloc | core不隐式分配/调用宿主；alloc显式要求allocator | 972 | 单独profile构建/执行；no_std不自动等于无heap |
| host端口 | IO/task/service的逻辑操作和运行期授权 | 967/968、973–975 | 已编译装配不携带旧授权，恢复重新授权 |
| `auto.platform.*` 实现 | target平台API、C/system ABI、必要适配 | 973（领域provider各owner） | `#if`/`#is`选中实现；缺或歧义provider具名失败 |
| foreign声明与生成binding/shim | C/Rust公开输入到可复现适配产物 | 967 | 生成物不是手工公共API第二真相；具名来源和hash |
| 源码/受检IR/ABC/native/A2X片段包 | 开发与发行各需的stdlib/runtime/adapter闭包 | 978、986/987 | 构建加载与运行加载分账；不每次编辑重跑Cargo |

947 T-10把此逻辑分层落到当前 `stdlib/auto`、`stdlib_assembly` 模型/manifest/loader、source-query package store的具体文件/接口，给出版本和迁移规则。禁止另外造一套与944/960/942不相认的loader、provider registry或全局缓存。旧 `.vm.at/.rs.at/.c.at` 按语义/实现兼容映射，不把后缀变成新ACE公共组织原则。

### 4.2 平台条件语义

语法沿现行 [hash-syntax](../specs/auto-lang/comptime/design/hash-syntax.md)，例如（拟议ACE M2用法，非当前可运行声明）：

```auto
#is sys.os {
    OS.Windows => { use auto.platform.windows.file }
    OS.Linux => { use auto.platform.linux.file }
}
```

`#if`同样可用于选择函数、类型、常量与导入。`#`上下文的`sys.os`取**编译目标**，构建宿主信息另命名；OS枚举与精确公共符号在947/966核现行规范后冻结。分支先做必要语法检查，再选择，未选分支不绑定/类型检查/链接平台依赖。选中导入进入所在模块，不困在临时块；有序初始化/冲突/循环按统一模块规则处理。缺平台或多个冲突实现具名拒绝，不把所有非Windows默认为Linux。

947只交合同/装配基础，不等待966完整CTE；966实施target条件最小能力并继续本包完整编译期职责；973消费正式交付。不能让947→966→972→948→947互等。Windows/Linux至少各有条件选择正反例与对应平台provider适用表；真实运行按正式支持矩阵取证。没有平台runner可以先做目标选择/代码检查，但不得声称该平台运行已验证。

### 4.3 ABI、语义与可分发性

字节/Unicode标量长度、parse失败、整数位宽/溢出、EOF/部分读写和资源释放沿943/现行Spec独立冻结；不能以性能或后端库默认行为改公共语义。纯算法默认Auto，intrinsic/外部优化实现须单独行为证明。

AC原生库经AutoABI复用Auto模块、经平台ABI调用OS；不隐式套旧VM或强制先发C/Rust业务源码。标准库runtime可以包含必要的预编译非Auto代码，须明确owner/ABI/allocator；“默认Auto”不等于否认runtime职责。

978缓存键覆盖实际读取的源/传递headers、crate lock/features/cfg、目标、工具链、provider/ABI/runtime/manifest版本、布局/效果与生成器身份。测试必须同时证明相关变化失效、无关变化不重编、失败保旧/恢复修复。编译工具依赖和运行依赖分别给清单：已生成native程序不得隐式依赖开发仓、旧VM、Python、rustc/clang才启动；显式声明的动态库依赖正常保留，首次导入生成所需工具可由SDK分发/声明。

### 4.4 Storage 与分配资源专题（2026-11-09，仅方向记录）

[Storage 与内存分配资源](memory/storage-and-allocation.md)作为Design04的分配专题，记录用户认可的方向：保留Storage用户模型、prelude默认政策与具体资源实例，按需抽取底层分配服务，不预先冻结独立Allocator语言机制或全部API显式allocator参数。本文的core/alloc职责与来源/释放要求保留；具体Storage/allocator接口在M2标准库阶段继续讨论后，由对应owner纳入正式实施合同。本次仅建立设计关联，不新增任务/验收门、不修改972等现有Plan或启动施工。

## 5. 真实源码矩阵与职责

下列为2026-11-09已读取的现存入口；新节点/fixture路径均由各T-01在相同owner下确认后创建，不声称现成支持。

| 输入→producer/事实 | 现存源码锚点 | 后继与拟改面 | 验证入口（固定组；未来按实际改面取并集） |
|---|---|---|---|
| C/Rust导入/提取与manifest | `crates/auto-bindgen/src/extractor.rs`、`type_map.rs`、`manifest.rs`；`docs/specs/auto-bindgen/project.md` | 967 T-09..11；核旧实现可复用层，接新ACE共享绑定 | `cargo t auto-bindgen ace-source ace-foreign-contract` |
| Checked foreign签名/资源/各lane | `experimental/ace/crates/foreign-contract/src/{typed,manual,resource,lane}.rs`；`experimental/ace/crates/abi` | 967 T-12..15；公共IR与四端adapter、ABI oracle | `cargo t ace-foreign-contract ace-abi ace-av ace-native ace-a2x ace-a2c-abi` |
| 模块包/持久store | `experimental/ace/crates/source-query/src/modules/{session,packages,packages_store,persistence}.rs` | 947/966/967合同与接入、978包/依赖/缓存闭包 | `cargo t ace-source ace-semantics ace-artifacts` |
| 公共标准库装配 | `crates/auto-lang-core/src/stdlib_assembly/{model,manifest,contracts,loader}.rs`；`stdlib/auto` | 947组织与装配；972/973消费者；禁止第二loader | `cargo t stdlib ace-stdlib ace-console` |
| `#`源码与目标条件 | `experimental/ace/crates/core/src/source/ast.rs`；`docs/specs/auto-cte/scalar-query-profile.md`；现行hash-syntax | 966 T-09..10，从有限CTE到模块条件导入；不调用旧VM执行# | `cargo t ace-syntax ace-source ace-semantics ace-comptime ace-a2x ace-av ace-native` |
| host/平台IO与旧provider语义调查 | `experimental/ace/crates/av-core/src/host.rs`；`crates/a2r-std/src/{fs,process,time}.rs` | 973平台provider与四端；968仅调度owner | `cargo t ace-host ace-stdlib ace-av ace-native ace-a2x stdlib` |
| 候选包与终验 | `scripts/test-groups.toml`；`crates/auto-test-case`；`docs/specs/language-sdk` | 978→985→986→987，沿真实SDK/产品入口 | 对应计划完整职责组与仓外消费者；不能以仓内fixture代安装 |

测试组从本仓固定表核名，未执行产品测试。外仓改面须读当地AGENTS并由原owner实施；本轮不在外仓写入或创建会话。

## 6. 阶段顺序、交接与M2签章

1. **M1正式tag后，组织/合同与分母**：先核[Design55 §1.1 M1-TAG](55-ace-m2-complete-language-and-product-replacement.md#11-m1-tag标准库准备计划的统一进入门)及对应实施授权；947保原console/资源前探并交分层、装配schema；948只在tag后才能按D943先盘点增补义务，最终仍等945/946/947。945/946在tag后消费947合同进入基础实现。943/944/959/960/961以正式归档身份消费，不重开历史计划。tag前不执行这些包的研究任务、probe或实现。
2. **可并行基础**：967在960/962/963与938/939齐后，按公共导入→C→Rust→含A2PY的各后端/AV来源与资源→缓存/真实库阶段推进；972在其原语言/基础库与本节967/1042适用依赖齐后实现默认Auto核心；966完成目标条件语义。不把972整库或968完整调度反向加为967前置。
3. **系统与构建集成**：973消费967/968/972/947/966，补四端系统服务与平台provider；974/975保完整领域责任；978消费正式接口，完成可分发新库/binding/shim与工程增量。其原依赖不减少。
4. **逐层验收**：985候选预认证收齐FFI/LIB义务及原MIX/REP/V06与新增MIX-07组合（目标选择→File→Rust解析→C数据库）；986隔离安装、升级/失败回滚/默认入口；987退役后重建发行物并重核受影响证据，才签M2。不得新增985↔986循环。

每项交接必须含源/工具链/目标/producer/工件身份、公共调用源码、独立expected/资源trace、真实命令/子终态、环境排除和未闭项。binding签名proof不代行为proof；已编译不代执行；源码开发树不代安装；条件排除不充PASS。

新增义务作为用户2026-11-09批准的**M2分母增补**，948建立逐操作/特征子行，985核与原959分母并集，不修改archived959，不借新ID掩盖旧缺口。任何required子行未闭均阻相应能力包及最终签章；缺环境具名pending，不能以最终“总数相等”代替逐行关闭。

## 7. 本轮变更与验证边界

本轮修订现有 drafting 947/948/966/967/972/973/978/985/986/987及Design55/路线/索引，不分配新Plan编号，不修改941/999或另一讨论拥有的Design41/54；保留全部原任务、验收与历史记录。各新增work任务须先于原最终work验证和独立review，所有计划进度仍为0。

实施时如某批超出可独立评审规模，须先把具体T/AC/V正式迁移到后继Plan并重核DAG，不能只写“未来完善”就通过母包。原计划的一般full-scope责任和本设计显式义务共同生效。

本设计是未来执行合同。本文中的源码锚点、链接、任务/验收映射和依赖图需做静态预检；不以文档预检替代编译、功能、性能、安装或独立实现复审。最终预检收据见M2路线本次修订节。


2026-11-13 本轮仅规划的静态核验：4份计划原95任务ID/勾选/数量/状态/进度与依赖元数据保持；新增3项AC、7项拟验证及3项拟Spec增量，全部未执行。9份文档本地引用、0–10章节、表格/围栏与新AC/V任务映射核对通过；F0→E0→B0与H0汇合HELLO、随后981按接口扩展的切片顺序不新增反向整包等待。963及其实施树未由本轮修改；未生成fixture/binding/SDK、未运行编译/产品/GPU/性能、未建树/提交/合入/tag。此为规划预检，不是实现独立review PASS。
