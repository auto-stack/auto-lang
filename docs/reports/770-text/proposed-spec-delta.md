# PLAN-770 规范增量提案（proposed-spec-delta）

- 计划：PLAN-770 r2 · T-07 产出（AC-06 的一部分）；**本文件是提案，不是
  canonical 发布**。沉淀由 merge 阶段（T-09，`/auto-plan:merge`）按本表执行：
  `.autoos/specs.json` upsert + 模块回写 + `python scripts/spec-index.py`。
- 原则：五项 SD 只登记**首批文本/字符真实编译的实验性支持面与实证状态**，
  不宣称完整 token/lexer 编译、不宣称 ACC Gen1、不宣称生产 ABI，不改变
  core/scalar profile 的支持/拒绝集合（零回归由 V-10 三旧 gates 证明）。

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
   len/len_bytes/char_at/slice 按 Unicode 标量索引（`A中😀` len=3；
   组合 e+U+0301 len=2）；slice `[start,end)` 空范围合法，负/反向/
   越界为运行期 RT-FAIL(-9/-10)+trap70；eq/ne 字节精确（NUL 不截断、
   无 normalization）；排序不在本批。
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
   注入无语言入口（归 runtime 测试）；trace 支持库与文本运行时不能
   共链一个 PE（`__rustc::rust_begin_unwind` 重复）。
10. 验证状态节：测试族索引（text_binding/text_hir/text_source/
    text_native/text_resources + runtime/text 三档 + verify-ac-text-770
    gate；引 verification.md）。

验收 ID：AC-01..05。

## SD-02 modify `docs/specs/auto-hir/project.md`

- before（现行 canonical 的确切状态）：`docs/specs/auto-hir/project.md` 由
  PLAN-741/767 沉淀，登记且仅登记两个 HIR 域——`HirDomain::Core`
  （`auto.hir.core.draft / 1 / core-i32-draft`）与 `HirDomain::Scalar`
  （`auto.hir.scalar.draft / 1 / scalar-i32-draft`）；类型面为 i32/bool，
  无 str/Str/char 类型、无文本表达式节点、无参数所有权模式、无 Drop
  语句、无资源计划（774 的 data-resource-contract 落在 auto-ac 侧，
  auto-hir 未登记任何资源身份）。
- after：增加 `HirDomain::Text`（`auto.hir.text.draft / 1 /
  text-i32-draft`）；BuiltinType/TypeKind 增 StrSlice/StrOwned/Char；
  ExprKind 增 12 个文本节点（str_lit/char_lit/text_len/text_len_bytes/
  text_char_at/text_slice/text_concat/text_clone/text_to_owned/
  text_to_borrowed/char_to_codepoint/char_from_codepoint）；LocalDef 增
  param_mode（View/MutBorrow/MoveOwn）；Stmt 增 Drop；CallBinding 增
  moved。verify_text 分域入口 + 文本资源前门与 TextResourcePlan；
  verify/verify_scalar 对 Text 域一律 verify.domain 拒绝；旧域规则
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

- before：runtime 仅 774 探针实证；无语言层文本 runtime；native 能力
  仅 core/scalar。
- after：登记 `runtime/text` 静态库（独立 workspace，`ac_rt_*` 内部
  ABI，774 合同同构 + text_eq/ne + RT-FAIL trace；.lib+.contract.json
  sidecar）与 source/HIR→native 文本编译能力（V-02..V-09 真实 PE）；
  双态表述：**实验性内部 ABI，非生产模块 ABI、非 ACC Cranelift 桥**；
  标量 record（771）不在本批。
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
