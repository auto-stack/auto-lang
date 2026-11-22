# M2 标准库核心（装配面）— 当前知识

> Current-state · PLAN-972 交付沉淀（2026-11-15；基线 commit c1b85904b）。
> 本文档是装配标准库（assembled std set）的权威现状：能力面、执行端、
> 分层、边界与已裁定偏差。历史与逐轮证据见归档计划
> [972](../../plans/archive/972-ace-m2-stdlib-core.md)。

## 能力面（62 公共符号，8 模块）

装配集 = `stdlib/ace/asm/{list,math,text,char,conv,hash,bytes,map}.at`，
经 `stdlib_assembled_files()` 冻结身份（include_str! 仓内源；仓外/漂移/
provider 层在绑定前 `ace.stdlib-reserved` 拒——防遮蔽）。lane =
packages-aggregates-ct 装配线（1043 来源后缀装配合同的消费面）。

| 模块 | 公共符号 |
|---|---|
| list | sum/find_first_gt/pop/total_len/reduce/find/any/all/filter/map/sort |
| math | abs/min/max/square/cube；abs_f/sqrt/powf/sin/cos（纯 Auto double 族） |
| text | byte_len/scalar_len/char_code_at/equal/starts_with/ends_with/contains/repeat/upper_ascii/lower_ascii/find/reverse/trim/replace/split/lines |
| char | is_digit/is_alpha_ascii/is_alphanum_ascii/is_ident_ascii/is_whitespace/to_upper_ascii/to_lower_ascii（codepoint 形） |
| conv | to_int/is_uint/from_uint/from_int |
| hash | codepoint_hash（S-06 裁定：codepoint 域默认） |
| bytes | byte_at/utf8_len/byte_of/byte_hash（byte 域）/to_hex/to_base64 |
| map | map_new/put/get/has/len/remove（交错 [k,v] 于 List<int>，引用语义） |

默认算法**全纯 Auto 单源**（AC-AUTO-STD：不以 libm/Rust 容器/C 库代签；
无 intrinsic 路径；外部引擎平价归 967）。double 族：sqrt=4^k 归一+Newton，
sin/cos=一次 trunc 归约+Maclaurin，powf=atanh-ln+Maclaurin-exp+2^k 缩放，
全递归。**已裁定偏差**：NaN 不可构造（checked ÷±0）→ sqrt/powf 非正输入
返 0.0；归约域 |sin/cos x|≤~1.4e18、|y·lnx|∈i32 域；~1e-15 相对精度非
正确舍入平价（五端同源同 IEEE 指令序 → 跨端同值）。

## 执行端（五端真跑同值）

AV（in-process）/ AC-native（mn 真机码 exit）/ A2R（rustc）/ A2C（cl）/
A2PY（真 OS Python）。常驻回归 = `v08_surface_972`（面清单奇偶校验 +
62 符号全演练独立 oracle 五端同值；删行/私增/改名即红）。六路径覆盖中
AV×C/AV×Rust 外库 dispatch 挂 967 merge（外部时序依赖，具名）。

## f64 发射面（T-09）

浮点字面量须显式宽度后缀（`2.5d`/`2.5f`；裸小数词法拒）。mn：cell word
承载 IEEE 位型（bitcast 按用）；fcmp 六比较/fadd/sub/mul/fdiv（÷±0 陷阱
-31）/fneg；Conv 三形（F32↔F64 精确；float→int 按 2^n 精确幂界守卫+sat
转换；int→float 源宽符号恢复+fcvt_from）。a2x-c：float/double 值型+结果
通道（a2x_result_f64/f32 三表）+比较内联/÷% 经 checked 助手。
a2x-rust：浮点比较内联（曾落助手名匹配前拒）。a2x-python：Collections
族参数/返回准入浮点。

## core/alloc 分层（T-10/V-10；978 具名清单）

core 28 符号（math 10/char 7/conv.to_int+is_uint/text 长度·索引·equal/
bytes 走查族/hash）不触 arena；alloc 26（text 物化构串族/conv 构串/
bytes 编码/list 全部/map 全部）经 mn arena=1039/1040 原生底座，无 malloc/
隐式宿主堆。V-10 证明（`v10_layers_972`）：core-only attempts=allocs=0；
alloc 平衡（invalid frees=0，账目闭合）；fail-Nth 注入=资源 trap 22+全清扫
无泄漏；欠配字节预算同陷阱。观察/注入/预算经 `MnDriver.resource_knobs`。

## 发射器修复（本包首触，常驻防回归）

mn I64 Div/Rem 在 I64 宽发射（I128 sdiv/srem 无 x64 降载）；mn 文本臂
（StrSlice/StrConcat/StrToOwned）入 dst 绑定表（新节点绑指纹，否则 Drop
校验 refusal=5/trap 70）；mn Conv 源宽符号恢复（窄 cell 零扩展字直
sextend 曾致负值 I32→I64=4294966875）；a2x-c `_mul128` 返回低位/出参高位
（曾颠倒——一切 i64 乘含 3×5 误溢出）；a2x-c View Record/Variant 实参
取址+VariantProject 根指针形+const 转换；a2x-rust View-List 参数名裸传。

## 源级纪律（模块内注释即规范）

owned Str 自由函数实参=移动（递归行走先 clone——contains_at 纪律）；尾调用
实参表先 bind 借用再移；borrow Str 参数直返须 clone；List<Str> push 元素
须整局部 `.move`；穷尽 `is` 于非 int/unit/Str 返回函数被面 filler 拒（匹配
收入 int 助手）；`to/from` 等保留字不作参数名；Str var 重赋值/取 s 的
helper 触 use-after-move；use 别名解跨模块同名。

## 已知边界与债务

- **a2x-c 发射器（KNOWN-DEBT）**：mut-ptr 局部被别名进"被返回的调用结果"
  时尾声双 free（堆损坏）——装配 stdlib 已源级规避（积累器 int 步函数+
  外层 tail-return 自有局部）；任意用户代码可触，修复=别名逃逸分析。
- mn/C 位算（& | ^ 发射）与 float rem：语法门已开（packages-aggregates-
  ct-ops pratt 表），发射面具名后续。
- hex/base64 解码：byte→Str 构造本面无（结构边界）；全 Unicode case/分类
  表：宿主数据表（967）；csv：数据表批；Str 键/泛型 map、比较闭包 sort：
  963 全泛型切片；regex/crypto（旧 #[vm] 族）：967 外部引擎；FStr 求值：
  964；I/O 原语：1041/973。
- Linux 执行腿：2026-11-14 裁定 NOT_RUN/owner=1063（Windows required 全跑）。

## 验证锚（常驻）

`stdlib_asm_972`（AV 正反例+五端整集腿+所有权/身份负例）、
`v08_surface_972`（面清单+62 符号五端）、`v10_layers_972`（core/alloc/
失败注入/预算）、`probe972`（装配 lane 前探）。冻结并集（T-01 表）为
merge 级门。

## PLAN-1082 增量（2026-11-20，HOF/List.new 方法面接 972 实集）

- **方法面直调冻结 stdlib**：`map/filter/reduce/find/any/all/pop` on List<int> 经 `compile_language_source` 按需 graft 冻结 `stdlib/ace/asm/list.at`（include_str 身份；`__std_list_` mangle + `pub` 剥离 + 方法形不动；字节稳定单测 `adapter::stdlib_list_graft_tests`）后 `CallFn` 直调——同源同编译器，无第二算法；packages-aggregates-ct 装配线身份/规则零改动。
- **`List.new(e1..en)` 受检兼容糖**（auto 面）：元素 VT 取自 e1 + 同质校验 + 同 push admission；零参具名拒 `source.list-new-empty`（`List<T>.new()` 承载空表）；不新增任意用户 varargs 语法。
- **边界**：~~非 i32 元素 `source.hof-elem-domain`（963 全泛型切片 owner）~~（PLAN-1120 起非 i32 元素的 map/reduce/find/any/all 走源内 typed 脱糖车道，见 language-parity PLAN-1120 增量；`filter` 已先行同车道；嵌套 List 元素仍具名拒）；`for_each`/`contains`/`sum`/`sort` 等未接方法面（972 集内未列或自由函数面，legacy 迁移以 `find`/`is` 规范化审计在案）；`pop` 保持 None-not-trap 空合同。
- 证据：`av.parity.1082.hof-real-call`（42）、`hof-family-legacy`（15/4/0/1）、`hof-pipeline-legacy`（60/5/5）、`hof-empty-face`（0/0）、`shims-int-list`（3/20/99/4/30/2/1）。

## PLAN-1120 增量（2026-11-23，非整数元素泛型 HOF 方法面）

- **整数实例限制更新到实测覆盖**：冻结 `list.at` graft 仍只承载 `List<int>` 实例（sum/find_first_gt/pop/total_len/reduce/find/any/all/filter/map/sort 实测公共方法覆盖未变）；非整数元素的 `map/reduce/find/any/all`（及先行落地的 `filter`）不再具名拒 `source.hof-elem-domain`，改由源解析器内联 typed 脱糖（snapshot 隔离 + 源序访问 + 冻结闭包/所有权契约），与整数 graft 车道并存、无第二 stdlib 算法。
- **回调与持有规则**：回调单参须为元素类型（`List<StrOwned>` 接受 `fn(Str)` 借用视图，同 filter）；记录元素走借用视图车道，其余 owned 元素每次访问克隆；`find` 绑定 AV `?T` Option 合同（Some 载荷 owned 元素 Move 入槽）；`reduce` 累加类型取自 init、可与元素类型不同，回调须返回累加类型；回调类型不符在绑定时具名拒 `source.type-mismatch`；`any/all/find` 短路、空表零回调、reduce 空表保持初值。
- 证据：`av.parity.1120.generic-list-hof`（`experimental/ace/crates/av/tests/cases/plan1120_generic_list_hof.rs`，9 例：V-01 独立预期 `2 2 world true true 3 1 3 6`、V-02 空表/未命中/短路计数/绑定拒绝、V-03 所有权/分配故障重放/回调失败经 try-catch 后输入可续用、int 近邻 graft 不变）；旧 `source.hof-elem-domain` 负控行迁移为回调类型不匹配负控（plan1082 `hof_non_int_callback_type_rejects`）。

## PLAN-1121 增量（2026-11-23，普通面 HashMap/HashSet 迁移容器）

- **迁移容器（非装配集）**：`stdlib/ace/asm/map_str.at` = 纯 Auto 记录类型
  `__std_HashMap`/`__std_HashSet`（keys/tags/ivals/svals 与 items 并行 List 底座，
  for-条件扫描，put owned-Str 移入、get_str 显式元素 clone、缺键/错面 `fail` 受检通道）。
  **不属于** 972 装配 62 符号面（`stdlib_assembled_files()` 不列本文件；v08 奇偶校验零改动）。
- **嫁接与拼写回写**：adapter 在普通 Auto 源提及 HashMap/HashSet 时嫁接（用户自声明同名
  类型优先，同 List 旧糖遮蔽规则）；resolve 拦截构造（`HashMap.new()` 未声明时落到
  `__std_HashMap`）与方法拼写（insert_str/insert_int/get_str/get_int → put/get 族；
  `insert`/`get` 按**接收者局部变量的静态值面**分派，int 为默认面——legacy `<str,int>`
  单态语义；面冲突/未知面 = `source.map-face-mismatch`/`source.map-face-unknown`）。
- **已验键值域**：字符串键；值域 {int, str}（每变量单态面；Dyn 实参经 `.to(int)` 显式转换，
  直插具名拒 `source.type-mismatch`；错值型在记录方法准入的 Verify 阶段
  `verify.type-mismatch` 拒）。首批公共操作：new/insert_str/insert_int/get_str/get_int/
  insert/get/contains/remove/clear/size/drop。
- **容器合同**：put 命中即原地覆盖（size 不增）；remove 命中返回 true 并压缩并行表；
  缺键 get = `fail("KeyError")`（try-catch 后 owner 可续用）；错面 get = 静态
  `source.map-face-mismatch`（graft 内运行时 ValueError 为纵深防御）；`drop()` 为
  显式"使用结束"标记（单元 no-op），释放观察在作用域退出（同 List 行归一）。
- **展示兼容规则**：AV 文本面 print(bool)=true/false；f-string 插值拒 bool
  （`source.fstr-arg`）——旧 [1,0] 展示迁移用显式 bool→int shim；V-01 原 oracle
  逐字保留（`age=30 score=100 has=1 size=1` / `apple=1 cherry=0 size=1`）。
- **证据**：`av.parity.1121.map-set`（`experimental/ace/crates/av/tests/cases/plan1121_map_set.rs`，
  26 例：V-01 旧行逐行承接含两条旧删除红例的成功删除重归类、V-02 覆盖/去重/错型具名拒、
  V-03 缺键协议/owner 存活/释放对账/整数 List 与 Dyn 近邻不回归）；
  旧行映射 `crates/auto-lang/test/cases/language/migration/legacy-map.tsv`（PLAN-1121 行组）。
