# aavm

> **Status**: experimental
> 备注：v2；Plan 429-434 系列收口
> 路径:`auto/lib`(v1 已封存于 `auto/lib-legacy/`)| 转译:`auto build`(pac.at)/ `auto trans`

Auto 自举编译器实验(AAVM v2):用 Auto 语言写的编译器前端 + 字节码 VM +
Rust 转译器(AA2R),验证 Auto 的自举能力。Plan 432 起 v2 六文件按依赖序
重写;Plan 434 增 a2r.at(AA2R)完成终极自举闭环。

## 目标与范围

- 用 Auto 实现 Auto 子集的完整编译链:token → lexer → parser(S-expr dump
  判据层)→ typeinfo → codegen → engine(栈式 VM,`ev_run` 入口)。
- Plan 434(AA2R):a2r 核心子集的 Auto 版 —— **Auto 写的 a2r 转译 Auto
  写的 AutoVM,产物是可独立编译的 Rust**;自举回路中不再有任何 Rust
  手写的编译组件。
- 不做:不追求与主编译器(crates/auto-lang)特性对齐;实验性质,不作为
  生产编译路径。多目标(c/py/js)/r2a/逃逸分析完整版明确不做(Plan 434
  Out of Scope)。

## 模块架构(v2,依赖序 = AUTO_LIB_FILES_V2 单一事实源)

```text
token.at ── lexer.at ── parser.at ── typeinfo.at ── codegen.at ── engine.at
   │           │            │                          │            │
 TokenKind   tokenize    parse_dump(S-expr)        cg_compile    ev_run
             lex_dump    + 434 扩展:泛型实例/       codegen_dump   (栈式 VM)
                         type-decl/enum-decl
                                                        │
                                                     a2r.at(434)
                                                        │
                                        aa2r_transpile / aa2r_transpile_merge
                                        (AA2R:token 游标直走,D39)
```

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| lib/token.at | TokenKind 139 变体 + keyword_kind/kind_name | 432 完结(M1) |
| lib/lexer.at | tokenize + dump;434 增 f-string/三引号(D38c) | 432 完结(M1)+434 |
| lib/parser.at | parse_dump S-expr 直出;434 增泛型实例/type-decl/enum-decl
  (D38a/b);514 P 状态类型 14 方法入 type 体(γ4) | 432 完结(M2)+434
  +**514(W3 方法化)** |
| lib/typeinfo.at | typecheck_dump(.type 推断层) | 432 完结(M3) |
| lib/codegen.at | cg_compile 字节码(I{op,s,n} 载体);511 增 struct 四件/
  全局变量/for-in 双通道/use 多编译单元+链接器(池合并+符号定址);514
  CG 28 方法入 type 体+cg_expr/cg_stmt p_kind 链 is 化(C3) | 432 完结(M4)
  +447+511+**514(W3 方法化/W4 C3)** |
| lib/engine.at | ev_run 栈式 VM(Val 判别结构/数组 arena);511 增
  get/set.field 分派/全局区/迭代器零迭代/ev_run_files(初始化序) | 432 完结(M5)
  +447+**511(ev_exec 抽核+多文件)** |
| lib/a2r.at | AA2R:主 a2r 核心子集的 Auto 版(Plan 434);514 增方法族
  发射(type 体方法/ext/static new/接收者合成/|> 脱糖)+自身方法化(Ar
  23 方法,塔顶:自己转译自己 rustc 零错) | 434(见其文件头 Snapshot)
  +**514(W2 方法发射/W3 方法化/W5 管道)** |
| pac.at | 包定义(`auto build` 转译入口) | experimental |

## 判据与闸门

- M1-M5(corpus_m1..m4,`cargo test -p auto-lang --lib --features
  test-vm-files -- test_aavm2 --include-ignored`):与 Rust 参考逐字符一致。
  `.line` 发射语义已定案(Plan 495,2026-08-31:P485-2 清偿——rust 为规范:
  语句边界+同线去重+is 单表达式 arm 体行发射;b14_line_dedup 回归钉;
  规格 `design/m4-bytecode-format.md` §发射模式考古)。
- 五方对比矩阵(Plan 433 四方 + 434 ⑤ aa2r;`parity/` 下
  `cargo run -- --root . --auto-binary ../target/debug/auto.exe aavm`):
  ① reference ② aavm_rust(六文件,见 divergences.md D38 主 a2r 缺口注)
  ③ aavm_vm ④ golden ⑤ aa2r —— 稳定集 corpus_m4 全绿。
- **Plan 511 新增闸门(2026-09-01)**:corpus_use 多文件双闸(M4 反汇编/
  M5 行为,`test_aavm2_m4_use_corpus`/`test_aavm2_m5_use_corpus`)+
  错误用例通道(`test_aavm2_m5_use_errors`,两侧错误文本一致)+
  harness 自证(`test_aavm2_m4_use_harness_selfcheck`,43 件单文件与 S4
  管线逐字符一致)+ **L3 Auto 侧单测**(`test/vm/aavm2/99_unit/`,
  `auto test` 直跑 13 件:引擎微行为/struct/D1 错误文本/模块解析;聚合
  生成器 `scripts/gen-aavm2-unit.py --check`)。语料:c05/p25/p26/p27/
  t08/b34–b43 + corpus_use 六用例+errors 三件。W0 考古规格:
  `design/midlang-w0-archaeology.md`(§7 W2 实测补充)。
- 能力矩阵(Plan 525 后,2026-09-03):struct 声明/构造/字段读写 ✅;
  全局变量 ✅;for-in 数组+返回数组调用 ✅;字符串下标(码点)✅;
  一元负 ✅;下标/字段复合赋值=宿主同文本拒绝(D1)✅;use 四形态+
  跨模块 fn+初始化序+传递依赖+合法环(D2)✅;pub 不过滤(D3)✅;
  **VBool 载体**(print/比较/逻辑结果 true/false,P474-旁支清偿)✅;
  **方法族**(type 体内方法/ext 并入/static fn/接收者 `.field` 简写
  读+写/方法链,独立 fn `Type.method` 重整+self 注入)✅;
  **is-struct 解构模式**(plain struct 恒命中+别名绑定;宿主 VM panic
  洞 525 顺修)✅;**内建容器 List<T>**(嵌套两层/方法集 new-push-get-
  set-len/注解四位=fn 参数·返回·let·struct 字段;CallNat 通道)✅;
  **闭包**(MakeClo/CallClo+VClo 载体,捕获按值首引序;D29 谱系自有
  模型)✅;**嵌套 fn**(就地编译,不捕获外层)✅;**pub type 跨模块**
  (dep tys 播种+链接合并)✅;**May/Option 最小面**(?T 返回注解+
  Some(e) 直通+None 哨兵+is Some/None 臂)✅。延后:生成器(W0 裁定,
  lib 0 使用)、`??` NullCoalesce 语料面、泛型 fn 声明与自定义泛型
  type(lib 盘点 0 使用;单态化决策点不存在——容器族走 CallNat 内建
  通道)、闭包/嵌套 fn 的 m4 反汇编层对拍(宿主 closure 编码乱码级/
  释放组规范化超本轮,判据面=发射闸+四路执行锚,g31-g33 迁位注记)。
- divergence 登记簿:docs/specs/aavm/design/divergences.md(判定规则见
  divergence-rules.md)。
- 风格升级(类 C → 一对一 Rust 对译)前提条件调研:
  idiom-upgrade-prereqs.md(2026-08-25;实证矩阵 + 宿主修复清单 H1-H6 +
  AA2R 扩展 W1-W6 + lib 改写点位与波次建议);立项(三份合并为一,顺序推进):
  [Plan 447](../../plans/archive/447-aavm-prerequisites.md)(aavm-prerequisites:
  ① 宿主加固 → ② aavm 新语法能力 → ③ lib 风格升级)。

## 后续计划队列（2026-09-02 存档,517 收口后领取起草）

> 双目标口径:**目标 1 = VM 模式跑通**(ev_run/ev_run_files,✅ 初步实现);
> **目标 2 = a2r 模式跑通**(lib 经 a2r 转译为 Rust 后行为与 AutoVM 一致)。
> 目标 2 现状勘误:非零——434 已收口自举闭环,旧子集(b01–b33)经
> `test_aavm2_compile_corpus`(主 a2r 转译版二进制,514 W2 起常规门禁
> 37/37)与矩阵②⑤腿(46/46)常态化验证;真缺口=511 中阶语料按待澄清①
> 缺省跳过(b34–b43/corpus_use 前缀 skip)、`auto build`(pac)产品路径
> 与 CLI 入口的 a2r 模式未验。

## 模块化结构(Plan 517,2026-09-02 折叠完毕)

lib 七文件依赖 DAG:`token←lexer←parser←{typeinfo←codegen←engine}`+
`a2r→{token,lexer,parser,typeinfo}`(engine 纯解释器层仅依赖 codegen);
互引 use(auto.lib.* 点路径定向)+pub 契约标注;拼接消费面由双轨剥离消化
(`aavm2_lib_source` 规则单一事实源);CLI 入口 `auto/aavm.at`(真模块
use 形态,行数协议 stdin,见 auto/lib/README.md);AA2R use 发射(g18)。
规格:`design/lib-modularization-map.md`(含执行期定案四条)。

## 收官口径(2026-09-03,532 开工后注记)

532(塔顶自举)是主线**最后一张伞形计划**——落地后 GOAL-017 完整口径
达成(双目标+塔顶自持+AA2R 自译),aavm 转入"用塔"阶段。此后**无预排
计划**,悬置项按触发条件领取:转译器家族(前置=塔顶稳定运行)/P525-1
AST 根治(触发=写回范式再痛点)/生成器与 `??` VM 臂(按需;注意 `??`
VM 侧静默空输出观察项)/OOP 完整面(远期)/532 W0 硬闸可拆补缺前置
(计划内逃生舱)。新语言能力同步走常设规约(下节),非计划形态。

## 能力同步规约(2026-09-02 起,常设判据;2026-09-03 起基建落地)

> **载体状态(Plan 523 W3 落地)**:四路统一 runner =
> `test_aavm2_fourpath_runner`(#[ignore],验收/折叠点档;
> `python scripts/aavm4_check.py --fourpath`);三件套金样 --check/bless =
> `test_aavm2_goldens_check`(`--check`/`--bless`);主 a2r 发射确定性
> 已实证(W0:27/27+merge 三连跑逐字节一致,无非确定位,免规范化)。

> **aavm ↔ AA2R 能力同步**:每当 aavm 目标语言新增能力 X(语料进
> corpus_m*/corpus_use),同步要求 AA2R(a2r.at)能**发射**含 X 语法的
> 程序——三面闸同绿才算能力落地:

1. **VM 闸**(目标 1):corpus 进 M2–M5,VM 模式与宿主对拍一致;
2. **AA2R 发射闸**:corpus_a2r 增同能力件,**VM 内解释执行 a2r.at**
   (`ar_run`)产物与主 a2r `transpile_rust` live 逐字符一致 + rustc
   实编译零错——每能力必跑的主判据;**转译版 a2r.at**(self-bin:AA2R
   转译自身→Rust 二进制,434 自举证据形态)归矩阵⑤腿节拍复核(折叠点/
   复审跑,非每能力必跑)——两形态实证会分叉(P511-1 两根因/slice 边界
   panic 均为"VM 绿、转译红"),不可只留其一;
3. **a2r 运行闸**(目标 2):compile corpus(主 a2r 转译版二进制跑语料)
   无跳过前缀覆盖该能力件。

> 任何一面临时跳过=显式债:divergence 登记 + 进队列计划,**禁止无登记
> 缺省暂缓**(511 待澄清①的处置方式自此作废)。主 a2r(Rust 写)是
> 发射 oracle,不在同步义务内;AA2R 的同步面=aavm 目标语言子集。

**验证矩阵(2×2)**:两组件(aavm 编译器 / a2r.at 转译器)×两执行形态
(VM 解释 / 转译成 Rust)= 四条路径,闸门基建全覆盖:

| 组件＼形态 | VM 内解释执行 | 转译成 Rust 后执行 |
|---|---|---|
| **aavm**(编译运行目标程序) | M5 闸门(corpus 经 ev_run)/矩阵③腿 | compile corpus 闸门(主 a2r 转译整 lib 二进制)/矩阵②腿 |
| **a2r.at**(转译目标程序) | corpus_a2r 闸门(VM 内 ar_run) | 矩阵⑤腿(AA2R self-bin:**含自身**的七文件 lib 自译→二进制,434 自举证据形态) |

> 矩阵①(reference)/④(golden)为 oracle 基准,不计入路径。②与⑤的
> 深度差=谁执笔转译(主 a2r vs AA2R 自身);更深一步"aavm 编译含自身的
> lib"= VM 侧自举塔顶,见队列③缓议项。

**单用例全闭环(理想判据,队列①落统一 runner)**:同一 .at 用例四途径
两族产物三重对拍——①path1/path3(执行输出)互拍且等于 oracle(参考实现/
golden);②path2/path4(Rust 译文)互拍且等于主 a2r;③**译文回链**:
path2 译文经 rustc 编译并**运行**,其输出再与 path1/3 对拍(回链现仅
探针冒烟覆盖且只验编译零错,系统化运行比对是缺口)。日常 CI 维持现有
分闸快跑;统一四路 runner 用于新能力验收与折叠点全量判据。

**用例三件套(金样锚定,队列①落格式)**:每用例备三份预期——
`case.at`(①源码)/`case.expected.out`(②预期执行输出,Plan 177 约定)/
`case.expected.rs`(③预期转译 Rust,Plan 263 a2r_tests 约定);四路径
结果与中间产物各锚定对应金样(path1/3/回链→②,path2/4→③),**主 a2r
译文同锚③**(oracle 自身回归首次入闸,live 对拍盲区闭合)。配套 bless
工作流(金样由参考实现+主 a2r 生成,`--bless` 再生、diff 走评审,有意
发射变更=显式 bless 提交);前置检查=主 a2r 发射确定性(HashMap 迭代
序类非确定位先规范化,M4 槽释放组先例);日常 live 对拍降为快内环,
金样为验收外环。**格式裁定(2026-09-02)=file-based 每用例目录**
(cookbook 为模板,含 reference.rs/运行时夹具扩展位):决定性理由=多文件
用例(corpus_use/跨模块)硬需求+多工件锚定天然形态+bless 逐用例再生
评审回滚+目录协议对 `auto test`/四路 runner 可见;"单文件好读"属
runner 层,以 manifest 解决。markdown 单文件(ast.rs markdown_tests
族)保留为纯文本变换轻量特例不扩面;Rust-inline 限定 Rust 内部单测,
**语言行为用例(输入 Auto 代码+预期输出形态)新增一律文件化,存量高频
触碰时迁移**——避免同一语言行为两处真相。

## 后续计划队列(2026-09-03 三计划已立项:523/524/525)

1. **a2r 模式中阶覆盖收口(目标 2 + 同步规约清欠)**——**已立项 [Plan 523](../../plans/523-aavm-a2r-midlang-coverage.md)**(17 步)——**队列①已核销(2026-09-03,W1–W4 全落):g19–g25 live 25/25、compile 前缀摘除全量 46/46、四路 runner 14/14 判定表、三件套金样 14 件+A2R_BLESS、auto build a2r 链冒烟脚本(scripts/aavm_build_smoke.sh)、aavm.at a2r 模式实测(b07=55;两洞登记 D-a2r-mode-entry)、D-AA2R-struct 清偿+同步规约四规约载体全部落地(本页 §能力同步规约 判据即其验收形态)**:
   ①AA2R 发射面补全:struct 声明/构造/字段读写(b34–b37 对应发射件
   g17+)、for-in 数组/字符串下标/一元负/全局变量(b38–b43 对应件),
   live 对拍主 a2r + rustc;②摘除 compile corpus 的 b34+/corpus_use
   跳过前缀,实测主 a2r 转译版二进制(lib 转译后自带 struct/use 编译
   逻辑,预判多绿;File shim 已为 std 直通真实现仅未触达),红项根因
   修复;③`auto build`(pac)产品冒烟常态化;④517 CLI 入口的 a2r 模式
   验证(依赖 517 落地);⑤清偿 D-AA2R-struct divergence 登记与
   511 待澄清①遗留口径;⑥**四路统一 runner**(单用例四途径一致判定):
   一个 .at 用例 → path1(ev_run)/path3(转译版 aavm 二进制)执行输出
   互拍+对 oracle,path2(ar_run)/path4(self-bin)译文互拍+对主 a2r,
   **译文回链** rustc 编译并运行比对输出(系统化补缺);转译二进制走
   内容寻址缓存不重复构建;产出逐用例判定表;⑦**用例三件套金样格式**
   (`case.at`+`case.expected.out`+`case.expected.rs`,复用 Plan 177/
   263 两约定)+ bless 再生工作流 + 主 a2r 发射确定性前置检查;主 a2r
   译文同锚金样(oracle 回归入闸)。
2. ~~**OOP 批(目标 1 续阶)**~~ **已交付(2026-09-03,Plan 525 六波
   全折叠,四路 29/29;详见 [Plan 525](../../plans/525-aavm-oop-batch.md)
   与能力矩阵)**。原立项(30 步六波,硬前置=523 archived;另 524 宿主小修批[CLI 透传/process.args/parity 新鲜度]可先于/并行 523)。原范围:aavm 目标语言的方法/impl(`type T { fn }`/
   `ext T`)编译执行、is-struct 模式匹配、跨模块类型共享(pub type)、
   闭包与嵌套 fn、泛型(List\<T\> 实例化——通往塔顶必经)、May/生成器;
   搭车清偿 P474-旁支 VBool parity(Val 载体扩展同批)。**验收口径(2026-09-02
   升级)=同步规约三面闸,新语件直接落 per-case dir 三件套金样格式,
   四路统一 runner 判定**——511 式"待澄清缺省暂缓"逃生舱关闭。触发:
   517 复审时按本页能力矩阵起草(范围依赖 514/517 终态)。
3. **塔顶自举**——**已立项 [Plan 532](../../plans/532-aavm-tower-selfhost.md)**(16 步,硬前置=531 archived)。前置 **[Plan 531](../../plans/archive/531-aavm-debt-clearance-batch.md) 债务清偿批已交付并归档(2026-09-03)**:tt 档两真缺陷修复+27 件 bless+复审清单挂载/aavm.at a2r 模式两洞清偿(argv.get 类型化 b07→55 全链+字段表 525 顺带清偿复证)/pac rust target 三缺口/May 裸值双侧(主 a2r+AA2R 镜像)/⑤腿预存 E0382 顺带清偿(矩阵 57/57);`??` 显式延后维持(P525-3 注记)。原范围:自举塔顶:aavm 编译 lib 自身
   (VM 自举);Auto 版转译器家族扩展:a2r 已有(AA2R/a2r.at,434 核心子集),
   剩余=其他语言目标(a2ts/a2js/a2py…)的 Auto 版。缓议理由:在 Auto 写
   工具链能重建自身之前,新转译器仍由 Rust 宿主编译,自举边际价值低;
   与 GOAL-003(Auto as Rust script layer)合流点在塔顶。
