# PLAN-831 T-09 规范增量候选（SD-01..07）

日期2026-10-24；Plan r2；proposal-only / needs_fix。绑定handoff.md §1的三仓完整SHA。当前canonical文件SHA256见source-audit.json的spec_inputs；目标不存在明确记absent。这里只准备沉淀文本与差异，未编辑canonical、ledger或索引。开放finding闭合后需更新实际事实，再由T-10复核。

## SD-01 · add · docs/specs/auto-ui/project.md

before：无独立UI仓总模块规范。

after候选：auto-ui既有仓持有auto-ui-dialect/compiler/runtime/tools与shell-pack五包；目录均为crates/<包名>。语言基础SDK/轻数据、project-model、std-only tool-execution与auto-api-gen adapter仍由语言仓单源持有。UI不得反依赖Man管理实现或旧auto-lang巨包；旧auto-lang留语言仓作为上层兼容组合，公开路径经真实UI owner再导出，不进入纯SDK闭包。最终版本由三仓固定commit和发行源码清单共同标识，不以path依赖或包版本0.1.0代替源码身份。

来源：UI Cargo.toml/五包Cargo.toml；lang兼容lib.rs；C2 project-model及api-gen变更。AC-01/02，V-01/02；F02/05/06未闭，不能先写已完整独立发行。

## SD-02 · add · docs/specs/repository-boundaries/ui-repository-delivery.md

before：仅已有生成UI bundle合同，无完整UI仓交付总表。

after候选：区分开发三仓同级path组、源码固定commit clone渠道与受管外部生成bundle。开发解析允许显式根/组内兄弟；发行消费者须使用已登记固定源码或验证过的内容寻址包，清理开发根覆盖并禁止隐式主检出回退。profile包含compiler/runtime、真实tool二进制及其运行资产；实际支持/平台/第三方工具前置逐项登记，缺环境为未验证，不能删除支持。source/fresh/locked/A→B/运行与失败恢复收据各绑定producer/工具/锁/资源身份。

多仓状态为prepared→UI landed→language switched→verified；单仓仅ff-only且核旧锚，失败保留已完成状态和恢复指令，不假称跨Git原子。真正取消/超时/交换失败仍按原受管保旧合同验证，不用合成组代替真实运行。U831完整收尾前不放行832。

来源：scripts/ui-repository-{distribution,switch}.py、792 driver、UI tools/rust_ui.rs。AC-03/04/05，V-03/04/05；F01/03/04/05/06未闭。

## SD-03 · modify · docs/specs/repository-boundaries/project.md

before：784保全与仓界准备事实，部分旧路径仍指语言仓UI包。

after候选：保留784历史，追加831的T-02快照45044cf9507d5628、10 refs/WIP与恢复读回记录，并明确payload和legacy分支不得作为清理对象。记录841文件迁移与语言原件删除（详source-audit.json），旧UI legacy从候选workspace退出。区分迁移分支候选、默认分支实际交付和远端动作：当前仅候选，T-11实际切换及T-12归档读回后再记完成；未执行远端推送。

来源：preservation.md/import-ledger.md/source-audit.json、切换脚本；AC-05/08，V-05。AC-08仍为merge-owned pending。

## SD-04 · modify · docs/specs/repository-boundaries/generated-ui-distribution.md

before：producer源码布局以单语言仓包路径为主。

after候选：受管UI五包实际源可由独立UI根提供；delivery_crate_source按显式UI根/组内UI源取得owner源码并投影为bundle内相对依赖，语言SDK源及其root inputs仍具名登记。保持canonical auto-ui-runtime+auto-lang-core双直依赖、vendored patch、版本pin、多槽锁、资源哈希及陌生目录/旧字节保护，不因搬仓禁用负控或扩大allowlist。交付身份需记录各producer根及固定revision/内容hash，不能只标语言根SHA而未绑定UI源。

来源：UI tools/rust_ui.rs的DELIVERY_CRATES/DELIVERY_ROOT_INPUTS/delivery_crate_source与producer指纹；792 driver。AC-03/04，V-03/04；F03/05未闭。当前实现按primary语言根优先再查UI根，不能沉淀为已强制唯一owner/错源拒绝。

## SD-05 · modify · 四个UI模块规范

targets：docs/specs/auto-ui-compiler/project.md、auto-ui-runtime/project.md、auto-ui-tools/project.md、auto-ui-dialect/project.md。

before：仓内crates路径、测试/资源源根与单仓命令。

after候选：同名路径属于auto-ui仓，公开类型/feature/动态状态仍单源。compiler自持schema/aura.at；runtime字体与.at资产、tools嵌入资产及真实auto-ui-tool随owner；dialect对留语言仓a2gd语料使用显式来源。测试/fixture/例程/blueprints/宿主CLI依赖必须指明实际owner，跨仓CI需保留原测试ID、feature选择、integration和单进程ui_gen档，不能凭源码文件未少宣布无丢测。现行Vue/Rust/VM-Iced及jet/ark/tauri/vscode能力按既有合同保留；实际运行与寿命矩阵须由迁后来源证明。

来源：五包blob对账、compiler schema include、runtime/工具资产、ci-entrypoints.md；AC-01/02/06，V-01/02/06。F01/02/03/04/06未闭，当前断根不能描述成已完成迁移。

## SD-06 · modify · 语言/共享声明/Man兼容规范

targets：docs/specs/language-sdk/project.md、project-model/project.md、auto-lang/project.md、auto-man-core/project.md、auto-man/project.md（共五份，沿用Plan原目标集合）。

before：UI包同workspace、tools/API经Man核心取展示及sidecar声明。

after候选：纯SDK缺UI检出仍可parse/compile/VM执行，normal/build/dev闭包不拉UI组合；旧auto-lang UI面仍上层再导出独立UI包，不混入纯SDK承诺。SidecarDecl/sidecar_decl_from_root与theme_decl_from_root/external_backend_from_root定义单源auto-project-model；Man旧sidecar入口再导出，pac_from_decl完整Man规则仍居auto-man-core，其展示字段委派轻规则。UI tools和API adapter不再依赖auto-man-core，不复制声明归一，不改变I-PROJECT wire、动态求值/缺省/覆盖语义或进程状态身份。通用owned tool-execution留语言基础供UI/Man共享。

来源：lang project-model/{sidecar,pac_display}.rs、man-core/pac.rs/sidecar shim、api-gen侧导入、UI tools/rust_ui.rs/vue.rs；AC-01/02，V-01/02。F02/05未闭；保留T-05语义回归证据且不得夸为全feature实编通过。

## SD-07 · add · docs/specs/auto-api-gen/project.md

before：无独立adapter模块规范，sidecar元数据读取Man规则。

after候选：auto-api-gen是语言服务/API生成adapter，物理owner留auto-lang，服务UI和Man消费者；sidecar声明从auto-project-model读取，零Man管理实现依赖。adapter不拥有第二份SidecarDecl或根声明解析，不持有UI宿主状态。真实接口与旧入口兼容由定向语义及类型消费验证，生成器资源和外部工具前置按实际消费者闭包登记。

来源：lang crates/auto-api-gen/{Cargo.toml,src/sidecar.rs}；C2单源扫描；AC-01/02，V-01/02。T-05 api-gen tauri测试预存红须具名归因，不把历史54/55写成全绿；F02未闭。

## 元数据与冻结约束

保留Plan的supersedes_spec_components、新增三目标及touched_goals=[GOAL-003,GOAL-007]，不更改Plan语义revision。progress/finding记录不构成合同修订。SD-01..07均覆盖，未自行退休后端、删除测试或将合同内缺口豁免为债务。待work闭合后，本草案及target输入hash须重新冻结；任何实现/依赖/配置/输入变化均重新判断受影响证据，T-10不能审核本快照后新增代码。
