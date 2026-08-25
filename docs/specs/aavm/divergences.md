# AAVM v2 divergence 登记册(plan-432 起随移植进度维护)

格式:编号(定义见 divergence-rules.md §2 + 本册新增)× 文件 × 理由。
433 四向对比以本册为"合法差异白名单"。

## auto/lib/token.at(S1)

- **D11**(×1):`Option<TokenKind>` 返回 → `Ident` 哨兵(keyword_kind 的 None 语义
  折叠进 Ident 分支;调用方以 != Ident 判定)。
- **D11b**(×1):match 语句 → if 链(VM 对 match-on-string/enum 的支持未验证,
  v1 全程 if 链是已证路径;kind_name 139 臂同)。

## auto/lib/lexer.at(S1)

- **D12**: `Peekable<Chars>` → `char_at(i)` + p/at 游标(431 D1 的变体:
  免 Vec 构造,char_at 返回 UTF-32 码点,a2r 侧映射 `.chars().nth()`)。
- **D13**: Lexer struct 状态 → `lex_dump` 单遍局部变量(S2 需要 push_token 回退时
  再封装成真实 Lexer 类型)。
- **D14**: token 流/Vec<Token> → dump 串直出(kind_name 层)——M1 闸门判据;
  S2 换真实 Token 结构(dump 驱动保留为测试面)。
- **D15**: 注释 buffer 后继(CommentContent/CommentEnd 经 VecDeque 缓冲)→
  直出同序 dump。
- **D16**: Pos.len 按 char 计(基线按 UTF-8 字节)——corpus 限定 ASCII;
  多字节语料进 M1 前需补字节宽度记账。
- **D17**(结构性,最重要):循环体内一律不用 `continue`——**VM bug 实证**:
  循环体含调用语句(如 `var c = s.char_at(p)`)时,`continue` 穿透执行后续语句
  (probe p11/p12:分支执行后落入同迭代余下语句,p 双增、丢尾字符);
  `break` 不受影响。全部改写为 else-if 链。**挂 242 tracker:S1-432**。
- **D18**: 基线块注释 quirk 照抄——Rust `slash_or_comment` 的内容循环在 `*`
  分支 `push(c)+next()` 连吞两字符(内容中 `*` 后一字符丢失,如 `/* block` 的
  内容是 `*block`);M1 闸门以基线为唯一依据故照抄。**上游修复会改 golden**,
  挂 432 债务簿(建议修 Rust 侧并同步本文件)。
- **D19**: 数字 token 的 text/len 语义照抄基线——text 剥下划线与 f/d/u/i/u8/i8
  后缀,Pos.len 用剥离后长度(与真实消耗不等,at 记账随基线漂移);
  Tok.len(记账)与 Tok.adv(p 推进)分离承载。

## 计数(S1 时点)

| 文件 | divergence 处数 |
|---|---|
| token.at | 2 |
| lexer.at | 8 |
| 合计 | 10 |


## auto/lib/parser.at(S2)

- **D20**: AST 结构(Stmt/Expr 枚举 + Box 树)→ `parse_dump(source) -> str`
  S-expr 直出(判据层;真实 AST 推迟到 S4 codegen 需要时定)。dump 由各
  parse_* 构造期拼装,E 载体 `{kind,word,name,args,dump,ity}`。
- **D21**: token 流承载 `Vec<Token>` + pushback → `List.new()/push/get/len`
  (sanctioned);前瞻 = 索引算术(p_peek / paren_prefix 扫描)。
- **D22**: Result 错误传播 + add_error 收集 → `p.err` 单槽 + 全函数入口短路;
  错误信息编码行号与 token kind,parse_dump 直返错误串使闸门显式爆红。
- **D23**: Type 枚举 → Display 字符串载体(builtin 表折叠;StrFixed 恒显
  "str" 使该折叠无损);let 无注解时的 parser 内联推断(infer_type_expr
  核心子集:字面量/Ident 作用域查找/Bina unify 简式/Array/Tuple)在 E.ity
  构造期完成。unify coercion 分支语料未及,按 Unknown 传播。
- **D24**: float/double 值显示 = 字面量文本剥分数尾零("1.0"→"1");
  Rust 为 f32/f64 shortest-roundtrip Display,十进制文本字面量二者一致,
  科学计数/大数留 Missing。
- **D25**(VM 缺陷规避,与 D26 同族,①仍有残留):
  - ①原生调用(auto.list.new)内联作结构体构造参数时返回值丢失——根因
    即 D26-②的 num_locals 启发式偷弹(sp 含兄弟表达式槽);RET 恢复修复
    后语句位安全,但表达式位零参调用的歧义仍在(启发式无法区分兄弟槽与
    参数),.at 侧维持"提升局部变量"写法规避。
  - ②List.pop 语句位静默毁栈/返回垃圾——D26 修复中一并修好(先 +1 回栈
    再释放,字符串哨兵保 TAG_STRING);.at 侧 depth 计数写法保留(更简)。
- **D26**(**已修复**,2026-08-24,详见 plan 432 执行结果):VM 字符串池
  RC 回归曾阻断 M2——循环体内以运行期字符串调 `List.push` 后读回即 UAF
  (`[RC canary] string tombstone access`)。根因两层:①`ListData<i32>`
  以负哨兵 -(idx+1) 存字符串(nano_value.rs encode_string 契约),但
  push/set/insert/clear/pop/remove 只对堆 id 记容器份额,字符串哨兵从不
  retain——调用方栈份额被 native 死区结算释放后,列表持裸哨兵指向已回收
  池槽,读回 +1 即复活墓碑;②`task.num_locals` 由 RESERVE_STACK 设置但
  RET 从不恢复,用户函数返回后 shim_list_new 的"有参"启发式
  (sp > bp+num_locals+2 即偷弹)读到被调者的局部数,sp 偶然越界即把
  兄弟表达式的值偷去当初始列表(len=4 幽灵元素)。修复:native.rs
  list_i32_elem_retain/release 记账五处 + child_pool_idxs 随容器死亡
  释放(types.rs/rc.rs/heap_object.rs)+ pop/remove 改"先 +1 回栈再释放"
  (原顺序在末次引用时自造悬垂,堆 id 分支同修)+ RET 恢复 num_locals
  + add_string dedup 命中墓碑防御。复现测试 repro_242_string_pool_uaf
  常驻;**M2 闸门 18 语料 diff=0 转绿**。堆侧同族缺陷(242:S2-432 续)
  亦已修:P419_UAF_TRACE=4000001 生死链显示 shim_list_map/filter 四处
  新建列表 id 以裸 push_i32 入栈(违反"首次 rc_push 建条目"契约)——
  栈拷贝无份额 → STORE_LOC 转移进局部槽仍 0 份 → copy-on-load 的 +1
  成"首次获取" → pop_arg 释放归零即对象在局部槽仍引用时死亡 → 槽位
  复用后 LOAD_LOC_1 RETAIN-AFTER-FREE → canary。修复 = 四处改
  rc_push_id;conformance_bootstrap 与 conformance_023(map/filter 行为)
  双双转绿。剩余 6 个 master 存量失败(5 cookbook "Assertion failed"
  行为断言 + charts 环境问题)与内存安全无关,另行处理。

## 计数(S2 时点)

| 文件 | divergence 处数 |
|---|---|
| token.at | 2 |
| lexer.at | 8(+S2 重构后 tokenize/lex_dump 包装,D13/D14 落地) |
| parser.at | 7 |
| 合计 | 17 |

## auto/lib/typeinfo.at(S3)

- **D27**: infer/expr.rs + codegen `.type` 属性 → `typecheck_dump(source) -> str`
  行式输出(判据层)。AAVM 不建 AST:自带语句级走查 + 优先级爬升型推断器
  (优先级表复用 parser.at 的 infix_l/infix_r/prefix_power/postfix_power,
  游标/类型解析复用 p_kind/p_next/parse_type/is_type_name/p_bind/p_lookup,
  不调用 parse_* 语句族避免 dump 副作用)——"解析与推断分离"(风险表
  第 1 行的预设路径)。类型仍以 Display 字符串为载体(D23);数组型
  "(array-type (elem T) (len n))" 与 Rust Array Display 同形,元素提取按
  该形状解析。TFnSig 注册表 = TypeStore 的 S3 裁剪(仅 fn 签名;类型/spec
  声明表 S4 按需)。未知型输出 "unknown" 对齐 Type::Unknown unique_name。
- 闸门锚点考叝始末:解释器路径无独立 typeck pass(infer/stmt.rs check_body
  仅 a2r 调用且丢弃、ParamChecker 零调用者),`.type` 行为通道(codegen
  infer_expr_type → infer_expr)是类型层唯一含 fn 返回传播的可观察输出,
  故 M3 = corpus_m3(可执行程序打印 .type)Rust 侧真执行 stdout vs AAVM
  typecheck_dump 逐行对比。混合算术 coercion/块级作用域/显式注解冲突
  检查 = Missing(语料未含,登记 typeinfo.at 头)。

## 计数(S3 时点)

| 文件 | divergence 处数 |
|---|---|
| token.at | 2 |
| lexer.at | 8 |
| parser.at | 7 |
| typeinfo.at | 1(+D23/D22 复用) |
| 合计 | 18 |

## auto/lib/codegen.at(S4)

- **D28**: Vec<u8> 字节码 + 链接器重定位(loader.rs)→ 指令 List
  (I{op,s,n})+ codegen_dump 序列化直出 —— 发射顺序/操作数逐字对齐
  Rust(考古见 m4-bytecode-format.md);FN_PROLOG/RESERVE 由 Rust 的
  "体后插入+地址平移"改为"占位+回填"(布局等价,免平移);CALL 的
  FuncCall reloc 改为 FnEntry 符号表序列化期解析;槽释放组按槽位升序
  (Rust HashMap 迭代序不定,dumper 同步归一)。作用域 depth 计数
  (List 只增不减,D25 写法)。
- 实现期修正:.type 属性停走(借 typeinfo.at 的 t_is_type_prop)、
  fn 体 { 后前导换行、纯赋值不预载 lhs、调用原子落入中缀环、
  str.cat 字符串加法、load.loc.2 快操作码、for-range 的 start 以
  min=18 界定不吞 `..`、I.n 载荷补齐(const/ret/prolog/local 编址)。

## auto/lib/engine.at(S5)

- **D29**: 字节码 Flash + ip 解码 + 任务调度 → 指令 List 直译
  (ip=指令索引;jmp/call 目标即索引);print 输出 → out 字符串收集
  (ev_run 返回,ev_run_t 带 trace 诊断模式);栈 = Auto List + 手记
  账 sp(ev_push 兼容 push/set);值 = Auto 原生值,布尔以 1/0 整型
  承载(规避宿主 ListData<i32> 的 bool nanbox 解码冲突——bool 值压入
  无类型 List 会被 decode_i32 成 i32::MIN 触发哨兵取负溢出,挂 242
  编码别名账);RET 帧/参数槽算术逐条对齐 engine.rs(参数
  bp-n_args+rel-1;cur_args 按帧入栈/恢复);main 入口查找无 main
  回落 wrapper=0(镜像 lib.rs 1183)。
- 调试插曲(留档):.at 侧 cg_eos 漏传参(1 参调用 2 参 fn)曾致
  宿主栈帧逐迭代蚀一槽——宿主对参数个数不匹配无任何诊断,静默毁帧,
  挂 242 建议账(调用方/被调方 n_args 一致性检查)。

## 计数(S4/S5 时点)

| 文件 | divergence 处数 |
|---|---|
| token.at | 2 |
| lexer.at | 8 |
| parser.at | 7 |
| typeinfo.at | 1 |
| codegen.at | 1(+D22/D25 复用) |
| engine.at | 1 |
| 合计 | 20 |


## M4 扩语料(2026-08-24 续,S5 未竟项收口)

- **D30**: 负 int 值域偏置编码(engine.at ev_enc/ev_dec,−1e9)。宿主
  无类型 List(ListData<i32>)的负值与字符串哨兵(-(idx+1))本质别名,
  负 int 经 set→get 往返读回池字符串(实测 -1 → "len")。配套宿主修复
  (blocker 级,计划纪律内已记录决策):native.rs push_tagged_value_rc
  增池界检查,越界哨兵回落裸 i32(此前为悬垂 string tag,读回
  "<invalid string index: N>");回归测试 repro_d30_negative_int_roundtrip
  常驻,全量零新增失败。v2 侧:const.i32/neg/算术/比较/下标在出入栈界
  编码解码,数组元素保持编码域一致,print 经 ev_fmt 解码;字符串对
  `v <= -1e9` 比较为 false(宿主实测),ev_dec 对字符串安全穿透。
- **D31**: 数组值 = Auto List 直存 v2 值栈(宿主 List 引用语义,
  set.elem 原地写生效),无 tag 编码/堆 id 间接层。整表 print 不可比:
  宿主 print(List) 打标签 id("4000000" 样),两侧堆状态不同必漂移
  ——语料以元素/len 为观察通道,避免整表输出。
- 下标赋值发射序镜像 Rust quick-fix(codegen.rs 5954):rhs→arr→idx→
  set.elem,value 展栈底;v2 单遍步行以游标快照两段解析(P.pos 直存
  直取,先发 rhs 再回头发 arr/idx)。
- 顶层非 fn 语句:Rust compile_and_link 的 other_stmts 走 compile_stmt
  无 .line(行号发射属 fn 体路径),v2 wrapper 步行同规则。
- 一元负号:operand+neg 镜像 Rust 6414 int 路径(neg.f/neg.d 为
  Missing);一元分支整合进 cg_expr 原子链(原 Not 提前 return 会漏
  后续中缀(如 `-x + 1`)形态,顺手修正)。
- .len():仅数组接收者(cg_expr arr_flag + CGVar.arr 绑定跟踪,镜像
  Rust infer_object_type==Array 闸 7177);str.len 走 CALL_NAT 家族,
  v2 侧 Missing(语料不含)。
- 布尔可观察行为对齐确认:宿主 print 对 bool 一律打 "1"/"0"
  (print(true)/print(1<2)/print(!false) 实测),v2 1/0 整型承载即
  正确编码,无需字符串化(本会话曾据旧 run_eval 金样反向实施后依
  实测回退;旧金样 "r1:true" 是 run_eval 返回值格式化,非 print 通道)。

## 计数(M4 扩语料时点)

| 文件 | divergence 处数 |
|---|---|
| token.at | 2 |
| lexer.at | 8 |
| parser.at | 7 |
| typeinfo.at | 1 |
| codegen.at | 2(D28 + 下标赋值游标快照两段步行;另复用 D22/D25) |
| engine.at | 2(D29 + D30/D31) |
| 合计 | 22 |

## 433 a2r 闭环(.at 侧改写,六闸门保持绿)

- **D32**(全库):裸 `List` → 类型化 `List<T>`(toks/ins/pool/fns/scopes/
  args_stack/offs/jzs/jends/parts/stmts/dumps/itys/elems)。VM 动态语义
  不变;a2r 需元素类型(Vec<Tok> 等),裸 List 会错映射为 Vec<String>。
  类型标注声明点:struct 字段/fn 参数与返回/var 标注;由函数返回值初始化
  的局部由 a2r 新增的 fn_ret_types 预扫描推断(433 a2r 修复)。
- **D33**(parser/typeinfo/codegen):结构体引用语义 → `mut p P`/`mut c CG`
  参数(a2r-11 机制:&mut 穿透 + 调用点 reborrow)。VM 忽略 mode 关键字,
  行为不变。
- **D34**(engine):值栈异构载体 → Val 判别器结构体 `{k, i, s}`
  (k:0=int/1=str/2=arr)。不用 enum:VM 枚举载荷值跨函数传参丢标签
  (probe20/22/23 实证:NONE/空串/i32 哨兵泄漏;结构体对照组全绿),
  **挂 242 tracker**。
- **D35**(engine,D31 续):数组值 = arena 侧表槽位,VArr 的 i 即 arena
  索引(索引即引用,镜像宿主堆 id)。VM 引用语义与 a2r 值语义在
  "arena 槽位 = 单一事实源"上对齐:set.elem 的唯一变异点 ev_arr_write
  (VM 原地写 + 写回无操作;a2r 克隆修改 + 写回)。
- **D36**(若干):VM 缺陷规避写法(与 D25 同族):
  ①原生调用不入(枚举/结构体)构造参数——`Val.VStr(pool.get(i))` 载荷
  丢失,先提升局部(load.str / get.elem str 分支);
  ②`ev_push(stack, sp, stack.get(x))` 的借用冲突(a2r 侧 &mut stack 与
  读 stack 并存)——读值先提升局部(load.loc.0/1/2、load.local、dup);
  ③`.str()` 调用与 str 拼接不入 str 参数/拼接位——提升局部
  (cg_emit_store/load、const.i32、.line、ret、ev_add);
  ④`parts.push(lhs)` 后条件重赋值触发移动分析(possible-skip reinit)——
  push 副本(`var lh2 = lhs + ""`,VM 语义等价)。
- **D37**(a2r 侧修复配套,VM 无关):int 与 char 字面量混型比较/减法由
  a2r 发射 `('x') as i64`(VM 码点语义对齐);`.len` 在含 len 字段的
  结构体上是属性读而非方法(Tok.len);merge 模式跨模块 mut 参数表与
  fn 返回类型表预扫描;bootstrap 自测 main 仅在 run_eval 存在时追加。

## 计数(433 时点)

| 文件 | divergence 处数 |
|---|---|
| token.at | 2 |
| lexer.at | 8(+D32 类型化/D37 char 由 a2r 消化) |
| parser.at | 7(+D32/D33) |
| typeinfo.at | 1(+D32/D33/D36-④) |
| codegen.at | 2(+D32/D33/D36-③) |
| engine.at | 3(D29 + D30/D31 重构 + D34/D35/D36) |
| 合计 | 23 类(累计登记;433 新增 D32-D37 六类) |

## 434 AA2R(Auto 版 a2r)

- **D38**(宿主/前端配套修复,Plan 434):AA2R 自举路径上的缺口收编——
  - **D38-VM**(宿主修复):`Str.char_at` 字节索引落在多字节字符中间时
    `s[i..]` panic → 边界安全化(`s.get(i..)`,非边界返回 0;ASCII 行为
    不变)。v2 lexer 首次扫描含 CJK 注释的 lib 源时触发(此前 corpus 全
    ASCII,该路径从未执行)。
  - **D38a**(parser.at):`parse_type` 增泛型实例 `Name<A, B>`(Display
    对齐 unique_name;M2 语料不受影响,新增 p15/p16 常驻闸门)。
  - **D38b**(parser.at):增 type-decl/enum-decl 声明(Display 对齐
    ast/types.rs TypeDecl::fmt 与 ast/enums.rs EnumDecl::fmt;members-only,
    枚举 gap-fill 对齐;成员/变体间空行 Display 无载体静默忽略)。
    用户类型经注册表**内联整个声明快照**(已声明→完整 decl;前向引用→
    `(type-decl (name X))` 空声明)——对齐 Rust `Type::User(TypeDecl)`
    的 Display 行为;p15 语料实证。
  - **D38c**(lexer.at):f-string(`f"..."`/`f"""..."""`/反引号)与三引号
    多行字符串——f-string 发射 **FStrStart + FStrPart(整段内容)+
    FStrEnd** 简化形(主 lexer 为逐段结构化 token;`$name`/`${expr}` 由
    a2r.at 发射侧解析,`${expr}` 内容原样直通,校准子集见 D40);
    三引号内容实换行入 Str。M1 语料不含此类 token,闸门不受影响。
  - **D38d**(a2r.at):作用域栈 unscope 时**清空弃用槽位**——depth 计数
    模式(D25)下,上一函数分支作用域的残留条目会在相同槽位被下一函数
    的查找命中(跨 fn 同名遮蔽,`return b` 被陈旧 param=0 条目遮蔽的
    实证);清空后槽位复用无污染。
- **D39**(a2r.at 方法论):AA2R 不经 parse_dump 的 S-expr(字符串字面量
  内引号歧义:lexer 解码后的原文重嵌入 dump 无法区分真实引号),改以
  token 游标直走(432 codegen.at 同款),复用 parser.at 的 p_* 游标/
  优先级表与 typeinfo.at 的 t_is_type_prop;预扫描类型表走**裸名形**
  (不经 D38b 内联 Display——那是 dump 判据层口径)。
- **D40**(a2r.at 与主 a2r 的已知文本差异,行为等价):主 a2r 的
  mut-参数签名 `mut c: &mut CG`(绑定可变位)按逃逸分析选择性保留,
  AA2R 统一省略(行为等价,unused-mut 由 allow 抑制);主 a2r merge 模式
  的逐文件 `// Auto-generated` 分隔注释未复刻;struct 派生三分派
  (float/List→3 派生)与 else-后空行规则等格式细节已对齐;`${expr}`
  内容原样直通(方法调用/复杂表达式形态的 f-string 插值不支持)。
- **主 a2r 缺陷(已修,242 #18)**:a2r.at 曾在主 a2r 下转译 45 错——链式类型
  推断缺口(Index 元素/clone 透传/Dot 泛化)致 `.get(i).field` 发射 Option 形
  (E0609)、赋值位字段读/ident 重绑定缺 auto-clone(E0507/E0382)、str 型入
  String 位缺 to_string。2026-08-24 修复后 ② 回归整目录(含 a2r.at 七文件),
  五方矩阵全绿,golden 零回归;细节见 242 tracker #18。

## 计数(434 时点)

| 文件 | divergence 处数 |
|---|---|
| token.at | 2 |
| lexer.at | 8 + D38c(+D32/D37) |
| parser.at | 7 + D38a/D38b(+D32/D33) |
| typeinfo.at | 1(+D32/D33/D36-④) |
| codegen.at | 2(+D32/D33/D36-③) |
| engine.at | 3(D29 + D30/D31 + D34/D35/D36) |
| a2r.at(新) | D38d/D39/D40(方法论文档于文件头) |
| 合计 | 26 类(434 新增 D38a-d/D39/D40;宿主修复 D38-VM 一项) |

## 447 部分①②(宿主加固 + aavm 语法能力;2026-08-25)

- **D41**(永久,语言能力边界):Auto 无"模式 + guard"组合臂——Rust
  `VI(n) if n > 10 =>` 在 Auto 解析报错(卫语只能独立成臂 `if cond ->`)。
  一对一改写时拆独立卫语臂或调臂序;宿主 H4(447-①)保证独立卫语臂
  的 a2r 发射正确。写法规范定稿于 divergence-rules.md §4a-6。
- D11b/D34/D36①② 的**收账**在部分③(γ 系列逐步还原时逐条标注终态,
  见 11.3);本节仅登记 D41 与规范修订。

## 计数(447-①② 时点)

| 文件 | divergence 处数 |
|---|---|
| (全库前瞻) | 26 类 + D41;D11b/D34/D36①② 待部分③ 收账 |
