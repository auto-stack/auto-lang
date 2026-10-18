# PLAN-809 T-01 合同基线（contract-baseline）

冻结日期：2026-10-18。基线代码：master `2046a025b`（worktree `D:/autostack/.wt/lang-809/auto-lang`，分支 `codex/plan-809-dev`，同基面）。
本文是 PLAN-809 的合同冻结物：穷举 Pac/Target/Dir/Port/Builder/UI 生成器读点，划定规则/效果归属、
API 兼容方案、I-PROJECT 字段合同、图错误合同、真实工具命令矩阵。T-02..T-08 按此执行；发现与本文冲突时
先回本文修订（recorded adjustment），不得静默偏离。

## 0. 前置核实

depends_on 七计划全部 archived（docs/plans/archive/ 下 780/786/791/795/800/803/805 均在）。
主检出无 `crates/**`、`test/**`、`packages/**` 未提交 WIP（2026-10-18 核实，仅 docs/plans 与 .autoos 簿记）。
六个前置轻包已在 auto-man Cargo.toml 登记依赖（auto-project-model/auto-package-lock/auto-package-resolver/
auto-man-assets/auto-toolchain-core/auto-gen），对应门面文件就位（见 §4）。

## 1. 核心结论（先读这节）

1. **core 输入边界是 Node/Obj，不是 AutoConfig**。`AutoConfig = {code: String, root: Node, args: Obj}`
   （crates/auto-lang/src/config.rs:53），VM 执行（`eval_config_with_vm`）只发生在 `from_code/parse/read`；
   `Pac::new` 全程只读 `config.root`/`config.args`（auto_val 类型）。因此 core 的声明归一入口签名为
   `pac_from_decl(root: &Node, args: &Obj)` 一类，语言求值留在 auto-man adapter。`AutoConfig`/`BackendConfig`/
   `ThemeDecl` 不进 core。
2. **唯一类型重导出是既定模式**。795/800/803/805 四计划已确立：纯数据迁入轻 crate，auto-man 旧路径
   `pub use core::*` 保持类型身份（如 `builder/ninja/config.rs:10` 对 auto-toolchain-core）。Pac/Target 的
   **公开字段**同理处理（见 §8）。
3. **纯规则中有两处内嵌 FS 探针**：`get_root_dir` 的 `apps/<name>` is_dir 检查（target.rs:127-133）与
   `Target::merge` 的 `path.is_dir()`（target.rs:1163-1166）。core 化后改为**注入谓词**
   （`dir_exists: impl Fn(&str) -> bool`），auto-man 侧传真实 FS 实现，测试传内存 map——规则单源、
   观测效果边。
4. **链接闭包是纯函数族，环=挂死已证实**。`resolve()` 的 `while !done`（pac.rs:1055-1057）在内部
   target 环（A↔B 或自环）下永不收敛；外部 link（名字不在 targets_map）被 `continue` 跳过、不阻塞。
   有界图准入=迭代无进展检测 + 具名错误（§7）。
5. **HashMap 序已非确定**。`resolve()` 末段 `targets_map.values()` 迭代（HashMap RandomState）决定
   major_targets 顺序——现行为本就跨运行不稳定，oracle 必须用集合/偏序比较，不承诺字节稳定排序。
6. **I-PROJECT 落 auto-project-model 新模块 `generation_request`**，复用其 SourceInfo/provenance/raw 保留
   体系；`DependencyEntry<M>` 不透明泛型标签（auto-package-resolver）是"携带语言侧模式语义但不让 core
   读它"的既有解法先例。

## 2. 读点穷举：Pac / Target / Dir / Port / Group

### 2.1 Pac（pac.rs，2319 行）

字段全集（pac.rs:51-189）：name, backend, backend_config(Option<BackendConfig>，语言类型), api, port(Port),
version, ports(Vec<Node>), builder(Option<Box<dyn Builder>>), targets(Vec<Target>), index/device_index
(Shared<Index>), props(Obj), build_at, build_location, show_headers, defines, scene(Scene), members,
front_port/back_port(Option<u16>), media_root/photo_root/window(Option<(f32,f32)>)/window_fit/title/title_zh/
icon/category/theme/theme_decl(语言类型)/accent/desktop_render/ui_config/external_backend(Option<AutoStr>),
shadcn/default_classes(bool), is_update(bool, 私有)。

方法归属表：

| 方法 | 行 | 归属 | 说明 |
|---|---|---|---|
| `new(AutoConfig)` | 196 | **规则→core**（拆为 from_decl） | root wrapper 提升、render/backend 解析、全部标量字段解析、targets 收集、lang 继承、未知 kind 警告跳过。仅消费 root/args |
| `display_title_in` | 542 | core | 纯判定 |
| `transpile_autot` | 555 | adapter | 语言转译 |
| `has_backend_config/default_frontend/frontend_types/backend_type/frontend_output_dir/output_dirs/is_backend` | 567-641 | **core**（BackendConfig 以视图输入） | 依赖 BackendConfig（auto-lang 类型）——core 侧输入改为归一后的轻投影（§8.2） |
| `is_workspace/is_ui/workspace_members/has_multi_backend` | 648-674 | core | 纯 |
| `print_target_info/print_targets` | 678-700 | adapter | 表格打印（tabled） |
| `has_device_prop` | 702 | core | 纯 |
| `print_atom` | 721 | adapter | `.am/*.atom.at` FS 写 |
| `get_target/deps/libs/apps/build_targets/build_targets_mut/contains/copy_target/list_port_names` | 743-784 | core | 纯过滤 |
| `get_state_file/try_load_port/save_port` | 787-829 | adapter | `.am/state.at` FS IO + 交互选择 |
| `set_port` | 831 | **规则 core + 效果 adapter 拆分** | build_at/build_location/builder 名选择是规则；`make_builder` 构造与 println 是效果。core 产 `PortSelection{builder_name, build_at, build_location}` |
| `set_show_headers` | 864 | core | 纯 |
| `targets_map/get_build_names` | 877-894 | core | 纯 |
| `resolve_target_deps/check_all_resolved/build_resolved_links/try_resolve_target_links/apply_resolved_links` | 901-1003 | **core**（本计划主迁物） | 链接闭包纯函数族，现 private |
| `pull/download` | 1005-1017 | adapter | 网络/git |
| `resolve` | 1019 | **编排拆分** | download(effect) + set rel/defines/port(数据) + 闭包(core) + major 合并(core 规则) + scan(effect, 选择规则在 core) |
| `merge_port` | 1111 | core | port Node→Port 纯解析（含 CompilerConfig 解析，类型已在 auto-toolchain-core） |
| `update_repo/download_deps/download_git/copy_local/materialize_local_dep/download_dep/download_repo/starts_with_digit` | 1335-1672 | adapter | git/索引/FS/进程全效果；其中**去重/合并决策**（target_names 集合、new_deps 收集、lib merge 规则）抽为 core 纯函数（§3 D-组） |
| `exe_path` | 1674 | core | 纯投影 |
| `build/run/clean` | 1689-1726 | adapter | Builder 委托 |
| `collect_srcs` | 1728 | adapter（FS 经 Dir） | |
| `all_incs/to_node/to_atom/relative_path/path_depth/files_list` | 1744-1896 | core | 纯投影（files_list 带 #[allow(dead_code)]） |
| `Scene::from_str` | 41 | core | 纯 |

测试锚点（迁 core 时随之搬迁或留对拍）：test_title_zh_parsing_and_display_chain、test_desktop_render_parsing、
test_theme_decl_block_parsing、test_shadcn_parsing、test_default_classes_parsing、test_window_fit_parsing、
test_scene_parsing、test_pac、test_pac_dep_local、test_pac_atom_root_wrapper_flattened、
test_pac_unknown_kind_skipped_not_panicking、test_check_all_resolved*、test_build_resolved_links_no_links。

### 2.2 Target（target.rs，1700 行）

字段全集（target.rs:69-101）：name, version, kind, lang, origin, from, at, rename, is_scan, recurse,
show_headers, dirs(HashMap<AutoStr,Dir>), autos, srcs/incs(HashSet), selects/skips(HashSet), links(Vec<Node>),
is_resolved, is_main, props(Obj), deps(Vec<Target>), dep_names, all_links(Vec<Node>), all_deps, rel, defines,
port(Port)。

| 方法 | 行 | 归属 | 说明 |
|---|---|---|---|
| `TargetKind::from_str` / `TargetOrigin` | 31-53 | core | 纯 |
| `new/from(node,pac)` | 158/191 | **core** | 声明归一。`from` 内 `get_root_dir` 有 FS 探针→注入谓词（§1.3） |
| `extract_name_and_version/extract_kind/extract_origin/calculate_location/extract_directories/extract_selects` | 270-427 | core | 纯（extract_directories 用 dir_filter） |
| `location/local_name/config_name/set_parent/parent/all_incs/is_major/get_dir/has_auto/libname` | 429-624 | core | 纯（libname 按 port.platform） |
| `to_node/to_atom/get_extra_output` | 507-613 | core | 纯投影（IAR extout/groups 分支） |
| `transpile_auto/transpile_rust_project_multi/generate_api_server/transpile_auto_with_cache` | 625-1129 | adapter | 语言转译+FS 写+Parser/ApiExtractor |
| `exists/downloaded` | 1131-1146 | **观测**（效果边；core 接受注入谓词结果） | |
| `merge_link/merge/set_rel/set_port/set_defines/check_links/apply_link` | 1149-1194 | core（merge 内 1 处 FS 探针→注入） | |
| `check_dirs` | 1203 | 观测 | FS 校验（错误文本冻结） |
| `scan` | 1215 | **规则/效果拆分** | 选择规则（is_scan 分支、srcs/incs 并集、dirs 只取 incs 不取 srcs、deps 递归、scan_auto 的 .at 收集与排除表）→core；read_dir/路径探测→adapter 枚举器 |
| `scan_auto/scan_auto_dir` | 1295-1324 | 同上 | 排除表：pac.at/device.at/os.at；跳过 `.` 前缀与 `rust` 目录；仅 recurse 时下潜 |
| `clean` | 1326 | adapter | FS 删除+git check |
| `print_info/TargetStatus::from` | 1342-1351 | adapter（展示）/core（数据） | |
| `group_target/group_dir/groups` | 1358-1399 | core | 纯输出数据规划（IAR 组树） |
| `collect_srcs` | 1409 | adapter（FS 经 Dir） | |
| `get_root_dir`/`dir_filter`/`set_to_vec` | 119-155 | core（get_root_dir 注入谓词） | |

测试锚点：test_dir_from_node_with_demo（apps/demo 根目录规则）、test_target、test_extract_name_and_version_with_id、
test_extract_kind、test_extract_scan_flag*、test_extract_recurse_flag、test_extract_sources_and_includes、
test_extract_local_path_dep（origin/from/at 判定）、test_filter_dirs。三个 #[ignore] 测试（VM codegen 已知缺陷）
随迁保持 ignore 语义。

### 2.3 Dir / scanner / file_types / Group / Port

| 单元 | 归属 | 说明 |
|---|---|---|
| `Dir::from_str/from_node/set_lpath/update_root/to_node/append_root/logical_depths`（dir.rs:39-163） | core | 纯数据 |
| `Dir::scan/scan_recursive/scan_flat/scan_dir`（dir.rs:165-270） | **拆分** | 递归/筛选规则→core（以注入枚举器驱动）；fs::read_dir→adapter |
| `check_incs/scan_specific_srcs`（dir.rs:325-367） | 拆分 | 绝对/相对 root 解析规则→core；is_file/is_dir 探测→注入 |
| `scan_dir_flat`（dir.rs:369-448） | **规则 core + 枚举 adapter** | skips 精确名+glob 过滤、FileFilter.should_include、头文件处理（use_header 决定 srcs 或 incs 标记）是核心选择规则；read_dir 是效果 |
| `scanner.rs`（192 行） | scaffolding | `scan_standard_subdirs`（已参数化注入 scan_dir_flat）规则→core；`scan_directory` 是 **TODO 空壳**（早期重构遗留，本计划清偿或删除，登记） |
| `file_types.rs`（253 行） | core | FileType/FileFilter 纯注册表 |
| `group.rs`（92 行） | core | Group 树/XML 纯输出 |
| `Port`（port.rs） | core | 数据+解析；`Port::default()` 读 env::consts::OS→改为显式 platform 输入（core 提供 `Port::for_platform(os,arch)`，auto-man 传真实 env） |
| `node_ext.rs`（62 行） | core | NodeExt trait 纯扩展 |
| `index.rs` | 拆分 | `Index{map}/lookup/PacInfo`→core 数据；`Index::load`（FS+AutoConfig 解析 index.at）→adapter；`IndexStore/PacIndex`（git pull/list 展示）→adapter |
| `util.rs` | 拆分 | hash/split→core 工具；`select_or_default_port/select_backend`（dialoguer 交互）、`kill_process_on_port/http_*`→adapter |
| `error.rs`（AutoManError） | core（错误类型随迁） | |
| `git.rs/fs.rs/pkg.rs/up.rs` | adapter | 全效果（进程/网络/FS） |

### 2.4 输入边界证据

- `AutoConfig`（auto-lang/src/config.rs:53）：`{code, root: Node, args: Obj}`；`Pac::new` 只读 root/args。
- `BackendConfig`/`BackendType`（auto_lang::config）：Pac::new 内 `BackendConfig::from_value(&effective_value)`
  一次调用后仅做枚举匹配。core 侧以自有归一枚举承接（§8.2），auto-man adapter 保留 BackendConfig 解析并转换。
- `ThemeDecl`（auto_lang::design_tokens::decl）：Pac::new 内只做 Obj→字段提形。core 侧持轻声明结构
  （name/extends/mode/colors: Vec<(String,String)>），auto-man 转换为语言 ThemeDecl 喂 compose。
- `Index::load` 读 index.at 经 AutoConfig——core 只收 `HashMap<AutoStr, PacInfo>`。

## 3. 规则/效果归属总表（对计划 §5.2 表的逐项落地）

**core（auto-man-core，依赖仅 auto-val + auto-toolchain-core + auto-package-lock/resolver 视需要）**

| ID | 规则 | 现位置 | 迁移形态 |
|---|---|---|---|
| R-01 | pac 声明归一（全部标量字段/scene/members/toggles/window/title 族/theme 族/port 解析） | Pac::new + merge_port | `pac_from_decl(root,args)` + `port_from_node(node)` |
| R-02 | target 声明归一（kind/origin/from/at/rename/dirs/links/selects/skips/lang） | Target::from 族 | `target_from_node(node, pac_name, dir_exists)` |
| R-03 | 链接闭包（direct/transitive links、dep_names、all_links、major 合并、is_resolved） | resolve_target_deps 族 + resolve 中段 | `resolve_target_deps(&mut map) -> Result<NoProgressInfo>` |
| R-04 | 源选择规则（FileFilter、skips glob、头文件、标准子目录表 src/include/inc、.at 排除表、scan_specific/check_incs 解析序） | dir.rs/file_types.rs/scanner.rs/target.rs scan_auto | `select_dir_entries(entries, kind, skips, show_headers)` 等（枚举器注入） |
| R-05 | 场景/端口/后端选择（builder 名选择、build_at/build_location、show_headers 联动） | set_port 前半 | `select_port_binding(port, backend) -> PortSelection` |
| R-06 | Group/纯输出数据规划（to_node/to_atom/all_incs/groups/extout/libname/exe_path/files_list） | pac.rs/target.rs/group.rs | 直迁 |
| R-07 | 依赖下载的**决策**面（按 origin 分类、exists 跳过、target_names 去重、new_deps 收集、lib merge、版本 pin 规则 latest→master/v 前缀） | download_deps/download_repo 中段 | `plan_downloads(targets, exists_pred) -> DownloadPlan`（每项 typed：{target, origin, version_pin}）；adapter 执行 |
| R-08 | 有界图准入（新增，§7） | —（新合同） | resolve 循环无进展检测 |

**adapter（auto-man 保留，具名不伪报）**

| ID | 效果 | 现位置 |
|---|---|---|
| E-01 | AutoConfig 求值（VM）、index.at 读取 | AutoConfig::from_code/Index::load |
| E-02 | git clone/checkout/pull/detached 检查 | git.rs/download_repo |
| E-03 | 本地依赖物化（worktree/junction/symlink/copy 回退） | materialize_local_dep/copy_local |
| E-04 | FS 枚举与探测（read_dir/is_dir/is_file） | dir.rs scanner.rs 内 |
| E-05 | 语言转译（a2c/a2r/a2ts、multi-file、API server 生成） | target.rs transpile 族 |
| E-06 | Builder 执行（ninja/cargo/vue 命令、进程、日志） | builder/ 树 |
| E-07 | 状态与输出文件（.am/state.at、atom.at、pac.lock 读写、exporter） | pac.rs/lock.rs/exporter |
| E-08 | 交互选择（dialoguer 端口/backend 选择） | util.rs |

## 4. 已有轻包复用边界（前置调查结论）

- **auto-package-lock**（795）：LockEntry/LockFile/codec/verify 全规则已在包内；auto-man/lock.rs 是
  Target/Git adapter（投影+git commit 观测）。809 **不复制**；core 若需锁类型，直接依赖该包。
- **auto-package-resolver**（800，std-only）：DependencyEntry<M> 不透明模式标签先例。
- **auto-toolchain-core**（805）：CompilerConfig/FlagMapper/CommandTemplates 已单源；`builder/ninja/{config,mapper,templates}.rs`
  是纯 re-export。**Port.compiler 类型即 auto_toolchain_core::config::CompilerConfig**——core 依赖它即合法继承。
- **auto-man-assets**（803）/ **auto-gen-core**（791）：资产与生成编排，809 不动其边界。
- **auto-project-model**（780/786）：ProjectDescriptor/raw_props/raw_nodes/unsupported/provenance 体系；
  786 唯一生产消费面 project_display_reader（64/65 护栏）。809 的 generation_request 模块落此包
  （SD-02/SD-03），**不扩大静态语法域、不伪造静态等价**。
- **auto-man→auto-lang 语言边是 777 基线 grandfather 项**：本计划不给 auto-man 减语言依赖（那是 R08/R09），
  只保证新 core 闭包干净。

## 5. UI 生成器读点穷举与 I-PROJECT 字段合同

### 5.0 总体发现（决定 I-PROJECT 形态）

**9 个生成器文件（vue/api_gen/rust_ui/jet/ark/tauri/tauri_backend/vue_shadcn/node_ext）没有一个接收
`&Pac`/`&mut Pac`；Target 零读取。** 全部入口只收 `root_dir: &Path`（个别加 output_dir/backend/bool）。
工程状态获取分三层：

- **层 A（生成器内行级文本扫描 pac.at 原文）**：纯数据投影。散落 7+ 个独立扫描函数
  （parse_pac_name/parse_shadcn/parse_default_classes/parse_npm_deps/parse_style_files/parse_i18n/
  parse_pac_window/parse_pac_title/parse_pac_exe_name/parse_pac_dep_roots/has_jet_backend/has_ark_backend/…）。
- **层 B（生成器内重型解析）**：AutoConfig::from_file（VM 求值）+ 内部 Pac::new——**仅两处字段真正
  需要它**：theme_decl（vue.rs:3253-3266、rust_ui.rs:6379-6391）与 external_backend（rust_ui.rs:6061-6071；
  api_gen.rs:47 经 auto-lang resolve_back_api）。其余 Parser/a2r 转译是生成算法本身（R04 域）。
- **层 C（调用方 &Pac 读取）**：automan.rs 的 pac_* getter 族 + main.rs run handler 把字段**种成环境
  变量**（AUTO_VM_WINDOW/AUTO_VM_TITLE/AUTO_UI_THEME/AUTO_HTTP_PORT 等 14+ 个）转交生成器/运行时。

推论：I-PROJECT 合同 = 把层 A 的散乱扫描与层 C 的字段透出**统一为一份具名请求数据**；生成器签名从
`root_dir` 改为请求时，其读取语义不变（同 key 同值），只是来源单源化。

### 5.1 字段合同（T-02 冻结实现）

**A 组：Pac 字段直投（纯投影）**——name、title/title_zh（含 locale 链 display_title）、icon、category、
theme（标量）、accent、desktop_render、ui_config、media_root、photo_root、window/window_fit、
front_port/back_port、api、scene、members、shadcn、default_classes、external_backend（路径字符串）、
backend/frontend 选择（以归一 frontend_types 结果承载，非 legacy 字符串）。

**B 组：pac.at 键、Pac 结构体没有的（层 A 扫描键）**——npm_deps（数组/对象/link 三形态）、styles、i18n、
exe_name、dep{path} 清单（本地依赖根，含 deps/*/pac.at 合并语义的**输入**：dep roots 列表）、
rust_sidecar 声明、front 目录声明（front: 路径 / app("front") 语法）、back.project。这组键证明
**合同字段集 ⊃ Pac 字段集**——I-PROJECT 不是 Pac 的子集投影，而是"生成所需工程数据"的完整合同。

**C 组：来源/覆盖顺序**——请求携带 `origin: RequestOrigin`（StaticAdmitted{source: SourceInfo} |
DynamicEvaluated{…}），静态来源不得宣称已求值（对齐 project-model 既有 unsupported 语义）；
覆盖顺序冻结：CLI 显式 > 请求携带声明 > 内建默认（与 main.rs env 注入序一致；rust_ui.rs:2664-2689
的生成物内嵌 env 与 main.rs env 注入是双通道，抽取时统一为 CLI env > pac 声明，行为不变）。
路径按 pac/source 根解释而非 cwd（§5.3 P-02）。

**D 组：明确不进合同（重效果）**——theme_decl 的**取值路径**（AutoConfig VM 求值+resolve）：合同携带
原始块声明（轻声明结构）或已 compose 结果，compose 由 adapter 完成；.at 全量编译（Parser/
ui_build_shadcn_*）、api.at 解析（try_full_parse）、a2r 转译、cdylib 装载（rust_ui merged）、后端项目
pac.at name 探测、AutoVM HTTP server——全部留 R04/运行期。

**无副作用**：请求构造零网络/零建目录/零编译；消费请求不触发 pac.at 重读（这是消灭"每 demo ~2s
重复解析"热点（gallery 缓存注释）的合同基础，但性能优化本身不是验收项，行为等价才是）。

### 5.2 生成器 × 字段消费矩阵（V-02 oracle 面）

| 字段/键 | vue | jet | ark | rust_ui | api_gen | tauri | 调用方(main.rs env) |
|---|---|---|---|---|---|---|---|
| name | ✓(3311) | ✓(21) | ✓(19) | ✓(3034) | — | — | ✓(AUTO_APP_ID) |
| title/title_zh | ✓(2531/3311) | — | — | ✓(3097) | — | — | ✓(AUTO_VM_TITLE) |
| window/fit | — | — | — | ✓(3078/2665) | — | — | ✓(AUTO_VM_WINDOW) |
| theme/accent | env | — | — | env(6304) | — | — | ✓(AUTO_UI_THEME/ACCENT) |
| theme_decl | ✓(3260) | — | — | ✓(6387) | — | — | — |
| shadcn/default_classes | ✓(2557/2587) | — | — | — | — | — | — |
| npm_deps/styles/i18n | ✓(2608/2727/2778，含 dep 合并) | — | — | — | — | — | — |
| exe_name | — | — | — | ✓(3053) | — | — | — |
| dep{path} | ✓(3160) | — | — | ✓(2796) | — | — | — |
| rust_sidecar | — | — | — | ✓(563) | ✓(1224) | — | — |
| front 路径 | ✓(2489) | — | — | — | — | ✓(编排) | — |
| back.project/external_backend | — | — | — | ✓(6070) | ✓(47) | ✓(编排) | — |
| backend/render 选择 | — | ✓(75) | ✓(35) | — | — | — | automan(1221/1646) |
| api | — | — | — | ✓(6148) | backend 参数 | ✓ | ✓(AUTO_BACKEND_IMPL) |
| media_root/photo_root | ✓gallery(7264) | — | — | — | — | — | ✓(AUTO_MEDIA/PHOTO_ROOT) |
| front_port/back_port | env | — | — | env | — | — | ✓(AUTO_HTTP/FRONT_PORT) |
| desktop_render/ui_config/icon/category/scene/members | — | — | — | — | — | — | automan getter 族/main |

（括号内为 vue.rs 等文件内行号证据；"env"= 经环境变量间接消费。）

### 5.3 接线风险与预案（T-06 冻结）

- **P-01 重复解析热点**：workspace gen 中 automan.rs:1216 Pac::new 与各生成器 from_workspace 再读
  pac.at——改请求后一次构造多处消费；行为等价以"同输入产物对照"验收（V-02），不以时延为验收。
- **P-02 cwd 语义**：现生成器以 root_dir 相对路径工作；合同路径按 pac/source 根解释——接线时逐点
  核对（V-02 有"不同 cwd 路径仍按 source 根"用例）。
- **P-03 双通道 env**：rust_ui 生成物内嵌 env 与 main.rs env 注入——统一优先级 CLI > pac，输出不变。
- **P-04 external_backend**：api_gen 侧行扫（app_registry.rs:348 先例）可平铺进合同；rust_ui merged
  模式的后端 cdylib 装载（6093-6253）留运行期 adapter，合同只带路径字符串。

## 6. Builder 树与效果边界

### 6.1 Builder trait（builder.rs:15-28）与工厂

8 方法：`build(&mut self, &mut Pac)`、`setup(&mut self, &mut Pac)`、`finish(&mut self, &Pac)`、
`target(&mut self, &Target, &Pac)`、`clean(&mut self)`、`run(&mut self, &Pac, Vec<String>)`、
`enable_memory_output`、`get_memory_output`。工厂：`BuilderKind{Ninja(path),Cargo(path),Vue(path)}`
+ `from_str("ninja"|"cargo"|"vue", path)` + `create_builder/new_builder/make_builder`（builder.rs:30-73）。
**iar/ghs/cmake 不走 Builder**——automan.rs:1033-1039 注明 make_builder 只识三族，IDE 工程导出走
exporter 分流（exporter/{cmake,ghs,iar}.rs）。

### 6.2 三实现的 Pac/Target 读点（V-05 冻结面）

| Builder | Pac 读取 | Target 读取 | 副作用（效果边，保留 auto-man） |
|---|---|---|---|
| Ninja（builder/ninja/builder.rs，549 行） | targets 遍历（deps/rename 去重排序）、**collect_srcs()（&mut！）**、port（compiler/name）、all_incs()、defines、name、get_target()、port.at(run) | name、kind、srcs（仅 .c 经 build_source）、links+main_arg().repr()、libname()、port.platform、rename、deps、incs、defines（后三者仅喂注释掉的 CMake 遗留代码） | CompilerResolver::resolve_executable×4（PATH/磁盘）、reset_ninja_file（删+建）、写 build.ninja（includes/defines/cflags/rules）、finish 经 resolve_ninja_host spawn ninja/n2、run 扫 exe+dialoguer 交互、clean 删文件/目录 |
| Cargo（builder/cargo.rs，383 行） | targets、name | lang=="rust" 过滤、version（空/latest→0.1.0）、kind（App→main.rs/Lib→lib.rs）、name（空回落 pac.name）、deps[].lang/name | 写 Cargo.toml、create_dir_all(src)、缺桩 main/lib、src/**/*.rs 内容扫描推断 crate 依赖（scan_rs_for_crates）、router/commands.rs 存在→注入 axum/tauri 依赖、find_auto_lang_path 上溯、spawn cargo build/run/clean |
| Vue（builder/vue.rs，98 行） | **零读取**（参数全 `_pac`/`_target`） | 零 | pkg::run_script("build"/"dev")（npm/pnpm 进程）、remove_dir_all(dist) |

**core 化影响**：Builder 的 trait 签名不变（仍收 &mut Pac/&Pac——Pac 类型经重导出身份不变）；
Ninja/Cargo 对 Pac/Target 的**读取面**就是 V-01/V-05 的字段 pin 清单（§6.2 第一二列）；规划性逻辑
（dep 先建排序、Cargo.toml 内容合成、build.ninja 行合成模板参数）中**纯数据决策**部分随 R-05/R-06
进 core，进程/文件执行留 adapter。Ninja setup 里 `pac.collect_srcs()` 是 **&mut Pac 的隐藏状态变更**
（把 dirs.srcs 汇入 target.srcs）——core 化后归入"构建动作规划"的显式步骤（R-06），不再藏在 builder
setup 内。

### 6.3 ninja/ 子树剩余归属（805 后）

- `compiler_store.rs`（281 行）：am.at 动态加载（AutoConfig 解析）——**adapter**（语言+FS）。
  P805-D1 已知债（get_kid Str 键 vs Int 键恒空载）不动。
- `resolver.rs`（367 行）：PATH/磁盘/进程观测（validate/get_version spawn 编译器）——**adapter**。
- `runner.rs`（223 行）：ninja/n2 探测链 + `cargo install --git` 自动安装——**adapter**（含网络）。
- `builder.rs`（549 行）：编排+AutoGen 模板渲染+执行——**adapter**，规划输入经 core 投影。

### 6.4 其余效果设施

up.rs（296 行）**未编入**（lib.rs:18 注释禁用，zip 依赖已移除）——不迁不修。pull.rs 仅测试。
sidecar.rs（348 行）：pac.at rust_sidecar 节读取（AutoConfig）+ .rs 复制/Cargo.toml 注入——adapter，
其**声明解析**（SidecarSpec 提形）是 R-01 同族规则，随 core 提供纯函数、IO 留 adapter。

## 7. 有界图准入失败合同（新增，R-08）

**现行行为**（pac.rs:1054-1057）：`while !done { done = resolve_target_deps(&mut map) }`。

- 终止性：无环时每轮至少新解出一个 target，≤ n 轮收敛（is_resolved 单调）。
- 挂死条件：存在内部 target 环（A→B→A 或自环 A→A）时，环成员永非 resolved，`check_all_resolved` 恒 false。
- **外部 link 不阻塞**：`try_resolve_target_links` 对 `targets_map.get(&link.id())` 未命中 `continue`
  （pac.rs:958-960）——名字不在 map 的 link 视为外部/系统链接，允许存在且不计入传递闭包。
  **新错误合同不得把外部链接名判为 missing dependency**（计划 §5.3 明确）。

**新合同**（迁入 core 时生效，仅对新调用路径；旧行为已挂死无 oracle 可冻结）：

1. 每轮记录新 resolved 数；一轮结束 resolved 计数零增长且仍有未 resolved target → 返回
   `GraphError::NoProgress { unresolved: Vec<TargetRef>, cycle_candidates: Vec<Vec<AutoStr>> }`。
2. `cycle_candidates` 由未 resolved targets 的 link 边导出的子图内做 DFS 找环（具名带 target 路径，
   如 `app(main) -> dep(common) -> app(main)`）；找环失败时退化列出全部 unresolved 及其出边（可证明
   无进展仍成立）。
3. 错误在 resolve 阶段返回，**不开始后续 scan/build**；错误类型带 Display 文本人链。
4. 合法 DAG/外部 link/重复 link 行为与旧臂逐字段一致（V-04 手算 oracle）。

**oracle 设计**：diamond 图、重复 link、断开图（外部 link 名）、自环、双环、无进展混合图——期望值
手算（闭包成员/dep_names/all_links 集合比较 + 错误的 unresolved 名单集合），不用旧臂运行输出作
排序敏感比较（HashMap 序，§11）。

## 8. API 兼容冻结方案

### 8.1 公开面盘点

auto-man lib.rs `pub use pac::* / target::* / dir::* / port::* / group::* / builder::* / automan::* / ...`
（lib.rs:99-153）。Pac/Target/Dir/Port/Scene/TargetKind/TargetOrigin/Group/File/PacInfo/Index 均是
公开类型且**公开字段**可被外部构造（计划 §5.3：pin 结构体构造 consumer）。

### 8.2 方案：分类迁移 + 扩展 trait 保方法语法

**类 1——无效果字段的结构体（Target/Dir/Port/Scene/Group/File/TargetKind/TargetOrigin/PacInfo/
FileFilter/FileType/NodeExt）→ 整体迁 core**；auto-man 对应模块 `pub use auto_man_core::...`
（类型身份不变，805 模式）。Rust 不允许跨 crate inherent impl（计划 §2 红线），因此这些类型上的
**效果方法**（Target::transpile_auto/transpile_rust_project_multi/generate_api_server/
transpile_auto_with_cache/scan/scan_auto/scan_auto_dir/check_dirs/clean/collect_srcs/print_info、
Dir::scan 族、Index::load 等）在 auto-man 以**扩展 trait**承接：`pub trait TargetEffects
{ fn transpile_auto(&mut self) -> ...; ... }`，impl for core::Target，旧模块路径 `pub use` 该 trait。
方法调用语法 `t.transpile_auto()` 对 glob 导入者不变（auto-man 内部与 `auto_man::*` 导入者均覆盖）；
这是字段构造兼容之外的唯一签名级风险点，T-03 编译 probe 显式 pin。

**类 2——Pac（有效果字段 builder/index/device_index/backend_config/theme_decl）→ 结构体留在
auto-man**，全部旧 pub 字段名与类型不动（`targets: Vec<Target>` 中 Target 即 core 类型经重导出）。
Pac 的纯规则方法保留为 Pac 的 inherent impl（合法——Pac 仍定义于 auto-man），函数体一行委托 core
自由函数（`frontend_types`→core::frontend_types(&self.render_view()) 等）。`Pac::new(AutoConfig)`
签名不变，函数体委托 `core::pac_from_decl(&config.root, &config.args)` 后装配效果字段。

**类 3——纯函数族直迁**：resolve_target_deps 五函数（原 private，升 core pub）、targets_map/
get_build_names/merge_port 解析/to_node/to_atom/all_incs/groups/path_depth/exe_path 等。

**backend_config 处理**：core 的 PacDecl/规则视图持自有 `RenderSelection`（Single/Multi/Split 轻
枚举 + legacy 字符串；BackendType 轻等价复制于 core，from_str 对齐 auto-lang 版）；auto-man
`Pac.backend_config: Option<BackendConfig>` 字段保留，纯方法委托时做一次 BackendConfig→
RenderSelection 转换。`theme_decl` 同理：core 持 `ThemeDeclLite{name,extends,mode,colors}`，
auto-man 转语言 ThemeDecl。

**不改**：pac.at 格式、错误文本、HashMap/HashSet 字段类型（AutoStr 键）、公开方法签名。

**编译 probe**（T-03 落地为 `crates/auto-man/tests/plan809_management_core.rs` 的 `api_compat_probe`
模块）：字面量构造 Pac{...全字段...} 与 Target{...全字段...}（证明字段构造不被破坏）、调用全部 pin
公开方法（frontend_types/is_workspace/display_title_in/get_target/build_targets/libname/to_node/
all_incs/exe_path/Target::from/Pac::new/check_links 等）、验证 `use auto_man::*` 下扩展 trait 方法
语法可用。

### 8.3 不兼容时的处理

按 §8.2 分类后，风险最高的是扩展 trait 方法语法对**非 glob 导入者**的影响——排查：仓内消费者
（auto-man 内部、crates/auto/src/main.rs、tests/）均为 glob/模块路径导入，probe 覆盖；auto-man 无
仓外消费者（workspace 内部 crate）。若实施中发现第三类不兼容（如 Shared<Index> 生命周期），回到
本文修订并记 recorded adjustment，不得偷偷 Deref 包装或静默删字段。

## 9. 双状态消除方案

- **单份类型**：Target/Dir/Port 等只有 core 一个定义；Pac.targets 的元素类型即 core::Target
  （经重导出同身份）。auto-man 不存在第二份 target/dir/port 状态结构。
- **单份状态**：可变状态（targets/props/defines/is_resolved...）的权威就是 Pac/Target 本体；
  效果方法（set_port/scan/download/build）直接写这些字段；规则函数以 `&mut` 视图驱动同一份状态。
  不引入 PacDecl 运行时副本（PacDecl 仅作为 from_decl 的**构造产物形状**，构造后字段即搬入 Pac，
  不长期并存）。
- "字段读写后重新 view/规划时可见新值"（计划 §5.3）：resolve/scan 后 Pac.targets 即当前值；
  生成器经 I-PROJECT 快照读，不经缓存（§5.1 无副作用项）。
- to_atom/print/exporter 等消费直接走 core 投影函数。

## 10. 真实工具命令矩阵

### 10.1 CLI 命令 → Pac 方法调用链（crates/auto/src/main.rs，V-05 受影响命令面）

| CLI | main.rs | Automan 方法 | Pac 链 | 809 影响 |
|---|---|---|---|---|
| new/init | 1036/1066 | create_by_template/create_app | 无（Templates::copy） | 无 |
| build | 1098→1181 | new→build | transpile_autot + resolve + (iar/ghs/cmake: exporter.export \| else pac.build→builder.build)；rust 臂 resolve+transpile+build | **高**：resolve/build 全链经 core |
| run | 1258→1625 | new→run | UI 后端不经 Pac（vue/tauri/jet/ark/rust_ui/vscode 各 run_*）；rust 非 UI 臂直接 spawn cargo | **高**：pac_* getter 族改经 I-PROJECT/请求读取 |
| fetch | 1928→1942 | pull+scan | index_store.pull(git) → pac.pull→download→download_deps → resolve | **高**：download 决策经 core DownloadPlan |
| clean | 1862→1876 | clean | 读 .am/pac.atom.at→Pac::new→Dep/Device t.clean()；pac.clean→builder.clean | 中：Pac::new 经 from_decl |
| export | 2022→2035 | export | set_port(make_builder) → transpile_auto → resolve → exporter.export | **高** |
| gen | 2258→2290 | gen | workspace 成员逐个 Pac::new(1218)；无 builder | **高**：生成器读接 I-PROJECT |
| info | 2053→2066 | info | AutoConfig::read(pac.atom.at)→Pac::new→print | 中 |
| open | 2074→2087 | open_ide | 读 pac.port.builder/build_location/name | 低 |
| device | 1979→1987 | list_devices/set_port | pac.set_port→make_builder | 中 |
| deps/env/config/cache/test | — | — | 无 Pac | 无 |

### 10.2 真实工具执行面（V-05 driver 设计输入）

- Ninja 族：resolve_ninja_host 探测链（AUTO_NINJA env → PATH ninja → PATH n2 → **cargo install --git
  自动安装 n2**——driver 须预置 AUTO_NINJA 或 PATH 内 ninja，禁止触发网络安装）；编译器经
  CompilerResolver（PATH 探测 + 可 spawn --version 验证）。
- Cargo 族：spawn cargo build/run/clean（本机 rust 工具链）。
- Vue 族：pkg::run_script（npm/pnpm，见 pkg.rs PkgManagerKind 探测）——V-05 最小项目用本地
  node_modules fixture 或 npm 可用性检查，环境不足则该项 pending 显式记录（计划 AC-04 允许）。
- git：clone/checkout/pull（本地受控 git 裸仓 fixture，不触网）。
- **V-05 driver**：`crates/auto-man/tests/fixtures/plan809/` 下受控最小工程（C/ninja + rust/cargo +
  vue 三族 + local dep + port + scan 选择），由 plan809_management_core.rs 编排：构造→求值→闭包→
  规划→真实 builder 执行→产物断言（编译运行或等价可消费产物）；失败注入（工具失败/不可写输出/
  缺依赖）断言失败阶段与 exit。

### 10.3 环境矩阵与 pending 规则

| 工具 | 必需 | 缺失处理 |
|---|---|---|
| cargo/rustc | 是（本仓开发环境必有） | 无 |
| ninja | ninja 族用例 | 缺→该族真实执行用例 pending 显式记录，规划层断言仍跑 |
| node/npm | vue 族用例 | 同上 |
| git | local dep/git 用例 | 是（开发环境必有） |

## 11. 序与 oracle 策略

- 凡经 HashMap 迭代产生的顺序（resolve 后 targets 序、to_node 中 dirs 序、Index map 序）不作为
  字节稳定承诺；oracle 用集合比较或排序后比较。
- **既有输出文本已含此非确定性**（to_atom 的 targets 顺序），迁移不引入新的序承诺也不修复旧序
  （修复=行为变更，越权）。
- V-01/03/04 的"与冻结旧臂一致"具体化为：逐字段集合相等 + 已排序投影相等 + 错误文本对齐
  （错误文本本身不含 HashMap 序）。

## 12. T-01 未决项 → 任务映射

| 未决项 | 归属任务 |
|---|---|
| I-PROJECT 字段全集（生成器读点穷举） | T-02（本文件 §5 并入后冻结） |
| scanner.rs `scan_directory` TODO 空壳处置 | T-05（清偿：接入 core 选择规则或删除+登记） |
| 生成器项目读取改 I-PROJECT 的逐点接线 | T-06 |
| CompilerStore P805-D1 已知债 | 不动（越权；KNOWN-DEBT 已登记） |
| 模板先建后求值窗口（803 遗留） | 不动（计划 §1 明确不顺手修） |
| 三个 #[ignore] VM codegen 缺陷测试 | 随迁保持 ignore，不修 |
