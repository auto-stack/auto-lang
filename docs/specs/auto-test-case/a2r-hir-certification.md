# a2r-hir 认证通道（recipes `ac-a2r/hir-scalar-emit-v1` ＋
`ac-a2r/hir-record-text-emit-v1`）

> 状态：active（PLAN-892 交付标量通道；PLAN-902 交付 record-text 通道；
> 与旧 `a2r/emit`＋`ace.m1.cert/1` **并列**，互不授信、互不改动）。

## 注册（796 probe seam）

- 两个 a2x recipe 均进 `REGISTERED_PROBE_RECIPES`（append-only）；绑定
  `a2x_hir: { profile, entry, caps }`（required-for-recipe，其他 recipe 声明
  即装载拒绝）；步词表=单 `run`。**profile 集按 recipe 严格不相交**（标量=
  core|scalar；record-text=record|text|text-record；越界=装载拒绝）。
- **artifacts 期望通道仅 a2x recipes 可声明**（decode 按 recipe 门控；其他
  probe recipe 声明 artifacts 装载即拒——不破坏 av/cte/mn 合同）。
- 分发：`test_cases/ac_a2r_hir.rs`（双 recipe 共适配器；echo 按 case 自身
  recipe 核验）＋ mod.rs。worker 按 profile 选通道（fileset/证书 schema 随
  通道），绝不混通道授信。

## 日常腿（snapshot，spawn0）

适配器只 spawn **预构建 a2x-worker**（`a2xworker/1`；env `AC_A2X_HIR_WORKER`
覆盖→`<repo>/target/release`→`experimental/ace/target/release`；**绝不
自动构建**——缺失=受控 infra ERROR 带维护命令）。协议/recipe 回显核验＋
worker 自报 `spawned_rustc=0` 父侧断言＋manifest 父侧重哈希＋golden 逐字节
＋未声明多余产物拒。**工具内容凭证**：worker 二进制 sha256 入每个 verdict；
可选 `AC_A2X_HIR_WORKER_SHA256` 钉死（不匹配=infra，永非语言判决）。

## 认证腿（显式 certify）

### 普通源 CT 的真实 worker 入口（PLAN-958）

scalar profile 授予 `ac.source.cte.scalar/v1` 时，Rust/C 的 daily 与
certify 四操作均经同一个 source-cte 前门，消费真实 AV 执行、结构回灌及
事实车道形成的 Checked HIR。meta 样例另需 `cte.meta.fields/v1`，实例/
子 CT 样例另需 `ac.source.instances.scalar/v1`；没有 CT capability 的
`#` 源受控拒绝。原通道身份仍是 scalar，证书绑定同次实际 HIR 域、原始
source hash 和已授 capabilities；不能用 Session 层发射文本检查代替
真实 worker 认证。

永久入口为 `cargo t ace-a2x --test-name worker_ct_958`：七个原936源两端
daily 共14操作、两端显式认证共14操作，并含普通源/无cap与错oracle/
错reference控制。认证后用原 owned executor 再运行现场实际/参考可执行
文件，逐字节核手算值42/43/7/42/43/42/40及空stderr；普通源的原独立
值与日常spawn0检查同时保留。C编译认证遵循原Windows/MSVC条件。

同一现场 actual 集＋逐例手写独立 reference（新 i32/trap 形态）分别真实
rustc 编译运行，stdout 值/trap 线与 stderr 标记序双方一致等于闭式 oracle；
严格证书 `ace.a2r-hir.cert/1`（绑定全文件集＋reference 字节＋实测 rustc -vV）；
错 oracle/错 reference→REJECTED。受控执行=Job 子树＋deadline（编译≤120s/
运行≤30s；挂起目标被杀受控拒）。

### 序列化文件准入（PLAN-951）

`CertDraft` 的 typed 验证与重复序列化字节稳定均不能代替文件格式验证。
scalar、record-text、collections 的 `to_json` 输出必须可由标准严格 JSON
decoder 直接解析，不由消费者补逗号、删尾逗号或宽容修复。字段顺序及原
schema/recipe、完整文件集、工具、能力、配置键材语义保持；解析后能力列表与
配置字符串（含其内部 JSON 的转义）须与 producer 的原 typed 值一致。
回归入口为 `cargo t ace-a2x --test-name cert_json_951`，包含 null/集合非空配置、
空/多项能力列表、字符串转义与坏分隔符拒绝控制。

### 实际 HIR 域绑定（PLAN-952）

证书的 `profile` 与 `hir_domain` 均绑定同一次实际编译产物：HIR 域直接取
`EmitOutcome.hir_domain`，与该产物的 manifest 一致。不从 profile 字符串
另建映射或在未知拼写时默退 core。标量、core、record、text、text-record、
collections 的真实认证回归由 `cargo t ace-a2x --test-name cert_domain_952`
核对 actual/reference 的值与 cert/manifest/事前正式域；六个 profile 均执行，
不能用手构证书或文件可解析代替域身份验证。

## 资产与门

`test/cases/a2r-hir-892/`：B01/B04/B13/B14/NAMED-01 五例（源=冻结语料字节
复制；golden=真实 worker 冻结＋跨 nonce 确定性复验；reference=手写独立）。
测试面 `a2r_hir_892_tests`（日常绿/篡改 golden 红/缺 worker infra/凭证钉/
certify 腿+反控）。冻结 load256 正式采样（31 修订预认证）：hit/restart/
body-edit P90=427/490/176ms≤2000ms，fresh 单列（828 先例）。

## record-text 通道（PLAN-902 交付）

- `test/cases/a2r-hir-record-text-902/`：R07/R08/T03/T05 四例（源=冻结
  ace-m1 字节复制 hash 钉定；golden=真实 worker 冻结＋跨 nonce 一致；
  reference=手写独立 std 形态）；固定组 `case-a2r-hir-record-text-902`
  （tools=case-parent+a2x）真实发现执行 4/4。证书
  `ace.a2r-hir.record-text.cert/1`（严格 `validate_as`：标量身份不验 rt
  证书、坏 verdict/空绑定/缺工具身份拒）；certify 反控（错 oracle/错
  reference/晚失败 REJECTED；协议腿经真实 worker）。作用域验证入口
  `scripts/verify-a2r-record-text-902.ps1`（八作用域收据；formal=四状态
  ×30 样本预认证窗口，暖门 P90=6/21/40ms）。标量五例/58 语料与旧 a2r
  产物不变。

## Collections 通道认证（PLAN-904）

- 并列 recipe `ac-a2r/hir-collections-emit-v1`（仅 profile `collections`；contract
  recipe/注册表/profile 绑定表互斥）；fileset `auto.a2x.rust.collections.fileset/1`
  （4 文件，manifest 绑替换后完整源 sha + execution_config）。
- 证书 `ace.a2r-hir.collections.cert/1`：`execution_config`（canonical 定键序 JSON：
  mode/budget/cancel/events/fault）为**必填键材**——debug/release 分键互不授信；旧
  schema 携带该字段=无效。两腿（actual+手写 std-only reference）真编译运行对独立
  oracle；父侧全文件重 hash+字节金样+未声明额外文件拒+路径域校验（相对/无 `..`/
  归一化唯一/大小写碰撞具名拒）。

### 904 golden manifest 独立 oracle（PLAN-933 交付）

- 20 个 `golden-*/manifest.json`（D-931-3 经日常 emit 通道补齐入库）由**独立
  reference 逐字节重建核验**：oracle 测试（`a2x-worker/tests/cases/
  coll_manifest_oracle_933.rs`）不 import 任何 `ace-*` 生产 crate，自含
  FNV-1a-64/SHA-256（公开算法+公开测试向量自检），从跟踪 `src/*.at`、三份金样
  字节与冻结合同字面量重建全部 16 字段+JSON 字节格式，20/20 与金样逐字节一致；
  错输入哈希/错源/错 config/近邻 schema/错文件序/缺尾 LF/篡改金样副本在字节与
  typed 双轴均被拒（V-07）。**分轴保持**：该 oracle 是 emit/oracle 轴的独立可审
  计面，不替代日常 case 通道（真实 worker+父门逐字节判）与显式 rustc 认证腿；
  actual emitter 永远不是 oracle 生成器。
- store/session 四态轴（`store_coll_933.rs`）：20 源逐例 fresh（绑全源
  sha256+冻结 config）/stale-base 具名拒/空白编辑（身份重绑、生产 0——
  未改函数生产 0 门）/语义编辑（生产恰 1）/历史回访（store hit、生产 0）/
  restart 重装配字节一致。恢复→首编辑 SYN 腿归 PLAN-929（D-931-1 pending），
  本通道不冒充。
