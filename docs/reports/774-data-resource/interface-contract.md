# PLAN-774 下游接口合同（interface-contract）

- 计划：PLAN-774 r3 · T-05 冻结（AC-01/04；D-07 交付物）
- 消费者：770（文本 profile）、771（record profile）。本表是两分支的**唯一**
  接口来源；语义定义一处（decisions.md §3），实现各自最小化，不建通用框架。
- 边界重申：以下全部是**内部运行时/ABI 合同与探针实证面**，不是生产模块 ABI、
  不是 ACC Cranelift 桥、不宣称任何 Auto 语言 feature 已实现。

## 1. 冻结符号表（探针 shim，`p774_*`；770 的 runtime API 同构重命名即可）

机器类型按 Windows x64 / WindowsFastcall；`i32`=I32、`ptr` = I64 裸指针。
770 落地时以语言侧名字包装（如 `res.own_make`），**签名形态不得改变**。

| 符号 | 机器签名 | 类别 | 语义（成功/失败） |
|---|---|---|---|
| p774_own_make | (seed: i32, len: i32, out_len: *mut i32) -> ptr | ABI-C / AP-1 | 分配 len 填充 (seed+i)&0xFF；out_len=长度或负码；null=失败/零长 |
| p774_own_free | (ptr) -> () | ABI-C | 唯一清理；null 无操作 |
| p774_own_clone | (src: ptr, len: i32, out_len: *mut i32) -> ptr | ABI-C / AP-2 | 独立分配拷贝 |
| p774_own_concat | (a: ptr, a_len: i32, b: ptr, b_len: i32, out_len: *mut i32) -> ptr | ABI-C / AP-3 | 长度检查先于读/分配；结果拥有转移 |
| p774_own_from_lit | (lit: ptr, lit_len: i32, out_len: *mut i32) -> ptr | ABI-C / AP-4 | 字面量提升（分配+拷贝） |
| p774_borrow_sum | (ptr, len) -> i32 | ABI-B | 只读聚合（探针观测用） |
| p774_borrow_read | (ptr, len, idx) -> i32 | ABI-B | 越界/负值 = -11 |
| p774_rec_view | (p: ptr) -> i32 | ABI-D | 校验和 = flag+2·count+3·tag（探针观测用） |
| p774_rec_mut | (p: ptr, addend: i32) -> () | ABI-E | count+=addend；flag 翻转 |
| p774_rec_out | (seed: i32, out: ptr) -> () | ABI-E | 全字段写入 caller 存储 |
| p774_text_scalars / p774_text_bytes | (ptr, len) -> i32 | ABI-B | 标量计数 / 字节长度 |
| p774_char_at | (ptr, len, idx: i32) -> i32 | ABI-B | 返回 char（u32 位型）；-7/-10 |
| p774_text_slice | (ptr, len, s: i32, e: i32, out_len: *mut i32) -> ptr | ABI-B | 借用切片（无分配）；-7/-9/-10 |
| p774_char_from_codepoint / to_codepoint | (i32) -> i32 | ABI-A | -8 = 非法标量 |
| p774_len_from_i32 / p774_len_to_i32 | (i32 / i64) -> i32 | ABI-A | D-01 转换边界 |
| p774_set_budget / p774_fail_nth_alloc | (i64 / i32) -> () | 控制 | 预算（存活字节）/第 n 次分配失败注入 |
| p774_shim_error / p774_summary / p774_check_invariant | () -> i32 / () / () -> i32 | 控制 | -6 / SUMMARY 行 / 不变量+SUMMARY |
| p774_trap_after_alloc / p774_hang_after_alloc | (i32) -> ! | 控制 | AV 0xC0000005 / 死循环（Deadline） |
| p774_h_alloc / h_clone / h_drop / h_get | (i32) -> i32 等 | R2 对照 | **仅淘汰证据用，770/771 不得消费** |

状态码注册表与模型拒绝码注册表：probe-cases.md §0（-1..-11 与 verify.* 8 码），
两命名空间不相交、均无深度类码。

## 2. ABI 类别与布局（同 decisions §D-05，此处为消费面摘要）

- ABI-A 值：i32/bool(0/1)/char(u32 位型)，4/4。
- ABI-B 借用：(ptr,len) 双参；callee 只读不保存不释放。
- ABI-C 拥有返回：ptr 走 RAX、长度/负码走 out 参；取得即拥有；**无聚合 sret**。
- ABI-D/E record：*const / *mut / caller 预置存储；布局 = 声明序、自对齐 4、
  size=4n、offset(fᵢ)=4i；无重排。
- panic 不穿 ABI；预期失败一律值返回；ABI-C 失败时已分配资源由 callee 回收。

## 3. 资源表示与计数（同 decisions §D-06，消费面摘要）

- 拥有 buffer = [4 字节头（magic+len 记账）][用户字节]；用户指针=base+4；
  free 按 header 验证，双 free 计数暴露且物理跳过（确定性）。预算按**用户字节**。
- 计数口径：attempts=分配器调用（长度/预算检查后）；allocs/frees=成功数；
  live/live_bytes/peak 同名；零长不产生 attempt。
- 正常清理 LIFO、全路径唯一；fatal trap 只记 OS 回收（SUMMARY 缺席为证）。

## 4. 770（文本）消费表

| 需要 | 来源 | 落点 |
|---|---|---|
| int 精度与内部长度/转换 | D-01 | source/hir 类型面；len 边界码 -2/-3/-1 |
| UTF-8/标量索引/char 名义/范围与 NUL/空串 | D-02 | len()/len_bytes()/char_at/slice 语义与码 -7..-10 |
| str/Str 逐语境 + 提升与容器禁令 | D-03 | 前门检查（含 verify.field-borrow/container-borrow） |
| 前门借/拥有规则与码 | D-04 | verify 层实现（native 调用计数 0 的拒绝路径） |
| ABI A/B/C + 拥有 buffer API | D-05/§1 | native 调用面（own_* / text_* / char_*） |
| 分配器/预算/注入/清理/trap 边界 | D-06 | runtime 侧 + V-03..06 同构用例 |
| HIR 身份 | D-07 | `auto.hir.text.draft / 1 / text-i32-draft`（770 注册） |

## 5. 771（record）消费表

| 需要 | 来源 | 落点 |
|---|---|---|
| int 精度（字段 i32/bool） | D-01 | 字段类型面 |
| receiver 规则（view 借用 receiver；显式 mut 自由函数；mut fn 不支持） | D-04 | verify 层 |
| ABI A/D/E + record 布局 | D-05 | native 调用面与 lowering 布局 |
| 无堆交互 | D-06 | i32/bool record 无拥有 buffer；清理规则不触达 |
| HIR 身份 | D-07 | `auto.hir.record.draft / 1 / record-i32-draft`（771 注册） |

## 6. 共同热点与合并顺序（D-07）

共同热点（两分支都改，语义定义必须一处）：`src/source/`（AST/typecheck 入口）、
`src/hir.rs`（域身份/节点）、`src/descriptor.rs`（schema）、`src/verify.rs`
（域入口与资源前门）、`src/native.rs`（ABI lowering）、`src/main.rs`（CLI 面）。

合入顺序（裁定）：

1. 774 复审合入（本合同入 canonical Specs 后）。
2. 770 与 771 并行实施；**先完成者先合入**（预期 770 体量更大，771 可能先成）。
3. 后完成者在**新整合基线**上：重放自身全门禁 + 首个已合 profile 的回归面；
   共同热点冲突优先以本合同/decisions.md 语义为准解冲突。
4. 非文本合并冲突（身份/签名/Drop/能力门不兼容）：暂停依赖实现，回到
   new 修订合同，不用宽松 verifier 或跨域 fallback 绕开。
5. 组合 text-record、List、enum、use：后续 owner（Design 36 §7），两分支单独
   通过**不构成**组合能力；联合正确性不是 merge-owned 延后项，组合 profile
   需重新 verify 与联合语料。

## 7. r1 复审修订（P774-R1-01..06 同步；取代上文冲突表述）

### 7.1 追加/修订符号（机器签名同 §1 约定）

| 符号 | 机器签名 | 说明 |
|---|---|---|
| p774_rec2_out | (seed: i32, out: ptr) -> () | 嵌套 record：Inner{x@0,y@4}+flag@8，size 12 |
| p774_rec2_view | (p: ptr) -> i32 | 校验和 x+2y+3·flag（D-05 嵌套布局） |
| p774_h_get_checked | (h: i32) -> i32 | R2 公平对照：active 追踪版 get（陈旧句柄=-11） |

**计数口径修订（§3 同步）**：`frees`=成功释放（完整外部账本验证 + HeapFree 非零）；
新增 `free_attempts` 与 `invalid_frees`（FREESTAT 行）；双释放不读已释放内存。
分配 op 成功时 trace 附 ` id=N`（资源 id，PE 进程内单调）；释放 trace `FREE id=N` /
`FREE-INVALID`。SUMMARY 六字段不变。

**预算语义**：`set_budget(v≥0)` 真实字节预算（**0=零字节预算**）；负值清除上限。

### 7.2 唯一身份表（D-07；770/771 下次 revision 必须采用）

| profile | 身份三元组（schema/revision/profile） | 现状漂移（下游责任） |
|---|---|---|
| text | `auto.hir.text.draft / 1 / text-i32-draft` | 770 §5.1 现为 `text-draft`，须改齐 |
| record | `auto.hir.record.draft / 1 / record-i32-draft` | 771 §5.1 现为 `record-draft`，须改齐 |

### 7.3 源级拼写（与机器 API 分层）

- char↔int：源级 `c.to_codepoint() -> int` / `char.from_codepoint(n)`；机器层
  p774_char_from_codepoint/to_codepoint。770 §5.1 的 `.to(int)` 须改齐。
- 字面量/转义/跨行/char 单标量规则与码（text.literal-newline / text.invalid-escape /
  text.invalid-scalar / text.char-multi / text.char-empty）：冻结于
  decisions.md §3.8 R1-04.4，770 词法层实现。

### 7.4 参数模式（D-03 修订）

参数默认 view（str 与 Str 同）；显式 `mut Str`（排他借用）；显式 `move Str`
（转移，调用点 `.move`，callee 清理，缺 `.move` → verify.ownership-required）。
借用参数不可保存/释放/逃逸；`return <view>` 仅借入参数合法。

## 8. r2 复审修订同步（P774-R2-01..04）

- 借用返回合法来源 = 存活借入参数**或静态字面量**（恢复 D-03 原文；§7.4"仅借入
  参数"作废）。模型来源五类：LocalOwned / ParamBorrowed / ParamMutBorrowed /
  ParamOwned / StaticLiteral；View/EndView 区域对全部可读来源可追踪，合法输入
  无 panic（v04_r2_source_mode_matrix 冻结合法/非法矩阵）。
- 参数显式模式三分适用于 str 与 Str：默认 view；显式 `mut`（借入排他 / 对拥有
  值排他）；显式 `move Str`（转移+调用点 `.move`）。`mut str` 区域覆盖整个函数体
  （无中途关闭/降级；返回其视图拒绝 borrow-escape）。
- 跨行字符串字面量合法（行界原样入值，LF/CRLF 不归一）；`text.literal-newline`
  作废。转义白名单 `\n \t \r \0 \ \"`（str）+ `\'`（char）+ `\u{hex}`；未知
  转义 `text.invalid-escape`；取材表见 decisions §3.9。
- 新增符号：`p774_fail_ledger_nth(i32)`（账本元数据失败注入；元数据分配不计
  用户三计数）。回滚/旧表释放的 OS 结果已捕获接线。
- 身份表（§7.2）与源级拼写（§7.3）已在 770/771 r2 同步修订中落地（主检出
  2026-10-13 书记提交），不再是"下游待办"。
