# 871旧方法缺口：PLAN-881规划调查（2026-10-29）

只读Spec/源码/永久测试调查，无新运行证据。调查master 62fb3e3f1；后续取号提交ce77c3fe8。执行合同见[881](../881-shared-record-method-lowering.md)，删除门见[871](../871-ac-native-switch-and-old-lowering-retirement.md)。实施必须刷新实际revision。

864的T-01/T-02已勾、T-03未勾，frontmatter仍drafting（进度簿记不一致仅记录，不替owner修改）；冻结件不是已交付代码，本包不消费其WIP。

## 旧能力与拒绝点

| ID | 已核旧能力/依据 | 当前共享链静态事实 | 881关闭条件 |
|---|---|---|---|
| M-01 | source-record static；record-source/valid/p01-records.at的Pair.new；ACE-M1 src/R08.at | hir_adapter.rs的lower_checked_data/owned遍历Method即lower.method，即使未调用也整体拒绝 | 原R08=42、771原例=0；不能删方法或改原源为自由函数冒充 |
| M-02 | 771 record_native::receiver_once_then_args_in_order、嵌套i.p.left_is | hir.rs约定view params[0]，verify.rs检查owner/type/mode；shared还未映射Method | receiver一次先于args，原trace=aab、exit0，view不复制owner且不允许写 |
| M-03 | source-text-record沿771；782 p01 Tok.new(move Str)、text_len；text_record_native::v12_token_microprogram | owned入口整体拒Method；mn lower.rs已支持RecordView表示，不能把根因归成缺方法机器指令 | p01手算30、Token原源57；一LIT7B+一CLONE7B各一次free、normal live0 |
| M-04 | 相同源函数级生产/参数模式 | lower_function_data_unit/owned_unit_with_modes也拒Method；owned_fn_sig_for_def/callee_mut_record_params提取body模式仅匹配Function | whole/unit模式一致；move/mut显式参数及回写责任不丢，receiver不重复 |
| M-05 | 函数级稳定身份/依赖/恢复 | cache::split_units用fn:<name>；function_queries的scalar入口显式只收Function | work沿data/owned真实入口接通；同名方法/自由函数不混链，body改只生产自己、接口依赖失效、重启重验 |

core/scalar/text没有用户record方法准入，继续拒非法Method。text内建len/slice等是独立指令lowering，不是本包用户方法派发。方法静态库存不是全旧五profile覆盖库存；871仍须逐profile核验所有消费者。

## 与871其他缺口分账

- 原R08当前ac=gap:mn-lower-method、av=gap:av-lower-method，独立oracle42；881为实际producer，交付后871接受证据才关闭。
- 870登记view eq/ne的gap:upstream-s-eq是通用文本Bin路径，另列S能力owner待立项，不归方法自动关闭。
- split_units内嵌类型索引未重映射属跨函数类型链接。方法必要修复在881 T-04，必须扰动多名义record/异构签名类型表序验证；非方法通用兄弟族继续由871列账。
- mut资源字段替换释放时点、mn多帧trap回卷、旧native fatal与AV台账是已登记边界。若阻塞方法原合法例，881 T-02须给差异/方案且未决不pass；全域补齐不自动属于本包。
- 871最终仍独立承担旧五profile、CLI/协议/7脚本/cache、shadow、切默认和G-DELETE；保留text_resources.rs的verify层TextResourcePlan及调用。D881不能代替全局G-COV。

## 交接

864拥有公共source/HIR/lowering/会话与H注册/binding。881只读准备可提前，公共实施等D864和实际S/N/H释放接受，不能因“不同函数”并写。默认866释放后、865前插881；提前插窗必须登记owner释放接受。871删除还等覆盖/消费者/旧键恢复门齐全，未就绪让窗。不自动联系其他会话。

交接含Plan修订、delivery、Spec/profile/schema/API/lowering版本、cache失效/恢复、兼容证据/限制、释放/接受。接受864后重验原L04/L05=42与List<Str>变体51，profile4 opcode/资源/预算不变；不扩collections用户方法。

## 读取时SHA-256

| 文件 | SHA-256 |
|---|---|
| docs/specs/auto-ac/source-record.md | 53ef209ae7fc87cd6023384fdc0d173aa45a627650f04f33d73917d1cb6461dc |
| docs/specs/auto-ac/source-text-record.md | f0560f00ec8467f4288e62f67e74eea512dbc7bfd7cc5e0a6bacc008227583e2 |
| experimental/auto-mir-lower/src/hir_adapter.rs | 4a5e885f18d849ac57e957be7846a43d7933522072aa5ba328049e024c5b50f7 |
| experimental/ac-av-probe/probe/src/function_queries.rs | 47007f6acdcf5ad6baf277abbd6b9abac2ce91bd0846707edb7147cda29767f8 |
| docs/plans/attachments/864/work/collections-source-contract.md | 27f0df6dd5ac5a5cfc5facdfb3ee97a12585c21c2e50f073f3e4a1af7612da71 |

命令：Get-Content相关Spec/fixture/tests；rg检索DefKind::Method/lower.method/receiver/ParamMode/split_units/R08；Get-FileHash -Algorithm SHA256。未执行cargo或fixture；T-02/T-05负责运行验收。
