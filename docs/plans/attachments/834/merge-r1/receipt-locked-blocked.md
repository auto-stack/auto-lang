# PLAN-834:r1 T-10 merge receipt — blocked, 2026-10-23

受审冻结提交`ebeab12d2888b9c9fb7e00e3c24479fb3c028011`、SDK/两个实验workspace锁/180项源码资产身份未改变。独立R1 PASS保持；计划仍`reviewed / 9 of 10`。本次未修改实现，没有合入、规范发布、ledger投影、归档或删除工作树。

## 已执行且通过

- driver完整`cargo test --locked --offline -- --nocapture`：42 passed /0 failed /0 ignored；完整`cargo build --locked --offline`通过。
- AGP完整同名test/build（`--target-dir ../autographics-auto-driver/target`）：162 passed /0 failed /0 ignored。
- 原隔离冷消费包C的完整test：42 passed /0 failed /0 ignored；前后verify_source均通过。原work隔离冷构建/producer不可用收据继续有效，未冒称本轮重做隐藏操作。C显式worker16MB、RUSTFLAGS保留Windows PE32MB。
- 12/16 AGP原生探针通过：gpu-foundation、shared-runtime、ag2d-breakout、ag3d-risk-slice、ag3d-squash、lifecycle、metrics、verifier-negative-controls、surface-compose、surface-scenarios、surface-lifecycle、independent-consumer。三项Surface含正式像素/资源/输入路由负控，不抵扣待完成的实际窗口输入门。
- independent-consumer实际从源码导出B、locked release构建、与A确定性回放29字段对比零差异，closure/collateral通过。其AGP_BASE_COMMIT绑定受审SHA；另prepared收据核整Plan仅两实验workspace/拟议Spec写面，无root生产代码/锁变动。
- 根cargo整档不适用本合同experimental-only写面；未跑t/tv/tt/tb/tu/taa/docs_gen/tf。

## 未通过与诊断（不豁免）

`iced-shared-device`首轮和一次具名诊断重跑：实际初始截图及同设备路径成立，但click未观察到；901tick有界退出，缺after_button/resized截图，exit1。原报告与日志保留在agp/iced-shared-device、agp/iced-shared-device-retry。截图已检查按钮位置；实测dpi192/物理2000x1600。外部诊断一次实际按钮点击和队列move/down/up均记录；未修改任何应用/测试oracle。

[桌面诊断](desktop-diagnosis.txt)和[复查](desktop-recheck.txt)：前台为“Windows 默认锁屏界面”，SetForegroundWindow=False，真实键鼠无法到达探针。请求用户解锁并保持桌面解锁；未以超时或synthetic headless结果替代真实输入。

另一次原生Breakout无输入160帧回放exit1：completed tick停留14/50，host ticks628，实际sounds1/1，但errors/failures含`callback budget exhausted`。见[native report](native-breakout-noinput/window.json)。这不是pass，锁屏条件下的预算拒绝尚需在解锁环境重跑归因；若仍出现，回work修复并对受影响内容重新独立复审。未静默豁免，也未把待归因问题宣称为已闭合产品缺陷。

## checkpoint与续跑

prepared：受审身份/写面/当前canonical输入hash、ledger解析与writer探测已记录。在线store不可达、ledger v70可解析；仅准备了guarded offline projection脚本和字节备份，**尚未执行投影或canonical修改**。prepare-specs.py受runtime-gates-complete.txt前置保护，该标记未建立。

landed / ledger_refreshed / archived / cleaned：全部pending。autographics-dev仍540c1e70f0a4682af245191ffb36fa0d8a636665。唯一worktree、plan-834-dev分支及生成消费包B/C均保留；没有运行清理或删除。预存其他会话docs WIP保留。

解锁后先确认实际桌面条件，再从唯一实施树运行[remaining-window-gates.ps1](remaining-window-gates.ps1)，补Iced counter/content两道正控及blank_blit/transparent_blit反控、surface-offscreen/input/ime-os window、双游戏native正负控；重跑no-input/audio/scene并对预算拒绝归因。不可重跑已通过的完整测试来掩盖待完成输入门。全部实际通过后才准备/核对Spec delta、只ff源码到autographics-dev、master知识投影、归档与wt-guard clean清理；main canonical/ledger若有并发变化先重核快照，不能绕过hash guard。

原完整runner与外部诊断脚本仅是证据/续跑辅助，并非实现更改。最终摘要见[gates-summary](gates-summary.json.txt)。
