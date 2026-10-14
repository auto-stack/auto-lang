# PLAN-780 T-01 — 受管 pac.at 盘点与 static feasibility 报告

基线：worktree `D:/autostack/.wt/lang-780/auto-lang`，branch `plan-780-dev`，
base `075a9b4ab6cb4284f3c259b348f2571322107fec`（= master tip）。
盘点工具：`pac_corpus_survey.py`（本目录，字符串/注释感知的形态扫描器；
扫描 oracle，不是解析器）。原始结果：`report.json` / `report.md`。

## 1. 语料分类（149 个受管 pac.at）

| 分类 | 数量 | 判定依据 |
|---|---|---|
| static | **144** | 仅字面量平铺 prop / 字面量节点（dep/app/lib/…）/ 字面量数组、对象；注释任意 |
| dynamic | **5** | 全部为 auto-man 脚手架模板（`crates/auto-man/assets/templates/{app,capp,clib,lib,libdemo}/pac.at`），含 `${...}` 插值（由 scaffold args 求值） |
| unknown | **0** | — |

- dynamic 指示符全语料仅出现 **interpolation** 一种（无裸变量引用、无赋值语句、无值内调用、无反引号 f-string）。
- dep 形态：语料只有**裸名形式** `dep <name> { ... }`（28 个文件，全部 `path:` 声明）。
  `dep "<name>"` 引号形式与 `dep("<name>")` arg 形式为语法支持面（PLAN-635 门/Plan 375 测试用），
  语料未用——冻结子集把它们归为"语法支持、语料未证实"。
- members 数组全部单行；`members` 无多行形态。
- 语料中值形态：引号字符串、十进制整数（`front_port: 4046`）、裸 `true`（`recurse: true`）、
  字符串数组、对象值（`theme: {}` 块、`back: { project }`、`npm_deps: { "@x/y": "link:..." }`
  ——含**带引号对象键** `@`/`/` 字符）、嵌套对象（a3ui-replica `targets.vue.frontend`）。
- ~~`#` 行注释形态在 pac.at 语料不存在~~ **勘误（T-03 对拍发现）**：examples/ui 全族 pac.at
  使用 `#` 行注释（PLAN-683 头注等，19 个文件）；.at 语义确认 `#` 非 `[` 后缀即行注释
  （parser.rs:8785，Plan 442 C2）。静态扫描器已按同规则支持。
- **顶层 JSON 式对象文档**（PLAN-729 minimal project format，`{ "scene": …, "members": […] }`）：
  examples/http_server/{files,uploads}/pac.at 两例——对象对成为 root props；survey 脚本
  未单列此形态（分类仍 static，正确），Rust 扫描器 T-03 补齐。
- **VM 对象字面量键迭代序跨进程不稳定**（T-03 三探针实测：同输入不同运行 repr 顺序不同）：
  任何对 pac 对象值 repr 的逐字比较都是在比噪声；对拍以结构（按名取键/排序键值对）为准。
  生产消费全部按键名读取，故此前从未暴露——已记入 T-04 owner 注记。

## 2. 读点分类（谁在读 pac.at、用什么机制）

### A. 执行型 VM 读取（dynamic-capable，oracle 旧臂）

| 读点 | 机制 | 字段 |
|---|---|---|
| `AutoConfig::from_code/from_file/read`（auto-lang config.rs:86） | `eval_config_with_vm`：Parser(Config 模式)→Codegen→链接→AutoVM 字节码执行（lib.rs:6297）；`on/off` 注入为 Bool 全局 | 全部 prop/kid 的求值结果 |
| `Pac::new(AutoConfig)`（auto-man pac.rs:196） | 从 VM 结果 Node 提取模型 | name/version/lang/backend/render(3形态)/api/scene/members/defines/front_port/back_port/media_root/photo_root/window(+fit)/title/title_zh/icon/category/theme(标量+块)/accent/desktop_render/ui_config/back(external_backend)/shadcn/default_classes/targets(app/lib/bag/**dep**/device/test 节点) |
| `external_backend_dir`（config.rs:330） | `AutoConfig::from_file` 探针（args port=win32） | `back.project` / 裸 `back` |
| `AtomReader::parse`（auto-lang atom.rs） | 编译器 Parser→Atom 树（非 pac.at 生产消费；测试/导出面） | — |

### B. 行级文本读取（static 文本扫描——待消除的"文本副本"）

| 读点 | 字段 | 备注 |
|---|---|---|
| `parse_pac_fields`（auto-lang ui/app_registry.rs:767） | 平铺 key:value（title/title_zh/name/icon/category/render/daemon/desktop_exe/desktop_render/window(fit)/opens/media_root/photo_root/desktop/back 截断） | `#` 视为注释；嵌套形态截断（`back` → `{ project`），由 parse_pac_back_project 补 |
| `parse_pac_back_project`（app_registry.rs:348） | `back: { project: "..." }` 单行嵌套 | strip_chain 容错 |
| `parse_pac_title` / `parse_pac_display_title[_in]` / `parse_pac_name` / `parse_pac_scalar`（auto-man vue.rs:2518–2558） | title/title_zh/name 任意标量 | |
| `parse_shadcn` / `parse_default_classes`（vue.rs:2583/2613） | shadcn/default_classes 开关 | 与 Pac::new 布尔语义对齐（注释明示） |
| `parse_npm_deps`（vue.rs:2659） | npm_deps（数组/对象/简写/多行缩进 4 形态） | **唯一消费者**（Pac 不建模 npm_deps） |
| `parse_pac_exe_name`（rust_ui.rs:3026） | exe_name | **唯一消费者**（Pac 无此字段） |
| `pac_declares_dep` / `pac_workspace_member_dir` / `pac_dep_version_violation`（auto-lang lib.rs:3241/3259/3283） | dep 裸名/引号两形态（词边界）、members 末段匹配、dep 块 version-plane 键守卫 | PLAN-635 声明门 + PLAN-647 Q5 |

### C. 其他平面（非 pac.at，盘点边界）

- `dep_scanner.rs`（auto-lang）：**源码内** `dep(...)` 语句扫描（Rust crate 依赖预注册），不是 pac.at 读取。
- `.am/pac.atom.at` 历史文件 `root { }` wrapper：Pac::new 有 hoist 兼容（pac.rs:199）——语料外形态，冻结子集之外。

## 3. 冻结的 static 最小子集（语料证实）

**字段（DTO 面向）**：

- identity：`name` `version` `lang`（字符串）
- render：`render`（single `"x"` / split `{front,back}` / multi `["a","b"]`）+ legacy `backend` 回退（字符串/数组/对象）
- 依赖：`dep <bare-name> { path: "..." }`（+引号/arg 语法形态，标注"语料未证实"）；dep 体 `path` 键（version-plane 键 = PLAN-647 违例面，模型侧按未知键编目并保留原文）
- 工程组织：`scene` `members`（字符串数组）
- 展示/桌面：`title` `title_zh` `icon` `category` `opens` `desktop` `desktop_render` `daemon` `desktop_exe` `theme`(标量) `accent` `ui_config` `window`(`WxH`/`fit`) `media_root` `photo_root` `front_port` `back_port` `api`
- 开关：`shadcn` `default_classes`（Bool / 裸 on/off / 引号 "off"——**VM 把裸 on/off 求值为 Bool，静态解析给 Str；Pac::new 归一化两者等价**，DTO 比较在归一化后进行）
- 外部后端：`back: { project: "..." }` 对象与裸字符串两形态
- npm 依赖：`npm_deps`（数组 / 引号键对象 / 嵌套对象 `{ link: }`）——编目入 raw props（Pac 不建模，vue.rs 文本读）；DTO 不做语义建模，只保证**不丢**

**语法面**：平铺 `key: value`（值 = 字符串/整数/裸 true/false/on/off/数组/对象）、
`//` 与 `/* */` 注释、`key: { ... }` 对象值（含带引号键）、裸名/引号/arg 节点头。

**显式 unsupported-static（解析器必须分类、不得静默丢弃）**：
`${...}`/`$ident` 插值、反引号 f-string、值位裸标识符（builtin 集合外）、
`x = ...` 赋值语句、值内调用 `ident(...)`、语料外 wrapper（`root { }`）。
出现任一 → 文件分类 dynamic/unsupported，**不得**计入对拍相等。

## 4. normalized oracle（对拍定义）

- 旧臂（oracle）：`AutoConfig::from_code(text, probe_args{port:"win32"}) → Pac::new → 抽取冻结字段归一化元组`。
- 新臂：`auto_project_model` 静态解析 → `ProjectDescriptor` → 同形归一化。
- 归一化口径跟 Pac::new 逐点对齐：字符串 trim；空串→None；`render`→BackendConfig 等价形态
  （枚举在 model 侧独立定义，比较按归一化字符串/结构）；`theme/accent/desktop_render` 小写化；
  开关按 `off/false/no/0` 集合；`window` WxH 数值边界与 `fit` 互斥；`back` 取 `project` 子键或裸串；
  `front_port/back_port` u16 解析失败→None；`scene` 枚举（未知→Default）。
- 比较域 = 冻结字段；`unsupported` 非空或解析错误 → 该文件排除出"相等"声明并显式计数（不假绿）。
- oracle 与 fixture 语料：149 个受管文件 + 专门反例（V-02/V-03/V-05 fixture）。

## 5. T02 依赖核定（plan §2 授权 T-01 决定）

- `auto-project-model` 依赖 = **auto-atom + auto-val**，无 serde（无协议需求）、无 auto-lang/auto-man/auto-vm/UI。
- `auto_atom::AtomParser` **复用于值层**（每个 prop 值文本是完整单 Atom 值，`AtomParser::parse` 可解）；
  **文档层不适用**——AtomParser 只接受单一顶层 Atom 值，pac.at 是平铺多入口文档（`key: value` 序列 +
  `dep name { }` 节点，`parse_node_body` 也不收 `dep name`/`dep "name"` 头形态）。文档级入口扫描器
  在 model crate 内实现（纯 Rust，零第三方依赖），产出 auto_val 值 + 逐字段 source 行号。
- 该核定即 plan §10"纯 parser 不足则显式保 adapter"的落地：值层复用 AtomParser、文档层自持最小扫描器，
  不另造完整 compiler，不把 AutoVM/Builder 类型带入 model。

## 6. 风险与非目标确认

- 5 个模板文件保持 dynamic 分类；本 Plan 不改模板、不切换任何生产读取（含 Pac::new/parse_pac_fields/
  parse_npm_deps/pac_declares_dep——全部原样保留）。
- `workspace/targets/scripts/description/frontend` 等键无生产读者（§2 核查）——编目为 raw props，
  不建模、不丢弃。
- 模板 eval 仍属 compiler adapter 面（§5 详细设计约束），model 小包的存在不声称 auto-man 摆脱语言依赖。
