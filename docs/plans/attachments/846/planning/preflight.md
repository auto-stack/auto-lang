# PLAN-846 r1 planning preflight

2026-10-25；stage:new；草案静态检查通过，G845完整交付pending。

- Plan SHA-256：`7485145fc1532cc42d8977af51602e6178e57d9cfd40fd84fb3a4c0c5f1001d0`。
- 结构：0..10节、10任务/10AC/12case/4Spec delta唯一；owner_stage = work8/review1/merge1；全部AC有任务负责。状态drafting0/10。
- 取号：主检出既有allocation.lock排他句柄下重核active/archive max与counter，new-plan.sh分配846；next-id847。未建846树/branch、未prepare、未实施/构建/运行、未创建agent/chat。
- 来源：26个固定Git对象按各自ref验证blob、原始SHA-256/字节数通过；19份文档输入hash一致。e518为产品观察，a0e2两对象为845测试定位修复；不将候选冻结为846基面。
- 前置状态：845期间从e518独立R2 reviewed9/10回退有界测试定位修复execution_done8/10，随后a0e2独立R3 pass恢复reviewed9/10；完整门/实际合入/知识归档/guard/到期收尾仍pending。G845须全闭合，T01再核正式完整SHA/SDK/三锁/Spec；不复用旧R2作新完整门证据。
- 可行性：T02最多两种局部装配方案，真实Host builtin ModelData/导入Registry → spatial快照 → actual Auto即时query → GPU/Stop/项目重开；版本/精度/128节点原CPU预算不能成立则回new，不能改SDK或暗减分母。
- 兼容：v2/profile8显式选择、v1 API/bridge/profile7及reserved集保持；旧合法ag_aabb_* helper仍有效。同raw bytes可共享payload，准入/编译/绑定/Undo/闭包身份按API区分，scene4字段结构不变。不修改840/845/841及Schema/ACE/HMR的范围。
- 几何与预算：有限叶子AABB/全模型box8角变换、world坐标、闭区间、visibility独立；group/跨维/坏参数与unavailable具名sticky错误；每callback32次请求、33失败。原100k/250ms/30s/2s总预算与128节点控制保留，误差容差不改bool分类；隐藏/退化/深组/多mesh/同callback写后查均有case。
- owner与证据：每资源每Run最多一次local box扫描，warm不重扫/重传；20轮source/VM/spatial/GPU最后owner与合法pin、同binary仓外2D/3D trigger逻辑、OS/GPU最终像素正反、partial/source consumer逐项由work关闭。无engine事件队列或物理解算保证。
- 时序：本轮未跑Cargo/docs_gen/build/原生/GPU；未来work/review定向/check/fmt，final独立pass后merge才冻结完整矩阵，source只落autographics-dev，root/迁移门保持。
- 尾检：相对链接/锚点与附件存在、ignore规则、空白与scoped diff通过；19份输入未改、canonical未写。

结论：可评审的下一Plan草案；G845实际完整交付后按后续work指令启动T01/T02。所有案例为拟议验证，本轮没有执行pass。
