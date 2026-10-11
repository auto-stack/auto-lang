---
plan_id: PLAN-763
plan_revision: 1
status: drafting
feature_name: consolidate-root-packages
created_at: 2026-10-11
updated_at: 2026-10-11
supersedes_spec_components: [docs/specs/design/repository-layout.md]
new_spec_components: []
touched_goals: []
affects: [aavm, blueprint, parity, website]
current_step: 0
total_steps: 5
---

# [PLAN-763] 根目录第二轮归并与未维护 CI 清理

## 变更摘要
用户已授权检查八个 Actions 工作流，用处不大则删除；其余目录按此前建议合并。本计划落实已有授权。保留历史提交、测试覆盖、标准库与本机配置。

## 目标
Git 根树从 30 项减为 24 项；完整保留五组资产及消费入口。

## 架构方案
auto/ → packages/aavm/；blueprints/ → packages/blueprints/；parity/ → test/parity/；schema/ → docs/schema/；patches/ → crates/patches/。stdlib 独立；逻辑模块名 auto.lib 与 bps 保持不变。隐藏工具目录保持自动发现位置。

## 需求分析与背景调查
依据 docs/specs/overview.md、design/repository-layout.md 及各模块现有实现。七个测试/构建流最近五次均失败，网站部署零次执行。删除八个旧定义，不新增或触发部署；本地验证命令仍可运行。提交历史能确认加入时间和关联计划，不能证明操作者是 AI。

## 详细设计
代码修改与验证在 D:/autostack/.wt/lang-763/auto-lang；主检出仅计划簿记。移动同步 Cargo patch、schema include、AAVM 清单、blueprint 发现与 pac 依赖、parity 仓根推导和脚本。检查活跃仓内与仓外消费者；不修改别的会话检出，不删本机缓存。冻结历史计划/报告保留旧路径；现行 Specs 与入口采用新路径。

### 规范增量
修改 docs/specs/design/repository-layout.md：24 项布局、五处新归属、当前无 GitHub Actions 自动部署/测试，本地门禁继续执行。同步受影响模块现行路径与索引，不改变语言/组件/协议语义。账本按 docs/specs/README.md §5 兼容期回退做保守投影，保留所有无关条目。

## 测试设计
逐文件映射/哈希证明资产保存；扫描旧路径并逐项解释；cargo check -p auto-lang；cargo t、tv、tu、tt、tb 及 docs_gen 对应触面；parity 独立 workspace check/test 与 CLI 冒烟；AAVM 按 AGENTS 跑 taa（遵守重型档主检出单实例纪律）；blueprint 命令与示例生成冒烟；Cargo metadata 确认 patch 来源。按证据区分预存失败，不把未验收条目当通过。tf 合入时检查到期规则。

## 验收标准
- AC-01：八流程的来源、触发条件、远端记录与删除理由有持久报告；八定义删除，无自动发布替代。
- AC-02：五目录无丢件；Git 根树 24 项，stdlib 与隐藏共享配置保留。
- AC-03：编译、blueprint/schema/AAVM/parity 入口与 Cargo patch 消费新位置；依赖命名及资产语义保持。
- AC-04：验证、复审、规范投影证据绑定实际提交；预存问题独立记录。
- AC-05：线性合入、归档、guard 清理专属工作树；保护其他会话及本机缓存。

## 执行步骤
- [ ] T-01：冻结八工作流来源与 GitHub 记录，形成 docs/reports/763-ci-audit.json。
- [ ] T-02：迁移五目录，同步消费者，形成资产映射核验。
- [ ] T-03：完成作用域验证与冒烟，保存结果和失败归因。
- [ ] T-04：提交实现，以 auto-plan:review 从产物重建验收结论（同会话复审，明确限制），审定 Spec delta。
- [ ] T-05：auto-plan:merge 完成规范/账本、ff-only 合入、归档与 guard 清理。

## 复审记录
待执行。基面 4aa461d1c0765436ab974817ee4ad7ebb8ba719c；外部依赖在工作树创建后记录。

## 待澄清事项
无；已有授权。先审计仓外消费者，保护其他会话检出。
