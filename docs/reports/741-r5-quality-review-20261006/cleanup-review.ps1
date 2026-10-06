$ErrorActionPreference='Stop'
$group='D:\autostack\.wt\lang-741'
$items=@(
    @{Path='D:\autostack\.wt\lang-741\auto-lang'; Main='D:/autostack/auto-lang'; Head='6baed9bba84016dd8610221a9a0358195444385e'},
    @{Path='D:\autostack\.wt\lang-741\auto-down'; Main='D:/autostack/auto-down'; Head='fba6563ed2148ce85e68208863159b4ccccac710'}
)
$guards=@()
foreach($item in $items){
    $root=(Get-Item -LiteralPath $item.Path -Force).FullName.TrimEnd('\')
    if($root -ne $item.Path -or (Split-Path -Parent $root) -ne $group){throw "Unexpected absolute target: $root"}
    $head=& git -c "safe.directory=$root" -c core.excludesfile= -C $root rev-parse HEAD
    if($LASTEXITCODE -ne 0 -or $head -ne $item.Head){throw "Revision changed: $root"}
    $state=@(& git -c "safe.directory=$root" -c core.excludesfile= -C $root status --porcelain --untracked-files=all)
    if($LASTEXITCODE -ne 0 -or $state.Count -ne 0){throw "Dirty worktree: $root"}
    if((Get-Item -LiteralPath $root -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Root reparse point'}
    $stack=New-Object 'System.Collections.Generic.Stack[string]'
    $stack.Push($root); $count=0
    while($stack.Count -gt 0){
        $directory=$stack.Pop()
        foreach($entry in Get-ChildItem -LiteralPath $directory -Force){
            $count++
            if($entry.Attributes -band [IO.FileAttributes]::ReparsePoint){throw "Reparse point: $($entry.FullName)"}
            if($entry.PSIsContainer){$stack.Push($entry.FullName)}
        }
    }
    $guards+=@{path=$root;head=$head;git_clean=$true;reparse_points=0;scanned_entries=$count}
}
# Only remove after BOTH complete preflight checks; no recursive shell delete.
foreach($item in $items){
    & git -c "safe.directory=$($item.Main)" -c core.excludesfile= -C $item.Main worktree remove $item.Path
    if($LASTEXITCODE -ne 0){throw "git worktree remove failed: $($item.Path)"}
}
if((Get-Item -LiteralPath $group -Force).FullName -ne $group){throw 'Unexpected group path'}
if(@(Get-ChildItem -LiteralPath $group -Force).Count -eq 0){Remove-Item -LiteralPath $group}
$receipt=@{reviewed_commit=$items[0].Head;guard_script_missing=(-not(Test-Path -LiteralPath 'D:/autostack/wt-guard.sh'));bash_guard_failure='WSL VHDX D:/wsl/ext4.vhdx missing; ERROR_PATH_NOT_FOUND';authorized_equivalent='prior r1-r5 receipt and current Plan §10 precise path/ReparsePoint fallback';guards=$guards;lang_tree_removed=(-not(Test-Path -LiteralPath $items[0].Path));dependency_tree_removed=(-not(Test-Path -LiteralPath $items[1].Path));group_removed=(-not(Test-Path -LiteralPath $group));branches_deleted=@();unrelated_worktrees_preserved=$true}
$receipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath 'C:/Users/zhaop/AppData/Local/Temp/741-r5-review-20261006/cleanup.json' -Encoding UTF8
$receipt | ConvertTo-Json -Depth 6
