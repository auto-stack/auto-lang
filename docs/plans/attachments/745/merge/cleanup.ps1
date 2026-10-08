$ErrorActionPreference='Stop'
$taskMain='D:/autostack/auto-lang'
$taskPacket=Join-Path $taskMain 'docs/plans/attachments/745/merge'
$taskGroup=[IO.Path]::GetFullPath('D:/autostack/.wt/lang-745')
$taskTree=[IO.Path]::GetFullPath('D:/autostack/.wt/lang-745/auto-lang')
$taskDependency=[IO.Path]::GetFullPath('D:/autostack/.wt/lang-745/auto-down')
$taskDependencyMain='D:/autostack/auto-down'
$taskDependencyExpected='fba6563ed2148ce85e68208863159b4ccccac710'

function Invoke-TaskGit([string]$taskCwd,[string[]]$taskArgs){
    $taskOutput=& git -C $taskCwd @taskArgs
    if($LASTEXITCODE){throw ('git failed: '+($taskArgs -join ' '))}
    return $taskOutput
}
function Get-TaskCommon([string]$taskRepo){
    $taskCommon=Invoke-TaskGit $taskRepo @('rev-parse','--git-common-dir')
    if(-not [IO.Path]::IsPathRooted($taskCommon)){$taskCommon=Join-Path $taskRepo $taskCommon}
    return [IO.Path]::GetFullPath($taskCommon)
}
function Assert-TaskOrdinary([string]$taskPath){
    $taskItem=Get-Item -LiteralPath $taskPath -Force
    if(-not $taskItem.PSIsContainer -or ($taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw ('not ordinary directory: '+$taskPath)}
}
$taskArchiveGate=Get-Content (Join-Path $taskPacket 'assertions-archived.json') -Raw | ConvertFrom-Json
if($taskArchiveGate.outcome -ne 'pass'){throw 'actual archive gate missing'}
foreach($taskPath in @('D:/autostack/.wt',$taskGroup,$taskTree,$taskDependency)){Assert-TaskOrdinary $taskPath}
foreach($taskPath in @($taskTree,$taskDependency)){
    if([IO.Path]::GetDirectoryName($taskPath) -ne $taskGroup){throw 'path outside exact intended group'}
    if(Invoke-TaskGit $taskPath @('status','--porcelain')){throw ('dirty tree: '+$taskPath)}
}
if((Get-TaskCommon $taskTree) -ne (Get-TaskCommon $taskMain)){throw 'auto-lang ownership mismatch'}
if((Get-TaskCommon $taskDependency) -ne (Get-TaskCommon $taskDependencyMain)){throw 'auto-down ownership mismatch'}
if((Invoke-TaskGit $taskTree @('branch','--show-current')) -ne 'plan-745-dev'){throw 'wrong development branch'}
$taskTreeHead=Invoke-TaskGit $taskTree @('rev-parse','HEAD')
Invoke-TaskGit $taskMain @('merge-base','--is-ancestor',$taskTreeHead,'HEAD')
$taskDependencyHead=Invoke-TaskGit $taskDependency @('rev-parse','HEAD')
if($taskDependencyHead -ne $taskDependencyExpected -or (Invoke-TaskGit $taskDependencyMain @('rev-parse','HEAD')) -ne $taskDependencyExpected){throw 'dependency drift'}
$taskBefore=@{key='PLAN-745:r1';main_head=(Invoke-TaskGit $taskMain @('rev-parse','HEAD'));tree_head=$taskTreeHead;dependency_head=$taskDependencyHead;tree_common=(Get-TaskCommon $taskTree);dependency_common=(Get-TaskCommon $taskDependency);group=$taskGroup;paths=@($taskTree,$taskDependency);clean=$true;all_commits_landed=$true;ordinary_paths_verified=$true}
$taskBefore | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $taskPacket 'cleanup-before.json')

# Each no-follow guard is immediately followed by removal of that exact tree.
& D:/soft/Git/bin/bash.exe D:/autostack/wt-guard.sh $taskTree | Set-Content (Join-Path $taskPacket 'guard-remove-auto-lang.txt')
if($LASTEXITCODE){throw 'auto-lang fresh guard failed'}
Invoke-TaskGit $taskMain @('worktree','remove',$taskTree)
if(Test-Path -LiteralPath $taskTree){throw 'auto-lang tree remains'}
& D:/soft/Git/bin/bash.exe D:/autostack/wt-guard.sh $taskDependency | Set-Content (Join-Path $taskPacket 'guard-remove-auto-down.txt')
if($LASTEXITCODE){throw 'auto-down fresh guard failed'}
Invoke-TaskGit $taskDependencyMain @('worktree','remove',$taskDependency)
if(Test-Path -LiteralPath $taskDependency){throw 'auto-down tree remains'}
Invoke-TaskGit $taskMain @('branch','-d','plan-745-dev')
Assert-TaskOrdinary $taskGroup
if(@(Get-ChildItem -LiteralPath $taskGroup -Force).Count -ne 0){throw 'group not empty; preserved'}
# Empty exact group only; never recursive deletion or shell forwarding.
Remove-Item -LiteralPath $taskGroup
if(Test-Path -LiteralPath $taskGroup){throw 'group remains'}
$taskLangList=Invoke-TaskGit $taskMain @('worktree','list','--porcelain')
$taskDownList=Invoke-TaskGit $taskDependencyMain @('worktree','list','--porcelain')
if(($taskLangList -join "`n") -match 'lang-745' -or ($taskDownList -join "`n") -match 'lang-745'){throw 'Git registration remains'}
& git -C $taskMain show-ref --verify refs/heads/plan-745-dev 2>$null
if($LASTEXITCODE -eq 0){throw 'development branch remains'}
$taskRecovery=Invoke-TaskGit $taskMain @('rev-parse','refs/auto-plan/745/reviewed')
if($taskRecovery -ne 'bb8cb4b4330d505059ea20e7bfe561a93c1051f3'){throw 'review recovery ref missing'}
@{key='PLAN-745:r1';outcome='pass';tree=$taskTree;tree_head=$taskTreeHead;dependency=$taskDependency;dependency_head=$taskDependencyHead;group=$taskGroup;fresh_guards=@('guard-remove-auto-lang.txt','guard-remove-auto-down.txt');guard_source_sha256=(Get-FileHash 'D:/autostack/wt-guard.ps1').Hash.ToLower();disk_absence_verified=$true;git_registration_absence_verified=$true;merged_branch_removed=$true;empty_group_only=$true;recovery_ref='refs/auto-plan/745/reviewed';recovery_commit=$taskRecovery;completed_utc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $taskPacket 'cleanup.json')
Write-Output 'PLAN-745 original tree, read-only dependency tree, merged branch and empty group removed; recovery ref preserved.'
