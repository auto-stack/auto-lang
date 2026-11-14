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
