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

## 已落地检查点

- 722：master以`git merge --ff-only 939497eadc5b189aca61a10c91387c6f0e7a32d6`从142458d21快进，先落入四主产品基础；723：再`git merge --ff-only plan-723-dev`到`02f4b1a90e288ad3dc9d970a435ad29693f9fe43`。实际main HEAD与该交付提交相等，两次均无merge commit。
- master与交付分支的website/docs/specs/website diff为空；master生成器--check58兼容路由通过；同步后的独占4249集成冒烟`npx playwright test demo-catalog.spec.ts --grep 'frozen|overview contains' --workers=1`：3 passed(15.2s)，包括冻结28实图与EN/ZH整页完整内容/筛选/键盘图片行为。原70检查基线与重放网站代码全等。
- 4个r2 canonical hash匹配，模块plans已回写，INDEX重生成无语义变化；原.autoos/specs.json保留，未通过失败的store发出任何写请求。只停止本计划4248临时musk store，验证监听PID命令行后停止，其他实例和4235预览保留。
- outcome：网站与canonical交付完成；ledger_refreshed/archived/cleaned blocked（旧schema读取失败）。722/723保持reviewed，719旧检查点不冒领解除。单独记录到主检出计划，可在账本兼容后仅续做缺失检查点，不重复合入代码。
- 主线无本会话未提交的实现；已有其他会话.tmp-vm探针、.snap.new等未跟踪文件保留，不纳入合入。未push或部署公开站。
