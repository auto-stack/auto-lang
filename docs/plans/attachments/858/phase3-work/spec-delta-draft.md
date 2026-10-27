# PLAN-858 r3 Phase 3 — frozen baseline and policy delta draft

The following is a review draft, not an applied canonical change. User ruling: development-stage ACE turnaround gates use nearest-rank P90 <= 2000 ms; 10 valid measurements allow at most one above 2000 ms. Keep existing formal sample counts, actual timing boundaries, results, structure, resources, failures, and raw observations. Preserve historical P95/P99 measurements as measurements, not new-policy evidence. Later architecture-wide measurement will establish final targets.

| Delta | Target | Before → proposed after |
|---|---|---|
| SD-02 | `docs/specs/auto-ac/mir-native-cache.md` | Parent-observed MIR-native edit/body/signature result P95<1s → P90≤2s; keep full preparation/IPC/link/run/admission/comparison and identity/structure. |
| SD-02 | `docs/specs/auto-av/cache-and-conformance.md`; `docs/specs/auto-av/resident-function-install.md` | Formal persistent and parent-verified AV gates P95<1s → P90≤2s with ≥30 raw revisions, exact changed-function installation and all independently checked results. Treat earlier 798 r7 measurement as historical, not present-day 1s mandate. |
| SD-03 | `docs/specs/auto-cte/frozen-cache.md` | Formal release body/meta/resident edits P95<1s → P90≤2s; preserve 115-function ≥256KiB load, hash identity, correct results, resource and no-full-rewarm gates. |
| SD-03 | `docs/specs/auto-semantics/query-cache.md`; `docs/specs/auto-test-case/profile-v2-native.md` | 1MiB body edit at producer and IPC parent path P95<1s → P90≤2s; preserve prep/worker/admit windows, per-sample status and tiling. |
| SD-03 | `docs/specs/auto-abi/conformance-cache.md` | Body-edit and descriptor-reject P95<1s → P90≤2s, preserve six scenarios ×30 and C/worker parent paths and resource gates. |
| SD-04 | `docs/specs/auto-test-case/architecture.md`; `docs/specs/auto-lang/project.md` | Change current dual time gate to P90≤gate and current 796 gate to P90≤2s; preserve test IDs (including historical names), full batch selection, P95 diagnostic history and window/resource isolation. |

Statistics: for valid N windows sorted ascending, choose one-based rank `N-floor(N/10)`; equivalent to at most floor(N/10) above 2000ms. 9/1 at N=10 and 27/3 at N=30 pass, 8/2 and 26/4 fail; equality passes. Do not drop failures or substitute synthetic runs for formal evidence.

Historical module plans.md summaries retain the result recorded at the time, with a current-policy cross-reference added during merge rather than rewriting measured history. The design files listed below contain prior 1s objectives; their current policy sections need explicit supersession notices, preserving historical rationale. Six main-checkout design files already have unrelated uncommitted edits; merge must reconcile without overwriting or committing them.

## Targets and input hashes

| Path | SHA256 of source working-tree bytes at draft |
|---|---|
| `docs/specs/auto-ac/mir-native-cache.md` | `c8c3912c1397b4bb93f04c348eb5e2be9c5357d8285036c5f9669138f145ac0f` |
| `docs/specs/auto-av/cache-and-conformance.md` | `f1e3d3d3010bca1930119d28c5234918dab14cfc1076789c8724acf5d0222575` |
| `docs/specs/auto-av/resident-function-install.md` | `c23bf7cfd62dd720388b3e2d34fd5c6d392c6cbdac1e10dfaeb5a21dc25ec88f` |
| `docs/specs/auto-cte/frozen-cache.md` | `886521fa6f9132c8bfc51beb4cbd24920e7f52f5842d665f81adbe7ce48b3074` |
| `docs/specs/auto-semantics/query-cache.md` | `6ac0326a5486043027e105641131b00029f81c673189c4a968a3ccce258d2b20` |
| `docs/specs/auto-abi/conformance-cache.md` | `e4b5aeac7eb2be2cb464fb98740504750d93219c9abcace0c51aa6ea546f91d8` |
| `docs/specs/auto-test-case/profile-v2-native.md` | `e8c1e491c7a424d6b4cc236613a1b91ab60e05b597cda294c1437470a31e73fc` |
| `docs/specs/auto-test-case/architecture.md` | `ab061aaa2e87dc5a47156192f2f1922f8c473311389352f3668bd7b255581bcc` |
| `docs/specs/auto-lang/project.md` | `839eedb556f84c776177b411aba479c7c964ce7e2a0ae2cb95086d056bb92eca` |
| `docs/design/35-autodown-test-cases.md` | `ec5a28e8d9591c09b7a14a60ec492bb235539c7fc8480a2f76b64eef2555a9da` |
| `docs/design/39-ac-incremental-syntax-and-language-platform.md` | `fc2e36e5f5d1315ed37035a7aa35c322bd717852eb9c9e33105c30c2f91cd340` |
| `docs/design/40-compiler-artifacts-and-cache-lifecycle.md` | `2b70ae539e3527d59ebceb74277353da07b7f339d0f244cda837c2b988a9f378` |
| `docs/design/43-ac-av-typed-mir-and-abc.md` | `9b252bd33da4a76ea2534f0404a99af8514b94bfc5ece53ab57a320c3ea9d941` |
| `docs/design/44-av-data-layout-place-and-ownership.md` | `1b462ce63cf22ed4692270ea6d7943caf3d07e571b07aa9871109cb86c84c3d5` |
| `docs/design/strategy/auto-native-backend-evolution.md` | `228d36c706f2b7f9ab940ecd770e6719d0faace8844c8ece2ac22768b1a9d60a` |
| `docs/design/strategy/av-source-programs-and-avm-replacement-roadmap.md` | `357dbdd1ae930987263ae0105838251e0a0aa28e73b3f9ef59ffd75855e8a968` |
