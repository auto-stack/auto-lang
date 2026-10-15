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
| A01 语法表 | Future | grammar-id/revision | required | 按代际共享 |
| A02 词法 | Future | snapshot+scanner-abi | required | 冷 |
| A03 CST | 探针造数 | snapshot+grammar+parser-ver | required/provenance | 不可变子树页;全树驻留永非命中前提 |
| A04 AST 摘要 | **codec** | profile+schema+snapshot+frontend-ver | provenance | **内容=签名投影（int/i32 归一;无 span）**——body 编辑共享 chunk |
| A05 宏 | Future | 展开输入+env | required | 副作用不可控即禁共享 |
| A06 语义/接口 | Future | profile+summary-ids | required | 接口查询不装载函数体 |
| A07 HIR | **codec** | hir-domain+profile+caps+snapshot+codec-ver | provenance(→A00) | 恢复 Bundle **不带信任**,一律重过真 verifier |
| A08 pass | Future | hir-artifact+pass-ver | required | 只缓存已实现 pass |
| A09 对象 | **codec** | **HIR 查询键哈希**+target+backend+entry | provenance(→A07,**非 required**) | 独立可链接;上游 HIR 冷/删不影响 |
| A10 链接/发布 | **codec** | object-key+toolchain | required(→A09)+provenance(→A07) | 回执内容=链接输入纯函数;交付 exe 是用户产物 |
| A11 诊断 | Future | stage-key+revision | provenance | 有界 |
| A12 热重载 | Future | generation+state-schema | required | 非普通 LRU |

实际 codec 面 = **A00/A04/A07/A09/A10**（core+scalar 受限可执行子集）;其余层为
schema 接缝 + 合法/非法描述样例（生产/消费经 `require_producible` 受控拒绝）。

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
