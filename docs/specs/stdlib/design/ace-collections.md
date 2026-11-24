# ACE 有限集合公共算法切片（PLAN-946）

> Status: experimental（std.list 公共算法 + packages-aggregates 承载面；实现=`stdlib/ace/std/list.at` +
> `experimental/ace/crates/core/src/source/packages/*`（packages-aggregates 面）+
> `experimental/ace/crates/{source-query,a2x-store,a2x-rust,a2x-c,native,av-core,av}` 对应臂；证据=
> `experimental/ace/crates/av/tests/cases/{stdlib_list_946.rs,p946_t01_probe.rs}`）。

## 1. 公共算法合同（冻结 std.list，身份=仓库文件内容）

`stdlib/ace/std/list.at`（包 `std`，经 `stdlib_aggregates_inputs()` 供 aggregates 线；`is_frozen_std_module` 身份校验，
非逐字节同源输入 `ace.stdlib-reserved` 拒）：

| 算法 | 源码形态 | 语义 |
|---|---|---|
| `sum` | `pub fn sum(xs List<int>) int` | for-in 归约（无闭包；闭包 HOF 归 964/972） |
| `find_first_gt` | `pub fn find_first_gt(xs List<int>, threshold int) ?int` | 首个 >threshold 元素 Some(x)，否则 None；条件一律**括号化**（见 §5 歧义） |
| `pop` | `pub fn pop(xs List<int>) ?int` | `remove(len-1)` 包装；**空 pop=None**（旧 VM `ListStorage::pop` 原合同），空 remove 裸形=trap `list-index`（方法表无 `drop`，释放经 `clear`） |
| `total_len` | `pub fn total_len(xs List<Str>) int` | 拥有 Str 元素长度折叠（元素 view 只读） |

方法表（基面 collections profile 冻结）：恰为 `new/push/insert/get/set/remove/len/clear`；`pop`/`drop`/容器级 `clone`
不在源码面（`clone` 双面拒"elements clone explicitly"；D-962-C4 use-after-move 为 KNOWN-DEBT 非合法所有权）。

## 2. 承载面：packages-aggregates（grammar profile `packages-aggregates`）

packages-modules 全表面＋集合族最小语句面（`for-in`/`is` 及 match 臂）＋`?T` 可选类型位（参数/结果）＋
`List<T>.member` 接收原子。aggregates-only 扩面（range-for、索引赋值、tuple、union、解构、grid、数组字面量、`..`）
**不在语法面**；plain/ct 裁剪逐字节不变（grammar-id 按 profile 隔离）。管线：`parse_package_source_aggregates`
→ `bind_packages_aggregates`（Walker 面感知：Optional/GenericType 放行、`List`/`Result` 内建名义与
`Some/None/Ok/Err` 变体 ctor 原名透传）→ `compile_package_product_aggregates`（resolve/typecheck=Aggregates、
List/Variant 类型表按 962 域发射；lane 携带 `av.aggregates/1` 准入）→ `lower_checked_closures_with_init`。

**域边界（具名）**：math/text 属 TextRecord 域不进 aggregates 线（混域图 `str` 具名拒）；单模块与 packages 两面下
List 参数拒绝码分别为 `source.list-value-context`（resolve 层）与 `source.module-symbol`（binder 层）。

## 3. 跨单元/跨端 List 实参语义（统一为共享/独占顺序借用）

AV 链接快照按全单元族标志答 collections 谓词；跨单元 List/Variant 实参**保持调用者所有权**（Rc 共享，被调可变对
调用者可见——pop 后 len=2 四端一致 3002）。Rust 生成器＝View List 参数 `&'a mut A2xList<E>`＋调用点 `&mut`；
C 生成器＝可变指针＋调用点取址；native＝I64 句柄过 ABI＋889 序幕容器参数 SHARE（move 分支对容器为死码）＋
无闭包容器模块路由 collections 线（`build_collections_checked` 桥）。跨模块拥有文本移出（`remove` 返回 owned Str）
后源 `clear` 释放，返回文本仍可读。

## 4. 失效/恢复与资源边界

body 编辑＝精确段失效（仅被编模块段移动，std.list 字节稳定）；同 store 首编辑后首跑即新值；a2x restore 未变产品
0/0 重发；编辑前 receipt 用前拒。元素型变更 `source.type-mismatch`、元数变更 `source.argument`（backend 前）。
资源边界：AV arena fault 扫（每次分配尝试具名中止 live=0）＋ a2x `A2X_RT_FAULT_ALLOC` typed trap＋OOB=ListIndex。

## 5. 已知边界与具名 pending

- **ctor 歧义（既有，非本包引入）**：`if a > t {`（var RHS＋同行 `{`）被 record-ctor `t { }` 形抢先提交而拒——
  plain 与 aggregates 两面均拒（测试钉死）；源码惯用括号化形 `if (a > t) {`（945 先例）。
- **AV/C＋AV/Rust＝具名 pending**：阻塞①包绑定层拒 use.rs/use.c（单模块面）②967 F0 帧仅单 &str 族；解锁=967
  包级 foreign 准入与富帧（1041 已归档先例同款分账）。std.list 纯算法无外部 dispatch（设计稿纯函数分类条款）。
- **加载**：std.list 为纯 `.at` 公共文件，无 #in/来源后缀需求；#in/装配腿归 [context-source-assembly.md](context-source-assembly.md)（1043 域）。
- 原生底座：算法消费调用者/区域 Storage（1039 `platform.mem/1` 域），不直触平台 ops；通用 Storage 策略归 972。

## 6. 验证入口

`cargo t` 固定组并集（最终冻结基线，merge 记录）：`stdlib ace-source ace-lowering ace-av ace-native ace-a2x
case-ace-m1-collections-864 case-ace-m1-native-collections-889`；套件 `stdlib_list_946`（31 例，四端真工具链：
rustc/cl/mn 机器码/OS Python）＋`p946_t01_probe`（11 例）。六路径分账：AV/native/A2C/A2R/A2PY 实跑＋
AV/C、AV/Rust 具名 pending。

## PLAN-1121 增量（2026-11-23，普通面迁移容器合同）

`HashMap`/`HashSet` 迁移容器（`stdlib/ace/asm/map_str.at`，普通 Auto 面嫁接，非本装配集）的
键值与生命周期合同（细节见 [m2-stdlib-core](../../ace-migration/m2-stdlib-core.md) PLAN-1121 增量）：

- **容器资源**：迁移 map 的存储 = 四条并行 List（keys List<Str>/tags List<int>/ivals
  List<int>/svals List<Str>），set = 一条 List<Str>；全部经冻结 collections 方法表
  （new/push/get/set/remove/len/clear），无第二容器机制、无 arena 外堆。
- **键相等**：精确文本相等（Str==Str，元素视图不经转换不与 str 形参混比）；扫描为
  for-条件线性查找；插入序由 collections 表保留，但合同不承诺迭代序。
- **替换/清空**：put 命中原地覆盖（大小不变）；remove 命中压缩全部并行表（返回 true），
  未命中返回 false；clear 清空全部并行表。
- **缺键行为**：缺键 get 走受检失败通道 `fail("KeyError")`（可 catch，catch 后 owner
  存活可续用）；contains 给存在性；迁移面 `drop()` 为使用结束标记（释放观察在作用域退出）。
- **所有权**：写方法 owned-Str 形参移入（`.move` 入存储）；`get_str` 返回显式元素 clone
  （D-962 elements-clone-explicit；容器级 clone 保持双面拒）；记录实参跨函数 Rc 共享、
  被调变更对调用者可见；变量间赋值 = MOVE（`mir.use-after-move`）。
- **静态面纪律**：每接收者局部变量一个值面（int/str，int 默认）；面冲突与未知面为
  绑定期具名拒（`source.map-face-mismatch`/`source.map-face-unknown`）；错值型在
  Verify 阶段 `verify.type-mismatch` 拒。

## PLAN-1131 增量（2026-11-24，跨函数传递、返回与最后所有者）

普通面迁移容器（非本装配集）作公共类型跨函数使用的合同；类型绑定与旧导入见
[m2-stdlib-core](../../ace-migration/m2-stdlib-core.md) PLAN-1131 增量。

- **跨函数传递**：持有资源的容器记录作实参按共享引用传递（不复制）：被调方的插入、删除、
  清空对调用者可见，无需 `.move`；`HashMap`/`HashSet` 注解与构造器是同一记录身份。
- **返回**：被调方 `return m` 把**局部**容器交给调用者；调用者继续查询、替换、删除均合法，
  返回后调用者是其所有者。经形参收到的容器不可直接返回或存入记录
  （`mir.return-type` / `mir.aggregate-field-type`，资源记录的既有规则，债务 D-1131-PARAM-RETURN）。
- **最后所有者与释放**：存储随最后持有帧退出而释放——被调方局部未返回则随被调帧退出；
  返回给调用者则随调用者作用域退出；经形参的失败路径（`fail`）同样对账。arena 上
  分配 == 释放、live = 0；两种归属的分配数相同（同一存储，返回不触发复制）。
- **值形状跨调用**：静态值面按**各函数自身的观察**追踪（形参是该函数的新局部）。读一个本函数
  未观察过的形参上的统一 `get` = `source.map-face-unknown`（写 `get_int`/`get_str`，或先
  insert）；同一函数内的面冲突 = `source.map-face-mismatch`。被调方可向调用者的 map 加入
  另一面条目（容器按条目存面）：调用者用统一 `get` 读该条目走受检失败通道（ValueError，
  可 catch，owner 存活）——交界处的异面不产生静默错读；调用点不做静态面核对
  （债务 D-1131-CROSS-FN-FACE）。
- **StringBuilder 资源**：片段为 owned `Str`，存于一条 `List<Str>`；每次 `build()` 的拼接文本是
  独立的 owned 分配，随作用域退出释放（账本差值对每次 `build()` 恒定）。
- **跨真实模块边界（r2）**：普通 Auto 包闭包里，一个模块导出接收/返回容器的函数，另一个模块
  经 `use app.<模块>.<名>` 调用——绑定到的是同一份迁移记录；被调模块的插入、覆盖、删除对调用
  模块可见；入口模块对返回的容器继续查询、删除；容器作为记录字段经第三个模块传递亦成立。
  入口退出时 arena 分配 == 释放、live = 0、无残留帧。
- **遮挡规则**：模块声明或导入了同名类型时，该类型优先于公共容器（包绑定器先解析模块声明与
  导入，再把名字交给嫁接回退）。

