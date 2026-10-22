# PLAN-815 T-07 子批3 R1修正轮定向复审（R2）

2026-10-22，**pass（仅本轮受影响范围）**。原 **F-815-T07-B2-R1-03 / P2 CLOSED**，B2-R1-01/02 保持 CLOSED，三项均闭，无新增 finding。允许推进 T-08 定稿与 T-09 全计划终审；本次不等同整 Plan reviewed 或已交付。

## 冻结、独立性和范围

- 唯一树 `D:/autostack/.wt/lang-815/auto-lang`，branch `codex/plan-815-dev`，HEAD **afd1a66bff2584879687d2016eefe2f8153eb726**，base **9aee8659d8571ad889d5ccc46086c5102d8f64d6**。Plan r3；生产修复源仍 e22fc75924a7f058a5df8150956b2d66268c5ecd，M814-B 交付祖先 93a25c924d1beafab130588bad4d1041dc86e48c 未变。
- reviewer=Codex `/root` 延续独立审查上下文；有早期合同/账本参与限制，未参与当期修复；未创建 agent/第二实现树。只读审实现、运行定向断言、写主检出审查记录；未修实现/写canonical或ledger/合入/归档/清树。
- 修改只有两处 cfg(test) 新测试及矩阵/batch3文档。两文件生产前缀相对base逐字一致，集成测试/driver/资产/依赖/锁/配置/host/proc_host 均未变。起止同HEAD clean、源码和Spec零漂移，详[baseline](baseline.json.txt)、[终核](final-baseline.json.txt)。
- 当前 worktree debug工具 SHA256 **6d8427f71fa6b59ba80e4a75e3e60d98ae4f5790884f48f8a843db4b654ed3aa**。原发行33/33和实际生效对照继续绑定上轮包B工具 d2d0054c… 的原始身份，未把它冒称为本轮新建包；无生产改动不意味着不同构建二进制哈希相同。最终发布身份由T-08据实际冻结产物定稿。

## F-815-T07-B2-R1-03 关闭核验

| 原误标/补充面 | 本轮核验 |
|---|---|
| display.title/title_zh、runtime.window | 表中已补Rust generate消费；PacSource请求读取→wrap_example→AUTO_VM_TITLE/AUTO_VM_WINDOW初始化链准确。保留上轮真实请求值压过磁盘值、901x617/333x222生成物差异，未外推为本轮实际开窗。 |
| runtime.api | 已从runtime其他携带项单列；请求臂api_is_rust→run_vm默认backend_impl与split分支→真实generate_api/start_api_server链正确；显式AUTO_BACKEND_IMPL环境优先有说明。源码路径核验与本轮未实际构建Rust后端明确区分。 |
| toggles.default_classes | 已改Vue实际生成/转译消费；from_request归一→生产算法传参链准确。上轮on带默认class、off裸span差异证据已引用，新三态读取断言通过。 |
| dep_roots | 已补Rust resolve_bps_dependencies消费；相对根和缺省deps/<name>锚定已补，Vue原产物证据保留。 |
| theme_decl、raw_nodes/raw、sidecar | 已补主题激活与会话env两路径、VSCode具名节点props和sidecar_spec结构化载荷。核查run_vm:527仅非托管形态直接apply_theme_decl；托管形态已有每请求env路径，不将二者误当同时全局激活。sidecar布尔信号和实际载荷区分合理。 |
| 其余携带行 | 结合请求读取缝、模型方法及生产调用链重核：identity.version/lang、display其余域、runtime其余五项、scene、members、render legacy/effective、sidecar布尔未发现被漏标为主链效果的实际请求读取。成员计数与raw_nodes中的同名字段不混淆；不以只grep字面字段代替方法/别名核查。 |

既有操作由op选择、固定实际布局根、相对请求根、请求值/缺席权威等边界不因本次文档修正变化；未授权新运行模型或新字段承诺。原关闭条件“准确的字段/profile/入口/来源/效果/限定与证据映射”在本次受影响范围满足，不再把原错误归类标为冻结限制来绕过验收。

## 本轮实际执行及复用

- `cargo test -p auto-ui-tools --lib pac_source_window_title_api_dep_roots_request_arms -- --test-threads=1`：**1/1，0.09s**，[原始日志](pac-source.log.txt)。请求窗口/标题与盘值不同、API缺席不回读盘值、dep请求根/默认锚定、Fit及legacy对照断言实际执行。
- `cargo test -p auto-ui-tools --lib vue_reads_default_classes_comes_from_request_toggle -- --test-threads=1`：**1/1**，[日志](default-classes.log.txt)。true/off/Unset三态断言通过。
- 两个改动Rust文件 rustfmt --check、git diff --check 均通过；既有 lib-test18 warnings 未产生本次新增未处理诊断，[卫生记录](hygiene.json.txt)。未跑全lib或整档测试，不将work181/184当作本轮独立总数。
- [复用依据](evidence-reuse.json.txt)：未变的生产源码/依赖/配置/输入允许复用原[33/33发行](../review-t07-batch3-r1/driver-receipt.json.txt)、[377文件资源对账](../review-t07-batch3-r1/resource-reconcile.json.txt)、[六个实际生效/失败/资源来源对照](../review-t07-batch3-r1/holdout-results.json.txt)及三枚子批3定向测试，保留其原观察和覆盖边界。本轮未重跑792冷构建、全矩阵实际窗口或完整后端构建。
- A/L/T04、前期联合修复、子批1与子批2已闭范围不重开。本轮也未替代T-09对既有披露债务及全计划AC/Spec的最终裁定。

## 规范、任务状态和后续

[SD受影响核验](spec-delta-reviewed.md)接受更正后的事实映射；冻结矩阵/交接副本保存在本目录，未沉淀canonical/live ledger。T-07三个子批的已审证据可用于后续整体收口。

**下一步：T-08按真实冻结源/产物刷新指纹、定稿SD与交接，再进行T-05～T-08整任务联合收口及T-09独立终审。** 此次请求限定B2-R1-03受影响范围，且T-08明确尚未定稿，故不提前一次勾四项或标全计划reviewed；当前Plan保持executing10/17，原三项finding均已CLOSED。

最终pass后的merge-owned裸cargo t+实际触面档、777 master并集/必要发行接线核验、串行master合入、Spec/ledger/发布收据/归档与guard清理仍pending。此定向pass不等于这些门禁或正式发布完成。
