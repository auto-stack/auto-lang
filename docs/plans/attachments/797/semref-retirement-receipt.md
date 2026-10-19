# PLAN-797 临时语义快照退役收据（T-06，2026-10-19）

按计划 §5.6 与 [parallel-contract](../ac-foundations-20261016/parallel-contract.md)
"共同语义与测试接缝"的退役义务：D-796 满足后，797 必须切换到 PLAN-796
真实语义合同、删除本地重复模型、留收据，不允许双权威模型共存。

## 门证据

- D-796：`docs/plans/archive/796-ac-semantic-metadata-inference-probe.md`
  已归档（delivered 9/9），`docs/specs/auto-semantics/` 已落盘，
  master 含 796 合入链（`13367d127` 一线，2026-10-19 复核）。
- D-788：`docs/plans/archive/788-autodown-ac-native-turnaround.md` 已归档。

## 退役项

| 项 | 值 |
|---|---|
| 临时格式 | `autoabi-semref-snapshot/0`（status `temporary-797-local`） |
| 临时内容指纹 | blake3 `15f81fd02a19ff6b9b03b4f64d43d04d721fd524f3419b7cf3ae0f66926d32fa` |
| 引入/移除 | T-01（`b00100cbd`）引入；T-06 本次移除 |
| 移除内容 | `semref.rs` 的 `SNAPSHOT_JSON`/`verify_snapshot`/`snapshot_hash` 及其重复校验；相关键/描述符/期望调用点全部迁移 |

## 替代绑定（唯一权威）

- 实现：`experimental/auto-abi-probe/src/semref.rs` 现仅通过合并版
  `ac-semantics-contract`（`experimental/ac-semantics/contract`，crate
  `ac-sema-contract`）铸造名义引用与内容版本：
  - `mix_fn_ref()`：`DeclRef::new(pkg "probe797", module "autoabi", Fn,
    "p797_mix", ContentVersion::of_canonical(encode_typedesc_bytes(fn TypeDesc)))`
    + `verify()`；
  - `rec_mix_ref()`：Record `RecMix`，内容版本 = 单成员 `RecGroup::group_content()`；
  - `binding_hash()`：两 decl 的 logical+content 经 blake3——进入
    descriptor semref 分量、ABI-L0/ABI-O1 缓存键、admission 比对。
- 语义侧编辑（字段集/签名）必然改变绑定 → ABI 侧 semref 输入变化 →
  admission `SemRefMismatch` 受控拒绝（`abi_binding_semref_gate_on_real_identity`）。
- 布局/调用规范/物理签名仍归 797（layout.rs/descriptor.rs），符合合同
  "797 只读消费：类型描述 codec；ABI 布局与调用规范归 797 独立拥有"。

## 重验

- 切换后 `experimental/auto-abi-probe` 全套（含 T-01..T-05 证据面）绿；
  三腿/缓存/资源行为不依赖快照内容，仅键与 semref 输入换轴。
- 正式 AutoDown 执行（`abi/interop-v1` recipe 经统一 runner）见 T-06 交付
  （`crates/auto-lang/test/cases/autoabi/` + `abi_recipe_corpus_executes_green`）。

无第二套权威语义模型残留；本收据为该义务的闭合记录。
