# PLAN-828 R2 needs_fix：三项同批处理

原树 `D:/autostack/.wt/lang-828/auto-lang` / `plan-828-dev`，受审冻结 `cfa67eeceb4555e55b978354d3fd7ab508e80664`。当前还有并发T-08 worker/计数访问器WIP，先对账所有权与实际修改；保留，不reset、不偷偷覆盖。按 `/auto-plan:work` 同树修复，新冻结提交须覆盖实际全部改动，不能仅给旧cfa报PASS。

先读 [review.md](review.md) 和三个独立源控件。B2-C01/C02/C04已测闭合保持，不重做候选模型。

1. **B2-C03/P2仍OPEN**：func_slots以裸生产func号为全局key的前提不成立。两份真实同owner单元的不同函数可各占生产slot1，link当前接受并用后项覆盖。`projection-controls.rs`要求正确来源159..164，或在checked link有依据地拒绝不可表示组合；不能静默259..264。优先在实际执行身份/代码对象与snapshot图之间建映射；若选择约束入口，须有可执行前置验证及合法支持边界，不能只是注释。保留原R1重排和真实编辑反例绿、六槽序、不可变payload身份及旧pin来源。不要把本例误解为要求session声明重排走body快路径。
2. **T07-R2-01/P2**：manifest严格版本与集合校验。当前重复helper可掩盖遗漏main，format999仍matched。正确owner envelope不等于正确目录；未知格式/坏行/重复/缺项应拒warm，完整合法目录允许顺序变化。收编`t07-controls.rs`原两反例，加合法重排正控，保留partial无root与拒绝后自愈，独立ABC/HIR准入不削弱。
3. **T07-R2-02/P2**：F-11 required引用与实际消费。当前manifest edges空、units无artifact目录，GC保活靠每行soft根，不是manifest required闭包。用原shared store API落实当前必需ABC/来源等引用及恢复消费，按既有再生规则分类HIR/provenance；明确历史根退役和共享project/用户所有权，不改783全局语义。原控制保留manifest根、退役独立soft根后，GC删除9个必需工件，admitted0；修后应保活当前恢复闭包且admitted3、值/来源正确。保留普通soft根GC正控；补带独立合法root的共享project哨兵及历史可回收控制，不能以全历史永远root解决稳定水位。

先在interface-decision补“根因→被改变前提→生产路径→机制→正反控→剩余边界”的三行，尤其诊断C03为何旧例全绿却不变量未闭；不要逐例special case。保持Plan r3/原AC/V/C/SD，若真需更改语义/支持面才带具体矛盾回new。

永久收编新控制并完成所有修复后，运行对应定向与必要影响面：新projection/T07、原link8/execution4/lifetime2、b2 13/t07 6/b1 11/session18（明确skip两个512）/contract22/t01r4/r1 86/core、真实commit失败组件、关闭6＋3；shared恢复/owner触面则cache_tier。测试须绑定同一新冻结，保留输入/命令/真实选择/exit/库身份；不要在并发源已变后继续把旧库或新WIP结果冒绑定旧commit。两512和正式字节峰值按其原owner/最终受影响验收明确交付，不用库存计数或文本延期当完成。

四项全部原反例通过不代表本轮残项免修；同样不撤已通过的owner/实际调用闭包/有限切片、事务与第一批机制。三项修完一批冻结送审，不要求每项单独往返。禁止整档t/tv/tt/tb/tu/taa/tf、docs_gen或merge；整份最终验收和merge-owned门pending。
