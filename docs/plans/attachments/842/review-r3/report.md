# PLAN-842 独立最终复审 R3：needs_fix

独立子代理 /root/review842；继承实施交接，但从新提交、原始选择列表及独立真实反例重建结论。计划 r2；同组实施树均 clean：lang 5f3bd3701c24a916312189ea1be84de9a7cc8196、UI 98119a4a2d3f6a9a5e90efc629dd45625b4e970e、Man 0062f9224ba6b987f8e202827fa8f2e1b971ec8d；down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8。语言工作树含主线 a547f52c5fbd5a24a203e94051af1e040c808f42 并集；其他基面 UI 4903bef64532750d1597627b1197f3d2cac1effc、Man 9a75d3bab6eaeda961d5dcbf7932258f89e650af。规范与测试配置散列见 [输入](input-hashes.json)，SD 提案见 [冻结副本](spec-delta-snapshot.md)。

## F-842-R3-01：未校验实际消费的受管 SDK 全闭包（P1）

违反 AC-02/04/08、V-02/V-06c、R09/R10/R12 和受管安装的来源/失败保护。以 r3-final-full 的独立安装副本、生成 UI 正式 fixture、独立的已验证旧交付为输入，清空 AUTO_LANG_ROOT、AUTO_LANG_CRATE、AUTO_UI_SOURCE_ROOT、AUTO_DELIVERY_SOURCE_ROOT、AUTO_RUST_WORKSPACE 和 AUTO_UI_TOOL_BIN。

- 将复制安装的 sdk/crates/auto-ast/src/lib.rs 追加无害注释。实际 man-install verify exit 2 并报告该受管文件 changed；实际 auto.exe gen --delivery-root exit 0，修改内容进入新交付，旧 app sentinel 被覆盖且交付清单改变。见 [摘要](summary.log)、[安装校验](verify-install.log)、[生成日志](generation.log)。
- 另分别在 auto-ast/src 加入未登记文件、移走已登记 ast.rs：两次正式生成均 exit 0，前者把多余文件带入交付，后者产物缺必需源码，两次覆盖旧 app。见 [额外文件](extra-summary.log)、[缺失文件](missing-summary.log)。
- 单独修改受管 sdk/stdlib/auto/image.at、sdk/crates/patches/iced_widget/src/lib.rs，各自实际生成 exit 0、篡改内容被复制进交付并覆盖旧输出。见 [根输入](stdlib-summary.log)、[patch](patch-summary.log)。
- 仅对 sdk/Cargo.lock 追加合法注释，install verify exit 2；实际生成 exit 0、覆盖旧 app，并在交付 manifest 中以原固定 git_head、git_dirty=false 记录变化后的锁 hash。见 [锁摘要](lock-summary.log)、[安装校验](lock-verify.log)。
- 开发模式另用完整隔离安装，先成功生成旧 workspace/app，然后仅修改 sdk/crates/auto-val/src/lib.rs；安装版 auto.exe gen exit 0 且覆盖旧 app sentinel。auto-val 是实际语言依赖，而当前预检只查四个语言包。见 [开发摘要](dev-transitive-summary.log)。

根因已定位：auto-api-gen/src/source_owner.rs::validate_installed_language_source 只验证 SDK 根 Cargo.toml 和四个语言 crate；UI 工具的 delivery_preflight_producer 对其他闭包 crate、patch、五个根输入和 Cargo.lock 只检查存在，后续直接复制/读取。正式交付的现行闭包为 DELIVERY_CRATES、DELIVERY_PATCHES、DELIVERY_ROOT_INPUTS 与根锁等；开发 workspace 引用的传递依赖与交付闭包并不完全相同，需根据实际选择的受管来源核销，而非将任意项目资产当 SDK。非安装的开发源码仍需允许正常的工作树变更，显式来源优先语义须保持。

关闭 oracle：从真实有效安装及真实旧产物出发，分别注入已登记文件内容变化、缺失、额外文件，以及所有实际消费类（语言/UI crates、patch、根级输入/锁等）的代表性坏输入；要求具名失败、无隐式借源、旧 workspace/app 或交付逐字不变；恢复后运行真实入口及验签/构建。无需为不消费的文件 invent 新合同，具体受管范围以安装收据与选中闭包为准。

## F-842-R3-02：已校验语言源码在 staging 复制期间漂移未拒绝（P1）

违反与 F-R3-01 相同的来源一致性和失败保旧合同。开始时复制安装的全部受管文件均正常，旧交付存在。运行真实 auto.exe gen --delivery-root，观察本轮独占 staging 下 crates/auto-lang 目录出现，证明安装预检已通过并已开始复制；此后才在尚未复制的 sdk/crates/auto-media-core/src/lib.rs 追加 sentinel。该包属于目前四个已验证 crate。实际 exit 0，变化后的文件进入新交付、旧 app sentinel 被覆盖。见 [摘要](drift-summary.log)、[完整输出](drift-generation.log)。UI crate 在 delivery_copy_dir 后有 receipt 对 staged copy 验证，语言 crate、patch 与根输入没有相应复制后绑定；单次预检不能证明最终输出仍为已验证字节。应覆盖复制期间变动且在任何合法投影修改 staged Cargo.toml 之前核验源副本，失败不得落位。

## 本轮验证、受影响范围与审查边界

- R2 显式/自动发现坏根、安装缺包不回落及恢复在受修复版本的真实正反例和源码中有控制；R1 失败重试保备份代码未被 UI 本次 16 行来源选择修复改变，且受影响测试已通过。它们不足以推出新增完整闭包和复制期间一致性。
- 用提交内实际 driver 独立读取最终 list-only 原始 JSON：language 3212、UI 3117、tt 3563、tb 3102 selected，原基线与映射后 missing/suppressed 均为零；21 个迁移目标全部实际 listed+selected，UI 9 项确在 --test design_tokens_recipe 目标。独立重跑 driver 定向测试 18/18，通过缺失/忽略负控。十个 design token、一个 scene 的迁移已核对历史；handler scope 旧泄漏→新恢复为 841 的明确语义变化，不能称字节等价。list-only 不等于全档运行。
- 相对本次实际合流基面 a547f52c5 的 lang 全 diff-check 和 UI/Man 基面至 HEAD diff-check 通过；R2 末尾空白已闭。
- AC-01 与 AC-03 的未改部分使用已绑定的工作期证据；AC-02、AC-04、AC-08 因上述实际反例 fail；AC-05 的 21 项选择缺口定向闭合，但源变更及绑定仍需重核；AC-06 本轮 needs_fix；AC-07/整档/真实落地均为 merge-owned pending。SD-04/06 的“坏 producer 拒绝并保旧”和受管来源一致性尚不成立，其余 SD 条目按已读证据保留待受影响复审，不沉淀为已通过。
- 824 批量档的 plan707 cancellation 曾实际失败，是否偶发或由合流改变尚无充分归因。无全档在本次复审执行，后续统一门不得记作通过。
- 本轮是连续第二次独立最终复审 non-pass；依据 review 技能，下一次送审前 work 必须记录共同不变量、遗漏原因、完整输入/复制后校验设计和覆盖调整，不能只补这几个样本。与开发来源、经投影的安装源、跨 owner UI 收据、源复制后投影及恢复控制一起验证。

交给 work 同轮处理 F-R3-01/02；修复后重新绑定实际三仓提交、受影响真实安装与应用/交付证据，再进行独立复审。review pass 前不得运行合入全档或归档。
