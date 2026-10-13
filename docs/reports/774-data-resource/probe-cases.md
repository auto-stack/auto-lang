# PLAN-774 探针用例冻结表（probe-cases）

- 计划：PLAN-774 r3 · T-02（§5.3 冻结关口：T-03/T-04 只实现本表已明确预期的候选，
  不得用探针自己的输出反写 expected）
- 本文所有数值（哨兵、大小/对齐/offset、trace 行、状态码、计数、预算）**从
  decisions.md §3 冻结合同独立推导**（手算），不来自任何生成产物。
- 入口：探针 crate `experimental/ac-core/probes/data-resources/`（独立 workspace，
  含 no_std 静态库 shim）；调用者由 Cranelift 0.126 WindowsFastcall 生成，被调用者
  = shim 显式 ABI 导出（`p774_*`），链接真实 PE（rust-lld + kernel32.lib +
  shim 静态库），经 ac-core `link::run_deadlined_split` 受控执行器运行（60s 截止；
  V-06 超时控制用 5s 显式截止）。宿主测试不得直接调用 shim 冒充机器调用通过。

## 0. 冻结常量（全局）

**状态码（i32 通道负值；0 = 成功）**

| 码 | 值 | 类别 |
|---|---|---|
| res.len-overflow | -1 | 长度算术越界（先于分配与缓冲读取） |
| res.len-negative | -2 | i32→长度转换负值 |
| res.len-too-large | -3 | 长度(u32)→i32 超界 |
| res.budget | -4 | 预算拒绝（不触分配器） |
| res.env-oom | -5 | 分配器返回 null（真实或注入） |
| res.shim-error | -6 | 显式 shim 错误 |
| text.invalid-utf8 | -7 | 运行期解码失败 |
| text.invalid-scalar | -8 | 非法 Unicode 标量构造 |
| text.range-reversed | -9 | start > end |
| text.range-oob | -10 | 标量下标越界/负值 |
| res.index-oob | -11 | borrow_read 字节下标越界/负值 |

**模型拒绝码（前门，native 调用计数 0）**：verify.use-after-free /
verify.double-move / verify.use-after-move / verify.mut-conflict /
verify.borrow-escape / verify.field-borrow / verify.container-borrow /
verify.ownership-required。

**计数口径**：`attempts` = 分配器调用次数（长度/预算检查之后）；`allocs` = 成功
分配数；`frees` = 成功释放数；`live` = allocs−frees；`live_bytes` = 存活分配字节和；
`peak` = 分配/释放边界处 live_bytes 最大值。零长度请求不产生 attempt。

**trace 行（ASCII，stderr，逐行断言；不含地址值）**

```
MIX a=%d b=%d c=%u d=%d
MANY a=%d b=%d c=%d d=%d e=%d f=%d g=%d
BSUM len=%d
OWN-MAKE seed=%d len=%d
CLONE len=%d
CONCAT a=%d b=%d
LIT len=%d
REC-VIEW flag=%d count=%d tag=%u
REC-MUT addend=%d
REC-OUT seed=%d
TRAP-PRE live=%d
HANG-PRE live=%d
SUMMARY attempts=%u allocs=%u frees=%u live=%u live_bytes=%u peak=%u
H-ALLOC h=%u
H-CLONE h=%u rc=%u
H-DROP h=%u rc=%u
H-GET h=%u v=%u
TEXT-SCALARS n=%d
TEXT-BYTES n=%d
CHAR-AT i=%d v=%u
SLICE s=%d e=%d len=%d
FROM-CP cp=%d
```

**shim 导出面（冻结符号）**：ABI 族 `p774_mix/p774_many/p774_borrow_sum/
p774_borrow_read/p774_own_make/p774_own_free/p774_own_clone/p774_own_concat/
p774_own_from_lit/p774_rec_view/p774_rec_mut/p774_rec_out`；文本族
`p774_text_scalars/p774_text_bytes/p774_char_at/p774_text_slice/
p774_char_from_codepoint/p774_char_to_codepoint`；长度边界
`p774_len_from_i32/p774_len_to_i32`；控制/故障 `p774_set_budget/
p774_fail_nth_alloc/p774_shim_error/p774_summary/p774_check_invariant/
p774_trap_after_alloc/p774_hang_after_alloc`；R2 对照族 `p774_h_alloc/
p774_h_clone/p774_h_drop/p774_h_get`。

**PE 退出码约定**：生成入口内做机器侧断言后 `ExitProcess`——0 = 全部断言通过；
1 = 断言失败（trace 仍在）；V-06 trap 用例退出 0xC0000005（=3221225477）。
每个正常 PE 末尾调用 `p774_check_invariant` 并以其返回值参与退出码
（allocs==frees 且 live==0 → 0，否则 1）。

## V-01 决策溯源审计

D-01..07 每项 → 现行 Spec/源码/探针证据三元组（owner=work；实收 =
decisions.md §2 事实表 + §3 裁定 + 本表用例）：

| 决定 | Spec/设计依据 | 源码/消费者证据 | 探针用例 |
|---|---|---|---|
| D-01 | source-scalar-i32.md:15-31；numeric-boundary §2-4 | ac-core Constant{i32}/trap70；lexer.at:44-52 | V-05 L 边界、V-07 计数 |
| D-02 | Design 36 §3 候选；MD-408 | lexer.at cur_char；string.rs str_char_at；ValueData::Char | V-07 全表 |
| D-03 | Design 02 §String | a2r 映射表；token.at/parser.at 消费者 | V-03、V-04 M4 |
| D-04 | Design 04 §Trinity；PLAN-667 修正 | ownership/ 未接线 | V-04 M1-M5+R2 对照 |
| D-05 | project.md:16-18；Design 04 §Parameter Passing | native.rs 现行 i32 面 | V-02 全表 |
| D-06 | （无既成承诺） | test-support no_std 模式 | V-03/05/06/08 |
| D-07 | Design 36 §5/§7 | — | interface-contract.md（T-05） |

覆盖边界：无无证据默认；D01..07 到 T-05 无必需 open 项。

## V-02 五类 ABI 真实 native 调用（决定 D-05）

布局独立推导（Win x64：整型/指针参数 RCX/RDX/R8/R9，其后栈传；≤8 字节返回 RAX；
不使用聚合返回）：Rec1 = { flag: bool @0，count: i32 @4，tag: char(u32) @8 }，
size 12、对齐 4、字段 offset = 4·i。

| Case | 输入（调用者→shim） | 期望（机器断言 + trace） |
|---|---|---|
| V02-1 ABI-A 混合标量 | `p774_mix(a=17, b=1, c=0x4E2D, d=96)` | 返回 17+1+20013+96 = **20127**；trace `MIX a=17 b=1 c=20013 d=96` |
| V02-1s 异质顺序控制 | 同上但 a↔d 互换（a=96, d=17） | trace `MIX a=96 b=1 c=20013 d=17`——与 V02-1 oracle 在 a/d 位**必须可区分**（通道识别参数交换） |
| V02-2 寄存器/栈边界 | `p774_many(100,101,102,103,104,105,106)`（第 5-7 参走栈） | 返回 Σ=**721**；trace `MANY a=100 b=101 c=102 d=103 e=104 f=105 g=106` |
| V02-2s 边界互换控制 | d↔e 互换（d=104, e=103；一寄存器一栈位） | trace 该两位互换可区分 |
| V02-3 ABI-B 借用读 | data 段 `"Auto"`（4 字节 41 75 74 6F）→ `p774_borrow_sum(ptr,4)` | 返回 65+117+116+111 = **409**；trace `BSUM len=4`；之后 SUMMARY live=0（callee 未释放） |
| V02-4 ABI-C 拥有返回 | `p774_own_make(seed=7, len=16)` | ptr≠null、out_len=16；内容 byte[i]=(7+i) mod 256 → `p774_borrow_sum` = (7+22)·16/2 = **232**；trace `OWN-MAKE seed=7 len=16`；caller `p774_own_free` 后 live=0 |
| V02-5 ABI-D record view | 写 Rec1 {flag=1@0, count=5@4, tag=0xE9@8}（233）→ `p774_rec_view` | 返回 flag+2·count+3·tag = 1+10+699 = **710**；trace `REC-VIEW flag=1 count=5 tag=233` |
| V02-5s 字段交换控制 | 值互换存：offset4 存 233、offset8 存 5 | trace `REC-VIEW flag=1 count=233 tag=5`（校验和 1+466+15=482 ≠ 710）——通道识别字段交换 |
| V02-6 ABI-E record mut | 由 {1,5,233} 起 `p774_rec_mut(p, addend=10)` | count+=10、flag 翻转 → {0,15,233}；后续 rec_view = 0+30+699 = **729**；trace `REC-MUT addend=10` |
| V02-7 ABI-E out-return | 栈上预置 12 字节存储 → `p774_rec_out(seed=5, out)` | 写 flag=5&1=1, count=5, tag=5|0x40=0x45(69)；rec_view = 1+10+207 = **218**；trace `REC-OUT seed=5` |
| V02-8 嵌套调用 | 生成 f_inner（rec_mut）+ f_outer（先 rec_view、调 f_inner、再 rec_view） | 首末校验和 710→729（同 V02-5/6 值）；两级调用均为真实机器调用 |

边界：Windows x64 单向；不含公开 C 双向互调承诺；不含 sret 聚合返回（拒绝候选①）。

## V-03 寿命与计数（决定 D-03/D-06）

| Case | 场景 | 期望 SUMMARY（attempts/allocs/frees/live/live_bytes/peak） |
|---|---|---|
| V03-1 借用不释放 owner | make(7,16)→borrow_sum→borrow_read(ptr,16,15)→free | `16 1 1 0 0 16`；borrow_read 返回 (7+15)mod256=**22** |
| V03-2 move 无重复清理 | make→（寄存器移动到第二槽位）→free(新槽位) | `1 1 1 0 0 16` |
| V03-2n 双清理探测控制 | 同上但新旧槽位各 free 一次 | frees=2≠allocs=1 → check_invariant=1 → **exit 1**（探测能力证据） |
| V03-3 clone 独立 | make(7,16)→clone→两者均 free | `2 2 2 0 0 32`；机器断言 ptr1≠ptr2（相等则 exit 1） |
| V03-4 分支早退 | f(seed)：make→若 seed：{free;return 1}；否则 clone+双 free+return 0；seed=1 与 seed=0 两 PE | 两 PE 均 live=0、exit 0；`1 1 1 0 0 16` / `2 2 2 0 0 32` |
| V03-5 循环出口 | loop 3 轮 {make; free} | `3 3 3 0 0 16` |

## V-04 非法状态前门拒绝（决定 D-04；规则模型，非生产 verifier）

模型算子：make/view/end-view/mut/move/free/return-view-of(local|param)。
每例断言三件：拒绝入口（op 序号+阶段 verify+码）、**被拒操作 native 调用计数=0**
（dispatch 台账）、合法兄弟例真实 PE 通过（exit 0、live=0）。

| Case | 序列 | 拒绝码@op | 合法兄弟 |
|---|---|---|---|
| V04-M1 | make p → free p → view p | use-after-free@3 | make→free（不使用） |
| V04-M2 | make p → move p → move p | double-move@3 | make→move |
| V04-M5 | make p → move p → view p | use-after-move@3 | make→view→end-view→move |
| V04-M3 | make p → view v → mut p | mut-conflict@3 | make→view→end-view→mut |
| V04-M4 | 局部 owned → return view | borrow-escape@return | return view-of-param（owner 在外） |
| V04-R2 | R2 族：h_alloc(0x11)=h1 → h_drop(h1) → h_alloc(0x22)=h2（复用槽）→ **h_get(h1)** | **无拒绝**——v=0x22(34) 静默错读、无错误码（trace `H-GET h=1 v=34`） | R1 对照 M1 前门拒绝+计数 0 |

V04-R2 即候选 R2 淘汰的实证：句柄层无前门性质，违例静默通过；记录于 decisions §D-06。

## V-05 故障注入与失败类别（决定 D-01/D-06）

每例断言：返回状态值、out_len 负码、先前已分配资源按合同回收（live=0）、
exit 0（失败是值不是崩溃）、attempts 计数、与 malformed/budget/trap 可区分。

| Case | 注入 | 期望 |
|---|---|---|
| V05-1 AP-1 构造失败 | fail_nth=1 → make(7,16) | ptr=null、out_len=**-5**；`1 0 0 0 0 0`；exit 0 |
| V05-2 AP-2 clone 失败 | fail_nth=2 → make ok→clone | clone ptr=null/-5；free 原 buffer；`2 1 1 0 0 16`；exit 0 |
| V05-3 AP-3 concat 失败 | fail_nth=1 → concat("Ab","cd") | null/-5；`1 0 0 0 0 0`；exit 0 |
| V05-4 AP-4 字面量提升失败 | fail_nth=1 → own_from_lit("Auto") | null/-5；`1 0 0 0 0 0`；exit 0 |
| V05-5 零长度 | make(7,0)→free(null) | ptr=null、out_len=**0**；attempts 增量 **0**；live=0；exit 0 |
| V05-6 concat 长度溢出 | concat(a_len=0x40000000, b_len=0x40000000)（检查先于读缓冲） | null/**-1**；attempts 增量 0；exit 0 |
| V05-7 转换负值 | len_from_i32(-5) | **-2**；无请求 |
| V05-8 转换超界 | len_to_i32(0x80000000) | **-3**；无请求 |
| V05-9 shim 错误 | p774_shim_error() | **-6** |
| V05-10 concat 成功反控 | concat("Ab","cd") | ptr≠null、len=4、内容 Σ=65+98+99+100=**362**；caller free；live=0 |
| V05-11 borrow_read 越界 | borrow_read(ptr,16,16) 与 (ptr,16,-1) | 均 **-11** |
| V05-12 类别区分矩阵 | — | malformed=前门码（V-04）；budget=-4（V-08）；env=-5（本表）；len=-1/-2/-3；trap=0xC0000005（V-06）——五类码值互异、机制互异 |

## V-06 fatal trap 与超时分类（决定 D-06）

| Case | 输入 | 期望 |
|---|---|---|
| V06-1 trap 后 OS 回收 | `p774_trap_after_alloc(16)` | trace `TRAP-PRE live=1` 后写 null → AV；exit = **0xC0000005(3221225477)**；stderr **无 SUMMARY 行**（未执行清理、不冒称析构；进程内存由 OS 回收，收据记录之） |
| V06-2 执行器超时分类 | `p774_hang_after_alloc(16)`，5s 显式截止 | trace `HANG-PRE live=1` 后死循环 → run_deadlined_split 返回 **Err(Deadline)**（受控超时分类，非成功非崩溃） |

## V-07 Unicode/索引边界（决定 D-01/D-02）

fixture（data 段字节，冻结）：T1=`41 E4 B8 AD F0 9F 98 80`（"A中😀"）；
T2=`65 CC 81`（e+U+0301）；T3=`00`（NUL）；T4=`FF FE`（非法 UTF-8）。

| Case | 输入 | 期望 |
|---|---|---|
| V07-1 | scalars/bytes(T1) | **3 / 8** |
| V07-2 | scalars/bytes(T2) | **2 / 3**（组合字符不聚类） |
| V07-3 | scalars/bytes(T3) | **1 / 1**（NUL 合法非终止） |
| V07-4 | scalars(T4) | **-7**（text.invalid-utf8，运行期解码） |
| V07-5 | char_at(T1, 0/1/2) | **65 / 20013(0x4E2D) / 128512(0x1F600)** |
| V07-6 | char_at(T1,3)、char_at(T1,-1) | 均 **-10** |
| V07-7 | char_at(T3,0) | **0**（NUL 是合法 char） |
| V07-8 | slice(T1,0,1) | len **1**、首字节 0x41 |
| V07-9 | slice(T1,1,3) | len **7**（跨标量切片按字节范围） |
| V07-10 | slice(T2,0,2)、slice(T1,1,1) | len **3**；len **0**（空范围合法） |
| V07-11 | slice(T1,2,1) | **-9**（reversed） |
| V07-12 | slice(T1,0,4)、slice(T1,-1,2) | 均 **-10**（oob/负下标） |
| V07-13 | from_codepoint(0x1F600 / 0xD800 / -1 / 0x110000) | ok=128512；**-8；-8；-8** |
| V07-14 | to_codepoint(20013) | **20013** |

byte 与 scalar 计数不混：V07-1 与 V07-2 的双值断言同时锁定两套单位。

## V-08 规模与预算（决定 D-06）

| Case | 输入 | 期望 |
|---|---|---|
| V08-1 | 生命周期循环 N=0 | SUMMARY `0 0 0 0 0 0` |
| V08-2 | N=1（make(7,16)+free） | `1 1 1 0 0 16` |
| V08-3 | N=4096 | `4096 4096 4096 0 0 16`；无 live 增长（验证规模，非语言上限） |
| V08-4 | set_budget(16)→make(7,16) | 成功（0+16 ≤ 16，边界内側） |
| V08-5 | set_budget(16)→make(7,16)→make(1,1) | 第二次 **-4**（16+1 > 16，边界外側）；free 后再 make(7,16) **成功**（释放回收额度） |
| V08-6 | 深度上限不存在 | 模型拒绝码集合（§0）枚举无深度类码；shim 无深度计数器——审计断言，非运行例 |

预算单位=存活字节；入口=每次分配请求（长度校验后）；两侧临界=恰等/超一。

## V-09 隔离与旧门禁（AC-05）

- scope diff：本计划改动仅限 `experimental/ac-core/probes/data-resources/**`、
  `scripts/verify-ac-data-resources-774.ps1`、`docs/reports/774-data-resource/**`、
  `docs/plans/774*`（簿记）；`crates/**`、根 Cargo、registry/schema/source、
  auto-lib **零 diff**（git diff 审计断言）。
- 既有 gates 重跑：741 `verify-ac-741.ps1 -SkipMainGates`、745
  `verify-ac-source-745.ps1`、772 `verify-ac-scalar-767.ps1` 全绿；
  既有支持/拒绝面与身份不变。
- 命令（唯一工作树）：`cargo check --manifest-path
  experimental/ac-core/probes/data-resources/Cargo.toml --locked --all-targets`、
  `cargo test --manifest-path … --locked`、`cargo fmt --manifest-path … --check`；
  不跑根 cargo t/tv/docs_gen（无 crates/root 定义改动）。

## r1 复审修订（oracle 增补；与 decisions.md §3.8 对应）

**计数口径**：`frees`=成功释放；新增 FREESTAT 行
`FREESTAT free_attempts=%u frees_ok=%u invalid=%u`；分配成功 trace 附 ` id=N`；
释放 trace `FREE id=%u` / `FREE-INVALID`；双释放负控 = invalid=1 + exit 1
（不读已释放内存）。check_invariant 收紧含 live_bytes==0 ∧ invalid==0。

**V-02 增补**：

| Case | 输入 | 期望 |
|---|---|---|
| V02-9 | rec2_out(seed=5)+rec2_view+rec1 view 混合 | x=5,y=6,flag=1；校验和 5+12+3=**20**；REC2-VIEW x=5 y=6 flag=1；REC-VIEW flag=1 count=5 tag=233 同 PE |

**V-03 增补**：

| Case | 期望 |
|---|---|
| V03-6 LIFO | A(id1),B(id2) 创建，B→A 释放：trace `FREE id=2`→`FREE id=1`；FIFO 控制产生可区分的另一顺序 |
| V03-7 嵌套 break/continue | 4 轮循环（i==1 continue 清理、i==2 break 清理、自然路径清理）：SUMMARY `3 3 3 0 0 16` |
| V03-8 临时语句尾 | make→BSUM→BSUM→free：trace 顺序 BSUM/BSUM/FREE id=1（使用先于清理） |
| V03-9 callee 清理+return 转移 | move-in 由 consume 释放；produce 经 i64 单元把指针移回调用方：SUMMARY `2 2 2 0 0 16` |
| V03-10 clone 内容独立 | from_lit("Auto")→clone→free 原→BSUM(clone)=**409**→free：LIT id=1/CLONE id=2/FREE id=1 |

**V-04 增补**：M4b 来源验证（free 后 ReturnViewParam=use-after-free；ParamOwned
ReturnViewParam=borrow-escape；ParamBorrowed 合法）；M4c 真借用返回 PE（(ptr,len)
跨帧经 i64/i32 单元，调用方 borrow_read=65）；M5 合法兄弟真实 PE；field/container
入口（StoreFieldView/StoreContainerView 拒绝 + StoreOwnedClone 合法 + PE）；
CallMoveParamImplicit/Explicit（缺 .move 拒绝；显式转移 callee 清理 PE）；
R2 checked 对照（h_get_checked 对陈旧句柄=-11）。

**V-05 增补**：V05-13 带 owner 注入（AP-3/AP-4 失败时 16 字节 owner 存活并按合同
回收）：SUMMARY `2 1 1 0 0 16`。

**V-08 增补**：

| Case | 期望 |
|---|---|
| V08-4a 64/65 并发 | 直线 64/65 个 1 字节 buffer 逆序全释放：SUMMARY `(64,64,64,0,0,64)`/`(65,65,65,0,0,65)`，exit 0 |
| V08-5a 额度归还 | budget=65；65×(1B)存活→逆序释放→单次 65B 请求成功：SUMMARY `66 66 66 0 0 65` |
| V08-5b 零预算 | set_budget(0)：1B 请求→null+**-4**，attempts=0；set_budget(-1) 清除后同请求成功 |

**V-06 修订**：V06-2 前驱 trace 经文件重定向观察（`HANG-PRE live=1` 在 deadline
后可读）；分类仍为 Err(Deadline)。

## r2 复审修订增补（P774-R2-01..04）

| Case | 内容 | 期望 |
|---|---|---|
| V04-r2a | ParamBorrowed→View→EndView→ReturnViewParam | 合法（无 panic）；ledger borrow_read=1、borrow_return=1 |
| V04-r2b 来源×模式矩阵 | 8 合法脚本 + 15 非法脚本（冻结码逐格） | 全部按矩阵判定 |
| V04-r2c | 静态字面量返回 PE（rodata 字面量跨帧 view+调用方读） | exit 0，首字节 65 |
| V04-r2d | 借入 mut 参数 PE（callee 独占经参改写 record） | 710→729，REC-VIEW/REC-MUT trace |
| V05-14 | 账本首表注入失败→回滚→清旋钮后成功 | SUMMARY `2 1 1 0 0 16`；FREESTAT `1 1 0` |
| V05-15 | 16 owner 存活时扩表注入失败→回滚→owner 完好→清旋钮扩表成功→全释放 | SUMMARY `18 17 17 0 0 16`；FREESTAT `17 17 0` |

跨行字面量/静态字面量返回/转义白名单的语义恢复与取材表：decisions §3.9。

## r3 复审修订增补（P774-R3-01/02）

| Case | 内容 | 期望 |
|---|---|---|
| V08-6 | set_budget(0xFFFF_FFFF / 0x1_0000_0000 / 0x1_0000_0001) 后各请求 2 字节 | 三例均成功；SUMMARY `1 1 1 0 0 2`（预算 u64 原值，无截断） |
| V05-15a | 16 owner 全存活：扩表注入失败→第 17 项回滚→清旋钮重试（真跨容量）→读回 17 buffer 内容→逆序全释放 | SUMMARY `18 17 17 0 0 17`；FREESTAT `17 17 0` |
| V05-15b | 空位复用控制：扩表失败→释放 1 owner→重试进现有表 | SUMMARY `18 17 17 0 0 16`；FREESTAT `17 17 0` |

矩阵勘误：来源×模式矩阵非法脚本 **16** 组（文档此前误写 15）。
