# PLAN-770 第五轮最终独立复审

2026-10-14，review_scope=final，outcome=**needs_fix**。独立审查上下文，未参与实现，无额外 agent；依据当前主检出合同、canonical Specs、同一实施树的已提交代码与本轮实际运行重建结论。

- plan_revision: 2
- worktree: `D:/autostack/.wt/lang-770/auto-lang`
- branch: `plan-770-dev`
- reviewed_commit: `1f8027500d52cc69bfe279fe652c4f454c919523`
- base_commit: `e849e06f976816dda3db0c68a520d485a69904a7`
- dependencies: 772/774 已归档并在基面；771 未合入；runtime 与两锁文件受审提交内。
- skill SHA256: `0D68BA05276AFF05A143C8F27010EE02B0A9BDED14C27A0133D3A774C20F0CF3`（auto-plan-review/SKILL.md）。
- 输入/产品指纹：[baseline-fresh.txt](baseline-fresh.txt)。主检出期间发生其他计划簿记/门禁顺序修复合入，结束 master 为 `4a9c5958c523343f2458a2e19cfbce1bbad01b07`；相关 canonical 与实施树六份输入逐一 hash 相同，见 canonical-comparison.txt。实施 HEAD 与 clean 状态保持。
- SD 冻结：[proposed-spec-delta.fresh.frozen.md](proposed-spec-delta.fresh.frozen.md)，SHA256 `DCC0717A873EEB4D8054716FEF77858DA80E1CDD09E4B1441B94D400AA8F2307`。未发布 canonical/ledger。

## P770-R5-01：短路跳过侧丢失已执行 LHS 的临时释放（P1 blocking）

原合同 AC03/05、V04/V07、774 D04 要求每分配唯一清理，临时保持至完整语句尾。受影响 T05/06/07。

入口：`src/native.rs:2202` 在进入整个 Logical、尚未求值 LHS 时记录 temps_mark；`:2440` 把从此边界起的所有 pending_temps 都当 RHS 临时，在 skip 边改传 null。因此 LHS 已产生的拥有缓冲也被错归到「未执行 RHS」，语句尾 null 守卫跳过它的 FREE。

可直接重放的原例：

```auto
fn main() int { let ok bool=("a"+"b").len()==0 && "c".len()>0; if ok { return 1 }; return 0 }
```

`p01-lhs-and-skip.at/.atom` 两腿 check0/build0/run0，stderr 为 CONCAT id=1、TEXT-SCALARS n=2，**无 FREE**。p02 镜像 `||`（左 true 跳过 RHS）同样漏 FREE，run1；p03 嵌套 Logical 已分配 id1/id2 后外层跳过，两枚均无 FREE。三例程序化 Bundle 均 Checked（api-fresh-results.txt）；真实程序正常返回，不能以 fatal OS 回收豁免正常语句尾义务。

正控制 c01（两侧执行）两腿 exit1，FREE id2 后 id1；历史 R4 p01/p03/p11 的纯 RHS 跳过仍零分配，p02/p04 执行侧仍正常。R4-02 **旧例闭合成立，但完整路径不变量未闭合**；新实现引入这条 skip-side 漏释放回归。

纠正要求：在 LHS 已完成、RHS 将进入时冻结 RHS 专属临时边界；保留此前临时的真实指针/已合流指针与语句尾顺序。永久 oracle 覆盖 &&/|| 执行与跳过、LHS/RHS 各自及双侧拥有临时、嵌套 Logical，以及后续 mark 仍先于 FREE；保持既有 4096 帧/语句尾测试，不以提前释放替代。

## P770-R5-02：嵌套 break/continue 的目标出口状态被 if join 丢弃（P1 blocking）

原合同 AC03/04/05、V04/V06 与 774 D04 要求追踪借用来源、前门拒绝悬空借用；受影响 T02/04/05/06/07。

入口：`src/text_resources.rs:1474` 确实把 break/continue 的 frees 标为 ScopeExit，但 `:1594` 的 if continuation 只保留 incoming，**同时丢失终止臂的 backing/dead 状态**。`:1637` 的 AwaitLoop 只有最终 body_st，没有按目标携带独立 break/continue 出口。因此不能从 if fall-through 状态恢复实际到达 loop exit/re-entry 的借用来源。

```auto
fn main() int { var v str=""; var i int=0; for i<1 { if true { let s Str="abc"; v=s.to(str); break } }; return v.len() }
```

p04 上例与 p05（在同一 if 内 i=i+1 后 continue）两腿 check0/build0，程序化 Bundle 均 Checked。实际 FREE id1 **先于**循环后的 v.len()。p04 本轮 source exit3/`TEXT-SCALARS n=3`（读取已释放内存恰仍有效）、HIR exit70/`RT-FAIL code=-7`；p05 source/HIR 都 exit70/-7。历史附件 replay-output.txt 的 p04 source 也曾 exit70，证实 UAF 结果可变。oracle 应要求前门 `verify.owned-scope-escape`，不能按某次程序返回3判断合法，也不能仅要求运行期 trap。

c02 正控制 owner 在外层仍活着，经过相同 break 包装后两腿 exit3，读取先于唯一 FREE。R4 p10 自然块出口拒绝、r4 循环体自然出口永久测试保持；R4-01/03 **旧例闭合有效，目标控制流出口尚未闭合**。

纠正要求：将正常 continuation 与 return/break/continue 的目标出口分开保存和合流；if 不污染正常后续读取，同时 loop 消费所有到达自身目标的出口与 re-entry 状态。永久组合包含直接/嵌套/双臂 break/continue、自然迭代、存活外层 owner 与静态借用正控，保持 returning-move 合法读取及无重复 FREE。单纯恢复「所有终止臂并入 continuation」会重开 R4-03。

## 实际验证与覆盖

完整 fresh gate：`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-text-770.ps1`，在上述实施树运行，**13/13 PASS**。收据 gate-20261014-172651，持久副本 [gate-fresh-pass.txt](gate-fresh-pass.txt)、gate-fresh-raw/。ac-core **244 passed/1 ignored**，runtime **20 passed**；两 workspace check 无 warning、fmt clean；741/745/767 实际重跑。唯一 ignored 是原有 60s descendant-pipe 专项，与本次变更无关；未运行根 t/tv/tf/docs_gen。源码审查未见 stray dbg/TODO/FIXME 或新调试输出。

本轮新集 **9 source + 9 HIR**（p01..06/c01..03），原始结果与 input SHA256 在 replay-fresh-output.txt；源构造真实 Bundle→verify_text 结果见 api-fresh-results.txt。p06 命名实参同源 wrapper 被 mut-conflict 拒，c03 异源 exit4/LIFO；没有新增该项发现。早期附件这两个 .at 使用错误的 `=` 命名拼写，已改为实际语法 `:` 后重跑；旧 replay-output/api-results 不作为本轮有效证据。

本轮重放冻结历史原字节：R4 **15 source+15 HIR**，R2 **17 source+19 HIR**，R3 **5 source+7 HIR**，见 historical-r4/ 各层 replay-fresh-output.txt（合计37 source+41 HIR）；原例及正控制恢复返工后预期，包括 marks 的专用旗标。Drop/requires/非法引用的实际 Bundle 变异与标量/文本4096生命周期由本次完整测试重跑，断言源已核对。未因新增反例跳过其他既定风险面；不宣称穷尽所有合法控制流组合。

复现：先运行 fresh gate 保证 CLI/runtime 产品，再按 replay.ps1 运行 source/HIR 两腿；replay-api.ps1/emit-probes.rs 用当前 debug rlib 链接后输出 Bundle 结果及 canonical Atom。API 辅助程序链接需 `-L native=<windows_x86_64_msvc-0.52.6/lib>`（初次遗漏导致链接失败，补齐后全部实跑，非语言诊断）。

产品指纹说明：baseline-fresh 在 gate 尚运行时采集；745/767 gate 中的实际 `cargo build --locked` 于17:26:44重建 CLI，旧 hash `CDBF56A3A76F56BDD0AF4F5C217C147F0A878360FA7CB6416E050DFAC64D76A5` 换为 `40271E9EE817EEBDAACDB38E24BBA6C68E52758DD080F86905C7CD57473CD88B`。本轮新例及历史 CLI 重放均在 gate 返回之后，使用该重建产品；结束 hash 见 final-stability.txt。HEAD/实现 clean/Specs/SD/锁/runtime/合同均未漂移；不是把 gate 前缓存 CLI 当作本轮产品。

| AC | Task / 实现 | 本次判定与证据 |
|---|---|---|
| AC01 | T01/02/05，身份/descriptor/verify/receipt | 受测面 pass；新旧身份与需求隔离 tests/gates 保持 |
| AC02 | T02/03/04/05，Unicode/char/native | 受测操作面 pass；全 text_hir/source/native、runtime测试与真实两腿 |
| AC03 | T02/03/04/05，resource plan/native/runtime | fail；R5-01 正常路径漏释放、R5-02 悬空借用放行 |
| AC04 | T02/04，真实程序化 verifier | fail；p04/p05 Bundle Checked，未拦 unsafe input；既有变异断言保持 |
| AC05 | T02/04/05，顺序/短路/来源/深输入 | fail；短路临时责任及目标控制流状态缺口；其他 marks/深输入/静态诊断保持 |
| AC06 | T06/07，gates/SD/hash | partial；完整 gates 实跑绿、边界/指纹明确；SD完成声明须待上述不变量闭合后定稿 |
| AC07 | T08 独立复审 | fail；两项原合同内阻断，不授权以债务延后 |
| AC08 | T09 | merge-owned pending |

SD01..05 的路径/before/有限支持边界与 touched_goals 空解释有效，冻结 hash 未变；实现完成声明不能以聚合绿证明。没有修改合同、缩围或增添新的规模承诺。

next：同树 work，先修正诊断/状态及覆盖模型并永久化新旧组合 oracle，再提交、重跑受影响重放与完整 gate、交独立 fresh final review。T01/T03 保留，T02/T04/T05/T06/T07 重开，current_step=2/9；T08/T09 未完成。禁止 merge/归档；没有修复实现或更改 canonical/ledger。结束实施树 HEAD/clean/相关输入复核见 final-stability.txt。
