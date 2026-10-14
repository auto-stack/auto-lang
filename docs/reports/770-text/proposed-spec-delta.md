# PLAN-770 规范增量提案（proposed-spec-delta）

- 计划：PLAN-770 r2 · T-07 产出（AC-06 的一部分）；**本文件是提案，不是
  canonical 发布**。沉淀由 merge 阶段（T-09，`/auto-plan:merge`）按本表执行：
  `.autoos/specs.json` upsert + 模块回写 + `python scripts/spec-index.py`。
- 原则：五项 SD 只登记**首批文本/字符真实编译的实验性支持面与实证状态**，
  不宣称完整 token/lexer 编译、不宣称 ACC Gen1、不宣称生产 ABI，不改变
  core/scalar/record profile 的支持/拒绝集合（r6 起 771 已先合入，零回归
  由 V-10 联合 gates 证明：741/745/767 + verify-ac-record-771 实跑）。

## SD-01 add `docs/specs/auto-ac/source-text.md`

新文件（AC-01..05 的语言面合同）。内容骨架（源自
[design.md](design.md) §2..§7，merge 时按 Spec 模板重排）：

1. 状态行：experimental（首批真实 native 编译；非生产）。
2. 身份三重：source `auto.source.text.draft`、HIR
   `auto.hir.text.draft / 1 / text-i32-draft`、descriptor
   `schema/text.atom`（scalar 超集，只增不删）；未知身份受控拒绝；
   旧 profile 不自动获得文本能力。
3. 类型与语义面：str（借用 UTF-8 切片）/Str（拥有缓冲）/char（独立
   Unicode 标量）；字面量形态（普通/跨行保 CRLF/白名单转义/`\u{1-6hex}`；
   非法转义 `text.invalid-escape`、非法标量 `text.invalid-scalar`、
   多字符 `text.char-multi`、空 `text.char-empty` 均词法层拒绝）；
   **len/char_at/slice 按 Unicode 标量单位**（`A中😀` len=3、0/1/2 对应
   三字符；组合 e+U+0301 len=2）；**len_bytes 是显式 UTF-8 字节长度**
   （774 D-02；`A中😀` len_bytes=8）——两者不可混称；slice `[start,end)`
   空范围合法，负/反向/越界为运行期 RT-FAIL(-9/-10)+trap70；eq/ne
   字节精确（NUL 不截断、无 normalization）；排序不在本批。
4. 资源与语境（消费 774 D-03/04）：逐语境 str/Str；`.to(Str)` 显式提升
   （AP-4）、`.to(str)` 借用化不转移；`+` 双文本→拥有 concat（AP-3）；
   `.clone()`（AP-2）；字面量拥有语境提升；`=` 对 owned 目标拒绝
   （verify.resource-assign）；owned 块词法作用域
   （verify.owned-scope-escape）；借用不外逃（verify.borrow-escape）；
   `.move` 调用点转移（verify.ownership-required/use-after-move/
   double-move/mut-conflict）；LIFO 出口清理由 verifier
   TextResourcePlan 冻结、native 只消费。
5. 方法解析：内建方法按（接收者类型,方法名）绑定；用户同名函数按裸名
   绑定，两命名空间不冲突（遮蔽优先）。
6. char 面：`'x'` 字面量单标量；`==`/`!=` 标量值比较；
   `c.to_codepoint()` 恒成功；`char.from_codepoint(n)` 越界/代理为
   运行期 RT-FAIL(-8)+trap70；无隐式 char/int。
7. 能力/合同门：`ac.runtime.text.v1` 显式声明+提供；`--support-lib`
   伴随 `<lib>.contract.json`（api/revision/target/lib_sha256 对真实
   库校验）；失败码 runtime.contract-missing/-hash/-target/-api 在
   发布前拒绝；收据记录库+合同指纹。
8. 运行期出口策略：负状态→runtime 先写 `RT-FAIL code=<n>` stderr 行
   →生成代码 ExitProcess(70)；70 是测试 profile trap 码。
9. 边界：无 List/Map/record/enum 文本字段、字素索引/normalization/
   f-string/raw/正则、生产 print/File/process、公开 ABI；预算/故障
   注入无语言入口（归 runtime 测试）；trace 支持库与文本运行时**两个
   no_std staticlib** 各携带 `__rustc::rust_begin_unwind`，二者不能
   共链一个 PE——mark×text 共链面由 `test-support/marks.rs` 的纯 COFF
   对象承载（r1 复审 F-01 路径 A，已可用）。
10. 验证状态节：测试族索引（text_binding/text_hir/text_source/
    text_native/text_resources + runtime/text 三档 + verify-ac-text-770
    gate；引 verification.md）。

验收 ID：AC-01..05。

## SD-02 modify `docs/specs/auto-hir/project.md`

- before（现行 canonical 的确切状态；r6 刷新——771 已先合入）：由
  PLAN-741/767 沉淀、PLAN-772（计算管线修复）与 PLAN-771（record 首批）
  合入后的现行 canonical 登记**三个 HIR 域**——Core、Scalar 与 Record
  （`auto.hir.record.draft / 1 / record-i32-draft`：名义 record 类型、
  record_ctor/field_get/borrow_view/borrow_mut 表达式、field_path place、
  method 声明与参数 mode view/mut）；text 域及 str/Str/char 类型、12 个
  文本表达式节点、MutBorrow/MoveOwn 参数所有权模式、Drop 语句与资源计划
  **均未登记**（774 的 data-resource-contract 落在 auto-ac 侧）。
- after：增加 `HirDomain::Text`（`auto.hir.text.draft / 1 /
  text-i32-draft`）；BuiltinType/TypeKind 增 StrSlice/StrOwned/Char；
  ExprKind 增 12 个文本节点（str_lit/char_lit/text_len/text_len_bytes/
  text_char_at/text_slice/text_concat/text_clone/text_to_owned/
  text_to_borrowed/char_to_codepoint/char_from_codepoint）；LocalDef 增
  param_mode（View/MutBorrow/MoveOwn）；Stmt 增 Drop；CallBinding 增
  moved（与 771 的 record ParamMode 在实现树中统一为
  View/MutBorrow/MoveOwn——record 的 `mut` 即 MutBorrow）。verify_text
  分域入口 + 文本资源前门与 TextResourcePlan；各域 profile 防线互拒
  （text/record/旧域隔离由 check_profile_ops 统一承担）；旧域规则
  零改动。
- rationale：信任凭证（身份/类型/资源义务随 HIR 冻结）；AC-01/04。
- 验收 ID：AC-01/04。

## SD-03 modify `docs/specs/auto-hir/stage-contract.md`

- before：文本前端/清理 lowering 均为 planned/未登记。
- after：登记 text 前端（lexer/parser/resolve/typecheck/adapter 的
  文本面）与 native 清理 lowering 为 **experimental 已实现**（首批
  str/Str/char 真实 PE）；通用 pass（完整 token/lexer 编译、容器、
  record 文本字段）仍 planned；RT-FAIL+trap70 出口策略入运行期合同。
- rationale：阶段隔离（实验性实现与 planned 边界分开）；AC-04/05。
- 验收 ID：AC-04/05。

## SD-04 modify `docs/specs/auto-ac/project.md`

- before（r6 刷新——771 已先合入）：runtime 仅 774 探针实证；无语言层
  文本 runtime；native 能力为 core/scalar + record（771 首批，含
  copy-elision 与 ABI-D/E）；文本 runtime/native 未登记。
- after：登记 `runtime/text` 静态库（独立 workspace，`ac_rt_*` 内部
  ABI，774 合同同构 + text_eq/ne + RT-FAIL trace；.lib+.contract.json
  sidecar）与 source/HIR→native 文本编译能力（V-02..V-09 真实 PE）；
  双态表述：**实验性内部 ABI，非生产模块 ABI、非 ACC Cranelift 桥**；
  与 record（771，已先行合入）保持域隔离，不承诺 text+record 组合
  profile（r6 整合基线只证明共存零回归）。
- rationale：不冒称生产 ABI；AC-02/03/06。
- 验收 ID：AC-02/03/06。

## SD-05 modify `docs/specs/auto-acc/project.md`

- before：MD406/407/408 无文本 profile 取证；真实 lexer 剩余前置未列。
- after：登记 MD406/407/408 的有限 text profile 取证（lexer.at 的
  char_at/len/slice/拼接需求与 token.at 的 keyword_kind/kind_name
  需求现在有首批真实 native 语言面可对拍——但**完整 token/lexer 编译
  仍是后续 owner**，770 只提供语言原语）；不自动改旧人工绑定。
- rationale：双清单分离（已提供原语 vs 未编译完整 lexer）；AC-06。
- 验收 ID：AC-06。
