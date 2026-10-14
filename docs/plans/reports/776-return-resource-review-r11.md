# PLAN-776 最终独立复审 R11（2026-10-14）

**outcome: needs_fix。** F776-R10-01 的生产修复已验证闭合，未发现新的运行时缺陷；仍缺提交内的五 setter 回归矩阵。新测试只运行 status，其余四项函数指针从未调用，不能按“五 setter 全覆盖”验收。需补真实参数、实际调用和断言后提交，复审范围可限定于该测试与证据修订。

## 1. 基线与证据

- stage: review；review_scope: final；plan_id: PLAN-776；plan_revision: 1。
- reviewer_identity/limitations: 当前 Codex 审查上下文，根据提交、源码、原反例及独立探针重建结论；本上下文未实施修复，未委派子 agent，不声称额外模型独立性或延续旧审查会话。
- reviewed_commit: `38ad38abaf7d0fc40adacbb775d3f0258aaa53a7`；repair_diff_base: `dd41d684dc2d8c1321034ed56d4a8cb38eab2894`；base_commit: `5e21561d2c303a02319ff7a29e4607933e09a38f`。
- worktree_path: `D:/autostack/.wt/lang-776/auto-lang`；branch: `plan-776-dev`；实施树 clean。dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- spec_inputs: [input-hashes.json](../attachments/776/review-r11/input-hashes.json)，七个既有 canonical Spec 与 R10 输入相同；新 Spec 尚未发布。SD 冻结副本 [reviewed-spec-delta.md](../attachments/776/review-r11/reviewed-spec-delta.md)，SHA256=`D5730778FA8EE21A005D6F934DBCD7C1BAFD49ED3B75EAA7B7548CC5E57D9CDB`。
- 生产库重新 build、独立链接实际 shim，库/HEAD/hash 见 [baseline.txt](../attachments/776/review-r11/baseline.txt)；输入及可重放入口 [probe.rs](../attachments/776/review-r11/probe.rs)、[reproduce.ps1](../attachments/776/review-r11/reproduce.ps1)。探针不是替代生产实现的 mock。

## 2. F776-R10-01：实现与原反例已闭合

五 setter 源码均撤显式 `rc_release(res_nv)`；`StakeGuard` 不 retain，其 Drop 恰一次消费，成功路径 `rc_push` 重建返回份额；mutation 失败时 guard Drop 同样消费一份。receiver 使用清槽的 `pop_arg_nv`。新增代码未改变返回资源表示、Future 值桥、类型分类或生成轨。

原 V10-R1 真实重放：shared Response RC before=2，status 返回后=2，释放返回槽/reap 后=1，global 存活，快照 status=201；旧 UAF 消失。

独立新增 `setter-matrix` 使用合法 String 池参数与实际 ListData<i32> [65,66] 参数，对 **status/header/text/html/bytes 各三格，共15格** 实际调用：

| 场景 | 断言与 actual |
|---|---|
| RC1 独占 | setter 返回 RC1、状态/header/body 实际修改；释放返回槽后 RC0，reap 回收 |
| RC2 共享 | setter 返回 RC2、修改正确；释放返回槽后 RC1，reap 后全局快照仍可读；最后释放全局后回收 |
| 非凭据 receiver + 外部份额 | 返回 Err，参数一份被消费，外部 RC1 存活；最后释放后回收 |

15/15 带 assertion 通过，结果 [setter-matrix.log](../attachments/776/review-r11/setter-matrix.log)。另重放 shared iterator，消费 RC2→1 且 reap 后凭据有效；Future typed record/字段修改/object literal/array/closure/helper 均 JSON42，普通 496 位碰撞及 false 分支恒 JSON496；SSE direct/local/helper/int helper/Internal 六形态均实际收到 data42。此证据关闭生产缺陷，不证明仓内测试已覆盖它们。

## 3. F776-R11-01 / P2：提交内五 setter 矩阵未实际执行

影响 T-07/T-08、AC-02/AC-06 的回归证据和 AC-09 对账。这是 R10 已要求的 oracle 遗漏，不新增功能范围。

位置 `crates/auto-lang/src/tests/plan776_http_return_tests.rs:2687`、`:2700`、`:2839`：

```rust
let setter = vec![ /* 五个函数指针，其中四项是占位参数 */ ];
// 共享、独占、重复调用、错误早退各段都直接调用 shim_http_response_status。
let _ = setter; // 没有遍历或调用另外四项
```

header 的闭包只压一个 i32（实际需要 key/value 两个字符串），bytes 压 i32(0)（实际需要有效 ListData<i32>）；它们是未完成的占位，不能直接遍历现有 vec 就声称修好。测试注释“五 setter 全覆盖”、提交说明及 Plan 交接“RC2 共享矩阵”的范围必须和 actual 对齐。×5 链也是五次 status，而非五种 setter 混合链。

修复：用合法参数逐项实际执行五 setter 的 RC1/RC2/错误 receiver 矩阵，断言每项真实 mutation、返回份额、丢弃返回值后的共享存活和最后释放回收；保留或增加实际混合 setter 链验证。去掉未使用的占位表，更正文档覆盖范围。可参考本轮独立探针输入；应落在原实施树的正常测试中并提交。只需定向测试及必要 check，不应再跑整档 t/tv/th。

## 4. 验收与规范审查

| AC | verdict | 对应任务/证据 |
|---|---|---|
| AC-01 | pass | T02/03/07；碰撞专项、普通值负控制 |
| AC-02 | partial | T02 生产修复 pass；T07 五 setter 仓内回归矩阵缺失 |
| AC-03 | pass（既定代表覆盖） | T03/04；66专项与独立 Future/SSE 重放；未宣称全笛卡尔组合 |
| AC-04 | pass（既定代表覆盖） | T04；Failed/Nil/缺结果/取消既有专项，705 定向验收；P776-D3 边界保持 |
| AC-05 | pass | T02/04；owner/过期/耗尽与普通数字负控制，shared iterator 重放 |
| AC-06 | partial | T05 生产修复和1000闭环 pass；T07 仓内共享 setter 矩阵缺失 |
| AC-07 | pass（当前边界） | T06；来源与755/738协议守卫、SD05 setter消费源码/探针 |
| AC-08 | pass（已冻结支持面） | T06；生成/分类代码与依赖未改，沿用R10已标明的R7/R8生成证据，仅File/Upload/scaffold JSON/bus SSE，不冒称742真实业务体已交付 |
| AC-09 | fail | T08；测试实际调用和覆盖声明不一致 |

SD-01..07 的持续合同和 spec-impact 路径/GOAL-003、GOAL-016保持；本轮 setter 恰一份消费条款与生产源码一致。规范草稿冻结保留，证据范围需随 F776-R11-01 更正。历史修订链不是对旧协议的重新授权；merge 沉淀应仅发布最终规则。review 不发布 canonical/ledger。

重开 T-07/T-08，保留 T-01..T-06 与已通过子项，current_step=6/8；status=executing，plan_revision=1（不改变语义合同）。F776-R10-01 运行时缺陷关闭，F776-R11-01 不可用债务登记授权延后。

## 5. actual 验证与后置门禁

- `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 -j1 --no-fail-fast`：66/66，30.986s，0 timeout；新 test pass 不等于其五项表已运行。
- `cargo nextest run -p auto-lang --lib --features test-http-e2e -E 'test(plan755) or test(plan738) or test(plan734) or test(plan705) or test(plan394)' -j1 --no-fail-fast`：126/126，156.613s，1 slow，0 timeout；原始结果见 [compat-tests.log](../attachments/776/review-r11/compat-tests.log)。
- 生产 lib build 成功；独立探针原例及15格矩阵通过。最初探针误引用私有 convert module 的编译错误已改为公开 VMConvertible re-export 后重编，未修改生产代码。
- 本轮两份 Rust 文件 rustfmt --check（edition2021，skip_children=true）、`git diff dd41d684d..HEAD --check` 通过。编译警告为既有仓库警告；本轮两文件增量未新增警告，不声称全仓零警告。
- 本轮未跑整档 t/tv/th/tt/tu/taa/tf。R10 work 交接此前运行的 t/tv 结果不代替 final pass 后 merge-owned 门禁。后置适用门禁：裸 t、VM触面 tv、HTTP th 与 auto-man 相关验证；tf主检出batch-only。Spec/ledger沉淀、归档、wt-guard清树仍 pending。

next: work，在同一lang-776树完成 F776-R11-01 测试与证据修订、提交，然后针对该差异独立复审。无实施修复、合入、归档或清树动作。
