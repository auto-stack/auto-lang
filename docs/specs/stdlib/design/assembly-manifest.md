# 标准库装配契约与 AssemblyManifest（PLAN-738）

> 来源：PLAN-738（rv3，delivered @`3713337d9`）。装配合同的数据面见本文；
> 后台 provider 索引见 [backend-assembly.md](backend-assembly.md)；
> 模块解析侧的选层/缓存边界见
> [module-resolution.md](../../auto-lang/frontend/design/module-resolution.md)。

## 两类清单分离

- **inventory**（`stdlib/auto/**` 全量扫描）：模块/层文件/公开
  fn/method/type/field 分母、解析结果、provider 候选。失败层记诊断，
  不从分母删除；解析失败整体输出 partial（非零退出），不统计漏项为 pass。
- **AssemblyManifest**（某次编译/请求的真实选择快照）：只记录该次请求
  实际装配的内容。目录扫描不得冒充 manifest；inventory/candidate 不冒
  actual。

## 必含字段

schema/provider 目录版本、execution target（vm/rust/c）、environment
（native/browser）、consumer、编译 features、按序 selected sources
（公共在前、选定目标层在后，含层字节边界与内容指纹）、依赖闭包指纹、
实际引用证明（reference proofs）、providers 输入身份（含本地
producer/compiler 源与 Cargo 版本）。

## 双重身份（schema 4）

- `fingerprint` = **共同装配身份**：payload 的消费者中立投影（consumer
  名置空、`consumer_input` 角色源剔除）哈希。同 fixture/同 target/同
  features/同来源闭包下，CLI actual、编译会话、生成收据三个消费者相等
  （三角对拍冻结）。
- `consumer_fingerprint` = **消费者收据身份**：全量 payload 哈希（旧
  schema 3 单指纹语义）。新鲜度门/收据对拍/服务烘焙常量统一使用它；
  跨消费者断言一律用 `fingerprint`。

业务输入（api.at/db.at 等）只进收据身份，不改装配身份；target/
provider/来源闭包变化改变共同身份。未知/旧不完整快照=陈旧或拒绝
（保守再生），null 指纹不能相互证明新鲜。

## ACE 有限模块清单（PLAN-944）

`ace-semantics-contract::assembly::Manifest` 使用独立域
`ace-stdlib-assembly/1`，不复用或改写上述 legacy schema 4。
`AssemblyManifest::decode_v4` 仍按旧字段顺序及 FNV 双投影读取历史4；
两种入口相互拒绝另一种格式，未知 schema/producer/profile 不作默认兼容。

新清单共同指纹覆盖 profile、host、platform、全部源内容身份、完整声明
（module/symbol、签名/mode、语义版本、显式 requirements）及操作引用证据。
消费者指纹另覆盖 producer、ABI、runtime、发射工具配方和物理 provider
闭包。AV 与 HIR→Rust 在相同语义输入下共同指纹相等，消费者指纹不同。
便携 ID 不携绝对目录；改变 provider 只改其消费者身份，不改共同身份。

公共操作证据由 SDK `stdlib_assembly::ace::current_operation_reference`
重新扫描943合同/公共声明，并核最终 `NativeInterface` 的独立 adapter。
当前实证为 `io.read_line` 的签名证明，保逻辑操作 ID/版本、相对声明及
producer、声明 hash、adapter 合同 hash；数字 native ID 不是操作身份。
它不使该操作在有限标量源域可调用，也不把 signature_checked 升为 executed。
新证据记录的 verification 只接纳 `signature_checked`，不能从缓存提升为
executed；声明 FNV16、adapter/input64 均要求规范小写十六进制。

结构解码与当前身份准入分开：恢复的期望清单须来自当前 source/contract/
provider 输入，不能复制候选载荷充当期望。重复声明/未知字段、坏指纹、
不符的 host/platform/ABI/runtime/发射配方在消费前拒绝。
具体绑定、恢复、单事务发布及授权见
[ACE绑定与装配](ace-binding-and-assembly.md)。

## 验证等级（legacy 与新证据同义）

declared → resolved → bound → signature_checked → executed。文件存在/
名称登记最高到 resolved/bound；producer 签名核对（含适配契约）后才
signature_checked；真实执行才有 executed。未知不得升格；
**Resolved-only 不是引用核心调用成功的依据**——strict 发射/生成入口
只接受充分证明。

## 快照不可变与便携性

freeze 后同输入重算逐字节一致；便携源 ID（stdlib 相对）与本地诊断
路径分离；同内容异绝对根共同身份稳定。

## 指纹族

manifest 双指纹（上文）之外，`stdlib_assembly_fingerprint` 为内容级身份
（inventory+目录 schema+目标+全部层内容+宿主实现 build 输入），用途
不同、都目标敏感；read 失败闭合拒绝（fail-closed）。

## 核心六模块 strict 门

io/net/async/http/json/sse 的**被引用闭包** strict：缺实现/冲突/未证明
=拒绝（SIGNATURE_DRIFT=已证漂移、SIGNATURE_UNVERIFIED=无证明），不产出
成功产物；未引用的 unsupported 声明不拒绝模块导入；同名用户符号干净
遮蔽 stdlib。

## 公共面契约覆盖完整性（PLAN-755）

strict 门的拒绝语义（上节）之外，**契约声明面必须覆盖公共层全部被选中
callee**：stdlib 公共层（如 `stdlib/auto/http.at` pub 面）每个「解析成功且
shim 已绑定」的符号——即真实程序引用时走 `verify_core_reference` 的
callee——必须在 `NativeInterface::production()` 最终身份声明（注册/inventory
推导/覆盖全部之后）有独立适配契约，且 `signature_matches` 对公共声明逐字
核对。缺声明（callee 有实现但无契约）与声明漂移（如 inventory 按 Rust 栈
载荷推导返回 int、而公共面身份为 Response）都是缺陷，不是降级路径；句柄类
返回一律按公共身份声明（显式身份最后落位，覆盖 inventory 推导与 merge 撤
销语义）。

- 新增公共 native 的改动必须同步补契约；验收物含契约覆盖清单对账表。
- 常驻守卫：`plan755_http_public_face_contract_coverage` 走查（缺契约/漂移
  即红，含走查空转守卫）+ `plan755_http_contracts_survive_cffi_merge`
  （`AutoVM::new` 同款 CFFI merge 后契约存活）。
- resolved-but-unbound 符号（公共名解析到 registry id 但接口无绑定）属
  shim 缺口，走 PROVIDER_CLAIM_NO_CALLEE 分诊，不在契约侧补（无 producer
  可证）。
