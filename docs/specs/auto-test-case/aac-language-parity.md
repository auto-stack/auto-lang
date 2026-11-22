# AAC Gen1 语言对等（PLAN-1053 SD-01 预备稿）

> 状态：work 阶段预备 delta（按 PLAN-1053 §5 SD-01：work 准备、review 核
> 实际 revision、merge 再沉淀为 canonical）。本稿描述的是**已实测交付**
> 的行为与合同，不是规划。

## 1. 主体与工具身份

- **Gen1 整机编译器** = `experimental/ace/aac/**` 全部 60 个 Auto 模块的
  单一产物，经绑定 Gen0 工具链（sys 装配 → 编译 → 后端）产出两种真实
  原生可执行入口（Windows x64）：
  - `aac1053-gen1.exe`（AC(mn) 车道整机产物；boot 入口触达每一子系统）；
  - `aac1053-compile.exe`（整机编译命令；PLAN-1047 交付的语义管线驱动
    复用为入口，A2R+rustc 车道；env 合同 `AAC1047_INPUT/HIR/AAC1055_ABC`）。
- **工具身份合同**：`prepare-gen1` 产出准备目录，三份 manifest 钉扎
  （driver/compile/tools：全源 fingerprint、exe blake3、runtime 摘要、
  工具摘要、rustc 身份）；stale/篡改/替换一律具名拒绝。被 spawn 的
  corpus worker（gen1-worker）以 prep 钉扎副本为准，父侧核摘要。

## 2. 机器后端服务边界

编译产物的机器后端走 PLAN-1051 的 `ace-aac-bridge` 服务（独立进程，
AQ01/AR01 帧）；请求模式只携带 MIR ABC 与身份钉，**无源码字段**
（serde deny_unknown_fields）。Gen1 机器产物的 PE 导入面只允许 OS 系统
DLL（实测 KERNEL32+BCRYPT）；collections runtime 为静态摘要钉扎对象
链接；任何 Rust 前端/编译器服务库不得成为其动态依赖。

## 3. 语料接管合同

- 新 recipe `ac-aac-gen1/native-run-v1`（auto-test-case 注册 + Mn 探针
  族）；共享语言语料的 ac-native 探针逐 check 接手（recipe 切换 +
  去 `#ignore`），在固定组 `case-aac-gen1-language-parity`（target
  `gen1`）中执行；AV/A2R 视图显式 target 过滤，互不影响。
- 同一 Auto 源码字节分别由 AV / A2R / Gen1 后端独立判定（exit/流各自
  oracle）；CL 矩阵重放显示 Gen1 与 AV exit 零失配；前端缺口（具名
  stage/code 诊断）记 named-gap，不充 PASS、不静默跳过。

## 4. 已知缺口（named pending，owner 见 PLAN-1053 §10）

- pipe 方法 `lower.method` 缺口（1047 归档已开放）；
- 未约束泛型形参解析（985/A 具名 AC 差额）；
- 控制流/legacy 形 hir.unsupported、legacy 变量习语、async/CT/AGE
  组合 aac.pending-work/1（CL 族缺口图，41 例）。
