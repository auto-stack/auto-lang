# PLAN-856 合入门禁暂停记录

- 结果：`blocked`。冻结代码 `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`，SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`；权威工作树 `D:/autostack/.wt/lang-856/auto-lang` / `plan-856-dev`，末次核对 clean。`autographics-dev` 仍为 `05fdc81c546aff735eefdb6c39fbc5fb35dc546a`；未落源码、规范、ledger、归档或清树。计划保留 reviewed、7/10。
- 双端完整测试各 80 入口、300/300 通过，清单完全一致且保留 855 的 288 项；新旧 21 生成文件逐字节一致，双端各7项生成控制通过。消费端先前一次无诊断中断及一次已有预算文案断言不稳定必须保留，详见 budget-diagnosis.md。
- 原生门：producer新增视口、窗口、交互/重开、hierarchy/pixel、assets、play（遮挡后重试）、user、step 已通过。`projects` 原始 15 用例运行中 `bind-patrol` 报 `play phase 1: native fixture pointer occluded by another process`；现场在 `D:/autostack/plan-artifacts/plan-856/merge-r1/projects/` 和 `projects.log.txt`。之后同一源码、SDK、可执行文件和配置的隔离 `bind-patrol` 在 phase 0 超时，现场在 `projects-diagnostic-bind-patrol/` 和 `projects-diagnostic-bind-patrol.log`；其根因未知，不能断言就是遮挡。play早先也出现遮挡，另存重跑已通过。只读进程检查未发现残留autographics测试进程；不能辨认具体遮挡者。
- 当前需提供无遮挡、可获得焦点且不与其他自动化争用的桌面；确认条件改善后，从原冻结提交重跑该单场景。若仍失败，调查焦点/资源状态，不无限重试；若通过，在全新目录运行项目完整15项和后续UI relocation，按实际成功目录更新审计引用，保留所有失败现场。然后继续 producer/consumer 所有原有与新增 native、sweep、motion、owner/content 门，逐项审计并检查复用输入，再进入源码落地/规范ledger/归档和guard清理。
- `gates.py` 当前的 `projects` 输出路径已存在且只含失败现场，不能覆盖；恢复前必须调整编排以生成新目录，并让 relocation、audit 和证据索引引用真实的成功输出。当前仅 play 的新目录已适配。现有预算问题不声称已修复；初次无诊断中断同样不伪造根因。

- 恢复诊断：无其他AutoGraphics验证进程的只读快照后，`projects-diagnostic-bind-patrol-resume/` 再次 phase0 超时。单独的后台窗口观察 `projects-window-observation/` 运行时窗口清单未列出测试窗，亦phase0超时；后台启动方式可能影响窗口显示，不能将该观察当作正常窗口环境下的根因证明。等待桌面可交互/无遮挡确认后继续。
