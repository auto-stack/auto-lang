# auto-a2x 增量与 cache

> 状态：active（AX-A 形态，`a2x-store` crate；PLAN-892 T-04/§5.4）。

## 显式有限模块装配（PLAN-944）

`ModuleA2xTarget::{cold_assembly,prepare_batch,restore_assembly}` 复用现有
provider、批量/单函数 producer、语义计划、片段 store 与文件集组装。
新根 `a2x-source-modules/2` 绑定 `ace-stdlib-assembly/1` 清单、每函数物理
context 和入口实际可达的显式 requirements。原根 `/1`、原无 context
入口/键字节保持原义；旧入口拒绝新根，恢复新根须当前期望清单。

`AXS1` 仍不读取 target/provider 字段。只有 `AXR1` 的显式模块装配配方
增加可选键组件 `assembly_context_v1`，内容是该函数 own/direct-read
声明版本及其消费者的 producer/ABI/runtime/发射配方、owner-scoped
provider 输入。普通旧配方完全不带该组件。provider 的空 owners 表示
link/装配输入，不表示“全部函数失效”。未改片段沿原 artifact 复用；
另一消费者的物理输入变化不触发本消费者 lower/print。

该有限标量入口的 runtime 身份为实际 provider runtime 文本 SHA256，
toolchain 身份为当前 `fragment_key_inputs` 发射配方的内容身份（语言/
edition/backend/version/runtime/printer）；外部 rustc 可执行产物的认证
继续归原 Rust consumer，不在此缓存 compiler executable。

`source-modules-root/3` 持久化显式声明 context、原 CST/facts/声明环境与
两个目标 receipt 的 required 闭包。`AssemblyHost` 只通过既有
`replace_current_root` CAS 发布这一个根，准备不安装。取消、绑定失败、
事务插入/退休失败均保旧根/文件/可执行代码；恢复只读回原 payload，
不以冷编译代替。恢复后的首 body 编辑仍是一函数 prepare/lower/print，
无关函数零生产；需求变化重验真实读取函数。

新模式 `run_authorized` 每次先调用当前调用方的授权检查，再把当前文件集
交实际执行 consumer；清单/根不持有 grants 或宿主指针。详见
[ACE绑定与装配](../stdlib/design/ace-binding-and-assembly.md)。

## 层注册（A2X 自有命名空间，不占 Design 40 A00–A12）

| 层 | 内容 | ChunkKind | 再生 |
|---|---|---|---|
| `AXS1` a2x-function-semantic | 每函数语义计划（owner/decl/profile/hir_schema/签名/sem_fp/caps/pass 入键；**零 target 字段**） | Hir | FromUserInputs |
| `AXR1` a2x-rust-fragment | 单函数打印片段（语义键输入 + lang/edition/backend/backend_version/runtime_sha256/print_version） | Object | FromUpstreamRecipe |
| `AXM1` a2x-fileset-manifest | 装配 manifest（required 边钉住片段集） | ObjectManifest | FromUpstreamRecipe |
| `AXS2` a2x-rt-function-semantic | AX-B record/text 语义计划（同 AXS1 键形；签名=视图化 mode+名义渲染，pass=`a2x-prepare-rt/1`） | Hir | FromUserInputs |
| `AXR2` a2x-rt-rust-fragment | record/text 片段（runtime_sha256=**record-text runtime** 内容哈希——与标量族永不交叉命中） | Object | FromUpstreamRecipe |

两族并列于同一 ManagedStore（`a2902/*` 独立命名空间）；标量族键/字节/命中
不变。AX-B 指纹携带 callee canonical 签名与 record 形状摘要（被调方接口/
名义变更使调用方失效；布局重排稳定）。

## 会话合同（`A2xSession`）

- **冷**允许整源；**暖纯 body 编辑只走片段**：`parse_function_fragment` 只解析
  被编辑函数（整源解析会重建前端瞬态，管理预算必然破——实测依据）；
  候选事务=prepare→validate→commit 单次发布（generation 递增）。AX-B：
  `cold/restart_assemble_profile` 接纳 record/text/text-record（rt 文件集
  含名义 record+克隆 helper 前置，纯居民组装零 lowering 也完整）；text 族
  warm 编辑走 `verify_text_function` 凭证（保留环境排他种子）；**record 族
  源（含 `type` 声明）的 warm 片段编辑暂不可用**（828 graft 的 type span
  重锚=前端工作项），其编辑走整路径且未改单元零生产（实测）。

## 实测（AX-B，2026-11-01，本机）

代表性资源负载（资源 record+方法+24 caller 队列；type-free 变长编辑负载）：
32 修订全预认证；hit P90=6ms / restart=21ms / body-edit=44ms（暖门
≤2000ms）；fresh P90=722~827ms 单列；store 命名空间 ≤2.93MB。
- **居民复用**：按源名键居民片段；未改函数暖编辑**零生产零写入**（ProducerStats
  机械断言）；A→B→A 经 store 命中复用真实历史字节（不铸同 hash 假复用）。
- **restart 再准入**：新会话清居民后纯 store 命中重建（字节一致）；read-only
  miss 真实重算零写。
- **配额门**：put 前按空间报告核算，越界=具名 `a2x.resource` 拒绝且旧态完整
  （拒后 restart 仍纯命中）。
- **close**：`CheckedEnv::close` 后一切暖编辑为 `source.closed`（先于查表——
  close 清空 fn 表）；居民与 store 继续服务装配。
- **计量**：`ProducerStats`（parse/prepare/verify/lower/print/persist/bytes/
  hit/resident/whole_fallback）为真实计数，探针逐项断言。

## 实测（2026-10-30，本机）

- load256 规模（273,575B/115 fn）暖稳态 max 现时 WS **29.7MiB ≤ 48MiB**；
  冷整源峰值 ~338MB（debug==release，~1.2KiB/源字节）＝上游 ac-core 前端
  特征，归 AC 线，非 A2X 预算面。
- 正式采样（31 修订全预认证）：hit P90=427ms / restart 490ms / body-edit
  176ms（三暖门 ≤2000ms）；fresh（冷整源）P90=10229ms 单列（828 先例）。

## Collections 增量与恢复（PLAN-904）

- 层：AXS3（`a2x-collections-function-semantic`）/AXR3（`a2x-collections-rust-fragment`）；
  runtime 内容 hash 入 fragment 键（标量/rt 命中不可满足 collections 键）。
- 暖合同 `warm_edit_ranged`：base 完整源 sha 绑定/UTF-8 range 精确单声明/同名片段；
  替换后完整源 sha + execution_config 入 manifest。restore/restart 逐内容重验再命中，
  只读写 0，close 先于一切查找。

## PLAN-921：C 片段层 AXC1、来源绑定锚 AXP1 与纯恢复

- **AXC1**（`a2921/axc-c-fn/v1`，producer `auto-a2x-c`）：C 片段键 =
  owner/decl/sem_fp + lang=c/standard=c11/backend=auto-a2x-c/
  backend_version/runtime_sha256/print_version(a2x-c-print/1)。与
  AXR1..3 互不可命中（backend/standard/runtime/print 全异）。
- **AXP1**（`a2921/axp-source-anchor/v1`，producer `a2x session`）：键
  owner/profile/source_sha256/backend/entry；载荷为行式严格编解码的锚
  （每函数 origin/slice sha/sem_fp/signature + runtime 文本 + 全文件集
  sha 回执）。**发布即提交点**：锚的 RequiredData 边在 put 时活性检查
  ——锚落地 ⇒ 其片段集完整存活；崩溃于锚前=store 无锚可恢复（恢复拒绝，
  绝无半可恢复态）。同 key 异内容=store 冲突（内容寻址不可变）；
  字节同一再发布幂等且不耗 quota。
- **纯恢复**（`restore_source_bound`，新进程入口）：caller 供完整源+
  声明上下文/目标/工具身份；按源 sha 查锚→严格解码→逐函数 slice 绑定
  重验（R）→语义/片段载荷字段重验（R）→provider 重装配并逐文件对回执
  （R）。**P0**（lower/print/persist/parse 零增量）；任何失配=命名拒绝
  且**什么都不安装**；无锚=拒绝（冷 fallback 违反 P0 契约）；非标量族
  拒绝（view 衍生 prelude 无法盲装配）。恢复后暖编辑经 `ensure_env`
  整源重建（M 类，env_rebuilds 计数；此后仅被编辑函数生产）。
- **ranged 暖编辑扩展到标量族**：C 链的暖编辑走 base+range+fragment 授权
  （严格超集）；成功后按 next_sha 重绑锚（origin 平移+edited 函数
  slice/sem_fp 重绑+全文件集回执刷新）。
- **计数器**：restore_lookup/binding_checks/payload_verifies/
  fileset_verifies/anchors_published/env_rebuilds；恢复绝不触碰生产
  计数器——"cold 算 key 冒 P0"由计数器断言机械反控。
## PLAN-925：C record-text 片段层 AXC2、来源锚 AXP2 与方法暖编辑（CA-B）

- **AXC2**（`a2925/axc-c-rt-fn/v1`，producer `auto-a2x-c`）：C RT 片段键沿
  AXC1 形 + RT runtime/print 身份；与 AXR1..3/AXC1 互不可命中。
- **AXP2**（`a2925/axp-rt-source-anchor/v1`）：v1 字节不动、新增**携带 prelude
  seed 行**（`a2xc-rt-seed/1`，与 AXP1 严格配对按 family 分层）；RT 锚仅 C
  目标发布（Rust rt 保持无锚现状）。冷发布/ranged 重绑带 schema+seed；按
  owner 限定 decl 重绑（裸名会致方法条目漏绑+错位切片 sha 自洽——已修）。
- **纯恢复放宽至 RecordText+C**：AXP2 查找→切片/语义/片段/文件集逐项重验→
  view=None+seed 盲装配→装 residents、env=None；P0 计数器断言同 AXP1。
  下一编辑经 `ensure_env` 整源重建（M 类，env_rebuilds 计数）后走函数/
  **方法**暖路——restore→next **method** edit 用 ranged 授权
  （`warm_edit_ranged_method`；非 ranged 方法编辑需活 env，具名拒）。
- **方法暖编辑**：窄方法片段（`parse_method_fragment`，Receiver Static|View）+
  owner 限定 env 槽位（裸名 fn 表 vs 方法 `d_f{slot}_{name}` 分叉）+脚手架
  方法 span inert（904 规则补齐）。签名/mode/名义字段/borrow/实际 body 读取
  依赖精确失效；未改兄弟方法/同名自由函数/caller resident 字节恒等；
  whole_fallback 恒 0（机械断言）。
- **事务/失败**：失败编辑回滚源绑定（manifest 仍绑 next：预设+失败还原）；
  readonly 零发布；reader lease 保活到最后 owner 后物理 GC。
- **P/R/M 分账（实测 2026-11-03）**：512 真实不同修订（256 body+256 method）
  全 cl 预认证；hit P90=7ms/restart=11/body=22/**method=34**/restore=13；
  fresh 440 单列；未改单位账本（编辑态 lower/print=30、persist=60）；
  A2X 工作集合计 ~10MiB/48MiB；子进程证人恢复 P0（lower=0）+ranged 方法
  编辑 env_rebuilds+1。CA-D 交接身份：seed/schema/recipe/fileset/cert 全部
  版本化字面量，漂移=STALE/拒，不静默覆盖。

## PLAN-931：Rust 打印轴 a2x-print/2 与 owned 临时责任入键

- **AXR1..3 的 `print_version` 升 `a2x-print/2`**：owned 临时消费者锚定
  释放（D-925-2 关闭）、external 受检 modes、MoveOwn 按值转移改变 Rust
  片段字节。三族共享该字段——旧版本（/1）片段在任意族**永不命中**，一次性
  miss 重产（三代实测：/1 铸 store→/2 重开全 miss 重产→第三代全 hit）；
  按裁定登记而非清 store。collections 打印字节未变（20/20 金样三文件字节
  核对）仅键随轴变化。
- **external modes 依赖**：片段 external 调用消费 `ExternalFn.modes`（受检
  事实），被调方 mode 变化经语义指纹失效调用方片段；同一程序整模块与单
  函数片段发射字节等价（fn_main 段逐字节）。
- **Rust 恢复首编辑边界**：Rust record-text 无 AXP 等价锚——`warm_edit`
  /`warm_edit_method`（片段合同）为既有增量路（未改函数生产 0、片段字节
  恒等）；restore→首编辑无预热链待 PLAN-929 接缝，未交则验收 pending。

## AXC3/AXP3：C 集合通道的增量与纯恢复（PLAN-926）

- 片段层 **AXC3**（`a2926/axc-c-coll-fn/v1`）：C 集合片段自有命名空间（与
  AXR3 Rust 集合及 AXC1/2 互斥；runtime_sha256 区分）。元素名义身份、variant
  名/臂序/payload 图、param mode/callee 接口入 AXS3 语义键；声明（record 布局/
  臂序/mode）变更冷再入重建**真实读者**、非读者 store 命中、等价表内容全命中。
- 锚 **AXP3**（`a2926/axp-coll-source-anchor/v1`）：seed 携带（名义类型表
  records+variants+lists），C 后端集合族恢复（Rust 集合目标保持冻结无锚）。
  纯恢复 P0（无 parse/lower/print/persist）+R>0（lookup/binding/payload/fileset
  验证）盲重装配字节恒等；恢复后下一 body 编辑 env_rebuilds+1、编辑单元恰
  lower/print+1、锚重绑；坏编辑不发布（旧锚可恢复）；readonly 拒锚；lease+GC
  存活；C 标量锚不服务集合恢复（profile 键隔离）。
- 暖授权：`warm_edit_ranged` base=会话当前全源、range 恰单声明、fragment 同名；
  manifest 绑定被替换**全源** sha256。A→B→A 历史字节复用（store 命中、零生产）。
- 正式采样（实测 2026-11-04）：904 冻结负载四态×30、512 修订（256 left_pad+
  256 right_pad）逐修订 cl 预认证对闭式 oracle；数据见 PLAN-926 §9 T-10 收据。

## PLAN-929：AXD1 声明面环境层与恢复后首编辑

| `AXD1` a2x-decl-env | 冷装发布的 CheckedEnv 声明面（`core-decl-env/1`；签名/mode/receiver/记录/VT 池；key=owner/profile/source_sha/backend/entry） | ObjectManifest | FromUserInputs |

- `ensure_env` 整源 parse+resolve 路径退役：恢复（P0）→首个 ranged/
  method 编辑经 AXD1 清单装配 env（深验绑定；清单缺失=具名拒绝
  `a2x.decl-env`，不静默整源回退）；whole 轴 0、parse 只计片段面；
  `env_rebuilds` 如实 +1（R 轴装配）。
- 片段生产者六处全部经统一前门桥；925/926 provider/runtime 面零触碰
  （TargetProvider 之后全 SemanticView）。

## Rust 打印配方失效（PLAN-941）

Rust provider 既有 `print_version` 轴从 `a2x-print/2` 升为
`a2x-print/3`：record 字段写现在为值根生成正确的可变绑定。AXR1/2/3
共享该打印版本轴；语义层键与独立 C 片段键不因这次修复改变。
旧 `/2` 缓存仍可存在，但当前 provider 不得命中其中的错误 Rust 片段；
第一次打开须按新键生产，后续同身份打开可完整命中并保持装配字节。
不通过删除旧 store、改旧证据或伪造恢复计数取得通过。真实 store
三代回归包括旧缺陷片段的 rustc E0594、新片段实际值19及再次命中。

## PLAN-938 增量：foreign reader 恢复/文本输出准入（reviewed）

- 恢复首编辑/失败修复/原子发布沿 source session
  （abi/source-foreign-session/1）；AXCF1 fragment cache 命中不产生输出账目。
- C fragment/fileset 文本（generated.c/foreign_imports.h/manifest）在共享请求
  输出账户精确准入；runtime header/状态 runtime 字节相同副本仅记活内存。

## PLAN-939 增量：resolver-owned 常驻页路线（r3）

- 资源源（foreign-only supply lane 的 body）在 Frontend 目录中以带
  `resolver-owned` 标记的常驻页存在——页**不携带类型事实**（零伪造），
  仅记录 owner 身份、body 文本摘要与 foreign 环境身份；冷/暖编译均真实
  重解析，P/R 计数如实。warm body-edit P90 门以"含真实 resolve 的全链
  时间"计。
- 失效轴：layout/mode/provider/contract 变更经 digest 键精确失效。
- **具名后继（非现状）**：事实面 text-record typed-leaf 增量（solver 输入
  投影+页编码携带）使资源体获得真 R 复用——见 KNOWN-DEBT 与 939 §10。
