# 复合源码单元投影与目录（collections/closures 增量面）

> **Status**: experimental（PLAN-928 交付；实现=`experimental/ace/crates/core/src/source/cst/units.rs`）
> 来源：PLAN-928；上游 [project.md](project.md)；前门/九 profile 见
> [source-frontdoor.md](source-frontdoor.md)；规则合同见
> [grammar-contract.md](grammar-contract.md)。

## 定位

Collections/Closures 两 profile 的语法单元目录与增量投影：整源冷构建
（`UnitCatalog::build`）把顶层 `fn`/`type`/`enum` 与每 owner 的 lambda
（源序 ordinal）密封为**不可变 `Rc` 语法体 + 内容指纹**（FNV-1a 64 身份
用，非密码学）。单 lambda 编辑只重解析/重投影目标单元；owner 编辑中同文
lambda 复用既有 Rc 体。**语法复用≠语义复用**：目录对上层携带
`semantic_recheck` 信号，捕获/类型/写性事实仍由 closures_fn/resolve 的
`decide_capture`/registry 唯一裁决。

## 单元身份（§逻辑键）

- 键 = module 指纹 + 逻辑 owner + kind（Fn/Lambda/TypeDecl/EnumDecl）+
  ordinal。CST 节点号/绝对 offset/单独 body hash 不能替代逻辑身份。
- **mint_span** = 铸造时坐标（跨编辑携带不变，身份面）；**当前 span** =
  目录本地包装字段（`SealedFn.span` 等），上方编辑后由漂移维护平移——
  **绝不原地 mutate 共享 `Rc` 载荷的 span**（pin 视图不受新修订污染）。
- 同文 lambda 异 owner 绝不合并（各自密封体）；同 owner 重复同文以
  ordinal 区分；插入/删除/重排使序-文错位（数量变或异文错位）＝具名
  非快路径（`owner-lambda-topology-changed`），整 owner 如实重产不得假 0；
  **零编辑重放**（同字节切片）＝全单元生产 0、身份 Rc 同一、无重验义务。

## 候选事务与 pin 生命周期

- `stage_owner_edit` → `commit_staged`：解析/投影/复用决策全在暂存面，
  目录/源/计量提交前零变异；坏候选=源码诊断（零变异），取消=丢弃。
- 候选携带 `base_revision`（staging 时源指纹）与目录 `instance` 身份
  （进程内，不入清单）：stale（基修订已过）或 foreign（异实例，同源
  文本也拒）提交＝typed `StaleForeign` 拒绝且零变异。
- `CatalogPin`：修订局部快照，多代共存、内容指纹可溯、跨修订不串用；
  最后 owner（pin/目录）释放后载荷真实回收（Rc 强计数可观察）。
- 变更事件：`OwnerSignatureChanged`/`DeclChanged{dependents}`（语法面
  名字引用）/`DeclSetChanged`/`LambdaTopologyChanged`——直路与暂存提交
  两面同判定。

## 无指针清单与 codec（929 消费面）

- `CatalogManifest`（schema `ac-source-units/1`）：profile（合同 slug，
  非 grammar id）+ caps 版本 + module/单元内容指纹 + mint/current span，
  无宿主指针/trusted 位，确定性排序。
- `encode_manifest`/`decode_manifest`：行式文本；**消费前校验**——坏
  schema/未知 profile slug/未知 kind/坏数字/坏 span/空行一律拒绝；
  数字合法但指纹被篡改者可解码、由与当次导出全等比对
  （`DecodedManifest::matches_manifest`）拒绝。全编译来源恢复 P0 与真实
  store 接线归 929；本目录不承诺恢复深验。

## 计量与预算

- `ProductionMeter`：produced（物化）/reused（Rc 复用）/discarded（失败
  尝试废弃物化单列）——复制/物化一个未改 body 即计该 body 真实生产，
  不藏在目录/簿记；`edit_lambda`/`edit_owner` 回执逐键列 produced/reused。
- 显式 `FrontBudget`（64MiB 档）贯穿冷构建与编辑；超限=typed
  `ResourceExhausted`（资源态非源码拒绝）；目录走查（collect/extract）
  为显式工作栈迭代，深链深度只受解析预算约束不爆线程栈。

## 验证锚

`ace-core::conformance::source_units_928(_t09)`（差分 vs 手写 oracle、
生产计量/Rc 身份反控、候选生命周期、漂移/mint 稳定、codec 伪造拒绝、
零编辑/重排/删除、大源有界）；文件化
`case-ace-source-composite-928`（复合面 parse+无损重建、profile 门负控、
值腿、同文 λ 异 owner 独立语义 oracle U1=1112）；固定组 `ace-source`。
