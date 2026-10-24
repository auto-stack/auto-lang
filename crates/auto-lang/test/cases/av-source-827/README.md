# PLAN-827 av-source-827 corpus — 真实源码程序经正式 recipe 的验收基线

套件经统一 runner（TestCase v2）probe/run 接缝由既有 recipe **`ac-av/core-run-v1`**
执行（T-01 复用裁定：不新增 recipe/worker/workspace，见
`docs/plans/attachments/827/contract-decision.md`）。worker `av_probe_worker case-serve`
只接收声明式参数与源码字节，不读取 expected；判定全在父侧。

## 源码身份（inputs 副本对账回执）

| case 源 | 原件（只读） | SHA256（字节恒等，T-01 冻结） |
|---|---|---|
| `src/p-gcd-euclid.at` (AP-01) | `experimental/ac-core/fixtures/scalar-source/valid/p-gcd-euclid.at` | `6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3` |
| `src/p-binop-args.at` (AP-22, T-04) | `experimental/ac-core/fixtures/scalar-source/valid/p-binop-args.at` | `46d6a431a5c1b9bbe6e2e87b8732c09a9c584b58eb92799fdb4ab073f423e625` |
| `src/p-short-circuit-trap.at` (AP-23, T-04) | `experimental/ac-core/fixtures/scalar-source/valid/p-short-circuit-trap.at` | `d6871b77c9b02e797c77a2cf34900e005fe97062bf1fc95ffcdf1cf7707052a9` |

副本与原件字节恒等由独立 workspace 测试 `av_source_827_seeds_identity` 双向核验
（原件 SHA 对本表 + 副本 SHA 对原件）。变体/负控只在 `experimental/ac-av-probe/probe/
tests/fixtures_827/` 或 case 声明内，**不回写 AP 原目录**。

## oracle 形态（手审冻结，见 contract-decision §3）

- AP-01/AP-22：`success` + `exit 0`（源内 gcd/算术组=手审独立 oracle）。
- AP-23：`error-run` + trap `av.trap.div-by-zero`（false && 的 RHS 必不执行；
  true && 的 RHS 必执行并除零 trap——任意 Err/超时不充当负例）。
