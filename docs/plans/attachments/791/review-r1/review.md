# PLAN-791 final review R1 — needs_fix

2026-10-16；plan_revision=1。审查对象为现有实施树 `D:/autostack/.wt/lang-791/auto-lang`，branch `plan-791-dev`，HEAD `0cd3e3e9f7e448fc95a1ef61b13761ac02d7e65e`；diff base `e4ac72a6265c85cdadeedbabc7a0c6fd93948e48`。入场与测试后 `git status --porcelain=v1` 均空，HEAD未漂移。独立审查本聊天未参与实施，未调用额外agent，未改实现。

## 当前结论与 findings

| ID | 严重性 / 类型 | 违反的现有合同、证据、修正要求 |
|---|---|---|
| F-791-R1-01 | P2 / 原验收覆盖遗漏 | §6 V02明确要求冻结 `use` 库行为，§5/V03要求保留库加载顶层错误边界；但V02所有GenerationSpec的`lib_files`/`lib_paths`均为空，V03也全空，fixtures无use库案例。V02文件头声称lib_files定义不可见，却无对应断言。`missing_data_top_level_error`仅缺数据文件，不能证明`load_lib_file`路径。补已有旧提交上的库输入/输出或错误阶段对照：非空lib_files、模板use/非空lib_paths、缺失库文件（顶层Err且既有输出字节不变、无后续模板写入），再以相同输入重放新实现。既有不支持行为应冻结实测失败，不能顺手修解释器或用新输出再造旧expected。受影响AC-02/05、T-01/05/06。 |
| F-791-R1-02 | P2 / health gate未过 | `cargo fmt -p auto-gen-core -p auto-gen -- --check` exit=1，涉及generator.rs、lib.rs、common/mod.rs、v02_compat_golden.rs、core/generate.rs；`git diff --check e4ac72a62 HEAD` exit=2，auto-gen/Cargo.toml:33 new blank line at EOF。定向测试新增6条dead_code warning，均来自V03编译common/mod.rs中的data_dir:20、goldens_dir:24、update_goldens:28、copy_tree:32、rel_files:45、assert_tree_matches_golden:67。这些是本计划新测试，不属于auto-lang/shim的预存警告。对触面格式化、清掉EOF空行，并通过拆分helper/精确注解等合理方式处理测试辅助函数的编译范围；提交后复跑fmt/diff-check与定向测试。受影响AC-05、T-05/06。 |

以上为合同内返工项，登记debt不表示批准延期。没有观察到已测生成行为的回归；缺失的库路径验收仍不能以绿色总数代替。

## 验收覆盖与复现

所有命令在上述实施树执行，未设置UPDATE_GOLDENS。工作树外部访问/构建使用环境批准的执行权限。

| AC / V | 实际检查 | 结果与边界 |
|---|---|---|
| AC-01 / V01 | `cargo test --manifest-path test/fixtures/modularity/gen-core-consumer/Cargo.toml`；Git Bash执行`test/fixtures/modularity/gen-core-consumer/verify-deps.sh`；独立manifest与实际tree | pass，4 tests，闭包脚本PASS/exit0；normal/build为core→auto-val及叶依赖，无auto-lang/auto-man/auto-vm/UI。真实文件字节、guard保留、dry_run、render失败哨兵均断言。 |
| AC-02 / V02 | `cargo test -p auto-gen-core -p auto-gen` | 部分通过：V02=11/11，覆盖Atom、多模板、AutoFile/Code既有失败、fstr_note怪癖、legacy及CLI、命名。库路径覆盖缺失，见F-01。 |
| AC-03 / V03 | 同上，逐条阅读断言及迁移diff | pass于已承诺边界：V03=8/8、core=12/12；render失败旧字节不变、无新目录、dry_run零写、失败后批次继续、guard保留/overwrite与conflict查询。旧GuardProcessor::merge本来恒返回Ok，未要求新增guard拒绝行为；此处不把detect_conflicts误当生成器自动拒绝，也不声称写盘事务。 |
| AC-04 / V04 | generator.rs/lib.rs与基线diff；auto-man消费清单；core manifest、guard/error迁移/re-export；module-boundaries登记与baseline diff | pass，CodeGenerator.generate实际调用auto_gen_core::generate；legacy只调用core的feed_guarded_legacy，解释器仍在adapter。两种guard算法语义不同，保留并存正确。Man assets不经auto-gen、exporter/builder仍经legacy，未伪报独立。未重跑仓级strict命令，原收据单列为work证据；真实独立consumer闭包已复现。 |
| AC-05 | 规范/净删/兼容/health独立审查 | fail：F-01/02。 |
| AC-06 | merge-owned | pending。 |

定向总计：auto-gen lib 11、V02 11、V03 8、core lib 12、auto-gen doc 5 = 47 passed；另external consumer 4 passed。auto-gen bin及core doc均0 tests。work收据末段的“16lib”不作为当前计数（实际11）。依赖auto-lang打印411条旧警告，shim-metadata 1条旧警告；本计划Rust触面不包含这些文件。新增6条测试警告单列F-02，不能用“零新警告”概括。

金样来源审查：`9c4780c54a12bcabe15e3f7836b430e0fb98b262`仅添加测试/fixtures/dev依赖及调整既有markdown预期，尚未迁移生产实现；`git diff 9c4780c54 HEAD -- crates/auto-gen/tests`为空。金样在拆分前已入Git且后续未rebless。本次没有在另一checkout重新执行旧臂，因此旧臂运行收据与此Git来源验证是不同证据，未混称完全独立重跑。

独立反控选读：good/bad/good批次断言含错误模板归属与失败文件缺席；render失败复用既有字节哨兵；dry_run目录枚举为空；core默认double-.at命名及既有空文件合并尾换行；legacy首次无文件与再生成有文件保留差异。未增加路径沙箱/原子批次/异常规模等新要求。

## 规范增量冻结及版本

SD-01新增`docs/specs/auto-gen-core/project.md`，SD-02修改`docs/specs/auto-gen/project.md`；目标/AC映射及单核心、Auto adapter职责与实现一致，`supersedes_spec_components`、`new_spec_components`、GOAL-003/016保持。旧Spec的“简单文本替换”确实与AST/AutoInterpreter实现不符，本次修正文案合理。

两份拟沉淀卡片仍属worktree准备稿，review没有发布main canonical Specs或改ledger。新卡片的“库定义不可见”尚缺F-01对应运行oracle；待补证后才能最终冻结通过。auto-gen卡片“模板执行与guard feed走同一核心”措辞不够准确：源码中legacy模板执行仍在auto-gen，只有guard feed进core，建议T-06同时明确，避免与core禁止解释器规则冲突。

指纹（SHA256）：

- 入场main Plan：`0EC3A466D983162283BB01CAD90A84005EFB4B7CC2CCC342C670BE39A69B584E`（添加本review记录前）。
- main canonical auto-gen/project.md：`E65E864C1680CA37B98F200EC587E9CE89A59753FC6AD75B306299E1146A4B51`；overview：`AB37E25B9EFC9EDA73084AEA1A5F71CADE5CA5B456A328653FA7FD73C14898F0`。
- 准备稿auto-gen/project.md：`DD94510986893B003F5A4A4EFFCFFBBC55C458DB0C3AC04A4634AB5087323995`。
- 准备稿auto-gen-core/project.md：`0AB2DE7A6D9EE5B474399D932869D60A06D6A732D6E9E6EC89ABA9B8FB7878BC`。
- 冻结规范delta：由完整受审commit及base固定；命令`git diff e4ac72a6265c85cdadeedbabc7a0c6fd93948e48 0cd3e3e9f7e448fc95a1ef61b13761ac02d7e65e -- docs/specs/auto-gen/project.md docs/specs/auto-gen-core/project.md`可重建，以上after文件hash锁定内容。
- workspace Cargo.lock：`2887F43A4E75B631F8720C027515EA39D76C13A50538B7C501438D9761A4E028`。
- external consumer当前Cargo.lock（忽略文件，独立解析产生，非发行锁保证）：`8F29B176D19547BD7AA970C7096B19C9B14213C2705B10C6A42DDDC3EAD6E54E`。
- module-boundaries.toml：`8DF801E7748DC07C72427F4DA62302708B1F27D95C751C0D313A78D46532DFC3`；baseline JSON：`9F58E30312AE609D60978D4617FA83C3668F5CC4774523E3D0777026DBF26ACA`。
- sibling auto-down：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- auto-plan-review技能：`0D68BA05276AFF05A143C8F27010EE02B0A9BDED14C27A0133D3A774C20F0CF3`。

## 后续

回work，在同树补两项并提交；保留旧验收ID与本轮证据。T-01/T-05/T-06重开，T-02/03/04保留完成，T-07未通过，3/8。既有AutoFile/AutoCode/lib作用域、fstr_note预存行为只登记未来债，不要求本次修语义。裸cargo t及本crate/adapter定向全测由merge在最终审查通过的新冻结提交上运行；本轮未跑任何整档t/tv/tt/tb/tu/taa/tf，未合入/归档/清理。
