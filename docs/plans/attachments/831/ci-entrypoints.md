# PLAN-831 T-09 实际 CI / 本地门入口清单

绑定 [handoff.md](handoff.md) §1 三仓 SHA（R4 分类收口终态：lang `538e3b8fe`、UI `4250885`、down `c39ac7f9`），日期 2026-10-24（R1 刷新）。两实现仓 `git ls-files .github/workflows` 均空，无受管远端 workflow；本地受管入口即 CI 入口。**跨仓编排已落地**（cross-repo-tiers.py，F831-09-04 闭合）。

所有命令在列明的原权威树运行。组内 auto-down 为只读固定 SHA。不得创建链接绕过。

| owner / cwd | 受管入口 | T-09 修复轮状态 |
|---|---|---|
| lang | `python scripts/ui-repository-fixture.py --phase source-switched` | 终态 46 pass/0 fail/1 recorded/1 pending（收据 scratch-ui-repo-fixture/t09-final-all.json）；recorded=SRC-P4c 清洁面复核（无残留）；pending=TYP-PROBE 零 cargo 缺省——专项跑见下行 |
| lang | `python test/fixtures/modularity/ui-repository-consumer/driver.py --phase source-switched --compile-probe type` | F831-09-02 闭合跑：24/24（TYP-PROBE=pass：A default+run/B core-only/C core+显式UI 三组合实编+单源身份运行断言+双 source 负控拒绝；收据 t09-f02-type.json）；selftest 19 检（S17 收据身份突变负控） |
| lang | `python scripts/modularity-audit.py --mode strict --no-cargo-check` | F831-09-04 闭合：跨仓 ui_to_language 围栏真实化（auto-ui 仓 362 文件、code_fence_zero）后 strict pass 0v/5gf **零警告**；基线锚定 77114ec6f（自 5339085ea 重采样链） |
| lang | `python scripts/cross-repo-tiers.py run` / `... list` | 新建（F831-09-04）：三仓档位编排实际入口——run=串行队列 lang-t→lang-tu→ui-t→ui-tu→ui-ti 首红即停+三仓身份收据（重型档/tf 不进队列）；list=各档正典 ID 面（只构建不运行；迁前后对拍工件 tier-parity-f831-09-04.json：三共享包逐 ID 相等 247/86/1872、tu 863 集合恰等） |
| lang | `python test/fixtures/modularity/generated-ui-consumer/driver.py`（792 driver） | F831-09-03 闭合重跑：38/39（收据 scratch-consumer/driver-receipt.json，sources 三仓齐记经 F831-09-05 admission 采信）；唯一红=V06c jet/ark【环境阻塞·具名】（本机无 gradle/Hvigor，非静默跳过）；V06a 生成 Rust 应用真实窗口 run、V06b 六 profile 真实生成、V06c vue/rust/vscode 真实构建链 outcome=completed、V06d 安装中取消+重试恢复、V06e vm 解释运行 |
| lang | `python scripts/ui-repository-distribution.py` | 终源重跑 5/5（收据 distribution-receipt.json：sources 含 down 齐记；fresh/locked/重定位/隔离 SDK） |
| lang | `python scripts/ui-repository-switch.py drill` | 终源重跑 19 检全过（收据 switch-drill-receipt.json：sources.lang=工具源仓 HEAD；真仓 land/switch 归 T-11） |
| lang | `python scripts/legacy-preservation.py inventory/preserve/verify/recover-drill` | 沿用 T-02 历史保全事实（preservation.md），本轮未重新 preserve |
| lang | `cargo test --manifest-path test/fixtures/modularity/<probe>/Cargo.toml`（ui-package 等 5 探针） | 路径已切 UI 兄弟；本轮经 DST-4 隔离 clone 内 SDK consumer 真实 build+test 复验 |
| UI | `cargo t` / `cargo tu` / `cargo ti` / `cargo tn` | t=五包 lib（ui_gen 经 .config/nextest.toml 排除——对齐原语义）；tu=ui_gen 单进程；ti=集成档（tools 四目标+osconfig；**实跑 28/28**，15 skipped=ignored heavy 面按门语义归 --ignored 证据跑）；tn=native 手工档（native-dnd+test-native-dock 双 feature；fastrand 泄漏修复后 2 测试可列举，真光标交互仍手工门） |
| lang / 自动化实机 | `.agents/skills/autoui-verifier/scripts/test_vm_mcp.py --auto-bin <树>/target/debug/auto.exe`（+`AUTO_UI_TOOL_BIN` 钉工作树工具） | F831-09-03 寿命矩阵三应用全过：001-helloworld（窗口）/041-auto-edit（终端族）/020-music-player（媒体族，150s 真实时延窗）；**bin 必须钉工作树**——PATH 命中主检出陈旧 auto.exe（缺 T-07 output_root 锚）/auto-ui-tool.exe（缺 run_ui 操作面），均为未合入分支的主检出既知事实 |

## 原档位与目标责任对账（T-09 修复轮后状态）

- 语言树 t/tt/tb=auto-lang+auto-lang-core；tv=auto-lang；tu=auto-lang（ui_gen 面 0 选择——基线同形态，测试全住 compiler 包，两侧对拍一致）；tf/taa/ta/t3 经 path 依赖 `-p` 仍覆盖 UI 五包（`cargo pkgid -p auto-ui-compiler` 双向解析在案）。
- UI 树 t/tu/ti/tn 四档 + `.config/nextest.toml`（ui_gen 排除/slow-timeout 60s·2；无登记重内存测试→无资源队列，机器闸门归语言仓 heavy 族）。
- 迁前后正典选择面对拍：[tier-parity-f831-09-04.json](tier-parity-f831-09-04.json)——t 面三共享包逐 ID 相等、tools+shell-pack 为超集（原 t 未选）、tu 面集合恰等（基线 867 含两条双 #[test] 重复注册=现 863 唯一）、ti=28/tn=2 增量档记录在案。

## 门所有者与顺序

work（本轮已完成）：F831-09-01..06 全部闭合（证据见 handoff §5 与本表）；定向验证含 ti 实跑、编译探针三组合、777 strict、792/distribution/drill 终源重跑、三族 MCP 冒烟。未运行整档 t/tv/tt/tb/tu/taa（合同后置）。

T-10：独立审核最终代码（lang `538e3b8fe`/UI `4250885`）、全部 work-owned AC、SD 与收据身份；pass 显式保留 merge 门 pending。T-11：冻结三仓输入执行语言裸 cargo t 及实际触面档、UI 适用门并实际落地/读回。T-12：归档沉淀；tf 到期仍主语言检出单实例。
