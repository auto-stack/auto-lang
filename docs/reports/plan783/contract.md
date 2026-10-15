# PLAN-783 产物契约登记(A00–A12 实际覆盖面,T-02)

权威实现:`experimental/ac-artifact-probe/src/schema.rs`(`LAYERS`/`SAMPLES`
为唯一登记源);本文件是人读摘要。实际 codec 面 = **A00/A04/A07/A09/A10**,
其余层为 ProbeSeam(A03)或 Future 接缝(生产/消费经 `require_producible`
受控拒绝——`admission_unknown_schema_and_future_layers_refused`)。

| 层 | 覆盖 | 生产者 | 查询键成分 | 引用类别 | 驻留/回收 |
|---|---|---|---|---|---|
| A00 输入 | **Codec** | 会话 | source-bytes+逻辑名(域分离;**不含路径**) | provenance | 源字节为受管不可变拷贝;用户源不归仓所有 |
| A01 语法表 | Future | 语言定义管线 | grammar-id/revision | required-data | 按代际共享 |
| A02 词法 | Future | lexer | snapshot+scanner-abi | required-data | 冷 |
| A03 CST | ProbeSeam | 探针造数 | snapshot+grammar+parser-ver | required-data/provenance | 不可变子树页;全树驻留永非命中前提 |
| A04 AST 摘要 | **Codec** | 真 AST 投影 | profile+schema+snapshot+frontend-ver | provenance | **内容=签名投影(int/i32 归一,无 span)**;body 编辑共享 chunk |
| A05 宏 | Future | 展开管线 | 展开输入+env | required-data | 副作用不可控即禁共享 |
| A06 语义/接口 | Future | resolve/typecheck | profile+summary-ids | required-data | 接口查询不装载函数体 |
| A07 HIR | **Codec** | 真 binder | hir-domain+profile+caps+snapshot+codec-ver | provenance(→A00) | 恢复 Bundle **不带信任**,一律重过真 verify |
| A08 pass | Future | pass 管线 | hir-artifact+pass-ver | required-data | 只缓存已实现 pass |
| A09 对象 | **Codec** | 真 lower_object | **hir 查询键哈希**+target+backend+entry | provenance(→A07,**非 required**) | 独立可链接;上游 HIR 冷/删不影响 |
| A10 链接/发布 | **Codec** | 真 staged link | object-key+toolchain | required(→A09)+provenance(→A07) | 回执内容=链接输入纯函数;交付 exe 是用户产物,不在 GC 删除集 |
| A11 诊断 | Future | 各阶段 | stage-key+revision | provenance | 有界 |
| A12 热重载 | Future | runtime | generation+state-schema | required-data | 非普通 LRU |

## 身份/引用规则(实测钉面)

- **域分离哈希**:blake3(tag+长度前缀成分);`identity_domain_separation` 断言
  (ab,c)≠(a,bc) 防拼接攻击。
- **QueryKey↔ArtifactId↔ChunkId 分离**:同内容不同键→不同 artifact 行、
  **同 chunk**(跨工程内容共享,V-01 块数不变);artifact 内容=层+键+chunk+边。
- **LocalRef/ExternalRef**:owner/类别/界外/错 schema 四类拒绝
  (`external_ref_validation_rejects_bad_owner_and_bounds`)。
- **对象键以 HIR 查询键哈希为成分**(非 artifact id):缺源对象命中必须在
  不编译前提下可算;body-only 编辑经 snapshot 链失效对象(V-01/05)。
- **provenance ≠ required-data**:GC 只沿 required 闭包保活;A09→A07 是
  provenance——HIR 回收后对象仍链接运行(V-06)。
- **同键异内容 = Conflict 报告**,非 last-writer-wins(V-07);僵尸行(块已删)
  允许重发布替换(死行非活主张)。

## 关键"非法样例"钉面(SAMPLES 摘选,均有测试)

- 快照以绝对路径为内容键 / 摘要含 span / 持久 trusted 构造 Checked /
  对象命中后重全编译只改统计 / body-only 复用旧对象 / GC 删用户交付 exe /
  runtime 代际走普通 LRU——全部被对应 oracle 断言拒绝(见 §6 V 表)。
