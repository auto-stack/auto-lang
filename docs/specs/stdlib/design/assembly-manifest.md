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

## 验证等级

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
