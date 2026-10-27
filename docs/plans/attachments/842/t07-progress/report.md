# PLAN-842 T-07 前版验证（已重开）

工作期冻结 lang `7bbd2cc9b9e0d373ea0182df343ddd75a7d8e4f3`、UI `1ef875b574bad75b4de07232c0103cc63e213bd7`、Man `484d9d2fe0904f13c9aff29ebbebea564bf8eac0`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；若后续更改代码、依赖、配置或输入，相关证据须重新绑定。此项不等于 review 或 merge。

- Core 实装 636 文件、清单 SHA-256 `8808b766d12349fc146289331c094e795b84328140d01a3efb8a62776a97586d`：完整 verify、实际 VM 程序、UI 功能具名缺席通过；安装内 SDK 独立 `cargo check --locked` 通过，正常/构建/dev 闭包不含 UI、旧 facade 或 shell-pack。独立外部工程直接依赖安装内 SDK，fresh lock/locked 测试 3/3（语言程序、管理规则、协议往返）通过，依赖树无 UI/旧 facade。未知版本、未知 profile、篡改升级拒绝和保旧恢复复测通过。
- Full 实装 6799 文件、清单 SHA-256 `945251248b255f3ec2b2e85310d653e03784ec91e4650a6620599c97edab1769`：实际 CLI 生成 Rust 交付、fresh lock/locked build（131.8s）、生成宿主默认/显式 provider 探针 1/1、依赖树与资源验收通过。两个无效显式根及坏 SDK producer 均具名拒绝并保旧，恢复后 verify/再生成/资源 verify 全绿。等长资源篡改被校验器识别，恢复原字节后通过。安装 CLI 实际读取自身 SDK blueprint 标记，恢复后整安装再次 verify；真实 Vue 示例生成并生产构建生成 dist。
- 最终源码外部消费者逐项复验七个旧入口、三个旧精确版本和旧 feature 拒绝，新 0.2.0 入口与默认 VM 实际执行通过；轻文法 6/6、外部 UI dialect 48/48、完整 UI 文法 21 passed/1 既有 ignored。真实生成后端 wire 与上传服务各 1/1。T-05 的前代 Core/Full 升级、旧版恢复、swap 故障保护和数据/缓存迁移证据仍适用，安装器及数据输入未变；此处重新执行当前版本的未知安装反例。
- 现存 warnings 已保留，未宣称健康审查通过。七个早期测试产生的未跟踪 fixture 锁保留；宿主探针锁已受管。T-08 负责矩阵和测试 ID/alias 核销，T-09 冻结迁移文档与 Spec delta，独立 review、merge-owned 门禁及实际交付仍 pending。

T-08 审计补齐两项原未挂载 CSS 断言后，UI/Man 提交已改变；本报告绑定的安装不再是最新版本，T-07 已重开重验。

## T-08 补测后的重新绑定（当前通过）

UI `36b5dc0f0e673943f725eb4a80c4c98541632236` 新增两个 `#[cfg(test)]` CSS 控制，Man `ada29fbefb53e57d61bab5341e26dd5419ab1650` 更新 UI pin；lang 和 down 未变。原 T-07 安装作为历史保留，不再冒认为当前。此次 Core 636 文件，manifest SHA-256 `67a77467c7dd84f5902d138b27b0ab946504a4bd37e556133807c93c793e2247`；Full 6799 文件，manifest SHA-256 `3767ea6703d242f562988b8eedcc56fd2d1029547dfd551c8e2344e2b4c41ca2`。

- 新 Core verify、实际 VM 程序、UI 缺席诊断、安装内 SDK locked check、独立外部 consumer fresh lock/locked 3/3 和闭包均通过；未知 schema/profile/篡改拒绝及恢复复测通过。
- 新 Full verify、真实 Rust 生成/fresh lock/locked build（138.7s；使用明确指定的既有 target 缓存）、生成宿主 1/1、依赖树、坏显式根/坏 producer 保旧及恢复通过。真实 Vue 生成/生产构建、安装 blueprint 专属标记、等长资源篡改和未来 schema 拒绝/恢复均通过，Full 安装最终 verify 通过。新旧 auto-ui-tool 可执行字节不同，故未复用旧蓝图运行证据，而是实际重跑。
- 后端、方言、版本、VM 与宿主 consumer 所用语言、UI runtime/compiler、依赖和测试代码未变；本轮 UI 唯一源码变更是 `vue.rs` 末尾的测试模块，Man 唯一变更是固定 UI SHA，故可复用前述定向证据。若再改变这些输入须重新核验。

T-08 继续修正旧 facade 的错误源码说明后，语言/UI/Man 提交再次改变，T-07 已重开；前两轮安装只作历史，必须针对审计后的最终来源重新绑定。

## T-08 说明审计后的最终重新绑定（当前通过）

最终工作期来源 lang `2de6176a9e573d04bbb72122996e70b90dd70288`、UI `533417e1c5d0bd171c83aeeae8b0dfdec783a41c`、Man `3f09779b2a3d4ce31f00d105f3f18520accf02b9`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。新 Core 636 文件，清单 SHA-256 `f2074f575c42bacb8218b5ac8c9db3c7fa34aea57b1240c27dbc93b11c4b1c1b`；新 Full 6799 文件，清单 SHA-256 `e3447d3abcc87dbfab7fee7d9a1992cbd499d9166e70b08f2c647c5b91c8d726`。前两轮安装仅为历史。

- Core verify/真实 VM 程序/UI 缺席诊断、安装内 SDK locked check、独立外部 SDK consumer fresh lock/locked 3/3、依赖树隔离，以及未知版本/profile/篡改/失败恢复均通过。
- Full 安装 verify、真实 Rust 交付 fresh lock/locked build 与生成宿主 probe 1/1、坏显式根/坏 producer 保旧和恢复通过；实际 Vue 生成与生产构建、安装内蓝图读取和恢复、资源等长篡改/未来 schema 拒绝与恢复均通过；所有探针后 Full 安装再次 verify。详情见 `audited-delivery-logs/` 与同目录日志。
- 语言/UI 此轮差异仅源码文档和消费者 manifest 注释，Man 仅更新固定 UI commit；后端/文法/模型行为、代码和测试配置不变，原真实 wire/上传、grammar/codec 等定向证据按其具体输入复用。最终 review 与 merge 必须重新核对来源、配置与证据。

## R1 修复后的最终重新绑定（当前工作期）

R1 来源和备份恢复两项及语言来源邻接已修，上一节来源转为历史；最终提交、Full/Core 实装、真实外部锁定构建、失败恢复、Vue、资源和隔离证据见 [修复报告](../repair-r1/report.md)。三树 clean；复审与 merge 门仍 pending。

## 独立 R2 修复及主线并集后的重新绑定

上一轮安装和其来源转为历史；最终三仓提交及 Full/Core 实装、真实坏源保旧与恢复、外部锁定消费和验证复用依据见 [R2 修复证据](../repair-r2/report.md)。独立复审、合入门禁仍 pending。

## 独立 R3 修复后的最终重新绑定

前次来源与安装转为历史；完整 SDK 收据与 staging 实际字节校验、开发源路径依赖、最终 Full/Core 实装和真实外部消费见 [R3 修复报告](../repair-r3/report.md)。独立复审和 merge 门仍 pending。
