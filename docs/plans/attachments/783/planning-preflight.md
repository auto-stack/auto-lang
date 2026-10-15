# PLAN-783 r1 起草预检

日期：2026-10-15。stage: new；outcome: pass；next: work（待用户执行指令）。这是草案一致性预检，不是实施验证或独立最终复审。

## 授权与范围

用户要求规划下一计划；本轮仅调查、取号、编写计划与来源记录，不创建执行树，不实施，不创建/联系代理。新计划号783由主检出 scripts/new-plan.sh 分配，调用处用 Local\AutoLangPlanAllocation mutex串行，并核对活跃/归档最大号和.next-id；分配后核唯一性。

主检出调查基面：1b9cb8db692eab7e04e8f71155f159e375b52edf。源文件SHA-256见planning-inputs.tsv；782等在途计划可能继续变化，work须重核实际状态。本次保留780/782、债务表及其他人的未提交文件。

## 合同预检

- Design39/40/41覆盖架构先行要求；以Spec与实际公开接口界定现状。
- 782不是强前置；必验core纵线与未来text-record集成分开，不使用未合入提交。
- T-01为有界全路径可行性门槛，不先假定二进制HIR恢复、公开API或旧BlobStore并发语义已成立；失败不得跳过AC。
- A00–12是完整边界登记，不承诺本计划实现所有payload；实际codec及未来CST/FFI接缝明确区分。
- AC-01–06归work，AC-07归review，AC-08归merge；9项任务初始0完成。独立review同树顺序进行，绑定提交，不以自检代替。
- V-01–12均有owner、入口、区别错误实现的oracle及边界；新入口尚未实现，不作为已执行证据。
- 分页128MiB负载与GC64/48MiB负载分属不同store；实验预算不冒充用户批准的产品上限。GC明确硬pin超额时的有界退让。
- 生命周期探针只证明文件/映射reader，不冒称DLL/callback安全；无pack，不冒称compaction交付。
- 源码/用户交付品不进入删除集；坏块/未知schema/语义非法/资源不足分别受控；缓存HIR需重新verify。
- 实施前定向必验不能后置；完整独立probe套件merge-owned。无crates源码或语法参考变更，不运行根cargo测试/docs_gen。
- SD-01–05均有AC和T-07/09承接，当前仅拟增量，未编辑canonical Specs。

## 现场核实

既有experimental/ac-core/tests/source_native.rs明确四条oracle：p01=14、p04=3、p12=0、p13=70；源文件位于fixtures/source/valid。已核对parser、compile_parts_with、lower_object、link_object_staged和BlobStore公开入口。新测试filter、脚本和probe模块是T-01–06的未来产物。

起草检查：0–10编号章节、frontmatter/current_step/total_steps、AC/T/SD/V追踪、链接存在、取号唯一性及diff空白检查。未运行任何编译或测试，不宣称运行性能或实施可行性已经通过。
