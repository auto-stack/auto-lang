# PLAN-838 r1 — planning preflight

2026-10-23；stage:new；outcome:blocked（G837完整交付pending）；草案静态检查pass。next:837完成T-10后，用户启动838 work，先T-01冻结再T-02可行性。

- 主检出排他allocation锁+new-plan脚本取838，active/archive唯一，下一号至少839，锁文件保留。
- drafting r1、0/10、章节0..10；10任务（8work/1review/1merge）、10AC、11case、4SD唯一；依赖/验收owner完整。
- 23个不可变Git源码blob与SHA256复核，16个知识/计划输入记录WIP字节指纹；在调查中吸收837 final R2 pass，仍不冒认其落分支/归档。
- 实际renderer512实例/`as u16`与合同支持边界核对；大索引/坏buffer/同名mesh及先import image的缺口安排在正式新资源准入关闭。
- 概念核查：静态无贴图子集、GLB/node/decoded/GPU/live-history/磁盘残留预算、资源ordinal实际Auto输出、GPU staging失败回滚/旧Arc复用、v1/v2严格导入/v3显式保存一致。
- 文件合同为资源先行、单入口最后提交；owned正确孤立blob可残留并有界/可重试，不宣称多文件原子。项目锁/entry锁顺序及partial残留由T-02/06明确闭合，外国文件不删不改。
- T-03 codec/预检、T-04上传/owner、T-05 Auto事务、T-06 Host/文件、T-07原生/搬迁各自关闭相应case部分，不提前依赖未建的正式入口。
- 新scene-assets为proposed；scene-hierarchy须837先沉淀；本轮未改canonical/837实施或Plan。
- 本地链接全部存在，新Plan及附件未被ignore规则屏蔽；无838树/分支。
- 本轮仅Plan/附件/.next-id；未准备/实现/build/Cargo/原生图形测试或创建agent，保留他人WIP；所述test/fixture是待实现任务。

Plan SHA256: c0d508738ca7a8a889954ed0c071178898f9c5fb98e11ea10ff7e3b7c35fedc0
