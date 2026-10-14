# PLAN-770 验证收据（verification）

- 计划：PLAN-770 r2 · T-01..T-07（work 阶段，AC-01..06 证据）
- 工作树：`D:/autostack/.wt/lang-770/auto-lang` @ `plan-770-dev`
- 基线：工作树基面 `e849e06f9`（= master）；本收据时 HEAD `c4e169cec`（F-01 返工提交；r1 复审受审基线为 a87de0bd7）
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

## 2. 命令与结果（唯一工作树）

| 命令 | 期望 | 实际 | 收据 |
|---|---|---|---|
| `cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets` | 零错误零警告 | PASS | gate 原始输出 |
| `cargo test --manifest-path … --locked`（ac-core 全套 18 二进制） | 全绿 | **200/200**（含 text_native 19、text_resources 10、text_hir 16、text_source 10、text_binding 11；旧面 scalar_native 8/source_native 10/native_execution 11 等逐位保持） | raw core-test.txt |
| `cargo fmt --manifest-path … --check`（ac-core 与 runtime/text 两处） | clean | PASS | gate |
| `cargo check/test --manifest-path experimental/ac-core/runtime/text/Cargo.toml --locked` | 全绿 | **20/20**（fault 6、lifecycle 5、smoke 2、textops 4、trap 2、contract 1） | raw rt-test.txt |
| `powershell … scripts/verify-ac-text-770.ps1` | 13 步全绿 | **PASS**（check/test/fmt ×2 + 旧 gates 741[-SkipMainGates]/745/767 + scope 隔离 + 库/合同指纹；**绑最终 HEAD c4e169cec**，F-02 处置） | `experimental/ac-core/target/ac-verify-receipts/770/gate-20261014-124436.txt` |
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
| V-10 集成 | verify-ac-text-770.ps1 13 步（三旧 gates 实跑 PASS=旧制品面保持；scope 隔离=无 crates/test 越界=strict acc_inventory 前提）+ v10_contract_gate_rejections + v10_rejecting_build_preserves_sentinels（SENTINEL 三件保持） | 全绿 |

## 4. 发现并修复的实现问题

| 问题 | 定位 | 修复 |
|---|---|---|
| **F-01（r1 复审 blocking）：V-07 未按冻结 oracle 交付** | trace 支持库与文本运行时两 no_std staticlib 各带 `__rustc::rust_begin_unwind`，共链 duplicate symbol；首批以文本运行时自身 trace 替代 mark oracle（未批准缩围） | **路径 A**：`test-support/marks.rs` 以 `rustc --crate-type=lib --emit=obj -C panic=abort` 产纯 COFF 对象（无 panic 入口，测试内自断言该属性）与运行时共链；v07 恢复 mark_a/b 包装 concat/实参/比较/短路+被跳过分配 oracle，source 腿 + 直 HIR 腿（n02-order-marks.atom，绑定映射换序证明）双覆盖 |
| 4096 链 segfault（零 trace） | 每个 ABI-C 调用独立 4B 栈槽 → 16KB 帧跨 stack guard page 未探测 | 每函数共享一个出参槽（写后立即读，无生命周期重叠；帧恒定）@53e8ac7c4 |
| 合同门 api_revision 解析 None | 数字字段解析未跳过冒号 | 解析先定位 `:` |
| sha256 K[59] 笔误（0x8cc70808） | 复制错误，与 runtime 侧实现对拍发现 | 0x8cc70208；三处实现（CLI/tests/common/runtime contract.rs）已机械比对一致 |
| marks 对象符号自检窗口长度错 | `windows(16)` 对 14/17 字节 needle 恒 false（自检空转） | 按 needle 长度开窗（`windows(n).any(w == needle)`） |

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

见 gate 收据 `gate-20261014-124436.txt`（HEAD c4e169cec、rustc 1.98.0、
rust-lld sha256、runtime-lib/contract sha256、12 个源文件 sha256、
gate 脚本自指纹、raw 输出目录）。CI/复审可直接复跑
`scripts/verify-ac-text-770.ps1` 对拍。
