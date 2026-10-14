# PLAN-742 rev 1：第五次复审规范增量冻结稿

审面 `2174f0de28dd786369b2e44e94bfa1a1ac806499`；基面
`2c40e1fe4673c2041ae3c366037d7eab38017d81`。沿用五个 canonical 目标、
SD/AC IDs、GOAL-003/007；没有语义合同变更，plan_revision=1。
当前规范版本 SHA256 见 baseline.json；本次不发布 canonical 或 live ledger。

| ID / 操作 / 目标 | before → after：拟沉淀的持久规则 | 验收与实际边界 |
|---|---|---|
| SD-742-01 / modify `docs/specs/auto-man/design/api-generation-integrity.md` §2、`docs/specs/auto-man/project.md` | 空桩/不完整内联描述 → 非薄 handler 走 a2r，先转译 companion；限定真实模块身份，保留业务字面量，按实际签名和表达式实参边界补借用；返回形态按值/Option/void/File 适配，失败不能假成功。已有canonical描述需按实际覆盖校准。 | AC01/02/03/04；旧20例和标准库后缀模块闭合，但 `<` 比较实参被当泛型角括号，后续str实参漏借用（F-742-R5-01）。完整调用规则partial，不能批准完成 |
| SD-742-02 / modify `docs/specs/auto-lang/trans/overview.md` | 未记录体路径补正 → body路径补集合构造/语句分割，fs面有mkdir_all/dir_names；后处理只作用于真实代码，业务值不改变；宿主调用签名/返回可编译。 | AC01/04/06；trans源码未改，旧值与新多行字符串实编对拍通过；API完整可编译仍受R5-01阻塞 |
| SD-742-03 / modify `docs/specs/a2r-std/project.md` | 单所有权SqliteDb → Arc<Mutex<Connection>> clone共享连接/事务，最后clone drop关闭，String输入可用；dir_names返回排序的直接子名称、略点名、失败为空，非递归、不限定普通文件。 | AC06；Rust源码不变，原源码控制与tu仅保留历史证据身份。本轮当前后端强杀/同目录重启/S10值读回通过，不扩大为VM UI交互 |
| SD-742-04 / modify `docs/specs/auto-lang/ui/overview.md` | widget/store主题种子可能分叉 → 检测主题且有store依赖时，首store.dark_mode同源读取__AUTO_UI_THEME__；限定既有首store形态。 | AC05/06；本轮Vue smoke13/13（主题/CRUD）通过，VM交互未验；不追加固定19场景或视觉全等门禁 |

`supersedes_spec_components` 为上列五个实际路径；`new_spec_components=[]`，
更新既有模块规则即可；`touched_goals=[GOAL-003, GOAL-007]`。
比较布尔表达式+普通str实参仍是已有模块调用形态，反例不增加语言支持面。
单==历史归因仍为类别级，未伪造已丢WIP的源码行。D-742-1、Upload源码级接线及db旧regex候选维持既有边界。
