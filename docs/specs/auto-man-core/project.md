# auto-man-core — 管理/构建规划规则核心（PLAN-809 SD-01，as-built）

状态：work 树 canonical 草案（随 PLAN-809 merge 入主检出）。

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
- `planning`：select_port_binding（PortSelection）+ select_rust_package（RustPackageSpec）+ **BuildOrderItem+ordered_build_renames**（deps 先建/rename 去重执行序）。

## 门面契约（auto-man 侧）

无效果字段类型 re-export 保身份（805 模式）；效果方法以 TargetEffects/DirEffects 扩展 trait 承接（use auto_man::* 方法语法可用）；Pac（效果字段承载者）留在 auto-man，纯方法一行委托 core。

## 验证锚点

crate 单测（closure/selection/planning/target/backend 全规则 oracle）；auto-man plan809_management_core（api_compat_probe 全字段构造 pin/production_delegation_parity/环/V-05）；man-core-consumer 外部全链。

## PLAN-831 展示面委派（C2，as-built）

pac_from_decl 完整 Man 规则仍居本 crate；其 theme/external_backend 展示字段一行委派 auto-project-model 单源（SidecarDecl/pac 展示提取）。sidecar 旧入口再导出 shim 保身份；UI tools 与 auto-api-gen 不再依赖本 crate。
