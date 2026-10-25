# PLAN-832 T-09 交接与 readiness（jet/ark 环境项未决）

状态：**work 材料齐备，但不送整体 execution_done / final pass**。AC-03 的 jet/ark 真实构建仍缺环境，T-10 不得据此启动整体终审。

## 1. 当前身份

- Man：`356f2c8186d07bc18567b18179c3cad7bda2d483`（生产/CI）；`3f85c634448a642b264a684d896f9e5bbe18356b`（独立发行矩阵）。
- lang：切换 `ba1ce5901d76643a8878f5d0f4663c3ca4904ea5`；规范增量在本 worktree 后续提交。
- UI：`aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- full auto SHA256=`eed5af33b2bfeef0d4f21a63a56c1ec82c5c4e3e2e3253ef6fb9292b8a2766e0`；auto-ui-tool SHA256=`d74eba1298943ad9c2160206cf83b0fa5e417955b00b0700cc0fcbb234cfbc49`。
- 收据基线：Core 643 files、full 6608 files；根 Cargo.lock 与安装 manifest 均受管，确定性身份=固定源+锁+清单，二进制 hash 仅为参考产物。

## 2. AC 结果与证据

| AC | 结果 | 当前证据 |
|---|---|---|
| AC-01 Core/单源 | pass | [T-05](t05-verification.md)：normal/build/dev 无 UI、安装 SDK locked check、真实管理/VM；[切换 consumer](t08-source-switched.json.txt)；[最终来源](t09-source-audit.json.txt) |
| AC-02 命令/API/寿命 | pass | [T-06](t06-verification.md)：client 30、CLI driver 7、compat 33、service/watch/退出；旧新失败阶段一致；24/122/4 测试 ID 逐字一致 |
| AC-03 三仓 UI/安装 | **partial / open** | full 安装 6608 文件、真实三仓生成/build/run、独立 clone 14/14、792 37/37；**jet/ark 真实构建缺 Gradle/Hvigor，未获 832 延期授权** |
| AC-04 发行/恢复 | pass | [T-07](t07-verification.md) 14/14；[取消恢复](t07-cancel-recovery.md)；792 fresh/locked/重定位/坏源/取消/恢复通过 |
| AC-05 历史/事务 | pass | [保全](preservation.md)：29 refs/23 tags 恢复、tree 一致；目标先行→消费者切换→验证顺序已执行 |
| AC-06 删除/测试/CI/SD | pass | 9 package+6 fixture 实际删除，185 迁移文件均在 Man；[CI](ci-entrypoints.md) 323 目标；777 0v/4gf、资产 20/410397B；SD-01..06 已写 canonical 并重建 61 项索引 |
| AC-07 独立 final review | pending | 仅在 AC-03 未决项解决后进入 |
| AC-08 merge | pending | 冻结裸 cargo t + 触面档、落地/ledger/归档/guard 均未执行 |

## 3. 全部 V 复核

- V-01：Core 独立安装、真实管理/协议执行、单一 source，pass。
- V-02：完整命令/flags/JSON/stdin/路径兼容，pass；默认 run 与 split-demo 的旧新失败阶段也一致。
- V-03：固定三仓 UI 工具生成/build/run、首次开发与安装，**jet/ark 构建 pending**。
- V-04：fresh/locked/重定位/升级/坏锁/hash/source/patch/半安装/取消/交换与恢复，pass。
- V-05：legacy refs/WIP/ignored 保全、恢复演练、切换失败保留，pass。
- V-06：service vm/rust 配置与 ready、serve/req、watch/HMR、reader/child/线程寿命和测试 ID/CI，pass（Rust service 的真链证据沿 816 既有与 T-06 配置/VM 实跑，未冒称本轮重跑冷 Rust service）。

## 4. 来源与统计

[t09-source-audit.json.txt](t09-source-audit.json.txt)：185 个迁移文件全部在 Man、语言仓全删除；173 same/10 manifest/1 hygiene/1 source-path rewrite，零缺失/异常漂移。

- 生产/测试迁移源：185 文件，属于搬仓而非净删除。
- 另补测试资产：199 pac.at + 12 autoui-skill + 54 project-model + 9 媒体 fixture=274。
- 生成物：target 不作来源；独立锁和安装 manifest 重新生成，不充当第二生产实现。
- 第三方：未新增 vendored 实现；crate 内嵌资源与许可保留。

## 5. 验证执行与复用边界

本轮实际执行：独立发行 14 项、切换 consumer、Core/full 安装及 SDK locked check、管理库/CLI/compat 定向测试、777 strict、Man 档位列举、792 37 项、Vue/VM 实机、取消/回滚恢复。merge 统一裸 cargo t 和触面档 **仍 pending**，未以 scoped 检查替代 merge 门。

未变输入复用：815/816/831 的协议/owned 生命周期与 UI release 收据；T-05/T-06 的 Core/full 安装和 compat 证据；T-07 的独立 clone/恢复证据。代码、依赖、配置与输入未变处才复用，未宣称机械重跑所有冷构建或全档。

plan734 tauri fixture 仍呈现上游已登记的排版金样差异；文件路径已切到 Man 且可读，非本迁仓缺文件/新回归，不纳入通过计数。

## 6. 唯一开放项与解除动作

**OPEN：jet/ark 真实构建环境。** 当前仅生成通过；本机无 Gradle/Java/ANDROID_HOME 与 hvigorw/DEVECO。832 未继承 831 r3 的延期授权。解除方式二选一：

1. 提供真实工具链并执行 jet/ark 构建及产物检查；或
2. 用户正式修订 832 的验收合同，明确该面的后续环境门控渠道。

在此之前 Plan 保持 executing，T-09 不勾、T-10 不启动、不得合入或宣称 M1 完成。其余工作 owned 项无已知开放 finding。
