# auto-semantics 语义合同（身份/版本/页种类准入）

> **Status**: experimental（PLAN-796 冻结；现行规则以 D21–D24 为准，历史
> 执行日记不作为规范）。登记实现 = `experimental/ac-semantics/contract/src/
> {identity,codec,metadata}.rs`。

## 名义身份

- `LogicalId` = 域哈希（package, module, kind 轴, name）——与注册顺序、
  进程重启、hash 迭代序无关；同名跨包/跨模块不合并。
- `DeclKind`：`Fn=1`（顶层函数）、`Record=2`（名义类型）、`Method=3`
  （record 方法）。**方法身份含所属 record**：`method_logical_id` 把
  owner 编入 kind 轴（`method::<owner>`）——同模块 `A.get`/`B.get`/
  顶层 `get` 三向不碰撞；既有 Fn/Record id 字节不变。
- `DeclRef`（持久化名义引用）携带 logical/package/module/kind/name/
  `owner`（仅 Method）/content。`verify()` 重导 logical：ownerless 方法、
  带 owner 的 Fn/Record、篡改 owner 一律拒绝。

## 内容版本与版本轴

- `ContentVersion` = canonical 语义内容哈希（签名/字段序/body recipe；
  位置与空白不入）——位置漂移不换版本，签名/字段/主体编辑换版本。
- 页 schema `ac796/sema-page/v2`、codec 2（方法 DeclRef tag3+owner）：
  **payload 语义变更必须随版本轴换键**（旧 v1 页不可达，无同键异值）。
- **逐查询语义版本轴（D24 规则）**：凡改某查询页头/载荷语义而不动整体
  schema/codec，必须为该键补显式语义轴（FnSig 键 `hdr-sem=
  fn-sig/sig-only/v1`；ModuleIndex 由 `index-content-fp` 全量换轴）。

## 页种类覆盖映射（payload → 键/版本 → 读回准入 → 反控）

| 页种类 | payload 字段 | 键/版本轴 | 读回准入 |
|---|---|---|---|
| RecordMeta | shape（decl+字段）+ methods（decl+签名）+组有效版本 | field-set-fp + ref-versions-fp + unit-topology-fp + schema | `admit_shape_refs`（decl 与字段 ref 全量当前）+ 方法结构门（kind=Method 且 owner=本页 record）+ `admit_method_ref`（verify/当前包模块/owner 下当前声明/当前签名版本）+ 组有效版本比对 |
| RecGroup | 成员 shapes + 组内容版本 | group 键（members+algo+schema+producer+group-content-fp） | 成员 `admit_shape_refs` + 组有效版本 + 当前成员集比对 |
| FnMeta | fn decl + 签名投影 | fn-sig-fp + **hdr-sem**（签名级头语义）+ schema | decl 全量名义准入（verify/当前包模块/顶层 Fn 无 owner/当前声明存在/content=当前签名 canonical）+ 签名级头比对 |
| ModuleIndex | fn 候选 + record 候选 + decl 签名版本表 | **index-content-fp**（全量 payload 字节）+ schema | 字段级准入：包/模块、两组候选集、decl_versions 逐项对当前 input |
| InferBody/CachedResult | 推导事实 + 依赖 manifest | fn-sig-fp + fn-body-fp + called-sigs-fp + src-fp + schema | 页头 target_content（签名+body）+ facts 归属 + manifest 新鲜分类（Fresh/ProvenanceStale/Stale） |

**读回准入通则**：不以页头哈希等价替代未检查的嵌入事实——每个嵌入名义
ref 都须当前（包/模块/存在性/内容版本）；外属、外包、除名、旧版本一律
受控拒绝，只有全当前页可命中。

## 候选/负依赖

- 新增成员/原不存在名字被定义 ⇒ 负依赖失效（候选集指纹入键）；
  未读 body 的签名查询不误失效（FnSig 头=签名级）。
- 来源投影：`src-fp` 入 InferBody 键（位置漂移=新键新算，spans 恒当前）；
  反射读 doc/source 的相关变化经 manifest source-map 失效（fail-closed）。

## CT/泛型/runtime 接缝（模型面）

- `CTRequest/CTResult`、`InstanceKey`、`RuntimeProjectionPolicy` 仅冻结
  Schema 与准入：portable 值/类型描述可入，地址/宿主 handle/Unknown/
  未检查可执行输入拒绝；模型计数标 `model_compute`，真 AVM 才允许
  `vm_execute`。递归准入（任意深度 Unknown）与结果 marking 三重校验。
- 797 只读消费：类型描述 codec（unit/i32/u32/i64/u64/f64/bool/char/
  byte buffer/opaque/函数/名义 record 间接引用 + view/mut/owned）；
  ABI 布局与调用规范归 797 独立拥有。

## 覆盖矩阵（可审阅接口）

`contract/src/metadata.rs` `CATEGORY_MATRIX` 是 meta 类别/owner/schema/
消费者的唯一登记源；`matrix_invariants` 强校验：无重复 id、Future 行有
owner、页载体类别必须登记**当前**页 schema（v1/v2 漂移在库级拦截）。

## PLAN-817：真实执行域指针（只追加，模型合同不变）

796 模型合同零修改（`CtResult::admit_as_model` 拒 vm_execute；`ModelCt`/模型计数
`model_compute` 不能铸造执行结果——模型 producer/伪 vm 标记经 cte_admit_/cte_freeze_
反控钉死为受控拒绝）。真实执行走独立版本化 envelope（`ac817/cte-executed/v1`，
producer=`plan817/cte-probe` 绑定真实 AV 成功），契约见
[auto-cte/scalar-query-profile.md](../auto-cte/scalar-query-profile.md) 与
[auto-cte/frozen-cache.md](../auto-cte/frozen-cache.md)；consumer 回灌消费 796
`InferSession::infer_function`/`SemaInput` 事实源，CtRef 属新 recipe IR 不进求解器。
