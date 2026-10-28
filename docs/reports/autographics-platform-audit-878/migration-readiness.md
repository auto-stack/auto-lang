# 5. 纯迁仓评估（migration-readiness）

对迁移设计文档 `docs/design/autographics/repository-move-and-integration-gates.md`
（审计时为 master 工作副本、尚未入 git，SHA-256 前 16 位 `6b43f3b00efe39d4`）
建议形态的 XP-0 只读评估：AG 移入 auto-ui 仓
`experimental/autographics-{prototype,auto-driver}`、保持嵌套独立 workspace/锁/旧 SDK。
本文件不执行迁移、不修改 auto-ui、不分配迁仓 Plan。

## 1. auto-ui 目标侧事实（固定对象 `7095f536…`）

| 触面 | 观察事实（固定对象） | 对迁仓的含义 |
|---|---|---|
| workspace 形态 | 根 `Cargo.toml`：`members=["crates/*"]`，resolver 2（blob `6d528f03`） | 嵌套独立 `[workspace]` 的 AG 目录**天然不进默认成员**；零默认构建负担 |
| 语言 SDK 边 | runtime 直接 path 依赖 `../../../auto-lang/crates/{auto-lang-core,auto-parser-api,auto-val,auto-media-core,auto-text-core,auto-terminal-core,a2r-std}`（**活 master 检出**）+ optional autodown-core | auto-ui 消费**活的**语言仓；AG 消费**冻结的** `.sdk-source` 副本——两条 SDK 消费路径不相交，AG 迁入不改变 auto-ui 的 SDK 边 |
| AG 引用 | `git grep -il autographics 7095f53 -- crates/**` = **0 命中** | 无反向依赖、无路径假设；迁入无消费者破坏 |
| iced 线 | runtime `default=["ui-iced"]`（iced 0.14 系，PLAN-814 自语言仓迁出） | 与 AGP 的 stock iced 0.14 同 0.14 线但**不同消费形态**（生产 patch 在语言仓/生产栈，AGP 用未打补丁库版）——版本相近 ≠ 共享设备/资源 owner（迁移设计文档 §1 已警示） |
| 831/842 | 831 archived 12/12、842 archived 17/17（规划事实，现行 Spec） | auto-ui 独立交付链已稳定运行 |

**无语言→UI→语言重环判定**：auto-lang 不依赖 auto-ui（AG 源只是语言仓 experimental
下的**非成员**目录）；auto-ui 依赖活语言仓 SDK；AG 依赖自带 SDK 副本。AG 迁入
auto-ui 后：auto-ui 的默认 `crates/*` 依赖集不变（AG 不进 members），AG 的新位置
不引入任何指向 auto-lang 的边（其 SDK 边内嵌 `.sdk-source`）。**判定：默认 UI 零
新增依赖/构建负担成立**（基于 manifest 静态事实；auto-ui 构建未重跑——其交付 Spec
与 831/842 收据为准，本审计不重证）。

## 2. 迁移触面清单（保持面/需修正面）

按迁移设计文档 §4"候选布局不保证脚本原样工作"，以下触面在实际迁仓 Plan 中必须逐项盘点：

| 触面 | 当前位置/形态（8b12632d 实测） | 迁仓影响 |
|---|---|---|
| 固定源 root | auto-lang 仓路径 `experimental/autographics-*`@`8b12632d`；12 blob 身份 + 全树 | 新身份映射"旧 repo+commit+path → 新 repo+commit+path"；旧 commit 不是新仓祖先，不得重写 hash |
| SDK 来源 | driver `.sdk-source`（内容锁 681 文件，pin `7b987ca8`）；`tools/prepare_sdk.py --repository <auto-lang>` 从**语言仓 git 对象**重建 | **pin 对象在 auto-lang 已不可达**（cat-file 失败）——迁仓后若副本丢失将无法从 git 重建；迁仓 Plan 必须先解决副本的长期保全（锁定副本或恢复对象可达性） |
| 身份/导出工具 | `verify_source.py`（cargo metadata 闭包断言：target 不逃逸、17 包、29 path 边、AGP 锁身份保留） | 断言逻辑与新路径布局兼容（相对 root 解析），但**未在迁移后布局重放**——迁仓验收必须重放 |
| 构建栈配置 | `.cargo/config.toml`、rust-toolchain 若在（本仓根栈） | 检查两 workspace 是否继承语言仓根配置（独立 `[workspace]` 已隔离锁；根栈 alias/环境需盘点） |
| 测试夹具 | driver `tests/fixtures/**`、AGP `testdata/`、`assets/`（含 .ogg/.glb） | 小夹具随迁；体积大的资产按迁移文档 §4"不因体积推测删除"，需 LFS/保留策略决定 |
| 受控 runner 来源 | driver `tools/`、`.agents/skills/autoui-verifier`（语言仓）对 AG 的引用面 | 工具脚本内的路径假设（相对/绝对）逐个核对 |
| 文档链接 | 语言仓 specs/design → `experimental/` 相对链接 | 迁仓后改为跨仓链接或新归属索引（语言仓仅留历史/索引） |
| 历史收据 | `research/**`（57 个交付/身份文件）、plan-860 消费包副本 | 历史证据保留原处或显式归档，不当生产包搬移 |

## 3. 门分离 reaffirm（四件事不得捆绑）

| 门 | 是否因本审计自动放行 |
|---|---|
| 纯源码归仓（搬仓 Plan） | **否**——本审计只提供触面清单与 R-SDK 保全风险；时机建议维持"874/875 实际交付冻结后、下一批公共运行协议开发前"的短暂停写窗口，另立独立 Plan |
| UI/Graphics 运行与声明融合 | 否——等 UI 领域/组件/资源合同与实际执行环境（ACE 路线）各自成熟 |
| ACE 替换旧 SDK | 否——AG 现消费 pin 7b987ca8 旧 SDK；替换是独立增量/状态/HMR 门 |
| 切默认/退役 | 否——需兼容矩阵/回退窗口/消费者迁移合同 |

"归仓与融合一并等 ACE"是**现行历史排期**：迁移设计文档 §3 已将其调整为"纯归仓在
技术上可先于 ACE 成熟"，本审计**确认该调整在依赖/环/隔离事实上成立**（§1），
但不取消、不提前任何门；最终排期是用户决策。

## 4. 迁仓研究边界声明

本评估未运行 auto-ui 或 AG 构建/测试（XP-0 禁令）；"默认 UI 零新增负担"是
manifest/workspace 形态静态判定，非构建实测。迁仓执行（若获安排）需独立 Plan：
两仓唯一树、停写窗口、切源顺序、可恢复快照、迁移前后身份/资源/分母可追溯。
