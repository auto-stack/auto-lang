# PLAN-809 R10 最终规范增量（review批准的merge输入）

reviewed_commit=390d02acc40b2141293730be7eee2b2f95113ec1；plan_revision=1。
本文件只冻结拟发布正文/局部增量，未修改工作树候选Specs、主检出canonical Specs或ledger。
R10校准是已证实事实的文字修正：规划函数已为ordered_build_steps；sidecar声明已在本合同内；旧trait调用须显式导入。并非新增需求/语义修订。
merge应从当前canonical文档组合以下增量，保留其他已合入计划知识；不得拿旧工作树整份project.md覆盖当前主检出。当前canonical及收到的候选版本SHA256见review-r10-baseline.json.txt；收到的原文另存review-r10-spec-delta-source.md。

## SD-01 add docs/specs/auto-man-core/project.md

before=无该规则核心卡；after=管理/规划单源、效果边界、具体实例身份及兼容入口；rationale=核心抽取且不虚报整CLI独立；AC01/03/04。
发布正文：

# auto-man-core — 管理/构建规划规则核心（PLAN-809 SD-01，as-built）

状态：active（PLAN-809）。

## 职责与边界

单源承载 pac.at 声明归一、Target/Pac 纯选择与投影、链接闭包、源文件选择与构建规划的**纯规则**。语言执行（AutoConfig 求值/转译）、Git/索引/FS 观测与物化、Builder 实际执行、theme compose 留 auto-man 具名 adapter。

依赖闭包恒轻：auto-val / auto-atom / auto-toolchain-core / auto-project-model（+log/glob-match）。无 auto-lang/Parser/VM/UI/进程/网络；外部消费证据 = test/fixtures/modularity/man-core-consumer（cargo tree 断言）。

## 模块与规则（as-built）

- `scene`：Scene（workspace/ui/c/default）。
- `backend`：BackendName（BackendType 轻等价：from_str 含 ark/remote 别名、output_dir rust→"back"）+ RenderSelection（Single/Multi/Split/Absent；from_value 对齐 BackendConfig 三形态）+ frontend/backend/has_multi 选择。
- `target`：Target/TargetKind/TargetOrigin 声明归一（from_with_probe 观测注入；2 参 from 兼容入口保留原 is_dir 探针冻结签名）+ libname/to_node/to_atom/groups 投影 + merge/link 族。
- `dir`/`group`/`node_ext`/`file_types`：数据与纯工具。
- `port`：Port + port_from_node（含完整编译器配置解析）。
- `pac`：pac_from_decl→PacDecl（root wrapper 提升/render-backend 选择/标量族/默认 port builder ninja|cargo|noop）+ 投影自由函数族（all_incs/pac_to_node/exe_path/display_title_in/targets_map/filters）。
- `closure`：resolve_target_deps 五函数族（直接 link 含外部名保留 all_links、传递边只追 map 内、dep_names 只收 map 内、重复 link 不去重）+ **resolve_target_closure 有界准入**（内部环→GraphError::NoProgress：unresolved 全名单+DFS 灰标环候选链；外部 link 不触发；错误后不开始 scan）+ assemble_major_targets + **下载决策族**（dep_fetch_action/pinned_version/admit_new_dep_libs 观测注入/admit_transitive_deps）。
- `selection`：select_flat_entries（skips 精确+glob/FileFilter/头文件）+ STD_SOURCE_SUBDIRS+聚合 + .at 排除表 + auto_recurse 过滤 + check_incs/scan_specific_srcs 解析序。
- `planning`：select_port_binding（PortSelection）+ select_rust_package（RustPackageSpec）+ **BuildOrderItem+BuildOrderStep/ordered_build_steps**（deps 先建、rename 去重；item_index/from_dep 保留首次访问的具体实例，执行器按索引取回，不按名字重猜）。

## 门面契约（auto-man 侧）

无效果字段类型 re-export 保身份（805 模式）；效果方法以 TargetEffects/DirEffects 扩展 trait 承接（use auto_man::* 方法语法可用）；Pac（效果字段承载者）留在 auto-man，纯方法一行委托 core。

## 验证锚点

crate 单测（closure/selection/planning/target/backend 全规则 oracle）；auto-man plan809_management_core（api_compat_probe 全字段构造 pin/production_delegation_parity/环/V-05）；man-core-consumer 外部全链。

兼容入口边界：Target 的旧二参 from/get_root_dir/merge 入口保留既有只读 Path::is_dir 探针；相应 _with_probe 入口显式注入观测。Port::default 的 OS/ARCH 是编译期目标平台常量。这些兼容边界不引入语言、进程或网络执行栈。


## SD-02 add docs/specs/project-model/design/generation-request.md

before=无独立请求数据合同；after=来源/上下文/原始声明、真实生成读取与sidecar三形态；rationale=R04的轻项目读取前置；AC02/03。
发布正文：

# generation-request — I-PROJECT 生成请求数据合同（PLAN-809 SD-02，as-built）

状态：active（PLAN-809）。

## 合同

一次请求承载 UI 生成所需全部工程侧数据：origin（StaticAdmitted{source,unsupported}/DynamicEvaluated{source}）、project_root、identity/display/runtime/scene/members/render（RenderDecl）/toggles/theme_decl（ThemeDeclLite）/dep_roots/package_assets（npm_deps/styles/i18n 求值 Value）/exe_name/has_rust_sidecar/front_decl/input_sources/output_root/mode/raw+raw_nodes。

不变量：
- 无副作用（构造零网络/建目录/编译）；
- 无伪造求值（is_evaluated：静态 origin 需 unsupported 空；RenderSpec::Unknown 不假称前端）；
- 无字段丢失（未知键 raw 保留；B 组键 npm_deps/styles/i18n/exe_name/front/theme 块从 raw 目录回收）；
- 路径按 source 根纯代数（resolve_against_root，cwd 无关）；
- locale 链 display_title(prefers_zh) 镜像 Pac::display_title_in。

## 消费接线（as-built）

auto-man `generation_request_adapter`：request_from_pac（A 组全量投影）+ apply_evaluated_root/apply_evaluated_props（B 组，Pac.props 即求值 root 全量 props）+ request_of/request_of_in 调用入口（输入清单、Generate/Run、输出根由实际调用上下文投影；Automan 应用有效 render 覆盖）。生成器读缝：jet/ark（JetReads/ArkReads）、rust_ui（PacSource：name/exe/window/title/dep_roots/api/external/theme 激活）、vue（VueReads 九字段+Value 等价转换）；全部 `_with_request` 入口，legacy 自扫保留为冻结 oracle（R04 退役）。automan gen/run/build/open 分派全量经请求（零遗留 grep 审计）。

## 边界（as-built 校准，R4）

- **sidecar 声明在本合同内**（R4-02 后）：节点/对象/嵌套三形态经请求承载（raw_nodes+raw），core::sidecar_decl_from_root 单源解析，PacSource 全生成/交付入口消费，不回读主 pac.at。
- dep 自身 pac.at 的 npm_deps/styles 合并（vue 构建内跨文件资源读取，主声明已合同化）、api_gen 的 auto_lang resolve_back_dir——跨文件效果边，R04 迁生成器时收编。
- 全量/无损表述以本 Plan 实测矩阵为准（对拍+来源反转+干扰排除用例见 plan809/adapters 测试）。

验证：project-model generation_request_tests（8）+ project-request-consumer（3+闭包单证）+ vue plan809_v02 双源对拍 + generation_request_adapter 2 完备性。


## SD-03 modify docs/specs/project-model/project.md

before=静态模型及PLAN-786有限消费者；after=保留原静态合同并增加独立请求消费者；rationale=动态已求值输入不扩张静态准入域；AC02/03。
局部修辞：将PLAN-786段的“是当前唯一生产消费面”澄清为“是静态pilot的当前唯一生产消费面”；保持该段64/65保守护栏及其他正文。
追加以下独立模块节（如并集中已存在则协调替换该节，避免重复）：

## generation_request 独立消费（PLAN-809 SD-03，as-built）

`generation_request` 为具名独立模块（非静态扫描器一部分）：输入可以是静态准入描述（from_descriptor——静态子集与 786 护栏不变、不扩语法域）或动态求值后的显式投影（dynamic/直接构造），不强制工程先过静态扫描器。静态 origin 携带 unsupported 不伪造已求值（is_evaluated 诚实判定）。外部消费证据 = project-request-consumer（闭包 auto-atom/auto-val 单证）。


## SD-04 modify docs/specs/auto-man/project.md

before=原管理规则和生成读取分散在Man；after=core委托/请求读取及具名效果边，声明解析不延期；rationale=准确记录所有者和剩余模块；AC01..04。
追加以下节（如并集中已存在则协调替换该节）：

## PLAN-809 管理核心委托（SD-04）

- 单一规则源为auto-man-core：声明归一、链接闭包与有界图准入、源选择、构建步骤身份及下载决策。数据类型重导出保持身份；效果方法由TargetEffects/DirEffects承接，调用方需引入相应trait（use auto_man::*包含）。Pac/Builder/Automan仍是auto-man中的效果承载者。
- I-PROJECT为auto-project-model的独立generation_request模块；vue/jet/ark/rust_ui/tauri的项目读取及gen/run/build/open分派经显式请求adapter。静态模型/786保守护栏保持，动态求值不伪装为静态准入。legacy入口保留为兼容和行为oracle。
- sidecar声明解析属于本合同：core单源解析节点/Obj/属性Node，保持节点优先；请求读取在同一根层还原声明，不回读主pac.at。模块复制/Cargo注入仍是auto-man的文件效果，未把声明解析延期到R04。
- 仍留上层的效果/模块：AutoConfig求值、转译、Git/索引下载与文件观测物化、三Builder命令执行、exporter、UI生成算法本体；依赖自身pac.at的资源合并及api_gen::resolve_back_dir跨文件效果保持具名边界。后续搬迁不意味着本计划已把整个CLI或UI生成器独立化。
- auto-man-core属于man-ecosystem域，normal/build/dev依赖闭包不反向进入语言/Man/UI执行栈；交付给后续R04的接口版本以本计划实际合入SHA为准，未合入工作树提交不作为已发布接口。