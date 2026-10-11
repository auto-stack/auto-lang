# PLAN-761 收尾批量回归（2026-10-11）

代码基线 `5314cf0f18774acc7628237fabbca13cf868362c`，上一覆盖755，760整五落地触发；最高已交付761。三腿串行主检出运行，`--no-fail-fast`，退出均100；不宣称全绿。

| 档 | 测试时间 | run/pass/fail/skip |
|---|---|---|
| tf | 132.405s | 6265/6263/2/519 |
| tt | 150.944s | 5614/5610/4/1789 |
| tb | 104.214s | 5298/5296/2/1541 |

时间为测试段；tf/tt/tb首次切换含编译/链接，未计入表中。画廊/assembly探针暖缓存仍有83–110s尾测。

## 分诊

- REG-3：app_registry的scan/launch三腿同红。scan策展19 vs20，缺041-auto-edit；launch走主检出target/debug/auto-edit.exe后20s incubation超时。该本地文件75,840,000字节，修改时间2026-10-11 11:46:48；已有KNOWN-DEBT REG-3记录同名产物噪音。窗口app_registry代码零变更。未改名/删除其他会话产物。
- back_provision：仅tt失败，OS10013 PermissionDenied导致懒启proxy缺席。738-independent-review已有同测试间歇观测，tf/tb本轮绿。确切端口/排除段未另做实验，不伪称已定位根因。
- 新观察chart：仅tt的gallery_chart_components_render_geometry失败（3.426s）：/bar-chart缺h6；日志handler_BarChart_Init parked/CpuRunnable。tf同提交6.435s通过，tb通过。单次drive_scheduler_to_quiescence对仍在途CPU片的progressed判据为待证候选；无法确认属于feature/负载/跨仓源/生产缺陷。归档Plan711追溯旧契约，窗口vm_bridge只改注释。已建立[PLAN-762](../762-chart-init-batch-flake.md)分诊草案，保持原断言，未修复或销案。

## 范围与限制

- PLAN-761原资产/路径/配置/合并验收已通过；不撤销目录整理，不重开其归档。新观察由独立Plan跟进，未把未知根因塞入known-red豁免。
- aavm触发面窗口未命中且lib.rs无变更，跳过taa；不跑t3。
- 每腿启动前无其他cargo/nextest门禁。他会话随后启动fix-bp-root-env-lock scoped检查，tb期间启动日常档；并发情况已记，不作因果推断。本会话未并行两个批量档、未停止他人进程。
- tf/tt/tb运行代码均5314cf0f1；tb期间只提交762草案，不改变实现。用户原有两处WIP保留，未纳入提交。
- 单点状态见[批量收据](../.last-batch-regression.json)。原日志保留主检出已忽略scratch/root-cleanup-761/batch-{tf,tt,tb}.log，SHA256如下：

- tf: `6bb9461cbc5132b47407ec3f5548db2f51b5c8f911a85260b48f28824e5f4057`
- tt: `04850645b195c9b2648cf3711a1614c1ad02ef0c69f318ffe9649f645cbfdc5d`
- tb: `7af8751426b2a69bd8421ec7e6cb9c08bd3c45dd0659f0904b5121a660b782ea`
