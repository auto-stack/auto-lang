# PLAN-797 规范增量提案（spec-delta）

> 状态：**R8 修复后重置稿**。绑定 R8 修复轮（F-R4-02 descriptor
> 如实化：fallible 出参列表/三出参 ledger/handle 出参/callback ctx
> 车道/i64 元素视图 + record mut/out 生产面与原生提交/回滚控制；
> F-R4-07 真实编译器依赖闭包消费与命中重验；F-R4-09 每请求完整父
> 路径计时与逐样本先落盘（含 Job 对象峰值内存/I/O 归因）；F-R4-10
> 精确表示字节暂存门 + worker 累计产物预算前后检查 + 并发 GC 全链；
> F-R4-12 自身告警清零与终稿事实对齐）。真实 semantic contract 绑定 /
> 正式 AutoDown 执行 / 性能复测已按实际通路重写）与 T-07 材料提交；
> canonical Spec 输入 hash 为下方当前实测值（R2 曾警示 goals.md 漂移，
> 已按当前树重核）。§5 五行意向表是计划契约，本文件是可审阅的规范
> 正文；**不得**在 review/merge 前把本文发布为 canonical Spec 或改
> ledger；本文自身 hash 由 freeze-proposal 与复审记录外置绑定。

| 项 | 值 |
|---|---|
| plan_id | PLAN-797 |
| plan_revision | 1 |
| draft_for | T-07 / F-797-R1-02 |
| candidate_profile | `autoabi.win-x64.v0.1-candidate` |
| implementation_tree | `D:/autostack/.wt/lang-797/auto-lang` @ `plan-797-dev` |
| freeze_state | **candidate**，不是 “ABI v0.1 已发布” |

## Spec inputs（canonical，worktree 终态实测 SHA256，2026-10-20 R8 复核未漂移）

| path | SHA256 | 角色 |
|---|---|---|
| docs/specs/auto-ac/project.md | 008D039E7C2897619407C2E3281907847542921750EC40D53397934A72240A9F | SD-04 导航补丁的目标 |
| docs/specs/auto-ac/data-resource-contract.md | 85B8074B6045F8F1A7DB391D50347182E05E601EC8BEB7D2EB88DAD48161B1C1 | SD-05 分域声明的目标 |
| docs/specs/auto-ac/artifact-contract.md | 2C5305E7FEE33595CEBFE1CB8399F4318984D38D45A8278A0513C841C2049DFE | 产物/链接语境输入 |
| docs/specs/auto-cache/project.md | 7A48BA94FDAE6B2A80CF51CBFF9E49A101942C28217D782FAFBB9A81AA4FB30C | cache key/失效语境输入 |
| docs/specs/auto-test-case/project.md | F2993EC85AA9DF0FD7FFA41D7E159FEA053E592C18D940565A4CD616A8B9AD82 | .case.ad / AutoDown / probe seam 语境输入 |
| docs/specs/auto-semantics/semantic-contract.md | 8E27FCDB6592B7F44A86A85F6BE1619022C72A88A0EA67B1CBD10AF0B897AC2D | 真实语义绑定依据（T-06 消费面） |
| docs/specs/goals.md | EC5170B6B769E33F24735E35541A071EE6E499F09D5BD328F47BF64B6F1E2D7D | touched_goals 依据（当前 `[]`） |

---

## SD-01 | add | docs/specs/auto-abi/project.md

### before

无 `docs/specs/auto-abi/` 公共有限 profile 规范。Design41/42 与
ABI RFC 是目标文档，不是已发布 ABI；774 `probes/data-resources` 的
ABI-A–E 是内部冻结面，不能被读成公共 AutoABI。

### after（规则）

1. **域**：公共边界属于 ABI 域 `autoabi`，与 774 内部 ABI-A–E
   **无兼容关系**；禁止任何“两者可互换”表述。
2. **候选/冻结状态机**：`v0.1-candidate` →（独立 review + merge 门禁）
   → 本 target 有限 profile 冻结。未过门禁不得登记“已冻结”。
3. **支持边界**：仅 Windows x86_64-pc-windows-msvc / Windows x64 调用
   约定；声明源为受限 declaration IR，**不是** Auto 源码 parser 支持面。
4. **工具**：真实 C 侧用可定位 MSVC cl.exe（或 clang-cl）独立编译；
   缺工具 = infra 阻断，禁止 Rust repr(C) 伪通过。
5. **拒绝面**（有限 profile 支持表，不是永久删去战略）：按值聚合、
   variadic、bitfield、任意 union、C++ ABI、异常/unwind 穿界、
   跨线程持久 callback、热重载/DLL 事务、跨域自动转发。
6. **版本五元组**：ABI 域、target/profile、接口 revision（descriptor
   指纹）、runtime 服务表版本、语义 schema 版本分开存在；未知/错配在
   **调用前**拒绝，native 被调次数 0。

### rationale

防止把候选探针冒称为完整 ABI / 产品源码支持面。

### acceptance IDs

AC-01–06

---

## SD-02 | add | docs/specs/auto-abi/windows-x64-v01.md

### before

无首版 Windows x64 合同正文。

### after（规则，经实证）

1. **调用约定**：Windows x64；公开 AutoABI 函数 = 规范 C 标识符符号 +
   显式 context（fallible）+ `i32 status` + 显式 out 指针**列表**。
2. **纯形态**：`ret(params...)`，标量直传直返；参数/返回 ∈
   `{i32,u32,i64,u64,f64,bool(u8 仅 0/1),char(u32 合法标量)}`。
3. **可失败形态**：`i32 status f(ctx*, params..., out*)`；out 是**声明序
   列表**（`p797_ledger` 三出参、`p797_two_step` 出参携带 provider 拥有
   的 handle 均为一等形态）；status 0=成功；**成功才提交 out**；失败
   清理本次部分分配，caller 旧输出不变。
4. **布局**：声明序 + 自然对齐；checked 算术；nominal 身份（同布局不
   自动互换）；与独立 C `sizeof/_Alignof/offsetof` 对照为 oracle。
5. **buffer**：`{i64 ptr, u64 len}`；`len=0` 允许 null；文本另标 UTF-8；
   i64 元素数组视图的长度单位是**元素数**（`buf:i64`，非字节数）；
   越界/溢出/非法 bool/char 受控拒绝。
6. **handle**：opaque owned handle + 原方 destroy + 可校验身份/代际；
   double destroy / 错误 provider / 借用逃逸受控拒绝；handle 可经出参
   槽交付（所有权随 descriptor 声明）。
7. **callback**：仅同步 per-call + 显式 ctx（callback 自身的 ctx 车道是
   描述的物理形态的一部分）；不保存、不跨线程/代际；保存/异步能力 =
   明确不支持并拒绝（不是 loader 承诺）。
8. **record 视图**：const/mut view 与 caller-owned out record 为一等
   车道；mut 提交与失败“逐字节不变”、out 只在成功提交，均有独立 C
   消费者正反原生控制（`p797_crec_mutate`/`p797_crec_make` + 期望翻转
   负控）。
9. **descriptor**：`autoabi.descriptor/3`（/2 单出参形态不再解码），
   canonical JSON + blake3 接口指纹；含 domain/target/profile/packing/
   semref/records/symbols（每个公开符号的完整物理参数/出参列表/所有权/
   callback ctx）/runtime services；未知 format 拒绝。C provider 的
   descriptor 与**独立审定的 header 原型逐项对照**（header 文本机械
   提取声明集 + 手工物理形态表为 oracle，非自洽性测试）。
10. **资源上限**（probe 运行策略）：build≤60s、run≤5s、total≤70s；
   stdout/stderr≤1MiB；IPC≤4MiB；单产物≤64MiB/请求累计产物≤128MiB
   （生产前预留检查 + 每次生产后累计复查，超限=该请求 resource 终止，
   不再运行后续生产）；受管 cache 热点≤8MiB + 暂存/索引另≤8MiB
   （暂存预算按**物化表示 + 索引/递归工作态的精确字节**计：结果结构
   体 + 真实路径串 + 容量余量 + memo 四表与链/seen 名字副本；索引态
   **增量记账、在每次分配其覆盖的副本前检查**，超限在越界记录处拒绝
   ——不得先建完整 memo 再拒，也不得以虚构最大深度/最大名长误拒实
   际极小的合法图；**两阶段峰值分别有界**：size 阶段=部分 memo+当前
   链，flatten 阶段=全量 memo+**按记录记忆化的最长 flatten 路径成本**
   （共享子图的完整展开路径，不因 size 走查 memo 命中被截短）+结果表
   示，后者显式入终门，不得以链回扣额度或走查链峰充当后阶段峰值）。
   超限 = resource 状态，与 ABI 不兼容拒绝分开。

### rationale

可被独立 consumer 按合同编写 callee/caller 并对拍。

### acceptance IDs

AC-01–03

---

## SD-03 | add | docs/specs/auto-abi/conformance-cache.md

### before

无 AutoABI 互调基准 / 缓存规范。

### after（规则）

1. **三腿**（每个 producer 独立命令/obj/lib + descriptor；consumer 独立
   构建并先 admission）：
   - A AutoABI caller → AutoABI callee
   - B AutoABI caller → 独立 C callee（cl.exe）
   - C 独立 C caller → AutoABI callee（可嵌套 C→AutoABI→C）
2. **独立 oracle**：C 侧 sizeof/offsetof + 运行结果；禁止双方同源错误
   layout 生成器互证；禁止以 “call ok” 日志当成功。
3. **负控**：swap 参数 / 错 offset / 漏调用 / tampered descriptor 必须
   可区分失败（exit 恰 1 或调用前拒绝），且已知不兼容签名不制造 UB。
4. **cache**：复用 783 ManagedStore，probe 自有层（ABI-L0/ABI-O1…），
   不改 783/787 注册/GC。key 含语义 ref、canonical decl-set（含 body
   identity）、target/profile、emitter id、C 源名+内容 hash + cl 指纹
   + flags。命中**从不**调 producer，**从不**缓存 PASS；每次仍
   admission + link + run + 重比较 oracle。
5. **C 依赖闭包与适用域**：`#include` 的真实依赖（含注释隐藏、续行拼
   接、宏 include 与系统/MSVC 头）**从编译器自身观测**
   （`cl -showIncludes`），内容哈希随对象一并存储；命中时**逐一重验**
   ，任一变化/缺失 = 真实 miss 重编（文本扫描不作为依赖真相；cl.exe
   二进制哈希不等于头文件内容哈希）。**适用域**同样入信封并逐项重验
   ：源解析目录、include 搜索目录、target、工具链/flags——同名同内容
   的源在不同目录/工具链下是不同编译，不得互用对象（F-R9-01）；
   **负依赖快照**（**候选位置全集**：源目录、-I 目录、每个已记录
   依赖的父目录（预处理器搜索包含者所在目录），**以及全部
   (依赖 P, 解析目录 D1, 链目录 D2) 三元组派生的
   normalize(D2+rel(P,D1)) 父目录**——真实解析 P 的 spec 必为
   rel(P,D1)，对每个链目录应用同一 spec 即得所有可能被新文件抢占
   的候选位置，含 `../` 父路径逃逸到一切已记录目录子树之外的形态；
   派生集合有上限，超限=不可证明适用性、保守不缓存；各目录一律
   **递归**全量清单，**只排除调用方按完整路径声明的编译/链接产物
   及其同位 staged-link 临时件（`<产物路径>.tmp-<txn>`）——不得按
   后缀或 basename 排除**：任何文件名、任何位置（含与产物同名、
   `config.tmp`）的文件都是合法 include 候选；完整路径=**扫描根+累
   计相对路径**（不得用已深入的当前目录再拼前缀），声明产物存在/
   被清理/嵌套均不改变清单（F-R16-01）；
   目录锚点一律 **canonicalize** 绑定实际解析位置与当前工作目录，
   相对字面量在不同工程间不得混用）入信封，任一目录树内新增/删除
   文件（遮蔽改变解析结果）= miss，即使全部已记录依赖内容未变；
   **信封格式版本化**：适用域算法加强即升版本（当前 v4），旧版本
   信封永不服务、miss 后按现行规则重证；**信封版本参与代际键**，
   旧合法代（即使适用域与现行完全一致）永不占用 v4 键，升级重编
   必然发布而非死锁 Conflict（F-R4-07/R10–R14、F-R9-01/R14、
   F-R15-02）；
   环境复原到历史状态时复用对应历史代是正确行为，判据是被服务代的
   适用域与当前环境完全一致。**适用域整体参与代际键**（F-R10-01）：合法重
   编（仅清单/目录变化、闭包不变）在新键下成功发布，不与旧代冲突；
   同代并发竞态的 Conflict 以已存规范代为准。tampered block = 受控
   CAS 拒；缺块 = miss 重算。
6. **生命周期**：512 次接口/实现修订后 live 集收敛；rooted+leased 存活；
   用户 source/exe 哨兵不变；失败/取消释放 lease 且不删交付物；并发
   worker + 逐查询 GC 的全链有真实 worker 证据，且 GC 的**实际回收**
   以收据（ran/deleted/freed）入 meter 断言——水位之下的强制调用
   ran=false 不得冒称回收，rooted 交付物在回收后仍可命中。
7. **性能**：六场景×30；硬门 P95 body-edit <1000ms、descriptor-reject
   <1000ms。正式口径：**每个 query 请求携带编辑后 deck**（产品形态的
   "编辑源→正确 PE 结果"单元；edit 为父侧本地替换，其耗时计入样本的
   edit_ms），计时边界=**父入口→该请求的 typed 判定完成→原始行落盘
   完成**（含请求构建/IPC/等待/解码/admission/判定/写行；门禁消费落
   盘完成后关闭的 span，行内保留判定时点值作对照），worker 内部阶段
   仅作归因子 span，不得以其和冒充请求耗时；**一请求一结果一原始行**（成功
   /失败/infra 均有行，行内 ok=该请求自身判定结果、含失配文本，后续
   请求独立分类；完整阶段、cache/compute、Job 对象进程与树峰值内存
   及 I/O、worker meter 含 GC 收据），在**下一请求运行前**由适配器持
   久化，**写行失败=基础设施错误**（证据链不得静默断裂）；断言只能消
   费这些原始行，不得批量后写或以整 case 总耗时平均替代 P95；工具准
   备单列。
8. **正式通路**：D-796/D-788 后经正式 AutoDown/probe adapter 复测；
   当前 Rust 原生测试是临时等价入口，**不能**充作 AC-05 正式复测。

### rationale

可重验的互调与增量证据，防止缓存假绿。

### acceptance IDs

AC-02–05

---

## SD-04 | modify | docs/specs/auto-ac/project.md

### before

AC project 未导航到公共 AutoABI probe。

### after

补一节 **ABI probe 导航**：指向 `docs/specs/auto-abi/*`（候选/冻结
状态、支持边界）；明确生产 Auto 前端 / 旧内部 profile **仍原状**，本
导航不扩大源码支持面。

### rationale

不扩大源码支持声明。

### acceptance IDs

AC-06

---

## SD-05 | modify | docs/specs/auto-ac/data-resource-contract.md

### before

774 数据/资源合同与未来公共 profile 边界未分域。

### after

补 **分域声明**：774 内部 ABI-A–E 与 `autoabi` 公共候选 profile 分属
不同兼容域；禁止用内部 ABI 的兼容承诺去描述公共 profile（或反向）。
资源 representation（buffer/handle/callback）在公共 profile 中按 SD-02
执行，不复用旧 text 内部 12/8 布局。

### rationale

旧承诺保持，防止误用兼容声明。

### acceptance IDs

AC-01、06

---

## touched_goals

当前 `touched_goals: []` 保持：goals 账本未列独立公共 AutoABI probe 目标；
探针尚未接入产品消费面，不能提前写成 GOAL 产品能力完成。T-07 终稿若
796/产品接线改变目标归属，再提案修订元数据。

## 历史未决项状态（已闭合）

- ~~临时 `autoabi-semref-snapshot/0` 退出收据~~：已退役并留收据
  （`attachments/797/semref-retirement-receipt.md`），T-06 起消费
  `ac-sema-contract` 真实合同。
- ~~正式 AutoDown 执行与性能复测数据绑定~~：T-06 交付；R8 起性能口径
  为每请求完整父路径（见 SD-03 §7）。
- ~~本草稿与最终实现 diff 再对账后才能冻结 hash~~：hash 由
  freeze-proposal 外置绑定，review/merge 前不发布 canonical。
