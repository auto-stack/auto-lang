# AAC 精确摘要、编译器 codec 与支持边界（PLAN-1056 交付稿）

> **Status**: PLAN-1056 work 交付稿（review 通过后由 merge 正式沉淀；本轮不宣称 reviewed）。
> 本 profile 描述 AAC（Auto 编译器主体）实际消费的**精确摘要**、**身份分帧**与
> **编译器 codec** 的已验证行为、所有权、版本与支持边界。
> 实现单源：`experimental/ace/aac/support/{digest,codec,atom_data}.at`（包身份
> `aac-support` 1.0.0，源文本即身份）；执行端证据：
> `experimental/ace/tests/aac-support/`（固定组 `ace-aac`）。

## 1. 为什么需要本 profile

Design 59 §9 与 CS-205/CS-209 指出：旧能力（975 的 JSON、972 的默认 hash）**不能**
被泛称为"现 Rust 工件身份已可复现"——972 的 `std.hash.codepoint_hash` /
`std.bytes.byte_hash` 是多项式混合（非密码学摘要），975 的 JSON 不是二进制帧。
AAC 需要的是与 Rust 工件层**逐字节一致**的：

1. `blake3` 非密钥 32 字节摘要（工件身份、`ace_artifacts::identity` 的分帧）；
2. FIPS 180-4 SHA-256（A2X 证书/回执内容哈希同族，`ace_a2x_core::sha256`）；
3. 身份分帧（`HASH_ALGO`、域标签、u64 小端长度前缀）；
4. 工件帧 `aac.frame/1`（与 `ace_artifacts::codec::frame_chunk` 逐字节兼容）；
5. 编译器 manifest DTO `aac.manifest/1` 与 canonical Atom 数据 reader/writer。

## 2. 包与所有权

| 轴 | 值 | 说明 |
|---|---|---|
| 包名/版本 | `aac-support` / `1.0.0` | 与公共 `std`（`stdlib/ace/std/`、`stdlib/ace/asm/`）**分离**：本包是 AAC 应用层支撑，不是公共标准库面 |
| 模块 | `digest`、`codec`、`atom_data` | 源文件内容即身份（`include_str!` 嵌入，无仓外来源） |
| 语言面 | `packages-aggregates-ct`（解析）/ `Aggregates`（语义） | 单一 Auto 源；AV 执行，Gen0（Rust 前端）编译被测 Auto，算法本身 Auto 执行 |
| 不变量 | 无公共 `std` 操作新增 | 因此不触发 std 五端覆盖合同；M2-A 冻结 std 切片身份不变 |

**为什么不是公共 `std` 操作**：BLAKE3/SHA-256 迄今只有 AAC 一个真实消费者；把
它们发布为公共 std 操作会立刻产生 AV/mn/A2R/A2C/A2PY 五端与 `stdlib/contracts`
的义务，而没有任何公共消费者。Design 56 的覆盖合同只在"新增公共操作"时适用；
本包因此具名为 AAC 支撑包，并由本 profile 记录其边界与后继升级路径（见 §8）。

## 3. 精确摘要（`digest`）

| 公共拼写 | 合同 | 已验证参考 |
|---|---|---|
| `sha256_hex(data Str) Str` | FIPS 180-4，UTF-8 字节精确，返回 64 位小写十六进制 | 进程内独立 `sha2` crate（未被验的对照：K/IV 常量取自 A2X 产品参考 `ace_a2x_core::sha256`，算法为独立 Auto 实现） |
| `sha256_hex_bytes(bs List<int>) Str` | 同上，输入=精确字节向量 | 同上 |
| `sha256_hex_parts(parts List<Str>) Str` | 流式：同一 payload 任意切分与一次性哈希结果相同（真实增量路径：尾缓冲 + 块排空，不是重新拼接）；`parts` 被消费（`remove(0)`） | 同上 |
| `blake3_hex(data Str) Str` / `blake3_hex_bytes` | BLAKE3 非密钥 256 位，64 位小写十六进制 | `blake3` crate（unkeyed `hash`） |
| `blake3_bytes(bs List<int>) List<int>` | 原始 32 字节摘要（帧层需要 raw digest，不是十六进制） | 同上 |
| `hex_bytes` / `hex_words` | 摘要/字节的可观察表示 | — |

**字节精确性契约**：
- 输入是精确字节序列：内嵌 NUL 是内容不是终止符；无 C 字符串截断；
- 32 位字在**受检 i64** 中计算，每步 add 显式模 2^32（`wrap32`），存储为
  `List<int>` 的二进制补码 i32 位型（`uval`/`sv` 双向无损）——任何端不得以不同
  方式回绕或陷阱；位算 `^ & |` 是全域操作，移位用除法/乘法表达（该面没有 `<<`/`>>`
  运算符）；
- 迭代是有界且**迭代式**的：AV 执行器按活跃槽数据计费，逐元素递归累加器呈二次
  计费（实测：1 KiB BLAKE3 的递归走法触顶 8 MiB 槽预算，迭代走法远低于上限）。

**命名边界（本切片不做）**：
- BLAKE3 的**增量/流式**接口（`update`/`finalize` 跨块状态机）未提供：AAC 当前消费者
  （源文本、工件字节）在内存中一次性可得；BLAKE3 的内部分块路径由 1/64/65/128 字节
  边界向量加 **2049 字节（三 chunk，触发二叉树合并）** 向量对参考实现验证；单 chunk 满
  1024 字节的独立向量未单列（同一 chunk 处理路径已由上述覆盖）。SHA-256 已提供真实
  增量路径（§3）。
- 密钥模式（keyed / derive_key）与扩展输出长度（>32 字节）不在本切片。
- 性能：纯 Auto BLAKE3 约 1M 指令步/KiB；AV 默认 1M 步不足，driver 以显式记录的
  64M 步预算运行（`AV_MAX_STEPS`）。加速是后继增量，不是本 profile 的合同。

## 4. 身份分帧（`identity_*`）

与 `ace_artifacts::identity` 逐字节一致：

```
blake3( "blake3-1.5" ‖ 0x00 ‖ family ‖ 0x00 ‖ { u64le(part_len) ‖ part }* )
```

| 公共拼写 | 说明 |
|---|---|
| `identity_hex(family Str, part Str) Str` | 单部件域哈希 |
| `identity_hex2(family Str, p0 Str, p1 Str) Str` | 双部件域哈希；长度分帧区分 `[ab,c]` 与 `[a,bc]` |
| `identity_frame_hex(family, p0, p1) Str` | 分帧前像字节（可观察 wire，不止摘要） |

id 文本前缀（`snap`/`art`/`chk`）与 `QueryKey` 的 `qk/<layer>` 族由消费者（1052）
按 `identity.rs` 构造；本包提供的是摘要与分帧原语，不替代工件层身份类型。

## 5. 工件帧（`aac.frame/1`，与 `codec.rs::frame_chunk` 逐字节兼容）

| 偏移 | 内容 |
|---|---|
| 0..4 | magic `0x5A4B3341` 小端（"A3KZ"） |
| 4..6 | framing version u16 LE = 1 |
| 6 | chunk kind u8（0..10，唯一 kind 表） |
| 7 | reserved u8，写 0，解码**从不校验** |
| 8..16 | payload 长度 u64 LE |
| 16..48 | payload 的 **raw** blake3-256 摘要（不分帧） |
| 48.. | payload |

- 解码强制**精确长度**与摘要复验；尾随字节是错误，绝不静默忽略；
- 结果字节的第 0 位是结果码（0=ok），因此"分类"本身是可观察证据。

## 6. manifest DTO（`aac.manifest/1`）

```
u16 version = 1 | u8 schema_tag = 1 | u8 reserved = 0
str layer | u64le anchor(i64 二进制补码，8 字节原样) | u32 pair_count
  (str key, str value) * pair_count
u32 edge_count | (str id, str refkind) * edge_count
其中 str = u32le(len) ‖ ASCII 字节
```

- **规范顺序**：pair 按 (key, value) 升序（key 相同则按 value，value 相同即同一 pair）；
  edge 按 (id, refkind) 升序。编码器排序，并在排序后按**与解码器相同的规则**拒绝
  **完全相同的重复 pair**（12）——编码器因此不可能产出自身解码器拒绝的字节；同 key
  不同 value 是合法规范序（编码成功且解码回 0）。解码器独立拒绝非规范顺序（13）与
  完全相同的重复 pair（12）；排序不确定性不会以 last-writer-wins 方式隐藏；
- **wire 不携带可信 Checked 能力**：格式里根本没有信任字段，恢复出的 DTO 永远不能
  冒充已验证对象；语义准入仍由 1047/1053 的重验路径负责；
- **预分配准入**：所有 count/长度在分配/迭代之前先按"实际剩余字节"校验
  （`count * 8 > remaining` 即拒绝），攻击性长度不会导致分配；
- `anchor` 的 8 字节原样搬运（解码端按定宽字段复制，不做值层重编码），因此
  `i64::MIN`/`i64::MAX` 不受重编码扰动；写入路径为 `codec.push_i64le(v i64)`
  （二进制补码 u64 小端，见 §7）；
- 该 DTO **不宣称**与 `artifacts` 内部的 `artifact_identity` 哈希构造逐字节相同：
  后者是哈希而非序列化；本 DTO 的排序规则是显式冻结的。

## 7. 结果码（两个 codec 共用）

| 码 | 含义 | 码 | 含义 |
|---|---|---|---|
| 0 | ok | 8 | checksum-mismatch |
| 1 | truncated-header | 9 | count-overflow |
| 2 | bad-magic | 10 | unsupported-version |
| 3 | unsupported-framing | 11 | unknown-schema |
| 4 | unknown-kind | 12 | duplicate-key |
| 5 | too-large | 13 | non-canonical-order |
| 6 | truncated-payload | 14 | non-ascii-bytes |
| 7 | trailing-bytes | 15 | empty-field |

admission 上限：`default_max_payload() = 4 MiB`，`default_max_items() = 65536`。
两者都是**调用方可传**的显式准入决定：帧格式本身仍允许 64 MiB（`MAX_CHUNK`），
本切片只是因为 AV 执行器无法物化更大的列表而取了更窄的准入边界——不是静默截断，
也不是把格式合法性窄化。

## 8. canonical Atom 数据 codec（`atom_data`）

纯数据 reader/writer：接受合法文档、产出**规范字节**、按结构化规则拒绝错误，并且
**绝不**构造任何"已验证"对象——语义拒绝与效果保真明确留给 1047/1053。两个接受模式
共用同一个 writer：

1. **无 schema 结构模式** `canonicalize(text Str) List<int>`：通用 Atom 数据，**没有**
   tag/字段白名单。唯一的内容检查是重复字段（8），外加语法类拒绝：空（1）、非法字符
   （2）、未闭合字符串（3）、非法转义（4）、非法数字（5）、不平衡/准入（6）、尾随
   token（7）。
2. **真实 schema 模式** `canonicalize_with_schema(schema_text Str, doc_text Str) List<int>`：
   先把**真实 descriptor schema 文档**（同一 Atom 方言，
   `experimental/ace/crates/core/schema/core-i32.atom`）解析成 tag/字段表，再校验文档：
   根 tag 必须是 `hir`（否则 10）、每个节点 tag 必须是 schema 声明过的节点（10）、每个
   字段名必须属于该 tag 的允许集合（9），同节点内重复字段仍是 8——**内联节点**
   （`{ k: 1, k: 2 }`）也一样，用与无 schema 模式完全相同的规则（`dup_all` 全 arena
   后置检查；schema 遍历只看带 tag 的节点，内联节点是字段值而不入栈）。

字节入口 `canonicalize_bytes(bs List<int>)` 与
`canonicalize_bytes_with_schema(schema_text Str, doc_bs List<int>)` 是同一两个模式的
"已经编码的字节"入口，使 `canonicalize(canonicalize(x)) == canonicalize(x)` 可以在没有
解码步骤的情况下表达（其长度准入按字节计为 1048576，四倍于标量上限，因此文本入口准入的
文档必然通过；结构检查完全相同）。

schema 表的读取规则：`nodes { node TAG { … } }` 下每个 `node` 的第一个 primary 是 tag 名；
允许字段名 = 其 `fields { … }` 体的条目名 + 其 `slots: [ … ]` 按真实 descriptor 映射得到的
**字段拼写**；`containers { name: [tags] }` 的条目名**只注册为节点 tag**（分组子节点），
**不**成为父节点的允许字段。`slots:` 的条目是**位序槽种类**，
不是字段名；真实 descriptor（`crates/core/src/descriptor.rs:815-822`）只按下列规则为其
接纳具名字段：

| 槽条目（按位置） | 接纳的字段拼写 |
|---|---|
| 第一个 `Id` | `id`（`Id` 不在首位时无对应字段） |
| `TypeUse`、`TypeRef` | `type` |
| 其他（枚举种类，如 `TypeKind`/`ExprKind`/`PlaceKind`） | `kind` |

因此 `type t_i32 i32` 的具名写法是 `type(id: "t", kind: i32)`；把标签字面量本身
（`Id`/`TypeKind`）当字段名会**双向反向**：拒绝真实 binder 接受的具名写法，又接纳它
以 `bind.unknown-field` 拒绝的 `type(Id: …)`。**container 名本身是合法节点 tag**（它就是
承载这些孩子的分组节点，如 `types { type … }`、`bodies { … }`），但**不是**父节点的字段：
真实 descriptor 把 container 放在 `spec.containers`（`crates/core/src/descriptor.rs:446-449`），
具名字段只对 `spec.fields` + 槽映射校验（`:815-833`），故 `module(types: …)` 是
`bind.unknown-field`（9），而 `module { types { … } }` 合法（0）。
`statements:`/`inline:` 在该方言里是**子/内联 tag 声明**而不是字段名，故不进入字段集合。
字段/孩子的**归属关系**（`containers` 成员、`branches` require/forbid、内联 tag 位置）
不在本层检查，归 1047/1053 与其余语义绑定。

**准入规则与本车道的可执行包络（实测，非承诺）**：文本 >262144 标量 → 6（在词法之前），
嵌套 >64 层 → 6；边界是"64 个嵌套节点合法、65 个拒绝"，而**字段值**位于其宿主节点深度 +1，
所以叶子带字段值的 64 层链在被算作值路径深度 65 时拒绝（64 层节点本身仍合法）。这两个
数字由固定组用例钉住（`atom_admission_rules` / `atom_admission_depth`）。同时，本 profile
记录的准入上限**宽于本执行器可物化的包络**：
- `text_bytes` 把整段文本转成**一个** `List<int>`，而冻结的 collections 合同把单表上限钉在
  65536 元素（`av-core::contract::MAX_LIST_ELEMS`，`av.resource.list-elems`）——65537 标量
  的文档由执行器以资源拒绝（不是 codec 结果码）；
- 该转换按 512 标量分块、每块仍重解析整个源缓冲（O(n²/512)），故 65536 标量的文档还会
  撞到槽预算，实际可在用例预算内规范化的文档约数 KiB（本切片用例最大 2048 标量合法近邻）。
这两条是**执行器包络**，不是把 codec 的准入规则收窄；真正的支持上限由本 profile 的规则
决定，包络补齐（分块词法/流式 writer）属后继增量。

### 8.1 writer 布局规则（冻结）

- 节点头 = `tag` + **唯一**一对括号组：`(primary, primary, name: value, name: value)`，
  组内一律 `", "` 分隔，primary 在前——即真实 writer 的
  `hir(schema: "…", revision: 1u32, profile: "…")` 形状；头与体之间恰好一个空格；
- **字段永远写在组里**：源文档 `{ body }` 里的字段会被上提到组中，因此"只在字段书写
  位置不同"的两份文档规范化结果完全相同——输出是**数据的函数**，不是输入布局的函数；
- 体只放**子节点**，每行一个，缩进 4 空格/层，源顺序保持（不重排）；没有子节点的节点
  一律 `tag(...) {}` 单行（源体里只有字段的节点同样如此）；
- 收尾 `}` 单独一行、位于该节点深度；每个值/节点行、包括根收尾行都以 `\n` 结束；
- 列表值 `[a, b, c]`、内联节点 `{ k: v, k: v }` 都在一行；字符串按同样四条转义重引号，
  数字按书写逐字重发。

### 8.2 规范性与幂等合同

- 真实 HIR 夹具（`experimental/ace/crates/core/fixtures/native/add-2-3.atom`）在两个模式
  下都必须被接受（状态 0）；
- 两个模式对同一文档产出的字节相同，且把该字节再喂回对应模式（字节入口）得到**逐字节
  相同**的输出：`canonicalize(canonicalize(x)) == canonicalize(x)`；
- **不要求**输出等于参考 writer 的逐构造成字节布局——那是 1047/1053 的具名交接；
- 反例纪律：不得用"接受一切"绕过重复字段检查，也不得让"规范形式"依赖输入布局。

## 9. 与后继 Plan 的交接

- **1052（增量 driver 与工件）**：把 `aac-support` 的摘要/帧/manifest 版本
  （`aac.frame/1`、`aac.manifest/1`、`blake3-1.5` 分帧）作为工件身份与恢复准入的
  单一事实源；store 物理布局仍由 1044/1052 决定；
- **1045/1051**：宿主/ABI 侧需要摘要时的装配路径就是本包（不另建第二份实现）；
- **1047/1053**：canonical Atom/HIR 文本的完整语义绑定（type/DefRef 解析、Checked
  构造）由语义 Plan 负责；本包只保证纯数据 roundtrip 与结构化拒绝；
- **公共 std 化（未决）**：若将来出现公共消费者，升级路径是"本包算法上移为标准库
  单源 + 按 Design 56 覆盖合同跑 AV/mn/A2R/A2C/A2PY 五端"，届时应新建或修订 spec
  delta，而不是让本 profile 静默扩面。

## 9.1 阶段合同（SD-AC-STAGED 载体）

本包执行 Design 59 的**阶段**规则，明写如下（这是 SD-AC-STAGED 的规范载体）：

- A 侧（M2-A）已交能力是**单向消费**：本包只消费已正式落主的 A 能力
  （`stdlib/ace/asm/bytes.at` 的 `byte_of`/`utf8_len`、共享 grammar 与
  `packages-aggregates-ct` 车道）；不新增、不改写 A 的冻结切片。
- A 侧**具名 AC 差额**可以在 B 期间与 AAC 实证互相交接：本包当前**没有**任何缺口依赖，
  即所有被消费的操作都已交付；若将来某切片依赖尚未交付的 AC 操作，必须先具名登记该
  切片阻塞，不得 fallback 到旧 VM/私有临时实现。
- CL（完整语言）行不因 A 期能力有限而删改；本包的 profile 只描述 digest/codec 支撑面，
  不冒领全 CL、Gen1–3 或 M2 最终通过。
- 阶段通过（本 profile 的 V-01..V-06）不等于完整 AAC 或 M2 三端签章；完整验收仍归
  1053/1054/1068。
## 10. 验证（本交付稿的真实命令与结果）

固定组 `ace-aac`（`experimental/ace` workspace，`-p ace-aac-support-conformance --lib --tests
--config-file ../../.config/nextest-full.toml`）：**25 例**，`cargo t ace-aac` → `PASS: ace-aac`
（`executed=25`、`gate_passed=true`、`max_parallel_tests_observed=4`）。

- **组级限流**：用例都在 av-core 的 AV 执行器上编译并运行整包，无限制并行时单例墙钟被同族
  争用推到 >10 s；`.config/nextest-full.toml` 的 `[test-groups] ace-aac = { max-threads = 4 }`
  加 `package(ace-aac-support-conformance)` override 把**组**限在 4 路（不再用与 fixed-tests
  冲突的 `-j` 参数）。组模式实测 25 例全部 ≤10 s：空闲窗口最大 8.05 s、总 26.07 s；串行
  `-j 1` 口径 25 例全部 ≤10 s（最大 9.27 s，总 110.96 s），作例内成本说明。外部计划构建会把每例
  墙钟整体抬高（同负载 A/B：4 路 max 12.9-15.2 s、2 路 max 17.4-20.3 s），那是主机争用而非本包
  成本，故组限流只去掉本组自身争用、按总墙钟取 4 路。
- V-01/V-02/V-03/V-04/V-06 的观测值与计划 §7 证据一致（本 profile §3/§6/§8 的每条规则都有
  固定组用例：摘要向量、帧 55 组、manifest 12/13/7/10/11/14/15 + 编码器同键不同值 0、
  atom 准入 6 与合法近邻、schema 槽位→字段映射、内联节点重复 8、真实 HIR 双模式定点）。
- V-05 的"全部覆盖腿"义务**不触发**（未新增公共 std 操作，见 §2）。
