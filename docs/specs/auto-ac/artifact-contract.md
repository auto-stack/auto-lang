# auto-ac artifact contract（A00–A12 产物契约）

> **Status**: experimental（PLAN-783 探针冻结契约；实现面=探针,非生产）
> 来源：PLAN-783（[归档计划](../../plans/archive/783-ac-artifact-contract-probe.md)）;
> 登记实现= `experimental/ac-artifact-probe/src/schema.rs`（唯一登记源）,
> 人读对照 `docs/reports/plan783/contract.md`。关联 [artifact-probe.md](artifact-probe.md)。

统一编译各层产物的**语义模型、身份、引用类别与失效规则**（Design 40 目标态的
实验性冻结）。本契约描述探针已实测的边界;全部条目不扩大语言面、不新增生产入口。

## 层级登记（A00–A12）

| 层 | 覆盖 | 查询键成分 | 引用类别 | 驻留/再生 |
|---|---|---|---|---|
| A00 输入 | **codec** | 源字节+逻辑模块名（域分离;**路径不入内容键**） | provenance | 用户源不归仓所有;仓持不可变内容副本 |
| A01 语法表 | **codec**（787 起真实语法生产者） | grammar-id/revision+scanner-abi | required | 按代际共享 |
| A02 词法 | **codec**（787 起真实语法生产者） | snapshot+grammar-id+scanner-abi+lexer-ver | required（→A01+源页） | 冷 |
| A03 CST | **codec**（787 起真实语法生产者;规范形分页） | snapshot+grammar+parser-ver | required/provenance | 不可变子树页;全树驻留永非命中前提 |
| A04 AST 摘要 | **codec** | profile+schema+snapshot+frontend-ver | provenance | **内容=签名投影（int/i32 归一;无 span）**——body 编辑共享 chunk |
| A05 宏 | Future | 展开输入+env | required | 副作用不可控即禁共享 |
| A06 语义/接口 | **受限探针**（PLAN-796） | 语义查询键全轴（见 auto-semantics/query-cache） | required | 接口查询不装载函数体；仅 796 探针 producer，不冒充旧 producer 扩大 |
| A07 HIR | **codec** | hir-domain+profile+caps+snapshot+codec-ver | provenance(→A00) | 恢复 Bundle **不带信任**,一律重过真 verifier |
| A08 pass | Future | hir-artifact+pass-ver | required | 只缓存已实现 pass |
| A09 对象 | **codec** | **HIR 查询键哈希**+target+backend+entry | provenance(→A07,**非 required**) | 独立可链接;上游 HIR 冷/删不影响 |
| A10 链接/发布 | **codec** | object-key+toolchain | required(→A09)+provenance(→A07) | 回执内容=链接输入纯函数;交付 exe 是用户产物 |
| A11 诊断 | Future | stage-key+revision | provenance | 有界 |
| A12 热重载 | Future | generation+state-schema | required | 非普通 LRU |

实际 codec 面 = **A00/A04/A07/A09/A10**（core+scalar 受限可执行子集）+ 787 起
**A01/A02/A03 真实语法生产者**（`experimental/ac-syntax/adapters/artifacts`，
见下方语法产物节）;其余层为 schema 接缝 + 合法/非法描述样例（生产/消费经
`require_producible` 受控拒绝）。

## 语法产物接入（PLAN-787，A01/A02/A03）

真实生产者 = `experimental/ac-syntax/adapters/artifacts`（加法扩展;旧 kind
字节与拒绝面不变）：

- **A01 `SyntaxGrammar`**：语法程序 canonical 二进制；装载身份重验（内容
  hash 与键 grammar-id 不符=受控 miss/错误）；可按（grammar-id, revision）
  从 store 装载——跨进程恢复不编译 .at 源。
- **A02 `TokenStream`**：token 流 + **完整 scanner checkpoint**（终态按末
  token 拼写重放还原真实 mid——嵌套注释深度/转义挂起/mode；恢复续扫与
  fresh 词法尾部全等）。键含 grammar 身份（跨 grammar 真读拒绝）；
  **A02 --required--> A01 + 源页**。源字节以内容寻址 **`SourcePage`** 持久化
  （快照 16KiB 页聚合 ≤256KiB store 源页，跨快照/工程共享），manifest 记
  源页表可流式重建原文。
- **A03 `CstPage`/`CstManifest`**：**持久化规范形**——页内容由树逻辑内容
  （根孩子文档序+先序 DFS+相对跨度/字段/标记）决定，与内存块分配历史无关；
  同一 snapshot 的 fresh/增量树页字节相同 → 同键 CAS 幂等复用；不可达节点
  不入页；键形历史稳定（布局演进必须引入布局版本分量）。目录分页 **2048
  条/页**（framed ≤512KiB）。按需页视图：重启小查询禁调 parser、视图 lease；
  **子树读取 admission 在真实入口验证整棵可达子树**（页内声明归属一致/
  绝对跨度≤源长/非 trivia 孩子⊆父 span、trivia 孩子右界⊆父 span/边字段在
  grammar 字段名表内/kind 在声明面内；decl_range 保持根级单点检查）。
- **范围查询坐标合同**：源覆盖窗口=outline name 字段绝对范围（不含前导
  trivia），origin 平移受校验（checked_sub 受控退出）；窗口驻留超限→受控
  ResourceExhausted，且**限额在物化之前判定**（驻留上界=limit+一 store 源页），
  绝不用空 name 冒充成功。
- **required 闭包**：put_cst 链 = manifest --required--> 数据页+目录页+A02
  （→A01+源页）；流式链 = manifest --required--> 数据页+目录页+源页+A01
  （流式无 A02 工件，直接闭包）。
- **发布仲裁（会话域）**：`published_revision` 仅存 store 会话内存、**不进
  内容工件**（同文本跨修订幂等复用；跨会话/重启 revision=0 是新文档发布，
  不是迟到任务）；原子序=先注册新根、成功后定点删除被替换旧根行——注册
  失败旧当前根完好保留。
- **五 kind 的公开产物读入口（按 key/按 id）全接入**：kind 解码以 codec
  帧层为单一事实源，store 元数据层委派之，无重复表漂移（未知字节仍拒绝）。

## 身份与引用规则（探针实测钉面）

- **身份三分** QueryKey / ArtifactId / ChunkId;blake3 域分离（长度前缀防拼接）。
- **恢复 HIR 不带信任**：磁盘 trusted 标志不能构造 `CheckedModule`;恢复路径一律
  重过本域真 verifier（`verify`/`verify_scalar`）——探针 V-02/V-04 实测依据。
- **对象键以 HIR 查询键哈希为成分**（非 artifact id）:对象命中查询必须在不编译
  前提下可计算（缺源恢复的必要条件）;body-only 编辑经 snapshot 链仍失效对象。
- **provenance ≠ required-data**：GC 只沿 required 闭包保活;A09→A07 是
  provenance——HIR 回收后对象仍链接运行（V-06 实测）。
- **同键异内容 = 不确定性报告**（Conflict 携带双 id）,永不 last-writer-wins;
  僵尸行（块已删）允许死行替换,重发布 upsert 回 live。
- **CAS 读取准入**：每次读取复验框架字节哈希 == 请求地址——自洽异址块整体
  替换被拒绝（R1 F-04/R2 反例实测）。

## 语义产物接入（PLAN-796，A06 受限探针）

A06 的**受限探针 producer** = `experimental/ac-semantics/probe`（PLAN-796；
[auto-semantics 项目卡](../auto-semantics/project.md)）：语义事实/接口页发布于
783 统一 store（层 `S796`、载体 `SnapshotRecord`、页 schema
`ac796/sema-page/v2`、codec 2）。仅此 producer 登记——旧 A06 Future 语义不因
本探针扩大；A05/A12 接缝维持 Future。消费方（HIR driver、797 ABI probe）按
[auto-semantics/semantic-contract.md](../auto-semantics/semantic-contract.md)
的名义身份与页准入只读消费；不绕过 consumer 身份校验，不把 HIR kind 冒充
语义值。
