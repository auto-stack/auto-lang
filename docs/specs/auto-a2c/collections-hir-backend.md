# auto-a2c collections HIR→C11 后端合同（CA-C）

> 依据：PLAN-926 §5（CA-C，T-02 冻结）；实现
> `a2x-c/src/{coll_runtime,seed_coll,lower_coll,coll_table,provider}.rs`。
> 沿 [scalar-hir-backend](scalar-hir-backend.md) 与
> [record-text-hir-backend](record-text-hir-backend.md)（checked 运算/源序/短路/
> 控制流/卫生命名/W4 WX/参数模式同章适用，不重复）；本合同只记 collections 增量。

## 支持域与 C 表示（T-02 冻结）

- 904 已冻结集合域：i32/bool、受限 owned Str（literal/len/clone）、无容器字段非递归
  record、有限名义 enum/Option/Result（臂序=disc）、局部 `List<T>`（T∈int/bool/Str/
  record/variant）。List 不可参数/返回/record字段/Variant payload/嵌套；char/str
  借用、闭包/HOF/Unit 入口原拒绝保留；零参 i32 自由函数 entry。
- List 按元素名义实例化 `a2x_list_<etag>{E* data; size_t len; cap}`（etag∈
  i32|bool|str|rec_*|var_*）；初始逻辑 cap4、×2 增长、MAX 65536；typed helper 无
  void* ops 表。Variant=`a2x_var_<name>{int32_t disc; union 按臂 p_<arm>}`，disc
  守卫一切访问（memset 后仅置活跃臂）；错臂投影=具名 VariantBranch 且 root 不被
  消费；take 移出载荷并清壳。record 沿 925 `a2x_rec_<name>`。
- 类型表 seed（`a2xc-coll-seed/1`：R/V/A/L 行式严格编解码）携带 records+variants+
  list 实例化；`generated.c` 的类型表按**依赖拓扑序**发射（声明序非拓扑——N889
  嵌套前向引用），环拒绝；list 结构体在元素结构体后。

## 责任：唯一受检 CollectionsBodyFacts（不重推导 ownership）

- lowering 只消费验证器 `CollectionsBodyFacts`（owned/exit_frees/replace_frees/
  err_frees/loop_survivors + body digest 凭证绑定）；缺事实/外来凭证具名
  `a2x.internal` 拒。S 面 join 语义（PLAN-926 修复）：**终止分支不约束 join**——
  then 臂 return/break/continue 后无 fall-through 端点，无 else 早返回形状的隐式
  else 存活集由 join 承接（旧 common=∅ 孤儿化存活集已修复，golden-L03 最小重冻结）。
- 每出口（return/分支汇合/回边/break/continue/每 fallible 节点错误路径）按 facts
  LIFO 清帧；嵌套块 fall-through scoped 死亡（exit_frees[(block,None)]）由块尾发射
  （尾终结语句保持自身出口键）；for-in 脱糖为 `loop{ if(i<n){体;i++} else break }`，
  回边 poll 错误路径=每迭代 scoped 集+幸存者（幂等释放使重叠安全）。
- **幂等 free**（T-02 冻结）：Str take 置空、list free 后 data=NULL/cap=0、record/
  variant free 后 memset owned 字节——第二次释放为 no-op（回边 poll 错误路径故意
  重列每迭代集合）。
- **wrap_consume**：push/insert/set 按值接收已 move 输入，helper 自身失败路径唯一
  回收未提交输入（该值 temp 退出 caller 侧 err_frees）；checked view（`_view` 出参
  返回 const E*）索引检查先于访问、不物化；Copy get 出参快照；remove 移出所有权；
  set 旧值 commit 时恰一次释放。
- **包装临时释放**：fallible 调用（a2x_stro_result/a2x_res_* 包装）的 `e<id>` temp
  被 free 调度引用时释放 `.value`（正式负载抓出的 C4133/内存破坏缺陷已修）。

## 失败原子性与计费（冻结表）

- 每真实 allocation 域 1..N 故障注入（A2X_RT_FAULT_ALLOC，门先于真实请求、失败
  尝试不计分配不产事件）；失败半树/前缀全部回收、旧 target 完整、清故障同进程
  retry 可核（探针直连 runtime 静态）。事件账本（coll:alloc/free-backing）字节级
  平衡为泄漏/双释放判据（realloc 增长使计数可合法合并）。
- 计费：2WU/slot（new 基4+增长）、ceil(bytes/8)WU 每 Str 拷贝、1WU/ctor、4WU/call
  frame、1WU/poll；默认 2^24（A2X_RT_BUDGET）；cancel（A2X_RT_CANCEL）仅在 poll
  观察=Cancelled；Budget/Cancelled/Resource/ListIndex/VariantBranch 严格分态。
  cleanup 零计费/零 poll/零分配（零余量下释放仍闭合）。深链见证（3072 层/256KiB
  小栈）来自同生成 runtime 的迭代 deep-link 助手（无递归）。

## 文件集与身份

- recipe `ac-a2c/hir-collections-emit-v1`、片段层 **AXC3**（schema
  `a2926/axc-c-coll-fn/v1`）、fileset `auto.a2x.c.collections.fileset/1`、cert
  `ace.a2c-hir.collections.cert/1`（执行配置为键材，见
  [a2c-hir-collections-certification](../auto-test-case/a2c-hir-collections-certification.md)）、
  seed `a2xc-coll-seed/1`、锚 **AXP3**（`a2926/axp-coll-source-anchor/v1`，见
  [incremental-and-cache](../auto-a2x/incremental-and-cache.md)）。
- 文件集：`main.c`（i32 观察协议 wrapper）+ `a2x_c_coll_runtime.h`（预算/取消/故障/
  活账 + Str 层 + 迭代深链）+ `generated.c`（拓扑类型表 + 前向声明 + 片段）+
  manifest（绑定被替换**全源** sha256——暖授权身份）。
