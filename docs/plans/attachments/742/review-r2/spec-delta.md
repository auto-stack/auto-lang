# PLAN-742 rev 1，第二次 final review 的知识增量冻结稿

当前实现为 `6d93cfdb73f5178b624846d46f88614ed999c158`，基面
`c151935f1b50a9aefb7164771bcc70ccf4b66c9d`。沿用 SD-742-01～04 与原 AC IDs，
不改变原验收语义。规范目标仍是原五个 canonical 文件；没有新组件，目标 GOAL-003/007。

| delta / 目标 | 当前观察与拟沉淀规则 | 本次裁决 |
|---|---|---|
| SD-742-01；`docs/specs/auto-man/design/api-generation-integrity.md`、`docs/specs/auto-man/project.md` | 非薄 JSON 体中模块限定化按字符串 mask/调用形态处理；Result 普通值返回已补 Ok；但 borrow 后处理仍扫描字面量、局部名字集合不体现作用域、File 体漏走限定化，Option 变量及 void 出口仍错误。最终规则应覆盖真实 binding、完整体路径和所有合法返回出口；不能把稳定反例降成“只覆盖四个例子”。 | partial，不批准当前完成申报；F-742-R2-01～04 |
| SD-742-02；`docs/specs/auto-lang/trans/overview.md` | 体路径 List 构造和语句拆分已有单行 literal/comment mask；普通/转义字符串样例保持。仅描述实际 emitter 形态，不声明支持任意 raw string、多行块注释或未来规模。下游借用 pass 缺陷仍须闭合后再确认端到端字面量保持。fs.mkdir_all/dir_names 路由未受修复 diff 改动。 | 旧例通过，完整 API 值保持仍 partial |
| SD-742-03；`docs/specs/a2r-std/project.md` | SqliteDb clone 共享一条连接和事务，最后一个 clone drop 关闭；open/exec/query 接 owned String。dir_names 返回排序后的直接子名称、略过点名、读失败为空；非递归、不限定为普通文件。源码和依赖与 r1 控制一致。 | 重用明确指纹绑定的 r1 Rust 控制证据；pass 于所测 Rust 面 |
| SD-742-04；`docs/specs/auto-lang/ui/overview.md` | 检测到 dark_mode 且有 store 时，首 store 读 bootstrap 同源主题种子。源码及 UI 生成测试族/配置未改，r1 tu 可重用；当前修复版 Notes 的双端主题和 legacy 冒烟本轮未运行。 | partial，联动待修复后补齐 |

本文件冻结的是当前可核查的知识提案与失败边界；不发布 canonical 或 live ledger。
首轮已先行沉淀的 e35f9e99b 历史内容保留，不能据此批准第二轮错误实现。
已知 D-742-1 继续另案，不并入本次修复范围。此次是第二次 final needs_fix，
下一次 work 交接必须先给出诊断及路径/出口矩阵调整，不能只增加失败例数量。
