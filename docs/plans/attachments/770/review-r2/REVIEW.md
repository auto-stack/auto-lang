# PLAN-770 r2 第二轮最终复审：needs_fix

2026-10-14；stage=review；review_scope=final；outcome=**needs_fix**。
本次是用户请求后的独立审查上下文，未参与该次实施；从 Plan/Specs、实际代码、
独立门禁和反例重建裁决，未创建额外 agent。上轮 F-01 已关闭，但资源与可信门
尚有 8 个合同内阻塞项。没有修改实施源码、合入、归档或发布 canonical Specs/ledger。

## 1. 受审基线与证据

- Plan：PLAN-770，plan_revision=2；输入快照 [plan-input.frozen.md](plan-input.frozen.md)，
  SHA256=`E013E9A31C224F97D9F247AF3BD1EC6B1AEDF265EBC44E8E13CD74877A543B70`。
- 唯一实施树：`D:/autostack/.wt/lang-770/auto-lang`，branch=`plan-770-dev`。
- reviewed_commit=`5061c20a9e63b79da0d9bb505f35c40cfb549484`；
  base_commit=`e849e06f976816dda3db0c68a520d485a69904a7`。
- 入场与结束 `git status --porcelain` 均为空，HEAD 未漂移；本次探针产物仅在忽略的
  `experimental/ac-core/target/review-770-r2/`，持久化审查材料在主检出 `docs/plans/`。
- 前置 774 的 canonical 沉淀提交=`930d05fae798917ce2ab3026bcfa8d0ab81713d5`；
  772 已归档，受审基面包含 scalar 修复。当前 master 相对基面的 ac-core 实现及
  本计划相关 auto-ac/auto-hir/auto-acc Specs 无新差异；其它模块文档变化不覆盖本裁决。
- [baseline.json](baseline.json) 冻结 implementation/dependency/Spec/verification-config
  指纹；review skill SHA256=`DE80711504407FDA06701FFDF33619CCF03A7A5885010C300CFD8C3FF32F05EC`。
- Spec delta：[proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md)，
  SHA256=`A34D70C4D22F20279B276D6DF14D9374B8584770B94AF7F77AC0A60114EEEF68`。
  design.md SHA256=`59A5616EFFAC3770212960CBFB6CBB224C7E63823B500B9DC949140F17043BB1`。

## 2. 独立验证结果与边界

在受审 worktree 执行 `powershell -NoProfile -ExecutionPolicy Bypass -File
scripts/verify-ac-text-770.ps1`：**13/13 PASS**，原收据
`gate-20261014-132113.txt`，持久副本 [gate-pass.txt](gate-pass.txt)，
原始输出在 [gate-raw](gate-raw/)。

- ac-core：**200 passed / 0 failed / 1 ignored**；已有 ignored 是
  `link::tests::run_exe_60s_deadline_bounds_descendant_pipe_hold`，本计划未更改执行器，
  未把该手动 60s 验收测试计为本轮通过。
- runtime/text：**20 passed / 0 failed**。
- 两个 workspace check --locked --all-targets 无 warning，fmt --check clean。
- 旧 gates：741（-SkipMainGates）/745/767 均实际通过，767 覆盖 772 修复面。
- 48 个 diff 文件均为 ac-core/文本运行时/770 script/report 面；无产品 crates/、
  test/、packages/ 或 canonical Specs 修改；strict acc_inventory 的未改绑定前提成立。
- 未运行根 cargo t/tv/tf/docs_gen；本计划是独立 experimental workspace。
- 第一次 gate 在 scalar gate 的重建步骤遇到 exe 文件拒绝访问，见
  [gate-first-attempt-failed.txt](gate-first-attempt-failed.txt)。该窗口与本审查 CLI 探针
  重叠；停止所有 CLI 探针后串行重跑上述完整门禁，错误未再出现，不列为实现回归。

新增独立反例/合法控制：**17 个 source 输入 + 19 个直接 Atom/HIR 输入**。
其中 8 个合法控制 source/HIR 双腿均 check/build/run 符合预期；9 个 source 反例
及 2 个额外 HIR 变异均有原始 JSON。每个 JSON 记录 HEAD、输入 SHA256、check/build/
run 的全 32 位退出码、分离 stdout/stderr、读错误与 deadline 状态。
本次最终记录有 34 次真实 PE 执行（source 16 + HIR 18）；p09 双腿在前门被误拒，未运行。
没有把预期应拒却返回 0 的案例计为通过。

CLI 入口复用仓库既有 `AcContainedRunner`（来自 verify-ac-source-745.ps1），每个
check/build/run 60s deadline；未另造解释器、未用 shell 8 位退出码。
[replay.ps1](replay.ps1) 可复跑所有冻结输入；`-HirOnly` 仅跑 Atom 腿。
[bundle-probes.rs](bundle-probes.rs) 经受审 rlib 链接，真实 API 变异、确认变异，再进入
verify_text/capabilities_check/native::lower_object；输出在 [bundle-probes.txt](bundle-probes.txt)。

## 3. 上轮发现关闭核查

| 原 ID | 本轮结果 | 独立证据 |
|---|---|---|
| F-01 V-07 冻结 oracle | closed | 全 suite 实际执行 `v07_mark_wrapped_concat_args_comparison_and_short_circuit` 与 `v07_mark_wrapped_order_direct_hir_leg`；纯 COFF marks 对象确实与 runtime staticlib 共链。补充 c07 的文本返回函数包裹 concat 操作数：exit2、stderr `abCONCAT...FREE id=1`；c08 的文本比较操作数：exit0、stderr `ab`。source/HIR 双腿一致 |
| F-02 最终 HEAD 收据 | closed | 本轮完整 13/13 绑 5061c20a9，全原始日志持久化 |
| F-03 SD-02 before 粗糙 | partial / CLOSE-01 | 类型与两域内容已细化，但仍写 PLAN-767，现 canonical 来源写 PLAN-772；其它 delta 文字问题见 §5 |
| F-04 空 touched_goals 解释 | closed | 当前 Plan frontmatter 已解释不修改 GOAL-*、不关闭 GOAL-017；不因新增语言原语冒称 ACC Gen1 |

## 4. 阻塞发现（原合同缺口，不能通过债务登记豁免）

### P770-R2-QA-01 [P1] verify_text 未核验 Bundle 身份

位置：`experimental/ac-core/src/verify.rs:108-133`。
影响：AC-01/04，V-01/06，T-02。

verify_text 直接以 `Verifier { domain: Text }` 检查内容，没有像 verify_domain 一样
先比较 bundle.domain。真实 API 将合法文本 Bundle.domain 分别改为 Core/Scalar，
确认变化后 verify_text **均返回 CheckedModule**，checked.domain 仍是伪造域；随后
find_entry 成功，lower_object 按 Core/Scalar 分派，在 native.rs:795 panic：
`verifier: text expressions exist only in the text profile`。
反控：TypeId/LocalId=u32::MAX 均 verify.bad-reference 拒绝。

要求：在获得 Checked 前拒绝错域（verify.domain）；覆盖三个域的正反入口矩阵与
程序化换身份，拒绝模块不得进入 native。不能在后端捕获 panic 来替代前门。

### P770-R2-QA-02 [P1] 删除 requires 可绕过显式能力和 runtime 合同门

位置：`src/verify.rs:108`、`src/main.rs:405-411`，均在 experimental/ac-core。
影响：AC-01/04/06，V-01/06/10，T-02/05。

把合法含 Str 字面量、len 和清理的 Bundle.requires 从含 `ac.runtime.text.v1`
清成空，真实 verify_text 返回 Checked；capabilities_check(&checked, &[]) 返回 Ok。
同一 Bundle canonical 化为 [mut02-no-runtime-requires.atom](mut02-no-runtime-requires.atom)。
CLI `check`=0；`build ... --entry d_main --support-lib <unchecked-runtime.lib>` **未提供
任何 --capability，所用库确实无 .contract.json**，仍 build=0，PE exit3、正常 LIT/FREE。
所用 unchecked-runtime.lib 是当前真实 runtime lib 的字节复制，不含 sidecar，
不是空库/伪 backend mock。构建收据 [mut02-no-runtime-requires.ac-link.txt](mut02-no-runtime-requires.ac-link.txt)
没有 runtime-lib/runtime-contract-sha256。

原因：运行时需求仅取 caller-controlled requires；finish_build 用该字符串决定是否
校验合同，实际发射的 ac_rt_* 调用没有反向证明义务。
要求：由验证后的真实操作/清理/签名需求建立 mandatory runtime requirement，
在前门拒绝缺声明并在 build 要求提供能力/真实库合同；保留无文本调用的 scalar
控制仍不要求 runtime。覆盖移除声明、缺提供、缺/错 sidecar 的完整构建与三件旧制品保护。

### P770-R2-QA-03 [P1] move 清理未按实际控制流分配责任

位置：`src/text_resources.rs:768-800`、`1009-1021`，以及 scan_expr/Logical 的
无条件 move 汇总；后端按错误 exit_frees 发射。
影响：AC-03/05，V-04/07，T-02/05。

| 输入（source 与同内容直接 HIR 均复现） | 正确义务 | 实际 |
|---|---|---|
| p01 `return consume(x.move)` | callee 清理一次，caller 不再释放 | check/build0，exit3，`FREE id=1` 后 `FREE-INVALID` |
| p02 `if true { let n=consume(x.move) }; return 0` | 执行 move 的路径不再持有 x | check/build0，exit0，`FREE id=1` 后 `FREE-INVALID` |
| p08 `false && consume(x.move)` | 未执行 RHS 时 caller 仍承担 x 清理 | check/build0，exit0，仅 `LIT id=1`，无 FREE |

反控 c02 独立 let 调用 move、c05 未执行 if-move、c06 已执行短路 RHS 均正常释放一次。
return 从 move 前 st_snapshot 建 frees，未排除本表达式移交值；merge_children 合并
dead，却仍保留 parent.live 的清理责任；scan_expr 把条件 RHS 的 move 无条件记死。

要求：修复 return/if/join/Logical/loop 相关责任模型，不把全路径“不可再读取”直接
等价成“全路径已释放”。覆盖有/无 else、两边移交、分支退出、嵌套短路、return
嵌套调用及 break/continue 的代表矩阵，确认每个实际分配唯一 FREE 且无 FREE-INVALID。

### P770-R2-QA-04 [P1] 默认 view Str 被当作 owned 返回

位置：`src/text_resources.rs:678-700, 770-780`（ReadLocal 的 owned 判定）。
影响：AC-03/04，V-04/06，T-02/05。

[p04-borrowed-param-owned-return.at](p04-borrowed-param-owned-return.at)：
`fn steal(s Str) Str { return s }`；main 同时拥有 x 与 `let y Str=steal(x)`。
参数 Str 按 774 D-03 默认为 view，callee 无权转移 caller 的 buffer。
实际 source/HIR 均 check/build0，PE exit3、一次 FREE 后 FREE-INVALID。
`let y Str = s` 的同类借用复制也未在 owned.contains(src)==false 的默认参数路径禁止。

要求：区分名义 Str 类型与持有清理权；只有独立 owned 构造/合法 move-in/拥有结果
可转移返回或拥有绑定。默认 view 与 mut 参数不可不经 clone/显式拥有拷贝取得清理权。
补默认 Str 参数与 move Str 参数的相反控制、局部绑定/返回两个出口，source 和 Bundle 均验证。

### P770-R2-QA-05 [P1] 拥有临时与借用调用返回的 provenance/寿命断开

位置：`src/text_resources.rs:157-217, 703-715`；`src/native.rs:2392-2453`。
影响：AC-03/04/05，V-04/06/07，T-02/05。

- p03 `let v str=make().to(str)`（make 返回拥有 Str）：source/HIR 均 check/build0，
  PE exit3、`LIT id=1` 无 FREE；应以 verify.borrow-escape 拒绝跨语句存临时 view。
- M05 确认变异 StrLit 的 TypeUse 为 StrOwned，经 TextToBorrowed 存 str local：
  verify_text 仍接受；[mut01-owned-literal-view.atom](mut01-owned-literal-view.atom)
  check/build0，exit3、同样漏 FREE。扫描不把 owned Call/owned StrLit 计为拥有临时。
- p07 合法语句内使用：`id(("a"+"b").to(str)).len()`，id 返回借自参数的 str。
  source/HIR 均 check/build0，PE **exit70**，stderr
  `CONCAT id=1 → FREE id=1 → RT-FAIL code=-7`。临时在借用返回被消费前释放，
  读取已释放缓冲；具体运行期失败码可能随释放后内存内容变化，正确 oracle 是
  生命周期顺序+值2，不能以改 UTF-8 检查掩盖 UAF。
  c03 的具名拥有者借用返回控制正常 exit3，free 在消费之后。

原因：scanner 仅追 owned-producing 的少数语法节点，没有完整表达式来源分类；
借用 Call result 的 TVal.backing=None，参数临时在 call 后立即 post_free，
不是计划承诺的完整语句尾。
要求：真正跟踪拥有结果、借入参数、常驻字面量、mut 与 move-in 的来源以及
借用返回与实参的联系。语句内合法借用保持 backing 至所有消费者结束；跨语句/
跨帧非法借用前门拒绝。补 call→slice/to/clone/return/let/assign 的代表路径。

### P770-R2-QA-06 [P2] mut 声明被接受但没有排他与禁止外逃检查

位置：`src/text_resources.rs:check_body/owned_reach/validate_moves`；
`src/verify.rs:775-796` 仅检查 mode/type 组合。
影响：AC-03/04，V-04/06，T-02/04。

- p06 `both(mut a str,b str)` 调用 `both(x.to(str),x.to(str))`：同一来源 mut+view，
  source/HIR check/build0、PE exit6；应在 verify.mut-conflict 拒绝，零 native 调用。
- p05 `fn escape(mut s str) str { return s }`：source/HIR check/build0、exit3；
  774 D-03 明确整个函数体的 mut 区域，返回其 view 为 verify.borrow-escape。
- c04 两个不同拥有来源传给 mut/view 参数 source/HIR 正常 exit4，LIFO free2/free1。

本 profile 暂无写方法不能豁免这一前门承诺；design.md §4.3 同样保证排他性。
要求：补调用点来源别名检查、mut 参数及其派生 view 的区域/返回检查、同源双 mut/
mut+view/间接借用的负例，保留不同来源与合法只读参数返回控制。

### P770-R2-QA-07 [P2] 循环内每轮新建拥有值的合法 move 被误拒

位置：`src/text_resources.rs:838-846` 与 Let 状态更新。
影响：AC-03/05，V-04/09，T-02/04。

p09 每轮 `let x Str="abc"; let n=consume(x.move)`，两轮分别分配/转移/清理，
不是把循环外同一个已移交 x 再用。source/HIR 的 check/build 都 exit1，
首次使用也报 verify.use-after-move。预扫 moves_inside 不区分循环内初始化的
新所有者与循环外所有者，入体前全部标死；Let 不清除这项 dead。

要求：按循环/初始化作用域重建每轮拥有义务；在 body 每轮 fresh 的所有者可正常
move，不允许循环外所有者重复 move，也不把非法路径降到 runtime。补两类正反
控制与 continue/break、嵌套 if 的组合。

### P770-R2-QA-08 [P2] runtime workspace Cargo.lock 未提交

位置：`experimental/ac-core/runtime/text/Cargo.toml:16-26` 及缺失的
`experimental/ac-core/runtime/text/Cargo.lock`。
影响：AC-06，T-03/06。

Plan T-03 记“锁文件入库”，但 `git ls-files experimental/ac-core/runtime/text/Cargo.lock`
为空，`git check-ignore -v` 命中根 .gitignore:8 Cargo.lock。
当前 ignored 文件 493 行，含独立 workspace 的注册表依赖，观察 SHA256
`36A992D94B5E272160E7558AA7338224187E07DC3CF7054F8D6557978B7E1AFC`，
副本 [runtime-text.Cargo.lock.observed](runtime-text.Cargo.lock.observed)。根 ac-core lock
不是该独立 workspace 的 lock。当前 --locked 测试成功依赖本地忽略输入，不能用
HEAD 在新检出复现冻结依赖或真实 library 指纹。

要求：将该 workspace 的真实 lock 纳入受审提交（可 scoped force-add，无需扩大
根 .gitignore 改动），独立干净输入验证 --locked 并更新 hash/最终 gate 收据。
T-03 runtime 实现/20 测试保留历史通过，但锁文件交付子项必须重新打开。

## 5. Spec delta 审查与遗漏扫描

SD-01..05 目标均为真实 repository-relative docs/specs 路径，metadata 的
supersedes/new entries 方向正确；touched_goals 为空有书面解释。仍不得发布，
因为资源/验证器的“已实现”断言有 §4 合同内缺口。

**P770-R2-CLOSE-01**（P2，文档纠正，不批准缩围）：冻结提案 SD-01 §3 将
len_bytes 与 len/char_at/slice 一起写成 Unicode 标量，违反 774 D-02 的显式字节长度；
应分清 len 是标量数、len_bytes 是字节数。SD-01 §9 的 trace 共链限制需精确指向
两个 no_std staticlib，明确本次 marks.obj 共链已可用。SD-02 before 的 PLAN-767
来源应和现行 PLAN-772 canonical 对账。修文字无需修改语言合同 revision。

没有发现产品 crates/old auto-lib 扩围、record/List/enum/use/ACC Gen1 的虚假宣称。
预算/故障注入仍由 runtime tests 承担，为 §9.2 既有边界；char_at 越界 -10 符合
冻结 ABI；未要求新语言故障注入 API。771 尚未合入，无 record 联跑义务。
本轮不把任何 §4 缺口延期到 merge。债务登记仅用于追踪，不能视为验收已完成。

## 6. AC → Task → 实物 → 结果

| AC | 结果 | 本轮证据与影响 |
|---|---|---|
| AC-01 | fail | 既有 triple/writer/isolation 测试通过，但 M01 换域获得 Checked、M06 能力声明删除被接受；QA-01/02 |
| AC-02 | pass（值语义受测面） | v02/v03 + source char/Unicode + runtime textops 均新鲜；UTF-8/NUL/组合符/边界/显式码点 oracle 保持；资源关联另由 AC-03 拒 pass |
| AC-03 | fail | p01..p09、mut01 source/HIR/Bundle 真实路径，重复释放/漏释放/UAF/模式缺失/循环误拒；QA-03..07 |
| AC-04 | fail | 既有 TypeId、类型、引用变异通过拒收控制，但 M01/M05/M06 与 p04/p05/p06 非法 Bundle 仍获得 Checked；QA-01/02/04/05/06 |
| AC-05 | partial | 上轮 V-07 mark oracle关闭，c07/c08 补充双腿正常；V-08/V-09 现存测试通过，资源顺序/短路与每轮 fresh move 缺口 QA-03/05/07 |
| AC-06 | partial | 13/13 新旧 gates 通过；scope/产品 hash 记录有效，但 ignored runtime lock、Spec 文本仍需修；QA-08/CLOSE-01 |
| AC-07 | fail | 独立审计已完成；8 个合同内阻塞项未清零，不能 reviewed |
| AC-08 | pending | merge-owned，未实施 closeout |

## 7. 返工交接与第三轮入场条件

本轮是第二次 final needs_fix，不因 F-01 已关闭或提交变化重置轮次。
按 review/work 技能，下一次 ready-for-review 前，work 必须先记录诊断与覆盖/设计
调整表（剩余 invariant、为什么前两轮漏测、影响 paths/states、实际修复与永久 oracle、
当前结果），本审查提出的冻结输入即共享 replay set。

重点是完整模型，不只特判这 9 个 source 示例：

1. 身份/实际 runtime call needs → verifier 拒绝 → capability/contract → native admission。
2. 名义类型与清理权分离；五类来源、拥有 Call/literal、借用返回联系、mut 活跃区域。
3. return/branch/Logical/loop/fresh initialization 的执行路径与清理责任，保留有效控制。
4. source/HIR/真实 Bundle 变异、真实 PE 配对 trace、静态负例零后端调用、失败制品保护。
5. 已提交 lock 与最终 HEAD 13 步门禁；修复后的 SD 冻结、文档/清偿表重建。

只修实施树 `plan-770-dev`，先提交再交新 final review；保留旧记录和反例。
Plan 回 executing，受影响任务 T-02/03（lock 子项）/04/05/06/07 重新打开，
T-01 保留完成，current_step=1/9。实现/联合正确性不能推给 T-09。
merge/archive/SD沉淀/ledger/index/实际计数/批回归到期判定/guard 清理仍 pending。

## 8. 重放命令

CLI 输入已冻结为 source 与 Atom，不依赖生成脚本才能重放；在实施树：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/770/review-r2/replay.ps1
```

先跑既有 ac-core test suite 构建 tool binary/runtime/marks.obj，CLI probes 与 Cargo
build/gate 串行，避免 Windows 正运行的 exe 无法替换。

程序化 Bundle 变异重放：

```powershell
$probeDeps = (Resolve-Path experimental/ac-core/target/debug/deps).Path
$probeLib = Join-Path $probeDeps 'libauto_ac_prototype-fe024210a31b39c2.rlib'
$probeOut = Join-Path (Resolve-Path experimental/ac-core/target/review-770-r2).Path 'bundle-probes.exe'
rustc --edition 2021 D:/autostack/auto-lang/docs/plans/attachments/770/review-r2/bundle-probes.rs --extern "auto_ac_prototype=$probeLib" -L "dependency=$probeDeps" -L native=C:/Users/zhaop/.cargo/registry/src/mirrors.aliyun.com-0671735e7cc7f5e7/windows_x86_64_msvc-0.52.6/lib -C link-arg=/STACK:33554432 -o $probeOut
& $probeOut D:/autostack/auto-lang/docs/plans/attachments/770/review-r2
```

上述 rlib 名绑定本轮构建配置；修复后以该新构建实际 rlib 路径替换，记录新 hash，
不可消费旧 rlib 伪造清偿。M01 用 catch_unwind 观察后端失守，不是产品修复或验收豁免。
