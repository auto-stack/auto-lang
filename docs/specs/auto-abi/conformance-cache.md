# auto-abi / conformance-cache

> **PLAN-858 开发期周转口径**：ACE 有效样本按升序取第 `N-floor(N/10)` 项（从 1 开始）为 P90，硬门为 P90≤2000ms；正式样本数不减少，10 项最多 1 项超过 2000ms，等值通过。缺失、非法或错误结果不得计入绿；保留各路径原有零值准入、真实窗口、结构与资源断言。P95/P99 作为诊断与历史数据保留；架构基本成形后以实测另定长期目标。详见 [TestCase v2 周转口径](../auto-test-case/profile-v2-native.md)。


> **Status**: candidate（互调基准与缓存规范，PLAN-797 R17 final pass）
> 实现：`experimental/ace/crates/abi`（tests t02–t06 为永久反例族）

AutoABI 互调 conformance 基准与对象缓存规范。每条规则由真实 cl 编译、
rust-lld 链接、PE 执行与 R4–R17 独立反例链支持。

## 三腿与 oracle

1. **三腿**（每个 producer 独立命令/obj/lib + descriptor；consumer
   独立构建并先 admission）：A AutoABI→AutoABI；B AutoABI→独立 C
   callee（cl.exe）；C 独立 C caller→AutoABI callee（可嵌套
   C→AutoABI→C）。
2. **独立 oracle**：C 侧 sizeof/offsetof + 运行结果；禁止双方同源错误
   layout 生成器互证；禁止以 "call ok" 日志当成功。
3. **负控**：swap 参数 / 错 offset / 漏调用 / tampered descriptor 必须
   可区分失败（exit 恰 1 或调用前拒绝）；已知不兼容签名不制造 UB。

## 对象缓存（ABI-O1，复用 783 ManagedStore，probe 自有层）

4. **key**：语义 ref、canonical decl-set（含 body identity）、
   target/profile、emitter id、C 源名+内容 hash + cl 指纹 + flags。
   命中**从不**调 producer，**从不**缓存 PASS；每次仍 admission +
   link + run + 重比较 oracle。
5. **C 依赖闭包与适用域**（F-R4-07/R10–R16、F-R9-01/R14/R15、
   F-R10-01、F-R15-02、F-R16-01 反例链）：
   - 真实依赖（含注释隐藏、续行拼接、宏 include、系统/MSVC 头）从
     编译器自身观测（`cl -showIncludes`），内容哈希随对象存储；命中
     逐一重验，任一变化/缺失 = 真实 miss 重编；文本扫描不作依赖真相，
     cl.exe 二进制哈希不等于头内容哈希。
   - **适用域**入信封并逐项重验：源解析目录（canonicalize 绑定实际
     解析位置与当前工作目录，裸/相对字面量不误拒）、include 搜索
     目录、target、工具链/flags；同名同内容源在不同目录/工具链下是
     不同编译。
   - **负依赖快照 = 候选位置全集**：源目录、-I 目录、每个已记录依赖
     的父目录，以及全部 (依赖 P, 解析目录 D1, 链目录 D2) 三元组派生
     的 `normalize(D2+rel(P,D1))` 父目录（真实 spec 必为 rel(P,D1)，
     对每个链目录应用同一 spec 得所有可能被抢占的候选位置，含 `../`
     父路径逃逸）；派生集合有上限，超限 = 保守不缓存。
   - 各目录**递归**全量清单，只排除调用方**按完整路径**声明的编译/
     链接产物及其同位 staged-link 临时件（`<产物路径>.tmp-<txn>`）；
     不得按后缀或 basename 排除（`config.tmp`、与产物同名异径的头都
     是候选）；完整路径 = 扫描根+累计相对路径；声明产物存在/被清理/
     嵌套均不改变清单。
   - **信封版本化**：适用域算法加强即升版本（当前 v4）；旧版本信封
     永不服务；**版本参与代际键**——旧合法代即使域与现行一致也永不
     占用 v4 键，升级重编必然发布而非死锁。
   - **适用域整体参与代际键**：合法重编（仅清单/目录变化、闭包不变）
     在新键下成功发布；同代并发竞态 Conflict 以已存规范代为准。
   - 环境复原到历史状态时复用对应历史代是正确行为（判据 = 被服务代
     的适用域与当前环境完全一致）；tampered block = 受控 CAS 拒；
     缺块 = miss 重算。
6. **生命周期**：512 次接口/实现修订后 live 集收敛；rooted+leased
   存活；用户 source/exe 哨兵不变；失败/取消释放 lease 不删交付物；
   并发 worker + 逐查询 GC 全链有真实 worker 证据，GC 实际回收以收据
   （ran/deleted/freed）入 meter 断言；rooted 交付物回收后仍可命中。

## 性能硬门（正式口径）

7. 六场景×30；硬门 body-edit 与 descriptor-reject 的 P90 均 ≤2000ms。
   - 每个 query 请求携带编辑后 deck（edit 为父侧本地替换，耗时计入
     样本 `edit_ms`）；计时边界 = **父入口→该请求 typed 判定完成→
     原始行落盘完成**（含请求构建/IPC/等待/解码/admission/判定/写行；
     门禁消费落盘完成后关闭的 span；行内保留判定时点值对照，
     F-R4-09/R11）。
   - **一请求一结果一原始行**（成功/失败/infra 均有行；行内 ok =
     该请求自身判定结果、含失配文本，后续请求独立分类；完整阶段、
     cache/compute、Job 进程与树峰值内存及 I/O、meter 含 GC 收据），
     在下一请求运行前由适配器持久化；**写行失败 = 基础设施错误**
     （F-R4-09/R10）。
   - worker 内部阶段仅作归因子 span，不得以其和冒充请求耗时；工具
     准备单列。
8. **正式通路**：经正式 AutoDown/probe adapter（`abi/interop-v1`
   recipe + `abiworker/1`）复测；Rust 原生测试是临时等价入口，不能
   充作正式复测（F-R4-09/R12）。
