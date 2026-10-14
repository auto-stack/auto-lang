# PLAN-770 验证收据（verification）

- 计划：PLAN-770 r2 · T-01..T-07（work 阶段，AC-01..06 证据；含 r4 返工）
- 工作树：`D:/autostack/.wt/lang-770/auto-lang` @ `plan-770-dev`
- 基线：工作树基面 `e849e06f9`；r6 起已整合当前主线（771 先合入，merge
  基 `6787bbd4a`）；本收据时 HEAD `314cbbeb6`（r7 返工提交；受审基线
  e01997dc5/643a1b3eb/7e0b9f712/a14f1470b/ebd0d05ee/1f8027500/42b0ba7da/92c14fe6c）
- 日期：2026-10-14
- 性质：**语言面真实 native 编译收据**（source/HIR → 真实 PE，runtime 真实
  分配/复制/释放）；不是完整 Auto 编译器，不宣称 ACC Gen1。

## 1. 交付物与提交

| 提交 | 内容 |
|---|---|
| 6dbbf2712 | T-01 设计冻结 + T-02 文本域 HIR/descriptor/verify 前门（text_hir 16/16） |
| e03bb725a | T-03 runtime/text 静态库（23 符号 + contract sidecar；19→20 测试） |
| 860c33dec | T-04 (1/3) 文本词法 + SourceProfile::Text 骨架 |
| 6cb5ec305 | T-04 (2/3) source 文本面端到端（text_source 10/10） |
| c357efdb4 | T-04 (3/3) CLI usage 文案 |
| 3e4b8620b | T-05 (1/3) text 模块发射 ac.runtime.text.v1 requires |
| 53e8ac7c4 | T-05 文本 native lowering + runtime 合同门 + 真实 PE 测试（text_native 18 + text_resources 10） |
| a87de0bd7 | T-06 verify-ac-text-770.ps1 + T-07 README/reports（r1 复审受审基线） |
| c4e169cec | F-01 路径 A 返工：test-support/marks.rs 纯 COFF 对象 + v07 冻结 oracle 恢复（source+HIR 双腿）+ F-02/03/04 处置（gate 收据 gate-20261014-124436.txt 绑本 HEAD） |
| 42b0ba7da | r4 返工：P770-R4-01..05（块出口寿命入状态/incoming-only join/入口 owner join/包装排他 fixpoint 传播/短路 RHS 临时路径合流）+ text_review_r4 17 测试永久化 |
| ebd0d05ee | r5 返工：P770-R5-01/02（LHS 临时边界冻结于 LHS 完成后/break-continue 目标出口状态由目标 loop 消费）+ text_review_r5 11 测试永久化 |
| 7e0b9f712 | r6 整合：master（771 record）并入同一实施树——ParamMode/MethodOut/CallFrame/profile 防线/词法 mut 关键字/算子面谓词统一调和，text/record 域隔离保持 |
| 643a1b3eb | r6 返工：P770-R6-01/02（循环重入 fixpoint+构造代际/不可达出口按帧可达性过滤）+ record gate 接入联合 text gate + text_review_r6 9 测试永久化 |
| 314cbbeb6 | r7 返工：P770-R7-01（终止态循环体不伪造回边——back-edge 分类显式化：continue 恒回边、body 态仅在可自然落空时回边）+ text_review_r7 10 测试永久化 |

## 2. 命令与结果（唯一工作树）

| 命令 | 期望 | 实际 | 收据 |
|---|---|---|---|
| `cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets` | 零错误零警告 | PASS | gate 原始输出 |
| `cargo test --manifest-path … --locked`（ac-core 全套，整合后含 record 面） | 全绿 | **331 passed + 1 ignored**（r7 净增 text_review_r7 10；r2/r3/r4/r5/r6 重放与 record/scalar/core 面在整合树上全绿） | raw core-test.txt |
| `cargo fmt --manifest-path … --check`（ac-core 与 runtime/text 两处） | clean | PASS | gate |
| `cargo check/test --manifest-path experimental/ac-core/runtime/text/Cargo.toml --locked` | 全绿 | **20/20**（fault 6、lifecycle 5、smoke 2、textops 4、trap 2、contract 1） | raw rt-test.txt |
| `powershell … scripts/verify-ac-text-770.ps1` | r6 起 14 步全绿（+record gate） | **PASS**（check/test/fmt ×2 + 旧 gates 741[-SkipMainGates]/745/767 + **verify-ac-record-771** 实跑 204.9s + scope 隔离 + 指纹）；绑 `643a1b3eb` 收据 gate-20261014-193430（收据卫生：本行旧引用 gate-20261014-155448/987172981 为 **r3 历史收据**，非现行绑定——r6 复审指出的陈旧表述在此更正） | gate-20261014-193430.txt |
| `powershell … scripts/verify-ac-record-771.ps1`（已合 record gate 独立复跑） | 24 步全绿 | PASS（23 cases；整合树上 record 面独立验证） | record gate 内嵌收据 |
| `powershell … scripts/verify-ac-741.ps1 -SkipMainGates` | 全步 PASS | PASS（8.2s） | 同上内嵌 |
| `powershell … scripts/verify-ac-source-745.ps1` | 全绿 | PASS（37.2s） | 同上内嵌 |
| `powershell … scripts/verify-ac-scalar-767.ps1` | 全绿 | PASS（21.8s；覆盖 772 修复面） | 同上内嵌 |

所有 PE/子进程复用 ac-core `link::run_deadlined_split` 受控执行器
（60s/子进程硬截止）；exit code 全 32 位读取（≥256 的值区分真实结果与
任何 8 位截断——V-09 用 exit 4096/16384 直接证明）。

## 3. 用例索引（plan §6 V-01..V-10 → 物化）

| Case | 物化位置 | 结果 |
|---|---|---|
| V-01 身份 | text_hir（域门/换壳拒）+ text_binding + v10_scalar_surface_needs_no_runtime（无文本 ops→零能力需求） | 全绿 |
| V-02 Unicode | text_native::v02_unicode_len_slice_eq（len=3/slice(1,3)=中😀/精确 stderr）、v02_combining（e+U+0301 len=2）、v02_nul（NUL 不截断）、v02_empty、v02_direct_hir_leg（n01-native-ops.atom 直 HIR PE + 收据 runtime-lib/contract-sha）；text_source source 腿 | 全绿 |
| V-03 char | v03_char_roundtrip_and_eq（20013 往返）、v03_invalid_scalar_traps_70（D800→RT-FAIL code=-8+exit70）、v03_codepoint_upper_bound（U+10FFFF 过/超拒） | 全绿 |
| V-04 正常寿命 | v04_lifecycle_traces_and_move_transfer（LIT/CONCAT/CLONE/FREE 逐 id 配对、LIFO、.move 转移给 callee 清理）、v04_cross_frame_owned_return（ABI-C 跨帧转移）、v04_borrow_never_frees（视图零释放） | 全绿 |
| V-05 失败寿命 | text_resources：slice 越界(-10)/反向(-9)/负界(-10)、char_at 越界(-10)/负索引(-10)、invalid scalar(-8) 全部 trap70+RT-FAIL 精确行；trap 后语句不执行；预算/注入边界归 runtime 测试（记录） | 全绿 |
| V-06 可信变异 | text_hir/text_binding 程序化 Bundle 变异门（T-02 面：类型/资源/身份变异拒收）+ v10 合同变异门（hash/api/revision/target/缺失/多载体） | 全绿 |
| V-07 顺序 | text_native::v07_mark_wrapped_concat_args_comparison_and_short_circuit（冻结 oracle：mark_a/b 包装 concat/位置实参/命名映射/比较/短路+被跳过侧零 trace 零分配，source 腿）+ v07_mark_wrapped_order_direct_hir_leg（直 HIR 腿 n02-order-marks.atom：绑定映射换序证明）；trace marks 以纯 COFF 对象与文本运行时共链（F-01 路径 A，见 §5.1） | 全绿 |
| V-08 静态/来源 | text_source：非法转义/标量/多字符 char 词法拒、CRLF 跨行、遮蔽（len 同名用户函数）；未执行分支静态合法性 | 全绿 |
| V-09 规模 | v09_concat_chain_4096（exit 4096；4095 CONCAT/FREE 逐对）、v09_resource_loop_4096（exit 16384；每轮 1 分配 1 释放）、v09_call_chain_4096_with_text_args（借用视图跨 4096 帧） | 全绿 |
| r3 重放集 | text_review_r3 9 测试：q01（嵌套不可达 move 不伪移交）/q04（可达臂 Drop 唯一释放）/c01（不可达 move 后合法读取）/q02+c02（同源 view 参数 mut 排他/异源合法）/q03（语句尾寿命：mark 先于 FREE）+4096 链探测帧/q05（纯清理函数必须声明 runtime）/q06（坏引用受控拒绝不 panic）；R3-01..05 全覆盖 | 全绿 |
| r7 重放集 | text_review_r7 10 测试：m01/m02（直接 HIR break-only 循环不再伪造重入，冻结 atom 双例 run0 精确 trace）/c05（源码 break-only 双腿）/return-only（终止矩阵：return-only 体不重走，exit3）；p01/p02（陈旧重入经调用/嵌套循环读取，源+冻结 atom 双腿拒）；c01/c02（自然/continue 重入刷新视图合法 exit6 逐轮配对）/c03（true-else 不可达 continue exit0）/c04（嵌套 fresh owner exit12） | 全绿 |
| r6 重放集 | text_review_r6 9 测试：p01/p02（`if false` 臂内 break/continue 不再污染出口，exit0 零分配零 FREE，双腿）/p03/p04（continue 与自然 back-edge 重入后读取上一轮已释放借用，源+冻结 atom 双腿拒 owned-scope-escape）/c01（外层存活 owner 重入合法 exit3）/c02（静态 view 重入 exit3 无 FREE）/c03/c04（R5-01 嵌套短路兄弟控制，精确每 id 一次 FREE/LIFO）；组合 oracle：重入后 fresh `let` 前的陈旧代际借用拒读 | 全绿 |
| r5 重放集 | text_review_r5 11 测试：p01/p02（短路跳过 RHS 时 LHS 临时仍语句尾释放，exit0/1 唯一 FREE，双腿）/p03（嵌套 Logical 外层跳过，LHS 两枚 id1/id2 全释放 LIFO，双腿）/p04/p05（if 臂内 break/continue 后借用已释放 loop 作用域 owner 拒 owned-scope-escape，源+冻结 atom 双腿）/p06（命名实参换序绑定同源 wrapper 拒 mut-conflict，源+冻结 atom 双腿）；c01（双侧执行 LIFO）/c02（存活外层 owner 经 break 合法 exit3）/c03（命名异源 exit4）正控；组合 oracle：mark 后于整个逻辑仍先于 LHS 临时 FREE、双臂 break/continue 各自 arm-local owner 后出循环拒读 | 全绿 |
| r4 重放集 | text_review_r4 17 测试：p01..p04/p11（短路 RHS 拥有临时：跳过侧零分配零 FREE、执行侧一次分配语句尾一次释放，双腿）/p05/p06（终止 return 臂不再污染后续读取，exit3 唯一 FREE）/p08/p09（分支自有 owned 局部合法，exit0 唯一 FREE）/p10（owner 出口后经 backing 读取拒 owned-scope-escape，源+冻结 atom 双腿）/p07（包装两 view 形参同源 mut/view 拒 mut-conflict，源+冻结 atom 双腿）+两层传播；c01..c04 正控保持；组合 oracle：mark 后于短路仍先于 FREE、循环体 owner 的 backing 出循环拒读、两层 wrapper 传播 | 全绿 |
| r2 重放集 | text_review_r2 18 测试：p01..p09/c02..c08/mut01/mut02（QA-01..07 全覆盖；source PE 精确 trace 唯一 FREE 无 FREE-INVALID + HIR 变异腿证通道共享 verify 门）；QA-08 锁文件入库 | 全绿 |
| V-10 集成 | verify-ac-text-770.ps1 13 步（三旧 gates 实跑 PASS=旧制品面保持；scope 隔离=无 crates/test 越界=strict acc_inventory 前提）+ v10_contract_gate_rejections + v10_rejecting_build_preserves_sentinels（SENTINEL 三件保持） | 全绿 |

## 4. 发现并修复的实现问题

| 问题 | 定位 | 修复 |
|---|---|---|
| **F-01（r1 复审 blocking）：V-07 未按冻结 oracle 交付** | trace 支持库与文本运行时两 no_std staticlib 各带 `__rustc::rust_begin_unwind`，共链 duplicate symbol；首批以文本运行时自身 trace 替代 mark oracle（未批准缩围） | **路径 A**：`test-support/marks.rs` 以 `rustc --crate-type=lib --emit=obj -C panic=abort` 产纯 COFF 对象（无 panic 入口，测试内自断言该属性）与运行时共链；v07 恢复 mark_a/b 包装 concat/实参/比较/短路+被跳过分配 oracle，source 腿 + 直 HIR 腿（n02-order-marks.atom，绑定映射换序证明）双覆盖 |
| 4096 链 segfault（零 trace） | 每个 ABI-C 调用独立 4B 栈槽 → 16KB 帧跨 stack guard page 未探测 | 每函数共享一个出参槽（写后立即读，无生命周期重叠；帧恒定）@53e8ac7c4 |
| 合同门 api_revision 解析 None | 数字字段解析未跳过冒号 | 解析先定位 `:` |
| sha256 K[59] 笔误（0x8cc70808） | 复制错误，与 runtime 侧实现对拍发现 | 0x8cc70208；三处实现（CLI/tests/common/runtime contract.rs）已机械比对一致 |
| marks 对象符号自检窗口长度错 | `windows(16)` 对 14/17 字节 needle 恒 false（自检空转） | 按 needle 长度开窗（`windows(n).any(w == needle)`） |
| **r3 复审 R3-01..05（5 项阻塞）** | join 用语法预扫描而非实际臂状态（不可达嵌套 move 伪移交→漏释放；可达臂 Drop 后父级仍排释放→FREE-INVALID；不可达臂 dead 污染→合法读取误拒）；view 借入参数无源身份（同源 mut/view 放行）；临时在消费者处即释放（违背完整语句尾合同）；派生需求漏清理义务（纯 move 参数函数绕过合同门）；需求扫描先于结构校验且直接索引（坏引用 panic） | join 改实际 WalkState（可达性剪枝/终止臂不入/隐式臂快照/不可达臂状态丢弃）；alias_reach/escape_reach 分离（借入参数进别名集）；语句尾 pending_temps + enable_probestack/inline（cranelift 0.126 默认无探测——r2 的 4096 帧崩根因）；plan owned/exit_frees 入派生需求+adapter 为 move-Str 参数声明；扫描后置于结构校验且 total 化 |
| **r7 复审 P770-R7-01（1 项 P2 回归）** | r6 fixpoint 无条件把 body 存态当回边——results 只存 WalkState 不区分终止方式，break-only/return-only 体把终止态送回 header 伪造第二次迭代，首读被误判陈旧 | 回边分类显式化：continue 出口恒入 fixpoint；body 态仅当 `block_terminates=false`（存在自然落空路径）才入；break/return 终止体零重入贡献，真实 break 态仍入 after-loop 合流（r5 语义）、return 态仍不合流 |
| **r6 复审 P770-R6-01/02/03（2 项 P1 + 1 项 P2）** | 循环体只从入口快照走查一次——continue/自然 back-edge 状态只并入 after-loop，未反馈 header 再验证（重入读取上一轮已释放借用 run70/-7）；r5 出口收集器无可达性——不可达臂的 break/continue 污染 loop_exits 误拒合法代码；771 先合入后未整合（受审树不含 record 实现、未跑 record gate、SD before 过期） | 循环体按 RE-ENTRY 状态 fixpoint 重走（base+back-edge dead/backing 合流；收敛键=dead+id投影 backing；诊断截断至最终 pass）；fresh owned `let` 清死记+构造代际递增，backing 携 (owner,gen)——陈旧代际读取拒 owned-scope-escape；帧级静态可达性过滤出口记录（静态合法性检查不变）；同一实施树 merge master 调和 ParamMode/MethodOut/CallFrame/词法 mut/算子面谓词/check_profile_ops，text gate 接入 record gate 成 14 步联合门禁，SD02/04 before/after 按合 771 后 canonical 刷新 | 
| **r5 复审 P770-R5-01/02（2 项 P1）** | 短路 RHS 临时边界在进入整个 Logical 时冻结——LHS 已分配的拥有缓冲被错归 RHS，skip 边 null 守卫吞掉其 FREE（p01/p02 漏 FREE、p03 两枚全漏）；break/continue 的目标出口状态被 if join 丢弃（r4 R4-03 只保留 incoming），loop 只见 body 自然完出口——臂内 break 后借用已释放 loop 作用域 owner 的读取放行（p04/p05 两腿 UAF） | LogicalLhs 步骤在 LHS 完成后冻结 temps_mark（LHS 区临时保留真实指针、语句尾无条件释放，仅 RHS 区经 join 传 null）；break/continue 状态按目标 LoopId 单独保存（loop_exits），由目标 AwaitLoop resume 与 body 自然完出口一并 dead/backing 并集消费（return 仍只指向函数、不合流；if join 不回退 R4-03） |
| **r4 复审 P770-R4-01..05（5 项阻塞）** | 块自然出口只排 FREE 不进走查状态（owner 在后续状态"非 live 非 dead"被当合法 backing→UAF 放行）；join 的 dead/backing 合并按 reachable 而非真正进入 join 的出口（终止 return 臂污染 continuation→合法读取误拒）；join 遍历全部函数 owned（臂内局部被误算跨臂责任→ownership-required 误拒）；调用点排他只看直接 callee 的 mut 模式（包装一层即绕过同源 mut/view）；短路 RHS 拥有临时指针入全局语句尾清理列表（skip 前驱无该 SSA 定义→backend.lowering） | 块自然出口/break/continue 将自身 frees 标记 `DeadCause::ScopeExit` 入状态（backing 校验闭环，p10/循环出口拒 owned-scope-escape）；join 只合并真正进入的出口（incoming=reachable&&fall-through+implicit，终止臂保留其终出口释放但不贡献 continuation，嵌套组合正确）；join 一致性问题只覆盖本帧入口 owner（entry_live 快照，臂内局部在自身作用域出口收口）；参数对排他约束 fixpoint 传播（种子=mut×其他参数对，包装体把约束再出口到自己的签名，verify_text 驱动迭代至不动点）；短路 RHS 临时经 join 块参数携带（skip 边传 null）+ 语句尾逐临时 null 守卫释放（跳过零分配零 FREE、执行一次配对、mark 仍先于 FREE、4096 探测帧不变） |
| **r2 复审 QA-01..08（8 项阻塞）** | 伪造域进 scalar lowering panic；requires 删除绕过能力/合同门；move 在 return/分支/短路路径双释放或漏释放；view Str 参数被当 owned 返回；owned Call/字面量视图与借用返回的 provenance 断开；mut 无排他/外逃检查；循环内 fresh move 误拒；runtime 锁文件被根 .gitignore 忽略 | verify_text 域前置拒+派生 runtime 需求（声明强制+build 门双触发+adapter eq/ne 补声明）；常量条件折叠+连接一致清理责任+短路条件 move 拒+return 先处理 move+循环只杀入口存活者；owned let/return 的 ReadLocal 禁令；scanner 补 owned Call/StrLit+backing 向量传播至末消费者（帧恒小）；mut 进 provenance（调用点别名/返回外逃/派生 view）；`git add -f` runtime/text/Cargo.lock；重放集 18 测试永久化（tests/text_review_r2.rs） |

## 5. 记录的边界（不宣称的面）

1. **mark_a/b × 文本的共链载体（F-01 裁定路径 A，已解决）**：trace 支持库
   与文本运行时都是 Rust no_std staticlib，各自携带
   `__rustc::rust_begin_unwind`，两个 staticlib 共链一个 PE 是 duplicate
   symbol（rust-lld 硬错）。首批交付曾以文本运行时自身 trace 替代
   mark oracle（r1 复审 F-01 判定未批准缩围）；返工后 trace marks 以
   **纯 COFF 对象**承载（`test-support/marks.rs`，`rustc --crate-type=lib
   --emit=obj -C panic=abort`——对象形态无 panic 入口符号，与运行时
   staticlib 共链无冲突），V-07 冻结 oracle（mark 包装 + source/HIR
   双腿）恢复。两个 staticlib 之间的共链限制本身仍在（对后续组合
   测试面适用）。
2. **预算/故障注入无语言入口**：`ac_rt_set_budget`/`ac_rt_fail_nth_alloc`
   是 runtime 控制 API，首批语言面无调用点——V-05 的注入/预算面由
   runtime/text 测试族拥有（20 测试），语言面以"故障=RT-FAIL+70 可观察"
   为 oracle（plan §9.2 预先裁定）。
3. **char_at 越界码为 -10**：冻结 runtime 将标量索引越界映射到 range 类
   （ERR_INDEX_OOB=-11 声明未用）；语言测试按实际 ABI 断言（-10），与
   design.md §3.4 的 -1..-11 注册表一致。
4. **bash `$?` 8 位截断**：真实 PE exit ≥256 时 shell 观察值 mod 256——
   所有断言走受控执行器的全 32 位码。

## 6. 输入/制品指纹

r7 联合门禁收据 `gate-20261014-202410.txt`（14/14 PASS，绑代码提交
314cbbeb6；record gate 实跑 ~201s）。r6 联合门禁收据
`gate-20261014-193430.txt`（代码树）与 `gate-20261014-194300.txt`
（最终 HEAD e01997dc5）。
r5 返工收据 `gate-20261014-175221.txt`（HEAD ebd0d05ee，13/13 PASS）。r4
返工收据 `gate-20261014-171125.txt`（HEAD 42b0ba7da）与
`gate-20261014-171634.txt`（最终 HEAD 1f8027500）；其间一次 gate 失败
gate-20261014-170638 为 `source_native::positive_corpus` 的 `receipt
missing` 瞬态窗口——与 r2/r3/r4 复审记录同族，零代码变化串行复跑通过，
原文保留不抹去。历史收据 `gate-20261014-155918.txt`（HEAD 92c14fe6c，
r3 终态）。收据含 rustc 1.98.0、rust-lld sha256、runtime-lib/contract
sha256、12 个源文件 sha256、gate 脚本自指纹、raw 输出目录。CI/复审可
直接复跑 `scripts/verify-ac-text-770.ps1` 对拍。
