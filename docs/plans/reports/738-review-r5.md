# PLAN-738 revision 3 补充复审 R5（2026-10-10，needs_fix）

- reviewed_commit：`9c255993b2a3163740cf886349013c7bea6aae32`，既有 worktree `D:/autostack/.wt/lang-738/auto-lang`，入场及临时探针恢复后均 clean。
- Phase 3 diff base：`2c1b4a763280b3994f51b441714b4f2120328fb3`；合入比较基面 `6d69dbdc78d99f23ba90a37c9559f7fb54dd7be3`；实际本轮未合入。
- auto-down：`895f8d0f9355c9f5ec3ce8fca268bdb768395846`，未改动。主检出入场 `49f17c5362591e63b5f42040602028c852201fb4`；其他 UI WIP 不参与本轮。
- 规范/收据/日志 SHA256、探针摘要与逐名红名单见 [738-review-r5-baseline.json](738-review-r5-baseline.json)。SD 冻结文本：主检出版本 `1b05d3d441830e855f0eae837a813b9820cd83e5` 的 `738-sd-drafts.md`，SHA256=`72558019bad702e9ae411ee047c011643c1689fb06a9dab1ff61318b98cfdf19`。
- 独立性：本会话参与过早期实现及 revision 3 计划修订，未参与 Phase 3 实现；本次为补充复查，不冒称新的完全独立上下文。结论由源码、实际 CLI 和实际生成/新鲜度反例重构。保留原独立 R3/R4 记录，但其 pass 不再覆盖本轮发现的未满足验收项。

## 1. 结论与范围

**needs_fix。** 删除 Resolved-only 包装豁免、独立 ADAPTER_RULES、HTTP/CLOSURE 修复、双指纹模型及 R3 测试属性修复均有实际成果，T-09/T-10 不回退。本轮新增两个确定性反例和一项未完成的消费者验收，分别命中 T-11/T-12；SD 与验收对账不能继续标整体通过。计划回 `executing`，重开 T-11..T-14，原 AC-01..08/SD-01..07 与 revision 3 合同不变。

## 2. 必修 findings

### P738-R5-01 / P1：首次 lock 物化豁免没有结束条件，后续依赖漂移仍新鲜

- 触面：`crates/auto-man/src/rust_ui.rs:4022-4042`，`api_gen.rs:1071-1089`。
- 根因：`lock_freshness` 的条件为 `a == b || a == "absent"`。生成前没有 lock 时，收据写 `absent`；首次构建/检查没有把收据绑定到实际 lock。该收据随后对任何 lock 内容都返回 true，并非一次性的“首次物化”。读取失败还会在 current 侧归一为 absent，不能据此宣称内容可核验。
- 实际反例：通过正式 `generate_api` 在隔离工程/隔离 workspace 生成收据，断言 recorded lock 为 absent；写入版本 0.1.0 的 lock 后调用实际 `backend_generation_is_fresh` 为 true；再把 lock 中依赖版本改成 0.2.0，同一门仍为 true。没有修改 API 源/stdlib/producer，也没有手写生成产物或绕过复用门。
- 探针命令：`python docs/plans/reports/738-review-r5-probe.py`。脚本仅临时注入一个审查测试，执行 `cargo test -p auto-man --lib review738_r5_lock_drift_after_first_materialization -- --test-threads=1 --nocapture`，最后在 finally 中按原始 bytes 恢复 Rust 文件。真实输出：

```text
R5_PROBE recorded_lock="absent" first_materialization_fresh=true later_lock_drift_fresh=true
later lock dependency drift must reject old receipt
test result: FAILED. 0 passed; 1 failed; 360 filtered out; finished in 0.91s
```

- 修复：首次成功构建/物化后绑定实际依赖身份（lock 与实际 features/选中依赖），后续严格比较；缺失/不可读不得永久放行。补“生成时 absent → 首次成功绑定 → 后续变更/删除/读取失败”的状态序列反例，并重跑正式生成/启动链。不能用只证明 absent 可以匹配任意字符串的真值表代替状态机验收。
- 映射：AC-06/07/08；T-06/12/13/14；SD-01/05。SD-05 的“lock 出现/变化即陈旧”与最终代码也未对齐，应描述经验证的一次绑定行为。

### P738-R5-02 / P1：公开 JsonValue receiver 方法仍绕过核心 strict 校验

- 触面：`crates/auto-lang/src/trans/rust.rs:5561-5586`（`collect_core_reference`），`stdlib/auto/json.at` 的 `JsonValue.len`；`738-phase3-reference-audit.md` 发射臂 #11 与 `738-sd-drafts.md` SD-04。
- 根因：Dot 调用对象为局部变量时，collector 直接返回成功，不按真实 receiver 类型/公共符号身份收集方法证明。公开 `JsonValue.*` 方法是六核心公共分母的一部分；报告把它们归 D3b 接收者分型面，与 §5.8.2/§6.4/T-11 的方法调用闭包要求冲突。KNOWN-DEBT 中 R1 已明确：P738-D1/D2 六核心部分不能仅登记为 D3b 后勾全。
- 实际 CLI fixture：

```auto
use auto.json
fn main() {
    let v = json.parse("{}")
    let n = v.len()
    print(n)
}
```

- 在独立临时 stdlib 根复制公共 `json.at`，先检查原签名，再只将 `pub fn JsonValue.len(self JsonValue) int;` 改成 `... str;`，实际 Rust producer 不变。两轮均运行 `target/debug/auto.exe stdlib inspect --actual <fixture> --target rust --format json --check`，均 **exit 0 / status=pass / violations=[]**。manifest 只有 `json.parse` proof，没有 `JsonValue.len`。
- CLI 文件 SHA256=`7c5e5e59fa57a4ea1546deb9cf8d533a1d2262cfb683ff3091093180cafb7ff0`。其 manifest 内嵌的 host.rs 与 trans/rust.rs provider FNV 与当前 reviewed_commit 的实际 bytes 一致（分别 `6129932843287186388` / `15583843421651482409`），不是使用旧 collector 产生的反例。
- 正向控制：`json.is_valid("{}")` 的已有 public int / producer bool 漂移仍 exit 1、SIGNATURE_DRIFT，普通模块调用门有工作；本反例定位在 receiver 路径。额外 `use auto.json as j` 探针在本树发生语法错误，不计为别名 strict 覆盖证据。
- 修复：由 resolver/类型/lowering 身份确定 receiver 的公共方法与实际 producer/adapter，并验证参数/返回/mode/绑定；对未经证明的被引用核心方法拒绝。不能因为缺元数据把已有能力一概改 Unsupported，也不能仅写债务保留放行。至少补同一 receiver 调用的公共/producer 漂移反例、同名用户方法隔离与最终 manifest 的方法 proof。
- 映射：AC-03/04/05/07/08；T-03/04/11/13/14；SD-01/03/04/07。

### P738-R5-03 / P2：三角验收未调用真实生成消费者，也未覆盖要求的 VM/C 入口

- 触面：`crates/auto-lang/src/tests/plan738_stdlib_assembly_tests.rs:1640-1702`，T-12 完成声明与 `738-phase3-manifest.md`。
- `triangle_consumers_share_assembly_identity_not_receipt_identity` 的 CLI 分支是库调用序镜像；编译分支用 `let _ = trans_rust_with_session(...)` 丢弃错误；“生成消费者”只是 `cli_manifest.clone().with_consumer_input(...)`，consumer 仍是 stdlib-inspect，没有调用 `generated_api_assembly`/正式 generate_api，也未读真实 generation.json/ready 的共同身份。
- 因此现有测试证明了中立投影/业务字段的局部性质，不能证明生成器、启动门、普通/persistent VM 与 C 最终快照都使用相同实际闭包。两个 t12 测试仅实例化 Rust，另外的 target 变化反例使用 VM 空 references 快照；报告把它们升级为完整消费者验收不成立。服务正例验证 ready 收据身份一致与 stdlib-only 失效，可以保留，但未补齐本项共同身份对拍。
- 修复：为同 fixture/同目标及同实际 provider 选择分别取得真实 CLI、会话/最终产物、生成收据/ready；覆盖普通/persistent VM、Rust、C、生成 API 各适用入口并断言完整共同身份/来源/证明。不同运行形态选择不同 producer 时记录合理差异，不能删除依赖/proof 或换成 clone 得绿。所有转译/生成 Result 必须断言成功，变更 producer/features/依赖的反例也要从真实消费者取证。
- 映射：AC-02/06/07/08；T-02/03/05/06/12/13/14；SD-01/04/05/06。

## 3. 门禁与证据复用

本轮工作树没有最终实现 diff；临时 Rust 审查探针恢复 exact bytes 后，执行下表门禁。日志名及 SHA256 见 baseline JSON，以下摘要为可在清理 worktree 后保留的证据。

| 检查 | 本轮结果 |
|---|---|
| `cargo t plan738 --no-fail-fast` | 69/69，76.729s；既有测试没有覆盖 R5 新反例。 |
| `cargo t --no-fail-fast` | 5183 run / 5167 pass / 16 fail，104.909s；13 已档 master 预存 + plan502/plan707_client/plan606_029 已档族；逐名名单见 JSON。 |
| `cargo tv` | 162/162，2.253s。 |
| `cargo tt --no-fail-fast` | 5554 run / 5540 pass / 14 fail，139.521s；13 与日常档红名单相同，额外 plan484_024 为 R3 已档 flake。 |
| `cargo tt plan484_024` | 隔离复跑 2/2，0.332s。 |
| 三 crate check | 结果与日志见 baseline JSON；不混入 auto-man 已档集成 fixture 目标。 |
| 本轮 8 个 Phase 3 触面文件 rustfmt + diff check | rustfmt exit 0，`git diff --check 2c1b4a763..HEAD` exit 0。 |
| lock 真实生成/复用门反例 | 期望拒绝，实际通过新鲜度门；负面审查断言失败，明确必修，非环境红。 |
| receiver 实际 CLI 反例 | 原始/返回类型漂移均 exit 0，方法 proof 缺失。 |

原 R3 `b3a4d660e` 的 HTTP/正式服务/三目标与 C stdio witness、原 R4 `9c255993b` 的 freshness 两独立测试和 api_gen 收据作为**识别过的既有正例证据**保留：R4 与当前代码/依赖相同，生产 diff 没有变化，未通过重复服务正例代替本轮缺失反例。此次未重跑 th 的环境 timeout 族或正式服务正例，也不把已有正例外推为 R5 两个反例通过。后续代码修复必须在新提交重跑受影响门禁及真实服务验收。tf/taa/tu 不触发。

## 4. AC / SD 判定

| AC | 判定 | 依据 / 关联任务 |
|---|---|---|
| AC-01 | pass（现有清点面） | full-scan/矩阵/分母族本轮随 plan738 + t 重跑；保留 R3/R4 AST/inventory 审阅证据，清点不等于引用方法已证明。T-01/02/09。 |
| AC-02 | partial | 选层/同源 witness 正例保留；真实消费者共同身份覆盖不足，R5-03。T-03/12。 |
| AC-03 | fail | Rust 包装契约门已修；公开 receiver 方法绕过，R5-02。T-04/10/11。 |
| AC-04 | partial | 既有源映射/use 隔离族持续绿；receiver 身份闭包遗漏，同名用户 receiver 反例待补。T-03/11。 |
| AC-05 | partial | target/environment 正拒绝与矩阵证据保留；公开核心方法未证明仍放行，不能声称核心门全闭合。T-04/11。 |
| AC-06 | fail | lock 后续漂移反例 R5-01 + 多消费者反例/覆盖 R5-03。T-05/06/12。 |
| AC-07 | fail | 真实服务正例有效；CLI receiver 检查假绿、生成复用门未检测 lock 漂移。T-06/11/12/14。 |
| AC-08 | fail | SD-04 接收者债务豁免、SD-05 lock 行为、T-12 完成声明/验收对账不符合当前合同；完成原 SD 修正后再审。T-13/14。 |

SD-01..07 影响路径/GOAL-003 保留；本轮不改 canonical/ledger，也不弱化原规范。SD-01/04/05/06 的完整 strict/消费者/失效陈述须按上述修复重锚；SD-07 不得将已约定六核心的未验证调用改为批准 D3b 延后。R3-02 计数勘误在 `738-phase3-acceptance.md` 的“17 条”仍未同步，随 T-13 一并纠正（当前 ADAPTER_RULES=15）。

## 5. 返回 work

1. 在原 738 worktree 处理 R5-01/02，提交代码及能捕获本轮反例的回归保护。
2. 补真实消费者与对应变更反例（R5-03），对齐实际运行形态/依赖身份，不只增加 fingerprint getter 单测。
3. 更新 SD 终稿、AC/任务对账与最终提交收据；旧 R3/R4/R5 证据保留历史，不混版本称通过。
4. 新最终提交上完成受影响门禁与正式服务验收，交独立上下文复审；本次不合入、不归档、不清理 worktree。

任务簿记：原 T-02..T-08 在主计划仍为未勾选，14/14 与实际标记不一致。本轮保留 T-01/T-09/T-10 完成，重开 T-11..T-14，current_step 按实际完整完成的任务计为 **3/14**；这不是删除其他任务已实施的代码或历史正例。
