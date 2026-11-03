# auto-a2c record-text HIR→C11 后端合同（CA-B）

> 依据：PLAN-925 §5（CA-B）；实现 `a2x-c/src/{rt_runtime,seed,lower_rt,provider}.rs`。
> 沿 [scalar-hir-backend](scalar-hir-backend.md)（checked 运算/源序/短路/控制流/卫生
> /W4 WX 全部同章适用，不重复）；本合同只记 record/text/text-record 增量。

## C 值/借用表示（冻结）

- `str` view=`a2x_str_view{const uint8_t* data; size_t len}`（UTF-8 **字节**长，
  NUL 是数据、绝不 strlen）；`Str` owned=`a2x_str{uint8_t* data; size_t len}`
  仅显式 free；`char`=uint32_t（非 C char）；record=私有 `struct a2x_rec_<Name>`
  （Str 字段按值拥有；嵌套按 seed 声明序拓扑）。view record 参数=`const struct*`、
  mut=`struct*` 真实 writeback、move=按值转移由 callee 按计划清理；i32/bool 沿
  921 checked int64 中间量。
- 文本语义：`len` 按 Unicode 标量、`len_bytes` 按 bytes、`eq/ne` 完整字节无归一化；
  `char_at/slice` 负值/越界/非法码点在访问/指针算术前具名 trap（TextIndex/
  InvalidScalar）；长度加法/size_t 转换/索引步进先 checked，不依赖 C UB。

## 参数模式与方法（消费验证器事实，不重推导）

- view/mut/move 模式、receiver=params[0] 先于实参各恰一次求值、Copy 求值点快照
  （后续 mut 不可追溯）、mut 文本仅排他读/转发、borrow 返回仅已验 view 参数来源
  ——全部来自 `ParamFacts`/`FnKind`/`BodyResourcePlan`；同根 mut+view 经 C session
  入口具名 `verify.mut-conflict` 拒。
- static/view 方法暖编辑沿窄方法片段（见 incremental-and-cache 的 AXC2/AXP2 节）；
  签名/未知 owner/receiver 形状变更=具名拒（保留槽位形状为权威）。

## 资源计划、失败原子性与出口清理

- lowering 只消费验证器 `BodyResourcePlan`（exit_frees/buffer_frees/owned_returns/
  replace_frees）；显式 clone/deep record clone 产独立 backing；move 责任转移、
  owned 返回保活、借用不 free；循环/分支/短路/trap/cancel 各出口按计划 LIFO 清。
- **作用域释放**：`mark`/`drain_since` 机制——多实参调用中，前一实参的
  view-consumed owned temp 在消费者（调用）之后才 free（先 pin 后 drain；Rust 侧
  同形缺陷登记 D-925-2）。
- **S 窗 owned 守卫**（验证器侧，T-05 修复）：mut 借用根不进 exit live 集——三处
  reown 点以 `plan.owned` 守卫（`text_resources.rs:1822/1951/2028`）；replace_frees
  经借用照旧。mut 资源 record 参数由 callee 排出口 free（902 语料无此形状的存量
  缺口，Rust 侧被幂等 take-free 掩蔽）。
- **失败原子性**：RHS/深 clone 全部 fallible 准备成功后才 commit（旧值存活、部分
  分配回收）；只承诺逐操作 precommit 原子性。逐分配域 fault 1..N 注入/N+1 不动/
  清 fault 同进程 retry；分配/owner 事件纯数据先限额、默认关闭旋钮不改生成 bytes。
- C exit 非零/panic/sanitizer 崩溃**不能**充 Auto trap（trap=stdout 具名行）。

## prelude seed 与文件集

- seed=`a2xc-rt-seed/1` 行式严格编解码（record 表：名/资源/字段+类型；无指纹无
  body）。冷时从活视图捕获，恢复时 view=None 装配传入；解码严格（错 schema/计数
  越界/尾随垃圾/畸形类型/数字开头名全部拒）。
- 4 文件集：`main.c`（921 字节协议包装）+ `a2x_c_rt_runtime.h`（typed trap/
  UTF-8 标量运算/分配预算+fault/事件 env 旋钮）+ `generated.c`（seed 派生 struct
  表+前向声明+片段）+ `manifest.json`；fileset `auto.a2x.c.record-text.fileset/1`，
  与 scalar/Rust 族严格互拒（跨通道配对=REJECTED tamper）。

## 正式预算与实测（2026-11-03，本机）

- 512 真实不同修订（256 body+256 method）全部真 cl 预认证（闭式 oracle；认证
  680s 错峰、不入计时）；六状态×30 样本：hit 7/restart 11/body 22/method 34/
  restore 13（P90 ms），全暖门 ≤2000ms；fresh 440 单列；A2X 工作集合计
  ~10MiB/48MiB；compile≤120s/run≤30s 沿 crun 冻结期限；Job/子孙/reader 清理
  沿受管执行器。
- 内部 C struct/指针布局是**私有**表示，不构成公开 AutoABI；公开 wrapper/import
  与 foreign 语法归 CA-D。
