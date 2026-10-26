# PLAN-850 r1 work handoff

Authoritative repository/worktree: `D:/autostack/.wt/lang-850/auto-lang`,
`plan-850-dev`, base `1c453e8166e574d9e2eae023eed1758b3f02f3e9`, frozen
`1db29917b7f7b26925cc1432fb3a74a0f4c01129`. Git clean. UI read-only sibling
`aee499e87093ff727f68ac8467c506e9ecf6ecdc`; auto-down read-only sibling
`c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`. No production Rust changed.

## Work-owned acceptance

| Invariant | Executed evidence | Result |
|---|---|---|
| Original external-package failure is distinguishable | original-external-member.log.txt, original full selector under the current layout | Cargo rejects external dev-dependencies; old command cannot execute tests |
| Language and UI identity conservation | lang-full-list.json.txt / ui-full-list.json.txt against committed ID inputs | 3718 language IDs + 3091 unique UI IDs, zero missing or added |
| All formal batch legs retain their own profile/feature surface | frozen-full-list/receipt.json.txt and four real list/build logs | language 3218 selected; UI 3073; tt 3553; tb 3109; pass |
| Other aliases no longer carry the same external-package defect | taa-list, ta-list, t3-list receipt/list/build logs | all actual two-owner listings pass; original full/t3 configurations unchanged |
| No zero-selection or dropped-owner green | controls.log.txt | permanent counterexamples reject missing IDs, filtered required IDs, empty full/scoped selection and missing roots |
| Observable child success/failure/lifetime | controls.log.txt | real exit0/7, all-four success, first-red stops queue, timeout, interruption and actual grandchild termination pass |
| Correct roots and frozen main input | controls.log.txt plus actual bidirectional cargo metadata check | wrong UI and reverse SDK owner rejected; worktree/full and dirty-main controls reject |
| Machine exclusion and WIP preservation | controls.log.txt | independent process cannot acquire held gate; report bytes restored after failure, locked original fails before tests |
| Original batch-red diagnosis | t01-six-replay-r2.log.txt, signature-isolated-r3.log.txt | 5/6 passed, one distinct P95 red; only that test replayed after external build load ended: 1/1 pass, P95 786.2717ms |

Final Python controls: **14/14 pass**. Rust launcher formatting and Git diff
checks pass. The cold unprepared worker timeout in t01-six-replay.log.txt and
the 1165.1593ms P95 failure remain raw diagnostic evidence. Thresholds, sample
counts and deadlines were not changed. No aggregate tier ran during work.

## Evidence reuse boundary

The real full-batch selection was captured on `de932d682`, the three additional
alias selections on `88875b3e7`. Compared with frozen HEAD, selector options,
feature sets, baseline ID/required sets, Rust launcher, production sources and
dependencies are unchanged. Later guards check both SDK owner directions,
broaden the identity diff, reject dirty/non-main full execution and strengthen
negative controls. Those guards were verified directly; their changes do not
alter the recorded selections. Temporary full/t3 group additions were reverted
exactly: MN probes are test-cases-gated and owned by tt, whose actual group
membership is recorded in tt-mn-groups.log.txt.

## Remaining owners

Work-owned findings, required coverage gaps and contract decisions: **none**.
Independent final review T04 is pending; the implementation session has not
claimed independence or a final pass. The explicit subagent authorization
question is pending. Implementation writes are paused on the frozen tree.

After final review, T05 owns the actual main-checkout single-instance full
batch (`cargo tf --batch`), all original child gates, receipt/green-anchor
admission, canonical deposit, ledger, archival and guarded cleanup. This
script-only change does not justify an extra bare cargo t/docs_gen campaign.
No landing, full-green claim, ledger write, archive or cleanup has occurred.
