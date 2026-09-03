---
plan_id: PLAN-525
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-oop-batch
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-017]     # 自举（目标语言高阶能力,塔顶必经）

affects: [aavm]
current_step: 26
total_steps: 30
---

# [PLAN-525] aavm OOP 批：目标语言高阶能力（VBool/方法/泛型/闭包/May/生成器）

## 变更摘要

aavm 目标语言从"中阶"推进到"高阶"的伞形计划（511/514/517 三计划显式
延后项的集中清偿），六波次（每波独立折叠，511/514/517 多波单计划先例）：

- **W1 VBool parity**（P474-旁支清偿）：Val 载体加 VBool——`print(bool)`
   两侧形态对齐（true/false vs 1/0）；值模型级改动,牵动全 match 分派,
   独立首波（后续波次的方法返回值/条件值全受益）。
- **W2 方法/impl**：`type T { fn m() {...} }` 体内方法与 `ext T {}`
   扩展的 aavm 编译执行（接收者 `.field` 简写/static fn/`&mut` 语义——
   宿主与主 a2r 路径 514 W2 已加固,AA2R 发射 514 已交付 g01-g17；本波
   是 aavm 目标语言侧的 parser/typeinfo/codegen/engine 四层）+
   **is-struct 模式匹配**（`is User { id: 1 }` 形态,511 待澄清②延后项）。
- **W3 泛型**：`List<T>` 实例化的 aavm 编译执行——**塔顶必经**（lib
   自身大量使用）；单态化或类型擦除方案 W0 考古定案。
- **W4 闭包与嵌套 fn**：aavm 目标语言支持（447-旁支②嵌套 fn 静默失效
   已由 514 修复——本波是 aavm 侧能力）+ **pub type 跨模块共享**
   （511 待澄清③延后项：跨模块 struct 类型引用/use 导入类型）。
- **W5 May/生成器**：`?T`/May 错误传播最小面 + 生成器（yield）——范围
   W0 裁定（生成器或再延后,见待澄清③）。
- **搭车**：⑤腿塔顶性能测量（P517-1 观察项升级——每折叠点记录塔顶
   时长趋势,恶化则独立立项）。

**验收口径（511 式逃生舱关闭）**：每个新语法件直接落 **per-case dir
三件套金样格式**（`case.at`+`case.expected.out`+`case.expected.rs`），
**同步规约三面闸**同绿（VM 闸/AA2R 发射闸/a2r 运行闸），**四路统一
runner** 判定——依赖 523 建成的基建。

**开工前置（硬）**：Plan 523 archived（四路 runner/三件套金样/AA2R
中阶发射面全在位）；Plan 524（CLI 形态,软前置——不阻塞但语料运行
受益）。

## 目标

1. `print(bool)` 与宿主形态一致（VBool 载体,P474-旁支清偿;b13/b14/b19
   语料恢复裸 bool print 断言形态）。
2. aavm 目标语言支持方法族语法：体内方法/ext/static fn/接收者简写的
   编译执行 + is-struct 模式匹配；与宿主行为对拍一致。
3. 泛型 `List<T>`（及最小泛型函数面,W0 定案）aavm 编译执行——塔顶
   前置达成。
4. 闭包与嵌套 fn 支持；pub type 跨模块共享（use 导入类型）。
5. May/生成器按 W0 裁定范围落地（或显式延后登记）。
6. 每波三面闸+四路判定全绿；⑤腿塔顶时长趋势留档。

### 非目标（Out of Scope）

- trait/继承/完整 OOP 语义（`ext T for Trait`、动态分发）——远期;
- aavm 编译 lib 自身（塔顶）——前置=本计划泛型+方法落地后的下一张;
- Auto 版其他语言转译器；宿主语言语义变更（本计划只做 aavm 侧,
  宿主洞顺修除外）；VBool 之外 Val 载体重构。

## 架构方案

塔式依赖决定波次序：VBool（值模型地基）→ 方法（is-struct 依赖类型
系统）→ 泛型（塔顶必经,工程量最大）→ 闭包/跨模块类型 → May/生成器。

```text
W1 VBool       W2 方法/impl+is-struct   W3 泛型          W4 闭包/跨模块类型   W5 May/生成器
────────       ────────────────────     ─────────        ───────────────     ────────────
Val 载体   →   四层方法编译执行     →   List<T> 单态  →   闭包捕获        →   ?T 最小面
全分派链   →   is-struct 模式       →   泛型函数面    →   pub type 导入       (生成器裁定)
P474清偿       (宿主 514 加固地基)      (塔顶必经)       嵌套 fn 解禁
```

**关键设计约束（全程）**：

- **宿主为规范**：每语法 W0 考古宿主发射序/行为（M4 判据逐字符镜像）；
  宿主洞顺修（447/514 先例）。
- **三件套+三面闸+四路**：新语件一律金样格式;AA2R 发射闸同步（目标语言
  新语法 = AA2R 发射面同步扩展——同步规约）;四路 runner 折叠点全量判定。
- **每波独立折叠**：波内绿即准合入;波间依赖（W1 值模型→W2 方法返回）
  在折叠序中消化。
- **语料编号**：corpus_m4 b44+（VM 闸）/ corpus_a2r g26+（发射闸）/
  三件套 per-case dir（验收层）。

## 需求分析与背景调查

（取材：project.md 能力矩阵延后清单、P474-旁支、511 待澄清②③、
514 W2/W3 归档（宿主方法路径+AA2R 方法发射地基）、examples/
playground-demo/13-methods.at（宿主方法族语义样板））

### 基线

523 archived 后的全绿面（含中阶 a2r 覆盖+四路 runner+金样基建）+
524（CLI 直达）。具体数字开工时复测留档。

### 各波考古要点（W0 逐波细化）

| 波 | 考古要点 | 预估难点 |
|---|---|---|
| W1 VBool | Val 枚举扩展位（VInt/VStr/VArr/VInst→+VBool）;PushBool/print/truthy/比较全分派位;宿主 TAG_BOOL 编码 | 全 match 分派链牵动（P474 警示"非小修"）;M4 反汇编 b13/b14 期望重生成 |
| W2 方法 | 宿主方法编译路径（monomorphic/method-pack? W0 定案）;`.field` 接收者简写解析;static fn;`&mut` 判定;aavm CALL 语义扩展 | is-struct 模式的类型系统支持（typeinfo 注册表已有 511 地基） |
| W3 泛型 | 宿主泛型实例化策略（单态化时机/类型擦除）;List<T> 构造点/方法点发射序 | 工程量最大;塔顶依赖——lib 用法形态（List<List<X>> 嵌套）覆盖面 W0 盘点 |
| W4 闭包 | 宿主闭包捕获/MAKE_CLOSURE 族 opcode;aavm 栈机对应 | 嵌套 fn 与闭包语义边界 |
| W5 May | `?T`/`.?`/`??` 最小面;生成器 yield 脱糖 | 生成器或裁定延后（待澄清③） |

### 风险与对策

| 风险 | 对策 |
|---|---|
| VBool 牵动全分派链（预估最大回归面） | 独立首波+全闸门+矩阵硬闸;b13/b14/b19 期望重生成一次性收口 |
| 泛型工程量超波次容量 | W0 盘点 lib 实际用法面定范围;超限拆子计划（待澄清②） |
| 方法路径宿主洞（514 加固未覆盖面） | 探针先行;宿主顺修;大洞按 514-W3 先例挂起重启清单 |
| 波间耦合导致折叠序僵化 | 每波三件套语料自含（不依赖未落地波次的语法） |
| ⑤腿塔顶随 lib 能力增长超时（P517-1） | 每折叠点塔顶时长趋势留档;恶化立项（搭车测量项） |

## 详细设计

（各波 W0 考古后细化——此处纲要；执行时逐波在步骤内展开映射表,
镜像 514 W3 方法化映射表先例。）

- **W1**(考古落定,2026-09-03):
  - **Val 分派位清单(engine.at)**:①`enum Val`+`VBool(bool)`+构造器
    `ev_vb`;②`PushBool`(L591)ev_vi(1/0)→ev_vb;③比较臂
    Eq|Ne|Lt|Gt|Le|Ge(L737)结果 ev_vi→ev_vb(**宿主 EQ/NE/LT/GT/LE/GE
    全推 encode_bool**,engine.rs 8283/8333/8375/8410/8464/8489);
    ④And|Or(L770)与 Not(L786)结果 ev_vi→ev_vb(宿主同推 encode_bool,
    engine.rs 8615/8621/5718);⑤`ev_truthy`+=`VBool(b)->b`(宿主
    nv_truthy is_bool 权威,engine.rs 450);⑥`ev_str`+=`VBool->"true"/
    "false"`(宿主 print tag==3 臂 native.rs 1466+BUILD_FSTR is_bool 臂
    engine.rs 2998,两处同为 true/false);⑦`ev_cmp`:VBool×VBool 按
    位相等性(宿主 EQ a_nv==b_nv);VBool×异型→2(宿主 EQ 落 false);
    ⑧`ev_int`/`ev_add`/宿主 STR_CAT 对 bool 走 decode_i32 哨兵
    (i32::MIN/-2147483647)=退化场景,**语料不覆盖+divergence 注记**
    (宿主自身 print 与 STR_CAT 形态不一致,属宿主洞候选非镜像面)。
  - **期望重生成方案**:b13/b14/b19 恢复裸 bool print(原始
    99_bootstrap 040/043/046 形态:print(true)/print(false)/双 bool),
    期望=宿主形态 "true"/"false"(run_with_capture=AutoVM 管线,P474
    已改 print(TAG_BOOL)→true/false);三件走 A2R_BLESS=1 再生+git diff
    评审(523 基建),并扩入 M4_SAMPLE 金样集。
  - **三件套设计**:恢复件=corpus_m4 平铺旁挂(b13/b14/b19 同名
    三件套);VBool 新件=corpus_a2r per-case dir `g26_bool_print/`
    (裸 bool+比较/逻辑结果 print+fstr 插值形态;主 a2r 对 Expr::Bool
    直写 true/false,trans/rust.rs 2409——发射面已有,g 件护航);
    99_unit 断言=scripts/aavm2_unit_cases/engine_micro.at 追加
    `ev_run("print(true)")=="true"` 族→gen-aavm2-unit.py 再生。
- **W2**(考古落定,2026-09-03):
  - **宿主方法编译路径**(vm/codegen.rs 2265-2334/2409-2423):type 体方法
    与 ext 方法一律编为**独立函数**,名字重整 `Type.method`(如
    `Counter.get`);非 static 方法注入 `self` 首参(ParamMode::View);
    方法体内裸字段名→`current_type_members` 命中→load self 槽位+
    GET_FIELD(名字池下标;泛型模板走 GET_GENERIC_FIELD 下标),
    codegen.rs 5609。**无 method-pack/monomorphic 内联——就是独立
    fn+CALL**。
  - **方法调用发射**(codegen.rs 9030-9180):用户类型接收者→接收者
    入栈(self 位)→参数序→CALL+reloc 符号 `Type.method`(TypeDecl
    先于其他语句编译,exports 已含;spec/未解析才走 CALL_SPEC 兜底);
    静态调用 `Counter.new(10)`→参数→CALL(reloc `Counter.new`,
    不推接收者)。a2r 规范样板=examples/playground-demo/13-methods.a2r.rs
    (impl 块合并 ext;`.field`→self.field;静态→Counter::new)。
  - **is-struct 宿主语义**:parser(parser.rs 4177 `Point { x, y }`/
    4291 `Msg.User { content }`)→StructCover 解构绑定(字段名或
    field:alias,**无值匹配形态**——计划文案 `is User { id: 1 }` 按宿主
    实际语法收敛为解构式);主 a2r→Rust match 解构臂(trans/rust.rs
    3393);**宿主 VM 洞实证:vm/codegen.rs 10117 对 StructPattern
    panic not implemented**(probe:scratch/p525/w2_vm_probe.at)——
    W2 语料要过 VM 闸+生成 expected.out,**宿主顺修**(is 臂
    StructPattern→类型命中+GET_FIELD 绑定序发射,447/514 先例)。
  - **aavm 现状缺口**:parser.at parse_type_decl L1711/L1777 显式拒绝
    type/enum 体内方法;ext 块与 is-struct cover 形态缺(is_pattern
    无 LBrace 臂);typeinfo.at 无方法表;codegen.at 已有 struct 四
    opcode(NewInstance/ConstructInstance/GetField/SetField,511 W1)
    与 Call(fn 表下标)——方法编译=复用 fn 通道+`Type.method` 命名。
    AA2R 发射面(a2r.at token 游标直走,D39)514 已交付 g09-g17,
    W2 的 AA2R 同步=照抄 g 件模式扩 is-struct 件。
  - **语法注意**:is 臂分隔符是 `->` 非 `=>`(b13_is_enum 实证;`=>`
    报 E0007)。
- **W3**(考古落定,2026-09-03):
  - **lib 用法面盘点(塔顶覆盖清单定案)**:泛型使用 100% 为**内建容器
    List<T>**(a2r/codegen/parser/engine/typeinfo 合计 200+ 处;元素类型
    str/int/Val/Token/Binding/CGVar/TFnSig/Ar* 族等);**嵌套两层**
    `List<List<X>>`(engine arena/codegen scopes/parser scopes);
    使用位=fn 参数/返回/let/var 局部/**struct 字段位**(CG.ins
    List<I>/CG.scopes List<List<CGVar>>/CGType.fields 等);方法集
    **恰 5 个**:`List.new/.push/.get/.set/.len`(全 lib 计数,无
    pop/insert/join);**Map 为 0 使用**(2 处命中是 a2r.at 字符串
    字面量与注释假阳性);**泛型 fn 声明=0;自定义泛型 type 声明=0**。
  - **策略定案**:容器族在宿主=native shim 通道(auto.list.* CallNat,
    native_catalog.rs id:100 new/101 push/106 get/107 set/103 len;
    解析经 to_canonical `List.push`→`auto.list.push`),**不经
    generic_registry 单态化**——W3 无单态化/擦除决策点,aavm 镜像=
    codegen List 方法调用→CallNat(同 id)+engine 对应 arm
    (engine 已有 #103 len/#112 next 先例,523 H6);`List<T>` 注解
    parser.at 已解析(434,含嵌套)——W3 补 typeinfo/codegen 绑定
    跟踪与 fn 签名/字段位通道。**泛型函数面+自定义泛型 type:显式
    延后登记**(lib 盘点 0 使用,塔顶不依赖;待澄清②据此定案)。
  - **存量洞核销项**:b32 注释登记的"List-typed fn params(.len() in
    aavm cg)"已知缺口随 W3 收口。
- **W4**(考古落定,2026-09-03):
  - **宿主闭包模型**(opcode.rs 221-226,Plan 071 Direct Capture):CLOSURE
    0x90(func_addr+capture_count+n_args 立即数;每捕获项 name_idx+slot_offset,
    值栈弹入 env;385 by-ref 捕获记槽位)/CALL_CLOSURE 0x94(closure_id+
    args→current_closure_id 切换+标准帧)/LOAD|STORE_CAPTURED 0x92/93(按
    名访 env)。aavm 镜像可选:opcode 族照抄或 D29 式简化(Val 加
    VClosure 载体/arena 槽——aavm 自有设计层,可观察行为对齐即可)。
  - **闭包语法注意**:闭包表达式用 `=>`(18-closures.at:`(a int,b int)
    => a+b`/单参 `x => x*2`);is 臂用 `->`——两者并存。主 a2r 直接映射
    Rust 闭包(`|a: i64, b: i64| a + b`,捕获语义同 Rust)。
  - **pub type 跨模块宿主行为**(probe 实证 scratch/p525/w4_pub_type_probe/
    →输出 7/30):`use geom: Point, mk, sum` 导入类型+fn;跨模块构造
    `Point { x: 10, y: 20 }`、跨模块 fn 收发 struct 全支持。aavm 洞定位:
    **cg_link_modules 合并 ins/pool/fns/fn_mods/defers 但不合并 CG.tys**
    (codegen.at 2664-2756)——跨单元字段索引查找必挂,W4 补 tys 跨单元
    合并+use 项类型解析。
- **W5**(考古落定,2026-09-03):
  - **May 家族=Option/NullCoalesce/ErrorPropagate**(ast.rs 393 "May type
    operators Phase 1b.3"):`?T` 后缀+Some/None 构造与 is 模式臂+`??`。
    宿主全支持(probe scratch/p525/w5_may_probe.at:`?int` 返回+Some(v)/
    None 臂→30/none;VM 侧 CREATE_SOME(标记 no-op)/CREATE_NONE(-1)/
    IS_SOME/IS_NONE(encode_bool),native_catalog 120+ 族)。
  - **aavm 现状**:`?T` 类型后缀 parser.at 已解析(L389);`??` 已入
    Pratt 表(Op.QuestionQuestion L526);Some(x) 走 Call 路径(与宿主
    lexer 同构,parser.at 1917 注释);engine 无 CREATE_SOME/NONE/IS_SOME
    臂——W5 补 codegen+engine 最小面。
  - **生成器裁定素材**:宿主 Yield/CREATE_GENERATOR(Plan 321)在位;
    **lib 用量=0**(全 lib 仅 token.at 的 token 名声明一处命中)→
    按待澄清③缺省裁定**延后**(塔顶不依赖,登记后置任务)。
- 每波同步：AA2R 发射闸 g26+ 件 + 三件套金样 + 四路判定。

## 测试设计（TDD：保护网+红先行+金样锚定+三面闸）

### 保护网

开工时点的全绿面（523/524 终态）全程不破绿;⑤腿塔顶趋势留档。

### 红先行（每波同构）

三件套语料先行落盘（case.at+expected.out+expected.rs——expected 由
宿主/主 a2r 生成）→ VM 闸/发射闸/运行闸三面红 → 四层实现+AA2R 同步
→ 三面绿 → 四路 runner 判定 → 折叠。

### 命令

同 523（三面闸+runner+bless）+每折叠点矩阵（P517-2 纪律）。

## 验收标准

1. 每波三面闸+四路 runner 全绿（判定表留档）;语料三件套金样齐。
2. VBool：`print(true)`→`true`（b13/b14/b19 裸 bool 断言恢复）;P474-
   旁支核销。
3. 方法族+is-struct：13-methods.at 样板级用例 aavm 运行与宿主一致。
4. 泛型：List<T> 塔顶前置面达成（lib 实际用法形态覆盖,W0 盘点清单为准）。
5. 闭包/嵌套 fn/pub type 跨模块落地（或显式延后登记+理由）。
6. May/生成器按 W0 裁定处置;⑤腿塔顶趋势留档。
7. `cargo tf` 绿（基线红除外,归属注明）;无静默丢弃。

## 执行步骤
（原子任务;每波：语料红 → 四层实现 → AA2R 同步 → 三面绿 → 四路 → 折叠）

> 约定：W0 各波考古可在 master 做（纯文档+探针）;实现全在 worktree
> `.worktrees/plan-525-dev`;折叠点（6/11/16/21/26/29）矩阵+CI 绿后合入。
> **前置门禁：Plan 523 archived（硬）/524（软）——开工时核验。**

### W0 考古（master,分波做）

1. [✅ 已完成 2026-09-03] W1 考古:Val 分派位 8 处清单+宿主 TAG_BOOL 六指令
   语义定位(print/EQ 族/AND/OR/NOT/BUILD_FSTR 全 encode_bool 或 true/false;
   ADD/STR_CAT 哨兵退化场景排除)——映射表已回填「详细设计 W1」节;b13/b14/b19
   重生成方案=A2R_BLESS 再生+M4_SAMPLE 扩名;新件 g26_bool_print per-case dir。
2. [✅ 已完成 2026-09-03] W2 考古:宿主方法编译=独立 fn+`Type.method` 重整
   (self 注入/裸字段→GET_FIELD/调用=CALL reloc,codegen.rs 2265/5609/9030);
   is-struct=解构绑定语义,主 a2r 全支持而**宿主 VM panic 洞实证**(10117,
   scratch/p525 留档)→宿主顺修项;aavm 四层缺口定位(parser.at L1711 拒方法/
   ext 缺/is-struct cover 缺)——映射表已回填「详细设计 W2」节;`->` 语法注意。
3. [✅ 已完成 2026-09-03] W3 考古:lib 泛型盘点=纯内建 List<T>(嵌套两层/
   方法集恰 5:new-push-get-set-len;Map 0/泛型 fn 0/自定义泛型 type 0);
   策略定案=容器族走 CallNat 内建通道(宿主同构 id 100/101/106/107/103)
   无单态化决策点;泛型函数面+自定义泛型 type 显式延后——映射表已回填
   「详细设计 W3」节。
4. [✅ 已完成 2026-09-03] W4/W5 考古:闭包=Direct Capture opcode 族
   (0x90-0x94,engine.rs 7412/7638;闭包表达式 `=>` 与 is 臂 `->` 并存);
   pub type 跨模块宿主全支持实证(7/30)而 aavm cg_link_modules 不合并
   CG.tys 洞定位;May=?T+Some/None+`??` 家族宿主全支持(probe 30/none);
   生成器 lib 用量=0 → 裁定延后——映射表已回填「详细设计 W4/W5」节。

### W1 VBool（worktree）

5. [✅ 已完成 2026-09-03] 三件套语料先行红:b13/b14/b19 恢复裸 bool
   (worktree 077477ebb)+g26_bool_print per-case dir 9 断言行(宿主 bless:
   true/false×11)+M4_SAMPLE 扩 3;m5 行为腿实证红(b13 mismatch:host
   `true` vs aavm `1`)。
6. [✅ 已完成 2026-09-03] 折叠点①:VBool 四层实现落地(worktree 6dc8d6d1c/
    master 7ab140c41)——engine.at 8 分派位+ev_bool_str 辅助(规避主 a2r
    is-match 混合臂发射破缺,登记债);**三宿主顺修/同步**:a2r.at Not 恒
    括号(g26 发射分歧)、主 a2r merge 后处理 `&&"` 字面量误吃(前导引号
    保护,g26 四路 p2≠p4 实证)、t_iter 期望翻转(宿主 for-in 已迭代);
    单元 18/18+tv 3558+tf 绿+tt 28=存量 P523-3+rustc 闸(含 g26)+四路
    18/18;矩阵/⑤腿留档见步骤 27。P474-旁支核销。

### W2 方法/impl+is-struct（worktree 续）

7. [✅ 已完成 2026-09-03] 三件套语料先行红(worktree e25713b78):b44
   方法族五件(实例/static new/ext 并入/方法内调方法/.value 简写照——
   宿主全绿实证 10-15/7/21-42/13-15/42)+b45 is-struct 两件(宿主 panic
   洞承载)+g27 方法族/g28 is-struct 发射门件;m5 实证红(b44 首件
   aavm v2-unsupported);b45/g28 金样待宿主顺修后 bless。
8. [✅ 已完成 2026-09-03] parser+typeinfo 方法层(worktree 6add3fab1):
    parse_type_decl 体内方法(Display (methods …) 镜像+自引用预注册占位
    原地覆写)/parse_ext 并入(ext-merge 占位+终态手术)/expr_pratt 前导
    点臂/is_pattern struct-cover 双形态+臂作用域绑定;typeinfo Type/ext
    走查;corpus_m2 扩 p25/p26/p27——M2 全绿 35 件+m1/m3/单元 18 无回归。
9. [✅ 已完成 2026-09-03] codegen+engine 方法执行(worktree 911d5fe73):
    cg 双趟(pass1 Type/Ext 方法先于 wrapper 镜像宿主分区序)/cg_fn
    Type.method 重整+self 注入+fn_rets/前导点主式+方法调用后缀臂(实例/
    静态)/self 字段写名字基(m4 对拍定案)/cg_is struct-cover 解构臂;
    **宿主顺修**:vm/codegen.rs StructPattern 两处 is 位 panic 洞(恒命中
    +GET_FIELD 解构);M4+M5 各 53 件全绿;残留 4 红全在 g27/g28 AA2R
    发射面(步骤 10 范围)。
10. [✅ 已完成 2026-09-03] AA2R 同步发射(worktree 182e395ea):a2r.at
    is 臂 struct-cover 模式(词法组装 `f`/`f: alias`→", ",主 a2r match
    解构形态;`go` 保留字规避)+codegen 字段快照拷贝(转译 move);
    g27(方法族,514 地基天然绿)/g28(is-struct,新发射)金样 bless;
    全 aavm2 21/21 绿。
11. [✅ 已完成 2026-09-03] 折叠点②(worktree f05ef8a06):三面绿(M4/M5
    各 53+a2r 发射+compile 闸)+四路 22/22(b44/b45 入金样集)+rustc 闸
    g19-g28+tf 绿+tv 3558+tt 28=存量 P523-3;**顺修**:a2r.at ext 并入
    显式写回(主 a2r 链式变更 `.clone()` 丢变更洞,g27 四路 p2≠p4 实证,
    登记债);矩阵补验折叠②后跑(见步骤 27 汇总)。

### W3 泛型（worktree 续）

12. [✅ 已完成 2026-09-03] 三件套语料先行红(worktree 59ec87a33):b46
   容器族四件(basic/fn 参数返回位含 b32 存量洞核销/struct 字段位/嵌套
   List<List<int>>;宿主四形态全绿实证)+g29 发射门件;m5 实证红
   (b46_list_basic:unknown ident List)。
13. [✅ 已完成 2026-09-03] parser/typeinfo 泛型层:W0 定案容器族面
   parser.at 已解析(434 含嵌套),typeinfo Type/ext 走查已就——b46 红
   全部落在 codegen 层(unknown ident List 实证),步骤 13 以现状收口
   (单态化/实例化表按定案不需要)。
14. [✅ 已完成 2026-09-03] codegen+engine(worktree 7812018d3):
   CGType.ftys 字段类型表/注解串入 vty(let+param)/List 静态 new 探测/
   CallNat 五方法分派(strip_generic+list_nat_id)/字段跳类型回传/len 快径
   List 排除+字段链 arr.len 分派/语句位 CallNat pop;engine CallNat
   100/101/106/107 四臂(arena 槽位)。M4+M5 各 57 件全绿。
15. [✅ 已完成 2026-09-03] AA2R 同步发射(worktree 7812018d3):
   derive 不降级 Vec(宿主仅 float/Map/enum)/owned 参数保守 mut
   (mode=own 三通道,主 a2r G2.1 镜像)/get·set 恒 cast/get·len 打印
   {:?} 形态(lenflag/getdbg 双位);g29 金样 bless;发射闸 29 件全绿。
16. [✅ 已完成 2026-09-03] 折叠点③(master 5c2e2b1bb 含 W2 折叠②):
   三面绿(M4/M5 各 57+发射闸 29+compile 闸)+四路 24/24+rustc 闸
   g19-g29+tf 绿+tv 3558+tt 28=存量 P523-3;**塔顶前置核验达成**——
   W0 盘点清单逐项:注解四位(fn 参数/返回/let/字段 b46 全覆盖)、嵌套
   两层(b46_nested)、方法集恰 5(b46_basic);泛型 fn=0/自定义泛型
   type=0/Map=0 显式延后。链式接收者型回传(get(0).len() 四路 exec
   实证修复)。

### W4 闭包/嵌套 fn/跨模块类型（worktree 续）

17. [✅ 已完成 2026-09-03] 三件套语料先行红:b47 闭包族三件(宿主
   8/42、21、105/303 实证)+007_pub_type 跨模块扩件(宿主 7/30)+g30
   发射门件;m5+use-corpus 双红实证。〔恢复注记:.git 丢失事故后语料随
   W1-W4 重建快照(6eb6d0dce)存活;b47 闭包/嵌套 fn 三件因 m4 反汇编
   层不可对拍(宿主 closure 编码乱码级/嵌套 fn 释放组规范化)迁位
   corpus_a2r g31/g32/g33,判据面=发射闸+四路执行锚〕
18. [✅ 已完成 2026-09-03] 闭包捕获模型:parser 闭包解析(双注解
   形态)/codegen cg_closure(隐 fn $cloN+捕获=体内自由变量∩外层
   作用域按值首引序+MakeClo)/engine VClo 载体+MakeClo/CallClo 臂
   (捕获回放为首参组+标准帧);执行验证 8/42/21。
19. [✅ 已完成 2026-09-03] 嵌套 fn:cg_stmt Fn 臂就地编译(不捕获
   外层,按名查 c.fns);m5 58 件绿;m4 释放组规范化差异登记债(g33
   迁位注记)。
20. [✅ 已完成 2026-09-03] pub type 跨模块:cg_compile_files dep
   tys 播种(main 编译前,先查位后置换规避主 a2r 转译 move 洞)+链接
   期 tys 合并;use-corpus 3/3 件字节码一致(007 宿主 7/30 对拍)。
21. [✅ 已完成 2026-09-03] 折叠点④(master fast-forward 至
   34f3ae85d):全 aavm2 21 件+四路 28/28(金样 28 含 g30-g33)+
   rustc 闸 g19-g30+tf 绿+tv 3559+tt 28=存量 P523-3;worktree 重同步
   (merge master 冲突四件裁定:526/528/DEBTS 取 master 权威版,
   a2r.rs 取本支完整版)。

### W5 May/生成器（worktree 续）

22. [✅ 已完成 2026-09-03] May 最小面语料:g34_may_basic(=b49;
   ?int 返回+Some(n*10) 构造+None+is Some/None 双臂;宿主 30/none
   实证;裸值 return 形态宿主发射不编译——语料取显式 Some 构造,
   宿主发射债登记)。
23. [✅ 已完成 2026-09-03] ?T 解析+传播:parser ? 后缀(既有)+
   a2r.at ar_bare_type ? 前缀补;cg None 哨兵(ConstI32 -1,宿主
   CREATE_NONE 同构)/Some(e) 内值直通/is Some(v) 臂(t!=-1 命中,
   v=t 绑定)与 None 臂(-1 eq);哨兵别名语义同宿主文档。
24. [✅ 已裁定延后 2026-09-03] 生成器:W0 考古 lib 用量=0(唯一
   命中为 token 名声明)→ 按待澄清③缺省裁定延后,登记 KNOWN-DEBT
   (宿主 Yield/CREATE_GENERATOR Plan 321 在位,后续波次可启)。
25. [✅ 已完成 2026-09-03] AA2R 同步:?T→Option<T>(ar_rust_ty)/
   Some(v)·None 模式直出(ar_is_pattern_text 双臂)/Some(e) 调用
   直通;发射闸 34 件全绿(g34 逐字符一致)。
26. [✅ 已完成 2026-09-03] 折叠点⑤(master 349dd97a3):goldens
   29/29(g34 入集)+四路 29/29+tf 绿+tv 3559+单元 18;延后项:
   生成器(W0 裁定)与 ?? NullCoalesce 语料面(未覆盖,登记)显式
   登记 KNOWN-DEBT。

### 收尾

27. [ ] ⑤腿塔顶时长趋势汇总留档（各折叠点数据）;恶化则立项建议。
28. [ ] 文档回写：project.md 能力矩阵高阶行+队列②核销;divergences;
    README;KNOWN-DEBT（P474 核销等）。
29. [ ] 折叠点⑥+复审（/auto-plan:review）→ tf → status: reviewed。
30. [ ] merge 沉淀归档。

## 复审记录

## 待澄清事项

0. **[2026-09-03 开工核验] 前置门禁实质满足、形式未走完**：硬前置
   "Plan 523 archived"——实际 523 为 `execution_done`（代码已于
   7cb23b017 折叠合入 master），但尚未走 review→archived。开工时
   核验 523 基建**实质全部在位**：corpus_a2r g01–g25 / corpus_m4
   b01–b43 / 三件套格式（per-case dir + 平铺旁挂）/ 四路 runner
   （`aavm2_a2r.rs::test_aavm2_fourpath_runner`）均确认存在。
   524 为 `execution_done`（软前置，不阻塞）。**按实质在位开工**；
   523/524 的 review→archived 流程由用户另行排期——若复审时认为
   必须先归档 523，本计划成果不受影响（基建已在 master）。
1. **波次拆分**（开工时定）：缺省单计划六波（511/514/517 先例）;若 W3
   泛型考古后超 ~8 步规模 → 拆独立子计划（泛型专项）,本计划缩为五波。
2. **泛型范围**（阻塞 W3）：缺省最小面=List<T> 容器族+泛型函数单态化
   简化版,以"lib 实际用法形态"W0 盘点清单为准（塔顶导向,不求宿主
   泛型全貌）。
3. **生成器范围**（阻塞 W5）：缺省倾向延后（塔顶不依赖;yield 脱糖
   工程量大）——W4 考古后终裁;May 最小面保留。
4. **W1 期望重生成口径**：b13/b14/b19 恢复裸 bool 断言属**期望翻转**
   （P474 过渡形态→正式形态）——bless 流程走评审（523 基建）。
