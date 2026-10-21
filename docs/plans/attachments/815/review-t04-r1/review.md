# Plan 815 — T-04 提前定向复审 R1

结论：**needs_fix**，六项开放；重开 T-04，Plan 保持 executing，进度 9/17。不是 T-09 全计划终审，也不撤销 M815-A/M815-L 已发布历史结论。

审查基线：r3；HEAD `90aed7fc7df875987cf9014a37fdc256e83fdae6`，diff base `7ffa2d88d9f5ee500a1f759f0a388f6899e825f2`；依赖 M814-A=`f6e07aceeeea3cb12698c747d61db5be43f7458c`。唯一树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`。首尾实现均 clean、HEAD 未变。完整输入身份见 [baseline](baseline.json.txt) 与 final-baseline.json.txt。审查者 Codex `/root`；同会话有早期合同/账本上下文，未参与 T-04 实施，从提交、合同、实际 consumer 和独立反例重建；没有新建 agent、会话或树。

## 开放 findings

### F-815-T04-R1-01 / P1：真实交付包缺少必需 path 闭包

定位：`crates/auto-ui-tools/src/rust_ui.rs:3847`、`3899..3902`（DELIVERY_CRATES）。合同 §2.3/§5 要求迁移同步补齐发行表与 792 消费，不依赖 T-05 runtime 发布。

真实生成器成功产出 5188 文件/37.3 MiB 的 bundle；以 bundle 自身运行 `cargo metadata --offline --no-deps` 立即 exit 101：auto-tool-protocol → auto-project-model 的 Cargo.toml 不存在。诊断副本只补入 auto-project-model 后仍 exit 101：auto-ui-tools → auto-api-gen → auto-man-core → auto-toolchain-core 不存在；再只补第二个包后 metadata exit 0。原 23 项清单遗漏这两个必需来源，fresh lock/build 在进入运行时之前就失败。

修复：闭包、根成员/来源清单/指纹/预检同步补全；验收至少使用真实 producer 生成包的隔离 metadata/闭包正反控，再按合同完成适用 792 证明，不以合成 test_producer_root 或只检查 23 个目录存在代替。证据：generate-cargo.log.txt、delivery-metadata.log.txt、delivery-after-auto-project-model.log.txt、delivery-after-auto-toolchain-core.log.txt、delivery-closure.json.txt；控制只改忽略目录中的生成物副本，生产代码未改。AC-01/04，T-04。

### F-815-T04-R1-02 / P1：迁入单测缺 dev-dependency，编译器测试目标无法编译

定位：`crates/auto-ui-compiler/src/store_deps.rs:100`、其 Cargo.toml dev-dependencies。新增 store_deps_scan_tests 使用 tempfile，但 auto-ui-compiler 清单没有依赖它；传递依赖不会自动成为可导入的直接依赖。

运行仅选资产锚的 `cargo test -p auto-ui-compiler --lib contract_matches_scaffold_assets -- --nocapture`，尚未执行两条测试就以 E0433 失败。日常/tu 等别名均编译此包 lib-test，因而这不是“六个 crate check 零错”能覆盖的状态，也不是 814 已有错误：store_deps 测试正由本轮新增。

修复：把迁入测试依赖完整挂到新 owner，编译该包测试目标并执行 store_deps 回归与受影响定向测试，不能删测试规避。证据 scaffold-anchors.log.txt。AC-01/05，T-04。

### F-815-T04-R1-03 / P2：兼容模块拆分丢失旧公开路径

定位：`crates/auto-man/src/lib.rs:55,87`（直接模块再导出）、`crates/auto-lang/src/lib.rs:3473`（整族删除点）。合同 §5 的 T-04 要求旧 Man 公开门面投影；计划公开兼容标准也没有授权删除旧根级函数。

真实 Cargo consumer 有三组 E0432：auto_man::vue::{run_vue_project, run_vue_project_with_request, start_gallery_back_proxy}、auto_man::rust_ui::{run_vm_ui, run_vm_ui_with_request, start_vm_server}，以及 auto_lang::{extract_store_deps_from_file, extract_store_imports_from_ast, strip_line_block_comments}。单把同一个 consumer 改用新 compat 模块和 auto_ui_compiler::store_deps，check 即成功，证明实现仍在而旧路径断裂。把仓内调用点全部改名不能证明旧 API 兼容。运行主体等待 M814-B 合法，但保留旧公开转发无需等 runtime 迁移。

修复：通过无复制算法的公开转发保留原模块/根路径，补外部 consumer 编译断言，涵盖同一旧模块的全公开支持面。证据 compat.rs、compat-cargo.log.txt、compat-control.rs/log.txt、migration-inventory.json.txt。AC-01/04，T-04（T-06 完整兼容仍 pending）。

### F-815-T04-R1-04 / P2：资产唯一来源切换未同步旧消费者/锁与 oracle

定位：`scripts/ui-layout-parity.mjs:15`；`crates/auto-ui-compiler/src/ui_gen/nav_contract.rs:186`、`sidebar_contract.rs:231`；`crates/auto-lang/tests/gallery_golden.rs:131`；`examples/remote/viewer/pnpm-lock.yaml:12`。

生产资产 355 文件和 drawlist 包 11 文件逐字迁移成功，旧来源也确已删除，但上述现行入口仍读旧目录。布局对拍实际 exit 1/ERR_MODULE_NOT_FOUND，指向 auto-man/assets/wm/layout.ts。nav/sidebar 的断言读旧 shadcn 目录并在缺文件时 panic；本轮实际测试被 F02 更早的编译错误挡住，**没有声称这两条已经执行失败**。画廊来源围栏仍按已删除的 bundled_root 判断来源。viewer package.json 已切换新 file: 路径，pnpm-lock 的 importer 和 resolution 仍为 packages/drawlist-renderer，破坏 frozen-lock 一致性。

修复：扫齐实际读取、脚本、锁文件与测试入口并指向唯一新源，保持失败保护与逐字/token 对拍；不恢复旧资产副本，也不删除旧 oracle。修复 F02 后执行 nav/sidebar 两条定向锚、布局脚本与适用 consumer 检查。证据 layout-parity.log.txt、migration-inventory.json.txt 及已记录源码位置。AC-01/04，T-04。

### F-815-T04-R1-05 / P2：格式化改写 plan734 金样，新增对拍红

定位：`crates/auto-man/tests/plan734_commands_fixture.rs:22`，迁后断言 `crates/auto-api-gen/src/api_gen.rs:7226`。

b6aa3c21e 把 generate_handler! 的多行调用压成单行；生成器仍输出原多行文本。auto-api-gen scoped nextest 54/55，唯一失败为 test_plan734_tauri_fixture_matches_generator。实际失败中的 generated 文本与 git diff base 的旧 fixture trim 后完全相同，与当前 fixture 不同（plan734-control.json.txt），因此是本轮新增，不能按此前披露的“fixture 独立编译 5 错”豁免。

修复：保留真实生成器金样的逐字意义并解决格式化工具与生成夹具的关系，重放此断言；不要仅放宽比较掩盖漂移。AC-04/05，T-04。

### F-815-T04-R1-06 / P3：变更文件格式检查未通过

定位：`crates/auto-lang/src/lib.rs:3475`。删除 store_deps 块后留下额外空行。只对本次 A/C/M Rust 文件运行 rustfmt --check（skip_children=true）exit 1，唯一差异是此处。修复上述项后一并清理并重查。证据 fmt.log.txt。AC-05，T-04。

## 实际验证与覆盖边界

| 项目 | 本轮结果 | 判断 |
|---|---|---|
| auto-ui-tools scoped nextest | 187 运行：184 pass、3 fail、1 skipped | 三红为 index_css 金样、shell pack freshness、merged CRUD fallback，和实施披露同名；其测试体归一后均与基面相同（old-red-source-comparison.json.txt）。本轮未在旧基面执行，不冒称独立重复了 stash 对照。 |
| auto-api-gen scoped nextest | 55 运行：54 pass、1 fail、1 skipped | 新红 F05；跳过项不算 pass。 |
| Man 请求缝 + plan811 consumer | 5/5 pass | 仅选两个 binary 的相应测试，124 skipped；不代表 Man 全包。 |
| 外部旧路径 consumer / 新路径对照 | 失败 / 成功 | F03 具有区分性。 |
| 真实 bundle 生成 / 自身 metadata | 生成成功 / 失败 | F01；逐个补来源的诊断副本证明两个漏项。没有将 metadata 对照当成实际 build/locked 完成。 |
| 编译器资产锚定向测试 | 编译 E0433，0 条执行 | F02；F04 的其他资产链证据来自现存源码和真实布局脚本。 |
| 资产搬迁 SHA256 对比 | gallery 1、wm 9、shadcn 345、drawlist 11 旧文件均相同 | 新 MIT 文本、清单及旧副本清空测试随包检查通过；额外复制不作为解法。 |
| 777 strict | pass，0 violations / 4 grandfathered | 不代表发行闭包/API 兼容通过；未刷新基线。 |
| diff --check / 变更 Rust 文件 fmt | pass / F06 | check 成功不等于无 warning；未处理依赖既有 warning 未冒称全仓零告警。 |
| 窄工具独立 fixture | 闭包 PASS；真实进程 4/4 pass | 原生 Cargo 重放同一精确闭包断言；首次链接盘满，回收本轮生成探针自己的两个 PDB 链接后重跑通过，原始失败和重跑日志均保留。 |

按 T-04 原范围对 AC-01 partial/fail（主体迁移和资产字节通过，公开兼容/测试闭包失败）；AC-02 当期请求缝定向 pass，未完成 B 的所有路径仍 pending；AC-04 已触及发行/兼容/旧 oracle fail。AC-03 与 T-05 实际交互/cancel/runtime 未审；AC-05 全计划终审 pending，新增卫生 F06 已开；AC-06 归最终 merge；AC-07..12 的已发布历史范围保留，本次仅重放受新增依赖影响的窄工具面。

本轮未执行整档 t/tv/tt/tb/tu/taa/tf。代码、验收与规范最终 pass 后的 merge-owned 冻结门仍 pending；当前未进入合入，也未发布 B、写 canonical/live ledger、归档或清树。

## 下一步

同一 815 树按六项一次修齐，补永久消费端/清单闭包/新 owner 测试与资产读取负控，定向验证后提交新 SHA 再送审。无需等 M814-B 才能修复这些 T-04 缺陷；T-05 的既定外部门保留。T-08 的最终规范增量与 T-09 完整独立验收继续未勾。本轮规范裁量与冻结候选见 [spec-delta-reviewed.md](spec-delta-reviewed.md)。

复现命令与辅助源码：probes.py（Cargo consumer/真实生成入口）、controls.py（仅诊断副本补两个来源与新公开路径正控）、fixture-closure.py、source-audit.py、migration-inventory.py、plan734-control.py。首次手工 rustc 探针因依赖/本机链接搜索配置不足没有形成有效验证，最终证据均以上述 Cargo 结果为准，未把工具环境失败记为产品 finding。
