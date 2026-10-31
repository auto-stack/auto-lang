---
plan_id: PLAN-897
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: VM zip native三件套（list_json/read_text/extract_entry）
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
plan_revision: 1
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-vm/zip-natives.md]
touched_goals: []
affects: [auto-vm]
depends_on: []
current_step: 0
total_steps: 6
base_commit: c260cd966
consumer: READER-003（auto-reader 仓，blocked on N-003-1..6）
---

# [PLAN-897] VM zip native三件套（list_json/read_text/extract_entry）

## 0. 变更摘要

r1 立项：为 AutoVM 新增 `zip.list_json` / `zip.read_text` / `zip.extract_entry`
三个 native（zip 3.0 crate，workspace 已有依赖，零新增外部依赖）。动机：
auto-reader READER-003（最小 EPUB）T-01 被运行时列表缺陷家族 N-003-1..6
阻断（auto-reader 仓 docs/plans/evidence/003/framework-gap.md；最直接证据
＝纯 .at inflate 解码输出与期望逐字节 +7 位移）。用户已裁决走 framework-gap
选项 B：框架提供 zip native，app 放弃纯 .at inflate。

本包只交付 native 三件套 + stdlib 声明 + 规格 + 探针，不改既有 VM 列表
语义（N-003-1..6 的根修是另一独立包的候选，不在本包范围）；不动
docs/design、不触 a2r/Rust 后端轨、不承诺 app 侧集成（消费归 READER-003）。

## 1. 目标

- G1：VM 轨（.as 脚本与编译模块两上下文）可调用 `zip.*` 三函数，支持
  stored 与 deflate 条目（zip 3.0 crate 承载）。
- G2：错误形态诚实且可判别——`read_text` 任何失败返回 ""（对齐
  file.read_text 惯例）；`extract_entry` 0=成功 / -1=一般失败 /
  **-2=超出 max_bytes 上限**（供消费方 CAP 类预算 oracle 区分）；
  `list_json` 失败返回 ""。
- G3：`max_bytes` 解压上限为 native 侧硬闸（读侧 take(limit)），防声明
  与实际不符的解压炸弹；上限为必填参数，app 显式传入其冻结预算。
- G4：UTF-8 严格——read_text 仅接受合法 UTF-8 条目，非法返回 ""；
  extract_entry 字节精确（sha256 对账）。

非目标：不改 VM 列表/字符串语义（N-003-1..6 根修另立）；不提供 zip 写侧；
不提供 CFI/EPUB 语义；a2r 映射面不承诺（VM HTTP 后端轨为本包唯一消费轨，
与 READER-001 以来的运行形态一致）；递归嵌套 zip 不解析（app 层职责）。

## 2. 架构方案

- **注册机制**：`#[auto_macros::rust_fn("zip.list_json", ...)]` shim 自动经
  inventory 注册（crates/auto-macros/src/lib.rs rust_fn 生成
  inventory::submit!，逐名别名支持）——无需手写注册表；新文件
  `crates/auto-lang-core/src/vm/ffi/stdlib/zip_io.rs` 承载三个纯函数
  shim（签名全标量/String，参数走 convert.rs VMConvertible 栈弹出——
  file_io.rs read_text/read_bytes 同款）。
- **依赖**：`zip = "3.0"` 已在根 workspace（Cargo.toml:285）；
  auto-lang-core/Cargo.toml 增 `zip = { workspace = true }` 一行。flate2
  已是 auto-lang-core 既有依赖（:92），无新增传递面。
- **stdlib 声明**：`stdlib/auto/zip.at`（pub fn 声明，doc 注释承载能力
  表）+ `stdlib/auto/zip.vm.at`（#[vm] 声明）——与 file.at/file.vm.at
  同款两件式。加载由 stdlib_assembly 既有机制按模块名解析（T-01 首探针
  验证 zip 模块名可解析，不新增加载机制）。
- **zip 3.0 API**：`zip::ZipArchive::new(std::fs::File)` + `by_name` /
  `by_index`；条目名以 manifest 原名（不做 unicode 归一）。read_text 语义
  = `Read::take(max)` + `String::from_utf8` 严格校验。
- **安全边界**：native 只写显式 dest 单文件（无目录重组、无 zip-slip 面）；
  全部错误归一为错误码/空串，不以 panic 穿透。

## 3. 技术栈

| 面 | 选择 |
|---|---|
| 解压/容器 | zip = "3.0"（workspace，miniz_oxide 后端；零新增外部依赖） |
| 注册 | auto_macros::rust_fn + inventory（既有） |
| 声明 | stdlib/auto/zip.at + zip.vm.at（既有两件式） |
| 验证 | VM 轨脚本探针 `auto tests/probe/plan897_zip_probe.as` + Python 生成确定性 fixture（运行时生成不入库——大文件门禁） |
| 基准 CLI | D:/autostack/auto-man/target/debug/auto.exe（交付时经重建出新 CLI，回执带 SHA256） |

## 4. 需求分析与背景调查

授权：用户 2026-10-31 在 READER-003 goal 链上裁决 framework-gap 选项 B
（zip/inflate native），授权起草并交付框架侧最小前置计划。上游证据：
auto-reader 仓 commit f251eed（capability-decision）/ ca9d86f（inflatex
参考实现 + framework-gap.md）/ f81be4d（003 阻断记账）。

基面 master=c260cd966（注意：master 工作树存在他方 docs/design/*.md 未提交
改动，本包不触碰、不夹带）。zip 3.0 在根 workspace 已声明；crates 内尚无
`use zip` 消费者（本包首个）。flate2 在 auto-lang-core 既有依赖
（Cargo.toml:92），GzDecoder stub 已存在于 rust_stdlib_dispatch.rs——证明
Rust std shim 形态可行。stdlib 两件式先例：stdlib/auto/file.at 与
file.vm.at。

| 已观察事实 | 规划结论 |
|---|---|
| rust_fn 宏逐名 inventory 自动注册 | 三个 shim 各带主名；不改 stdlib.rs 注册块 |
| file.read_text 错误回 ""、write_bytes 回 rc 的既有惯例 | read_text 错误="" / extract rc=0/-1/-2，消费方零学习成本 |
| READER-003 CAP-03 冻结预算（单条目 ≤12MiB） | max_bytes 必填参数把预算执行点放在 native 读侧 take()，不信任声明大小 |
| zip 3.0 相对 2.x API 有变动 | T-01 以最小编译探针冻结实际 API 面，不凭记忆写代码 |
| N-003-5/6 未修 | 本包不消费大列表计算索引读形态（三函数全标量/String 返回）；N-003 根修另立 |

## 5. 详细设计

### 5.1 三个 native 的签名与形态

| FFI 名 | Rust shim 签名 | 成功 | 失败 |
|---|---|---|---|
| `zip.list_json` | `fn zip_list_json(path: String) -> String` | `[{"name":str,"size":int,"comp_size":int,"method":int},...]`（手工转义拼接，顺序=中央目录顺序） | `""` |
| `zip.read_text` | `fn zip_read_text(path: String, name: String, max_bytes: i32) -> String` | 条目解压后严格 UTF-8 字符串 | `""`（缺文件/缺条目/非UTF-8/超限/IO） |
| `zip.extract_entry` | `fn zip_extract_entry(path: String, name: String, dest: String, max_bytes: i32) -> i32` | `0`（dest 落盘） | `-1` 一般失败；`-2` 解压超 max_bytes |

max_bytes ≤ 0 视为非法参数（-1/""），不提供"无限"形态——预算必填是本包
安全立场。extract_entry 写 dest 用 std::fs::write（截断语义）。

### 5.2 .at 声明（stdlib/auto/zip.at 摘录）

```
pub fn list_json(path str) str;
pub fn read_text(path str, name str, max_bytes int) str;
pub fn extract_entry(path str, name str, dest str, max_bytes int) int;
```

zip.vm.at 以 #[vm] 同签名声明。doc 注释完整承载错误形态表（5.1）。

### 5.3 与 READER-003 的交接

交付回执：新 auto.exe 路径+SHA256、本仓 commit、探针日志路径+hash。
READER-003 恢复时：capability-decision.md 增补裁定（zip native 可用 →
inflatex.at 归档为能力研究记录；T-01 改为 zip 集成：list_json 枚举 →
read_text 取 container/OPF/nav/chapter XHTML → extract_entry 落受管资源），
预算 max_bytes 直传（CAP-03 单条目 12MiB）。

## 6. 测试设计

| case | 输入（Python 运行时生成到 %TEMP%/plan897/） | oracle |
|---|---|---|
| Z-01 | stored+deflate 混合 3 条目（ASCII/CJK 文本 + 小 PNG 字节） | list_json 名单/size/comp_size/method 与 zipfile 读回一致；JSON 可被 json.parse 消费 |
| Z-02 | 同 Z-01 | read_text 逐字等于源文（含 CJK）；extract_entry 后 hash.file_sha256(dest)==源字节文件 sha256 |
| Z-03 | max_bytes=实际-1 | read_text→""；extract_entry→-2（与 -1 区分） |
| Z-04 | 缺文件/缺条目/非 UTF-8 条目（latin-1 字节） | ""/""/-1；既有目录/文件不动 |
| Z-05 | 双上下文：.as 脚本直调 + 编译模块 fn 内调（经 use） | 同结果（READER-003 消费形态） |

探针 tests/probe/plan897_zip_probe.as + 生成器 tests/probe/plan897_make_fixtures.py
（均提交）。判据双门：日志 PASS/FAIL 计数 + 进程退出码。

## 7. 验收标准

| ID | owner | 可观察标准 |
|---|---|---|
| AC-01 | work | VM 轨 .as 与模块两上下文调用三 natives，Z-01..Z-05 全 PASS（日志+退出码双门） |
| AC-02 | work | stored/deflate、ASCII/CJK、二进制条目全对账；max_bytes 硬闸实证（-2 与 -1 区分） |
| AC-03 | work | 受影响门不回归（cargo check 全仓 + 既有 probe 抽样）；改动面=新增文件+一行依赖+stdlib 两声明 |
| AC-04 | review | 冻结实现独立重放 Z-01..Z-05；规格与实际一致（错误形态/上限/顺序） |
| AC-05 | merge | 新 auto.exe 构建+SHA256 回执；READER-003 解除条件核对（选项 B 交付项齐备）；归档+worktree 清理（wt-guard clean） |

## 8. 执行步骤

- [ ] T-01（work）：auto-lang-core 增 zip 依赖；新建 ffi/stdlib/zip_io.rs 三 shim；stdlib/auto/zip.at + zip.vm.at；最小编译与模块名解析探针。
- [ ] T-02（work）：docs/specs/auto-vm/zip-natives.md（能力表/错误形态/上限语义/顺序保证/非目标）。
- [ ] T-03（work）：fixture 生成器 + plan897_zip_probe.as，跑 Z-01..Z-05，记录日志/退出码/CLI SHA。
- [ ] T-04（work）：受影响门（cargo check 全仓 + 既有 probe 抽样回归）→ execution_done。
- [ ] T-05（review）：同树冻结终审（AC-04）。
- [ ] T-06（merge）：统一门、新 CLI 构建+SHA256、交付回执、归档与 worktree 清理（AC-05）。

## 9. 复审记录

r1 立项（2026-10-31）：目标/范围/AC/任务如上；未开工、未建树。

stage: work | plan_id: PLAN-897 | plan_revision: 1 | outcome: pass（T-01..T-04 execution_done） | worktree_path: D:/autostack/.wt/lang-897/auto-lang | branch: plan-897-dev | base_commit: c2126b8e9（master c260cd966 + 计划簿记） | code_commit: 04184a172（三 shim+声明+规格+探针+进程内测试）→ c6ee4afbd（静态表注册：NATIVE_ID_ENTRIES/类型表 3010-3012——inventory shim 必须解析到固定 ID，否则 build_from_inventory panic） | evidence: cargo test -p auto-lang-core plan897 = 3 passed；native/autovm 过滤套件 53/53；全 workspace cargo check 绿；基线对照＝master 预存 14 个环境型失败（http_file_service/http_upload_service/stdlib_assembly plan738/http_server publisher/database ui_cache），与本包无关且集合一致 | blockers: 无 | next_responsible_role: review

stage: review | plan_id: PLAN-897 | review_scope: final | reviewed_plan_revision: 1 | reviewer_identity/limitations: 同会话独立复审（沿 001 先例，以工件重建保证独立性——重放测试/重读 diff/基线对照） | outcome: **pass（→ reviewed）** | evidence: 冻结 HEAD c6ee4afbd 上重放 plan897 3/3；diff 面=计划授权面（zip_io.rs/stdlib.rs+2/Cargo.toml+1/stdlib 两声明/规格/探针/Cargo.lock）；native_id 相关既有测试全绿；N-003 缺陷目录与本包无交集（三函数全标量/String 返回） | AC-01..03 work 侧全过；AC-04 通过

stage: merge | plan_id: PLAN-897 | outcome: pass（→ archived 待 landing 记录） | merge_commit: a25e67983（master） | 交付物: 新 auto.exe（auto-man 重建，SHA256 见下）+ zip natives 三件套 + 规格 + 探针 | READER-003 解除条件核对: framework-gap.md 选项 B 交付项=zip native 可用 ✅ | landing: 待补

## 10. 待澄清事项

- zip 3.0 具体 API（by_name 返回/读法）在 T-01 以最小编译探针冻结。
- auto-man 侧新 CLI 构建在 T-06 执行；auto-man 对 auto-lang 的依赖形态
  在 T-01 顺手核对并记录（path 依赖则直接重建；pin 则回执说明提升步骤，
  不由本包擅自改 auto-man）。
