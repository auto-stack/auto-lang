# PLAN-815 F-815-T07-R1-01 残留项定向终核

2026-10-22，**pass（仅本项受影响范围）**。F-815-T07-R1-01 / P2 CLOSED，无新增finding。前轮联合修复五项至此全部关闭；完整T-07、T-08最终交接及T-09终审继续待办，Plan r3保持executing10/17，本记录不授予整任务/整Plan交付pass。

## 对象与独立性

- 唯一树D:/autostack/.wt/lang-815/auto-lang，branch codex/plan-815-dev；冻结HEAD **c203d9642a58a3bceeb2075a7ee116e79a416e45**，修复 **f8c01f76bbd9b3c151388f4f0744701a808d7598**，base c050e718e09b66ebb6d7925c1f4e0ce89bb10dc5。实现提交只增加lifecycle.rs的12行断言，后续docs为交接证据，无生产增量。
- 工具SHA256 **f21db18136403d1124ecbc1338f61c5603544401c434e7d12fad20a60c37dfb5**，生产实现仍是e351cb2a，符合交接。开始/结束同HEAD clean，源码/依赖/配置/锁/当前Specs/二进制零漂移，见[baseline](baseline.json.txt)、[终核](final-baseline.json.txt)。
- review_scope=phase：只复核上一轮唯一残留的产物成功oracle。reviewer=Codex `/root` 延续独立审查上下文，有早期合同/账本参与限制，未参与本次实现；按/auto-plan:review独立执行原关闭控制及真实验收，没有创建agent/第二树、修改实现、canonical/live ledger、合入/归档/清树。
- [证据复用核验](evidence-reuse.json.txt)确认前轮全部已钉输入中仅lifecycle.rs变化。Rust front、Ark ID、close有界、PM缓存四项CLOSED沿原报告与未变源码/配置/输入保留，不机械重跑其全部测试；本轮重跑受影响产物oracle和lifecycle。777前轮0v/3g记录仅按未变manifest/依赖边/规则范围继承，不声称本轮完成master并集或792。

## F-815-T07-R1-01 / P2：CLOSED

既有AC-04/V-04要求的本Vue成功产物面：completed、error=None、真实入口及JS束存在、入口有实际存在的应用JS引用；失败/不可消费结果不能通过成功验收。前轮残留是CSS引用可以与未被入口加载的任意JS文件分别满足两个断言。

本轮新增js_references计数，在已核验文件存在的同一引用路径上仅对.js递增，并要求js_references>0；保留原报告、入口、JS文件及所有已识别资产引用的存在性断言。未把CSS-only例删除/ignore、未改弱原负控或改变产品能力。

独立重放[dist-reference-extra.py](dist-reference-extra.py)，逐字提取冻结测试的产物断言块，以真实私有文件做有效/失败控制，结果见[原文断言、输入和退出值](dist-reference-extra.json.txt)：

| 控制 | 本轮结果 |
|---|---|
| 入口引用存在的assets/main.js | exit0 |
| 入口引用MISSING.js | exit101，具名缺失资产 |
| 入口没有任何资产引用 | exit101 |
| 入口只引用存在的CSS、没有JS加载 | **exit101，entry document loads the application JS** |
| 入口引用存在的嵌套JS | exit0 |

该控制证明成功验收器对原故障敏感，不把人为坏HTML解释为产品真实生成器已产出坏文件。范围是原关闭条件和本生成器的Vue打包形态，不作通用HTML解析器承诺。

随后在同一冻结树实际执行：

`cargo test --locked --offline -p auto-ui-tools --test lifecycle -- --ignored --test-threads=1`

**3/3通过，20.15s**：真实安装失败负控、真实Vue构建/dist成功正控、generate→validate→run组合。详[lifecycle.log.txt](lifecycle.log.txt)。新断言没有把真实正常构建判红；默认执行会跳过的三个用例在本轮确实通过--ignored运行，未拿零选择充当验收。

唯一净改Rust文件rustfmt逐字相等，见fmt.json.txt；diff --check通过。无本轮新增编译诊断，依赖/UI包原有诊断沿原归因保持。本轮没有跑整档cargo t/tv/tt/tb/tu/taa/tf/docs_gen；没有为纯测试断言改动重建无关生产包或更新777基线。

## 范围、规范与后续

AC-04本项及当期测试卫生通过；其余完整AC不由本记录重新判定。[当期SD影响核验](spec-delta-reviewed.md)确认无产品规则变化，只有成功验收区分性加强；canonical与live ledger未发布。

已知未完成的完整master并集/792/fresh/locked发行、资源/许可证审计、V-01..04字段/兼容/失败矩阵、协议fixture服务端四回执/深度负控、real_tree Linux可移植缺口，均继续由work完成；T-08须依据这些真实结果和当前A规范定稿，随后T-09完整独立终审。不得以本项关闭提前勾T-07/T-08或标整Plan reviewed。

**下一责任角色work**：按既有授权推进完整T-07→T-08定稿→T-09。最终完整review pass后，merge owner才在冻结输入上统一跑裸cargo t及实际触面档，再串行实际合入、Spec/ledger/收据与归档清理；这些merge-owned门与发布均pending。Plan executing10/17，现有唯一树保留。
