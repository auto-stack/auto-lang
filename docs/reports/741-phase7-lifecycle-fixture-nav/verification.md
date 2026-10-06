# PLAN-741 Phase 7(r7)fixture 生命周期与导航门修复验证报告(2026-10-06)

r7 基线:worktree `D:/autostack/.wt/lang-741/auto-lang` 分支 `plan-741-dev`,
基线提交 `8296ff305`(v0.6-dev 含复审方 r7 再激活提交)。本 Phase 仅 Python
验证脚本/计划导航/README 修复,**零 Rust/依赖/canonical 改动**(合同 §Phase 7);
组内只读依赖 `D:/autostack/.wt/lang-741/auto-down`(detached @ fba6563e)。

## 修复摘要与 AC 对账

| 项 | 修复 | 验证 |
|---|---|---|
| R6-QA-01(P3) | **fixture 生命周期化**(fixture_tests.py 全量重写):计划源自动检测(docs/plans/ 部分执行态 或 git 历史重放的归档态);从源文本合成对照——部分执行态的全完成控制(全翻转+archived+链接转换+current_step=total,承载完整沙盒 repo 骨架)、归档态的 active 控制(status→executing+链接回退+任务保留);**同 ID 冲突变换按真实行状态翻转恰一行并断言文本实际改变**(修复 r6 的 T-35 硬编码零操作缺陷;偏好多行组使行间真正分歧);每例断言指定诊断子串(无关非零不掩败);TemporaryDirectory 自回收 | active 矩阵 12 用例全过(含全完成合成控制、冲突、QA-02 缺陷形态、坏链注入);archive 矩阵(git 历史重放 r6 归档态)12 用例全过(含 valid-archive-complete、合成 active 控制、stale 链接、坏链注入);两矩阵合计 **21 用例 + 2 沙盒导航控制全 PASS** |
| R6-QA-02(P3) | **最终门扩到导航/README**(final_assertions v3,显式版本化替代,--repo-root 支持沙盒):prototype README 741 链接、auto-hir/auto-ac 741 导航行(状态文本生命周期一致:激活期不得说 delivered、归档后必须 delivered+archive 链接、不得残留 executing)、带行号定位;--repo-root 允许沙盒骨架负控制 | active 模式:README+两导航行全部 active 形可解析;归档负控制:沙盒 repo(auto-ac 导航遗忘更新)→ exit 1 且输出 `auto-ac/plans.md:5`(文件+行号定位);沙盒全对 repo → exit 0 |
| 计数 | final_assertions v2 已修(current_step==完成唯一数),r7 36/41 → T-39/40 后 38/41,T-41 后 41/41 | active 模式实测 frontmatter 与实际交叉核对一致;v2 上线即抓到执行侧 35≠34 的能力已在 r6 证实 |

## 门禁证据(全部 worktree 实跑,2026-10-06;Category A 零 Cargo/docs_gen)

- final_assertions v3 active 模式:ALL-ASSERTIONS-PASS(frontmatter 36/41 交叉核对、
  13 链接自真实 active 父目录解析、13 链接自模拟归档转换后解析、README/两导航行、
  6 项 P741-* review 指针暂态标注)。
- fixture 矩阵:active 源 12 用例 + archive 源(git 历史重放 r6 归档文本)12 用例 +
  2 沙盒导航控制 = 全 PASS(逐例诊断子串匹配)。
- 生产面零 diff:`git diff 8296ff305 -- Cargo.toml Cargo.lock crates/ test/
  experimental/ac-core/src` 为空。
- 原型/60s/root 证据:按 r7 合同以源指纹复用 r6 复审记录(link.rs/main.rs/
  verify.rs/native.rs 相对 r5/r6 零改动,r6 复审 source-manifest 绑定仍有效)。

## 遗留

- merge(T-41 pass 后):实际更新两模块导航为 delivered+archive 链接、归档转换链接、
  以 archive 模式复跑 final_assertions(41/41 全勾门)、fixture --source archive 真实
  归档复跑、销账 R6-QA-01..02 行、guard 清理。
- root daily 37 红(8 例超历史并集)非 741 归因(r6/r7 零 root 改动),保持 r6 复审
  记录口径,不在本 Phase 扩围。
