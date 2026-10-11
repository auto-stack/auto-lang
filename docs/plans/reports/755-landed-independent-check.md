# PLAN-755 落地后独立核查（2026-10-11）

- stage: review；plan_id: PLAN-755；plan_revision: 1；范围：R2 两项修复闭合、blueprints 恢复事实、HTTP 串行复验。
- reviewed_commit: `cee541da7a4b1a3410bef8cbdfe2bed0c4c00c51`；该树相对应对报告基面 `d8fba471b`，crates/stdlib/.cargo/.config/blueprints/Cargo.toml 零差异。
- fix_commit: `5672e8e2d0c56f412301a99d54e5c7206fcb3d11`，已是 reviewed_commit 祖先；与原修复 `e121debf8` range-diff 全等。
- dependency: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`；根 lock 哈希与规范/配置版本见 baseline。
- 独立性：本会话曾提交 755 R2 needs_fix；未实施修复、合入或恢复 blueprints。本次从落地代码与新验收重建闭合结论。755 保持 archived，不重开。
- 隔离 detached 检出执行测试，源码未修改；仅复用普通构建缓存，无链接。没有在主检出重复日常全库门禁。

## 两轮审计原因必须分开

**上轮 R2 是真实的实现/门禁缺陷**：panic 结果通道只回传 `Any { .. }`，四个改动 Rust 文件格式失败；审计原文与反例就在已提交的 [755-review-r2.md](755-review-r2.md)，簿记提交 `ad7eee41f`。不能用本次 blueprints 删除来解释或推翻那一轮打回。

**本轮主检出额外红的报告**：应对记录描述 blueprint 未提交删除导致 18 项新增失败；恢复后定向 31/31、日常 5231/5218/13。当前本会话逐文件实核：68 个 tracked blueprint 文件全部存在，Git 内容哈希与 reviewed_commit 一致，验证时零 tracked 代码 WIP。该恢复事实独立成立；恢复前失败与恢复后全库数字保留为执行者的历史收据，不冒称本会话复跑了该全库。

## 已知 R2 缺陷的落地修复

- 三启动助手全部改用共享 `spawn_outcome_reporter`，统一 catch/unwind 与通道发送；`panic_payload_text` 对 String / &'static str 提取，未知类型明确兜底。真实 spawn/catch/channel 的负向冻结与三载荷形态测试已入库。
- 上轮同款四文件 rustfmt check，本次在 clean 落地树 **exit 0**。P755-R2-02 闭合。
- frontmatter 已将既有 assembly-manifest.md 的 modify 登记于 supersedes，new=[]。P755-R2-03 的元数据纠正保留。
- HTTP 契约、严格门与公共面未因本轮核查修改；R2 修复提交仅测试诊断核心与格式化。

## 串行复验

应对报告本次 `cargo th --no-fail-fast` 为并行档，102/100/2 红/0 超时；它自己明确记录并行 flake。**并行失败后隔离绿不能直接改写成新的串行 102/101/1 验收**。为消除这个证据缺口，本次另在隔离落地树运行：

```text
cargo nextest run -p auto-lang --lib --features test-http-e2e \
  --jobs 1 --no-fail-fast plan755 http_e2e
```

两个滤串按 OR 并集选中 4 个 plan755 守卫与完整 102 个 HTTP e2e；全部同二进制、串行实跑。

**实测结果：106 run / 105 pass / 1 fail / 0 timeout，265.241s，exit 100**。拆分为：

- plan755 守卫 **4/4 pass**，包含真实 panic spawn/channel 负向冻结及三载荷形态提取测试。P755-R2-01 闭合。
- HTTP 全档 **102 run / 101 pass / 1 fail / 0 timeout**；唯一红为 back_proxy corpora 参数 `:id` 导致 400≠200，与已录基线同形。
- 原上传/重定向与 `vm_quoted_boundary_wire` 均在本次串行跑通过。慢传输测试 77.895s 后通过；不把 nextest 的 SLOW 提示算成 timeout。
- 四文件格式 **exit 0**，源码起止 clean。生成代码、公开契约和门逻辑均无本轮改动。

**outcome: pass（范围为已知 R2 修复、恢复事实与串行 HTTP 复验）**。不是新的全库日常/批量回归签署，整库仍有在册红；归档状态不变。

## 收尾边界

- 不以“无罪/全维等价”抹去历史真实缺陷；准确表达为“755 R2 已修复；blueprints 删除造成的额外日常红已恢复”。
- back_proxy 仍是本计划明确豁免的预存红，其他日常预存红没有在本轮清偿；不称整个仓库全绿。
- `.last-batch-regression.json` 仍覆盖 740（2026-10-08）；755 为五的倍数且已落地，批量回归 DUE。当前报告与日常/HTTP scoped 验收均不代替 tf 批量收据；后续由 regress 按主检出干净度与机器单实例条件执行。本次没有擅自启动该机器独占档。
- 未修改或纳入执行者尚未提交的 `755-external-audit-response.md`；独立结果、基线与日志单独持久保存。
- 证据复制后，独立 auto-lang/auto-down 检出均 wt-guard clean，已移除两检出与空组目录；未创建或删除其他工作树。
