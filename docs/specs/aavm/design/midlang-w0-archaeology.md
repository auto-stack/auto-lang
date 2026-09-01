# W0 考古规格:struct 类型 / 全局变量 / 补缺四件 / use 模块化(Plan 511)

基线 master b4c670643,2026-09-01 实测。方法:宿主源码四线考古(parser/codegen/
engine/loader)+ 最小样本真机反汇编(`test_aavm2_m4_rust_disasm_print` 诊断通道,
scratch 探针已删)。**本文件是 W1/W2/W3 实现的发射序规范**;与 m4-bytecode-format.md
(既有能力)互补,冲突处以本文件为准(仅限本文件覆盖的新能力面)。

核心原则(Plan 511 架构约束):宿主为规范,M4 判据 = 反汇编**文本**逐行相等
(不是字节相等)——宿主 disasm 的渲染怪癖(见 §1.3 nop 幽灵、§3.1)同样要镜像。

---

## §1 struct 类型(W1)

### 1.1 语法与 AST Display(M2 判据)

声明(字段=名字+类型,**无冒号**;成员以换行分隔):

```auto
type Point {
    x int
    y int
}
```

TypeDecl Display(`ast/types.rs:701`):

```
(type-decl (name Point) (members (member (name x) (type int)) (member (name y) (type int))))
```

- 空节一律省略(无 parent/has/delegations/methods 时不输出对应段);
- Member(`types.rs:769`):`(member (name x) (type int))`,带默认值时追加
  ` (value <expr>)`;字段类型 Display 即 `Type` 的 unique_name(`int`/`string`/
  `[]int` 等内建名;用户类型=类型名)。

构造字面量 = **Expr::Node**(不是 Object):`Point { x: 10, y: 20 }` →

```
(node (name Point) (args (pair (name x) 10) (pair (name y) 20)))
```

- 走 rhs/atom 路径(parser.rs:3637,Ident 后跟 `{` 且该名在 scope 中是类型);
  Pair 参数经 `parse_braced_struct_args` → `Arg::Pair`;
- Arg Display(`ast/call.rs:202`):`(pair (name x) 10)`;args 容器:
  `(args <arg> <arg> ...)`;node 无 id/体时省略对应段(`ast/node.rs:66`)。

字段访问 = Expr::Dot(Pratt 后缀,链式 `p.a.b` → `Dot(Dot(Ident p, a), b)`):

```
(dot p.x)
(dot (dot p.a).b)
```

(`ast.rs:493`:`(dot {object}.{field})`。)

### 1.2 构造发射序(M4,实测反汇编)

**定案:struct 构造走 NEW_INSTANCE(0xB0)+CONSTRUCT_INSTANCE(0xB1)族——与
enum 载荷实例同路径**,不走 CREATE_OBJ(0x2E,那是裸对象字面量 `{k: v}` 与
未注册类型 fallback 的路径)。aavm engine.at 已有前三(new.instance/
construct.instance/get.generic.field),W1 补 set.generic.field 即可。

实测(`mk` fn,`Point { x: x, y: y }`):

```text
000e  load.local 128        ; 字段值按字面量序依次入栈(这里=参数 x)
0010  load.local 129        ; 参数 y
0012  const.i32 5           ; mono_name 字节数("Point"=5)
0017  new.instance "Point"  ; 名字字节内联在 opcode 后(disasm 渲染为带引号名)
001d  const.i32 2           ; 字段数
0022  construct.instance    ; 栈序 [..., v1..vN, instance_id, field_count]
```

- **字段值按字面量顺序位置填充,Pair 命名不重排**(宿主无 name→index 映射,
  codegen.rs:5229-5257;语料全部用声明序字面量,不测乱序语义);
- 字段不足补类型默认值(EDGE-04-B,W1 语料不覆盖);
- 构造表达式结果 = instance_id 压回栈(engine 4109),let 直接 store。

### 1.3 字段读 / 写(M4,实测)

`type` 声明本身**零代码发射**(仅 register_type;TypeDecl 先于其他语句编译,
harness compile_and_link 的 partition 既有行为)。

字段读 `p.x`(**非泛型用户类型**走名字池;泛型才走 get.generic.field 下标):

```text
005d  load.loc.0 0          ; 对象先入栈
005e  get.field field[0]    ; 0x2D,操作数 = u32 字段名字符串池索引
```

字段写 `p.x = 11`(注意:**走 set.generic.field,不是 LOAD_STR+set.field**——
写路径按 `var_types[p] is Type::GenericInstance` 分派,而 `let p = Node{...}`
形态恒记为 GenericInstance(codegen.rs:12919),故构造字面量绑定的变量写字段
一律 SET_GENERIC_FIELD(codegen.rs:6324-6352);LOAD_STR+SET_FIELD 是 Call 形态
`Point(10,20)`(var_types=User)与其余 fallback 的路径,W1 语料不触发):

```text
004f  const.i32 11          ; RHS value 先入栈(栈底)
0054  load.loc.0 0          ; instance_id(栈顶)
0055  set.generic.field field=0   ; 0xB3 + u32 字段下标
0057  nop                   ← 幽灵 nop ×3(见下)
0058  nop
0059  nop
```

**幽灵 nop 注记**:宿主 disasm 对 set.generic.field 的操作数按 **u8** 读
(disasm.rs:391-394 stale),而实际编码是 u32——高 3 字节的零被逐字节渲染成
`nop`。M4 对拍的是 disasm 文本,aavm codegen_dump **必须镜像这个渲染**
(发射 u32 操作数 + dump 按 1 字节读再补 nop 行)。`get.field` 的 disasm 是
正确的 u32 读(`field[N]`),无此问题。

### 1.4 引擎语义(M5)

- 实例 = VInst(既有 enum 载荷同载体,arena 池);CONSTRUCT_INSTANCE 消费
  `[..., v1..vN, instance_id, field_count]`,按类型注册表字段名表落位;
- get.field 按字段**名**定位(host engine 5236:GenericInstanceData 按
  field_names.position);set.generic.field 按字段**下标**写;
- 值语义:字段读写对 VInt/VStr/VArr 均按值拷贝(镜像宿主 rc_push 拷贝语义,
  aavm 无 RC,直接拷)。

### 1.5 print 形态分裂(语料约束)

- `print(p)`(native 路径)→ `<obj:4000001>`(堆 id,**非确定**,语料禁止);
- f-string `f"$p"` / to-str 路径 → `Point { x: 10, y: 20 }`(字段串
  `name: val`,Str 带双引号,join ", ";engine.rs:3595 同款三处)——确定性,
  语料用它断言对象形态;
- 语料主线用 `print(p.x + p.y)` 类标量读,避免踩形态分裂与堆 id。

### 1.6 推断层(M3)

- 构造表达式(Node)推断 **Unknown**(host infer/expr.rs:256);
- 字段访问 Dot(receiver=User/GenericInstance)→ member.ty(按名查声明,
  infer/expr.rs:335-389);其余 Unknown;
- typeinfo.at 需 type 注册表(name→字段名/类型表)支撑 Dot 推断;
  M3 dump 输出对齐宿主推断层格式(corpus_m3 既有约定)。

---

## §2 全局变量(W2)

### 2.1 判定与遮蔽(代码定案)

- **仅顶层 `var`/`const`**(StoreKind Var|Const 且 scope_stack.len()<=1,
  codegen.rs:1665-1681)进 `global_vars`;`let` 恒局部("let intentionally
  stays local");fn 体内 var/let = 局部(scope>=2);
- 同名声明重复注册按名去重,`global_inits` 只记首次;
- **读写优先级不对称**(镜像!):读 局部>全局(lookup_var 先于 global,
  codegen.rs:5550-5555);**写 全局>局部**(global 检查在局部之前, 6189-6192);
- Store 声明无 scope 守卫:fn 体内 `var/let x` 若 x 已在 global_vars(不可能
  发生——注册只在顶层;跨 fn 引用才命中 global)。
- is-pattern 绑定、fn 参数:纯局部槽;读遮蔽全局、写仍走全局(同名参数
  "读参写全局"怪癖存在,W2 语料 b43 用 fn 内 let 遮蔽读路径,不测参数同名)。

### 2.2 发射序(M4,实测)

顶层 `var count int = 100`(wrapper 内,声明序):

```text
0005  const.i32 100
000a  store.global 0        ; 0xC6 + u32 名字池索引;声明无 DUP
```

fn 体内读 `count`:`load.global 0`(0xC5 + u32 名字池索引)。

fn 体内赋值 `count = count + n`(表达式语句):

```text
0018  load.global 0
001d  load.local 128         ; 参数 n
001f  add 
0020  dup                    ; 赋值保留值(表达式结果)
0021  store.global 0
0026  pop                    ; 语句级丢弃
```

**fn main 体首重放**(关键!):有 `fn main` 时入口直接 spawn main,wrapper
顶层不执行;codegen 在 **main 体首、参数注册之前**按 global_inits 记录序重放
`<init> + store.global`(codegen.rs:1339-1348)。实测 main 开头:

```text
0036  fn.prolog args=0, locals=6
0039  reserve 6
003b  const.i32 100          ← 重放
0040  store.global 0
0045  .line 9                ← 之后才到第一条语句
```

- 重放是编译到 fn main 那一刻的快照:**语料约定 fn main 声明在顶层 var 之后**
  (b42/b43 遵守;声明序敏感性登记为怪序注记,不做语料);
- 无 fn main 时 wrapper 顶层自然执行(既有 wrapper 语义)。

### 2.3 引擎语义(M5)

全局区 = 模块级单份、**按名字字符串索引**(非槽位);LOAD_GLOBAL 未命中
**缺省压 0**(i32 0);STORE_GLOBAL 弹值覆盖。aavm engine 增全局表
(name→Val)+ 两条分派即可。

---

## §3 补缺四件(W2)

### 3.1 for-in 数组/表达式(主通道,实测)

`for v in a { ... }`(range 为非 Range 非 Call 表达式——数组字面量/数组变量/
字段访问;codegen.rs:2830-2967):

```text
0076  load.loc.0 0           ; <迭代对象> 只求值一次
0077  dup 
0078  arr.len                ; 0x48,len 缓存(一次)
0079  const.0                ; 0x12 计数器初值(注意:const.0 非 const.i32!)
007a  store.local 2          ; _counter   ← 三隐藏槽按添加序
007c  store.local 3          ; _array_len
007e  store.local 4          ; _array_ref
0080  load.loc.2 2           ; loop: _counter(load 槽2 有快形式 loc.2)
0081  load.local 3           ; _array_len
0083  lt 
0084  jmp.z -> end
0087  load.local 4           ; _array_ref
0089  load.loc.2 2           ; _counter
008a  get.elem 
008b  store.local 5          ; 循环变量 v(普通局部槽,首轮即覆盖,无预 store)
008d  .line 13               ; ← 体语句 .line(for 行自身的 .line 在循环前)
      <body>
0097  load.loc.2 2           ; continue 目标 = 增量段
0098  const.i32 1 
009d  add 
009e  store.local 2 
00a0  jmp -> 0x0080          ; 回跳(不带 .line)
00a3  push.nil               ; end: 循环域弹出释放组(_counter/_array_len/
      store.local 2 ...         _array_ref/v 按槽释放组归一,既有规范化条款2)
```

- for 语句行 `.line` 在 `<迭代对象>` 之前发射(例中 `.line 12` 在 0076 前);
- 三隐藏槽 + 循环变量 = 循环作用域 4 个槽,按 `_counter,_array_len,_array_ref,v`
  顺序 add_var(槽号连续递增);break 跳 end、continue 跳增量段(既有帧机制);
- **与 for-in-range 的差异**(既有):range 无隐藏槽、循环变量即计数器、
  end 每轮重算;
- `.len()` 接收者跟踪缺口①顺收:数组变量(含 List 型 fn 参数经 var_types/
  参数类型注解推断)`.len()` → `<recv>; arr.len`(三分流主路径,codegen.rs:
  7460-7467;str.len 走 CALL_NAT 170 不在 aavm 范围)。

**返回数组的调用** `for x in f()`(range 为 Call 且非 values/keys,
codegen.rs:2747-2829)走**迭代器协议**:

```text
<compile f()>
store.local _iterator
loop: load.local _iterator; call.nat nat#112(auto.iterator.next)
      dup; const.i32 -1; eq; jmp.nz -> end     ; nil=-1 哨兵
      store.local x
      <body>
      jmp -> loop                              ; continue 也回 loop(重取 next)
end:
```

aavm 引擎侧 call.nat 112 语义 = 对数组值迭代(游标推进,耗尽返回 -1);
宿主 shim 对 List 句柄按元素序迭代(native.rs:3176)。values()/keys() 特例
(CALL_NAT 103)超 W2 范围,不镜像、不语料。

### 3.2 字符串下标(实测)

`s[1]` 与数组下标共用 get.elem;字符串分支按 **Unicode 码点**计数/索引,
**返回码点 i32**(engine.rs:4654-4681;负索引归一化 -1=末位;越界压 0):

```text
00c3  load.loc.2 2           ; s
00c4  const.i32 1 
00c9  get.elem               ; "ABC"[1] → 66
```

aavm 引擎 get.elem 增 str 分支(码点计数;.at 语料限 ASCII,码点=字节,
实现仍按码点语义写)。`s.len()`(字节长,CALL_NAT 170)与 ARRAY_LEN 对字符串
返回字节长(2895-2902)的分裂不入语料。

### 3.3 一元负号(实测)

`<操作数>; neg`(0x35,int 路径;float/double 变体 NEG_F/NEG_D 超 v2 无浮点,
不镜像);一元 `+` 为 no-op(不发射):

```text
00d0  load.loc.2 2
00d1  const.i32 1 
00d6  get.elem 
00d7  neg                    ; → -66
```

### 3.4 下标复合赋值——**宿主不支持**(决策 D1)

宿主 codegen 对非 Ident LHS 的复合赋值直接编译错误(codegen.rs:6154-6158):

```
Compound assignment requires a variable on left side
```

无发射序可镜像。**定案(D1,宿主为规范)**:aavm 同文本拒绝 `a[i] += e`;
语料处置:b41 不进 corpus_m4(红闸无法承载错误件),改为 L3 99_unit 断言
ev_run 返回上述错误文本;divergences 登记一条(aavm 亦不支持,与宿主一致,
非分叉,仅注记)。计划原文"复合化"作废,以本定案为准。

---

## §4 use 模块化(W3)

### 4.1 语句形态与 Display(M2 判据)

四形态(parser.rs:6122-6165 parse_use_items + 6453 use_stmt):

| 形态 | 解析 | Display |
|---|---|---|
| `use db` | items=[], wildcard=false | `(use (module_path db))` |
| `use db: create, remove` | items=[create,remove] | `(use (module_path db) (items create,remove))` |
| `use db::{a, b}` | LBrace 分支(items=[a,b]) | 同上形态(items a,b) |
| `use db: *` | wildcard=true | `(use (module_path db) (wildcard))` |

- Display(use_.rs:29-54):`(use` + ` (kind c|rust|py)`(Auto 无 kind 段)+
  ` (module_path <点路径>)` + ` (items <逗号join>)` + ` (pub)` + ` (wildcard)` + `)`;
- 点路径多段:`use auto.greet_mod` → `(use (module_path auto.greet_mod))`;
- **宿主双解析器分歧注记**:宿主轻量扫描器(scan_use_statements)对 `{}`
  形态产脏 items——aavm 不复刻扫描器(aavm 单解析器),以 parser 侧为准,
  该分歧不影响 M2 dump 与链接(宿主链接靠 parser 侧 import_scope);
- 异构导入拒绝文本(错误通道基准):
  - `use.rs <crate>::...` 未声明 dep:`Crate '{crate}' not declared. Add `dep {crate}` before `use.rs`.`
  - `use.py`(无 feature):`Python FFI not enabled. Rebuild with `--features python` to use `use.py`.`
  - `use.c` 语法:`Expected <lib> or "lib", got {kind:?}, {text}`
  - `use mod.rs`(点路径形态)**不专门拒绝**,落 Module not found(见 4.2)。

### 4.2 模块解析(file shim)

- 模块路径 `.`→`/`(compile.rs:1188);**四级搜索**:
  ① CWD 相对 `<path>.at|.au|.auto` → ② stdlib(剥 `auto/` 前缀)→
  ③ `<path>/mod.at` → ④ 各 source_dir(主文件目录 + 每个已加载模块把自身
  目录 canonicalize 追加 → 传递依赖兄弟可达);
- aavm W3 判据只锁 **同目录相对解析**(corpus_use 主文件与模块同目录,
  source_dir=主文件目录;等价④);CWD/stdlib 路径不做语料;
- `auto.file.read_text`(native 1000)为 aavm 侧读取原语;
- 未找到:`Module not found: {module} (module_path={module_path}, parent_dirs=[])`
  (parent_dirs 为 Rust Debug;super 前缀路径才有非空,corpus 不用 super,
  固定空列表渲染);
- **环检测 = 宿主不报错**(决策 D2):loading_stack 命中即静默跳过
  (Plan 317 合法环支持,compile.rs:1135-1139;历史报错文本已移除,
  现测试断言 is_ok)。aavm 镜像 = 同语义跳过;corpus_use 的"环"用例改判
  **合法环可用**(A use B + B use A 互相调用),错误通道不含环;
- 去重:按模块名(compiled_modules)+ canonical path 双重去重。

### 4.3 可见性(决策 D3)

**宿主 VM 链路不过滤 pub**——codegen 导出无 is_pub 检查(codegen.rs:1305-1309),
链接器同样全量注册;pub 语义只在 a2r 转译器生效。**aavm 镜像 = 所有 fn 一律
导出**(pub 解析并 Display,但不影响导出面);计划原文"非 pub 不导出"作废,
以宿主行为为准。corpus_use 含一个非 pub fn 跨模块调用用例钉住该语义。

### 4.4 多编译单元与链接器

- 每模块一个编译单元(Module:code/exports/relocs/strings/...);dep 模块编译
  序:TypeDecl/EnumDecl 先 → 顶层 Store(var 初始化,声明序)→ Fn 声明 → HALT
  (compile.rs:1652-1683);模块名 = 文件 stem;
- 链接:**依赖模块在前、主模块最后**;fn 间 JMP-over 既有结构不变;
- 符号表:先注册裸名,**重名改 `{mod}#{name}` 限定名**(loader.rs:264-272);
  CALL reloc 符号解析三级:精确名 → 含点时 `mod#name` → 剥前缀裸名
  (loader.rs:286-312);失败 `Undefined symbol: {sym} in module {mod}`;
- 跨模块调用发射:源面 `mod.fn(...)` → CALL + reloc 符号 `mod.fn`;
  `use mod: item` 定向导入名注入作用域(import_scope[item]="mod.item",
  读发射 LOAD_GLOBAL 限定名 / 调用解析 mod#item);
- **模块初始化序**:链接后各 dep 模块**按布局顺序逐个** spawn 执行顶层
  (Store 段)到 HALT,先于主入口(lib.rs:1317-1333);主模块顶层(wrapper)
  仅在无 fn main 时执行(§2.2 重放互补);
- **全局区合并**:单一名字键扁平表;dep 模块全局键带 `mod.` 前缀
  (`db.notes`),主模块裸名(current_module="" 时);跨模块读导入常量即
  LOAD_GLOBAL `mod.name`;
- 字符串池合并:dep 序拼接去重 + 操作数索引重映射(确定性,Vec 序);
  M4 多文件对拍无需新增规范化条款(exports HashMap 迭代序只影响符号表
  内容,不影响 code 布局与 disasm 文本;relocs 是 Vec)。

### 4.5 aavm 侧结构(W3 实现锚点)

- parser.at:use 四形态 + Display;异构 use.rs/py/c 拒绝文本;
- 模块解析器(parser/codegen 间函数群):read_text + 递归 + 环跳过 + 去重;
- codegen.at:per-module CG 实例 + `mod#fn` 限定符号 + import_scope 注入;
- 链接器:缺省 engine.at 新段(`ev_link`:拼装/reloc 回填/初始化序执行/
  全局区);超 ~300 行或需独立测试面则升第七文件(步骤 17 决策点,缺省段);
- engine.at:`ev_run_files(main_path)` 入口(保留 ev_run 单源兼容)。

---

## §5 L3 决策:lib 符号引用方案(探针实证)

- ✅ `auto test -d <file|dir>` 可用:`#[test] fn` 在隔离 VM task 执行,
  裸 `assert_eq(a, b)`(native 5;**不带 auto. 前缀**,带前缀编译失败);
  字符串字面量支持 `\n` 转义(多行程序源可用单行串内嵌);
- ❌ `use auto.lib.engine: ev_run` 不可行:① lib 六文件互相依赖且自身无
  use 语句,单模块加载必炸(engine.at 引用 token.at 符号);② test_code
  的 session 不播种源目录(CWD/stdlib 之外不可达);③ ev_run 等入口非 pub;
- **定案:聚合生成方案**——`test/vm/aavm2/99_unit/` 的用例以「lib 前置拼接 +
  #[test] 体」生成单文件(生成脚本 + 生成物均入库,镜像 Rust 侧 harness 的
  AUTO_LIB_FILES_V2 前置拼接先例);`auto test -d test/vm/aavm2/99_unit` 直跑;
- ev_run 返回格式:**行间 `\n` 连接、无尾随换行**(print 拼接首个不前置换行);
- 观测项(不修):assert_eq 失败信息显示乱序大数(shim 消息格式化瑕疵,
  pass/fail 判定本身正确)。

---

## §6 怪序与决策记录汇总

| # | 事项 | 定案 |
|---|---|---|
| Q1 | set.generic.field disasm 幽灵 nop(u32 操作数被 u8 读) | 镜像宿主 disasm 渲染(§1.3) |
| Q2 | 字段写走 set.generic.field(var_types=GenericInstance)而非 set.field | 镜像(§1.3);LOAD_STR+set.field 路径 W1 不触发不实现 |
| Q3 | for-in 计数器初值用 const.0(非 const.i32 0) | 镜像(§3.1) |
| Q4 | 全局 fn main 体首重放(参数注册前)+ 声明序敏感 | 镜像;语料约定 main 后置(§2.2) |
| Q5 | 读写优先级不对称(读局部>全局,写全局>局部) | 镜像(§2.1) |
| D1 | `a[i] += e` 宿主编译错误 | aavm 同文本拒绝;b41 语料改 L3(§3.4) |
| D2 | 循环依赖宿主静默跳过(不报错) | 镜像;corpus_use 环用例=合法环(§4.2) |
| D3 | pub 在 VM 链路不参与导出过滤 | 镜像:全 fn 导出(§4.3) |
| D4 | `use mod.rs` 无专门拒绝,落 Module not found | 镜像(§4.1/4.2);错误通道用 use.rs/py/c 三文本 |
| D5 | L3 lib 引用 | 聚合生成方案(§5) |

W1 语料红线:print(struct) 禁用(堆 id);字段字面量声明序;fn main 后置
(全局语料);避免裸 bool print(P474 旁支不扩面)。
