# PLAN-828 R3窄修复：T07-R3-01/P2

使用`/auto-plan:work`，同树`D:/autostack/.wt/lang-828/auto-lang`、`plan-828-dev@3090f2326f3595a69132c80bee3a2891c5f41b76`。本次入口/结束均clean，T08脚手架已包含，不reset、不撤已有机制。Plan r3/executing0/11，原AC/V/C/SD不变。

**上轮三项已闭**：帧执行槽投影、严格目录集合/格式、required边GC保活和旧根退役的原反例已通过。只修[报告](review.md)的T07-R3-01，不重做它们或候选所有权。

根因已证明：shared store的artifact_id包含required edges；manifest key仍是旧format1＋fp(body)，旧producer同body空edges行占同key，新增边必Conflict。修复必须处理身份/版本前提，而不是吞Conflict、覆盖旧行、禁persist或要求用户清缓存。

先在interface-decision补一行“旧/新目录格式与key、required闭包是否参与身份、旧行读取/隔离/新图发布/旧根退役时序、冷回退和幂等重开”的具体设计。选择既有store合同下可验证的身份策略，使需要不同required图的产物不再占用同一旧key；不改783全局Conflict语义，不新增公共不可信构造口。若换键/格式，须解释旧root如何受控读取或miss，以及cold如何写新/下一次恢复如何自愈。

永久收编`migration-controls.rs`五行（四失败一个合法正控），保留：

1. 按cfa精确producer配方生成的合法旧manifest行，无所需units→restore冷fallback成功且再恢复可warm。
2. 同旧行直接warmup_persisted成功；旧行不被覆盖。
3. 完整合法旧ABC/sidecar闭包先warm admitted3，随后重开持久化不Conflict。
4. 完整旧闭包warm读后真实同内容空白编辑，reuse_hit=1且persist_error=0；图发布成功，经GC仍能恢复正确值/来源。
5. 新required图幂等重开＋同内容编辑正控保持，原R2 required-root/退役GC/用户哨兵控制保持。

fixture通过公共store和manifest读取，不使用私有resident_content_version；不要把反例换成未知format而提早拒绝，遗漏最初format1键冲突。可将测试合并整理，但输入前提和关闭值保留。补至少一次“旧→新→GC→恢复”的整条路径，不能只测新建临时store。

全部修复后跑新matrix、原projection1/T075/link8/execution4/lifetime2/trap组合、b2/t07/b1/session(显式skip两个512)/contract/t01r/r1、core＋真实commit失败组件、cache_tier和受影响sidecar/关闭控制。若确无触及AV源码，上一轮帧映射证据可按输入/依赖/配置一致性明确复用，不再要求重做前端；不要用复用覆盖变化的持久化路径。

只做定向；不运行整档t/tv/tt/tb/tu/taa/tf或docs_gen，不merge。原T08正式测量/IPC、F09双线、512最终、T09规范与告警清理及merge-owned门pending。一次冻结交完整SHA/clean/根因及各关闭值给review，不逐例单独往返。
