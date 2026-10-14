# PLAN-742 rev 1：第六次复审规范增量冻结稿

审面 `8d59f5e88962a51a7e608b5ffe9bd8d03ff22086`；基面
`2174f0de28dd786369b2e44e94bfa1a1ac806499`。沿用五个 canonical 目标、
SD/AC IDs、GOAL-003/007；没有语义合同变更，plan_revision=1。
当前规范版本 SHA256 见 baseline.json；本次不发布 canonical 或 live ledger。

| ID / 操作 / 目标 | before → after：拟沉淀的持久规则 | 验收与实际边界 |
|---|---|---|
| SD-742-01 / modify `docs/specs/auto-man/design/api-generation-integrity.md` §2、`docs/specs/auto-man/project.md` | 空桩/不完整内联描述 → 非薄 handler 走 a2r，先转译 companion；限定真实模块身份（标识符边界、完整 token），保留业务字面量（代码区掩码），按实际签名和表达式实参边界补借用（比较 `<`/`>` 不是泛型定界——发射器形态规律：泛型 `<` 紧贴标识符、比较两侧带空格）；返回形态按值/Option/void/File 适配，失败不能假成功。 | AC01/02/03/04；旧20例+标准库后缀模块+比较实参反例全部闭合（F-742-R5-01 修复经独立重放与两个新探针实证）；可批准为完成 |
| SD-742-02 / modify `docs/specs/auto-lang/trans/overview.md` | 未记录体路径补正 → body路径补集合构造/语句分割（代码区掩码门控），fs面有mkdir_all/dir_names；后处理只作用于真实代码，业务值不改变；宿主调用签名/返回可编译。 | AC01/04/06；trans源码本轮未改，旧值与多行字符串实编对拍保持通过；可批准为完成 |
| SD-742-03 / modify `docs/specs/a2r-std/project.md` | 单所有权SqliteDb → Arc<Mutex<Connection>> clone共享连接/事务，最后clone drop关闭，String输入可用；dir_names返回排序的直接子名称、略点名、失败为空，非递归、不限定普通文件。 | AC06；Rust源码不变。**注意**：主检出该文件已被 PLAN-773 沉淀改写（hash 见 baseline spec_sha256），merge 沉淀须基于当前 canonical 版本再合入，不得盲覆盖 |
| SD-742-04 / modify `docs/specs/auto-lang/ui/overview.md` | widget/store主题种子可能分叉 → 检测主题且有store依赖时，首store.dark_mode同源读取__AUTO_UI_THEME__；限定既有首store形态。 | AC05/06；Vue smoke13/13 于 r5 审核独立通过并沿用；VM交互未验为已接受边界，不追加固定19场景或视觉全等门禁 |

`supersedes_spec_components` 为上列五个实际路径；`new_spec_components=[]`，
更新既有模块规则即可；`touched_goals=[GOAL-003, GOAL-007]`。
比较布尔表达式+普通str实参仍是已有模块调用形态，不增加语言支持面。
单==历史归因仍为类别级，未伪造已丢WIP的源码行。D-742-1、Upload源码级接线、
db.rs旧regex候选维持既有边界；plan734_commands_fixture.rs 非 lib 测试目标
预存编译失败为 r6 新增债务候选（master 同态，非 742 引入）。
