# ace-migration: replacement scope and evidence

> Status: proposed (PLAN-922 SD-01/SD-02)。本文件只规范 M2 替换盘点的**口径与证据边界**，
> 不把任何条目标为 implemented；实现状态以各模块 Specs 与已交付计划收据为准。
> 来源：[PLAN-922](../../plans/archive/922-ace-m2-replacement-inventory-and-critical-path.md)（merge 时按实际路径修正）、
> [M2 合同 r2](../../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)、
> [v0.6 发布门增补](../../plans/reports/ace-v06-m2-release-gate-2026-11-02.md)。

## SD-01 全场景分母、分类与证据边界

### 分母构成（迁移台账的强制行集）

M2 替换分母 = 当前真实可达的：

1. **语言公共入口**：auto-lang facade 全部执行/转译 pub API、CLI 子命令与 runner 分派、
   REPL/持久会话/daemon、测试工具入口；
2. **领域消费者**：AutoUI（全部已支持模式/后端）、AutoMan（全部命令/求值/生成）、
   auto-web（服务宿主链）、AutoGen/SDK/模板、其余仓与安装物中真实调用旧引擎者；
3. **全部在用 A2X 目标**：Rust/C/Python/JavaScript/TypeScript/GDScript/Godot/TSCN 与
   r2a 逆向工具，按真实生产/测试/研究归属区分；
4. **发行/安装面**：默认与 all-features 构建、注册/feature/隐藏开关/fallback、
   包内二进制、安装/重定位/升级/回滚路径。

规则：以"入口 → producer → consumer"双向可追溯为准入；grep 零命中、目录名、
examples 计数均不构成覆盖证明。跨仓不可读/owner 未明/未发布安装物按具名 unknown
登记，不填"支持/无差额"。

### 状态分类（每行必选其一，禁混合百分比）

| 类别 | 判据 |
|---|---|
| implemented-verified | 新 ACE producer 存在且同一来源身份下有已执行独立 oracle 证据 |
| implemented-unwired | producer 已实现但该入口/consumer 仍走旧引擎（接线缺口） |
| evidence-stale | 声称已迁但证据非当前祖先/来源身份不适用/未重验 |
| capability-missing | 新引擎无该能力（须给最小真实工程切片与独立 oracle） |
| unknown | 未盘点/不可达/依赖未发布（须给 owner 与下一步只读核查） |

### 证据边界

- 运行证据必须绑定 repo/SHA/入口身份；历史 M1 收据对 M2 只作输入不作完成证明。
- "审计完整" ≠ "产品通过"：本口径下的任何登记都不替代 REP/V06 门的实际执行。
- 旧 required 静态格、M1 corpus（binding.tsv）、新 C/M2 门分账保持，不得互相冲销。

## SD-02 新旧义务、中立库与 disable/安装证据归属

- **义务迁移**：旧执行/转译义务经版本化 successor ledger/recipe/cert 迁移到新
  producer；原 source/expected/批准 NA/accepted 不改。旧 golden/证书不冒充新 producer
  身份；disable 后新版日常不为对拍重建旧引擎，追溯靠 Git 历史、独立已发布旧版本与
  行为 oracle。
- **中立库归属**：共享值类型/诊断/词法与文本工具/中立 runtime/native 静态库/渲染器/
  仍需的 Rust 库按真实依赖保留或迁中立 owner；判定依据是"是否夹带旧执行循环或旧
  AST 发射器"，不是目录名含 vm/trans。薄兼容 API 只能转发 ACE，不得包装旧算法。
- **disable/安装证据**：旧实现退出以 default+all-features 依赖图/构建单元/真实
  dispatch/发行包内容/日常测试选择为零为证；安装证据要求 clean 安装/仓外/重定位/
  升级/独立旧版回滚的实际收据，dev worktree、未发布绝对路径、本机旧缓存不算 fresh。
- **owner 交接**：跨仓迁移单写窗由对应仓 owner 以正式合同认领；盘点行只登记接口
  变动与需重核 consumer（带 SHA），快照不复制成新的永恒状态源，后续正式 SHA 到来
  只刷新受影响行。
