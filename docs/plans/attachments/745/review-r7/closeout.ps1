$ErrorActionPreference = 'Stop'
$root = 'D:/autostack/auto-lang'
$dir = Join-Path $root 'docs/plans/attachments/745/review-r7'
$planPath = Join-Path $root 'docs/plans/745-ac-source-checked-hir-native.md'
$plan = Get-Content -LiteralPath $planPath -Raw
if ($plan -notmatch 'status: execution_done' -or $plan -notmatch 'plan_revision: 1') { throw 'Unexpected Plan state' }
$plan = $plan.Replace('status: execution_done', 'status: reviewed').Replace('updated_at: 2026-10-07', 'updated_at: 2026-10-08').Replace('current_step: 7', 'current_step: 8')
foreach ($id in @('T-02', 'T-05', 'T-06', 'T-07', 'T-08')) {
    $plan = $plan.Replace("| $id | [ ] |", "| $id | [x] |")
}
$oldIntro = '唯一任务表。2026-10-07 第六轮最终复审 needs_fix 后，同日 r6 修复轮完成：T-02/T-05/T-06/T-07 以 r6 证据重勾（commit bb8cb4b43）；T-01/T-03/T-04 保留完成；T-08/09待完成，execution_done 7/9。历轮work收据保留为历史，不能覆盖第六轮发现与 r6 处置。'
$newIntro = '唯一任务表。2026-10-08 第七轮最终独立复审 pass（bb8cb4b4330d505059ea20e7bfe561a93c1051f3）：T-02/T-05/T-06/T-07 按本轮实际证据恢复完成，T-08 完成；T-01/T-03/T-04 保留完成，T-09 待合入授权与实际执行，reviewed 8/9。入口收据/frontmatter 称7/9而四个修复项仍空勾，本轮统一计数；历轮work/review记录保留。'
if (!$plan.Contains($oldIntro)) { throw 'Task introduction changed' }
$plan = $plan.Replace($oldIntro, $newIntro)
$receipt = @'
2026-10-08 stage=review，PLAN-745 r1 第七轮最终独立复审：**pass**。

- `review_scope`: final；`reviewer_identity`: 原规划及历轮复审Codex上下文，独立于GLM实施/修复上下文；未虚构模型/session，未另建树/委派/代做代码修复。
- `plan_id/plan_revision`: PLAN-745 / 1；`outcome`: pass；`worktree_path/branch`: D:/autostack/.wt/lang-745/auto-lang / plan-745-dev。
- `reviewed_commit`: bb8cb4b4330d505059ea20e7bfe561a93c1051f3；`previous_reviewed_commit`: 522b98a4c27d2064049e188d386f6958721b3a90；`base_commit`: eea7a80618d4d5e75749d666b7609a9a87925bbd；入口主检出v0.6-dev@05d2e1c8464f6d0b2366a040dbd99e29d7eac666。
- `dependency_revisions`: 组内只读auto-down@fba6563ed2148ce85e68208863159b4ccccac710；实现/依赖终审clean且HEAD稳定。
- `spec_inputs`: 四份当前Specs与四份验证配置的路径/SHA见[第七轮baseline](attachments/745/review-r7/baseline.json)；[final-baseline](attachments/745/review-r7/final-baseline.json)确认无漂移；冻结SD与当前提案一致，SHA256 cd027eae6395f8903f41f1623c20af49d67ec5fdf09631b30f52b8f5485bcdb2。
- `acceptance_results`: AC01..08全部pass，AC09 pending；ac-core114 passed/1 ignored；741与745各13步，745原74例及独立74/74；新增块矩阵N01..24全过；strict47 CHECK-OK。root check过（382历史warning）；t=4942/4972、30失败均在历史40红名中，新增名0；tv=158/162、4历史semver失败。未称根档全绿或严格同源码main对照，生产crates/Cargo/lock/.cargo/.config相对原基线无diff，归因见daily-attribution。
- `finding_disposition`: P745-R6-QA-01清偿（M01/M02=42、M06=192、M07=5、M18=23/ba、H09=111）；P745-R6-QA-02清偿（未闭合return/let/if/while准确text拒绝，M21 check1/build1且三件SENTINEL SHA全同）；P745-R5-QA-02剩余块项清偿（1000if/1500if-else/2048while真实PE，20000if到Checked、深畸形受控）；历轮其它清偿保持。
- `findings`: []（本轮无未清偿合同内发现）；遗漏/延后/workaround扫描及规范增量审定见完整报告，未批准缩域/延期；根历史失败保持公开。
- `pending_closeout_gates`: T09合入授权、v0.6-dev整合、当日verify745重跑、SD/MD追加后独立新鲜度及消费者核验、Specs/ledger/index/导航、批回归到期判断、归档9/9、wt-guard clean后原树/分支/空组清理；均未预勾。
- `evidence`: [第七轮完整报告](attachments/745/review-r7/REVIEW.md)、gates/corpus/parser/parser-extra/malformed-sentinel/block/calls/crash/resource/bundle/helper/csharp/supplemental结果及原始日志；[artifact-manifest](attachments/745/review-r7/artifact-manifest.json)绑定持久输入/结果/收据。
- `state`: reviewed 8/9，revision1；T01..08完成，T09待完成。修复收据称7/9但旧表仍留四项空勾，本轮以实际复验恢复并统一计数；不回写历史pass。
- `next`: 可进入 /auto-plan:merge T09，继续使用原树；本轮未合入、归档、清树或发布canonical/live ledger。

'@
if ($plan.Contains('第七轮最终独立复审：**pass**')) { throw 'Receipt already present' }
$plan = $plan.Replace('## 10. 待澄清事项', $receipt + "`r`n## 10. 待澄清事项")
[IO.File]::WriteAllText($planPath, $plan, [Text.UTF8Encoding]::new($false))
$debtPath = Join-Path $root 'docs/plans/KNOWN-DEBT-AND-RISKS.md'
$debtReceipt = @'

## PLAN-745 r1 第七轮最终独立复审（2026-10-08，清偿）

reviewed_commit=bb8cb4b4330d505059ea20e7bfe561a93c1051f3；**pass**；[第七轮报告](attachments/745/review-r7/REVIEW.md)、[计划](745-ac-source-checked-hir-native.md)。原同树保留，reviewed 8/9；没有合入、归档或canonical/live ledger沉淀。

| 原ID | 处置 | 独立证据 |
|---|---|---|
| P745-R6-QA-01 | 已清偿 | positional/named二元实参42、嵌套192、异构bool5、trace23/ba、H09恢复111；括号内非法逗号仍拒 |
| P745-R6-QA-02 | 已清偿 | return/let/if/while缺右括号均text/source.syntax准确拒绝；实际M21 check1/build1，exe/obj/receipt三件旧SHA逐件相同 |
| P745-R5-QA-02（剩余块项） | 已清偿，原finding整体关闭 | 无固定嵌套计数护栏；1000if/1500if-else/2048while真实PE、20000if到Checked、深未闭合受控；新增N01..24块流/作用域/循环/trace矩阵全过 |

历轮其它清偿保持；本轮无未清偿合同内finding。114 passed/1 ignored、741/745各13步、独立74/74、strict47新鲜；真实Bundle及执行器失败控制保持。root日常30失败全部历史共现、tv4历史semver失败，不称全绿或严格同源码main对照；生产路径无745 diff。冻结delta与终审HEAD/依赖/Specs/配置无漂移，证据见第七轮附件。T09真实closeout与批回归到期判断仍待执行，既有P745-D1中央取号脚本债务不在本计划修复范围。
'@
[IO.File]::AppendAllText($debtPath, $debtReceipt + "`r`n", [Text.UTF8Encoding]::new($false))
$count = [regex]::Matches($plan, '(?m)^\| T-\d\d \| \[x\] \|').Count
if ($count -ne 8 -or $plan -notmatch '\| T-09 \| \[ \] \|') { throw 'Task count mismatch' }
$f = Get-Content -LiteralPath "$dir/final-baseline.json" -Raw | ConvertFrom-Json
$f | Add-Member -NotePropertyName post_review_plan_sha256 -NotePropertyValue (Get-FileHash -Algorithm SHA256 $planPath).Hash.ToLower()
$f | Add-Member -NotePropertyName plan_state -NotePropertyValue 'reviewed'
$f | Add-Member -NotePropertyName completed_tasks -NotePropertyValue 8
$f | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath "$dir/final-baseline.json" -Encoding utf8
# Only ordinary, validated artifact leaves created by this review are removed.
$absoluteDir = [IO.Path]::GetFullPath($dir)
$leaves = @(Get-ChildItem -LiteralPath $dir -Recurse -File | Where-Object { $_.Name -in @('app.exe', 'app.obj', 'app.ac-link.txt') })
foreach ($leaf in $leaves) {
    $q = [IO.Path]::GetFullPath($leaf.FullName)
    if (!$q.StartsWith($absoluteDir + [IO.Path]::DirectorySeparatorChar) -or ($leaf.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Artifact is outside review directory or a reparse point' }
    if ($leaf.Name -eq 'app.ac-link.txt') { Copy-Item -LiteralPath $q -Destination (Join-Path $leaf.DirectoryName 'link-receipt.txt') }
    Remove-Item -LiteralPath $q
}
$parseErrors = @()
foreach ($s in (Get-ChildItem -LiteralPath $dir -Filter '*.ps1')) {
    $tokens = $null; $errors = $null
    $null = [Management.Automation.Language.Parser]::ParseFile($s.FullName, [ref]$tokens, [ref]$errors)
    $parseErrors += $errors
}
if ($parseErrors.Count -ne 0) { throw ($parseErrors | Out-String) }
$manifest = @(Get-ChildItem -LiteralPath $dir -Recurse -File | Where-Object { $_.Name -ne 'artifact-manifest.json' } | Sort-Object FullName | ForEach-Object {
    [ordered]@{path=$_.FullName.Substring($absoluteDir.Length+1).Replace('\','/');bytes=$_.Length;sha256=(Get-FileHash -Algorithm SHA256 $_.FullName).Hash.ToLower()}
})
[ordered]@{review_round=7;reviewed_commit=$f.head;outcome='pass';files=$manifest} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath "$dir/artifact-manifest.json" -Encoding utf8
"Reviewed 8/9; $($manifest.Count) evidence leaves fingerprinted; PowerShell drivers parse clean."
