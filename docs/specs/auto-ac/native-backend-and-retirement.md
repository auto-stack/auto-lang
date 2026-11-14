# auto-ac native 后端选择与旧 lowering 退役（PLAN-871）

> Status: active（2026-10-29 PLAN-871 交付）。冻结"共享 MIR 是唯一 native 语义源"这一事实，
> 并记录迁移窗口期的选择/回退/身份规则与退役门。

## 1. 现状（终态）

- **唯一 native lowering 路径** = `auto-mir-lower`（HIR → Typed MIR）+ `ac-mn`
  （Typed MIR → CLIF → per-unit COFF → 多对象 PE）。`experimental/ace/crates/core` 只保留
  前端 / Checked HIR / verify / 链接设施；**不再依赖 Cranelift，也不再包含任何
  HIR 直达 lowering**（`src/native.rs` 已删除）。
- `ac-core` 的 CLI 只剩 `check` / `check-source`（前端与检查角色）。native 构建面在
  `ac-native`（`experimental/ace/crates/native/src/bin/ac-native.rs`）。
- driver 层是唯一后端选择点（`ac-case-worker`、`ac-native`）；`ac-core ⇄ ac-mn` 不成立
  （ac-core 不依赖 ac-mn）。

## 1b. Linux x64 目标（PLAN-1040，2026-11-14 交付）

`ac-mn` 的第二个正式目标是 `x86_64-unknown-linux-gnu`（System V）：同一共享 MIR 唯一
语义源不变，按目标选择对象发射（ELF64 per-unit）与原语 stub；`rust-lld -flavor ld`
静态链接（无 CRT/解释器/动态依赖）；A09 身份轴含目标 triple，Windows 对象与 Linux
对象在 A09-ELF/A09 两层间永不错槽（结构准入按对象格式拒绝）。执行走 WSL2 验收环境
（见 [linux-native-platform.md](linux-native-platform.md)）。旧结论"唯一目标
x86_64-pc-windows-msvc"由本节修订为"双 admitted 目标"；Windows 车道全部冻结不变。

## 2. 后端选择（冻结于 T-02 F1，T-06 收口）

环境变量 `AC_NATIVE_BACKEND`，**每次请求读取**（无进程级缓存）：

| 值 | 语义（T-06 之后） |
|---|---|
| 未设置 / 空 | 默认表：五 profile 均为 `mn` |
| `mn` | 五 profile 均走共享 MIR |
| `<profile>=mn`（逗号列表） | 逐 profile 覆盖；profile ∈ core/scalar/text/record/text-record |
| `legacy` / `<profile>=legacy` | **已退役**：worker 与 `ac-native` 以 `request.backend` / exit 2 显式拒绝并提示卸载/改 `mn`；值仍可解析（可观察性），但不再执行旧路径 |
| 其他（未知 profile/backend、畸形对、重复 profile） | 显式拒绝（worker `request.backend`；CLI exit 2），不静默回退 |

## 3. 报告与 cache 身份

- **报告轴**：linked 行必须带 `ac-native-backend` = worker 的 `toolchain.backend`
  （`mn:<abi>+mir-v<MIR_LOWERING_VERSION>`；历史 legacy 行 = `cranelift-0.126/lower_object`）。
  非 linked 行（check/syntax、product consumer）不绑定该轴。
- **stale 重算**按行 profile 进行（开关是逐 profile 的）；`legacy` 解析结果为
  `retired:legacy`，**永不匹配历史行** → legacy 时代报告必然 STALE，不得静默当新鲜。
  畸形值解析为 `invalid:<raw>`，同样必 STALE。
- **cache**：mn 的 A09'/A10' 键含 backend/abi/lowering 版本与逐单元指纹；旧 783 单对象键
  与 mn 多单元键**结构不同**，不存在"同一逻辑单元两套键"；旧键在退役后无生产者，按键变体
  失效处理（无迁移义务）。mn 支持 `reuse` / `fresh`（不读旧单元）/ `read-only`（只读命中、
  从不发布，miss 真重算）。
- 收据 `ace.m1.cert/1` 键集不变；backend 只以报告轴表达。

## 4. 产物

- `ac-native`：`<exe>` + `<stem>.ac-link.txt` 收据（旧身份 needles：source-profile /
  hir-schema / hir-revision / hir-profile / source-input-hash；新轴：backend /
  mir-lowering / units / manifest-fp / 逐单元 SHA-256 / linker）。**不产出单一
  `<stem>.obj`**。
- `ac-case-worker`：manifest = `hir.atom + app.exe`（build/run）；mn 无单对象，故旧
  "build 必须同时带 app.obj"规则只对已退役 backend 适用。exe 按 logical 名发布到请求
  scratch。

## 5. 消费者与门禁

- 脚本：`verify-ac-741/745/767/770/771/782`（772 不存在）build 腿走 `ac-native`，
  check 腿留在 ac-core；每脚本保留"旧断言→新断言"映射（旧 `app.obj` 断言 → receipt
  存在且带 backend/mir-lowering 轴）。
- `scripts/verify-ac-871-retire.ps1`（Scope=retire）：G-DELETE 的常驻验证——退出文件缺失、
  旧符号 grep 零引用、`cargo tree` 无 Cranelift、三 workspace `cargo check --locked`、
  保留面存在，以及**反控：把旧符号片段注入 ac-core bin 必须编译失败**。
- 782 在后续计划树中作为差分旧门运行时，用 `AUTO_LANG_PLAN_TREE=<NNN>` 显式声明后继树
  归属（否则其 per-plan scope 断言按自身计划口径执行）。

## 6. 保留面（退役不得误删）

`verify.rs`（含移入的 `capabilities_check`）、`text_resources.rs`（TextResourcePlan 分析与
verify 调用）、`link.rs`、`descriptor.rs`/`atom_text.rs`/`hir.rs`/`source/*`/`body_work`、
fixtures/schema、`runtime_contract.rs`（PLAN-770 运行时合同门，`ac-native` 对 text/text-record
的显式 `--support-lib` 调用）、`coff_admission.rs`（A09' 恢复的结构校验）、`text_runtime.rs`
（运行时库定位/构建）、`native_exec.rs`（受控运行执行器）。

## 7. 退役验收（历史事实）

- G-COV：五 profile（core/scalar/text/record/text-record）逐行库存与冻结 oracle；
  T-03 shadow 两轮连续绿（旧腿 = 旧 lowering）后默认切 mn。
- G-DELETE：删除 `native.rs`（4652 行）及其独占辅助、ac-core CLI build 面、783 vertical、
  worker legacy 臂、av-probe 旧参照；`capabilities_check`/`runtime_contract` 移入 lib。
- 反控：shadow"变异 oracle 只红该行"；删除门"重引入旧符号必编译失败"。
- 迁移期暴露并闭合的 mn 能力缺口：view-record `t.clone()` 深拷贝、手工 HIR 纯 id 方法名、
  前向 `t_MutRet` 包装 ABI（由 def 级推导）、owned 字段直接读取（transfer/深拷贝）、
  构造器操作数快照顺序。


## PLAN-891：native宽标量typed入口

- 新增**独立版本化 typed ABI 选择** `msvc-x64/priv-typed-data-v1`（非旧
  priv-i32-bool-v1，非 797 AutoABI）：六标量整数位载体 + 纯值宽 record
  内部传递；typed 符号域 `ac9.<owner>.<unit>` 与旧域隔离。
- 唯一共享 MIR native 源不变：typed 输入为受检 MIR（文件化
  `mn-typed-fixture/1` → 共享 verify_mir → split → typed 准入），不走
  源码推导，不在旧入口放宽任何拒绝。
- 结果通道 `mn891.typed.frames/v1`：子 PE stdin 请求帧/stdout 单响应帧
  （nonce/签名/校验绑定，value/trap/resource/error 互斥），父侧独立
  oracle 判定 canonical (kind,bits)。详细合同见
  [auto-mir/native-typed-entry](../auto-mir/native-typed-entry.md)。
- 缓存/恢复：单元 COFF 键含 typed ABI 轴；typed wrapper 为居民产物按
  签名身份缓存；typed 居民会话暖编辑（whole fallback 恒 0）与冷恢复
  分账。计量：fresh/hit/restart/body-edit 各 20 样本，dev 门 P90≤2000ms
  （2026-10-31 实测最大状态 P90≈433ms）。

## PLAN-889：显式native集合消费者

显式`--source-profile collections`与`ac/mir-native-collections-v1`进入同一共享MIR→
CLIF/COFF→PE链，专用worker为`mn-collections-worker`；旧五域/default recipe/receipt精确键集
及retired:legacy拒绝不变，无旧HIR→CLIF恢复。支持与排除、runtime cell/2与runtime/2、
源码/layout/codegen/1、manifest/3和真函数增量详见
[集合原生合同](../auto-mir/native-collections-variants.md)。

报告绑定worker实际SHA、runtime build/content和linker/SDK/kernel32，集合族stale独立重算；
身份查询不是语言执行结论。runtime/config在候选commit前重查，失败保旧source/env/root/
manifest/PE/receipt；漏free反控只能ephemeral。891的typed/wide入口仍是后继规划。
