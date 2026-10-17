# PLAN-799 final review R2 — needs_fix

2026-10-17；plan_revision=1；review_scope=final。
受审 `4194fccc98f5c945288cceb030afd283726e1a8e`，基面 `143aff0a258fdca8bda9a3b8068dc8a9cced623c`。
既有工作树 `D:/autostack/.wt/lang-799/auto-lang`，branch `plan-799-dev`。
审查人为本聊天 Codex，独立于实施上下文；未暴露模型/session ID，不虚构身份。输入与依赖指纹见 baseline.json.txt。
本轮未修改实现、canonical Specs、ledger，未合入。

## Findings

### F799-R1-01 / P2 — closed

`auto_val::AutoResult` 已恢复为模型内部私有导入；旧门面的结果类型仍唯一指向诊断层 AutoResult。
原始 review-r1/glob-compat.rs 使用当前库编译 exit0；新增永久测试 ast_facade_glob_compat 1/1、ast_cross_path_identity 2/2 全通过。
补充独立 glob-matrix.rs 同时导入 ast/error/token/parser/vm，检查 AutoResult、Pos、Value、AutoStr、Op 的解析/身份，metadata 编译 exit0。
检查修复覆盖了导出不变量，而非只给反例加显式导入；未改变旧调用者。

### F799-R2-01 / P2 — open（修复引入的格式门失败）

位置：`crates/auto-ast/src/ast.rs:79`（导入块77–82行）。影响 T-02/T-06、AC-05 健康检查。
`rustfmt --check --edition 2021 crates/auto-ast/src/lib.rs crates/auto-lang/src/ast_parse_ext.rs crates/auto-lang/tests/ast_facade_glob_compat.rs crates/auto-lang/tests/ast_cross_path_identity.rs` exit1。
`pub use auto_lex::token::Pos;` 应按 formatter 排在 auto_val 导入之前；完整输出见 format.log.txt。R1同命令通过，本轮修复拆分私有/公开导入后改变了排序约束，属于新增格式失败。
修正：在原树按仓库 formatter 格式化该导入块并复跑上述命令，提交后送受影响复审。不能以“新测试fmt clean”替代根模块递归格式门。
这是仓库强制健康门，未给 reviewed；没有把它描述为运行期行为错误。

## 验收与证据

| AC/V | 本轮结果 | 证据与边界 |
|---|---|---|
| AC-01/V01 | pass | auto-ast 74 unit + 1 projection + 5 doctest；独立 AST fixture 3 unit + 1 projection 全绿（check-1/2.log）；consumer-metadata.json.txt：仅 auto-ast→auto-lex/auto-val/serde，无 auto-lang/Man/iced 反向包、无 dev/build 隐藏依赖 |
| AC-02/V02/03 | partial | 原始反例及永久回归通过，ast_parse_ext 11/11（check-5.log.txt）、新旧路径互传/坏输入/投影测试2/2；两条36-case golden重放通过。外部 ui-dialect-consumer 本轮受镜像环境阻断，不能宣称完整V03重放通过 |
| AC-03/V04 | pass | 32模型子文件再次按允许Pos/doctest路径适配逐字核对旧臂，模型单一/解析在上层保持；R1生产消费者461通过+2ignored可复用，原因：本轮仅模型导入可见性修复、两新集成测试、说明/SD；Parser/推断/VM/trans/UI算法、相关依赖与测试配置不变，已核对diff和依赖tree/hash。check三crate通过（check-6.log.txt） |
| AC-04/V05/SD | pass | 794 scroll-provider-consumer 5/5（check-3.log.txt）；777 strict pass，0 violations / 2 grandfathered（boundaries.json.txt）；交付driver结果见下节。SD-01导出说明已修，SD-02保持；frontmatter与SD表已补SD-03，路径/GOAL-003有效；冻结spec-delta.patch SHA256=62834ddfc34b92d650272c0f755396784a1eafd267fbd452605287cf282cab6f |
| AC-05 | fail | F799-R2-01开放，最终needs_fix |
| AC-06 | pending merge | 裸t+tv+tt+tu、合流新基线审查/门、tf到期判定、Spec/ledger/归档/guard清理未执行 |

### 环境阻断：外部 UI 方言 consumer

线上与 --offline 均 exit101（check-4.log.txt、ui-dialect-offline.log.txt）：fixture既有未受管 Cargo.lock 锁 zvariant 5.15.1，aliyun镜像索引只有5.15.0等版本，依赖来自zbus→ashpd→rfd；尚未进入源码编译/测试。
不把该失败归为799代码回归，也不偷偷改锁或换依赖后冒称同输入通过。该外部V03 oracle需环境恢复后重放，或提供完整输入未变的可复用证据。

### 其他检查

cargo check -p auto-lang -p auto-man -p auto-ast exit0；git diff --check exit0；format门如F02。
依赖auto-val/lex/diagnostics的tree、根Cargo.lock/.cargo/config.toml、旧golden SHA与R1完全一致（baseline.json.txt），没有新增运行期算法或Parser下沉；stock warnings不是本轮新问题。
完整SD冻结在本附件，未写main规范；当前main已合800，后续合入同步时审查受影响差异，现pass也不会覆盖未来代码。
主计划原有execution_done但current_step=2、四行未重新勾选的簿记不一致，本轮按实际验收重新计算；保留历史记录，不以旧勾选充当证据。

## 遗漏、延期与下一步

R1缺陷已完整闭合；未发现模型遗漏、重复实现、Parser依赖下沉或未授权退役。F02不能登记为延期债放行，外部consumer环境阻断单列。
已连续两轮final非pass；下轮work handoff须简述为何根模块格式检查漏检，并用同一根入口命令确认修复后的oracle覆盖，不应只重放新增测试文件。
继续使用原工作树修复、定向验证并提交；merge-owned整档门保持pending。

## 792合流独立重放收尾

正式 driver.py（未跳过生产CLI构建）exit0，driver-receipt.json.txt **25/25**，原始调用与结果见driver.log。V01 fresh解析+真实locked build通过（128s），V02不拷target的重定位locked build/metadata/资源同源通过，V03a–j全部负控与复绿、V04 dev路由及模板同源通过。A/B实际交付auto-ast/src/ast.rs的SHA256均等于本次受审源，含R1修复，不是旧producer。收据已复制到持久附件，不只引用临时目录。

入/出HEAD均为4194fccc98f5c945288cceb030afd283726e1a8e、实现树净。最终AC01/03/04 pass，AC02 partial（外部ui-dialect环境oracle未完成），AC05 fail（格式门），AC06 pending；outcome=needs_fix。
