# PLAN-780 T-04 — pac.at 字段 owner 对照表与文本副本消除接线决策

依据：T-01 读点三类清单（survey/T01-REPORT.md §2）+ T-03 对拍实况（144 static 相等证明
model 归一化与 Pac 逐点一致）。本表是 SD-01 spec 沉淀的素材面；**当前零生产切换**。

## 1. 字段 owner 对照表

「目标 owner」列指 static 默认切换后的单一事实源；「文本副本」列是当前行级文本读取点
（T-01 §2-B 族），切换后可删除。

| 字段/形态 | 当前读者（机制） | 文本副本（待消除） | 目标 owner | 接线注记 |
|---|---|---|---|---|
| name / version / lang | Pac::new（VM）；registry；vue parse_pac_name | vue `parse_pac_name`、registry `parse_pac_fields(name)` | `Identity` | 归一化已对拍相等 |
| render 三形态 + legacy backend + lang 推断 | Pac::new（VM `BackendConfig::from_value`） | —（无文本副本） | `RenderDecl` | rust-tauri 等 Unknown 形态 repr 由 effective 字符串承载，与 Pac 一致 |
| api / scene / members / defines | Pac::new（VM） | — | `RuntimeFields`/`members`/`defines` | members 非数组时 Pac 静默空 vec，model 分类 TypeMismatch——切换后行为收紧需在切换计划声明 |
| dep 声明（裸/引号/arg + path） | Pac::new targets（VM）；**声明门** `pac_declares_dep`（文本，词边界） | lib.rs `pac_declares_dep` | `deps`（声明面） | 声明门换 `deps` 名单时保留词边界反例（settings vs settings_v2） |
| members 门控 | `pac_workspace_member_dir`（文本，末段匹配） | 同左 | `members` | 末段匹配语义保留 |
| dep version-plane 守卫（PLAN-647 Q5） | `pac_dep_version_violation`（文本） | 同左 | `deps[].props`（原文保留） | 守卫改读 props，键集 {version,pin,rev,tag,branch,commit} 不变 |
| title / title_zh / icon / category | Pac::new（VM）；registry；vue `parse_pac_title/display_title` | vue `parse_pac_scalar` 族、registry | `DisplayFields` | **归一化分歧在案**：Pac 对 theme/accent/desktop_render 小写、registry 不小写——统一到 model（Pac 对齐）已在 DTO 落地 |
| theme 标量 + theme{} 块 | Pac::new（VM；块形态经 design_tokens compose） | — | 标量=`DisplayFields.theme`；块=raw_props 编目 | 块形态 compose 语义属展示域，DTO 化需后续计划（词表校验在 compose 层） |
| opens | registry `parse_pac_fields`（文本） | registry | `DisplayFields.opens`（raw 串；normalize_opens 留在消费点） | — |
| window WxH / fit | Pac::new（VM）；registry fit 推断（文本） | registry `window` 键 | `RuntimeFields.window` | 边界 200..=7680/4320 与 fit 互斥已镜像 |
| daemon / desktop / desktop_exe | registry（文本）；desktop_exe 另有 rust_ui `parse_pac_exe_name`（文本，唯一消费者） | registry、rust_ui | `DisplayFields` | exe_name 合法字符校验留 rust_ui 消费点 |
| media_root / photo_root | Pac::new（VM）；registry（文本） | registry | `RuntimeFields` | env 覆盖序留运行时消费点 |
| front_port / back_port | Pac::new（VM） | — | `RuntimeFields` | — |
| ui_config / entry | Pac::new（VM） | — | `DisplayFields.ui_config` | entry 属 target 域，raw 编目 |
| back.project（外部后端） | config.rs `external_backend_dir`（VM 探针）；registry `parse_pac_back_project`（单行嵌套文本） | registry | `external_backend` + `SourceInfo` 基点 | V-04 已对拍：基点=pac.at 所在目录，两臂解析一致 |
| shadcn / default_classes | Pac::new（VM）；vue `parse_shadcn`/`parse_default_classes`（文本，语义已对齐） | vue 双函数 | `ToggleFields`（normalized） | 三形态（Bool/on-off/引号串）归一已对拍 |
| npm_deps（数组/对象/简写/多行缩进） | vue `parse_npm_deps`（文本，唯一消费者） | vue（暂留） | raw_props 编目（暂不 DTO 化） | 形态宽于冻结子集（嵌套 `{link:}`、缩进续行）；DTO 化随切换计划专门立项 |
| targets（app/lib/bag/device/test 节点） | Pac::new → `Target::from`（VM，构建域物化） | — | 暂留 Pac | **红线**：Builder/物化不进轻 model（plan §2：不把整个 Pac 搬成新包） |
| workspace / targets / scripts / description / frontend 等无读者键 | 无生产读者 | — | raw_props 编目 | 迁移期保持编目不消费（T-01 §2 核查） |

## 2. 消除文本副本的接线决策

1. **auto-man 内部先行（无新依赖边）**：vue.rs 的 `parse_pac_title/parse_pac_name/
   parse_pac_scalar/parse_pac_shadcn/parse_pac_default_classes/parse_pac_display_title`
   与 rust_ui 的 `parse_pac_exe_name` 逐函数换 `auto_project_model::parse_file`
   （本计划已接依赖）；每函数替换保留其现有单测作回归面。`parse_npm_deps` 因形态宽于
   冻结子集**暂留文本**，随专门计划 DTO 化后替换。
2. **auto-lang 侧需依赖裁定（后行）**：registry `parse_pac_fields`/`parse_pac_back_project`
   与 loader 声明门三函数（`pac_declares_dep`/`pac_workspace_member_dir`/
   `pac_dep_version_violation`）换 model 需要 auto-lang → auto-project-model 依赖边
   （model 只依赖 atom/val，无环，但 auto-lang 依赖面扩大须立项裁定）——**本计划不做**。
3. **VM adapter 永久保留面**：脚手架模板渲染（asset.rs `eval_template` 管线）、任何
   `unsupported` 非空文件、`on/off` 求值语义。显式 adapter，禁静默 fallback（V-03 已钉）。
4. **切换判定门**：static 默认切换的前置=受管语料 dynamic 占比（现 5/149）与模板管线
   改造方案；属后续决策（plan §0「不切换所有生产pac读取」）。
5. **发现注记**：VM 对象字面量键迭代序跨进程不稳定（T-03 探针）——任何未来消费者不得
   依赖 pac 对象值的 repr 顺序，按键名读取（现状消费全部如此，无存量风险）。
