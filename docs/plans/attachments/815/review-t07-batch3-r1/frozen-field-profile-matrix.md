# PLAN-815 T-07 子批3 — 字段 × profile × 操作 消费矩阵

2026-10-22，work 阶段交付（F-815-T07-B2-R1-03 关闭件）。对象 =
I-PROJECT 唯一请求类型 `GenerationRequest`（auto-project-model，
809 交付）在 `auto-ui-tools` 三操作（generate_ui / build_ui / run_ui）
七 profile（vue/rust/jet/ark/tauri/vscode/vm）上的**逐字段**适用性、
唯一源/锚定、消费点与证据对照。基线：唯一树（本批 docs 提交）+
code `e22fc7592` + 本批测试改动；证据为实测测试名/驱动收据，不以
聚合总数代替。

## 1. 语义总则（先明确既有语义——不为补表新增运行模型）

1. **操作选择 = 协议 op，不是 `request.mode`**：客户端发
   `generate_ui/build_ui/run_ui` 哪个 op 就走哪个臂（`tool_ops.rs`
   分派）；`request.mode` 是请求自述意图，唯一被消费的约束是
   validate 的 run-without-output-root 规则 + summary 回显。
2. **落盘布局不按 `output_root` 重定向（冻结语义）**：generate_ui 的
   实际落盘根 = project_root 布局——vue/jet/ark/vscode →
   `gen/front/<profile>`、tauri → `gen/front/vue` + `gen/back/rust`、
   rust → 共享 workspace 成员（`tool_ops.rs::profile_output_dirs`）；
   报告 `output_root` 如实回实际根。`output_root` 的消费面 =
   validate（run 前置 + 角色校验）+ 消费方交接元数据。
3. **相对路径锚定 = project_root（非工具 cwd）**：
   `GenerationRequest::resolve_against_root`（generation_request.rs:316，
   纯路径代数，无 FS 访问）；validate 对 input_sources/output_root/
   dep 路径全部经 `anchor()` 按请求根解析。
4. **请求在场即权威（无第二真源）**：`*_with_request` 家族生产入口
   消费已求值请求；请求未声明的字段=缺席，不落回磁盘 pac.at 重读
   （`PacSource` 请求臂模式，本批钉 external_backend/exe_name 两例；
   vue 子批2 F-B2-02 已闭 pac 存在性门残留）。
5. **携带字段（declared-only）不是缺陷**：809 的语义是"字段无损单源
   交接"——部分字段当前生产链无消费（下表"携带"标注），其值经
   validate summary/wire 往返保真；后续接线消费方时更新本表。

## 2. 逐字段矩阵

| 字段 | 适用操作/profile | 唯一源与锚定 | 消费点（代码） | 限定/不适用依据 | 证据 |
|---|---|---|---|---|---|
| `origin`（static/dynamic） | 全部（validate 诚实性） | 809 求值来源 | validate：static+unsupported → Warning `static-unsupported-constructs`；summary `origin`/`is_evaluated` 回显 | metadata-only：声明求值形态，不触发求值 | `validate_flags_static_unsupported_without_faking_evaluation`；`validate_ok_report_echoes_request_without_loss` |
| `project_root` | 全部工作操作（锚） | 请求携带；相对路径全按它解析（总则 3） | validate：非绝对/缺失/是文件 → Error；`tool_ops.rs::project_root`（工作操作要求绝对） | — | `validate_reports_missing_paths_and_run_constraint`；`validate_field_rules_output_root_and_input_sources_boundary` |
| `identity.name` | generate/build/run 全 profile | 请求快照（不重读盘） | vue：package.json 名；vscode：package.json（`vscode.rs:171-180`）；jet（`jet.rs:153-166` JetReads）；ark（`ark.rs:131-141` ArkReads）；rust：app 包名/bin 名；tauri：vue 臂 | — | 字段矩阵（vue，Unicode 名）；`generate_ui_vscode_fields_come_from_request_not_disk_pac`（vscode，盘名不同值区分）；792 driver（rust，uiminimal） |
| `identity.version/lang` | 携带 | 请求快照 | 无生成消费（交接保真） | 携带（总则 5） | wire 往返（project-model `wire_tests`） |
| `display.title/title_zh` | generate（vue）；携带于其余 | 请求快照 | vue index.html `<title>`（AUTO_LOCALE=zh 默认 → title_zh 优先） | zh 链仅 vue 产物面 | 字段矩阵（`中文标题示例`）+ lifecycle Unicode（dist 携带 zh 标题） |
| `display` 其余 10 子字段 | 携带 | 请求快照 | 无生成消费 | 携带 | validate summary 保真（echo 测试） |
| `runtime.window` | 携带（validate summary） | 请求快照 | summary `window` 回显（Size/Fit 归一） | 携带：生成产物不按它改视口 | `validate_ok_report_echoes_request_without_loss`（1280x720） |
| `runtime.{api,scene,front_port,back_port,media_root,photo_root}` | 携带 | 请求快照 | 无 generate/build/run 主链消费（media_root 有 legacy 文本读面 `vue.rs:8004`，非请求臂） | 携带 | wire 往返 |
| `scene` | 携带 | 请求快照 | 生成器不分支（workspace/ui 语义在 CLI 组织层） | 携带 | wire 往返 |
| `members` | 携带（rust dev 组织观察面） | 请求快照 | dev 工作区路由经 CLI 工程组织（V04 面） | 携带 | 792 driver V04（dev 模式 rust-workspace 路由不变） |
| `render.render`（single/multi/split） | **全操作的 profile 选择器** | 请求快照 | `frontend_names()`（generation_request.rs:273）→ 服务集 gate（generate：vue/rust/jet/ark/tauri/vscode；build：vue/rust/jet/ark/vscode——vm 解释无构建步；run：vue/vm/rust/jet/ark/tauri/vscode）+ 分派臂；`arkts→ark` 归一（`tool_ops.rs:21`） | 不服务 profile 在任何副作用前具名拒绝；multi=全目标按声明序生成（R4-07） | driver V05g（`plan9` 拒绝零产出 + 服务集原文回显）；R4-07 多目标全生成（T-05 R4 修复轮）；tauri 双输出 `aggregate_output_root` 单测 |
| `render.{legacy_backend,effective_backend}` | 携带 | 请求快照 | 无工具臂消费（809 回退链语义交接） | 携带 | wire 往返 |
| `external_backend` | run（vm/rust 会话） | 请求在场即权威（**本批钉**） | `PacSource::external_backend`（rust_ui.rs:3320，请求臂直供值/缺席，legacy 臂才读盘 `back: { project }`）；vm 会话：backend 根=project_root.join(值)，托管形态经 `AUTO_UI_TOOL_VM_BACK_ROOT` 每请求精确条目（run_vm.rs:373-380），CLI 形态 set_external_back_root+cdylib | generate/build 不消费 | **新** `external_backend_request_is_authoritative_over_disk_decl`；`vm_session_env_is_replaced_per_request`（proc_host R4-01）；T-05 R3-01/R4-01 修复轮证据 |
| `toggles.shadcn` | generate（vue） | 请求快照（归一 `shadcn()`） | vue 生成 shadcn 组件检测/物化门 | off/_unset 语义冻结 | 字段矩阵（物化面）；`temp_project` 语料 `shadcn: off` |
| `toggles.default_classes` | 携带 | 请求快照 | 无生成消费（类面归一在 809） | 携带 | wire 往返 |
| `theme_decl` | generate（vue）；run（vm 会话主题） | 请求快照 | vue：`__AUTO_COMPOSED_THEME__`+主题名进 index.html；vm：`AUTO_UI_TOOL_VM_THEME_DECL` 会话 env（run_vm.rs:386-397） | — | 字段矩阵（theme{} 三断言） |
| `dep_roots` | generate（vue dep 组件）；validate advisory | 请求快照；path 相对 project_root | vue：dep 根 front 编入（MatrixDepWidget）；validate：根存在性 Warning | — | 字段矩阵（dep 节点解析+产物断言——复审确认非漏项） |
| `package_assets.npm_deps/styles/i18n` | generate（vue） | 请求快照；相对路径按请求根 | package.json 依赖合并 / styles 逐字节拷贝+main.ts 引用 / locale 拷贝+vue-i18n 实例 | — | 字段矩阵（dayjs^/index.css/zh.json 三断言） |
| `exe_name` | generate/build（rust [[bin]] 命名） | 请求在场即权威（**本批钉**） | `PacSource::exe_name`（rust_ui.rs:3250；legacy 臂 `parse_pac_exe_name` 读盘） | 合法字符 [A-Za-z0-9_-]（盘侧校验） | **新** 同款权威测试（req-bin/disk-bin 区分）；792 driver V01（app 二进制名） |
| `has_rust_sidecar` | 携带（形态信号）；载荷在 `raw_nodes` | 请求快照（raw_nodes 检测置位，generation_request.rs:169） | 布尔本身无独立生成消费；sidecar deps 载荷逐字随 `raw_nodes`，交付面 TOML 语义校验消费（外部/根相对路径拒绝） | 冻结语义：为 816/CLI 消费方提供"有无 sidecar"信号而不必解析 raw_nodes | 792 driver V03f/V03g/V03i（sidecar 三负控）；wire 往返（project-model wire_tests:135） |
| `front_decl`（prop 形态） | generate/build/run（前端目录解析） | 请求快照 | `util::request_front_dir`（**节点形式优先**，R2-F03；无声明回退布局探测链 src/front→source/front→front） | — | `generate_ui_vscode_fields_come_from_request_not_disk_pac`（prop `./src/front`）；`temp_project` 语料 |
| `front_node_form`（节点形态） | 同上，**优先于 prop** | 请求快照（wire 携带） | 同 `request_front_dir`；rust run 的 front 存在性 gate（tool_ops.rs:260-270） | — | 字段矩阵（`./ui/front` 节点形态=编译源）；T-06 R1-01（rust front 贯穿） |
| `input_sources` | validate（求值输入清单） | 请求快照；相对路径按 project_root（非 cwd） | validate：缺失→Error `input-source-missing`；是目录→Error `input-source-is-directory`；无根相对→Warning | metadata-only：validate 不求值（诚实边界）；动态 adapter 是未来消费方 | **新** boundary 测试（正负控+锚定）；`validate_reports_missing_paths_and_run_constraint` |
| `output_root` | validate（run 前置+角色）；交接元数据 | 请求快照；相对路径按 project_root | validate：run 模式无 output_root→Error `run-without-output-root`；是文件→Error `output-root-is-file`；summary `has_output_root` | **generate 布局不重定向**（总则 2，本批钉） | **新** boundary 测试 + 字段矩阵扩展（declared root 不被创建、报告根=实际布局根） |
| `mode`（generate/run） | validate 约束 + summary | 请求快照 | validate run-without-output-root 规则；summary `mode` 回显 | 操作臂=协议 op（总则 1） | `validate_reports_missing_paths_and_run_constraint`；**新** boundary 测试（run 正控：output_root 在位 → 该 finding 恰缺席） |
| `raw` / `raw_nodes` | 全部（保真交接） | 请求快照（逐字） | wire 往返无损；summary 计数（raw_props/raw_nodes）；sidecar/未建模节点载荷 | 未知键不静默丢（809 契约） | project-model wire_tests；validate echo 测试 |

## 3. 每 profile 代表性生成/消费证据（非全笛卡尔积，按支持合同选代表）

| profile | 代表证据（真实入口） |
|---|---|
| vue | 字段矩阵重测试（全字段→产物逐字段落点+删 pac 逐字节一致，本批扩展 output_root 钉）；lifecycle Unicode/空格真实 pnpm build（4/4） |
| vscode | `generate_ui_vscode_fields_come_from_request_not_disk_pac`（请求名 vs 盘名区分） |
| rust | 792 driver 全程（fresh/locked/重定位真实构建+V04 dev 同源） |
| tauri | tauri 双输出（vue 前 + gen/back/rust 后端）`aggregate_output_root` 单测 + 服务集矩阵（V05g 回显） |
| jet / ark | 服务集 gate（V05g 原文回显 in this build）+ JetReads/ArkReads::from_request 消费点（§2 identity/front 行）；run 面启动链 T-05 R7-R8 已闭 |
| vm（run only） | `vm_session_env_is_replaced_per_request` + T-05 R5/R8 Unix/Windows 会话面；本批 external_backend 权威钉 |

## 4. 本批新增区分性测试

| 测试 | 区分性 oracle |
|---|---|
| `validate_field_rules_output_root_and_input_sources_boundary`（tool_process，真实工具进程） | run 正控（output_root 在位→finding 恰缺席）/ output-root-is-file（Error）/ input-source-is-directory / 相对 input_source 按请求根命中工程内真实文件（cwd 侧不存在——锚定区分） |
| `external_backend_request_is_authoritative_over_disk_decl`（rust_ui lib） | 请求值 vs 盘值不同（request-back/disk-back）：请求在场=请求值、请求缺席=无（不落盘读）；legacy 臂读盘；exe_name 同款三态（req-bin/disk-bin） |
| 字段矩阵测试扩展 | output_root 声明根不被创建 + 报告根=实际布局根（布局不重定向钉） |

## 5. 覆盖边界（如实）

- jet/ark 的生成级逐字段断言未逐项展开（消费面=name+front 目录，§2 已
  钉消费点+服务集 gate）；build/run 面的外部工具链依赖（gradle/hvigor/
  设备）按支持合同不假装本机具备——run 面启动链已由 T-05 R7/R8 与
  T-06 R1/R2 轮闭。
- 携带字段（§1 总则 5）清单随字段矩阵表"携带"标注为准；后续接线新消费
  须同步本表与证据。
- `runtime.media_root` 等 legacy 文本读面（非请求臂）不属本矩阵消费面，
  归 809/legacy 兼容层。
