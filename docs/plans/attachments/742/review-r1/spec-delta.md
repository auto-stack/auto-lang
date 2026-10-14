# PLAN-742 rev 1 复审规范增量冻结稿

复审输入为 `c151935f1b50a9aefb7164771bcc70ccf4b66c9d` 的 canonical Specs。
历史沉淀 `e35f9e99b` 已在 master；本次不发布新的 canonical 内容。
以下为补齐遗留 Plan 的知识增量与核查结果，不能作为实现已通过的声明。

| ID | 操作 / 目标 | before → after（实际规则 / 待通过条件） | AC / 审核结果 |
|---|---|---|---|
| SD-742-01 | modify `docs/specs/auto-man/design/api-generation-integrity.md` §2；`docs/specs/auto-man/project.md` checked generation | 旧的空桩描述 → 非薄 handler 体走 a2r；伴生模块先转译，调用降为 crate 路径并按签名借用；集合构造与 Option+path 返回适配。现有 canonical 已收录，但文字不能证明字符串保持、所有声明返回形态均正确。修复 F-742-R1-01/02/03 后复验再定稿。 | AC-742-01/02/03/04；partial，暂不可批准 |
| SD-742-02 | modify `docs/specs/auto-lang/trans/overview.md` | 未记录体路径补正 → `transpile_body_stmts` 对 List 构造及粘连语句做后处理，fs 模块面补 mkdir_all/dir_names；应限定到语法节点，不能改写字符串常量。当前处理方式存在 F-742-R1-01，保留为当前缺陷而非把误改写定义成规范。 | AC-742-01/04/06；partial |
| SD-742-03 | modify `docs/specs/a2r-std/project.md` sqlite/fs | SqliteDb 单所有权 RAII → Arc<Mutex<Connection>> 可克隆，同一连接共享事务，最后一个 clone drop 才关闭；open/exec/query 可接 owned String。补 fs.dir_names：直接子名称、排序、忽略点名、读目录失败返回空，非递归；不承诺只返回普通文件。 | AC-742-06；Rust 实际运行控制 pass；跨 VM 证明仍归联动验收 |
| SD-742-04 | modify `docs/specs/auto-lang/ui/overview.md` | widget 与 store 深浅主题种子可分叉 → 已检测主题且有 store 依赖时，首 store.dark_mode 同源读取 __AUTO_UI_THEME__。仅描述现有检测/首 store 形态，不推广到任意 store 多实例。 | AC-742-05/06；tu pass，当前 Notes 双端重放 pending |

`supersedes_spec_components` 对应以上五个实际存在的目标；`new_spec_components=[]`，
因为均为既有模块规则更新，不另建规范组件；`touched_goals=[GOAL-003, GOAL-007]`。
SD-742-01～04 是既有实施范围的归纳，未削弱原 handler parity/可编译要求，未变更 plan_revision。

已知 D-742-1 是跨模块导入类型构造解析问题，历史 Notes 已明确另案边界，不能冒称修复。
须在债务簿登记。目录签名例外 P742-D9-SIG-01 已由 Plan 744 登记，保留其原身份。
