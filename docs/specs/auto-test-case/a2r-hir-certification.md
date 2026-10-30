# a2r-hir 认证通道（recipe `ac-a2r/hir-scalar-emit-v1`）

> 状态：active（PLAN-892 交付；与旧 `a2r/emit`＋`ace.m1.cert/1` **并列**，
> 互不授信、互不改动）。

## 注册（796 probe seam）

- recipe 进 `REGISTERED_PROBE_RECIPES`；绑定 `a2x_hir: { profile, entry, caps }`
  （required-for-recipe，其他 recipe 声明即装载拒绝）；步词表=单 `run`。
- **artifacts 期望通道仅本 recipe 可声明**（decode 按 recipe 门控；其他 probe
  recipe 声明 artifacts 装载即拒——不破坏 av/cte/mn 合同）。
- 分发：`test_cases/ac_a2r_hir.rs`（新文件）＋ mod.rs 两处追加。

## 日常腿（snapshot，spawn0）

适配器只 spawn **预构建 a2x-worker**（`a2xworker/1`；env `AC_A2X_HIR_WORKER`
覆盖→`<repo>/target/release`→`experimental/auto-a2x/target/release`；**绝不
自动构建**——缺失=受控 infra ERROR 带维护命令）。协议/recipe 回显核验＋
worker 自报 `spawned_rustc=0` 父侧断言＋manifest 父侧重哈希＋golden 逐字节
＋未声明多余产物拒。**工具内容凭证**：worker 二进制 sha256 入每个 verdict；
可选 `AC_A2X_HIR_WORKER_SHA256` 钉死（不匹配=infra，永非语言判决）。

## 认证腿（显式 certify）

同一现场 actual 集＋逐例手写独立 reference（新 i32/trap 形态）分别真实
rustc 编译运行，stdout 值/trap 线与 stderr 标记序双方一致等于闭式 oracle；
严格证书 `ace.a2r-hir.cert/1`（绑定全文件集＋reference 字节＋实测 rustc -vV）；
错 oracle/错 reference→REJECTED。受控执行=Job 子树＋deadline（编译≤120s/
运行≤30s；挂起目标被杀受控拒）。

## 资产与门

`test/cases/a2r-hir-892/`：B01/B04/B13/B14/NAMED-01 五例（源=冻结语料字节
复制；golden=真实 worker 冻结＋跨 nonce 确定性复验；reference=手写独立）。
测试面 `a2r_hir_892_tests`（日常绿/篡改 golden 红/缺 worker infra/凭证钉/
certify 腿+反控）。冻结 load256 正式采样（31 修订预认证）：hit/restart/
body-edit P90=427/490/176ms≤2000ms，fresh 单列（828 先例）。
