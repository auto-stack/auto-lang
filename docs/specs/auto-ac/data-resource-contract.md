# auto-ac 数据/资源合同（data-resource-contract）

> **Status**: experimental（合同 + 探针实证；**实现仍仅探针**——语言 feature 不因本文件变为 implemented）
> 来源：PLAN-774（2026-10-14 合入；经六轮独立复审收敛）。探针/决策/用例/收据：
> [decisions.md](../../reports/774-data-resource/decisions.md)、
> [probe-cases.md](../../reports/774-data-resource/probe-cases.md)、
> [interface-contract.md](../../reports/774-data-resource/interface-contract.md)、
> [verification.md](../../reports/774-data-resource/verification.md)。
> 消费者：770（文本）、771（record）——两者以本文件与 interface-contract 为唯一接口来源。

本文件冻结 AC 数据扩面（文本/标量聚合首批）的**共同类型、资源与内部调用契约**。
**边界**：不是生产模块 ABI、不是 ACC Cranelift 桥、不冻结公开 C 互调；完整所有权/
GC/List/enum/use/生产 I/O 不在面内（后续 owner，见 D-07）。探针（Rust/Cranelift
+ no_std shim）是可行性证据，不是 Auto native 编译证据。

## D-01 类型与宽度

- 新 profile 的源 int/i32 = 精确有符号 32 位（沿 core/scalar）；Bool 与 I32 不隐式互转。
- char 名义类型，内部 **u32 Unicode 标量值**（宽 4、对齐 4，ABI 通道以 I32 位型承载）。
- **内部地址/字节长度/标量计数独立于源 i32**：字节长度/标量计数 u32、指针机器宽。
  转换边界：源 i32→内部长度须非负（`res.len-negative`）；内部长度→源 i32 须
  ≤0x7FFF_FFFF（`res.len-too-large`）；长度加法（concat）在 i64 中间量判定，
  结果超 i32 通道上界即拒（`res.len-overflow`，先于任何缓冲读取与分配）。
  不宣称 i64/usize 是源语言类型。

## D-02 文本规则

1. 文本 = Unicode 标量值序列，**UTF-8 字节存储**；语言级文本值恒合法 UTF-8
   （非法字节序列仅探针对抗输入；运行期解码失败 `text.invalid-utf8`，记录偏移）。
2. **索引单位 = Unicode 标量**（非字节/字素）。默认 `len()` = 标量计数；字节长度
   显式 `len_bytes()`。组合字符不聚类（`e`+U+0301 = 2 标量/3 字节；slice 可切
   组合序列中间，首批无字素支持，明示非遗漏）。
3. **char 独立名义类型**；`char_at(i)` 返回 char；无隐式 char/int 互转。显式：
   `c.to_codepoint() -> int`（恒成功）；`char.from_codepoint(n)` 可失败——
   代理区/负值/>0x10FFFF 拒 `text.invalid-scalar`。
4. 范围：`0 ≤ start ≤ end ≤ 标量数`；start>end → `text.range-reversed`；越界/
   负下标 → `text.range-oob`；空范围 [s,s) 合法；NUL 合法标量非终止符；空串合法。
5. **字面量拼写**（词法层由 770 实现，码与本表冻结）：
   - **跨行字符串字面量合法**（沿既有承诺；消费者 packages/aavm/lib/engine.at
     引号内换行实证）：引号内行界**原样入值**（LF 保留 LF、CRLF 保留 CRLF），
     不归一化、不剥离续行前导空白。
   - 转义白名单（对齐旧 lexer 取材 + 新增 `\u{}`）：`\n \t \r \0 \\ \"`（str）、
     `\'`（char）、`\u{1-6hex}`（显式标量，非法标量前门 `text.invalid-scalar`）；
     **未知转义 → `text.invalid-escape`**（与旧行为"原样两字符入值"的差异为
     适配项，不宣称旧行为继承）。raw `` ` `` 串与 `"""` 三引号归 770 词法裁量。
   - char 字面量恰一个标量：`'ab'` → `text.char-multi`；`''` → `text.char-empty`。
6. char_at 旧三宿主差异（单字符 str/int 码点/Rust char）不采纳任一；新 profile
   返回 char。

## D-03 str/Str 逐语境

| 语境 | 规则 | 前门拒绝码 |
|---|---|---|
| 字面量 | 借用视图（常驻数据）；提升为 Str = 分配+拷贝 | — |
| 局部 str | 借用视图（字面量/拥有者 slice）；不拥有不清理 | — |
| 局部 Str | 拥有：构造/转移；出口唯一清理（LIFO） | — |
| 参数（str 与 Str 同） | **默认 view**（只读借用；callee 不得保存/释放/逃逸）——沿 Design 04/ParamMode 既有承诺 | — |
| 显式 `mut str` 参数 | 借入排他可变借用，区域覆盖整个函数体（首批无中途关闭/降级；返回其视图拒 escape） | — |
| 显式 `mut Str` 参数/值 | 对拥有值开排他借用 | — |
| 显式 `move Str` 参数 | 所有权转移入函数（调用点须 `.move`），callee 负责清理，实参禁用 | 缺 `.move`：`verify.ownership-required` |
| 返回 | 拥有 Str = 转移给调用者；借用 str = **借自存活借入参数或常驻字面量**（静态字面量不朽，合法返回来源） | 借自局部/move-in 参数/活跃 mut 借入：`verify.borrow-escape` |
| record 字段 | 仅 Str（拥有）；str 字段首批禁止 | `verify.field-borrow` |
| 容器元素 | 仅 Str；`List<str>` 既有禁止保留 | `verify.container-borrow` |
| clone / 显式转换 / move | `s.clone()` 独立分配；`s.to(Str)` 分配拷贝、`s.to(str)` 借用化；`s.move` 源死后禁用、清理随所有权 | `verify.use-after-move` |

## D-04 受限借/拥有（前门）

verify 层执行；不支持的形态显式拒绝，不放行也不转运行期。规则：

- 来源五类：局部拥有 / 借入参数 / 借入 mut 参数 / move-in 拥有参数 / 静态字面量。
- view 区域对全部可读来源可追踪（合法输入不 panic）；多个只读 view 可并存；
  view 结束回到其来源状态。
- mut 排他：至多一个活跃 mut 借用，期间无只读 view；借入 mut 参数区域覆盖整个
  函数体。
- move 后禁用；唯一清理（每分配恰一个责任方）；owned 局部出口 LIFO；临时值存活
  至所在完整语句尾（使用先于清理）；move-in 参数由 callee 清理；被调产出的
  buffer return 转移回调用方。
- 拒绝码注册表：`verify.use-after-free / double-move / use-after-move /
  mut-conflict / borrow-escape / field-borrow / container-borrow /
  ownership-required / reassign-conflict`（无深度类码——资源层不引入
  语法/嵌套深度上限；PLAN-894 起 `verify.resource-assign` 的旧owned Str
  局部 blanket 拒绝退役为 [source-text-replacement.md](source-text-replacement.md)
  的 var 目标完整根替换准入，Drop 非拥有关联仍沿用该码）。被拒操作
  的 native 调用计数为 0（前门性质义务）。

## D-05 内部 ABI 五类形态（探针冻结；非生产模块 ABI、非 ACC 桥）

平台 Windows x64 / WindowsFastcall / Cranelift 0.126；整型/指针参数 RCX/RDX/R8/R9
后栈传；≤8 字节返回 RAX；**不使用 >8 字节聚合 sret 返回**（拒绝该候选：Win x64
sret 语义依赖类型类别，跨工具链手工复刻易错且无首批消费者）。

| ID | 形态 | 机器表示 | caller / callee 责任 |
|---|---|---|---|
| ABI-A | i32/bool/char 值 | I32 按值（bool 0/1；char=u32 位型） | 传前已合法 / 只读 |
| ABI-B | 借用 buffer | (ptr,len) 双参 | 借用期有效稳定 / 只读不保存不释放 |
| ABI-C | 拥有返回 | ptr 走 RAX + `*mut i32` out 通道（成功=长度≥0，失败=负状态码） | 取得即拥有、唯一清理 / 失败回收已分配资源 |
| ABI-D | record view | `*const Rec` | 借用期有效 / 只读字段 |
| ABI-E | record mut / out-return | `*mut Rec` / caller 预置存储 | 排他 / 全字段写入 |

record 布局：字段按声明序、各字段自对齐；`align(R)=max(字段对齐)`；
`size(R)=round_up(尾偏移+尾宽, align)`；**嵌套 record inline 值语义**（无指针，
赋值=拷贝）；out-slot 互不重叠；尺寸 u32 checked，溢出前门
`record.layout-overflow`。panic 不穿 ABI；预期失败一律值返回。

## D-06 资源（裁定 R1 显式拥有/借用）

- 分配器：进程堆 kernel32 HeapAlloc/HeapFree；不复用 Rust String/Vec 布局。
- **完整外部账本**：堆支持可增长登记表（键=用户指针，值=(长度,资源id)），无容量
  上限、不静默丢记录；账本元数据分配是内部簿记，**不计入用户三计数**
 （attempts/allocs/frees 仅计用户资源请求）；元数据失败独立注入点
  （`fail_ledger_nth`），失败时用户请求按 env-oom 完整回滚（回滚/旧表释放的 OS
  结果捕获接线）。
- **计数口径**：attempts=分配器调用（长度/预算检查后）；allocs/frees=成功数
 （frees=账本验证+HeapFree 非零）；free_attempts 与 invalid_frees 分立
 （双释放/外来指针=账本未命中，**不读取已释放内存**）。
- **数值域 u64 全精度**：live_bytes/peak u64（普通加/减、无饱和）；预算
  `set_budget(v≥0)` u64 原值保存（**无截断**；0=零字节预算，负值清除上限）；
  比较 `live_bytes + len ≤ budget` 全程 u64；释放即时归还额度；观测通道
 （SUMMARY 行）u64 打印。
- 失败类别（运行期状态值）：`res.len-overflow/-len-negative/-len-too-large/
  -budget/-env-oom/-shim-error`（与编译期拒绝、fatal trap 三分；码值 -1..-6）。
- 文本运行期码：`text.invalid-utf8/-invalid-scalar/-range-reversed/-range-oob`
  （-7..-10）与 `res.index-oob`（-11）。
- 零长度：合法；拥有零长 = null+len 0，不产生分配器调用，free(null) 无操作。
- fatal trap：进程级终止 → OS 回收全部进程内存；不声称析构、不承诺 unwind；
  trap 前驱可观察效果顺序保持；死循环由 contained executor 分类 Deadline。
- **候选 R2（受限引用计数句柄）淘汰**——理由限定为复杂度/前置：满足 D-04 前门
  需 per-slot active+generation+静态所有权追踪（h_get_checked 对照实证），实质
  在运行时表上重建 R1 的静态模型，且预算记账共享句柄下归属不明；不声称引用计数
  方案本身不可能。`shared`/跨任务 ARC 属后续线。

## D-07 下游接口

- 身份表（唯一来源，770/771 已同步）：`auto.hir.text.draft / 1 / text-i32-draft`、
  `auto.hir.record.draft / 1 / record-i32-draft`。
- 源级拼写：`c.to_codepoint()` / `char.from_codepoint(n)`（机器 shim API 与源
  拼写分层）。
- 消费面：770 消费 D-01..D-06 全部 + ABI A/B/C；771 消费 D-01/D-04 receiver/
  D-05 A/D/E（i32/bool record 无堆资源）。
- 共同热点：source AST/typecheck、hir、descriptor、verify、native、main——语义
  定义一处（本文件），不得各自造通用类型系统。
- 合并顺序：774 合入 → 770/771 并行、先成先合；后成者在新整合基线重放全门禁 +
  首个 profile 回归；非文本冲突回 new 修订。组合 text-record/List/enum/use：
  后续 owner（两分支单独通过不构成组合能力）。

## 组合消费规则（PLAN-782 text-record）

在 D-03/04/05/07 之上追加组合域消费（保留全部 774 探针身份与既有承诺）：

- **D-03 增行（record 字段语境）**：Str 字段拥有（构造/clone/move 单位=完整
  根）；字段读取为借用视图；`str` 字段拒绝面扩至 RecordShape。move-in 资源
  record 参数=callee 根（callee 出口逐 Str 字段缓冲清理）。
- **D-04 增行（字段来源对偶）**：字段借用 backing 溯源到根（构造代际沿
  770）；活跃字段借用期间整个根 mut/move/替换保守拒绝（不承诺同根不相交
  字段借用拆分）；来源五类的组合对偶全套沿 770，owner 集合扩至资源 record
  根；owned_returns 扩 record 根（out-slot 转移）。**字段写永不复活已整体
  move 出的根**（verify.use-after-move；仅完整根重赋可重建）。
- **D-05 适用域划清**：标量 copy（赋值=拷贝）仅保留在无资源类型；资源
  record inline 聚合但 Str 字段含内部拥有 buffer——显式 move=责任转移零位
  拷贝、显式 clone=逐字段深复制；布局沿声明序公式（Str 字段 12/8、资源
  record 8 对齐、copyable 嵌套保持 4 对齐，checked u64/u32）。
- **统一清理序**：语句临时（文本+聚合）与函数出口（plain-Str 根+record 根
  字段缓冲）均按**跨类型创建序** LIFO；短路聚合临时带跨嵌套组合的存在
  标记（外层 join 转发内层标记值、外层 skip 恒 0）。
- **故障双通道**：注入经 build 工具测试面（wrapper 前奏调用既有
  ac_rt_set_budget/fail_nth_alloc/fail_ledger_nth，无语言入口）；组合域
  status 失败的**部分输出**（语句 pending 临时）在 fatal trap 前逆序回收、
  旧目标不提交；回滚后仍 RT-FAIL+ExitProcess(70)（无 Result/unwind；live
  根依赖 OS 回收）。纯 text 域 trap 合同原样。
- **D-07 增行**：模块含任意资源构造/clone/清理义务 ⇒ 派生
  `requires: ["ac.runtime.text.v1"]`（纯 copyable 模块零需求；caller 声明
  不可绕过——双触发门沿 770）。

## 公共 AutoABI profile 分域（PLAN-797）

774 内部 ABI-A–E 与 `autoabi` 公共候选 profile（
[auto-abi/](../auto-abi/)）分属**不同兼容域**：禁止用内部 ABI 的兼容
承诺描述公共 profile（或反向）。资源 representation
（buffer/handle/callback）在公共 profile 中按
[auto-abi/windows-x64-v01.md](../auto-abi/windows-x64-v01.md) 执行，
不复用旧 text 内部 12/8 布局。

## 探针实证状态

验证包 `experimental/ace/tests/data-runtime/` 与 no_std staticlib 包
`experimental/ace/crates/data-runtime/` 均属 `experimental/ace/Cargo.toml` workspace；
复用 ace-core link/executor 真实入口，60s contained、locked。
**64 项探针测试 + 1 项永久记账状态探针**全绿；五轮复审反例包（7+3+4+1+2）重放
全绿。一键门禁 `scripts/verify-ac-data-resources-774.ps1`（8 步：警告正反控制/
check/test/fmt/记账状态探针/scope 隔离/哈希收据；check/test/fmt 原文与失败收据
落盘）。用例索引见 probe-cases.md（V-01..V-09 及 r1..r5 增补）；收据见
verification.md（提交后基线）。探针非生产支持声明。
