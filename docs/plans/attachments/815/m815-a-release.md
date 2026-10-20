# PLAN-815 M815-A 阶段发布收据(T-14)

2026-10-20。**收尾核验更正：T-14c ledger_refreshed blocked，阶段发布条件尚未全部满足。**
详见 [收尾核验与恢复方案](t14-closeout/recovery.md)。代码/Spec已合入，原冻结门结果保留；
原机器可读JSON仅是历史执行副本，不能覆盖本更正。

2026-10-20。本收据是 **M815-A 阶段发布**(非整 Plan 交付):Plan 保持 `executing`,
唯一树/分支保留并同步 master 继续 B;不归档、不清理。机器可读副本:
[m815-a-release.json](m815-a-release.json)。

## 1. 身份

| 项 | 值 |
|---|---|
| plan / revision / phase | PLAN-815 / r2 / M815-A |
| delivery_commit | `e609caf1af511b96832f511ab9d1fa6c0c11f563`(master,`git merge-base --is-ancestor` 验证通过) |
| reviewed_commit | `7045e20fa71ba437bc80075f17d9aace755f5abd`(R4 stage pass) |
| 原始 diff base | `b9f7ab7e1`(27 files,+5289/−2) |
| 依赖交付 | 809/I-PROJECT = master `b819a602e`(已归档);无 814 WIP 依赖 |
| 真实工具二进制 | SHA256 `1dc1ec700c7dfdb310e8d61b8250909b6c4549548288daa8de213b69badeb07a`(与 R3/R4 审查钉定逐字一致) |
| 关键源指纹(SHA256 前24) | framing.rs `476df592e83ee24c3ec4c540` / wire.rs `717749ea88816d29aa9c6ad9` / validate.rs `60d5eb2012d625a94fbffc21` |

## 2. 发布范围(816 可消费的已发布窄能力)

- **协议版本 1**(auto-tool-protocol):describe / validate_project;`cancel` 为
  协议控制消息(NotCancellable/InvalidField 语义)。预算:帧载荷 8MiB(不含前缀)、
  进度 1MiB/op(预留)、队列 128(预留)、wire 深度 120 / 完整帧 122 / 解析器 128。
- **工具**:auto-ui-tool 真实进程(stdio 帧化;日志仅 stderr;退出 0/1/2)。
- **数据**:GenerationRequest 同源 wire codec v1(auto-project-model feature
  `wire`)。**未实现且明确拒绝**:generate/build/run 等一切 UI 生成能力
  (unknown-operation,零副作用)。

## 3. 冻结门(T-14 在冻结提交上执行)

| 门 | 结果 |
|---|---|
| 裸 `cargo t`(worktree @7045e20fa) | **5303/5303 passed**(117s;1 slow>60s 无碍;前两轮 ac_case/probe_adapter/plan499 失败归因为并发负载 flake——隔离重跑全过,与本 diff 触面无关[auto-lang/auto-man/auto-vm 0 文件]) |
| fixture 闭包 | verify-deps.sh PASS(无 auto-man/auto-lang/auto-vm/auto-gen) |
| fixture 进程 | 4/4(AUTO_UI_TOOL_BIN 真子进程) |
| 777 strict | pass / 0 violations / 2 grandfathered / 75 edges |
| 触面判定 | tv/tt/tu 零触发(diff 不含 VM/trans/UI 生成代码) |

## 4. Spec/ledger 受管增量(非归档)

- SD-01-A add `docs/specs/auto-ui-tools/project.md`;SD-02-A add
  `docs/specs/tool-protocol/project.md`;SD-04-A modify
  `docs/specs/project-model/project.md`(wire 节);SD-06-A modify
  `docs/specs/repository-boundaries/generated-ui-distribution.md`(A 来源边界节)。
  内容按 R4 校对稿 [spec-delta-reviewed.md](review-r4/spec-delta-reviewed.md)。
- ledger v56→v57 原写入仅核对JSON/ID，**未通过store schema，不算受管发布完成**。
  四条 P815-A-{D1,T,R,REL}缺少工厂字段，三条active状态非法；
  v56备份本身也含1060条不兼容旧记录及六个不完整section。
  原件/备份保留；修后脚本拒写，完整条目仅为草稿，等待独立全库恢复。
  spec-index 重生成(50 projects);spec-lint 唯一 ERROR(design/ 缺
  project.md)为 PLAN-777 预存,非本轮引入。

## 5. consumer 解析路径(读回验证)

- 协议/数据 crate:workspace 成员 `crates/auto-tool-protocol`、
  `crates/auto-project-model`(features=["wire"])——816 经 path 依赖直接解析。
- 工具二进制:`cargo build -p auto-ui-tools` → `target/debug/auto-ui-tool.exe`
  (SHA256 见 §1);fixture 门以 AUTO_UI_TOOL_BIN 指向该产物验证真实子进程消费。
- 816 放行条件:本收据 + R4 pass + 冻结门齐备(§3)——**ledger项阻断，尚未全部满足**;816 不得静态
  链接 auto-ui-tools 实现进 CLI。

## 6. 未交付(B 阶段;不因本收据宣称)

生成/build/run 主体、真实 runtime 集成(等 M814-B)、资产物理迁移与 embed 切换
(等 M814-A)、旧 Man 门面投影、tool-protocol-consumer 独立 fixture、完整
V-01..04、SD-01..06 完整 B 面。M815-B 在同一树(`D:/autostack/.wt/lang-815/
auto-lang`,分支 `codex/plan-815-dev`,已同步合入后 master)继续。

## 7. 收尾身份补充

完整输入及四份canonical Spec的SHA256见 [source-manifest.json.txt](t14-closeout/source-manifest.json.txt)，
含按路径排序的聚合指纹及R4校对稿hash；ledger原件/备份/schema来源SHA见
[ledger-schema-audit.json.txt](t14-closeout/ledger-schema-audit.json.txt)。四条修正目标仅为
[待发布草稿](t14-closeout/ledger-items-proposed.json.txt)，未覆盖实际账本。
