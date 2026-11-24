---
plan_id: PLAN-1124
status: executing
feature_name: "AV 标准库动态 JSON、泛型编码与 CSV 迁移"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-24
plan_revision: 1
current_step: 0
total_steps: 6
depends_on: [PLAN-966, PLAN-975, PLAN-1064, PLAN-1088, PLAN-1096]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-data.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [stdlib, auto-av, auto-semantics]
---

# [PLAN-1124] AV 标准库动态 JSON、泛型编码与 CSV 迁移

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](archive/1096-av-parity-full-language-certification.md)。

## 1. 目标

补齐 Windows AV 普通程序里的动态 JSON 树及常用访问/格式化 API、已准入记录的泛型 JSON 编码，以及旧 CSV 四个公开操作的迁移。

## 2. 架构方案

公共 Auto `std.json`、`std.csv`（拟） → 共享 Auto 算法与受检布局 → AV。重用 975 的语法验证和标量/字符串处理。动态树存储采用现有合适的受检值或有序项序列。泛型编码先核 966 字段事实和 975 的显式 schema/spec 接口。

## 3. 技术栈

`stdlib/ace/std/json.at`、`stdlib/auto/{json.at,encoding/csv.at}`、已验 975 数据组、普通工程数据语料。

拟实施树为 `D:/autostack/.wt/lang-1124/auto-lang`，分支 `plan-1124-dev`。

## 4. 需求分析与背景调查

[975 数据规范](../specs/ace-migration/m2-stdlib-data.md)已验 JSON 语法、路径取值、原语构造和 `decode<T>` 切片，并明确保留完整 `JsonValue`、泛型 encode 和 CSV 差额。旧 [CSV 公共声明](../../stdlib/auto/encoding/csv.at)有默认和自定义分隔符的解析/编码四形。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

**执行授权（2026-11-23，auto-plan-goal 范式）**：用户 `/goal 完成计划1123和1124` = work + 独立复审 + 合同内有界修复 + merge 的整链授权（目标/验收/阈值/兼容承诺/仓库与动作范围/预算不变；PLAN-1123 已交付归档 `4a2054466`/`1f5c88f03`，本会话只验证不改写）。实施树 `D:/autostack/.wt/lang-1124/auto-lang`（分支 `plan-1124-dev`，基面 master `8a52d7364`；同组 `auto-down` 兄弟 detached `456c86a` 只读消费）；材料区 `D:/autostack/plan-artifacts/plan-1124/`（已登记 `.retention/policy.json`）。主检出预检：代码路径零 WIP，他方文档/计划改动保持不触碰。

### T-01 冻结记录（2026-11-23，worktree `plan-1124-dev`）

- **D-1124-T01-1 模块来源与 975 兼容**：`stdlib/ace/std/json.at` 保持逐字节不变（供 975/1064/AGE `Faces::Plain` 消费）；普通 Auto AV 侧增加 `stdlib/ace/std/json_auto.at`（与 `json.at` 合并构成普通 `Faces::Auto` 的 `std::json` 与单文件嫁接体）及 `stdlib/ace/std/csv.at`（构成 `std::csv` 与单文件嫁接体）。`Faces::Auto` 包导入支持 `use std.json.*` / `use std.json: ...` / `use std.json` 与 `use std.csv.*` / `use std.csv: ...` / `use std.csv`。
- **D-1124-T01-2 动态 JSON 数据模型与访问边界**：`JsonValue` 采用自包含平坦 LCRS（left-child/right-sibling）节点池（`tags`/`ivals`/`fvals`/`svals`/`kvals`/`first_child`/`next_sibling`），迭代式解析与子树复制（无调用栈递归，子值独立拥有并可单独释放）。键序保留插入顺序；对象重复键采 **last-member-wins**（原地更新首次键槽的值，不产生重复键）。数字按 RFC-8259 严格校验，存 `f64` 并记录精确 `i32` 标量标志；`.as_number() -> f64`；`.as_int() -> int` 仅接受可精确表示的 `i32` 整数（小数或越界抛 `TypeError`）；非有限浮点（NaN/±Inf）序列化抛 `JsonError`；补齐 AV `MirInstr::TextDisplay` 对 `Value::F64`（复用 `fmt_console_f64`）与 `f64.to_str()` 支持。深度上限 128、输入上限 1 MiB、节点上限 65,536。`parse` 对坏输入/超限返回 `None`，`parse_strict`/`minify`/`prettify` 抛 `JsonError`，`is_valid` 返回 `false`。`.get`/`.get_at` 缺键或越界返回 `None`（与 `Some(null)` 明确区分），`.as_string`/`.as_number`/`.as_int`/`.as_bool`/`.as_array`/`.keys`/`.len` 类型不符抛 `TypeError`。因 `type` 在 `auto-source.at` 为普通面保留词，动态类型查询定名为 `.kind()` / `.type_of()`（与旧 `Json.type_of` 一致）。
- **D-1124-T01-3 泛型 JSON 编码合同**：沿 975/1088 显式 `spec JsonEncodable { fn json_encode() Str }` 与 `encode<T has JsonEncodable>(value T) Str`、`encode_pretty<T has JsonEncodable>(value T) Str`；未声明遵从 `JsonEncodable` 的记录由 `verify_auto_bound` 具名拒绝（`source.instances-bound`）。当普通单文件或跨模块记录声明 `as JsonEncodable` 且未手写覆盖 `json_encode` 时，编译器在 `instantiate_auto_program_scoped` 基于受检 AST `TypeItem.fields`（声明序字段名与类型）自动合成 `fn json_encode() Str`；首批字段类型覆盖 `int/i32/i8/u8/u32/i64/u64/uint`、`f64/f32/float/double`、`bool`、`Str/str`、`?T`、`List<T>` 及嵌套 `JsonEncodable` 记录（带环检查），不支持的字段类型或非 `JsonEncodable` 嵌套记录在绑定期具名拒绝（`source.json-encode-unsupported`）。
- **D-1124-T01-4 CSV 格式合同**：保留 `stdlib/auto/encoding/csv.at` 四个公开名：`csv_parse(text str) Str`、`csv_parse_with_delimiter(text str, delim str) Str`、`csv_encode(data str) Str`、`csv_encode_with_delimiter(data str, delim str) Str`（另可附二维数组 helper）。按 RFC-4180 处理引号包裹分隔符/换行/转义 `""`、合法空字段（`a,,c`、`a,`、`""`）与 `\r\n`/`\n` 行尾；空串返 `"[]"`；未闭合引号/非引号内裸引号/闭引号后杂字符抛 `CsvError`；非法分隔符（非单标量、引号或换行）抛 `CsvError`；预算（1 MiB / 65,536 单元格）超限抛 `CsvError`；错误经 `try/catch` 捕获后零 arena 泄漏。

**复审 r1 后的冻结补全（2026-11-24，work 修复轮）**。复审 F-01..F-08 表明上面四条记录留有未定义或与实现不符之处；以下五条把它们补全，与上面记录冲突时以本节为准。目标、验收标准与范围不变。

- **D-1124-T01-5 数值解析与序列化文本**：JSON 数字文本按 RFC 8259 语法校验后 **正确舍入** 为 `f64`（平位取偶），超出有限范围抛 `JsonError: number overflow`，不足最小次正规数之半读为 `0`；最多考察 800 位有效数字（任何 double 的舍入边界都不需要超过 767 位），其余非零位作为粘滞位参与舍入。序列化时，精确 `i32` 整数输出整数字面量（含 `-2147483648`；字面量 `-0` 保留符号），其余数值输出 **最短往返十进制**：读回得到同一个 `f64`，十进制指数 `k`（首位的权）在 `-6 <= k <= 20` 时用位置记法，否则用 `d.ddde<k>`。这是数据格式，不复用控制台打印格式（`f64.to_str()` 仍按控制台语义）。最短候选有多个时取最接近精确值者，等距时取末位为偶数者（与 ECMAScript `Number.prototype.toString`、Python `repr` 一致；Rust 的 `{}` 在等距时向上入，例如 `253883181407725.125` 得 `.13`，因此不能充当这一条的 oracle）。运行期求得的 `-0.0` 输出 `0`；`f32` 字段经 `f64` 输出。（修复轮 r1 补充：这半句原先缺失，oracle 种子 7 的 `253883181407725.140624` 暴露了等距取偶的实现缺陷，已修，见 §9。）
- **D-1124-T01-6 格式化与缩进布局**：`minify(s)` 与 `prettify(s)` 是文本级格式化——先完整校验输入（语法与预算），再原样保留每个记号并重排空白，数字文本不变。缩进布局固定为：每个成员独占一行、每层缩进两个空格、`"key": value`、空容器写作 `[]` / `{}`、末尾无换行。`encode_pretty(v)` 是 `encode(v)` 的这种缩进排版，两者只差空白。
- **D-1124-T01-7 预算的单位、适用入口与测试边界**：三种预算（输入 `1_048_576` **字节**按 UTF-8 计、节点 `65_536`、嵌套 `128` 层；CSV 为字节与单元格 `65_536`）在 `parse_strict`/`minify`/`prettify`/`is_valid` 与 CSV 的解析、编码入口生效。计数预算在输入达到 `131_071` 字符后先数分隔符（合法文档里每个逗号或冒号恰好引入一个值，短于它的输入达不到上限），超限即在构建任何节点前拒绝。这些上限高于默认执行限额（`ExecLimits::default()`，100 万步）下能完成的解析规模，对提高限额的宿主生效。字节预算与两种计数预算的两侧都在提高限额的用例里按精确边界断言。计数预算的接受侧用一个首字符即非法的同长度文本：它必须先通过准入才会到达解析器；边界上的完整解析（65,536 个节点）需要数千万步，且在当前 VM 上逐字符读取长文本的墙钟是平方级（§10 债务候选 D-1124-AV-TEXT-01），不作为用例。先数分隔符的预扫按 256（长文本 1024）个标量的短片段读取，数到预算即停——本 VM 上直接逐字符读取长文本，每次读取都要物化整段文本。（修复轮 r1 修订：原写法是“接受侧只在预算之内断言到可执行规模”。）
- **D-1124-T01-8 单文件嫁接规则**：程序引用了模块的公开名（用作值、调用目标、类型或 spec；注释与字符串字面量里的名字不算）且未自行声明该名时才嫁接。嫁接体的自由函数统一改名到保留命名空间（`__std_json_`、`__std_csv_`），公开名只是转发函数；程序仍使用库的其他入口时，被程序遮蔽的公开入口不再附加。因此程序自己的 `fn parse`、`fn minify`、`fn json_cp` 不会被库替换或重绑。程序声明了库的记录名 `JsonValue` 时，嫁接的记录整词改名为 `__std_json_JsonValue`（库的入口返回这个类型），程序自己的 `JsonValue` 仍是它自己的——与 `packages` 面里两者经模块限定共存一致。程序名与保留名（`__std_json_*`、`__std_csv_*`、`__json_*`、`__csv_*`）相同时得到普通的重复定义诊断（`source.duplicate`）。`JsonEncodable` 是入库契约名，不改名：程序另行声明同名类型时 `encode` 的约束无法满足，诊断落在调用处。这与 list 嫁接的既有做法（`__std_list_` 命名空间）一致。（修复轮 r1 修订：原写法是“与库的记录名 `JsonValue` 相同时得到重复定义诊断”，那会让程序自己的类型挡住整个库，已按本条改掉。）
- **D-1124-T01-9 覆写生效的位置**：D-1124-T01-3 的自动合成只针对未手写 `json_encode` 的记录；手写的 `fn json_encode() Str` 在顶层、嵌套字段和 `List` 元素三种位置都被调用。`List` 元素是视图：覆写体若需要整个容器字段，该调用在绑定期被语言既有的视图规则拒绝（`source.view-escape`）。
- **D-1124-T01-10 CSV 分隔符与行尾**：分隔符须是单个 Unicode 标量且不是引号或换行（`;`、`\t` 与非 ASCII 单标量都可以）。解析接受 `\r\n` 与 `\n` 行尾（单独的 `\r` 也视为行尾），编码输出 `\n`；退化行（空行、只含空字段的行）不保证往返。D-1124-T01-4 的分隔符规则不变；复审指出的不符在规范正文（曾写成“拒绝多字节分隔符”），已随 SD-01 修正。

## 5. 详细设计

动态值覆盖对象、数组、文本、数值、布尔、null；区分缺键、null、类型错误和坏文档。T-01 冻结键顺序、重复键、数字精度、递归深度/字节预算与失败语义，并用跨模块记录证明编码字段名和类型的实际来源。首批泛型编码需覆盖有代表性的记录、可选字段、嵌套记录及文本/数字；能力边界在 API 上明确。CSV 按旧公开声明的二维数据边界核实引号、分隔符、换行、空字段及错误；保留现有 975 低层和 typed decode API。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/ace-migration/m2-stdlib-data.md`](../specs/ace-migration/m2-stdlib-data.md) | 增加动态 JSON、泛型编码和 CSV 的独立公共合同、旧 API 映射及受检范围 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01/02 · T-02 | 解析 `{"n":null,"a":[1.5,"中"]}`、未知键、畸形 JSON 和序列化 | 键/数组/非整数值可访问，区分 null 与缺键；紧凑/缩进输出按冻结精确预期检查，畸形输入明确失败。 |
| V-02 | AC-01/02 · T-03 | 两个跨模块、不同字段形的记录做泛型编码/再读 | 逐字段核独立文本或结构预期，字段重命名使实际输出改变；不满足编码合同的实例具名拒绝。 |
| V-03 | AC-01/02 · T-04 | 默认及自定义分隔符；被引号包住的逗号、换行、空字段、截断 | 解析得预定二维数据，反向输出按冻结字节合同，合法空字段和未结束引号不同；预算或解析错误后临时值可释放。 |

冻结验证命令为 `cargo t stdlib ace-stdlib ace-source ace-semantics ace-av case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [ ] AC-01 · owner_stage=work。V-01–03 都从普通工程真实调用公共 API，结果与独立预期相符。
  - [↩ 复审 r1 判 fail，2026-11-23] V-01 fail（F-01 序列化不保值、F-02 i32 最小值 trap、F-08 数值精度）；V-02 fail（F-03 `encode_pretty` 改写宽整数、F-04 `List` 元素绕过用户覆写）；V-03 partial（F-07 预算无测试且单位不符）。详见 §9。
  - [✅ work 修复轮 1 完成 2026-11-24] 提交 `9a9c296ec`。F-01、F-02、F-03、F-04、F-08 与自查的 N-1、N-5、N-6 已修并有永久断言；一致性 `plan1124_json_csv` 26/26、语言用例 `av.parity.1124.*` 8/8、仓库外数值 oracle 34,629 个输入 0 不一致。待 T-05 重审确认。
  - [↩ 复审 r2 判 partial，2026-11-24] V-01 数值、树与格式化成立；V-02 的 `List` 位置覆写只对全标量记录成立（F-14）；V-03 语义成立，预算见 F-15。详见 §9。
  - [✅ work 修复轮 2 完成 2026-11-24] 提交 `b5ada0bd0`。F-13、F-14、F-15、F-17、F-18 已修并有永久断言：`plan1124_json_csv` 37/37（串行最慢 7.76 s）、语言用例 9/9（含新增 `graft-local-names`）、近邻 54/54。待 T-05 重审确认。
- [ ] AC-02 · owner_stage=work。旧接口义务和已验 JSON 近邻逐项保留，错误和资源预算可观察。
  - [↩ 复审 r1 判 partial，2026-11-23] 近邻回归全部保留，四个 CSV 旧公开名保留；预算只在提高 `ExecLimits` 后可观察，CSV 单元格预算与恰 1 MiB 侧无断言（F-07）；新回归一项（F-05）；测试残留临时目录（F-09）。详见 §9。
  - [✅ work 修复轮 1 完成 2026-11-24] 提交 `9a9c296ec`。F-05、F-06、F-07、F-09、F-10、F-11、F-12 与自查的 N-2、N-3、N-4 已修；近邻 54/54；字节、节点、单元格三种预算都按精确边界断言两侧并核对 arena 账本；修复后的每轮运行 `%TEMP%` 目录计数不变。待 T-05 重审确认。
  - [↩ 复审 r2 判 partial，2026-11-24] 近邻 54/54，字节预算在全部入口成立；嫁接触发对局部名误判（F-13）；CSV 预扫漏单独 CR，阈值以下的稠密输入先建后拒（F-15）。详见 §9。
  - [✅ work 修复轮 2 完成 2026-11-24] 提交 `b5ada0bd0`。F-13（作用域引用分析 + 不嫁接优先编译、疑问时重编）、F-15（预扫数 CR、阈值 65,536）、F-16（新增行 rustfmt 归因 0）、F-17（规则入注释与三条新断言）、F-18（负零符号、`csv_encode_rows` 单元格预算、规范五处）已修；`%TEMP%` 残留计数 30 → 30。待 T-05 重审确认。
- [ ] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
  - [修复轮 1 之后待重审 2026-11-24] 受审提交 `9a9c296ec`（基线 `d206320e8` 之上）。
  - [复审 r2 2026-11-24] outcome: needs_fix（`review_scope: repair`，受审提交 `9a9c296ec`）；F-13..F-19 见 §9。修复后对新提交重审受影响的 AC。
  - [修复轮 2 之后待重审 2026-11-24] 受审提交 `b5ada0bd0`；重审 F-13、F-14、F-15、F-16、F-17、F-18、F-19 的关闭情况，以及新增 11 个一致性用例与 1 个语言用例（10 秒门、串行最慢 7.76 s）。
  - [复审 r1 已执行，2026-11-23] outcome: needs_fix（受审提交 `d206320e8`）。修复后对新提交重审受影响的 AC。
- [ ] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [ ] T-01 核定数据接口、资源合同和编码事实
  - owner_stage=work。重放 975 与旧 CSV/JSON 代表输入。固定数据模型和歧义政策，验证真实字段事实或显式 schema/spec，再冻结兼容表。
  - [↩ 复审 r1 回开 2026-11-23] 回开范围仅限冻结记录的补全：缩进格式（F-06）、预算单位与适用入口（F-07）、数值范围与精度承诺（F-08）、分隔符规则与规范正文统一（C4）。下列原完成说明是实现方的主张，与 §9 复审记录冲突处以后者为准。
  - [✅ 修复轮 r1 完成 2026-11-24] 冻结记录补全 D-1124-T01-5..10 已写入 §4，含修复轮内的三次自我修订：T01-8（库记录 `JsonValue` 遇同名声明时改名）、T01-5（等距取偶；Rust 的格式化不是这一条的 oracle）、T01-7（预算两侧都按精确边界断言）。
  - [↩ 复审 r2 回开 2026-11-24] 只限冻结记录的文本：T01-7（CSV 预扫的阈值与行尾）、T01-8（“未自行声明该名”须按作用域理解）、T01-9（`List` 元素位置的覆写只对字段全为复制标量的记录成立）。
  - [✅ 修复轮 2 完成 2026-11-24] T01-7 阈值与 CR 行尾、T01-8 的作用域口径、T01-9 的 `List` 元素边界都已按复审要求落到 §4 与规范正文（提交 `b5ada0bd0`）。
  - [✅ 已完成 2026-11-23] 基线 `json975` 10/10 绿（3.46s）；核实 `stdlib/ace/std/json.at` 受 975/1064/AGE `Faces::Plain` 消费锁定，确定普通 Auto AV 增补模块、LCRS `JsonValue`、last-member-wins、`f64` 精度/预算、`JsonEncodable` 基于 `TypeItem.fields` 合成及 RFC-4180 CSV 四公开函数合同（详见 §4 T-01 冻结记录）。
- [ ] T-02 实现动态 JSON 读取与格式化
  - owner_stage=work。依赖 T-01。复用解析器已验语义交付动态树和访问方法，验 V-01。
  - [↩ 复审 r1 回开 2026-11-23] F-01（f64 文本序列化不保值）、F-02（i32 最小值 trap）、F-05（JSON 嫁接按裸标识符触发与同名去重）、F-06（缩进未冻结）、F-07（节点预算无测试、单位不符、大输入墙钟超线性）、F-08（数值非正确舍入）、F-09、F-10。下列原完成说明是实现方的主张，与 §9 复审记录冲突处以后者为准（“扁平 O(N)”在 AV 墙钟上不成立）。
  - [✅ 修复轮 r1 完成 2026-11-24] F-01、F-02、F-05、F-06、F-07、F-08 与 N-1、N-2、N-3：`stdlib/ace/std/json_auto.at`（正确舍入的解析、最短往返文本、文本级格式化、字节预算与分块的分隔符预扫）、`adapter.rs` 与新文件 `stdlib_refs.rs`（按引用嫁接、保留命名空间、库记录改名、冲突诊断指向程序声明）。
  - [↩ 复审 r2 回开 2026-11-24] F-13（引用分析把局部名字当成引用）、F-16（`instances.rs` 新增行一处格式）。
  - [✅ 修复轮 2 完成 2026-11-24] F-13：`stdlib_refs.rs` 作用域栈 + `adapter.rs` 不嫁接优先/疑问重编；F-16：新增行 rustfmt 归因 0。单元 7/7、一致性 37/37、语言用例 9/9（提交 `b5ada0bd0`）。
  - [✅ 已完成 2026-11-23] `stdlib/ace/std/json_auto.at` + `adapter.rs`/`packages/binding.rs`/`resolve.rs`/`av.rs`：交付扁平 O(N) LCRS `JsonValue`（`parse`/`parse_strict`/`is_valid`/`minify`/`prettify`、`.kind`/`.type_of`/`.keys` 后成员胜出、`.get`/`.get_at`/`.at`/`.get_*`/`.as_*`）、`1 MiB` / `65536` 节点 / `128` 深度预算及 `f64.to_str()`/`TextDisplay(F64)`，保持 `stdlib/ace/std/json.at` 零字节改动。
- [ ] T-03 实现泛型 JSON 编码
  - owner_stage=work。依赖 T-01/T-02。经普通模块交独立实例与编码失败，验 V-02。
  - [↩ 复审 r1 回开 2026-11-23] F-01（f64 字段）、F-03（`encode_pretty` 改写宽整数）、F-04（`List<R>` 元素绕过用户 `json_encode` 覆写）、F-10（格式与 `let _ = …?`）。下列原完成说明是实现方的主张，与 §9 复审记录冲突处以后者为准。
  - [✅ 修复轮 r1 完成 2026-11-24] F-01（浮点字段）、F-03、F-04、F-10：`json_encode_synth.rs`（`List` 元素走用户的 `json_encode`）、`instances.rs` 与 `modules/packages.rs`（检查合成步骤的返回值）。
  - [↩ 复审 r2 回开 2026-11-24] F-14（带自有字段的记录作 `List` 元素时覆写不被调用，规范与计划写得过宽）、F-16。
  - [✅ 修复轮 2 完成 2026-11-24] F-14：`json_encode_synth.rs` 在 `List` 字段处具名拒绝（`source.json-encode-unsupported`），规范写实际规则；F-16 新增行格式。用例 `f14_*`（两面）通过（提交 `b5ada0bd0`）。
  - [✅ 已完成 2026-11-23] `json_encode_synth.rs` + `instances.rs`（补全 `Stmt::Try`/`AggIsMatch`/`With`/`Yield` 重绑与 `Option` 字段归整）+ `packages.rs` + `cst/project.rs`（`List<f64>`/`tok_affix_num` 修复）：基于 `TypeItem.fields` 合成 `JsonEncodable`（覆盖全宽整数、`f32`/`f64`、`bool`、`Str`/`str`、`?T`、`List<T>`、嵌套记录、`Empty`、`Custom` 重写），拒绝未合规记录（`source.instances-bound`）、非法字段/嵌套未合规/循环（`source.json-encode-unsupported`）、伪造 `std`（`ace.stdlib-reserved`）与非有限浮点（`JsonError`）。
- [ ] T-04 迁移 CSV 公开操作
  - owner_stage=work。依赖 T-01。落实二维语义、分隔符与错误，验 V-03，汇入公共语料。
  - [↩ 复审 r1 回开 2026-11-23] F-05（CSV 嫁接触发器）、F-07（CSV 字节与单元格预算无断言）、F-11（嫁接错误的 span 保护不完整）、F-12（规范文本与行为不符）。下列原完成说明是实现方的主张；其中“单测 ≤0.3s”与复审实测不符（`v01_and_v02_packages…` 实测 1.608 s）。
  - [✅ 修复轮 r1 完成 2026-11-24] F-05（CSV 嫁接）、F-07（字节与单元格预算）、F-11、F-12 与 N-4：`stdlib/ace/std/csv.at`、`language_source_worker.rs`（嫁接区错误定位到程序的调用）、语言用例 `numbers-exact`、`pretty-bytes`、`graft-own-names`、`override-list`、规范 `docs/specs/ace-migration/m2-stdlib-data.md`。
  - [↩ 复审 r2 回开 2026-11-24] F-15（CSV 预扫漏单独 CR 与稠密输入）、F-16、F-17（库错误的位置可能落到另一个入口的调用上）。
  - [✅ 修复轮 2 完成 2026-11-24] F-15：预扫数 CR/CRLF、阈值 65,536，七条 `f15_*` 断言；F-16；F-17：规则入注释 + 三条新断言（另一入口、分支、程序函数）。`csv.at` 与规范同步（提交 `b5ada0bd0`）。
  - [✅ 已完成 2026-11-23] `stdlib/ace/std/csv.at`：交付纯 Auto RFC-4180 CSV（`csv_parse`/`csv_parse_with_delimiter`/`csv_encode`/`csv_encode_with_delimiter`/`csv_parse_rows`/`csv_encode_rows`，支持 `std.csv` 与 `std.encoding.csv`），补齐 `language_source_worker.rs` 嫁接错误 span 保护；落地 `crates/auto-lang/test/cases/language/stdlib/json-csv.case.ad`（登记 `language.suite.ad`）与规范 `docs/specs/ace-migration/m2-stdlib-data.md`（SD-01）。
  - **Work Readiness（dev/debug 实测）**：`ace-core stdlib::tests` 4/4 passed（0.23s）；`plan1124_json_csv` 6/6 passed（总 1.37s，单测 ≤0.3s，arena 0 leak）；`av.parity.1124.{json-dynamic-tree, json-encodable-schema, json-csv-packages, csv-rfc4180}` 4/4 PASS（热测 725ms / 727ms / 1099ms / 647ms，均 ≤10.0s）；近邻回归 `json975` 10/10、`plan1120` 11/11、`plan1121` 26/26、`module_v_1064` 7/7 全绿。
- [ ] T-05 独立复审
  - owner_stage=review。依赖 T-04。核 AC-03、SD 增量、实测结果和旧用例映射。
  - [复审 r1 2026-11-23] outcome: needs_fix（`review_scope: final`，受审提交 `d206320e8`）；发现 F-01..F-12 见 §9。修复提交后重审受影响的 AC。
  - [修复轮 1 之后待执行 2026-11-24] 受审提交 `9a9c296ec`；重审 F-01..F-12 与 N-1..N-6 的关闭情况、SD-01 与新增用例的 10 秒门。
  - [复审 r2 2026-11-24] outcome: needs_fix（`review_scope: repair`，受审提交 `9a9c296ec`）；F-13..F-19 见 §9。修复提交后重审受影响的 AC。
- [ ] T-06 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1124 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

2026-11-23 · work/T-01 · 基线 `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-av --test conformance json975` 10/10 PASS（3.46s）；核实 975 `json.at`、`stdlib/auto/json.at`、`stdlib/auto/encoding/csv.at`、`natives_formats.rs` 与 `strings_json.rs`，冻结 §4 T-01 合同（D-1124-T01-1..4）。next: T-02。

stage: goal | plan_id: PLAN-1124 | plan_revision: r1 | round: 1 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1124/auto-lang | commit: 8a52d7364 | next: work/T-02

对账记录（2026-11-23，goal 驱动会话，非阶段收据）：接管时 `status: execution_done` 且 T-01..T-04 已勾选，但 §9 只有 T-01 的 work 收据与 round 1 收据，缺 T-02..T-04 的 work 收据和 work `pass` 交接记录；树与提交的事实是实施树干净停在 `d206320e8`（基面之上唯一提交）。缺失的 work 收据不予补造，以本节复审记录和后续修复轮收据为准。用户另行发起的首个独立复审中途挂掉，磁盘、会话、进程均无产物。

2026-11-23 · stage: review · review_scope: final · PLAN-1124 r1 · outcome: needs_fix。实现方的测试与近邻回归在 `d206320e8` 上全部重放通过，但独立反例证实了合同内的错误结果（F-01、F-02、F-03）和一项新回归（F-05），AC-01 不成立。next: work。

stage: review | review_scope: final | reviewer_identity: 发起会话派生的 general-purpose 子代理（全新上下文，未参与实现；环境模型标识 GLM-5.4；无会话 ID）| plan_id: PLAN-1124 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1124/auto-lang | branch: plan-1124-dev | reviewed_commit: d206320e86ecc6bb7f7d4ba0b9c61089201df8f7 | base_commit: 8a52d7364e5b218e0a08b167038a5d71d54ee72f | dependency_revisions: auto-down 兄弟树 detached 456c86ac886f9781e56f6153ce9aa41cd59e5c8e | spec_inputs: docs/specs/ace-migration/m2-stdlib-data.md @d206320e8 blob 83bd7a3ed（基面 8b32fe786），sha256 30aab0a310c109bd17ee2508a7681ae33e775dd7aeb7bfa5f4ad0bd3e62fd6d4 | acceptance_results: AC-01 fail · AC-02 partial · AC-03 fail · AC-04 pending(merge) | pending_closeout_gates: 冻结组并集（见本节末） | findings: F-01..F-12 | evidence: D:/autostack/plan-artifacts/plan-1124/review/（`verdict-r1.md` sha256 3746b0e3f13df23ec610b241bc2a607d0aab06b6e25a6824335def6975f28a2e；原始目录 1.1 GB，其中探针 crate 的构建目录是可删除缓存，merge 时随材料区整目录删除；下列内容为持久摘要）| next: work

**基线核实**

- 实施树进出各查一次 `git status --porcelain` 均为 0 行，收尾时 HEAD 仍为 `d206320e8`；`base..HEAD` 仅 1 个提交，20 个文件 +3645/−17。
- master 现值 `0486981556bd8d128c69ded9888319ac4a65fc02`，较基面多 1 个只改 `docs/plans/KNOWN-DEBT-AND-RISKS.md` 的 docs 提交，未触及本计划路径；分支已不能快进，合并前需变基。
- `stdlib/ace/std/json.at` 的 blob 为 `95b6c561ec5aaf1e5cf195358dfae62f4252b82e`，HEAD 与基面相同。
- diff 未触及 `.autoos/`、`specs.json`、`index.json`、`KNOWN-DEBT-AND-RISKS.md`、`.retention`、`docs/plans/`。
- 驱动会话在记录前抽查了 F-01、F-02、F-04、F-05 的原始输出，与复审报告一致。

**审查局限**：基面行为未实跑（没有基面 worktree），凡称“基面上如何”处均为 diff 读码推断。占用设门偏离两处：18:19:10 的 `cargo nextest list` 未对占用计数设门，与他方 18:19:06 起的 `cargo test -p auto-lang-core --lib file_session::tests::` 并发约 54 秒；20:46:38 查到 `busy_count=2` 仍跑了 6 个亚秒级 worker 探针。其余起跑均以 `busy_count == 0` 为门。20:31–20:39 机器内存耗尽，8 个重型探针以 `memory allocation … failed` 退出，结果作废。

**验收结果**

| AC | 结论 | 要点 |
|---|---|---|
| AC-01 | fail | V-01 fail（F-01、F-02、F-08）；V-02 fail（F-03、F-04）；V-03 partial（F-07） |
| AC-02 | partial | `json975` 10/10、`plan1120` 11/11、`plan1121` 26/26、`module_v_1064` 7/7；四个 CSV 旧公开名保留；预算只在提高 `ExecLimits` 后可观察（F-07）；新回归 F-05 |
| AC-03 | fail | 本记录即执行结果；独立反例证实 F-01..F-08 与收尾缺陷 F-09..F-12 |
| AC-04 | pending | 归 merge |

**发现清单**（高 = 合同内的错误结果或不可恢复失败；中 = 合同内缺陷或新回归，影响面较窄；低 = 收尾与卫生）

- **F-01（高，原需求缺口；AC-01、V-01、T-02、T-03）f64 文本序列化不保值。**
  - 实测：`minify("1e-10")` 得 `0`；`minify("[1e-10,5e-324,1e-320]")` 得 `[0,0,0]`；`minify("0.9999999996")` 得 `1`；`minify("2.0000000005")` 得 `2`；`encode(M{tiny:1e-10, almost:0.9999999996, over:2.0000000005})` 得 `{"tiny":0,"almost":1,"over":2}`。101 个数值输入对照正确舍入的 double：27 个变 0、6 个被改成整数、12 个差 1–4 ULP、3 个有限值被拒、3 个越界被拒、1 个无输出（trap）、47 个正确。`as_number()` 读回的内存值完好。
  - 原因（已确立）：`json_auto.at:942` 的 `__json_f64_str` 调 `x.to_str()`；`resolve.rs:21596` 把它降为 `TextDisplay`；`av.rs:8464` 交给 `fmt_console_f64`（`av.rs:11875`，与整数相差小于 1e-9 即输出整数）。控制台格式被复用于数据序列化。
  - 关闭 oracle：近整数小数（差小于 1e-9）、次正规数、`1e-7`、大指数各取代表值，断言输出文本解析回的 double 与输入逐位相等，并补 `parse_strict(minify(x)).as_number()` 往返；不改控制台打印语义。
  - 证据：`review/worker-out/minify-numbers.json`、`review/probes-out/p1124r.c1.encode-f64-fields/`、`review/logs/16-oracle-analysis.txt`。
- **F-02（高，原需求缺口；AC-01、V-01、D-1124-T01-2）`-2147483648` 触发不可捕获的 trap。**
  - 实测：`minify("-2147483648")` 与 `minify("[-2147483648]")` 报 `av.trap.int-overflow`；`encode_pretty(R{m:-2147483648})` 在 `try` 块内 trap，`catch` 没有接住（`encode` 正常）。进程内复现 `Trap { kind: IntOverflow }`。
  - 原因（已确立）：冻结的 `json.at:59` `json_int_str` 做 `n = 0 - n`；`json_auto.at:977` 对精确 i32 节点调用它。`json.at` 不能改，修补点在 `json_auto.at`。`as_int()` 与 `as_number()` 不经过它，实测正常。
  - 关闭 oracle：`minify`、`prettify`、`json_encode`、`encode_pretty` 对含 i32 最小值的文档给出精确文本，且不产生 trap。
  - 证据：`review/worker-out/p1124r.c1.int-min-{minify,array,encode-pretty}.json`、`review/probe-crate/out/int-min-1.json`。
- **F-03（中，原需求缺口；AC-01、V-02、D-1124-T01-3）`encode_pretty` 改写宽整数。**
  - 实测：`encode(W{u:9007199254740993u64, s:-9007199254740993i64})` 精确；`encode_pretty(同值)` 得 `9007199254740992` 与 `-9007199254740992`。
  - 原因：`json_auto.at:668` 的 `encode_pretty` 实现为 `prettify(value.json_encode())`，文本经 f64 树往返，同时继承 F-01 与 F-02。
  - 关闭 oracle：`encode_pretty(x)` 去空白后与 `encode(x)` 逐字节相等；取值覆盖超过 2^53 的 u64/i64、i32 最小值、近整数小数。
  - 证据：`review/probes-out/p1124r.c1.wide-int-encode-pretty/`。
- **F-04（中，合同澄清；AC-01、V-02、SD-01 §2）`List<R>` 元素不走用户的 `json_encode` 覆写。**
  - 实测：`encode(Holder{customs:[Custom{x:1}]})` 得 `{"customs":[{"x":1}]}`，而单值的自定义覆写生效。
  - 原因：`json_encode_synth.rs:303` 的元素视图走内联字段投影。计划 D-1124-T01-3 与规范“用户显式 `fn json_encode() Str` 优先保留”都没有这个例外。
  - 关闭 oracle：同一记录在顶层、嵌套字段、`List` 元素三个位置的编码文本一致；规范写明实际规则。
  - 证据：`review/probes-out/p1124r.c5.list-of-empty-and-custom/`。
- **F-05（中，新回归；AC-02、T-02、T-04，共享面 `adapter.rs`）按裸标识符嫁接导致既有程序无法编译。**
  - 实测（对照：去掉触发词后同样的程序正常）：
    - 只声明 `type JsonValue { v int }` 得 `source.undefined: record JsonValue has no field named root`，span 54760–54764，而文件只有 110 字节。
    - 注释含 "encode" 且自定义 `fn parse(s Str) Str` 得 `source.type-mismatch: expected Str, found str`，span 41821。
    - 自定义 `fn json_cp` 并调用 `parse_strict` 得运行期 `JsonError: malformed number or unexpected token`。
    - 正例：用户自定义的 `encode`、`minify`、`csv_parse` 仍被正确使用。
  - 原因（已确立）：`needs_stdlib_json_graft` 在原始文本上按标识符边界匹配 `encode`、`minify`、`prettify`、`JsonValue`、`parse_strict`、`encode_pretty`、`json_str`、`json_is_valid`、`json_escape`，注释与字符串字面量都会命中；嫁接的公开名与内部助手没有改名；`existing_fns` 与 `existing_types` 静默丢弃同名的嫁接项，嫁接体随之重绑到用户符号。基面没有 JSON 嫁接（读码，未实跑）。
  - 受影响的兄弟路径：去重也改变了既有 list 嫁接的冲突行为（用户定义 `__std_list_map` 时现在静默用户胜出，而 `adapter.rs:349` 的注释仍写保留重复函数诊断）；`Item::Enum(_) => continue` 目前是死分支，将来会静默丢项。
  - admission 预算：`AUTO_SOURCE_ADMISSION_CAP` 是能力 id，不是尺寸上限；尺寸预算是 64 MiB，嫁接增加的 66 KB（JSON）与 14 KB（CSV）可以忽略，没有观察到预算拒绝。代价在编译时间，见 O-1。
  - 关闭 oracle：注释或字符串含触发词且用户定义 `parse`、`JsonValue` 的程序保持基面行为；补一条同名冲突的断言；注释与实现一致。
  - 证据：`review/worker-out/` 下的 `user-type-jsonvalue.json`、`comment-trigger-user-parse.json`、`user-helper-json-cp.json`；`review/probes-out/p1124r.c9.*`。
- **F-06（中，原需求缺口；AC-01、V-01、T-01）缩进输出没有冻结，也没有精确字节断言。**
  - 实测格式：2 空格缩进、`"key": value`、空容器写成 `{}` 与 `[]`、无尾换行，与 Python `json.dumps(indent=2, ensure_ascii=False)` 一致。现有断言只有往返比较和 `contains("\n  \"a\": -8")`。
  - `as_array()` 在实现方的任何测试里都没有被调用，其自然用法受既有的 `verify.view-escape` 规则限制（O-9）。
  - 关闭 oracle：T-01 写明缩进格式；一个有代表性的文档断言完整字节；`as_array()` 的结果至少有一条可用性断言。
  - 证据：`review/probes-out/p1124r.c7.prettify-bytes/`。
- **F-07（中，原需求缺口；AC-02、V-03、D-1124-T01-4）预算合同缺口。**
  - 没有测试：CSV 的 1 MiB 与 65,536 单元格预算、JSON 的 65,536 节点预算，在实现方的 Rust 测试与语言用例里都没有断言。
  - 单位不符：`str.len()` 的单位是 Unicode 标量（`"中".len()` 得 1，`len_bytes()` 得 3）。计划写“1 MiB”，规范写“`1_048_576` B”，错误文本写 "input byte budget exceeded"，实际是 1,048,576 个标量，UTF-8 下最多 4 MiB。
  - 产品入口不可达：默认 `ExecLimits` 是 1M 步、2 s。JSON 约 1,500–1,900 个节点、CSV 约 10–15 KB 或约 5,000 个单元格就报 `av.resource.instr-steps`。预算只在提高限额时可观察。
  - 提高限额后的结果（仓库外探针 crate）：

    | 探针 | 结果 | arena |
    |---|---|---|
    | `json-bytes-1048575`（1,048,577 标量） | `JsonError: input byte budget exceeded` | 5/5，live 0 |
    | `json-bytes-cjk-1048575`（3 MiB 字节） | 同上 | 5/5，live 0 |
    | `csv-bytes-1048577` | `CsvError: input byte budget exceeded` | 5/5，live 0 |
    | `csv-encode-bytes-1048571` 与 `-1048572` | 同上 | 5/5，live 0 |
    | `json-nodes-65535`（65,536 节点） | `OK len=65535` | 196,677 次分配全部释放 |
    | `json-nodes-65536`（65,537 节点） | `JsonError: node budget exceeded` | 196,676 次分配全部释放 |
    | 深度 127、128 成功，129 报 `JsonError: nesting depth exceeded` | 边界含 128 | 检查器路径不暴露 arena |
    | `json-bytes-1048574`、`json-bytes-cjk-1048574`、`csv-bytes-1048576`、`csv-bytes-cjk-349526`、CSV 65,536/65,537 单元格及尾部分隔符三项、`csv-encode-cells-*` | 未验证：机器内存耗尽或批次被停 | 未观测 |

  - 墙钟超线性：65,536 个节点耗 1646 s（约 43M 步），而 1,500 个节点约 1.2 s（约 1M 步）。步数线性而墙钟超线性，原因没有定位，推测是 `List` 写入时的宿主侧核算（假设）。新增的预算边界用例必须满足 10 秒门。
  - 关闭 oracle：节点与单元格预算两侧的断言加零泄漏断言；计划与规范写明单位与适用入口。
  - 证据：`review/probe-crate/out/`、`review/probes-out/p1124r.lim.*`、`review/probes-out/p1124r.c8.len-units/`、`review/probes3-out/p1124r.c2.depth-boundary-v2/`。
- **F-08（中，合同澄清；AC-01、V-01、T-01“数字精度”）数值解析不是正确舍入，且拒绝部分有限值。**
  - 实测：1–4 ULP 误差见 `1e100`、`1e200`、`1e300`、`1e308`、`12345678901234567890`、`0.1234567890123456789`、`99999999999999999999` 等 12 个；`1.7976931348623157e308`、`-1.7976931348623157e308`、`1.7976931348623158e308` 都报 `JsonError: number overflow`。
  - 原因（读码）：`json_auto.at:773` 的 `__json_parse_f64` 累加 18 位有效数字后，乘以平方求幂的 `__json_pow10`。
  - 关闭 oracle：各量级取值与正确舍入结果逐位相等，或由计划明确写出精度与范围承诺。
  - 证据：`review/logs/16-oracle-analysis.txt`。
- **F-09（低，收尾缺陷；AC-02）测试每次运行残留临时目录。** `plan1124_json_csv.rs:113–132` 的 `run_pkg_value_released` 在 `PackageAvTarget.store` 仍打开时调用 `remove_dir_all` 并忽略错误；每个目录留下 `journal.db`（69,632 B）与 `locks/`。复审一次运行前后 `%TEMP%\p1124-*` 由 27 变为 30，收尾仍是 30。关闭 oracle：跑一次后计数不变。
- **F-10（低，收尾缺陷；健康检查）。** 新增行上 17 处未通过 `rustfmt --check`：`plan1124_json_csv.rs` 4 处、`json_encode_synth.rs` 9 处、`adapter.rs:724`、`project.rs:15156`、`packages/stdlib.rs:336`、`resolve.rs:18098`；同文件另有 252 处基面既有差异，不计在本提交头上。新增 1 个 `#[allow(clippy::too_many_arguments)]`；3 处 `let _ = …?` 丢弃了返回的来源列表（`adapter.rs`、`source-query` 的 `instances.rs` 与 `modules/packages.rs`）。受影响 crate 的编译器警告（8 + 2 + 86 条）都不落在本提交的改动行上，没有新增。非测试代码没有调试打印、`todo!`、`unimplemented!` 或注释掉的代码。`pub mod json_encode_synth` 是跨 crate 调用所需。证据：`review/logs/fmt-summary.txt`、`review/logs/20-diff-scan.txt`。
- **F-11（低，收尾缺陷；T-04）嫁接错误的 span 保护不完整。** `language_source_worker.rs` 只保护了 `language_error.span`。未捕获的错误与 trap 仍带着落在嫁接区的 `source_span`，例如未捕获的 `csv_parse("\"unclosed quoted field")` 与 F-02 的 trap；经检查器时表现为 infra "invalid worker source span for the submitted source"，真实诊断被遮蔽。关闭 oracle：未捕获的 std 库错误的 span 不超出用户源；检查器给出被判定的诊断而不是 infra。
- **F-12（低，规范增量；SD-01）规范文本与行为不符。** 见下文“规范增量结论”。

**预检断言 C1–C9 的结论**

| 断言 | 结论 | 依据 |
|---|---|---|
| C1 f64 序列化丢值 | 证实，范围比断言更广：所有绝对值小于 1e-9 的值都变 0 | F-01 |
| C2 节点上限 65,536 | 证实：65,536 接受，65,537 抛 `JsonError: node budget exceeded`，捕获后零泄漏；默认限额下不可达 | F-07 |
| C3 CSV 预算 | 部分证实：超过 1 MiB 时解析与编码都抛 `CsvError` 且零泄漏；单元格预算与恰 1 MiB 侧未验证 | F-07 |
| C4 CSV 边界 | 闭引号后杂字符抛 `CsvError`；空串返回 `"[]"`；换行与多字符分隔符被拒。非 ASCII 单标量分隔符（`"中"`、`"😀"`、U+200B）被接受，与计划 D-1124-T01-4 一致，与规范正文不一致 | `review/probes-out/p1124r.c4.boundaries/` |
| C5 `JsonEncodable` 拒绝路径 | 直接递归、互递归、可选递归、`List` 递归、`List<List<Str>>`、`List<?Str>`、嵌套未合规、spec 带类型参数都具名拒绝（`source.json-encode-unsupported`；spec 参数为 `source.unsupported`）。`uint`、`float`、`double` 别名可编码；跨模块可选载荷与可选嵌套记录编码正确 | `review/probes-out/p1124r.c5.*`、`review/probes3-out/p1124r.c5.cross-module-optional-payload-v2/` |
| C6 `as_int()` | 证实：`1.5`、`2147483648`、`-2147483649`、`1e3`、`100.0`、字符串、布尔都抛 `TypeError: json value is not an integer`；`-2147483648` 正常。越界下标抛 `JsonError`，类型不符抛 `TypeError` | `review/worker-out/as-int-only.json`、`at-edges.json` |
| C7 缩进输出 | 格式见 F-06；现有断言不满足 V-01 的“冻结精确预期”，T-01 没有冻结缩进格式 | F-06 |
| C8 `str.len()` 单位 | 是 Unicode 标量，不是字节；它同时决定预算与分隔符校验 | F-07、C4 |
| C9 共享面 | 触发器与去重见 F-05；`coerce_err_to_str_owned` 的改动生效（`fail(动态 str)` 带出真实文本）；`instances.rs` 补全的遍历可用；`to_str` 扩展是 F-01 的通道；`project.rs` 的门控键是 `is_numeric()` 而非 `is_auto_language()`，连带 Numeric、Aggregates、Scheduler 三个冻结面（O-5） | `review/probes-out/p1124r.c9.*` |

**约定范围之外的观察（不阻塞）**

- O-1 注释含 "encode" 的 hello-world，编译加执行从 45 ms 变成 456 ms；字符串含 "prettify and minify" 是 530 ms。
- O-2 `-0.0` 输出为 `0`，零的符号丢失。
- O-3 CSV 的退化行不可往返：空行读成一个空字段，尾部空行与单空字段行编码后丢失。与 Python `csv` 对 34/36 个随机样本逐字节相同，差异只在这些退化情形。
- O-4 记录里写 `static fn json_encode()` 得到的是 `source.duplicate`，不是签名错误时的具名 `does not implement`。
- O-5 `project.rs:2095`、`2845`、`2908` 的门控键是 `is_numeric()`。对冻结 profile 的净效果没有隔离；关键字子集里的 `source_cst_927::differential_corpus_all_profiles` 在本提交上通过，最终留给固定组。
- O-6 `json_encode_synth.rs:28` 用硬编码的 `p3_stdm4_json_json_str` 探测 package 模式，与 std 包的改名方案耦合。
- O-7 `source-query` 的 `modules/packages.rs:92` 对不在输入里的模块（自动插入的 `std::json`）回退到 `"1.0.0"`。它今天与 `STDLIB_VERSION` 相同；版本号一变，这些实例的身份哈希不会跟着变。
- O-8 严格性方面：`01`、`-`、`1.`、`.5`、`+1`、`Infinity`、`NaN`、尾逗号、裸控制字符、孤代理都被拒；重复键后成员胜出且位置保留。
- O-9 `as_array()` 返回的 `List<JsonValue>`，对元素用 `.get(i)` 或 `for-in` 再调方法会报 `verify.view-escape`，只有 `remove(i)` 可用。普通记录列表同样如此（对照探针），是既有的语言规则，不算本计划的缺陷；但它让 `as_array()` 的自然用法不可用，而实现方没有为它写任何测试。

**计时表：本会话实跑范围**

| 范围 | 结果与执行/排除数 | 准备构建 | 发现 | 测试 | 清理 | 总墙钟 | 占用等待 |
|---|---|---|---|---|---|---|---|
| `nextest list -p ace-av --test conformance -E 'test(plan1124_json_csv)'` | 清单 6 | 53.72 s | 含在总墙钟内 | — | 未观测 | 54.7 s | 未等待（设门偏离） |
| 同过滤 `nextest run` | 6/6 PASS，1478 skipped | 0.48 s | 未观测 | 1.617 s | 未观测 | 3.1 s | 88 s |
| 近邻 `list`（`json975`、`plan1120`、`plan1121`、`module_v_1064`） | 清单 54 | 0.20 s | 未观测 | — | 未观测 | 1 s | 0 |
| 近邻 `run` | 54/54 PASS，1430 skipped | 0.26 s | 未观测 | 4.895 s | 未观测 | 5.6 s | 0 |
| `ace-core --lib -E 'test(stdlib::tests)'` 的 list 与 run | 4/4 PASS，87 skipped | 46.86 s + 0.15 s | 未观测 | 0.256 s | 未观测 | 50 s（run 自身 1.0 s） | 200 s（放弃后重试）+ 233 s |
| 官方工具准备 `fixed-tests.py case-av-language-parity --prepare-only` | PREPARED | 62.42 s + 32.88 s + 35.61 s（warm 0.41 s） | — | — | — | 133.8 s | 0 |
| 语言用例 `av.parity.1124.*` ×4（检查器逐例 `--case`） | 4/4 PASS | 复用上项 | 每进程含套件 inventory，未单列 | 见新增用例表 | 未观测 | 进程墙钟 3.6 / 3.3 / 3.7 / 2.9 s | 0 |
| 模块子集第一轮（路径前缀过滤；`-p ace-core -p ace-source-query -p ace-av-core --lib --tests`） | 81/81 PASS，23 个二进制，1240 skipped | list 65 s（含测试二进制构建）+ 0.40 s | 未观测 | 1.836 s | 未观测 | 约 69 s | 0 |
| `-p ace-av --lib --tests --features ace-av/test-case-batch -E 'test(/language_source_worker/)'` | 8/8 PASS，1512 skipped | list 43 s（含 feature 构建）+ 0.22 s | 未观测 | 0.291 s | 未观测 | 约 44 s | 0 |
| 模块子集第二轮（关键字命名过滤，同三个 crate） | 201/201 PASS，23 个二进制，1120 skipped | 0（二进制已构建） | list 4 s | 3.344 s | 未观测 | 约 8 s | 938 s |
| 六组 `--select-only` | 只打印选择，不构建 | 无 | — | — | — | 约 1 s | 不适用 |
| `rustfmt --check`（14 个改动的 `.rs`） | 17 处新增未格式化 | 不适用 | — | — | — | 数秒 | 不适用 |
| 仓库外探针 suite #1 / #2 / #3（49 + 39 + 8 例） | 原始观测，非 PASS/FAIL 判定 | 不适用 | — | — | — | 392 s / 73 s / 8 s | 21 s / 0 / 0 |
| 直接驱动 av-worker 的微探针约 20 个 | 原始观测 | 不适用 | — | 每个 0.04–0.5 s | — | — | 21 s / 42 s / 87 s / 0（另 254 s 等待后放弃一次） |
| 仓库外探针 crate 构建（dev profile） | 成功 | 32.4 s（全量依赖）+ 1.43 s | — | — | — | — | 335 s + 401 s |
| 探针 crate 批次（20 个） | 10 个完成；8 个因机器内存耗尽作废；`csv-encode-cells-*` 2 个未完成（批次被停） | — | — | 快项 <2 s；`json-nodes-65535` 1646 s、`json-nodes-65536` 1759 s | — | 20:01:51 起至 20:58 停止 | 274 s |

未执行：`cargo t all`、`ace-perf`、六个冻结组整组、Linux/WSL 用例。构建档位：nextest 为 `test` profile；工具准备的实际 argv 是 `cargo build --locked -offline -p … --bin …`，三件工具均为 `debug`（`scripts/test-governance/toolchains.json`），没有 `--release`。

**计时表：本计划新增用例逐例计时**（全部在 10 秒门内）

| 新增用例 | 测的行为与 oracle | profile | 实际秒数 | 结果 |
|---|---|---|---|---|
| `plan1124_json_csv::v01_json_dynamic_tree_keys_dup_types_and_errors_released` | 树访问、重复键后成员胜出、缺键对 null、`minify(prettify(x))` 往返、3 个捕获错误、收尾 arena 零泄漏 | cargo test | 0.498 | PASS，通过 |
| `…::v01_json_depth_and_budget_limits_fail_safely` | 深度 127 成功、130 失败；超过 1 MiB 的输入 `is_valid` 为假且 `parse_strict` 报 budget；零泄漏（`max_steps` 提到 10M） | cargo test | 0.845 | PASS，通过 |
| `…::v01_and_v02_packages_cross_module_json_schema_and_rename` | 跨模块 `JsonEncodable` 编码，经自家解析器核字段与改名；零泄漏 | cargo test | 1.608 | PASS，通过 |
| `…::v02_json_encodable_negative_controls_and_non_finite_float` | 未遵从记录、`char` 字段、嵌套未合规的具名拒绝；inf 触发 `JsonError` | cargo test | 0.736 | PASS，通过 |
| `…::v03_csv_rfc4180_delimiters_roundtrip_and_error_cleanup` | 引号、换行、空字段、`;` 与 `\t` 分隔符、4 个错误后零泄漏、包导入 | cargo test | 0.564 | PASS，通过 |
| `…::v02_single_file_widths_empty_custom_override_and_forged_std_rejected` | 空记录、自定义覆写、各宽整数；伪造 `package std` 被拒 | cargo test | 0.534 | PASS，通过 |
| `ace-core stdlib::tests::auto_json_and_csv_inputs_match_frozen_sources` | 冻结身份与篡改负控 | cargo test | 0.022 | PASS，通过 |
| `ace-core stdlib::tests::auto_json_and_csv_parse_cleanly_in_packages_and_single_file_profiles` | `json.at` + `json_auto.at` 与 `csv.at` 在两种 profile 下可解析 | cargo test | 0.255 | PASS，通过 |
| `av.parity.1124.json-dynamic-tree`（exit 42） | 动态树与错误路径 | av-worker debug（已准备） | `request_wall_ms` 1303.85（`step.wall_ms` 540.88） | PASS，通过 |
| `av.parity.1124.json-encodable-schema`（exit 42） | 泛型编码经自家解析器回读 | 同上 | 1074.11（561.35） | PASS，通过 |
| `av.parity.1124.json-csv-packages`（exit 42） | 跨模块，`use std.csv` 与 `std.encoding.csv` | 同上 | 1468.74（781.52） | PASS，通过 |
| `av.parity.1124.csv-rfc4180`（exit 42） | RFC-4180 解析与编码 | 同上 | 716.98（228.37） | PASS，通过 |

- 新增测试义务就是上表 12 项，没有参数化输入或包装子项。计划 §8 里“单测 ≤0.3s”的说法与实测不符，`v01_and_v02_packages…` 实测 1.608 s。
- 沿可达调用路径没有发现日常测试触发编译：`PackageAvTarget::cold` 只做进程内降落、`ManagedStore::open` 和持久化；av-worker 与 auto-case 由显式的准备阶段构建；普通入口要求已验证的 prepared manifest，缺失或漂移时返回具名 `PREPARATION_REQUIRED`，不回退重建。
- 断言强度：`prettify` 的精确字节、`as_array()`、浮点与宽整数的文本输出都没有断言；合成编码只经自家解析器回读；浮点只测了 `1.5`、`2.25`、`1.5f32`。

**规范增量结论**（SD-01，modify `docs/specs/ace-migration/m2-stdlib-data.md`）

方向正确，把 975 的“M2-B 边界”条目换成了当前交付的公共合同，多数条文与实测一致。需修正后再沉淀：

1. “拒绝…多字节分隔符”应改为“拒绝非单个 Unicode 标量的分隔符”。
2. “`1 MiB`（`1_048_576` B）输入”的单位是标量，不是字节。
3. “严格遵循 RFC-4180”不准确：编码器输出 LF 行尾，解析器接受单独的 CR。
4. “用户显式 `fn json_encode() Str` 优先保留”要写明 `List` 元素例外，或由实现消除例外（F-04）。
5. 缺少计划冻结合同里的 CSV 预算、“空串返 `"[]"`”、“闭引号后杂字符”。
6. 没有写数值的范围与精度行为（F-01、F-08），也没有写产品默认限额下预算不可达。
7. “完整的动态 JSON 树”说得过满，树是只读访问面。

frontmatter：`supersedes_spec_components` 的路径存在且准确；`new_spec_components: []` 是因为只修改现有文件，merge 时需写明理由；`touched_goals: [GOAL-002]` 是真实 ID（`docs/specs/goals.md`）。

merge 事项：

- 规范里指向 `../../plans/1124-av-stdlib-json-and-csv.md` 的链接在归档时需改指 `plans/archive/`。
- 主检出 `docs/plans/KNOWN-DEBT-AND-RISKS.md` 第 4738 行（动态 `JsonValue` 与泛型 `encode[T]` 边界）与第 4739 行（`encoding/csv.at` 未迁移占位）需更新或关闭；第 4622 行 D-948-04 的命名漂移已被 `.type_of()` 的决定覆盖一半。行号以 master `048698155` 为准。
- 主检出 `docs/design/strategy/av-autovm-language-parity.md` 第 275 行的链接在归档后需要改。

**留给 merge 的 pending 门**（在冻结提交上执行并核对原始结果，本次复审没有整组跑）

| 组 | 选择范围 |
|---|---|
| `stdlib` | 根 workspace，legacy 域，`-p auto-lang -p auto-lang-core --lib`，按 `stdlib_assembly`、`file_session`、`vm::ffi::stdlib::`、`tests::stdlib_tests`、`plan738_*`、`plan872_*` 过滤 |
| `ace-stdlib` | `-p ace-stdlib-conformance --test conformance` |
| `ace-source` | `-p ace-core --lib --tests` |
| `ace-semantics` | `-p ace-semantics-contract -p ace-semantics -p ace-semantics-store -p ace-source-query --lib --tests` |
| `ace-av` | `-p ace-av-core -p ace-av --lib --tests --features ace-av/test-case-batch` |
| `case-av-language-parity` | AV 视图，`language.suite.ad` 共 555 个 RUN 检查 |

- 复审重放 `ace-av` 的 conformance 时没有带 `ace-av/test-case-batch`，冻结组会带。
- O-5 涉及三个冻结 profile，由 `ace-source` 兜底。
- Linux 与 WSL 用例按裁定排除。
- 未验证的探针（CSV 单元格预算、恰 1 MiB 侧）由 work 在修复轮一并补上断言。

**路由**：`status: executing`；T-01（仅冻结记录补全）与 T-02..T-04 回开，`current_step` 重算为 0；`plan_revision` 不变。下一步是 work，按 F-01..F-12 修复并补永久断言，优先 F-01、F-02、F-03、F-05、F-04，然后在新提交上重审受影响的 AC。`resolve.rs`、`project.rs`、`instances.rs` 的改动经读码与子集重放未见回归，最终由冻结组并集把关。

stage: goal | plan_id: PLAN-1124 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: waiting | worktree: D:/autostack/.wt/lang-1124/auto-lang | commit: d206320e8 | next: review（只读对账发现证据缺口；用户暂停目标，未写树、未起构建）

stage: goal | plan_id: PLAN-1124 | plan_revision: r1 | round: 3 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1124/auto-lang | commit: d206320e8 | next: work（按 F-01..F-12 修复并补永久断言；记录时目标仍处于用户暂停，未开始 work）

2026-11-24 · stage: work · PLAN-1124 r1 · 修复轮 1（对应复审 r1 的 `needs_fix`，受审基线 `d206320e8`）。修复提交 `9a9c296ec`（worktree `D:/autostack/.wt/lang-1124/auto-lang`，分支 `plan-1124-dev`，基面 master `8a52d7364`；记录时主检出 HEAD 为 `1ef4c0b7d`，merge 时变基）。next: review（T-05，在新提交上重审受影响的 AC）。

**修复轮 1 的范围与结果**。F-01..F-12 全部修复并落到永久断言；修复中自查出新缺陷六项，一并关闭（见下表之后的“修复轮自查”）。当前代码上的验证：`ace-core --lib`（嫁接与 stdlib 单元）9/9、`ace-av --lib`（worker 单元）1/1、`plan1124_json_csv` 一致性 26/26、近邻 `json975`/`plan1120`/`plan1121`/`module_v_1064` 54/54、语言用例 `av.parity.1124.*` 8/8 PASS；逐例计时见后。仓库外数值 oracle 四个种子共 34,629 个输入，0 个不一致。

**发现关闭表**（发现 → 不变量 → 受影响路径 → 修复与永久用例 → 当前结果 → 余留缺口）

| 发现 | 不变量 | 路径 | 修复与永久用例 | 当前结果 | 余留缺口 |
|---|---|---|---|---|---|
| F-01 | 数值文本读回为逐位相同的 `f64`；记录里的 `f64`/`f32` 字段走同一条文本路径；控制台打印语义不变 | `json_auto.at` 的 `__json_f64_str`、`__json_serialize`；`json_encode_synth.rs` 的浮点字段；`minify`/`prettify`/`encode_pretty` | `__json_f64_str` 改为精确整数区间上的最短往返十进制；`minify`/`prettify` 改为文本级重排（`__json_reserialize`），数字文本原样保留。用例：`f01_f08_review_counterexamples_read_back_exactly`（复审反例及同类 48 个，文本与读回位逐项对照）、`f01_formatters_keep_the_number_text`（精确字节）、`f01_minify_round_trip_keeps_every_double`（运行期 `parse_strict(minify(x))`、`parse_strict(prettify(minify(x)))` 与 `parse_strict(x)` 的 `as_number()` 相等）、`f01_f02_f03_record_fields_print_exactly`、语言用例 `numbers-exact` | 全部通过 | 无 |
| F-02 | 精确 `i32` 节点（含最小值）输出精确文本，不产生 trap | `json_auto.at` 对冻结 `json_int_str` 的调用 | `__json_int_text` 直接给出最小值文本；字面量 `-0` 用节点标志保留符号。用例：`f02_i32_minimum_and_negative_zero_serialize`（`minify`/`prettify`/`json_encode`/`encode`/`encode_pretty`）、上一行的记录字段用例 | 全部通过 | 无 |
| F-03 | `encode_pretty(x)` 与 `encode(x)` 只差空白 | `encode_pretty` | `encode_pretty(v) = __json_reserialize(v.json_encode(), 1)`。用例：`f01_f02_f03_record_fields_print_exactly`（超过 2^53 的 `u64`/`i64`、`i32` 最小值、近整数小数；并断言 pretty 去空白后与 compact 逐字节相等）、语言用例 `override-list` | 全部通过 | 无 |
| F-04 | 手写的 `fn json_encode() Str` 在顶层、嵌套字段、`List` 元素三处都被调用 | `json_encode_synth.rs` 的元素视图 | 含覆写的记录作 `List` 元素时调用元素视图上的 `.json_encode()`。用例：`f04_override_holds_for_list_elements`（`packages` 面与单文件面）、语言用例 `override-list` | 全部通过 | `List` 元素是视图：覆写体若需要整个容器字段，调用在绑定期被语言既有的 `source.view-escape` 拒绝（D-1124-T01-9、规范已写明） |
| F-05 | 不引用库的程序保持基面行为；库不重绑程序自己的名字；保留名冲突是普通的重复定义诊断 | `adapter.rs` 嫁接触发与合并、新文件 `stdlib_refs.rs` | 按程序的真实引用嫁接（穷举 AST 的引用/声明分析）；自由函数改名到保留命名空间、公开名转发；被程序遮蔽的公开入口不附加；程序声明了 `JsonValue` 时库记录改名；恢复 `_ => unreachable!`；保留名冲突的重复定义诊断指向程序自己的声明（list 嫁接基面原本指向嫁接文本）。用例：单元 `stdlib_json_csv_graft_follows_references`、`stdlib_json_csv_graft_keeps_the_programs_own_names`、`stdlib_module_graft_shape`；一致性 `f05_graft_follows_references_and_keeps_the_programs_names`（复审反例逐条）；语言用例 `graft-own-names` | 全部通过。O-1 的回归消失：注释含 “encode” 的 hello-world 经 worker 36.7 ms、字符串含触发词 37.5 ms，基线 36.8 ms（复审时 456 ms / 530 ms） | 无 |
| F-06 | 缩进布局冻结；格式化输出有精确字节断言 | T-01 冻结记录、规范 | D-1124-T01-6。用例：`f06_pretty_bytes_and_as_array_usability`（完整字节、树与文本两个格式化入口一致、`as_array()` 经 `remove` 可用）、`f01_f02_f03_…`、语言用例 `pretty-bytes` | 全部通过 | `as_array()` 的自然用法受语言既有的 `verify.view-escape` 限制（复审 O-9），规范已注明 |
| F-07 | 预算按字节计，每个文本入口都检查；节点与单元格预算两侧可观察；捕获后零泄漏 | `json_auto.at` 的 `__json_admit`、`csv.at` 的两处字节检查与分隔符预扫 | 单位改为 `len_bytes()`；输入达到 131_071 字符时先数分隔符，超限即在构建前拒绝；预扫按短片段读取、数到预算即停。用例：`f07_json_byte_budget_on_every_entry`、`f07_csv_byte_budget_on_every_entry`（1_048_576 字节通过、1_048_577 字节每个入口拒绝）、`f07_node_budget_refuses_the_first_excess`、`f07_node_budget_admits_the_last`、`f07_cell_budget_refuses_the_first_excess`、`f07_cell_budget_admits_the_last`；每个用例的退出都核对 arena 账本 | 全部通过（2.2–6.6 s，见计时表） | 默认 `ExecLimits` 下计数预算仍不可达（平台事实，规范已写明）。墙钟超线性的根因在 `ace-av-core`，不在本计划范围：见下文与 §10 |
| F-08 | 数值解析正确舍入，覆盖整个有限范围 | `json_auto.at` 的 `__json_parse_f64` 与大整数辅助函数 | 18 位以内且精确的快速路径，否则 1e9 进制 limb 大整数、单次舍入，最多考察 800 位有效数字。用例：`f08_exact_midpoints_round_to_even`（42 个，含 2^52、2^53 处的不对称区间）、`f08_exact_midpoints_near_1e300`、`f08_exact_midpoints_at_the_smallest_normal`、`f08_exact_midpoints_among_subnormals`（各 6 个，文本由精确有理数生成）、`f08_finite_range_and_long_digits`、`f08_shortest_text_breaks_ties_to_even` | 全部通过；仓库外 oracle 与 Python `float()` 逐位一致 | 无 |
| F-09 | 测试每次运行后不残留临时目录 | `plan1124_json_csv.rs` 的 `run_pkg_value_released` | 先释放 `PackageAvTarget` 再删目录，删除失败即 panic | 修复后的多轮运行 `%TEMP%\p1124-*` 目录计数保持 30（均为修复前残留，merge 收尾时清除） | 无 |
| F-10 | 本计划新增行通过 `rustfmt --check`；不新增 lint 豁免；不丢弃必须检查的返回值 | 改动的 `.rs` | 对本计划新增的行逐块格式化（既有差异不动）；三处 `let _ = synthesize…?` 改为检查返回值；没有 `allow(clippy::…)`、`todo!`、调试打印 | 本计划改动的文件上编译器警告 0 条 | 无 |
| F-11 | 未捕获的库错误与 trap 的 span 不超出用户源；检查器给出被判定的诊断而不是 infra | `language_source_worker.rs` | 嫁接区内的 span 不上报；并按 MIR 的调用图，把诊断定位到程序里进入该出错库函数的调用（源码顺序的第一处），检查器因此能正常判定。用例：worker 单元 `plan1124_uncaught_library_error_is_reported_at_the_call` | 通过 | 同一入口被调用多次时报告的是源码顺序上的第一处，不一定是对应报错的那一次（诊断文本本身说明错误内容） |
| F-12 | 规范文本与行为一致 | `docs/specs/ace-migration/m2-stdlib-data.md` | 复审列出的七点全部修正，并补上嫁接改名规则、数值文本的等距取偶、预算的入口与覆盖口径 | 已改，随本提交落盘 | 无 |

**F-07“墙钟超线性”的根因（复审未定位）**。`experimental/ace/crates/av-core/src/owned_text.rs` 的 `char_at`（第 264 行）与 `scalar_range_to_bytes`（第 236 行）每次调用都执行 `scalars(bytes)`，把整段文本收集成 `Vec<char>` 再取下标：单次 O(n)，逐字符扫描 O(n²)，而步数只是线性。这不是复审猜测的 `List` 写入核算。实测（测试档 debug 构建，`av-worker` 默认限额）：`is_valid` 处理 2^10 个空格 0.61 s、2^12 1.04 s、2^14 2.67 s、2^15 14.99 s、2^16 31.18 s（因步数预算终止）。解析器本身的这条路径没有在计划内改动——它在 `ace-av-core`，超出本计划的 `affects`；§10 当初把“在计划内修、另立计划还是调整阈值”留给用户裁定。现在的处理是：本计划不碰 `av-core`、不调阈值，只保证自己新增的预扫不走这条路径（按 256/1024 个标量的短片段读取，数到预算即停），六个预算用例因此都在 10 秒门内；平台层的修复登记为债务候选 D-1124-AV-TEXT-01（§10），默认另立计划，如需在本计划内一并修请指示。

**修复轮自查新发现并关闭的缺陷**（复审没有列出）

| 编号 | 缺陷 | 关闭方式 |
|---|---|---|
| N-1 | 最短文本在等距候选间没有取偶：oracle 种子 7 的 `253883181407725.140624` 输出 `.13`（应为 `.12`）。原因是取前 18 位时“被丢弃的数字是否全零”判断过宽，把整个 limb 非零当成有丢弃位 | 新增 `__json_dropped_nonzero` 精确判断；用例 `f08_shortest_text_breaks_ties_to_even`（20 个奇数八分点，期望值取自 Python `repr` 并冻结） |
| N-2 | 修复轮最初冻结的嫁接规则写的是“程序声明 `JsonValue` 且使用库时得到重复定义诊断”，那会让程序自己的类型挡住整个库，与 `packages` 面不一致 | 改为库记录整词改名为 `__std_json_JsonValue`；D-1124-T01-8 与规范已修订；单元断言、`f05` 与语言用例 `graft-own-names` 覆盖 |
| N-3 | 修复轮自己引入的“先数分隔符”最初逐字符读取，是平方级的：13 万字符的预扫让一个用例挂到 548 秒才结束 | 改为分块读取并数到预算即停（见 F-07）；同一类用例现在 2.2–6.6 s |
| N-4 | 保留名冲突的重复定义诊断落在嫁接文本内，经 worker 与检查器被判成 infra（list 嫁接基面同样如此） | 诊断指向程序自己的声明；worker 实测 `__json_admit`、`__std_list_map` 两个冲突的 span 都在 0..源码长度之内 |
| N-5 | “量级极端中点”用例的文本在生成时混进了换行，上一轮在“文本须是数字”处崩溃 | 用精确有理数重新生成，每个文本先在 Python 里核对落在两个相邻 double 的正中或其两侧一位，再拆成三个测试 |
| N-6 | 冻结记录曾写“等距取偶与 Rust 的 `{}` 一致”，这是错的：Rust 在等距时向上入（`253883181407725.13`） | D-1124-T01-5 已更正；平局测试不再以 Rust 的数字为 oracle |

**计时表：本会话实跑范围**（测试档 `test` profile，未优化；仓库外 oracle crate 为 dev profile；没有 `--release`）

| 范围 | 结果与执行/排除数 | 准备 | 测试 | 占用等待 |
|---|---|---|---|---|
| 仓库外 oracle crate 重建（dev profile） | 成功 | 6 s | — | 0 |
| oracle 种子 1124 与 7（并行，各 8 片） | 8784 + 8480 个输入，0 不一致 | — | 445 s / 426 s | 0 |
| oracle 种子 23 与 99（并行，各 8 片） | 8633 + 8732 个输入，0 不一致 | — | 424 s / 427 s | 0 |
| `ace-core --lib` 嫁接单元（`stdlib_list_graft_tests`） | 5/5 PASS，89 跳过 | 含在 2 s 内 | 0.70 s，最慢 0.69 s | 0 |
| `ace-core --lib` `stdlib::tests` | 4/4 PASS，90 跳过 | 含在 1 s 内 | 0.29 s | 0 |
| `ace-av --lib` worker 单元 | 1/1 PASS，32 跳过 | 12 s | 0.19 s | 0 |
| `ace-av` conformance `plan1124_json_csv` | 26/26 PASS，1478 跳过 | 含在 47 s 内 | 42.6 s，最慢 6.62 s | 0 |
| 近邻 `json975`、`plan1120`、`plan1121`、`module_v_1064` | 54/54 PASS，1450 跳过 | 含在 4 s 内 | 3.9 s，最慢 3.85 s | 0 |
| 工具准备 `fixed-tests.py case-av-language-parity --prepare-only` | PREPARED | 25 s | — | 0 |
| 语言用例 `av.parity.1124.*` × 8（检查器逐例 `--case`） | 8/8 PASS | 复用上项 | 步墙钟 0.64–1.69 s，见下表 | 0 |
| worker 探针 8 个（嫁接、保留名、复审 O-1 与 O-4） | 原始观测，非 PASS/FAIL 判定 | — | 每个 36–359 ms | 0 |

最终验证轮在提交 `9a9c296ec` 的源码上执行，总墙钟 04:48:03–04:50:05（122 s）；整条链（oracle 重建、四个种子、验证集）04:33:21–04:50:05。占用等待：本轮在此之前的三次构建与测试曾因其他会话（PLAN-1129 等）的测试在门前排队，单次约 1–10 分钟，未逐次累计。清理：测试不需要清理步骤，`%TEMP%` 目录计数不变。

**新增用例逐例计时**（全部在 10 秒门内；最终验证轮的 nextest 实测）

| 新增用例 | 测的行为与 oracle | 实际秒数 | 结果 |
|---|---|---|---|
| `f07_node_budget_admits_the_last` | 65,536 个值通过节点预算并进入解析器（首字符非法）；1,025 个值的合法文档完整解析 | 6.62 | PASS |
| `f07_cell_budget_admits_the_last` | 65,536 个单元格通过单元格预算并进入解析器；1,025 个单元格的行集解析并编码 | 4.83 | PASS |
| `f07_cell_budget_refuses_the_first_excess` | 65,537 个单元格在构建任何单元格前被单元格预算拒绝 | 4.12 | PASS |
| `f07_json_byte_budget_on_every_entry` | 1,048,577 字节在四个文本入口被字节预算拒绝；1,048,576 字节通过字节检查、由节点预算拒绝 | 3.09 | PASS |
| `f07_csv_byte_budget_on_every_entry` | 1,048,577 字节在 `csv_parse`、`csv_encode` 被拒；1,048,576 字节由单元格预算拒绝 | 2.74 | PASS |
| `f07_node_budget_refuses_the_first_excess` | 65,537 个值在构建任何节点前被节点预算拒绝 | 2.19 | PASS |
| `f08_exact_midpoints_at_the_smallest_normal` | 最小正规数及其下邻（奇数尾数）的中点及其两侧一位 | 2.00 | PASS |
| `f08_exact_midpoints_among_subnormals` | 次正规数里的中点及其两侧一位（千位数字） | 1.85 | PASS |
| `v01_and_v02_packages_cross_module_json_schema_and_rename` | 跨模块 `JsonEncodable` 编码经自家解析器回读；字段改名自动反映 | 1.84 | PASS |
| `f01_f08_review_counterexamples_read_back_exactly` | 复审 F-01/F-08 的反例及同类 48 个：文本是最短往返形式，读回的位与正确舍入结果逐位相等 | 1.78 | PASS |
| `f01_minify_round_trip_keeps_every_double` | 19 个文本经 `minify` 与 `prettify` 后读回的 double 与原文相同 | 1.18 | PASS |
| `f04_override_holds_for_list_elements` | 手写的 `json_encode` 在顶层、嵌套字段、`List` 元素三处文本一致（`packages` 面与单文件面） | 1.17 | PASS |
| `f08_exact_midpoints_round_to_even` | 42 个恰好落在两个 double 正中及其两侧一位的十进制文本，平位取偶，含 2^52、2^53 处的不对称区间 | 1.16 | PASS |
| `f05_graft_follows_references_and_keeps_the_programs_names` | 复审 F-05 的反例逐条；保留名冲突是 `source.duplicate`；程序自己的 `JsonValue` 与库共存 | 1.15 | PASS |
| `v01_json_depth_and_budget_limits_fail_safely` | 深度 127 成功、130 失败；超过 1 MiB 的输入被字节预算拒绝；零泄漏 | 0.88 | PASS |
| `f08_exact_midpoints_near_1e300` | 1e+300 附近的中点及其两侧一位（300 位数字） | 0.83 | PASS |
| `v02_json_encodable_negative_controls_and_non_finite_float` | 未遵从记录、`char` 字段、嵌套未合规的具名拒绝；inf 触发 `JsonError` | 0.74 | PASS |
| `f08_shortest_text_breaks_ties_to_even` | 20 个等距最短候选取末位偶数；期望值是冻结的 Python `repr` | 0.71 | PASS |
| `f01_f02_f03_record_fields_print_exactly` | 记录的 Str、int、f64、?Str、i64、u64、List 与嵌套记录字段输出精确文本；pretty 去空白后与 compact 逐字节相等 | 0.61 | PASS |
| `f08_finite_range_and_long_digits` | 超出有限范围被拒、下溢为 0、40 位数字 | 0.59 | PASS |
| `v02_single_file_widths_empty_custom_override_and_forged_std_rejected` | 空记录、自定义覆写、各宽整数；伪造 `package std` 被拒 | 0.51 | PASS |
| `v01_json_dynamic_tree_keys_dup_types_and_errors_released` | 树访问、重复键后成员胜出、缺键对 null、`minify(prettify(x))` 往返、3 个捕获错误；退出时 arena 账本为零 | 0.44 | PASS |
| `f01_formatters_keep_the_number_text` | `minify`、`prettify` 保留原文数字的精确字节 | 0.42 | PASS |
| `f06_pretty_bytes_and_as_array_usability` | 缩进布局的完整字节；树与文本两个格式化入口一致；`as_array()` 经 `remove` 可用 | 0.41 | PASS |
| `f02_i32_minimum_and_negative_zero_serialize` | i32 最小值与字面量 `-0` 在五个入口的精确文本，不产生 trap | 0.39 | PASS |
| `v03_csv_rfc4180_delimiters_roundtrip_and_error_cleanup` | 引号、换行、空字段、`;` 与制表符分隔符、4 个错误后零泄漏 | 0.34 | PASS |
| `ace-core` 单元 `stdlib_json_csv_graft_follows_references`（0.06 s）、`stdlib_json_csv_graft_keeps_the_programs_own_names`（0.69 s）、`stdlib_module_graft_shape`（0.03 s） | 引用分析、嫁接触发、程序自己的名字与保留名冲突、嫁接文本的形状 | 见括号 | PASS |
| `ace-av` 单元 `plan1124_uncaught_library_error_is_reported_at_the_call`（0.19 s） | 未捕获的库错误定位到程序的调用，不超出提交的源 | 0.19 | PASS |
| 语言用例 `av.parity.1124.json-dynamic-tree`（exit 42） | 动态树与错误路径 | 1.50 | PASS |
| 语言用例 `av.parity.1124.json-encodable-schema`（exit 42） | 泛型编码经自家解析器回读 | 1.45 | PASS |
| 语言用例 `av.parity.1124.json-csv-packages`（exit 42） | 跨模块，`use std.csv` 与 `std.encoding.csv` | 1.69 | PASS |
| 语言用例 `av.parity.1124.csv-rfc4180`（exit 42） | RFC-4180 解析与编码 | 0.64 | PASS |
| 语言用例 `av.parity.1124.numbers-exact`（exit 42） | 数字读回逐位相等、最短文本；i32 最小值与 `-0` | 1.59 | PASS |
| 语言用例 `av.parity.1124.pretty-bytes`（exit 42） | 缩进布局的完整字节；`as_array()` 可用 | 1.36 | PASS |
| 语言用例 `av.parity.1124.graft-own-names`（exit 42） | 程序自己的 `parse`、`json_cp`、`JsonValue` 与库共存 | 1.37 | PASS |
| 语言用例 `av.parity.1124.override-list`（exit 42） | `List` 元素走用户的 `json_encode`；覆写记录的缩进布局 | 1.27 | PASS |

**复用的证据与未执行的范围**

- 复用：复审 r1 的反例清单作为重放集（`f01_f08_…`、`f05_…`、`f04` 直接取自复审的探针输入）；仓库外 oracle 首轮 6333 个输入的结论由修复后的四个种子重新建立，不沿用。
- 未执行（留给 T-06 在冻结提交上跑）：六个冻结组整组（`stdlib`、`ace-stdlib`、`ace-source`、`ace-semantics`、`ace-av`（带 `ace-av/test-case-batch`）、`case-av-language-parity`）、`cargo t all`、`ace-perf`。Linux/WSL 用例按 2026-11-14 裁定排除，归专用 Linux 环境。
- 本轮只跑了受影响的组与近邻，没有为修复反复跑全量。

**当前身份**：worktree `plan-1124-dev`，修复提交 `9a9c296ec`（基线 `d206320e8`）；依赖为主检出同源的 workspace（`--locked --offline`）。

stage: work | plan_id: PLAN-1124 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1124/auto-lang | branch: plan-1124-dev | base_commit: d206320e8 | code_commit: 9a9c296ec | dependency_revisions: worktree 基面 master 8a52d7364（记录时主检出 HEAD 1ef4c0b7d，merge 时变基） | task_ids: T-01, T-02, T-03, T-04 | evidence: 本节发现关闭表与计时表；plan-artifacts/plan-1124（仓库外 oracle 与探针，merge 时按规则删除） | blockers: 无 | next_responsible_role: review

stage: goal | plan_id: PLAN-1124 | plan_revision: r1 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1124/auto-lang | commit: 9a9c296ec | next: review

2026-11-24 · stage: review · review_scope: repair · status: in-progress（进行中的标记，不是收据）。独立复审子代理 04:56 启动，07:47 因账号使用上限中断，08:37 带原上下文续跑；受审提交 `9a9c296ec`；证据目录 `D:/autostack/plan-artifacts/plan-1124/review2/`。复审记录与路由收据待其返回后追加在本行之后；若此时会话再次中断，续接的轮次应先读该目录里已有的证据，再决定重审还缺什么，不要从头重复。

2026-11-24 · stage: review · review_scope: repair · PLAN-1124 r1 · outcome: needs_fix（受审提交 `9a9c296ec2733dfc93c0fcaf9cd6928a2e207469`）。

实现方的测试全部重放通过，数值与字节预算这两块最重的合同经独立反例成立；但有三项合同内问题被复审自己的反例确认，另有四项低级问题，其中一项是修复记录声称已关闭而实测未关闭。next: work。

stage: review | review_scope: repair | reviewer_identity: 发起会话派生的子代理（全新上下文，未参与实现与 r1；环境模型标识 GLM-5.4；无会话 ID）| plan_id: PLAN-1124 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1124/auto-lang | branch: plan-1124-dev | reviewed_commit: 9a9c296ec2733dfc93c0fcaf9cd6928a2e207469 | baseline_commit: d206320e86ecc6bb7f7d4ba0b9c61089201df8f7 | base_commit: 8a52d7364e5b218e0a08b167038a5d71d54ee72f | dependency_revisions: 主检出同源 workspace（`--locked --offline`）；master 现值 `7bb90b1f77b8210ccebe3c1562aa053a9132aa1a` | spec_inputs: docs/specs/ace-migration/m2-stdlib-data.md @9a9c296ec blob `f320d532cd8f955d99ead00cd5ef113005f44a8c`，sha256 `a6837ee27ea1afe796fd90161d3dacd6e551a70dfde33feaa0ae09cc1d0dcef0`（r1 所审 `83bd7a3ed…`，基面 `8b32fe786…`）| acceptance_results: AC-01 partial · AC-02 partial · AC-03 fail · AC-04 pending(merge) | pending_closeout_gates: 变基并解决冲突、六个冻结组整组并集 | findings: F-13..F-19 | evidence: `D:/autostack/plan-artifacts/plan-1124/review2/`（索引 `out/EVIDENCE-INDEX.txt`；下列内容为持久摘要）| next: work

**复审 r2 的基线核实**

- 实施树进出各查一次 `git status --porcelain` 均为 0 行，收尾时 HEAD 仍为 `9a9c296ec`；修复 diff 13 个文件 +3980/−555，整计划 21 个文件 +7094/−41。
- `stdlib/ace/std/json.at` 的 blob 与基面相同（`95b6c561ec…`）；diff 未触及 `.autoos/`、`specs.json`、`index.json`、`KNOWN-DEBT-AND-RISKS.md`、`.retention`、`docs/plans/`；worktree 组目录内 reparse point 为 0。
- 提供的仓库外 oracle 经闸门重建：Finished 0.09 s、无重编译，即该可执行文件与 `9a9c296ec` 的源码一致。
- master 较基面多 19 个提交，其中 `249e836b2`（PLAN-117）触及本计划的文件。干跑 `git merge-tree --write-tree master 9a9c296ec`（当时 master `f083f403b`）报 `experimental/ace/crates/av-core/src/av.rs` 内容冲突；其后 master 只多了 3 个改 `docs/plans` 的提交，未触及本计划文件。
- 主检出的 Plan 文件在复审期间只多了一行“review r2 进行中”的标记，被审的主张没有变化。

**复审 r2 的审查局限**

- 基面没有实跑（没有基面 worktree，技能不允许为复审另建检出）。凡写“基面如何”处都是读 diff 的推断。
- 复审自己的探针和核对脚本出过多处错（Auto 字面量转义、把关键字 `tag` 当字段名、不合法的 `const`/`List::new(1, 2)`/列表 `clone` 写法、把 512 行当成 700 行、被自己的步数上限截断）。三个核对脚本（rustfmt 归因、警告归因、数值核对）的初版都会给出假结果，都是由对照发现、修正后才采用。记录里只引用更正后的结果；作废的输出列在 `out/budget/README-which-outputs-stand.txt`。
- 自建 harness 不是产品路径。它与仓库外 oracle 是同一批 crate 的 dev 构建，只提高了 `max_steps`（两个探针另提高了 `max_slot_bytes`）。产品路径的证据来自真实 av-worker、nextest 与检查器的重放。
- 未验证的项：存储值的零符号（语言内观测不到，除零 trap，`to_str` 不显示符号）；`csv_*` 公开名作局部名（只有读码推断，没有实测金丝雀）；`uint`/`float`/`double` 别名字段与深度 128/129 边界（沿用 r1 与 Plan 用例，未重测）。
- 未完成即放弃的探针：JSON 转义串的完整解析（三遍平方扫描超过复审的 400M 步上限，改用快速预扫探针）；`csv_encode` 对 65,537 个单格行的 JSON（被复审主动停止，由 700×100 的运行取代）。
- 占用：每次 cargo 前都经闸门，等待 0–1 s，机器空闲。计时轮（05:22:28–05:23:12）期间复审没有任何其他进程；此后它自己的并行负载最高 22 个 oracle 进程（20 逻辑核）。
- 中断：后台任务 1 小时上限杀掉了 `midpoints_uniform` 的首次运行（无结果），拆成两半重跑完成。账号用量上限造成一次会话中断（07:47–08:36）；中断前最后一个探针在 07:52:47 自行完成并落盘，续跑后只读取。

**复审 r2 的验收结果**

| AC | r1 | r2 | 要点 |
|---|---|---|---|
| AC-01 | fail | partial | V-01 数值、树与格式化成立；V-02 的 List 位置覆写只对全标量记录成立（F-14）；V-03 语义成立，预算见 F-15 |
| AC-02 | partial | partial | 近邻 54/54；字节预算全部入口成立；嫁接触发对局部名误判（F-13）；CSV 预扫漏 CR（F-15） |
| AC-03 | fail | fail | 复审 r2 记录即执行结果 |
| AC-04 | pending | pending | 归 merge |

**复审 r2 的发现清单**（severity 沿用 r1 的口径）

- **F-13（中，新回归；AC-02、T-02/T-04、D-1124-T01-8；F-05 的余留）单文件嫁接把局部名字当成引用。**
  - 不变量：不引用库的程序保持基面行为；规则是“程序引用了模块的公开名且未自行声明该名时才嫁接”。
  - 实测（真实 av-worker，各 5 次取中位数）：hello-world 41.1 ms；注释含 encode、minify 等 40.9 ms；字符串含触发词 43.4 ms；局部变量 `let parse = 1` 597.2 ms；局部变量 `let is_valid = true` 588.6 ms；参数 `helper(parse int)` 571.9 ms；嵌套 `fn parse` 569.5 ms；闭包 `var parse = fn …` 600.8 ms。
  - 金丝雀（程序声明 `fn __json_admit` 而不使用库）：单独存在时编译并运行（41.8 ms）；加上名为 `parse`、`encode`、`is_valid`、`minify`、`prettify`、`parse_strict`、`JsonValue`、`JsonEncodable` 的局部变量，或名为 `parse` 的参数，九种全部变成 `source.duplicate: function __json_admit is declared more than once`，span 0..38 指向程序自己的声明。基面没有 JSON 嫁接（读码），这类程序原本可编译。
  - 原因（已确立）：`stdlib_refs.rs` 的 `referenced_names` 把每个 `ExprKind::Var`（第 186 行）都记为引用；`declared_names`（第 32–48 行）只收顶层项。局部声明因此既算“引用”又不算“自行声明”。
  - 被误分类的位置：局部变量、参数、嵌套函数名、闭包绑定。程序自己的名字不被替换这一点仍成立：嵌套 `fn parse` 与库并用时 `parse(1)` 仍得 2。
  - 关闭 oracle：上述程序经 worker 的墙钟与 hello 同一量级；金丝雀程序编译并打印 7；`stdlib_json_csv_graft_follows_references` 的表里补上这四种位置的否定行，同时保留真实引用的肯定行。
  - 最小重放：`python -I worker_probe.py worker-src/g-local-parse.at`、`g-canary-json-local-parse.at`、`g-param-parse.at`（均在 `D:/autostack/plan-artifacts/plan-1124/review2/` 下）。
- **F-14（中，合同与实现不符且边界无用例；AC-01 V-02、T-03、D-1124-T01-9、SD-01 §2；F-04 的余留）带自有字段的记录作 `List` 元素时，手写覆写不被调用。**
  - 不变量（冻结合同）：“手写的 `fn json_encode() Str` 在顶层、嵌套字段和 `List` 元素三种位置都被调用。`List` 元素是视图：覆写体若需要整个容器字段，该调用在绑定期被语言既有的视图规则拒绝”。
  - 实测（单文件面与 packages 面逐行相同）：

    | 带覆写的元素记录 | 覆写体 | List 位置的结果 |
    |---|---|---|
    | `a int` | 读 `.a` | 正常，`{"tags":[[1]]}` |
    | `a int` + `name Str` | 只读 `.a` | `verify.view-escape` |
    | 同上 | 返回常量，什么都不读 | `verify.view-escape` |
    | 同上 | 读 `.name`、`.name.len()`、`.name.clone()`，或作实参 | `verify.view-escape` |
    | `a int` + `name Str`，无覆写 | （合成内联） | 正常，`{"tags":[{"a":1,"name":"n"}]}` |
    | 同一带覆写记录作普通字段 | 读 `.name`、`.name.clone()` | 正常，`{"one":[n]}` |

  - 实际规则：覆写记录只有在全部字段都是复制标量时才能作 `List` 元素，与覆写体读什么无关。诊断文本是 `a whole owned element is not a value; project a field or use remove`，诊断码是 `verify.view-escape`；规范与计划写的是 `source.view-escape`。
  - 原因（已确立）：`json_encode_synth.rs` 第 351–360 行对带覆写的元素合成 `self.<字段>.get(i).json_encode()`；`verify.rs` 第 3290–3315 行（本计划未改）对元素类型拥有自有数据的 `ListGet` 作值使用即报错，它不看方法体。
  - 覆盖边界：Plan 的两处用例（`f04_override_holds_for_list_elements` 与语言用例 `override-list`）都只用有一个 `int` 字段的 `Custom`。满足“全标量覆写记录”时，深度 2 嵌套、`?Item` 字段、整数极值、字符串转义的输出与 Python `json` 逐字节相同。
  - 关闭 oracle（二选一）：(a) 矩阵里被拒的六行在两面都输出覆写文本；或 (b) 规范与 D-1124-T01-9 写明实际规则与诊断码，并补两面的阳性（`a int`）与阴性（带 `Str` 字段）断言。
  - 最小重放：`p1124-review2-oracle.exe run pkg-matrix/single/m2-str-present-unread/prog.at` 与 `pkg pkg-matrix/pkg/m2-str-present-unread`。
- **F-15（中，合同内缺陷；AC-02 V-03、T-04、D-1124-T01-7、SD-01 §3；F-07 的余留）CSV 预扫漏数单独 CR，“构建前拒绝”只对部分输入成立。**
  - 不变量（规范原文）：“解析方向在输入达到 131_071 字符后先数分隔符（引号外的分隔符与换行，末尾换行不算），单元格超过 65_536 个即在构建任何单元格前抛 CsvError”。
  - 实测（65,537 格，只提高步数上限）：LF（131,074 字符）577 次分配、5.5 s；CRLF（196,611 字符）833 次分配、12.1 s；单独 CR（131,074 字符）131,651 次分配、1209 s；LF 32,768 行加 CR 32,769 行（131,074 字符）131,667 次分配、1200 s。四种都以 `CsvError: cell budget exceeded` 结束且账本平衡；带 CR 的两种先建满 65,536 格，墙钟约 220 倍。
  - 同一发现的另一半：`csv.at` 第 31–32 行的注释 “A cell takes at least two characters, so only long input can reach the cell budget” 不成立，空单元格只占一个字符。阈值以下同样先建后拒：65,536 个逗号（65,536 字符 / 65,537 格）65,598 次分配、211 s；`a,⏎` × 32,769（98,307 字符 / 65,538 格）98,366 次分配、581 s。
  - 对照（预扫正确的形状）：引号内的 65,536 个分隔符（CJK、emoji、`""` 转义各一次）都计 0，解析成 1 行 1 格；`"a"x` 加 65,536 个分隔符在 554 次分配时即拒，少一个分隔符则在 558 次分配后由解析器拒绝；`a,` × 65,535 加 LF（恰 65,536 格）被准许并解析完成。编码方向：`csv_encode` 对 700×100 格的 JSON 文本同样以 `cell budget exceeded` 结束（131,788 次分配、1540 s）；规范只对解析方向承诺预扫，这一点不违约。
  - 原因（已确立）：`__csv_separators`（`csv.at` 第 144–197 行）只数 `c == 10`；解析器却把单独 `\r` 当行尾（第 113–118 行）。
  - 影响边界：默认 `ExecLimits`（100 万步）走不到这条路径，对提高限额的宿主成立。
  - 关闭 oracle：CR、混合行尾、65,536 个逗号三种输入被拒时的分配次数与 LF 同量级；补用例并断言分配计数；改正规范与代码注释。
  - 最小重放：`p1124-review2-oracle.exe run probes/budget-csv-eol-cr.at 400000000`（约 20 分钟），与 `budget-csv-eol-lf.at`（约 6 秒）对照。
- **F-16（低，收尾缺陷；F-10）本计划新增行上仍有 4 处 rustfmt 差异，其中 1 处由修复提交引入。**
  - 实测（`rustfmt --check` 按新增行归因；归因脚本经两个对照验证）：`experimental/ace/crates/core/src/source/cst/project.rs:15159–15161`（`d206320e8`；即 r1 列的 15156）；`experimental/ace/crates/core/src/source/resolve.rs:18086–18089`（`d206320e8`）；`experimental/ace/crates/core/src/source/resolve.rs:18101`（`d206320e8`；即 r1 列的 18098）；`experimental/ace/crates/source-query/src/instances.rs:1218`（`9a9c296ec` 新增）。
  - 同一批 16 个文件里另有 256 处差异落在基面既有行上，不计在本计划头上。修复记录写的“对本计划新增的行逐块格式化”与 F-10“全部修复”与实测不符。
  - 其余健康项（实测）：编译器警告在构建日志 109 个位置里 8 个位于本计划改动的文件，0 个在新增行上；`todo!`、`unimplemented!`、新 lint 豁免、被忽略的测试、丢弃的 `let _ = …?` 均为 0；10 处 `eprintln!` 都在测试文件里作计时输出。
  - 关闭 oracle：`python -I fmt_attrib.py <worktree> 8a52d7364 <新提交> out.txt` 得 0。
- **F-17（低，诊断质量；F-11 的余留）未捕获的库错误可能把位置报到另一个入口的成功调用上。**
  - 实测：`is_valid("{}")` 成功后 `minify("{")` 失败，上报的 span 是 `is_valid("{}")`；同一入口先成功后失败，报第一处 `parse_strict("[1]")`；分支里的失败，报分支外的 `minify("[1]")`。13 个 span 探针里没有一个 span 落在提交源码之外，错误文本都正确。
  - Plan 写的余留是“同一入口被调用多次时报告源码顺序上的第一处”，比实际窄：实际是第一处能静态到达出错函数的调用，可以是另一个入口，也可以是没执行的分支。
  - 关闭 oracle：单元测试补“另一入口的成功调用在前”的用例并断言 span 落在失败调用内；或把实际规则写进计划与规范。
- **F-18（低，规范增量；F-12 的余留）SD-01 有五处与行为不符或留白。**
  1. 覆写规则与诊断码，见 F-14。
  2. CSV 预扫，见 F-15。
  3. “程序另行声明同名类型时 `encode` 的约束无法满足，诊断落在调用处”不属实。实测程序自带 `type JsonEncodable { v int }`（或同名 enum）并用库的 `encode`：编译并输出 `{}` 与 `{"x":1}`，没有任何诊断；没有用例覆盖这句。
  4. “131_071 字符（容纳 65,536 个分隔符的最短长度）”：行为属实，括号里的算术不对——65,536 个节点的最短合法文档是 131,073 字符，首个超限是 131,075。
  5. 留白两处：字面量 `-0.0`、`-0e5` 经树输出 `0`，字面量 `-0` 输出 `-0`（`parse_strict("[-0,-0.0]").json_encode()` 得 `[-0,0]`），规范没有写这种情形，按“读回得到同一个 f64”应得 `-0`；`csv_encode_rows` 没有单元格预算也没有字节预算（70,000 格全部编码，输出 139,999 字符），规范未提，而 D-1124-T01-7 写的是“CSV 的解析、编码入口生效”。
- **F-19（低，收尾簿记）材料区未登记。**
  - Plan §4 写“已登记 `.retention/policy.json`”，但 `D:/autostack/auto-lang/.retention/policy.json` 与 `D:/autostack/.retention/policy.json` 里都没有任何含 `1124` 的条目。按规则，未登记的在途目录 fail-closed。
  - 另：`%TEMP%` 下有修复会话留下的 `p1124-probe.rs`、`p1124-probe.err` 两个散落文件。

**复审 r2 对 r1 发现与自查缺陷的关闭结论**

| 发现 | 结论 | 复审自己的证据 |
|---|---|---|
| F-01 | 关闭 | 61,467 个输入的树文本与 ECMAScript、Python 逐字一致；`minify("  [ 1 , 2 ]  ")` 得 `[1,2]`，数字文本原样 |
| F-02 | 关闭 | edge 文件含 `-2147483648`，输出精确；`f02_…` 用例 0.389 s |
| F-03 | 关闭 | `u64` 最大值、`i64` 最小值、`9007199254740993` 精确；四份文档的 pretty 去空白后与 compact 逐字节相同 |
| F-04 | 对已测形状关闭，一般情形不成立 | F-14 |
| F-05 | 对注释、字符串、自有名字关闭；触发条件对局部名误判 | 注释 40.9 ms、字符串 43.4 ms；六个保留名冲突都是 `source.duplicate`，span 指向程序声明，不使用库时同样程序可编译；F-13 |
| F-06 | 关闭 | 四份文档的 `prettify` 与 Python `json.dumps(indent=2)` 逐字节相同 |
| F-07 | 单位与入口关闭；“构建前拒绝”对单独 CR 与稠密输入不成立 | F-15 |
| F-08 | 关闭 | 61,467 个输入 0 不一致，含 34,000 个中点族、7,157 个 2 的幂邻域、最长 3,152 位的文本 |
| F-09 | 关闭 | 复审整轮运行前后 `%TEMP%\p1124-*` 目录数 30 → 30 |
| F-10 | **未关闭** | F-16 |
| F-11 | 对已测情形关闭；跨入口归因有误 | F-17 |
| F-12 | r1 的七点已改；仍有五处不符或留白 | F-18 |
| N-1 | 关闭 | `ties` 文件 1,932 个输入 0 不一致；对照 oracle 把平位改向上时，708 处被复审的核对脚本抓出 |
| N-2 | 关闭 | 自有 `JsonValue` 与库并用正常；程序自带的 `JsonEncodable` 类型与库的 spec 并存 |
| N-3 | 关闭 | `json-node-65537` 1.98 s、299 次分配；`csv-cell-65537` 7.3 s、554 次分配 |
| N-4 | 关闭 | `fn __std_list_map` 加 List 使用得 `source.duplicate`，span 0..40 是程序的声明 |
| N-5 | 以独立证据确认 | 三个中点用例重放通过；复审自己的 34,000 个中点族输入全部一致。“文本已按精确有理数重新生成”这一过程性说法无法直接核验 |
| N-6 | 关闭 | T01-5 文本已更正；等距取偶对 Node 与 Python 成立 |

**复审 r2 对七个重点的结论**

1. 数值：通过，没有找到任何错误输入。独立 oracle 是 Node 的 `String(Number(text))` 与 Python 的 `float()`、`repr()`，两者在全部输入上一致。合同只有两处与 ECMAScript 不同：正指数写作 `e100` 而非 `e+100`；字面量 `-0` 保留符号。61,467 个输入分为 60,440 个数值、822 个越界、203 个畸形文本、2 个 `-0` 字面量，结果 0 不一致。覆盖面：全范围随机 double 与随机十进制文本；随机 double、2 的幂两侧、最小正规数、次正规数、范围顶部的精确中点及其 ±10⁻ᴺ（N 最大到展开末位之后 500 位）；900–3,152 位的长数字，非零位放在第 790–810 个有效位上，跨越 800 位截断；范围两端的越界阈值两侧。核对脚本自身经对照：未变异的假 oracle 得 0，变异后的假 oracle 被抓出 708 到 1,932 处不等。存储值的零符号未能观测；`-0` 与 `-0.0` 的文本差异见 F-18。
2. 预算：字节预算成立，节点与单元格预算的“构建前拒绝”有缺口。字节预算在 JSON 五个入口与 CSV 五个文本入口都按字节生效；合法的超限文档 `is_valid` 为 false、`parse` 为 None；多字节验证：524,292 个标量、1,572,868 字节的文档 0.13 s 即被拒。JSON 节点预算：65,537 个值在 299 次分配时被拒；恰 65,536 个值的同长度文本被准许、由解析器拒绝；逗号在字符串内（CJK、emoji、转义引号、转义反斜杠）的四种快速探针都计 0；无效文档在字符串外的 65,536 个逗号也被预扫拒绝。CSV 单元格预算：LF、CRLF、尾随分隔符、逗号串的形状都在数百次分配时被拒；单独 CR、混合行尾与阈值以下的稠密输入先建后拒（F-15）。所有探针的 arena 账本都平衡，包括捕获后的拒绝、解析器内的拒绝和 VM 自己的资源拒绝。接受侧以“同长度文本被准许后由解析器拒绝”证实。恰 65,536 格的完整解析在默认 8 MiB slot 预算下不可达，LF 与 CR 都在约 39,000 格处 `av.resource.slot-bytes`。Plan 用例里 `is_valid(over)` 的输入是全空格文本，无论有没有字节预算都是 false，不具区分力。新增用例最慢 7.169 s（`f07_node_budget_admits_the_last`，work 记的是 6.62 s），在 10 秒门内，余量约 2.8 s。
3. 单文件嫁接：自有名字保留成立，触发条件误判局部名字。注释与字符串不触发；程序自己的顶层 `fn parse`、`fn minify`、嵌套 `fn parse`、`JsonValue`、`JsonEncodable` 都还是它自己的；保留名冲突是普通的 `source.duplicate`，span 指向程序声明；误分类的位置是局部变量、参数、嵌套函数名、闭包（F-13）。“从不使用 JSON 或 CSV 的程序行为不变”只在读码上成立：没有触发时嫁接文本逐字节等于输入，合成步骤不改程序。带这些局部名的程序现在每次编译多付约 0.55 s。
4. `List` 元素上的 `json_encode` 覆写：两面一致，只在覆写记录全为标量字段时成立（F-14）。满足该条件时，深度 2 嵌套的输出与 Python `json` 逐字节相同，单文件与 packages 面相同；`?Item` 字段输出 `{"some":{"w":3,"q":30},"none":null,"flag":false}`；整数极值与浮点字段精确：`f32` 的 0.1 经 `f64` 输出 `0.10000000149011612`，运行期负零输出 `0`；字符串转义与 Python 相同。
5. worker 的 `reported_span`：没有越界的 span，程序自身的错误不受影响，跨入口归因有误（F-17）。span 是字节偏移：含 CJK 的 125 字节程序按字节读回正是 `csv_parse("\"unclosed quoted field")`；程序自身的 `fail(msg)` 与溢出 trap 保留原 span。
6. 健康：新增行上编译器警告 0；rustfmt 4 处（F-16）；其余干净。
7. 规范增量：方向正确，多数条文属实；五处不符或留白（F-18），嫁接规则的“没有自行声明该名”一句需要与实现对齐（F-13）。

**复审 r2 的计时表：实跑范围**

| 范围 | 结果与执行/排除数 | 准备构建 | 发现 | 测试 | 清理 | 总墙钟 | 占用等待 |
|---|---|---|---|---|---|---|---|
| 仓库外 oracle 身份重建（dev） | 无重编译 | 1 s | — | — | — | 1 s | 0 |
| 自建 harness 构建（dev，三个目标目录） | 成功；首次 36 s 因复审自己的代码编译失败 | 36 + 1 + 0 + 1 + 59 + 58 s | — | — | — | 约 155 s | 0–1 s 每次 |
| `nextest list` × 4（清单 26 / 54 / 9 / 1） | 完成 | 1.16 / 0.19 / 0.12 / 0.18 s | 含在总墙钟内 | — | 未观测 | 5 s | 0–1 s 每次 |
| `nextest run` plan1124_json_csv | 26/26 PASS，1478 skipped | 0.18 s | 未观测 | 43.5 s | 未观测 | 44 s | 0 |
| 近邻 json975、plan1120、plan1121、module_v_1064 | 54/54 PASS，1450 skipped | 0.19 s | 未观测 | 4.4 s | 未观测 | 5 s | 1 s |
| ace-core 嫁接与 stdlib 单元 | 9/9 PASS，85 skipped | 0.12 s | 未观测 | 1.16 s | 未观测 | 2 s | 0 |
| ace-av worker 单元 | 1/1 PASS，32 skipped | 0.18 s | 未观测 | 0.18 s | 未观测 | 1 s | 1 s |
| 工具准备 `fixed-tests.py case-av-language-parity --prepare-only` | PREPARED | 4 s | — | — | — | 4 s | 1 s |
| 语言用例 × 8（检查器逐例） | 8/8 PASS | 复用上项 | 每进程含套件 inventory，未单列 | 见逐例表 | 未观测 | 30 s | 0 |
| 数值 oracle（仓库外 oracle，12 个输入文件） | 61,467 个输入，0 不一致 | — | — | 分片墙钟合计 4,804 s（6 或 10 进程并行） | — | 约 05:37–07:04 | 0 |
| 预算探针（自建 harness） | 见重点 2，账本全部平衡 | — | — | 0.13 s 至 2,564 s | — | 与其他项并行 | 0 |
| 嫁接与 span 探针（真实 av-worker） | 68 次运行，37–781 ms 每次 | — | — | — | — | 约 2 min | 0 |
| JsonEncodable 探针（单文件、packages、矩阵、变体） | 见重点 4 | — | — | 每个毫秒级 | — | 约 1 min | 0 |
| rustfmt 归因（16 个 .rs，带两个对照） | 新增行上 4 处 | 不适用 | — | — | — | 数秒 | 不适用 |
| 合并干跑 `git merge-tree` | `av.rs` 内容冲突 | — | — | — | — | 1 s | 不适用 |

构建档位：nextest 为 `test` profile（unoptimized + debuginfo），仓库外 oracle 与自建 harness 为 dev profile，没有 `--release`。未执行：`cargo t all`、六个冻结组整组、`ace-perf`。Linux 与 WSL 用例按 2026-11-14 裁定排除。

**复审 r2 的计时表：本计划新增用例逐例计时**（新增测试义务共 40 项：26 个一致性用例、6 个单元用例、8 个语言用例，没有参数化输入或包装子项，全部在 10 秒门内）

| 新增用例 | 测的行为与 oracle | 实际秒数 | 结果 |
|---|---|---|---|
| `plan1124_json_csv::f07_node_budget_admits_the_last` | 65,536 个值被准许并由解析器拒绝；1,025 个值的合法文档完整解析 | 7.169 | PASS |
| `…::f07_cell_budget_admits_the_last` | 65,536 个单元格被准许；1,025 个单元格解析并编码 | 4.964 | PASS |
| `…::f07_cell_budget_refuses_the_first_excess` | 65,537 个单元格在构建前被拒 | 4.345 | PASS |
| `…::f07_json_byte_budget_on_every_entry` | 1,048,577 字节四个入口拒绝；1,048,576 字节由节点预算拒绝 | 3.152 | PASS |
| `…::f07_csv_byte_budget_on_every_entry` | 1,048,577 字节 `csv_parse`、`csv_encode` 拒绝；1,048,576 字节由单元格预算拒绝 | 2.729 | PASS |
| `…::f07_node_budget_refuses_the_first_excess` | 65,537 个值在构建前被拒 | 2.197 | PASS |
| `…::f08_exact_midpoints_at_the_smallest_normal` | 最小正规数附近的中点及两侧一位 | 1.999 | PASS |
| `…::f08_exact_midpoints_among_subnormals` | 次正规数里的中点及两侧一位 | 1.972 | PASS |
| `…::f01_f08_review_counterexamples_read_back_exactly` | r1 的 F-01、F-08 反例及同类 48 个读回逐位相等 | 1.773 | PASS |
| `…::v01_and_v02_packages_cross_module_json_schema_and_rename` | 跨模块 `JsonEncodable` 编码经自家解析器回读 | 1.649 | PASS |
| `…::f01_minify_round_trip_keeps_every_double` | 19 个文本经 `minify`、`prettify` 后读回相同 | 1.179 | PASS |
| `…::f04_override_holds_for_list_elements` | 覆写在顶层、嵌套字段、`List` 元素三处一致（两个面） | 1.167 | PASS |
| `…::f08_exact_midpoints_round_to_even` | 42 个精确中点，平位取偶 | 1.165 | PASS |
| `…::f05_graft_follows_references_and_keeps_the_programs_names` | r1 的 F-05 反例；保留名冲突；自有 `JsonValue` 与库并存 | 1.143 | PASS |
| `…::v01_json_depth_and_budget_limits_fail_safely` | 深度 127 成功、130 失败；超 1 MiB 被字节预算拒绝 | 0.882 | PASS |
| `…::f08_exact_midpoints_near_1e300` | 1e300 附近的中点及两侧一位 | 0.838 | PASS |
| `…::v02_json_encodable_negative_controls_and_non_finite_float` | 未遵从记录、`char` 字段具名拒绝；inf 触发 `JsonError` | 0.737 | PASS |
| `…::f08_shortest_text_breaks_ties_to_even` | 20 个等距最短候选取末位偶数 | 0.720 | PASS |
| `…::f01_f02_f03_record_fields_print_exactly` | 记录字段输出精确文本；pretty 与 compact 只差空白 | 0.602 | PASS |
| `…::f08_finite_range_and_long_digits` | 越界被拒、下溢为 0、40 位数字 | 0.591 | PASS |
| `…::v02_single_file_widths_empty_custom_override_and_forged_std_rejected` | 空记录、自定义覆写、各宽整数；伪造 `package std` 被拒 | 0.506 | PASS |
| `…::v01_json_dynamic_tree_keys_dup_types_and_errors_released` | 树访问、重复键、缺键、三个捕获错误；退出时账本为零 | 0.449 | PASS |
| `…::f06_pretty_bytes_and_as_array_usability` | 缩进布局的完整字节；`as_array()` 经 `remove` 可用 | 0.409 | PASS |
| `…::f01_formatters_keep_the_number_text` | `minify`、`prettify` 保留数字原文 | 0.408 | PASS |
| `…::f02_i32_minimum_and_negative_zero_serialize` | `i32` 最小值与字面量 `-0` 在五个入口的精确文本 | 0.389 | PASS |
| `…::v03_csv_rfc4180_delimiters_roundtrip_and_error_cleanup` | 引号、换行、空字段、`;` 与制表符分隔符、四个错误后零泄漏 | 0.347 | PASS |
| `ace-core` `stdlib_json_csv_graft_keeps_the_programs_own_names` | 自有名字与保留名冲突 | 0.676 | PASS |
| `ace-core` `auto_json_and_csv_parse_cleanly_in_packages_and_single_file_profiles` | 两个库在两种 profile 下可解析 | 0.280 | PASS |
| `ace-av` `plan1124_uncaught_library_error_is_reported_at_the_call` | 未捕获的库错误定位到程序的调用 | 0.179 | PASS |
| `ace-core` `stdlib_module_graft_shape` | 嫁接文本的形状 | 0.109 | PASS |
| `ace-core` `stdlib_json_csv_graft_follows_references` | 引用分析与嫁接触发 | 0.046 | PASS |
| `ace-core` `auto_json_and_csv_inputs_match_frozen_sources` | 冻结身份 | 0.009 | PASS |
| `av.parity.1124.json-csv-packages`（exit 42） | 跨模块，`use std.csv` 与 `std.encoding.csv` | 步墙钟 1.694 s | PASS |
| `av.parity.1124.json-dynamic-tree`（exit 42） | 动态树与错误路径 | 步墙钟 1.519 s | PASS |
| `av.parity.1124.numbers-exact`（exit 42） | 数字读回逐位相等、最短文本 | 步墙钟 1.478 s | PASS |
| `av.parity.1124.json-encodable-schema`（exit 42） | 泛型编码经自家解析器回读 | 步墙钟 1.467 s | PASS |
| `av.parity.1124.pretty-bytes`（exit 42） | 缩进布局的完整字节 | 步墙钟 1.213 s | PASS |
| `av.parity.1124.override-list`（exit 42） | `List` 元素走用户的 `json_encode`（全标量记录） | 步墙钟 1.193 s | PASS |
| `av.parity.1124.graft-own-names`（exit 42） | 自有 `parse`、`json_cp`、`JsonValue` 与库共存 | 步墙钟 1.169 s | PASS |
| `av.parity.1124.csv-rfc4180`（exit 42） | RFC-4180 解析与编码 | 步墙钟 0.641 s | PASS |

沿可达调用路径没有发现日常测试触发编译。一致性用例与单元用例是 nextest 在 `--no-capture` 并行调度下的实测值；语言用例是检查器报告的步墙钟。

**复审 r2 的规范增量结论**（SD-01，modify `docs/specs/ace-migration/m2-stdlib-data.md`）：方向正确，需修正后再沉淀。

- 需改正的条文：F-14（覆写边界与诊断码）、F-15（CSV 预扫）、F-18 的五处。
- frontmatter：`supersedes_spec_components` 的路径准确；`new_spec_components: []` 是因为只修改现有文件，merge 时需写明理由；`touched_goals: [GOAL-002]` 是真实 ID（`docs/specs/goals.md` 第 11 行），贴合度沿用 r1 的判断，未重新评估。
- 留给 merge 的 pending：变基（master 现值 `7bb90b1f7`，需解决 `av-core/src/av.rs` 的内容冲突；变基后受影响验证要在新提交上重跑）；六个冻结组整组并集（`stdlib`、`ace-stdlib`、`ace-source`、`ace-semantics`、`ace-av`（带 `ace-av/test-case-batch`）、`case-av-language-parity`），范围同 r1 所列，本次未整组执行；规范里指向 `../../plans/1124-…` 的链接归档时改指 `plans/archive/`；`KNOWN-DEBT-AND-RISKS.md`：r1 列的三行，以及 D-1124-AV-TEXT-01、DIAG-01、SYNTH-01、PKG-01（D-1124-AV-TEXT-01 的根因复审读码确认：`owned_text.rs` 第 226–270 行每次调用都把整段文本收集成 `Vec<char>`）；材料区登记（F-19）；`review2/` 现为 1.4 GB，其中 1.3 GB 是可重建的 harness 构建缓存（`probe-crate/target3`），随材料区删除。

**复审 r2 的路由**：`status: executing`；`next: work`，按 F-13、F-14、F-15 优先，然后 F-16 至 F-19 修复并补永久断言，在新提交上重审受影响的 AC。

- 回开范围：AC-01（V-02、V-03）与 AC-02；T-01（T01-7、T01-8、T01-9 的文本）、T-02（F-13、F-16）、T-03（F-14、F-16）、T-04（F-15、F-16、F-17）。`current_step` 重算为 0，`plan_revision` 不变。
- 这是第二次未通过的复审。按复审技能，下一次交接被视为就绪之前，需先核 work 对这三项中等级发现的诊断与覆盖、设计上的调整。
- O-5（`project.rs` 的 `is_numeric()` 门控）仍由冻结组兜底。

stage: goal | plan_id: PLAN-1124 | plan_revision: r1 | round: 5 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1124/auto-lang | commit: 9a9c296ec | next: work

2026-11-24 · stage: work · PLAN-1124 r1 · 修复轮 2（对应复审 r2 的 `needs_fix`，受审基线 `9a9c296ec`）。修复提交 `b5ada0bd0`（worktree `D:/autostack/.wt/lang-1124/auto-lang`，分支 `plan-1124-dev`，基面 master `8a52d7364`）。next: review（T-05，在新提交上重审受影响的 AC）。

**发现关闭表**（发现 → 修复 → 永久用例 → 当前结果）

| 发现 | 修复 | 永久用例 | 当前结果 |
|---|---|---|---|
| F-13 | `stdlib_refs.rs` 改为带作用域栈的引用分析：局部变量/参数/嵌套函数/闭包参数/模式绑定/循环变量/`catch` 与 `with` 绑定都不算引用，类型与 spec 名字不受局部遮蔽；`analyze()` 同时给出 `referenced`（不被遮蔽的值引用）与 `mentioned`（含局部）。`adapter.rs` 先在不嫁接时编译，成功即采纳；失败且文本仍提到库名时才重编并嫁接；重编被保留名冲突拒绝时回退到不嫁接的结果（程序自己的错误） | `stdlib_json_csv_graft_ignores_local_bindings`（9 种绑定位置 × 两个模块的公开名）、`stdlib_graft_prepares_once_unless_the_graft_is_in_doubt`、`f05_graft_follows_references_and_keeps_the_programs_names`、语言用例 `graft-local-names` | 通过 |
| F-14 | `json_encode_synth.rs` 对“带手写 `json_encode` 且拥有自有数据”的记录作 `List` 元素时，在 `List` 字段处以 `source.json-encode-unsupported` 具名拒绝（消息给出记录与字段），不再落到 `verify.view-escape`；全复制标量字段的覆写照旧在顶层/嵌套/`List` 元素三处生效。规范写明实际规则 | `f14_override_record_that_owns_data_is_refused_by_name`（packages 面与单文件面；另含不带覆写的逐字段路径与多标量覆写） | 通过 |
| F-15 | `csv.at` 的分隔符预扫把单独 CR 计为行尾、CRLF 只计一次，阈值从 131,071 改为 **65,536** 字符；阈值以下同样先数（注释改正：空单元格只占一个分隔符）。`csv_encode_rows` 在构建输出前先数单元格 | `f15_cell_budget_is_refused_before_building_{lf,crlf,lone_cr,mixed_line_endings,commas_only,dense_rows}`、`f15_cell_budget_counts_lone_cr_at_the_new_threshold`（65,536 格准许、65,537 格构建前拒绝） | 通过 |
| F-16 | 四处新增行按 rustfmt 格式化（`project.rs`、`resolve.rs` ×2、`instances.rs`）；本轮再按“新增行归因”复核 | `fmt_attrib`（`git diff HEAD` 新增行范围 × `rustfmt --check`）：**新增行上的 hunk = 0**（196 个 hunk 全部落在基面既有行） | 通过 |
| F-17 | `language_source_worker.rs` 的 `library_call_site` 注释写明规则是“首个静态可达调用，是提示不是运行记录”；测试覆盖另一入口成功调用在前、分支内失败、经程序自己的函数进入三种情形 | `plan1124_uncaught_library_error_is_reported_at_the_call` | 通过 |
| F-18 | `json_auto.at` 用节点标志 3 保留“负字面量舍入到零”的符号（`-0`、`-0.0`、`-0e5`、`-1e-400` 输出 `-0`；运行期零仍输出 `0`），`as_int`/`get_int` 对标志 3 抛 `TypeError`；`__csv_encode_rows_code` 先数单元格；规范正文同步（覆写边界与诊断码、CSV 预扫与 `csv_encode_rows`、`JsonEncodable` 名字规则、131,071 的算术、负零） | `f18_negative_zero_literals_keep_their_sign`、`f18_encode_rows_refuses_over_the_cell_budget_before_building`、`f18_spec_is_recognized_by_name` | 通过 |

**本轮修复中的自我修正与事故记录**

- `f14` 的 packages 用例最初把 `pub fn json_encode() Str` 写在 type 体内：packages 面在 CST 投影失败后回退到 Plain 面，报出误导性的 “`type` is not part of this source profile”。仓内契约写法是 type 体内 `fn`（`pub` 只用于模块项），用例已改。该误导诊断是既有行为（本计划未改投影回退），记为观察项。
- `f14` 的 `List` 元素期望最初写成 `"<1,2>"`（把覆写文本再当 JSON 字符串转义）。实现与顶层/字段一致地原样嵌入覆写文本（与 `f04` 的自定义覆写 同一规则，`json_encode_synth.rs:453` 与 `:406` 同一条 `append_out`），期望改为 `<1,2>` 并加注释。
- **恢复事故**：本轮编辑测试文件时一次 PowerShell `Set-Content -NoNewline` 把该文件的所有行拼成一行（内容未丢，换行丢失）。用 `plan-artifacts/plan-1124/repair/parts/r2_tests.rs.txt` + 本轮两处编辑重建，rustfmt 后按“去掉换行后逐字节与事故前快照相等”（80,016 字符）验证，再 `rustfmt --check` 通过；重建后 26 + 11 = **37 个一致性用例**与上述修复一起重跑（26 旧 + 11 新，`#[test]` 计数核对一致）。事故与恢复过程记录在此，供复审核对。

**本会话验证（修复提交 `b5ada0bd0` 的源码；nextest `test` profile，未优化）**

| 范围 | 结果 | 花时 |
|---|---|---|
| 构建 `cargo build --locked --offline -p ace-core -p ace-source-query -p ace-av-core -p ace-av --lib --tests --features ace-av/test-case-batch` | rc=0 | 7 s（增量） |
| `ace-core --lib`：`stdlib_list_graft_tests`（F-13 单元） | 7/7 | 0.78 s |
| `ace-core --lib`：`stdlib::tests`（冻结身份与两种 profile 解析） | 4/4 | 0.35 s |
| `ace-av --lib`：`plan1124_uncaught_library_error`（F-17 单元） | 1/1 | 2.63 s |
| `ace-av --test conformance`：`plan1124_json_csv` | **37/37**（并行 97.7 s；串行单测最慢 7.76 s） | 串行合计 81.9 s |
| 近邻 `json975`/`plan1120`/`plan1121`/`module_v_1064` | 54/54 | 3.96 s |
| `ace-source-query --lib`（`instances.rs` 只格式改动） | 104/104 | 2.58 s |
| `ace-stdlib-conformance`（`ace-stdlib` 组的两个用例） | 2/2 | 2.10 s |
| 语言用例 `av.parity.1124.*` ×9（检查器逐例 `--case`，含新增 `graft-local-names`） | **9/9 PASS** | 步墙钟 0.47–1.50 s |
| `ace-core --lib --tests`（`ace-source` 组目标全量，906 清单 / 本轮 1001 项） | 980/1001 run：**975 passed、5 failed、1 skipped**；5 个失败**在基线 `9a9c296ec` 上逐一同名复现**（`text_record_source::*`，见下） | 18.5 s |
| `%TEMP%\p1124-*` 目录计数 | 30 → 30（无新增残留；原有 30 个是修复前遗留，merge 收尾清除） | — |

**原红（不是本计划引入）**：`ace-core::conformance text_record_source::{flagship_combination_compiles_and_verifies, generated_atom_carries_the_combination_surface, move_arguments_transfer_whole_roots_not_views, named_ctor_source_order_is_the_written_order, positional_ctor_and_owning_contexts_promote_literals}` 在 `b5ada0bd0` 上失败；把本轮 12 个文件回退到基线 `9a9c296ec` 后同一命令得到**同样 5 个失败**（21 tests: 16 passed, 5 failed），故按原红记，不计在本计划头上。`compile_text_record_source` 走 `compile_parts_observed`，不经过本计划改动的嫁接/合成路径（读码）。组级原红另有 `ace-av` 13 项与 `case-av-language-parity` 4 项，逐条见下表。

**六个受影响固定组整组（`cargo t` 入口，`b5ada0bd0` 源码；已按 `--prepare-only` 准备后 build-free 运行）**

| 组 | 结果 | 原红/环境（在基线 `9a9c296ec` 上逐条复现） | 墙钟 |
|---|---|---|---|
| `stdlib` | **PASS**，176 项 | — | 153 s |
| `ace-stdlib` | **PASS**，2/2 | — | 6 s |
| `ace-semantics` | **PASS**，295 项 | — | 9 s |
| `ace-source` | 1001 项：996 passed、1 skipped、5 failed（组红） | `text_record_source::{flagship_combination_compiles_and_verifies, generated_atom_carries_the_combination_surface, move_arguments_transfer_whole_roots_not_views, named_ctor_source_order_is_the_written_order, positional_ctor_and_owning_contexts_promote_literals}`（回退到 `9a9c296ec` 后 13/13 同名失败） | 15 s |
| `ace-av`（带 `ace-av/test-case-batch`） | 1667 项：1654 passed（2 slow）、23 skipped、13 failed（组红） | `av_method_modes::use_after_move_of_method_arg_rejected`、`av_method_owned::{v03_flagship_value_thirty, v03_token_microprogram_fifty_seven, v10_method_field_borrow_return_zero_alloc, v10_second_alloc_failure_rolls_back_partially_built_output}`、`av_owned_first_case::av_owned_first_case_whole_path_and_live_zero`、`av_owned_recipe::av_owned_recipe_fixture_face_matches_hand_audit`、`av_test_batch::c_data_profile_keeps_its_scalar_entry_boundary`、`component_event_sessions_1106::v02b_text_leaves_are_pure_reads`、`resource_915::{r02_l04_arena_sweep_is_complete, r02_l04_value_sweep_is_complete, r02_l05_v05_sweeps}`、`ace-av-core layout::tests::layout_rejects_non_record_and_empty`（回退到 `9a9c296ec` 后 13/13 同名失败） | 89 s |
| `case-av-language-parity`（AV 视图，921 行 / 560 RUN 检查） | 组红：4 行非 PASS，其余 PASS（含 `av.parity.1124.*` 9/9） | `av.parity.1093.explicit-c-library` FAIL（`foreign.prepared`）、`av.parity.1093.c-header-spelling-rejected` FAIL（同上）、`py.slicing.1127.numpy-torch` / `py.slicing.1127.torch-zero` ERROR（`av.resource.execution-wall`，foreign 调用超时）；四条用同一套已准备工具在 `9a9c296ec` 上逐条复现同名结果 | 88 s |

**未执行**：`cargo t all`、`ace-perf`；Linux/WSL 用例按 2026-11-14 裁定排除。**T-06 注意**：`ace-source`、`ace-av`、`case-av-language-parity` 的组红全部由上面列出的原红/环境项造成，冻结组并集必须与同一基线比较，不要把组 `rc=1` 直接当成本计划回归。

**F-19 收尾**：材料区登记仍缺（`.retention/policy.json` 无 1124 条目），留给 merge 收尾按生命周期规则处理；`%TEMP%` 下 `p1124-probe.rs`/`p1124-probe.err` 两个 03:20 的 rustfmt 探针残留已移入 `plan-artifacts/plan-1124/work-r2/temp-leftovers/`。

stage: work | plan_id: PLAN-1124 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1124/auto-lang | branch: plan-1124-dev | base_commit: 9a9c296ec | code_commit: b5ada0bd0 | dependency_revisions: 主检出同源 workspace（`--locked --offline`） | task_ids: T-01, T-02, T-03, T-04 | evidence: 本节表格与 plan-artifacts/plan-1124/work-r2/（gate/ 与 cases/ 原始日志与报告；merge 时按规则删除） | blockers: 无 | next_responsible_role: review

2026-11-24 · stage: review · review_scope: repair · status: in-progress（进行中的标记，不是收据）。独立复审子代理 11:12 启动，受审提交 `b5ada0bd0`，证据目录 `D:/autostack/plan-artifacts/plan-1124/review3/`；复审记录与路由收据待其返回后追加在本行之后。若此处之后没有收据而会话中断，续接轮次应先读该目录里已有的证据，再决定还缺什么，不要从头重复。

## 10. 待澄清事项

T-01 冻结 JSON 数字、对象顺序与重复键；记录 CSV 旧边界及规范化。若编码依赖更广反射，先以现有受检接口落定机制和必要共享差额。

- **（复审 r1，2026-11-23；修复轮 1 定位，2026-11-24）预算边界的可观察性与大输入性能。** 根因已定位：`ace-av-core` 的 `owned_text::char_at` 与 `scalar_range_to_bytes` 每次调用都把整段文本物化成 `Vec<char>`，逐字符读取是平方级墙钟（§9）。它在 `av-core`，不在本计划的 `affects` 内。本计划的处理是不碰 `av-core`、不调阈值，只让自己新增的预扫按短片段读取，六个预算用例因此都在 10 秒门内；解析器自身的这条路径没有改动。平台层的修复登记为债务候选 **D-1124-AV-TEXT-01**（merge 时写入 `KNOWN-DEBT-AND-RISKS.md`），默认另立计划。如要在本计划内一并修 `av-core`，请指示；未指示前按“已定位、未修复、不阻塞本计划验收”处理。
- **（修复轮 1，2026-11-24）债务候选，merge 时写入 `KNOWN-DEBT-AND-RISKS.md`。**
  - **D-1124-AV-TEXT-01**：AV 文本按标量索引每次调用都物化整个字符串。Owner：ACE AV 核心文本运行时。关闭条件：按标量索引不再随文本总长线性增长，逐字符扫描 131_071 个标量的墙钟接近线性。
  - **D-1124-DIAG-01**（复审 O-4）：记录里写 `static fn json_encode()` 得到的是 `source.duplicate`，span 只有 1 个字符宽（worker 实测 `5..6`），而不是签名不符时的具名诊断。Owner：`json_encode_synth` 的覆写检测。关闭条件：这种写法得到具名诊断与覆盖整个声明的 span。
  - **D-1124-SYNTH-01**（复审 O-6）：`json_encode_synth.rs` 用硬编码的 `p3_stdm4_json_json_str` 探测 `packages` 模式，与 std 包的改名方案耦合。关闭条件：改按模块归属判断，或随 std 包改名方案一并处理。
  - **D-1124-PKG-01**（复审 O-7）：`modules/packages.rs` 对不在输入里的模块（自动插入的 `std::json`）把版本回退为 `"1.0.0"`；它今天与 `STDLIB_VERSION` 相同，版本号一变，实例的身份哈希不会跟着变。关闭条件：版本取自 `STDLIB_VERSION`。
  - 复审的 O-1（注释含触发词的编译变慢）已随 F-05 消失；O-2、O-3、O-9 已作为冻结合同写进 §4 与规范；O-5 由 merge 的冻结组兜底；O-8 无需处理。
- **（复审 r1，2026-11-23）合同澄清的默认取向。** 以下按默认取向进入修复轮，如需改变请在恢复目标时指示：
  - F-04：让 `List<R>` 元素同样走用户的 `json_encode` 覆写，与 D-1124-T01-3 一致。
  - F-07：预算按字节计，与“1 MiB”的冻结文本一致（`len_bytes()` 可用）。
  - F-08：以正确舍入为目标，并在 T-01 冻结记录写明数值范围与精度承诺。
  - [修复轮 1 已按上述默认取向落实 2026-11-24] 三项都已实现并有永久断言（§9）。
