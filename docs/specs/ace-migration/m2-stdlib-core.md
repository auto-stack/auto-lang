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
