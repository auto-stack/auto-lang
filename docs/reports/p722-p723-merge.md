# PLAN-722:r1 / PLAN-723:r2 合入准备与收据

2026-10-01用户授权本会话全部修改合入master。网站合入与规范账本发布分别记录；不把账本未成功读取当已归档。

## 准备与线性同步

主线准备基线142458d21ad25c434880cb9d1998911ab08d327d。719代码与规范已在master（plan-719-dev为master祖先）；715/718/720已归档，其他会话716修复与账本改动均保留。

722原reviewed_commit bb8bcefd389589f0522fc0b23951843bb975a8ab；723:r2原reviewed_commit 137bc5557855c7757a0f3754ed4466226fa752f3，receipt_commit 62eb4886f9f9b20e930dc681a061db6a4de4bb84。723沿用其专用worktree并git rebase master；原内部ce280b0a3合并在rebase中展开成线性依赖序列，没有新增主线merge commit。

`git range-diff 42fa2e0c0dfc666001f591f69fbe7ce5d688e06e..62eb4886f9f9b20e930dc681a061db6a4de4bb84 master..HEAD`：七项全部`=`。

| 原提交 | 等价重放提交 |
|---|---|
| 3fd4562da | d7b881e6c64ab7eaea46fc8049b2cb1f063568cb |
| a4456ad52 | 91689176b68b42d45cc4ab6b86dea86326cdd780 |
| bb8bcefd389589f0522fc0b23951843bb975a8ab | 939497eadc5b189aca61a10c91387c6f0e7a32d6 |
| 19beb3046fc3d888000514e651071ad4f2748f8c | d2d8fb2f7e843f91d3776150639be3117c780525 |
| a35a908a70ffe38abe0fa051f0465cb2270c9435 | ddb3dfce448233b420fda89182b3c687d67f17e3 |
| 137bc5557855c7757a0f3754ed4466226fa752f3 | fd2d09c58d763737dbe13090fe8ba8ff84a5c955 |
| 62eb4886f9f9b20e930dc681a061db6a4de4bb84 | 1cfd65993d3b35107af5a1fec029f1325707211e |

原723 receipt→重放tip的website及四Spec目标diff为空；未修改实现、依赖或测试配置，可复用177.57s最终构建和70 passed(3.2m)的验证。722八专题与guide的88项原证据保留；723覆盖后续截图与概览合并，不把722无图要求套用到批准的723新阶段。

## 规范与派生目录

四Spec沿p723-r2-spec-delta.json全部hash匹配：website/project.md、design/application-introductions.md、design/application-demos.md、design/demo-capture-catalog.md。canonical真实实现与r2契约吻合。website/plans.md只追加722/723交付历史行；scripts/spec-index.py重新生成。目标推进属于现有网站范围，不创建假Goal。

## 账本阻塞的实际证据

没有注册可调用Spec connector。启动既有官方musk.exe调试程序（2026-09-29产物），`serve --addr 127.0.0.1:4248 --workdir D:/autostack/.wt/lang-723/auto-lang`，使用真实workspace store读取GET /api/specs：HTTP500，`{"error":"failed to load specs"}`。当前受跟踪.autoos/specs.json缺project/version，条目为历史本地投影；与现有SpecsDocument{project,version,sections}不兼容。8211服务/api/specs为404，不当作可用store。

依auto-plan-merge“Load failure means stop”不发POST、不手工覆写/删除/迁移账本，不丢其他计划的条目。719已有同类归档待writer检查点；本轮722/723代码与canonical可落地，ledger_refreshed/archive/cleaned仍待兼容迁移或可读store，保留reviewed计划和工作树。不是要求用户重新批准已有网站修改。

## 产物与回归边界

网页已验证静态产物保留在723工作树，由4235预览服务读取；主检出website/.vitepress/dist/index.html不存在，本轮未公开部署。无后端/依赖仓产品代码改动，不触发Rust release重建。ledger阻塞导致工作树不移除，用户现有预览持续可看。

本轮Category A纯网站与内容任务，不运行Cargo tests/docs_gen。批量收据last_covered=715、2026-09-30T16:20Z，722/723不是%5且不足48h；历史720节点已获用户明确跳过（4717753c3），保持原收据，不借本次网站合入启动Rust重型回归。后续真实到期节点仍按规则处理。
