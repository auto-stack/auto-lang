# PLAN-787 可行性报告（T-01 冻结件）

- 基面：master `bf773ca9b`（783 已归档合入，D-783 门通过）
- 工作树：`D:/autostack/.wt/lang-787/auto-lang`，分支 `plan-787-dev`
- 证据提交：`c8eb1f216`（T-01/T-02 运行时核心 33/33）、`85686c3c0`（T-03 编译器+双 profile）、
  `386ccf982`（T-03 修复轮）、`f7021cb46`（T-04 迭代 unwind+恢复语义，65/65）

## 1. 冻结的包清单与依赖方向

| 包 | 路径 | 依赖 | 职责 |
|---|---|---|---|
| `ac-syntax-rt` | `experimental/ac-syntax/runtime` | blake3（仅此） | snapshot/position/handle/lexer/cst/parse/recovery/edit/outline 通用运行时 |
| `ac-syntax-grammar` | `experimental/ac-syntax/grammar` | auto-atom, auto-val, ac-syntax-rt | Atom 配置 → 程序编译器（校验+身份） |
| `ac-syntax-core-adapter` | `experimental/ac-syntax/adapters/core` | ac-syntax-rt/grammar, auto-ac-prototype | CST → ac-core Program 严格投影（T-05） |
| `ac-syntax-artifact-adapter` | `experimental/ac-syntax/adapters/artifacts` | ac-syntax-rt/grammar, ac-artifact-probe | A01/02/03 产物接入（T-06） |

依赖方向与计划 §2 一致：运行时不依赖 ac-core/Cranelift/auto-cache；语法文件位于
`experimental/ac-syntax/grammars/{auto-core,interface-mini-v1}.at`。运行时命令入口：
`cargo test --manifest-path experimental/ac-syntax/Cargo.toml -p <package> <filter>`。

## 2. 冻结的 grammar 配置 Schema（grammar-contract v1）

静态 Atom 文档：`grammar(name, revision, newline, stmt-end, sep, sync) { tokens{} rules{} pratt(t){} outline{} }`。

- token：`literal` / `charset`(+`cont`) / `quote`(+`escape`) / `line-comment` / `block-comment`(+`close`,`nested`)，`trivia: true` 标记；受限字符集（`a-zA-Z_` 区间语法），非任意 regex；非 ASCII 拒绝。
- 步骤：`lit(of)/tok(of)/ref(of)/pratt(of)` 或直写规则/token 名；`repeat: opt|many|list|list-nl`；`field`/`same-line`；包装节点 `list/list-nl/many/opt { x() {} }`。
- Pratt：`primary` + `op(binding, assoc)` + `group` + `call{args, same-line, nl-free}`。
- 生成期拒绝（定位到节点路径）：左递归（含经可空前缀）、可空 repeat（fixpoint 计入 repeat 语义）、未知引用、重复定义、非前进 scanner 规则、空 matcher、binding 越界。
- 换行策略旋钮（generic，非语言分支）：`newline`（敏感开关）、`stmt-end`（边界检查 `;`/`}`/换行）、`sep`（实际消费的分隔 token，仅 `;`——`}` 留给块规则）。
- 关键约束实测：**词法**最长匹配+同长声明序（关键字必须先于 ident 声明）。

## 3. 冻结的坐标/边界策略（T-01 要求的反 正反控制已入测试）

- 行以 `\n` 结束；CRLF 属同一行尾，`\r` 是行内普通字节；`between \r and \n` 是合法字节编辑点（snapshot 测试 `crlf_midpoint_policy`）。
- UTF-16 位置：emoji 代理对中点 **floor** 到标量起点（position 测试 `position_utf16_emoji_floor_policy`）；UTF-8 编辑边界拒绝非标量边界（`edit_rejects_overlap_and_bad_utf8`）。
- 行/段索引分页（`LineIndex` 按页换行表+前缀和），单次转换不全文扫描。
- 源页 16 KiB 上限、无下限（编辑局部性优先），内容寻址（blake3），目录=长度前缀和——开头插入后缀页 hash 原样复用（`insert_at_start_reuses_tail_pages`）。

## 4. 最小闭环证据（T-01 要求）

Atom(.at) → `ac-syntax-grammar::compile` → `GrammarProgram`（canonical 二进制+身份绑定往返：
`grammar_program_codec_roundtrip_from_atom`）→ 跨块词法（mini 嵌套注释/跨行字符串：
`grammar_mini_surface_parses_and_recovers`）→ 无损 CST（逐字节重建断言）→ edit→fresh
等价（edit 模块 splice/VersionedPublish 单测 + T-05 增量族）。

## 5. 双 profile 配置反控（V-02 前置证据）

- 关键字拼写 `fn→func`：同一运行时解析结果改变、旧拼写失效（`grammar_keyword_spelling_changes_parse_without_runtime_change`）。
- 优先级 `+`/`*` binding 互换：同一输入树形改变（`grammar_precedence_flip_changes_tree_shape`）。

## 6. 实施中发现并修复的关键缺陷（返工记录）

1. 多编辑同页重复拼接 → 联合跨度重建；2. List 把 `}` 当分隔符消费 → sep/stmt_end 分离；
3. Optional/RepItem 失败回滚到 alt 入口 → 步起点 Save；4. pratt bootstrap 不搬移后续操作数、
   运算符/括号前导 trivia 被 clear 丢失、close_call 重复挂载 args → 全部修复；
5. reduce 循环 vals 不足停滞 → 先弹 op 严格前进；6. 失败级联原生递归爆栈（4096+ 深嵌套）
   → 统一迭代 unwind；7. 恢复节点字节丢失/错位 → 前导 trivia 回扫重建+missing 定位；
8. 子帧消费不传播 → 恢复触发语义修复。

## 7. 公开接缝充足性结论

T-01 判据：现有接缝足以支撑 T-05（CST 字段/kind 命名空间已就绪）、T-06（canonical
program 二进制+Snapshot 页 hash 可直接入 A01/A02 key）、T-07（测试族命名已按矩阵就位）。
无需回规划。
