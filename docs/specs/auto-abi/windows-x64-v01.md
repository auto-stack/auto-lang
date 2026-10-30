# auto-abi / windows-x64-v01

> **Status**: candidate（v0.1 经实证规则，PLAN-797 R17 final pass）
> 范围：首平台 Windows x86_64-pc-windows-msvc / Windows x64 调用约定

首版 Windows x64 有限 profile 合同正文。全部规则由
`experimental/ace/crates/abi` 的真实编译/链接/PE 执行与独立复审证据支持
（R4–R17 反例链）；每条后注其 finding 出处。

## 调用与形态

1. **调用约定**：Windows x64；公开 AutoABI 函数 = 规范 C 标识符符号 +
   显式 context（fallible）+ `i32 status` + 显式 out 指针**列表**。
2. **纯形态**：`ret(params...)`，标量直传直返；参数/返回 ∈
   `{i32,u32,i64,u64,f64,bool(u8 仅 0/1),char(u32 合法标量)}`。
3. **可失败形态**：`i32 status f(ctx*, params..., out*)`；out 是**声明序
   列表**（三出参聚合、出参携带 provider 拥有的 handle 均为一等形态，
   F-R4-02/R8）；status 0=成功；**成功才提交 out**；失败清理本次部分
   分配，caller 旧输出不变。
4. **布局**：声明序 + 自然对齐；checked 算术；nominal 身份（同布局不
   自动互换）；与独立 C `sizeof/_Alignof/offsetof` 对照为 oracle。
5. **buffer**：`{i64 ptr, u64 len}`；`len=0` 允许 null；文本另标 UTF-8；
   i64 元素数组视图的长度单位是**元素数**（非字节数，F-R4-02/R8）；
   越界/溢出/非法 bool/char 受控拒绝。
6. **handle**：opaque owned handle + 原方 destroy + 可校验身份/代际；
   double destroy / 错误 provider / 借用逃逸受控拒绝；handle 可经出参
   槽交付（所有权随 descriptor 声明，F-R4-02/R8）。
7. **callback**：仅同步 per-call + 显式 ctx（callback 自身的 ctx 车道是
   描述的物理形态的一部分）；不保存、不跨线程/代际；保存/异步能力 =
   明确不支持并拒绝。
8. **record 视图**：const/mut view 与 caller-owned out record 为一等
   车道；mut 提交与失败"逐字节不变"、out 只在成功提交，均有独立 C
   消费者正反原生控制（`p797_crec_mutate`/`p797_crec_make` + 期望翻转
   负控，F-R4-02/R8）。

## descriptor

`autoabi.descriptor/3`（/2 单出参形态不再解码）；canonical JSON +
blake3 接口指纹；含 domain/target/profile/packing/semref/records/
symbols（每个公开符号的完整物理参数/出参列表/所有权/callback ctx）/
runtime services；未知 format 拒绝。C provider 的 descriptor 与
**独立审定的 header 原型逐项对照**（header 文本机械提取声明集 + 手工
物理形态表为 oracle，非自洽性测试，F-R4-02/R8/R9）。

## 资源上限（probe 运行策略）

- build≤60s、run≤5s、total≤70s；stdout/stderr≤1MiB；IPC≤4MiB。
- 单产物≤64MiB；请求累计产物≤128MiB（生产前预留检查 + 每次生产后
  累计复查，超限=该请求 resource 终止，不再运行后续生产，F-R4-10）。
- 受管 cache 热点≤8MiB；暂存/索引另≤8MiB：暂存预算按**物化表示 +
  索引/递归工作态的精确字节**计，索引态增量记账、在每次分配其覆盖的
  副本前检查（不得先建完整 memo 再拒，也不得虚构最大深度/名长误拒
  合法小图，F-R4-10/R10/R11）；**两阶段峰值分别有界**——size 阶段 =
  部分 memo+当前链，flatten 阶段 = 全量 memo+按记录记忆化的最长
  flatten 路径成本（共享子图不因 memo 命中被截短）+结果表示
  （F-R4-10/R12）。
- 超限 = resource 状态，与 ABI 不兼容拒绝分开。
