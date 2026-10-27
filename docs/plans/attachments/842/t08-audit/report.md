# PLAN-842 T-08 审计

三仓审计基线和当前 HEAD 见 [测试差额](test-id-diff.json)；T-07 因源码说明更新重开后，最终三仓安装与受影响验证已重绑完成。统计由 [脚本](audit-test-ids.py) 对差异 Rust 文件逐个读取两版 `#[test]` 函数，按文件/函数名及出现次数统计；它是有界静态清单，不冒充 nextest 全选择或运行结果。测试别名和分档配置逐文件未变，见 [收据](aliases.json)。

语言仓 86 个变更 Rust 文件，测试函数 387→397，未删、增加 10；UI 仓 16 文件 378→384，未删、增加 6；Man 仓 4 文件 7→6，两个静态测试函数随未挂载 `cmd_vue.rs`/`cmd_tauri.rs` 删除，另加 1。两处同名测试在历史 Man 代码没有模块挂载，不能冒称曾被当前 CLI 测试选择；其旧断言与 palette 在当前 UI owner 中分别映射为 `vue::plan842_retired_palette_controls::{cli_vue,tauri}::index_css_secondary_is_differentiated_from_muted`，各自验证对应历史 registry 的 light/dark secondary 值及当前真实 scaffold，2/2 通过。当前 `tauri.rs` 经 `VueProject::generate` 使用该同源 scaffold；实际 Vue 安装构建另有 T-07 证据。

| 矩阵 | 核销依据 |
|---|---|
| R01–R03 | T-04 七个旧公开入口逐一编译拒绝；规范入口、轻/完整 UI 文法、实际 builder 图标及 AST 类型身份通过。|
| R04–R05 | 最终外部方言 48 项和轻层 3 项保持；旧 helper 测试 ID 保留，Override/Append 与 session 语义不删。|
| R06 | T-06 旧 feature 具名拒绝、默认 VM 实跑、evaluator 特性 check，迁移指南解释 0.2.0 及旧源固定。|
| R07 | 正式 UI 兼容路径及合法 feature 保留；生成器/Man/示例迁 canonical；T-07 Core 缺席、Full Rust/Vue 与显式宿主通过；MPV feature 类型检查与缺库降级通过，未假称实机播放。|
| R08 | T-06 默认/显式/缺席、两实例、线程、正常约定退出与幂等验证；仅删登记时 SDK 直接调用红，按冻结条件保留必需单份登记与 hook，详见 T-06 报告。|
| R09–R10 | T-03/07 真实语言影子、仅影子、坏显式根、受管 Full SDK 的 owner/hash 及坏源保旧；开发工作区与交付共用来源校验。|
| R11–R12 | codec 旧/未知 domain、schema、tag、尾随垃圾拒绝；旧缓存失效、重建/reload；真实旧 Core/Full 升级及恢复，未知安装/交付清单拒绝，prepare/swap 保旧与未来版本备份保护。|
| R13 | `use-evaluator` 保留并 check；纯 SDK/非 UI VM 消费者、协议往返与失败/退出控制通过。|
| R14 | 当前生产 `UiGrammarSource`/`assemble_ui_grammar` 不存在；注释只记录 841 已退出。误称旧隐式文法仍存在的清单注释已修复。|
| R15–R16 | `find_store_decl_files`/`resolve_use_module`/`UseModuleResolution` 与 UI build、指纹公开薄入口保留；UI compiler 是单一实现，生成/缓存实际链路有 T-03/05/07 证据。|

777 边界：最终 Core 安装 SDK 和外部 consumer 的依赖树无 UI/旧 facade，轻文法 consumer 只引 canonical 0.2.0 dialect 和轻层；792 边界：UI source owner 与合法安装 receipt 一致性、source/staged copy hash、失败保旧和真实可搬动 fresh/locked consumer 有独立正反控。保留的旧名仅用于明确标注的 0.1 历史迁移示例、PLAN-810 一次性历史迁移/identity 脚本、旧退役说明或刻意编译失败的 probe；不代表当前生产调用。当前正式 UI 兼容路径和两项必需自动宿主机制按 R07/R08 保留，不伪称全部删除。

七份本计划测试生成的未跟踪 fixture lock 已按 0.2.0 离线刷新；每份原始字节与 SHA-256 保存于 [fixture-locks](fixture-locks/manifest.json)，核实副本一致后仅将这七个本次生成的工作树临时文件逐一移除。复跑需要锁时，按 manifest 的 `path` 将相应 `.lock` 复制回对应 `Cargo.lock`；新宿主 fixture 锁已纳入 Git。其他用户文件未动。最终审查仍须检查 warnings、格式、遗漏及每项 AC/SD；merge 档尚未执行。

R1 之前的工作期提交 lang `2de6176a9e573d04bbb72122996e70b90dd70288`、UI `533417e1c5d0bd171c83aeeae8b0dfdec783a41c`、Man `3f09779b2a3d4ce31f00d105f3f18520accf02b9`；当时 T-07 已重绑，R1 后以本报告末尾新提交为准。审计仅关本计划矩阵、当前三仓已知消费、历史脚本与测试差额；未知仓外用户不冒称已调查。

R1 修复后重新执行相同测试 ID 审计，结果及最终三仓 HEAD 见 [JSON](test-id-diff.json)；新增 UI 测试为失败重试保护唯一备份。七份快照不含 auto-api-gen，新增 `toml` 依赖不改变这些 fixture 的锁输入。R1 其他影响的验收证据见 [修复报告](../repair-r1/report.md)。

独立 R2 后补充核查发现，原文件内计数未覆盖 [824 批量记录](../../824/batch-r1/review.md) 的旧 ID 跨 owner 选择缺口；该静态审计不能单独证明全档选择。现已逐项映射 21 项，保证新目标可列举且未忽略，并将 9 项 recipe integration 实际加入 UI 全档；真实清单、缺失/过滤负控和单项运行见 [R2 修复报告](../repair-r2/report.md)。handler 从旧泄漏行为换为恢复语义属于 841 已批准的行为修复，非声称断言逐字等价。测试差额基线随重放更新为主线 `a547f52c5fbd5a24a203e94051af1e040c808f42`，原 18 个 842 提交 patch 等价；实际语言/UI/Man 最终 HEAD 见 [JSON](test-id-diff.json)。旧基线文件未删除。

R3 修复后的最终 HEAD 与基面见 [测试差额](test-id-diff.json)；21 项迁移在最终来源的实际列表仍全部 selected，见 [R3 修复报告](../repair-r3/report.md)。原基线文件未减项，全档尚待 review pass 后执行。
