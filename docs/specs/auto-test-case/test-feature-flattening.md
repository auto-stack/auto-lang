# 测试 feature 拍平与运行选择（Test-feature flattening）

> SD-01 新增合同（PLAN-901 / Design 52 计划B）。取代七个语言测试 feature 的
> 编译选择职责；运行范围由固定组、nextest 过滤与原 ignore/平台/环境条件决定。
> 分类：architecture ｜ 状态：active ｜ 来源计划：docs/plans/901-test-feature-flattening.md

## 1. 范围与不变量

- 七个语言测试 feature（`test-trans` / `test-cases` / `test-ace` / `test-book` /
  `test-aavm` / `test-http-e2e` / `test-case-batch`）自 `auto-lang` 与
  `auto-lang-core` 清单退役；测试代码在同一 Cargo 测试编译形态可见。
- `#[cfg(test)]`、平台条件、真实可选依赖与产品能力门（`code-editor`/`python`/
  `streaming-http`/`mem-profile`）与原 `#[ignore]` 语义全部保留；不得为
  "单一形态"隐式开启真实能力 feature 或批量加 ignore。
- `auto-test-case` 为两 crate 的常驻依赖；`test_cases` 模块与必要 API 常驻；
  三工具 bin（auto-case/ace-certify/test-case-pilot）恒为真实形态（无 feature
  缺席 stub），缺参/坏输入报错路径不变。
- trace 与批处理观察常驻编译但 env 驱动：无显式执行（`AUTO_TEST_TRACE_DIR` 等）
  时不 spawn worker、不写报告；普通语言返回值/诊断不变。

## 2. 运行选择与组边界

- 固定组职责与配置差异保持；去 feature 后等配置组仍由 driver 合并一次
  nextest 调度，不合并异配置组的资源语义。
- 旧靠 feature 缺席隐藏的测试族（转译/书稿/HTTP e2e/ACE 桥接/AAVM）常驻
  可见后，`daily`/`integration`/`auto-lang-core` 组以静态表达式边界钉住原
  选择集（逐 ID 实证零丢失零新增）；语料三族仍由 nextest default-filter
  排除出日常档（PLAN-700 语义）。
- AAVM/AA2R：默认 ignore + 显式 `cargo t aavm`/`--ignored` 手动入口、T3 条件
  与重内存分组（xl/lg/md + heavy_gate）全部保留；普通 VM/A2R 不误关。
- HTTP e2e：仍由 http 族固定组/原串行约束选择；跨 crate HTTP helper
  （spawn_outcome_reporter 等）为常驻中立支撑（`auto_lang::vm::ffi::http_server`
  经 auto-web re-export 链保持原路径）。
- ACE：语言 all 不隐式准备/构建 ACE worker；显式 `ace*` 组独占选择；ACE
  独立 workspace 内部 feature（如 `ace-av/test-case-batch`、conformance 自有
  `test-cases`/`test-case-batch`）与四 no_std 运行时 profile 不变。

## 3. 跨仓接收（B/D 边界）

- `auto-web`：域内 `test-http-e2e` feature 名保留（api-gen/Man/CI 入口），
  语言侧转发与 dev-dep feature 请求退役；helper 常驻消费。
- `auto-ui`：SDK 交付闭包含常驻 `auto-test-case` 与 `vendor/autodown-test-parser`
  （同形镜像、零 path 改写）；`PROJECTION_AUTO_LANG`/`PROJECTION_AUTO_LANG_CORE`
  收敛为恒等投影（旧 installed 中间形态的 test-cases 转发行不再被改写，重交付
  以 producer 清单整树覆盖 + receipt 钉树身份）。UI 自身 feature/后端拍平归 D。
  r2 闭包补全（890 谱系缺口修复）：`auto-generation-core` 与 web owner 家族
  `auto-web-contract`/`auto-web` 随包（`auto-ui-tools`/`auto-api-gen`/
  `auto-lang`/`auto-lang-core` 对其硬 path 依赖）；web 源经组内 auto-web
  兄弟或 `AUTO_WEB_SOURCE_ROOT` 解析；交付根 manifest 携带 web owner
  `[workspace.package]` 表与 vendored 快照树 exclude（嵌套 workspace 根
  不入交付工作区——交付消费侧 `cargo generate-lockfile`/`build --locked`
  实证）；7 个预存红交付测试转绿作修复反例对照。
- `auto-man`/`auto-api-gen`：Web 域同名 feature 转发链保持；无语言 feature 请求。
- D 不得为修 UI 构建恢复语言测试 feature。

## 4. 成本边界

- 测试 feature 导致编译形态分叉归零：正常库构建单元（同包/同 profile/同
  triple/同能力）不因测试选择新增同角色 rlib；test harness、dev 依赖与真实
  能力差异单元单列，不计入该断言。
- 日常 edit-rebuild 额外超过 120 秒（Design52 决策门）触发用户重议，不以
  回退/删测试凑门。
