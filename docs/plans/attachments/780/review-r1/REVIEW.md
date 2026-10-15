# PLAN-780 R1 独立 final review

结论：**needs_fix**。独立轻包和显式旧新探针已经建立，生产读取未切换；但静态分类、数据保留和失败传播存在五类行为阻断，另有独立consumer格式收口项。Plan返回executing / 1/7，不merge、不发布Spec/ledger、不清理树。

## 审面与独立性

- plan_revision=1；review_scope=final；唯一实施树 `D:/autostack/.wt/lang-780/auto-lang` / branch `plan-780-dev`。
- reviewed_commit=`6aa07b630939966ac793c9f4cf7b07060330d736`；base=`075a9b4ab6cb4284f3c259b348f2571322107fec`；本轮主检出初始=`715d96a9c9c1aa1ef97abdaa0eba7afa12add4e0`。
- auto-down组内依赖=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；auto-man/project Spec源SHA256=`bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3`。全受管输入/root与consumer锁及判定前复查见 `fingerprints.json`。
- 本评审会话参与拆分规划和777/778审查，未实施780；直接从提交与独立组合重建，未派遣agent或新增树。
- 源码/原锁/HEAD/status保持不变，树干净。新增探针源码与可执行产物均在本树被忽略的scratch；缺失输入控制仅修改了自有Git index副本，真实index前后hash相同。没有写真实生产pac或修改实施代码。

## Findings

### F780-R1-01 / P1：结构化值中的变量被静态化为字符串

AC02/V02–03；T02/T03/T05。

scanner.rs第101–120行（顶层对象）及第375–391行（属性对象/数组）只用`scan_unsupported_in_text`筛插值等标记，再直接委托AtomParser。该筛查不识别裸变量/call语义；AtomParser会把普通裸identifier解释为字符串（auto-atom/parser.rs::parse_identifier_value）。因此同一语义在顶层标量会unsupported，在容器里却宣称静态。

三个真实旧新对照：

| 输入 | 旧VM oracle（probe port=win32） | 新模型 | 分类 |
|---|---|---|---|
| `members: [port]` | `["win32"]` | `["port"]` | static=true / unsupported=[] |
| `back: { project: port }` | external_backend="win32" | external_backend="port" | 同上 |
| `{ name: "dynamic", members: [port] }` | `["win32"]` | `["port"]` | 同上 |

这是冻结的“值位裸标识符须unsupported”在数组/对象/顶层对象的组合遗漏，不是要求模型执行表达式。修复前必须有覆盖全部值入口的、注释/字符串感知的分类策略；不能让AtomParser丢掉词法身份后再猜是否变量。保留引号字符串/静态嵌套容器合法控制，动态明确分类或拒绝并保留证据，旧adapter继续消费。

### F780-R1-02 / P2：重复dep path取第一个，违背旧实现后声明覆盖

AC02/03，V02/V05；T02/T03/T05。

输入 `dep foo {\n path: "first"\n path: "second"\n}`。旧oracle deps=[("foo","second")]，新模型deps=[("foo","first")]，仍static=true。from_scan.rs第447–450行对声明序列`.find(path)`取首值；顶层重复键虽正确按后声明覆盖，dep体没有遵守同一冻结规则。

应按实际旧臂统一重复字段归一化与冲突/重复记录，保留完整声明信息；补node-body重复/普通单值兄弟控制，不能仅测试顶层title/render重复。

### F780-R1-03 / P2：未知节点内容静默丢失

AC02/03，原“不截字段、不丢未知形态”合同；T02/T03/T04/T05。

两种已有节点语法入口：

1. `dep foo { child Extra { marker: "__CHILD_SENTINEL__" } }`：from_scan.rs第445行收集children，随后建DependencyDecl时丢弃。结果deps只有foo，raw_nodes=[]，unsupported=[]，marker完全消失。
2. `custom("__ARG_ONE__", "__ARG_TWO__") { tag: "kept" }`：Scanner的NodeName::Arg含raw参数；raw_node_from第491行只取name字符串，把raw args丢掉。结果只剩custom/name=None/tag，两个参数均不可恢复，unsupported仍为空。

修复应完整保存未知/未建模节点的子树和参数（或以明确unsupported策略保留原始证据），不要求实现目标/插件语义。模型从inline输入构建时连源文件也没有，不能以以后重读source_path替代DTO的数据保留承诺。

### F780-R1-04 / P1：读取失败被当成空配置相等，真实corpus门假绿

AC02/03，V05/验收validator失败传播；T03/T05。

project_model_adapter.rs第239行 `read_to_string(path).unwrap_or_default()` 将missing/invalid UTF8都变为空文本。独立missing_source和invalid_utf8控制均得到old=Ok(empty)、new=Ok((true,empty))、equal_claim=Some(true)，而非stage/path读取错误。模型自己的parse_file会正确返回Io；缺口在用于验收的显式adapter，不应混淆。

进一步运行**原corpus测试**：复制真实Git index到scratch，使用仅传给测试子进程的GIT_INDEX_FILE，加入一个内容blob有效但工作树不存在的受管pac条目。未改真实index或源码。原`project_model_corpus_comparison_v02`仍exit0，计数从164/154静态相等变成**165/155静态相等**，把缺失必需输入计为成功；见 `gate-negative.json` / `.log`，original_index_unchanged=true。

修复需传播读取/编码错误并包含path；corpus required输入失败必须令门禁非零。其第445行任意`(Err,Err)`都归template_rejected也应收紧为明确已知模板/预期坏fixture分类，不能让修复后的I/O错误再被宽泛分类吞掉。保留合法空文件作为区别控制；不要通过降低计数或跳过失败文件修复。

### F780-R1-05 / P2：明确排除的root wrapper被宣称静态

AC02，T01冻结子集“root wrapper显式unsupported”；T02/T03/T05。

`root { name: "wrapped"\n render: "vue" }`在旧Pac::new hoist兼容后name=wrapped/backend=vue；新模型只是把它列为raw node，name为空/backend=c，却unsupported=[]/static=true。from_scan::apply_node对unknown node直接入raw，并未将T01已明确排除的历史wrapper分类为unsupported。

本pilot不要求实现wrapper归一化；按冻结合同明确拒绝/unsupported、保存该节点即可，不能在未实施兼容时给静态等价标志。新增未迁策略若改变语义合同，先修订，不悄悄扩大static含义。

### F780-R1-06 / P3 / closeout-required：独立consumer未格式化

模型crate与adapter格式通过；独立consumer被workspace exclude，因此根fmt并未覆盖它。单独rustfmt --check发现consumer/src/lib.rs多处格式差异，exit1，见 `fmt-adapter-consumer.log`。修复新fixture即可，不扩展到全仓旧格式清理。

## AC与证据

| AC | 结论 | 证据/边界 |
|---|---|---|
| AC01 | pass | model21/21、external consumer3/3；实际normal依赖树只含model/atom/val及基础库，无lang/man/VM/UI；不只看manifest |
| AC02 | fail | 已有语料正控通过，但独立结构化dynamic、dep重复、wrapper与信息保留反例失败，required读取失败门假绿 |
| AC03 | partial | source/path基本面、owner-map、无生产切换通过；原I/O/未知信息/归一化合同存在缺口 |
| AC04 | 审计完成未pass | 本报告、原始输入/旧新值、独立libtest/真实corpus反控与输入指纹 |
| AC05 | pending | merge-owned规范沉淀/导航/ledger/合流/归档/guard清理 |

独立复验：model21/21、consumer3/3、adapter5/5、check -p auto-lang -p auto-man exit0；格式：模型及adapter通过，独立consumer失败。`git diff --check`通过。既有编译warning留在原始日志，不称全仓零警告。

现提交adapter的实际统计是 **164=154 static_equal+6 dynamic+4 template_rejected**，包括新增的15个*.pac.at fixture。work摘要的149=144+4+1是T01采样面，不能冒称冻结提交当前全体计数。新坏fixture的双拒与唯一clib模板拒绝应分开说明，避免“template”分类掩盖错误；文档需同步范围和计数。

独立12项组合 **3pass/9fail**，真实libtest exit101。三项正控为引号静态容器、普通注释及EOF块注释与旧臂兼容。EOF块注释虽然新模型接受，但旧lexer同样接受并合成CommentEnd，本轮不把它列新阻断或擅自提高语法严格度。

oracle库由冻结提交 `cargo build -p auto-man --lib --locked --message-format=json`实际构建，记录rlib和native搜索路径，然后独立rustc libtest链接同一组产物；不存在用旧rlib替跑的问题。所有构建/测试均在唯一Plan树；未运行裸cargo t/tv/tu/tt/taa/tf整档。原有单包全目标plan734问题不作为本轮新finding，review使用计划指定--lib面。

生产读取未切换：pac.rs/vue.rs/rust_ui.rs/asset.rs/target.rs/auto-lang/src相对base无diff；adapter仅新增pub模块，没有生产调用点。轻包的存在不代表auto-man→lang依赖已经拆除。26行owner表是后续接线决策，实际迁移继续另立Plan。

## Spec、状态与重放

SD01/02源方向可接受，但静态等价/错误传播/raw保留仍未成立，冻结增量status=partial，不能发布为当前保证。supersedes_spec_components=[docs/specs/auto-man/project.md]，new_spec_components=[docs/specs/project-model/project.md]，touched_goals=[]。

原始命令/退出码/输入hash见 `runs.jsonl`、`fingerprints.json`、`probe-run.json`；源码 `independent.rs`；结果 `independent.log`。

```text
python docs/plans/attachments/780/review-r1/replay.py --implementation D:/autostack/.wt/lang-780/auto-lang
python docs/plans/attachments/780/review-r1/gate-negative.py
```

第二条硬绑定本Plan唯一树，仅创建自有index副本；修复后先阅读脚本中原测试入口/预期条件。两个控制脚本当前均非零，因为合同失败，不把“正常语料门exit0”解释为整个验收通过。

保留T01调查与冻结，T02..T05及后置验收证据重开，Plan **executing / 1/7**，r1不变。work同树修复全部findings、补永久组合与required输入失败反控、校正语料收据，提交后再独立review。最终review通过后merge-owned门禁=裸cargo t；没有触VM/codegen/trans/aavm，不追加tv/tu/tt/taa；批回归按merge到期判定。本轮不修实现、不merge/归档/清理树、不发布canonical Spec/ledger，保留778/782等他线WIP。
