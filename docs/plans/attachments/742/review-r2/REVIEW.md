# PLAN-742 rev 1：第二次独立 final review（2026-10-13）

结论：**needs_fix**。上一轮具体输入已转绿，F-742-R1-04 告警清偿；
字面量保持、作用域及返回出口不变量仍未完整闭合，并发现文件端点修复回归。
不能把四例 ALL-PASS 或完整回归全绿等同于整个发射路径正确。

## 基线

- reviewer：Codex `/root`，曾进行第一轮审核，未参与此次实施，无子 agent。
- review_scope=final；plan_revision=1；这是第二次 final needs_fix，不重置轮次。
- 实际实现工作树：`D:/autostack/.wt/lang-742/auto-lang`，`plan-742-dev`。
  HEAD=`6d93cfdb73f5178b624846d46f88614ed999c158`；
  base=`c151935f1b50a9aefb7164771bcc70ccf4b66c9d`。两文件实现改动已提交，
  crates/Cargo/config/stdlib 无 WIP；预存未跟踪 review-r1 附件保留，不作为未提交实现。
- 共享 Plan 从主检出读取并冻结为 [plan-input.md](plan-input.md)，入口 main
  `685026eb5103ff3284c3022abf8de7699c4eb1db`。未使用实施树的旧 Plan。
- auto-down 组内 sibling=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，干净；
  a2r-std=同工作树提交；Notes 当前源码 HEAD=`1391bd7eb2f6b3b08e9841d30b25125e47ced058`。
- [baseline.json](baseline.json) 固定源码、依赖、配置、Spec 和制品 SHA256。
  probe 唯一绑定 Cargo JSON compiler-artifact 的 auto_man rlib，带原 native linked_paths，
  不使用模糊 rlib glob 或实施者缓存 probe。所有生成、编译、测试在实际工作树执行。
- 规范目标沿用 SD-742-01～04；[冻结增量](spec-delta.md) SHA256
  `9B4CA3167BBA44D5389B7840AF07499A025B63EDFD9B1B171D5D9E32B507AF28`。
  没有修改 canonical Specs 或 live ledger。

## 上轮闭合与本轮完整不变量

| 上轮 Finding | 旧例重放 | 完整不变量 / 当前结果 |
|---|---|---|
| F-742-R1-01 | List/brace 字符串实编运行正确，api.at 与 repository.at SHA256 均与 r1 相同 | **partial**；下游借用 pass 仍改业务字符串，见 R2-01 |
| F-742-R1-02 | repository.filename 保持原值；真实模块调用对照 pass | **partial**；嵌套名字遮蔽被扩至整个 endpoint，File 体漏限定化，见 R2-02/04 |
| F-742-R1-03 | 原 api.at/repository.at 精确输入重生成，list/get/update 三 handler 实编运行：`list=0; get=404; update=ok:0`，原 E0308 关闭；新增 oracle 独立复跑通过 | **partial**；Option 变量和 void 提前返回仍类型错误，见 R2-03 |
| F-742-R1-04 | 删除 NL、旧 wrapper 和 old borrow helper；源码/编译日志核对无三项残留 | **closed**，不扩大到基线其他告警 |

原 fixture 保存在 [original-fixture](original-fixture/api.at)，运行记录
[runtime.txt](original-fixture/runtime.txt)。D3 的类别级来源及未提交 WIP 无源行归因
已检查，可接受为历史材料边界；新增实编 oracle 能发现非法 Rust 比较，不再只靠
`== ==` substring。不要求凭空重建遗失 WIP，也不冒称重放了那个原始输入。

## 本轮 Findings

### F-742-R2-01 / P1：后续借用 pass 仍改写字符串

归属：原字面量保持要求未完整修复；关联 R1-01/R1-02，AC-742-01/02，T-1/T-2/T-3。
位置：`crates/auto-man/src/api_gen.rs:2821`，尤其未经 mask 的 `.find(marker)`。

API 返回合法字符串 `"crate::repository::take(value)"`，同项目 companion 定义
`pub fn take(value str) int`，生成 Rust 字符串却成为
`"crate::repository::take(&value)"`。生成 exit 0，handler 实编及运行确认错误返回值。
新的 qualifier 不修改字符串，但随后的 `borrow_string_module_call_args` 对整份
api.rs 搜索 marker，继续把字面量当调用。

证据：[api.at](cases/borrow_literal/api.at)、[api.rs](cases/borrow_literal/api.rs)、
[generated-results.json](generated-results.json)。修复须覆盖完整后处理链的字面量/注释保持，
借用插入只作用于真实调用；不能只给 qualifier 加 mask。保留真实 &str 调用正例。

### F-742-R2-02 / P2：嵌套局部名字遮蔽了外层模块调用

归属：修复引入的作用域回归；AC-742-02，T-1/T-2/T-3。
位置：`endpoint_body_local_names`（api_gen.rs:2787）及 qualifier（:2713）。

先执行 `var n int = repository.ping()`；随后仅在 `if false` 子块内声明
`var repository str = "local"`。外层模块调用在真实语言作用域内有效，但局部名收集器
把所有 emitted let/for 名字汇总成整个函数集合，导致外层调用不限定化，生成
`repository.ping()` 并报 E0423。只删掉子块的对照调用生成 crate 路径、实编运行返回 ok。

证据：[scope_shadow](cases/scope_shadow/api.at) 与
[compile.log](cases/scope_shadow/compile.log)，对照 [module_control](cases/module_control/api.at)。
修复使用实际 binding 和声明可见范围；覆盖声明前、块内/块外、for/pattern/参数形式，
不把某个同名局部存在于任意子块视为整个 endpoint 都被遮蔽。

### F-742-R2-03 / P2：返回包装只关闭原样例，出口矩阵仍有类型错误

归属：原返回适配要求的剩余缺口，以及新增 void 修复逻辑不完整；AC-742-04，T-1/T-2/T-3。
位置：`wrap_inline_return`（:2545～2600）和 `saw_return`/尾状态逻辑（:4223～4247）。

1. `GET /api/echo/:id -> ?Probe`，局部 `p ?Probe = Some(Probe { ... })`，
   `if id > 0 { return p }`。生成签名为 `Result<JsonResponse<Probe>, StatusCode>`，
   但返回 `Ok(JsonResponse(p))`，把 Option<Probe> 交给 Probe，E0308。
   合法 Option 值应 Some→200、None→404；不能按表达式是否拼写 Some/None 判断其类型。
2. 空返回 POST，`if false { return }; var x int = 1`。裸 return 实际转译为
   `return None;`，VoidStatus 模式未转换，报 E0308；且 `saw_return=true` 把分支内
   有 return 误当作所有路径都返回，尾状态未补，fallthrough 另报 expected StatusCode, found ()。
   相同函数仅去掉 return 的对照实编成功。

证据：[option_variable](cases/option_variable/api.at)、[编译错误](cases/option_variable/compile.log)，
[void_branch](cases/void_branch/api.at)、[编译错误](cases/void_branch/compile.log)，
[void_control](cases/void_control/api.at)。修复按实际返回类型和控制流出口处理；
将这些输入连同 Some/None 直接返回、普通值+Result 及 void 有/无 path 放入实编/值对拍矩阵。
本轮不把路径 void 相似源码判断冒充已运行，已实际运行的是无 path 两例。

### F-742-R2-04 / P1：限定化迁移遗漏 File handler，原调用支持回退

归属：移除整文件 pass 时引入的入口遗漏；AC-742-02/04，T-1/T-2/T-3。
位置：`generate_api_rs` 文件端点体循环（:3926～3936）与 JSON 专属 qualifier（:4226）。

合法 FileResponse handler 通过 `repository.path()` 获取路径，再调用
`http.file_response("D:/files", path, "{}")`。生成成功，保留非法
`repository.path()`，handler 实编报 E0423。将取路径替换为字符串常量的对照
实编、运行 metadata stub 返回 readme.txt。旧版整文件 qualifier 覆盖这种调用，
新代码只在 JSON 体循环限定化，File 分支在此前 continue，遗漏调用。

证据：[file_module](cases/file_module/api.at)、[compile.log](cases/file_module/compile.log)，
对照 [file_control](cases/file_control/api.at)。这是旧入口受修复影响的回归检查，
不要求扩展 File API。共享每个体入口的限定化并按其返回种类分流；Upload 分支也
有独立体循环（:4054），本轮只做源码检查、未运行 Upload companion 反例，明确待查。
metadata stub 不读真实文件、不代表 HTTP wire 成功；编译错误来自原生成调用本身。

## 当轮证据与验收

| 检查 | 结果 |
|---|---|
| `cargo test -p auto-man --lib api_gen` | **46 passed / 1 ignored**，2.47s；[api-gen.log](api-gen.log) |
| `cargo t --no-fail-fast` | **5278/5278 pass**，75.396s，1544 skipped；[daily.log](daily.log) |
| `cargo tt --no-fail-fast` | **5649/5649 pass**，105.307s，1792 skipped；[trans.log](trans.log) |
| `cargo check -p auto-lang`、`cargo build -p auto-man --message-format=json` | exit 0；存量告警保留，r1 三项 dead code 不再出现；[check.log](check.log)、[build JSON](auto-man-build.jsonl) |
| 两触面文件 rustfmt、git diff --check | exit 0；[format.log](format.log) |
| 独立 public generate_api 13 个 case，逐个 handler 实编/运行 | 8 个符合预期；1 个实编成功但值错误；4 个编译失败。全部 generation Ok；[完整结果](generated-results.json) |
| 原 E0308 fixture 额外精确重放 | 三 handler 全部实编，list=0、get=404、update=ok:0；[证据](original-fixture/runtime.txt) |

13 case 对应：4 个 r1 精确字符串输入、escaped 字符串、borrow marker 字符串、
模块正常/嵌套遮蔽两例、void 无 return/分支 return 两例、Option 变量、File 常量/模块两例。
输入与生成物 SHA256 在结果中；四个旧例的 API 和 companion bytes 均与 r1 相同。
执行 runner 保留真实生成的签名和函数体，以等价 wrapper/type stub 隔离实编；
没有修改生成 handler，不冒称编译了完整服务器或跑了 HTTP wire。

| AC | 结果 |
|---|---|
| AC-742-01 | fail：构造/拆分旧例已绿，整个发射链字面量仍受 R2-01 影响 |
| AC-742-02 | fail：R2-01/02/04；合法 companion 调用和业务值保持未闭合 |
| AC-742-03 | pass 于可考材料/当前单比较实编；失落 WIP 的原具体输入不能重放，边界已注明 |
| AC-742-04 | fail：R2-03/04；原 E0308 样例已关闭但所有承诺出口未关闭 |
| AC-742-05 | partial：work 电池材料存在，本轮没有 Notes 新双端/legacy 冒烟运行；框架存在稳定阻塞，修复后再绑定新提交复验 |
| AC-742-06 | partial：r1 D7/D9 Rust 控制可重用；当前 Notes 主题/VM 联动仍未独立运行 |

重用范围：sqlite.rs/fs.rs/a2r_std.rs/vue.rs、Cargo.lock、验证配置与 r1 指纹一致；
本次仅 api_gen.rs/trans/rust.rs 变动，D7/D9 宿主源码和 fs dispatch 未改，UI 生成族未变，
故重用 r1 的 D7/D9 和 tu 862/2 ignored 证据。不把重用扩为本轮 Notes 双端运行。
没有触及 aavm/docs schema，未跑 taa/单独 docs_gen；tf 属 merge-owned 批量门。

work-r2 电池摘要写明 fails=0，36 断言；独立阅读脚本可见 S10 是外部驱动提醒，
不能仅因 stdout 出现 S10 就证明本轮 kill/restart 已执行。work-r2 没有 CLI SHA、
独立 S10 记录或 Vue/VM 结果，未据此签发 T-3 完成。此为证据边界，不冒称 Notes 产品失败。

## 第二次失败后的诊断与 next

| 剩余不变量 | 前一修复/测试为何漏掉 | 下一次 work 交接必须说明的调整 |
|---|---|---|
| 完整发射链字面量保持 | 仅给两个 text pass/qualifier 加 mask，borrow pass 仍整文件搜索；测试字面量不含已限定调用 marker | 盘点所有后处理阶段和消费者；明确每个改写的语法身份及字面量/注释保护，运行借用正例和字面量负例 |
| binding 的真实作用域 | endpoint 局部名集合扁平化；oracle 没有同名子块 | 按实际 AST/binding 和声明范围处理；保持外层模块与内层局部各自有效 |
| 全入口调用可达 | 全文件 pass 退役后只替换 JSON 循环；测试均 JSON | 逐 JSON/File/Upload 入口对账已迁移的资格化/借用链；失败入口与合法对照均实编 |
| 类型/出口完整返回 | 根据 Some/None 文本分类，saw_return 只查“任何 return”；oracle 仓储 stub 为空且没有 void | 按实际 Option 值和 fallthrough 处理；实编+值对拍覆盖两向分支、200/404、void/path/提前退出及普通值 Result |

next：`/auto-plan:work 742` 在同一工作树修复，提交前附诊断/矩阵调整与永久回归结果，
再独立 review。T-1/T-2/T-3 重开、0/3、executing；保留旧例关闭和已清偿告警事实。
这不是增加任意新规模或语言支持要求，而是验证原 parity 和修复自身已经修改的路径。

重放：`& D:/autostack/auto-lang/docs/plans/attachments/742/review-r2/replay.ps1 -Build`，
然后执行同目录 `execute-generated.ps1`。脚本退出 0 表示观察执行完成；
必须逐项对照本文 expected/actual，不能把所有 case 都产出记录称 ALL-PASS。
旧 fixture 实编/运行的 runner 与原输入已冻结；后续修复必须重新生成而非复用其旧二进制。
