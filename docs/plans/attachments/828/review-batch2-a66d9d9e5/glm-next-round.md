# PLAN-828 第二批 needs_fix：同树一次修齐四项

使用 `/auto-plan:work`，原树 `D:/autostack/.wt/lang-828/auto-lang` / `plan-828-dev`，基面 `a66d9d9e54e4db1de90edc078ed7d23d161e00d5`。不 reset，不撤后批及第一批机制；Plan r3、executing 0/11 和原 AC/V/C/SD 不变。先读 [review.md](review.md) 与实际控件，不按测试数或摘要猜预期。

## 先完成机制诊断，再一批修复

1. **B2-C01/P1：owner/context**。明确 checked linker 接收哪个既定上下文、每个 unit 的准入 owner 如何与之关联。当前仅名字＋签名会执行异 owner helper=41并返回43。按既有单模块规则拒错上下文，合法同 owner替换43、递归保持；不要增多模块语言能力或公开调用方自证字符串绕过信任门。
2. **B2-C02/P1：实际调用闭包**。区分 unit 的 import metadata 和实际 Call 引用；已准入合法 `hir.test.mark_b` Call 现在未绑定却可产生前置 MarkA。确保 snapshot 可执行前闭合/拒绝它，普通、steps、resume 无 guest 副作用后置拒绝；保留合法 lower 成 Trace 的源程序，不粗暴禁掉既有 intrinsic支持。已有缺函数、签名、重复名、缺入口拒绝不退化。
3. **B2-C03/P2：来源投影坐标**。查明生产 payload span.func 与 snapshot code slot 是不同坐标，明确在实际 dispatch view/结果构造处如何投影。真实代码＋真实 maps 同步重排后 start/end 必须64..69，而非33..33；synthetic偏移正好125..130。不得重写共享 code/HIR/span payload“修正”槽号，保持实际对象身份、旧pin来源和 sidecar身份校验。
4. **B2-C04/P2：有限切片恢复**。额外每段预算不能直接比较累计steps。连续Some(1)应能完成6步程序；Some(0)合理暂停；保留累计 hard fuel/frames/slot/trace，跨段不能重置硬限额，也不能用无限循环或取消时延新承诺替代验收。覆盖剩余预算/边界算术。

先把“根因→受影响生产入口→机制→合法正控/拒绝反控→剩余边界”写入 interface-decision 新 B2 节，再完成实现。若诊断确需改变共享 owner/ABI/语言或资源合同，带具体阻塞回 new；现有四项不要求扩大范围。

## 永久回归与定向验收

- 收编 `link-controls.rs` 的8个 tests（3合法正控＋5失败断言，5失败归4finding），保持输入和 oracle；可整理类型/API，但不能换成会被ABC早拒的 unknown import，或仅核wrapper。来源真实编辑例必须保留。
- 收编/保持 `execution-controls.rs` 四组合：6种槽序的异构类型/命名实参；增长/增槽新33旧32；真实派发副本值对而身份红；自递归/互递归新4旧3及trace一致。
- 收编真正 `session.commit` linkErr 的状态保持例；本次该控制已PASS，**无需重做候选所有权**。组件内 `#[cfg(test)]` 可替代审核include装配，但必须调用真实consumer，核root/source/revision/env generation/records的history+deps+origin/map/canonical/mint，旧trap全等，随后已有good候选仍可提交。
- 修复全部后，跑新控件及影响面的 permanent b1/session(显式跳过两个第三批512)/contract/t01r/r1、core lib，r23a close 6＋3；来源/绑定变化补必要的已有 sidecar/identity控制。若触及共享canonical/798消费者再跑对应定向，不无条件扩全档。
- Some(0)/正切片、fuel耗尽与旧PausedRun经close后继续是预算/寿命关键相邻状态；以可执行断言而非文本声明证明。允许共享机制修复，禁止逐例名单特判。
- 不运行整档 t/tv/tt/tb/tu/taa/tf，不 docs_gen；全部 findings 修齐再冻结同一提交。记录真实选择数、命令/库身份/退出、原反例与邻接正控。更新四行关闭表及§9，但不能代 reviewer写PASS；整份最终验收与merge-owned门仍pending。

交接需含完整SHA、clean状态、基面、四项逐行结果和证据路径。下一责任为review的受影响面复审；不要求每个finding单独交审，不撤第一批已闭机制。
