# PLAN-741 实现质量复审（2026-10-04）

结论：**needs_fix**。首条 HIR→Windows 原生闭环真实成立；本次新增边界反例揭示
四项核心缺陷，另有 CLI 文档与子进程截止时间问题。原有测试通过不覆盖这些反例。
本次仅复审，不改实现、不回滚交付，不重开已归档741；修复另开独立合同。

## 基线与独立性

- plan_id=PLAN-741，plan_revision=1，status=archived。
- reviewed_commit=c1ac219e73ee2ed1ef6ba8dfec49c131bfbf1a75。
- diff_base=951b6c70ff596f79e464ba977139669f2d196821；交付/ledger落点304519113。
  git merge-base --is-ancestor 304519113 HEAD 成功，实际实现已在主线。
- 本会话负责起草设计，未实施741；本次按源码、已提交diff及自行运行的反例判断，
  没有把执行侧勾选或旧pass当作本次结论。
- 在独立detached复审worktree执行；实施文件无未提交修改。
- 依赖：Cranelift0.126.2，Rust1.98.1，Windows x86_64/msvc，
  rust-lld与SDK发现使用本机状态。新反例没有外仓依赖。
- 此HEAD相对304519113的crates/、ac-core/、根Cargo、测试配置及验收脚本diff为空，
  本次重跑37项原型测试和fmt；旧主仓check/t/tv证据明确复用741归档§9
  （t既有26红，不声称全绿），不重复独占批量tf。
- 原有37项全部通过：text_binding11、hir_verify8、native_execution8、cli9、trace1；
  fmt通过。测试构建出现tests/common/mod.rs:138未使用descriptor import警告。
- 所有新增失败均从Atom reader→binder→verify→native/CLI真实路径产生。

## 输入指纹与Spec增量复核

| 输入 | SHA-256 |
|---|---|
| docs/specs/auto-hir/project.md | 04e95f7ce4805b7e409bf89af49fe9c43be0632157043470099e76e1fe83bee6 |
| docs/specs/auto-ac/project.md | 31f20c811950600a1afcdb5fb05a8199713188e7e3b54e1872dbd336f9870b17 |
| experimental/ac-core/Cargo.lock | 2b67753a80d4dd3bd35d3fd417628acc3f1d81c0029739637d64d8d54af85462 |
| experimental/ac-core/src/native.rs | 9f4f87534c0cd85618a21942ff803279d0cde6665296cbdaea79027103269e1e |
| experimental/ac-core/src/verify.rs | 1ddc9cf6669ead02948222347e009ec8922a6183271dda178b43e21be5dc0ae2 |
| experimental/ac-core/src/link.rs | a4b32ab4a8deff40d4c978ab29402aba197d18c4ca6eb042cc3d9bf0aa5918b3 |
| experimental/ac-core/src/main.rs | 400e67e65415aacdf4cf4a628de9f676dae97b0cfa14d46ba70ce900a25bddd4 |

SD-01/02确已沉淀到auto-hir/auto-ac Specs，当前增量不新增/覆盖Spec。
SD-01承诺owner和调用实参映射正确，受到QA-02/03反例挑战；
SD-02承诺bool为i32及失败构建不覆盖/子进程deadline，受到QA-01/04/06挑战。
要求修复实现并复验原承诺，不能通过缩减Spec掩盖问题。
supersedes=[]，new=[docs/specs/auto-hir/project.md, docs/specs/auto-ac/project.md]，
touched_goals=[]沿用归档合同，本次不发布canonical Spec/ledger。

## 发现（按风险排序）

### P741-QA-01 / P1：bool比较结果未归一到约定i32表示

位置：experimental/ac-core/src/native.rs:567（lt_i32）；:297与:388（locals/ABI为i32）。
Cranelift icmp返回i8，直接交给i32变量/签名。bool-local.atom的check=0，
build=101，panic为“declared type of variable var0 doesn't match type of value v2”；
bool-return.atom的check=0，build=1，Cranelift verifier拒绝返回宽度。
现有循环只把比较结果直接给brif，所以没有发现存储/参数/返回漏洞。
影响T-04，AC-04/05及SD-02。修复统一内部bool表示/转换，
补局部绑定、赋值、bool参数/返回与分支的真实native正例。

### P741-QA-02 / P1：按求值序号而非bindings校验形参类型

位置：experimental/ac-core/src/verify.rs:394–407（记录定位范围，非文件链接）。
代码比较eval_args[i]与params[i]，没有用bindings[param,arg]决定对应关系。
形参[i32,bool]、求值[bool,i32]、映射param0←arg1/param1←arg0本应合法，
binding-valid.atom却check=1；求值[i32,bool]配同一交换映射本应非法，
binding-invalid.atom却check=0，直到后端才拒绝。
影响T-03，AC-03及SD-01；CheckedModule被错误授予。
修复先独立验证映射，再按params[b.param]与eval_args[b.arg]比较；
保持eval_args副作用顺序。需不同形参类型的正反例，原全i32 trace不足。

### P741-QA-03 / P1：函数→body方向未校验owner与唯一归属

位置：experimental/ac-core/src/verify.rs:167–207；native.rs:306暴露后果。
verifier只遍历body检查body.owner对应函数，没遍历每个function反查body.owner。
shared-body.atom新增零参数alias，指向已有双参数add函数体，owner仍为d_add。
check=0，build=101，取alias入口参数时“index out of bounds: len 0/index 0”。
影响T-03，AC-03及SD-01。修复函数/body双向一致与唯一归属，
按每个实际函数签名校验body参数；补共享body、冒领body及同签名共享的反例。

### P741-QA-04 / P1：报告失败时旧制品已被部分替换

位置：experimental/ac-core/src/link.rs:198–211；main.rs:183–189。
exe先rename，之后写receipt，再放置obj。receipt写失败时CLI返回1，
但新exe已替换旧成功exe，obj仍旧。用原生exit5制品作基线，
将receipt路径改为目录，再构建exit1制品：错误link.receipt，exe哈希变化，
obj哈希不变，新exe实跑exit1。详见extra-results.json。
这不是模拟link失败；是真实发布阶段失败，违反“失败构建不覆盖”。
影响T-05，AC-07及SD-02。所有制品先准备/验证，再发布并对失败回滚；
覆盖receipt/obj权限失败、既有文件、并发或临时名冲突的合理范围。
不能只测试前置entry/linker失败的SENTINEL。

### P741-QA-05 / P2：README的CLI示例无法运行

位置：experimental/ac-core/README.md:29–41。
cargo run -- ac-probe check把ac-probe当子命令，真实exit2“unknown command”；
实际binary是auto-ac-prototype，Cargo.toml没有ac-probe bin。
去掉多余ac-probe后，add.explicit样例也没有d_entry，build仍entry.not-found。
归档计划链接仍指向已移动的active路径；descriptor路径有schema/schema笔误。
影响T-07，AC-07。统一bin命名或正确cargo命令，使用带入口的native fixture，
从README复制命令实际运行，并更新归档链接/路径。
验收脚本用cargo run -- check/build，故原script绿没有验证README示例。

### P741-QA-06 / P2：工具发现子进程没有硬截止时间

位置：experimental/ac-core/src/link.rs:31–34、:67–74。
find_rust_lld内rustc --print sysroot及SDK发现reg query直接Command.output，
没有走deadline机制。即使link/run有60s，CLI的发现步骤仍可无限等待。
这项为源码直接证据，未注入挂起的系统工具；不要误称已实测timeout。
影响T-05，AC-07。将发现/链接/运行统一到有deadline的执行器；
并采用并发pipe读取，避免现有run_with_deadline先等进程退出再读pipe造成输出阻塞。
测试用可控helper，不终止/篡改真实系统服务。

### P741-QA-07 / P3：测试构建有未处理warning

tests/common/mod.rs:138未使用descriptor import；本次cargo test打印警告。
范围小，但旧记录“健康检查无未处理warning”没有覆盖测试target。
修复清理import，并让相应检查覆盖all-targets。非核心阻断项。

## AC → 任务 → 证据对账

| AC | 任务 | 本次结果 | 证据与限制 |
|---|---|---|---|
| AC-01 | T-01/07 | pass | 根生产代码/依赖面零diff；复用对应已锁定依赖树证据 |
| AC-02 | T-02 | pass | text_binding11/11与往返/拒绝矩阵复跑 |
| AC-03 | T-03 | fail | hir_verify8/8，但QA-02/03新增反例打破类型/owner门 |
| AC-04 | T-02/03/04 | partial | descriptor/拒绝路径成立；QA-01暴露已承诺bool边界不一致 |
| AC-05 | T-04/05 | partial | add/count对象+PE正例全过；bool额外合法输入失败 |
| AC-06 | T-04/06 | pass | 溢出70、b→a/12和缺能力拒绝复跑 |
| AC-07 | T-05/07 | fail | QA-04真实失败覆盖、QA-05文档不可复现、QA-06截止漏洞 |
| AC-08 | T-07/08 | partial | 旧主仓门禁明确复用，新增完整性缺口需补测/复审 |

## 可复现证据与下一步

本目录fixtures保存5个缺陷相关Atom输入及1个loop-return控制样例，
results.json/extra-results.json保存本次退出码/诊断与制品哈希。
loop-return通过check/build且原生exit3，已排除为缺陷；不把未复现怀疑写成问题。
缺陷输入都是额外测试资料，未加入产品CI，未改原实现/既有测试。

在后续修复worktree内：
~~~powershell
cargo build --manifest-path experimental/ac-core/Cargo.toml --locked
pwsh -File docs/reports/741-quality-review-20261004/reproduce.ps1 -RepoRoot .
~~~
脚本有15s单子进程截止，输出到临时目录；记录结果而非把当前错误行为锁成新规范。
基线expectation见上文；修复后应bool合法程序原生成功、
合法映射check成功/非法映射check拒绝、共享body在verify拒绝、发布失败保留旧制品。
随后按变更范围跑原型全族及适用主仓门禁，再做独立复审。
建议先修QA-01..04，再补QA-05..07；此次不自动取号或实施。
