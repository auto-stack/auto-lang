# auto-a2x 增量与 cache

> 状态：active（AX-A 形态，`a2x-store` crate；PLAN-892 T-04/§5.4）。

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
