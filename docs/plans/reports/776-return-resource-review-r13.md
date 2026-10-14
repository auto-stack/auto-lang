# PLAN-776 最终独立复审 R13（2026-10-14）

**outcome: needs_fix。** F776-R11-01 已关闭：五 setter 的 RC1/RC2/错误 receiver 共15格均实际执行，剩余8格与共享最终回收断言已补齐，定向测试通过。仅剩本提交新增的未处理编译警告 F776-R13-01（P3）；生产修复与矩阵验收通过结论保留。

## 基线与复用范围

- stage: review；review_scope: final；plan_id: PLAN-776；plan_revision: 1。
- reviewer_identity/limitations: 当前 Codex 延续 R11/R12 独立审查上下文，未实施修复，未委派子agent，不声称额外模型独立性。
- reviewed_commit: `367f4b288b515d49938a0b5f3bf47cf9de6f9b33`；repair_diff_base: `d0fa9e5e2d0b50924698a01c591541f6e8744c8c`；base_commit: `5e21561d2c303a02319ff7a29e4607933e09a38f`。
- worktree_path: `D:/autostack/.wt/lang-776/auto-lang`；branch: `plan-776-dev`；实施树 clean。
- dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- spec_inputs: [input-hashes.json](../attachments/776/review-r13/input-hashes.json)；七个 canonical Spec、decision、SD 与 R12 相同。冻结 [reviewed-spec-delta.md](../attachments/776/review-r13/reviewed-spec-delta.md)，SHA256=`D5730778FA8EE21A005D6F934DBCD7C1BAFD49ED3B75EAA7B7548CC5E57D9CDB`。
- 本提交仅测试文件 +200，生产代码/依赖/测试配置/R11探针输入未变。复用 R11 原UAF重放、生产15格、66专项、126协议守卫及 R12 未变共享mutation证据；重新审查和运行当前变化的测试。生成证据仍仅既定 File/Upload/scaffold JSON/bus SSE 支持面，742与P776-D3/D4边界保持。

## 原 finding 闭合

新增 header/text/html/bytes 各两格，均使用有效参数并实际调用 shim：独占调用后 `heap_rc==Some(1)`，释放返回值/reap 后堆条目不存在、资源存活量回基线；错误 receiver 使用 ListData 非凭据对象，实际 Err 且外部份额保持 Some(1)。共享 settle 最后释放/reap 后现已断言 heap条目缺失和 live回基线。status已有矩阵及重复链保留。F776-R11-01 不再未完成，不要求新增运行路径或扩大测试规模。

## F776-R13-01 / P3：新辅助函数的 expect 未使用

位置 `crates/auto-lang/src/tests/plan776_http_return_tests.rs:2962`：`exclusive(..., expect: res::ResourceClaim)` 接收四组预期快照，但函数从未读取 expect。实际编译输出：

```text
warning: unused variable: `expect`
--> crates/auto-lang/src/tests/plan776_http_return_tests.rs:2962:69
```

此警告来自本次 +200 增量，不能归入既有仓库警告。违反仓库 Mandatory Independent Review Gate 的 Health Check（零未处理编译警告），影响 T08/AC09；当前独占测试只断言RC/回收，未使用其传入的mutation预期。本轮不把已有共享mutation断言或生产行为判失败。

建议在释放返回值之前，读取仍有效的返回凭据快照并与 expect 比较，复用 shared settle 的字段比对方法；此时不是读取回收后的tombstone，不会触发canary。若仅保留RC/回收oracle，也须移除多余参数与预期构造并明确覆盖范围。不能仅抑制警告后继续把未使用期望宣称为有效断言。

## actual 验证与结论

- `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776_r10_setter_single_release_shared -j1 --no-fail-fast`：**1/1**，0.130s，0timeout；[原始日志](../attachments/776/review-r13/setter-test.log) 含新增expect警告。
- 新测试文件 rustfmt --check（edition2021、skip_children=true）、`git diff d0fa9e5e2..HEAD --check`：通过。
- AC01..08 按 R11/R12 已注明的代表覆盖和支持面 pass；AC02/06 仓内矩阵缺口关闭。AC09因新增未处理警告 fail。SD01..07与生产行为一致，spec-impact路径及GOAL-003/016保持；不发布canonical/ledger。
- 仅重开T08，T01..07保持完成，current_step=7/8，status=executing，plan_revision=1。next: work，同树处理expect、提交后定向复审；生产修复与15格矩阵不用重做。
- 本轮未跑整档t/tv/th/tt/tu/taa/tf。final pass后merge-owned裸t/tv/HTTP及相关生成验证、主检出到期batch tf、Spec沉淀/归档/wt-guard仍pending；work提前整档结果不替代后置门禁。未修改实施代码、合入、归档或清树。
