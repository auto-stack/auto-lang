---
plan_id: PLAN-1027
status: archived
completion_kind: delivered
feature_name: ACE native UTF-8增量编辑保真修复
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-13
plan_revision: 3
current_step: 6
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-ac/native-closures-source.md, docs/specs/auto-ac/closures-function-production.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1027] ACE native UTF-8增量编辑保真修复

## 0. 变更摘要

2026-11-13 r3（T-01真实入口核证与有界修约）：开工重核 `master@6584e94d4`（1028 已合入，`closures-function-production.md` SHA256 更新为 `6ef4305d...`，与本计划无冲突）；建树 `D:/autostack/.wt/lang-1027/auto-lang`（分支 `plan-1027-dev`）及唯一材料目录 `D:/autostack/plan-artifacts/plan-1027`。T-01 在真实 `ClosuresNativeSession` 入口核证发现：`r2` §6 示例中的 `s.len_bytes()` 与 `s.char_at(0).to_codepoint()` 在 `Closures` profile 中不存在于 descriptor/lowering/native admission（`hir_adapter.rs:4504,4510` 显式限定 `!self.collections()`，`closures_lower.rs:116-135,213-236` 仅接纳 `StrLen` 即 `s.len()`，不含 `Char`/`StrLenBytes`/`StrCharAt`）；同时实测 `s.len()` 合法正例可运行并已取得真实 PE/前端反例（`"é"->"äé"` 返回错误 `exit_code=43` 而非 `42`、`"中"->"串中"` 触发 `source.internal` 非 UTF-8 边界 span、`同B hit` 失命中且接受源码含 `U+FFFD`）。按 §1 非目标（不扩语言/profile 支持）与 §5 先决决定边界，将 §6 具体测试输入修约为已验证的 `Closures` 合法支持域（`s.len()` + 双端字节/hit/独立 PE 断言），目标/AC/Case ID/6 任务/范围不变。

2026-11-13 r2当前：用户要求对1028以外计划逐份核是否足够定制。本包补真实输入、状态/故障设置、独立关闭断言、测试落点、任务改面和Spec具体增量；原发现/AC/Case/T-ID及6任务、drafting 0/6保留。只做规划与静态源码核对，不修改实现/测试/配置，不接管1028实施，也不宣称动态反例或产品通过。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F01，优先级**P1**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

## 1. 目标

对任何已支持的单区域编辑，splice(old, range, replacement)逐字节等于请求新源；UTF-8边界同时保护旧range与新replacement。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。不扩closure签名/捕获支持；真实Rust/PE验证未执行。

## 2. 架构方案

复用双端字符边界算法的内部机制，不让native依赖整个AV宿主；保base SHA、单body准入、source span与当前增量路由。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。等价机制由work选择；公共语义/支持/跨核心合同变化先具名设计及修约，不能临时加深度/规模限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。唯一工作树 `D:/autostack/.wt/lang-1027/auto-lang`（分支 `plan-1027-dev`，基面 `master@6584e94d4`，已创建并修复 Windows gitdir 路径）；work/review/merge顺序使用同树，禁止链接。材料唯一 `D:/autostack/plan-artifacts/plan-1027/`（已创建并登记为 `in-flight`）。

## 4. 需求分析与背景调查

2026-11-13 用户以 `/goal 完成计划1027`（`/auto-plan-goal`）明确授权本计划在既定范围内实施、独立复审、修复及合入；取代本节先前“暂不改实现”的历史授权状态，不改变验收和范围。

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2当前取材Git=c2760324da25795594ab0cc01b581e3a306d2375；r3开工基线 `master@6584e94d4`（1028 已合入，`closures_incremental.rs`/`closures_fn_session.rs`/`closures_fn.rs`/`native-closures-source.md` 与 `r2` SHA256 一致；`closures-function-production.md` 随 1028 追加 `§观察窗口恢复边界`，SHA256=`6ef4305d2f2f80ec5dec50eb3408751f36bab9e843f845d8dff392c3e9be4bab`，与 1027 无冲突）。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/native/src/closures_incremental.rs](../../experimental/ace/crates/native/src/closures_incremental.rs) | dd5a2747a9ca4a0185414ead6dfc55372407244f4669b94aff8d7d6eb0811de2 | 已核实（`master@6584e94d4` 字节一致） |
| [experimental/ace/crates/av/src/closures_fn_session.rs](../../experimental/ace/crates/av/src/closures_fn_session.rs) | c325ca50dd9059fdf024a60cdce62799e3b75a12c09f78aeab8d4917818b8597 | 已核实（`master@6584e94d4` 字节一致） |
| [experimental/ace/crates/core/src/source/closures_fn.rs](../../experimental/ace/crates/core/src/source/closures_fn.rs) | 644879a03134c35e8dd7bcbc884e648a26b562933dc91ede52cae522f33845ff | 已核实（`master@6584e94d4` 字节一致） |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-ac/native-closures-source.md](../../docs/specs/auto-ac/native-closures-source.md) | afdfae349b87d979fb1ee50500a8eeb0c358a66dec87e069672b874e71e4eb62 | 当前规则保持；`6584e94d4` 无漂移 |
| [docs/specs/auto-ac/closures-function-production.md](../../docs/specs/auto-ac/closures-function-production.md) | 6ef4305d2f2f80ec5dec50eb3408751f36bab9e843f845d8dff392c3e9be4bab | 随 1028 新增尾部观察窗口节；既有规则保持 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1把native会话入口写成AV类型，且未区分算法空串、完整程序与当前不支持的Unicode标识符；仅比较字符数也可能漏掉U+FFFD替换。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/native/tests/cases/closures_restore_tier.rs | a51f33342879971640a4195d6b9fee4ed6ae355d40bd3e558c84d25e5a359b68 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/native/tests/cases/closures_native_tier.rs | b13c15acdff363589738ee73fcfd90aefe181ebf8697ddc417d7f7d922603d12 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/core/tests/cases/text_source.rs | 13004299c104300a36a598c1581e82f4e9c62f3c5cf901a03fe0786b9857d8b7 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。

## 5. 详细设计

唯一范围=F01。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：对任何已支持的单区域编辑，splice(old, range, replacement)逐字节等于请求新源；UTF-8边界同时保护旧range与新replacement。 复用双端字符边界算法的内部机制，不让native依赖整个AV宿主；保base SHA、单body准入、source span与当前增量路由。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r2执行合同预检（当前优先）

**输入到消费路径。** 用ClosuresNativeSession::new(source,"main",workdir,None)创建真实native会话，apply_source编辑；新旧fresh各建独立会话和独立工作目录。低层diff单测放原src/closures_incremental.rs的cfg(test)模块，不新增native→AV宿主依赖。当前Grammar ident仅ASCII，非ASCII标识符只作受控拒绝近邻，不当合法语言正例。

**状态、故障与责任。** 算法单测必须逐字节核完整重建而非只核range边界；真实PE必须核exit_code与stderr错误/资源事实，不能只比较两个都错误的后端。source接受态的检查可在原模块私有单测做，不新增产品public accessor。凡工具缺失记录infra/未执行，不能沿旧helper的return当PASS。

**先决决定边界。** 上述固定输入与原支持域可从T-01直接核证；T-01仅选择等价局部机制，不把所有设计留到实施。若真实入口与假设不符，必须给可重放反证和合法邻例后修约，不能仅写unknown或删除原Case。

**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。任何新增limit或公共支持变化按本节具名决定，不推定已批准；等价内部实现不需重复许可。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（拟议） | docs/specs/auto-ac/native-closures-source.md | 现规则/支持域保持；补充以下修复后的明确合同：native源码编辑必须字节保真；明确双端切点、请求B与实际接受源一致，保原warm/non-fast域。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-02 | modify（拟议） | docs/specs/auto-ac/closures-function-production.md | 现规则/支持域保持；补充以下修复后的明确合同：记录修复后的回归入口与来源；不得将低层Unicode支持写成新增Unicode标识符承诺。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-native ace-av ace-source。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| UTF-01 | AC-02 / T-02 | 合法字符串/注释中的é→ê、中→串、😀→😁；非ASCII标识符仅拒绝近邻 | diff_single_region→ClosuresNativeSession::apply_source→frontend edit_body | 重建bytes==requested；fresh/warm的值、诊断及接受源码一致 | 拟实施，未运行 |
| UTF-02 | AC-02 / T-02 | ASCII、空串、相同源、多字节整段插入/删除 | 同入口 | 合法邻例保持；跨body编辑保持原拒绝/非fast路由 | 拟实施，未运行 |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r3真实入口输入与状态断言（T-01 核证修约）

T-01 在 `master@6584e94d4` 上实测 `r2` 原输入，确认 `Closures` 域真实支持面为 `Str` 字面量 + `s.len()`（`MirInstr::StrLen` → `mn_coll_str_len` 标量计数）与 `//` 注释；`s.len_bytes()` 与 `s.char_at(0).to_codepoint()` 在 `hir_adapter.rs:4504,4510` 限定 `!self.collections()` 且在 `av.closures` descriptor 报 `bind.unknown-case: no accepted case expr.text_len_bytes / expr.text_char_at`，`closures_lower.rs:116-135` 亦不接纳 `Char` 类型。按 §1 非目标（不扩语言/profile 支持）修约为真实 `ClosuresNativeSession` 支持的正反例：

```auto
fn main() int {
    let f = (x int) => {
        let s Str = "é"
        return s.len() + x
    }
    return f(40)
}
```
- **UTF-01 标量计数区分 PE oracle**：以上 `f(40)` 为基线，将 `"é"`→`"äé"`（期望 `exit_code == 42`，修前因 `U+FFFD` 替换变为 `43`）、`"中"`→`"串中"`（期望 `42`，修前因字节偏移越界触发 `source.internal` span 非 UTF-8 边界错）、`"😀"`→`"😁😀"`（期望 `42`，修前生成多个 `U+FFFD` 或 span 错）。
- **UTF-01 同标量计数与注释保真 oracle**：`"é"`→`"ê"`、`"中"`→`"串"`、`"😀"`→`"😁"` 及注释 `// 注释 é`→`// 注释 ê`（`f(41)`/`f(40)` 期望 `exit_code == 42`）；因 `s.len()` 对 `"ê"`/`"串"`/`"😁"` 与 `U+FFFD` 均为 `1`，改以双通道独立区分：
  1) `closures_incremental.rs` 私有 `#[cfg(test)]` 直接核 `frontend.source().as_bytes() == requested.as_bytes()`（如 `"串"` `[0xE4, 0xB8, 0xB2]` 非 `U+FFFD` `[0xEF, 0xBF, 0xBD]`）；
  2) `closures_restore_tier.rs` 集成面 `audit1027_utf_01` 核 `A冷启动 → B暖编辑 → 同B hit`（`route == "hit"` 且 `codegen_units.is_empty()`；修前因接受源含 `U+FFFD` 不等于 `B`，第二次 `apply_source(&B)` 失命中并重编译）+ `B独立fresh`（`ClosuresNativeSession::new(&B)` 一致性）+ `MnDriver::build_closures` 落盘后 `restore_closures_session` 对 UTF-8 编辑的暖更新与 hit，以及无关单元（`d_main`）`object_hashes()` 保持不变；
  3) 受控拒绝近邻：非 ASCII 标识符 `let café int = 1` 拒于 `source.unsupported`；`s.len_bytes()`/`s.char_at(0).to_codepoint()` 保持既有域边界拒绝，不扩语言支持。
- **UTF-02 低层字节保真与路由边界**：在 `ace_core::source::closures_fn::diff_single_region`（由 `ace_av::closures_fn_session` 与 `ace_native::closures_incremental` 共享纯函数，不新增 `native→AV` 宿主依赖）及 `closures_incremental.rs::tests::audit1027_utf_02` 核 ASCII、空串、同源、共享 UTF-8 前缀字节（`"é"`→`"ê"`、`"中"`→`"串"`、`"😀"`→`"😁"`）、同时共享前后缀字节（`"é"`→`"äé"`、`"中"`→`"串中"`、`"😀"`→`"😁😀"`、`"😀"`→`"𐀀"`）与多字节注释整段插删的 `splice(old, range, replacement).as_bytes() == new.as_bytes()`；并在 `ClosuresNativeSession` 核跨 body / 顶层新函数保持 `nonfast:crosses-units` 原路由。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| UTF-01 | A冷启动→B暖编辑→B独立fresh→同B hit + restore暖编辑 | `exit_code == 42`（含 `"é"→"äé"` / `"中"→"串中"` / `"😀"→"😁😀"` 独立 PE 值）；同标量/注释组 `同B hit` 命中且 `codegen_units` 空；私有测试 `accepted_source == B` 逐字节相等；无关单元 `object_hashes()` 不变；非 ASCII 标识符受控拒绝 |
| UTF-02 | ASCII/空串/同源/共享前后缀多字节/整段插删；body外或跨body改变 | `range` 双端 `is_char_boundary` 且 `splice(old, range, replacement).as_bytes() == new.as_bytes()`；域外仍原拒绝或 `nonfast:crosses-units`，无新语言支持 |

| 原Case ID | 拟新增永久测试名 | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| UTF-01 | audit1027_utf_01 | [experimental/ace/crates/native/tests/cases/closures_restore_tier.rs](../../experimental/ace/crates/native/tests/cases/closures_restore_tier.rs) | T-01固定设置与pre-fix反例，T-02补永久集成用例，T-03核真实发现/执行 |
| UTF-02 | audit1027_utf_02 | [experimental/ace/crates/native/src/closures_incremental.rs](../../experimental/ace/crates/native/src/closures_incremental.rs) | T-01已启用并录得pre-fix红，T-02共享修复并变绿，T-03核真实发现/执行 |

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，所有§6反例及邻例可在实际入口运行，重大未决先定，不私改expected。（`2026-11-13` 在 `D:/autostack/.wt/lang-1027/auto-lang` 实测完成：取得 `s.len_bytes`/`s.char_at` 域边界实证及 `s.len()` 真实 PE/span/hit 反例，日志见 `D:/autostack/plan-artifacts/plan-1027/t01-preflight.log` 与 `t01-probe.log`）
- [x] AC-02 / work：r3具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。（T-02/T-03 已在 `D:/autostack/.wt/lang-1027/auto-lang` @ `ddfe18f78` 落实并通过 `audit1027_utf_01` 与 `audit1027_utf_02`）
- [x] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。（`ace-source` 502/502、`ace-av` 1088/1088、`ace-native` 250/250 重放全绿，详见 §9 work 账本）
- [x] AC-04 / review：逐项核r3独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。（2026-11-13 核证 `ddfe18f78`，无未决缺口、无新 `native->av` 依赖、无新 `public` 访问器、无 workaround、修改文件零警告）
- [x] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。（Spec SD-01/SD-02 已于 worktree 提交并 `rebase master`、`range-diff` 1:1 一致，rebase 后跑通 `audit1027_utf_02` + `audit1027_utf_01`，`--ff-only` 合入 `master@a425e9adfbcf6881a12b85315fac7a5ae1a16bcd`；Spec 规范文件已核 SHA256 并归档本 Plan 至 `docs/plans/archive/`）
- [x] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。（已核并清理 `D:/autostack/plan-artifacts/plan-1027` 7 文件/34898B 及 `%TEMP%/plan1027-*` 3 目录/10 文件/208427B，刷新 `.retention/policy.json` 为 `deleted-at-merge (no holds)`；`wt-guard` 对 `.wt/lang-1027/{auto-lang,auto-down}` 均 `clean`，worktree、组目录与分支 `plan-1027-dev` 已全清）

### 验证证据摘要（merge 折叠归档，原始临时日志已清理）

- **T-01 治前反例（2026-11-13，已删除的 `t01-preflight.log` / `t01-probe.log`）**：`audit1027_utf_02` 在 `"é"->"ê"` 实败（`range=13..15`, `replacement=""`, 拼接 `[..., 239, 191, 189, 34]` vs `[..., 195, 170, 34]`）；真实 `ClosuresNativeSession` 下 `"é"->"ê"` 接受源码含 `U+FFFD` 且第二次 `apply_source` 失命中 `hit`、`"é"->"äé"` 返回错误 `exit_code=Some(43)`（预期 `42`）、`"中"->"串中"` 触发 `source.internal: span Span { start: 70, end: 81 } is not on UTF-8 boundaries after projection`、`s.len_bytes()`/`s.char_at()` 在 `av.closures` descriptor 报 `bind.unknown-case`。
- **T-02 靶向单/集成验证（`t02-verify.log`, `SHA256:614DDD9DDE...` 已清）**：
  - `cargo test --manifest-path "experimental/ace/Cargo.toml" -p ace-native --lib closures_incremental::tests` → exit 0，1/1 passed（`audit1027_utf_02`，4.81s）。
  - `cargo test --manifest-path "experimental/ace/Cargo.toml" -p ace-native --test conformance closures_restore_tier` → exit 0，3/3 passed（`audit1027_utf_01` + 2 原恢复用例，27.31s）。
- **T-03 固定组并集与环境竞态重放（`t03-groups.log`, `t03-replay-typed-inc.log`, `t03-ace-native-replay.log`）**：
  - `cargo t ace-source ace-av ace-native` 首跑（总墙钟 1291.68s）：`ace-source` PASS（502 selected / 502 executed / 502 ok / 0 fail / 1 excluded ignored `run_exe_60s_deadline_bounds_descendant_pipe_hold` / 1 native_unselected `performance`）；`ace-av` PASS（1088 selected / 1088 executed / 1088 ok / 0 fail）；`ace-native` 首跑 250 executed / 247 ok / 3 fail（仅 `typed_entry_incremental_tier` 3 例在冷并行下因既存 `typed_runtime.rs` 无跨进程文件锁触发 `Infra("typed runtime: os error 2")`；`audit1027_utf_02` ok 69.193s、`audit1027_utf_01` ok 110.342s、`closures_restore_tier` 3/3 ok、`closures_native_tier` 14/14 ok）。
  - 孤立重放 `cargo test --manifest-path "experimental/ace/Cargo.toml" -p ace-native --test typed_entry_incremental_tier` → exit 0，3/3 passed（2.28s）。
  - 完整组重放 `cargo t ace-native` → exit 0，PASS（250 selected / 250 executed / 250 ok / 0 fail / 0 excluded / 1 native_unselected `performance`，92.36s）。
- **T-05 rebase 后复核与 Spec 哈希（`a425e9adfbcf6881a12b85315fac7a5ae1a16bcd`）**：
  - rebase `master@a4ea11c34` 后 `range-diff` 1:1 无代码漂移；`audit1027_utf_02`（2.73s）与 `audit1027_utf_01` 重跑均 exit 0。
  - `docs/specs/auto-ac/native-closures-source.md` SHA256=`FCAFF1D24494F1EF2FDCCC848F872D55C3020913D67C5E79683D45B3505FEAC8`
  - `docs/specs/auto-ac/closures-function-production.md` SHA256=`195AB8710AFF24CDB740C3A15A5E1771E60303A10109DC84CBE0AA810503C1C0`

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、未来实施授权及当前源 | 已冻结 native 类型/语法/字符正反例；在 `closures_incremental.rs` 启用 `audit1027_utf_02` 实测 pre-fix 失败（`"é"->"ê"` U+FFFD `[239,191,189]`）并在真实 `ClosuresNativeSession` 取得 `"é"->"äé"` 返回错误 `exit_code=43`、`"中"->"串中"` 报 `source.internal` UTF-8 边界错、`同B hit` 失命中，以及 `s.len_bytes`/`s.char_at` 的 `av.closures` descriptor 边界实证。 AC-01 |
| T-02 / work | [x] T-01决定与可区分输入 | 仅修双端UTF边界/共享内部纯算法（提炼共享纯函数至 `ace_core::source::closures_fn::diff_single_region`，由 `closures_incremental.rs` 与 `closures_fn_session.rs` 共同调用），在原模块单测（含私有 `accepted_source` 检查）和 `closures_restore_tier.rs`（`audit1027_utf_01`）补永久反例；不搬AV宿主或修改profile。 AC-02 |
| T-03 / work | [x] T-02 | 运行真实fresh/warm/hit及原closures_restore_tier、closures_native_tier；核接受源码、对象/manifest身份、原warm计数，完成§6职责组。 AC-02/03 |
| T-04 / review | [x] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04 |
| T-05 / merge | [x] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05 |
| T-06 / merge | [x] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1把native会话入口写成AV类型，且未区分算法空串、完整程序与当前不支持的Unicode标识符；仅比较字符数也可能漏掉U+FFFD替换。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

本Plan r1已钉源/Spec文件无字节差额；额外读取的现测试/设施见§4。

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原2个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。

2026-11-13 stage: work | plan_id: PLAN-1027 | plan_revision: r2 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-1027/auto-lang（不存在） | branch: plan-1027-dev（不存在） | base_commit: 6584e94d4 | code_commit: 未产生 | dependency_revisions: 主检出 HEAD 6584e94d4；1028 已合入的 Spec 增量需开工时重读 | task_ids: T-01 未开始 | evidence: 主检出无代码路径 WIP；工作树列表无 1027；native diff 仍对新源码使用原字节切点及 lossy 转换，AV 版本同时校验两端字符边界，均为静态检查，未运行反例 | blockers: 会话文件系统写权限不覆盖计划规定工作树、计划材料目录及 Git 元数据 | next_responsible_role: work（恢复权限后建树并重核基线）。未建树、未构建、未修改实现或测试；保持 drafting 0/6。

2026-11-13 stage: goal | plan_id: PLAN-1027 | plan_revision: r2 | round: 1 | from: work | to: user | status: drafting | outcome: blocked | worktree: D:/autostack/.wt/lang-1027/auto-lang（未创建） | commit: 6584e94d4（基线，无实施提交） | next: user

2026-11-13 stage: new | plan_id: PLAN-1027 | plan_revision: r3 | outcome: pass（T-01真实入口有界修约，保持原目标/AC/Case/任务/范围不变） | worktree_path: D:/autostack/.wt/lang-1027/auto-lang | branch: plan-1027-dev | base_commit: 6584e94d4 | evidence: T-01 实测 `s.len_bytes()`/`s.char_at(0).to_codepoint()` 在 `av.closures` descriptor 报 `bind.unknown-case`（`hir_adapter.rs:4504,4510` 限定 `!self.collections()`，`closures_lower.rs:116-135` 仅支持 `StrLen`/`Str`）；按 §1 非目标不扩 profile，将 §6 实测输入修约为 `s.len()` + `"é"->"äé"`/`"中"->"串中"`/`"😀"->"😁😀"` PE 整数通道、`同B hit` 与私有 `accepted_source` 逐字节断言 | next_responsible_role: work（T-02/T-03）

2026-11-13 stage: work | plan_id: PLAN-1027 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1027/auto-lang | branch: plan-1027-dev | base_commit: 6584e94d4 | code_commit: ddfe18f7875d748dcb4a829a71730267294fa797 | dependency_revisions: master@6584e94d4（PLAN-1028 已合入 @7880f2a42；PLAN-1002 docs WIP 仅在主检出不影响本次 Rust/旧Case 交付） | task_ids: T-01, T-02, T-03（AC-01, AC-02, AC-03） | scope_changes: 在 `experimental/ace/crates/core/src/source/closures_fn.rs` 抽取 `pub fn diff_single_region(old: &str, new: &str) -> (Range<usize>, String)` 同时对齐 old/new 双端 UTF-8 字符边界；`experimental/ace/crates/av/src/closures_fn_session.rs` 与 `experimental/ace/crates/native/src/closures_incremental.rs` 复用该纯函数（无 `native->av` 依赖、无新 `public` 访问器）；`closures_incremental.rs` 补全 `audit1027_utf_02`（splice 逐字节、私有 `frontend.source().as_bytes() == requested.as_bytes()`、`nonfast:crosses-units`、非 ASCII 标识符受控拒绝与显式原生工具断言）；`experimental/ace/crates/native/tests/cases/closures_restore_tier.rs` 补全 `audit1027_utf_01`（多字节增/替换 PE `exit_code == 42`、`同B hit`、`d_main` object hash 不变、`MnDriver::build_closures` → `restore_closures_session` UTF-8 暖编辑与拒绝边界） | evidence: `t02-verify.log`: `ace-native --lib closures_incremental::tests` 1/1 passed (4.81s, exit 0) 与 `--test conformance closures_restore_tier` 3/3 passed (27.31s, exit 0)；`t03-groups.log` (`run-zz29v3vu/results.json`, wall 1291.68s): `ace-source` PASS 502/502 (excluded=1 ignored, native_unselected=1 performance), `ace-av` PASS 1088/1088, `ace-native` 首次跑 250 executed: 247 ok / 3 fail（`typed_entry_incremental_tier` 全部 3 例在冷编译期间并发触发既存 `typed_runtime.rs` 无跨进程文件锁的 `os error 2` 竞争；`audit1027_utf_02` ok 69.193s、`audit1027_utf_01` ok 110.342s、`closures_restore_tier` 3/3 ok、`closures_native_tier` 14/14 ok）；`t03-replay-typed-inc.log`: 孤立重放 `typed_entry_incremental_tier` 3/3 passed (2.28s, exit 0)；`t03-ace-native-replay.log` (`run-b3kew9dp/results.json`, wall 92.36s): `cargo t ace-native` PASS 250/250 ok (0 fail, excluded=0, native_unselected=1 performance)；修改文件零新增编译告警 | blockers: 无 | next_responsible_role: review（独立核证代码/Diff/Spec Delta 准备并触发 T-04）

2026-11-13 stage: goal | plan_id: PLAN-1027 | plan_revision: r3 | round: 2 | from: work | to: review | status: execution_done | worktree: D:/autostack/.wt/lang-1027/auto-lang | commit: ddfe18f7875d748dcb4a829a71730267294fa797 | next: review

2026-11-13 stage: review | plan_id: PLAN-1027 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1027/auto-lang | branch: plan-1027-dev | verified_commit: ddfe18f7875d748dcb4a829a71730267294fa797 | reviewer_identity: same-session（同一执行会话直接核阅 git diff 6584e94d4..ddfe18f78、源码、cargo check 与测试实跑日志；limitations: 非独立子代理/跨模型复审，已逐行核证所有变更文件与独立 oracle 输出） | checks: 1) `git diff --check 6584e94d4..ddfe18f78` 零空白/格式问题，仅改 4 个代码/测试文件（未动 Cargo.toml、未新增 `native->av` 调用、未新增 `public` 会话源码访问器）；2) 逐行核 `ace_core::source::closures_fn::diff_single_region(old, new)` 双端 `is_char_boundary` 收敛性与 `splice(old, range, replacement).as_bytes() == new.as_bytes()` 恒等性（含空串、同源、共享前后缀多字节、四字节 emoji）；3) 核 `audit1027_utf_01` 与 `audit1027_utf_02` 无静默跳过（显式 `assert!` 工具在位）、无 workaround，覆盖冷/暖/hit/`d_main` object hash 不变、`MnDriver::build_closures -> restore_closures_session` UTF-8 暖编辑、nonfast 路由及域边界拒绝；4) 核 `cargo check` 对改动文件零新增告警；5) 核 SD-01/SD-02 对 `master@6584e94d4` 现 Spec 无冲突（1028 增量与本计划正交）；6) 核 `ace-source` (502/502)、`ace-av` (1088/1088)、`typed_entry_incremental_tier` 孤立重放 (3/3) 及 `ace-native` 完整重放 (250/250) 证据齐全且首跑环境竞态（pre-existing `typed_runtime.rs` 冷并行 `os error 2`）已与重放绿分别留痕 | findings: 无阻断项、无延后/遗漏债务 | next_responsible_role: merge（T-05/T-06：在 worktree 落 Spec、验证并合入 master、对账并归档清理）

2026-11-13 stage: goal | plan_id: PLAN-1027 | plan_revision: r3 | round: 3 | from: review | to: merge | status: reviewed | worktree: D:/autostack/.wt/lang-1027/auto-lang | commit: ddfe18f7875d748dcb4a829a71730267294fa797 | next: merge

2026-11-13 stage: merge | plan_id: PLAN-1027 | plan_revision: r3 | outcome: pass | landed_commit: a425e9adfbcf6881a12b85315fac7a5ae1a16bcd（code @271aa374d + specs @a425e9adf） | canonical_specs: docs/specs/auto-ac/native-closures-source.md (SHA256 FCAFF1D24494F1EF2FDCCC848F872D55C3020913D67C5E79683D45B3505FEAC8), docs/specs/auto-ac/closures-function-production.md (SHA256 195AB8710AFF24CDB740C3A15A5E1771E60303A10109DC84CBE0AA810503C1C0) | output_reconciliation: 已逐项对账并清除 `D:/autostack/plan-artifacts/plan-1027`（7 文件 / 34,898B，含 `t01-preflight.log` SHA256 `83902DE6...`、`t01-probe.log` `115B0F1B...`、`t02-verify.log` `614DDD9D...`、`t03-ace-native-replay.log` `D3F2814E...`、`t03-groups.log` `0A3E2698...`、`t03-replay-typed-inc.log` `EB20F4D9...`、`t05-rebase-sanity.log` `475D19DC...`）及 `%TEMP%/plan1027-*`（3 临时目录 `plan1027-t01-53184`、`plan1027-unit-cross-53604-*`、`plan1027-unit-cross-57256-*`，共 10 文件 / 208,427B）；`.retention/policy.json` 已更新为 `deleted-at-merge (no holds)` | worktree_cleaned: true（`D:/autostack/wt-guard.sh` 对 `.wt/lang-1027/{auto-lang,auto-down}` 均输出 clean，已移除双工作树、组目录 `.wt/lang-1027` 与分支 `plan-1027-dev`） | batch_regression_check: `docs/plans/.last-batch-regression.json` 现记 `last_covered_plan_id=1015`、`timestamp=2026-11-12T13:22:41.317357+08:00`（距今约 33.5h < 48h，且 `1027 % 5 != 0`，本 Plan 自身不触及模 5 或 48h 到期；惟历史已跨越 1020/1025，建议整批后续适时按需运行 `/auto-plan:regress`） | archive_path: docs/plans/archive/1027-ace-native-utf8-edit-fidelity.md

2026-11-13 stage: goal | plan_id: PLAN-1027 | plan_revision: r3 | round: 4 | from: merge | to: done | status: archived | completion_kind: delivered | landed_commit: a425e9adfbcf6881a12b85315fac7a5ae1a16bcd | worktree_cleaned: true | outcome: completed | next: none

## 10. 待澄清事项

2026-11-13 恢复记录：原沙箱写权限阻塞已解除（当前会话已创建 `D:/autostack/.wt/lang-1027/auto-lang`、`D:/autostack/plan-artifacts/plan-1027` 并登记 `policy.json`）。

不扩closure签名/捕获支持；真实Rust/PE验证未执行。 T-01使真实反例/资源或IO故障可运行，未取得前不声称动态后果已证。等价实现不需重复审批；新支持/预算/公共合同变化先最小决定，不能用隐式门限或延期替原义务。不等待“新测试框架全部完成”。

本轮T-01入口已具体化，当前不需要用户再次确认等价内部实现。未取得真实动态反例的部分仍由本节与§6指定入口/邻例核证，不把预期当已观察结果。
