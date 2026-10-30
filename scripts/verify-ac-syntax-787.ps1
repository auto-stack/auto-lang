# PLAN-787 验收聚合脚本（V-01..V-13 矩阵）
# 用法：
#   scripts/verify-ac-syntax-787.ps1 -Scope acceptance   # 日常验收（含 work-owned 全矩阵）
#   scripts/verify-ac-syntax-787.ps1 -Scope closeout     # merge 冻结提交完整套件
# F787-R1-09/10 修复轮：
#   - perf_/syntax_budget_ 为 work-owned（V-11/V-13）→ acceptance 必跑（不再
#     后置 closeout；closeout 另跑完整工作区 + 受影响 783 独立套件）；
#   - closeout 的 783 触面套件参数修复（--lib 不再与重定向拼成单参数）。
param(
    [Parameter(Mandatory = $false)]
    [ValidateSet("acceptance", "closeout")]
    [string]$Scope = "acceptance"
)
. (Join-Path $PSScriptRoot 'test-governance/ace-cargo.ps1')


# 注意：cargo 向 stderr 输出编译警告（shim-metadata 预存归属）——EAP 必须
# 为 Continue，否则 PowerShell 把 stderr 行当 NativeCommandError 终止；失败
# 判定统一走 $LASTEXITCODE 与结果解析。
$ErrorActionPreference = "Continue"
Remove-Item Env:RUSTC_WRAPPER -ErrorAction SilentlyContinue
$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root "experimental/ace/Cargo.toml"

# 族表：family -> (package, 最小通过数)
$families = [ordered]@{
    "snapshot"              = @{ pkg = "ace-syntax-core"; min = 4 }
    "grammar"               = @{ pkg = "ace-syntax-grammar";     min = 6 }
    "grammar_reject"        = @{ pkg = "ace-syntax-grammar";     min = 7 }
    "lossless"              = @{ pkg = "ace-syntax-grammar";     min = 9 }
    "recovery"              = @{ pkg = "ace-syntax-grammar";     min = 4 }
    "core_projection"       = @{ pkg = "ace-syntax-core"; min = 4 }
    "incremental_local"     = @{ pkg = "ace-syntax-core"; min = 3 }
    "incremental_replay"    = @{ pkg = "ace-syntax-core"; min = 4 }
    "generation"            = @{ pkg = "ace-syntax-core"; min = 3 }
    "depth"                 = @{ pkg = "ace-syntax-grammar";     min = 4 }
    "depth_persisted"       = @{ pkg = "ace-syntax-artifacts"; min = 1 }
    "syntax_cache"          = @{ pkg = "ace-syntax-artifacts"; min = 5 }
    "syntax_admission"      = @{ pkg = "ace-syntax-artifacts"; min = 6 }
    "compatibility"         = @{ pkg = "ace-syntax-core"; min = 3 }
    "perf"                  = @{ pkg = "ace-syntax-core"; min = 2 }
    "syntax_budget"         = @{ pkg = "ace-syntax-artifacts"; min = 2 }
}

$failed = @()
foreach ($family in $families.Keys) {
    $spec = $families[$family]
    Write-Host "== $family ($($spec.pkg)) =="
    $out = & Invoke-AceCargo test "--manifest-path" $manifest "-p" $spec.pkg "--" $family "--test-threads=1" 2>&1
    $text = $out | Out-String
    if ($LASTEXITCODE -ne 0) {
        Write-Host $text
        $failed += $family
        continue
    }
    $passed = 0; $ignored = 0; $anyFailed = $false; $saw = $false
    foreach ($m2 in [regex]::Matches($text, "result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored")) {
        $saw = $true
        $passed += [int]$m2.Groups[2].Value
        $ignored += [int]$m2.Groups[4].Value
        if ($m2.Groups[3].Value -ne "0" -or $m2.Groups[1].Value -eq "FAILED") { $anyFailed = $true }
    }
    if (-not $saw) {
        Write-Host $text
        $failed += "$family (no-result)"
        continue
    }
    if ($anyFailed) {
        Write-Host $text
        $failed += "$family (sub-failed)"
        continue
    }
    if ($passed -lt $spec.min) {
        Write-Host "FAIL $family passed=$passed < min=$($spec.min)`n$text"
        $failed += "$family (min)"
        continue
    }
    if ($ignored -gt 0) {
        Write-Host "FAIL $family ignored=$ignored (skip/pending not allowed)`n$text"
        $failed += "$family (ignored)"
        continue
    }
    Write-Host "PASS $family passed=$passed"
}

if ($failed.Count -gt 0) {
    Write-Host "FAILED families: $($failed -join ', ')"
    exit 1
}

# closeout：完整独立语法工作区 + 受影响的 783 独立套件（schema/codec/admission 触面）。
if ($Scope -eq "closeout") {
    Write-Host "== syntax workspace full (all packages, all targets) =="
    & Invoke-AceCargo test "--manifest-path" $manifest -p ace-syntax-runtime -p ace-syntax-grammar -p ace-syntax-core -p ace-syntax-artifacts 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAIL syntax workspace full suite"
        exit 1
    }
    Write-Host "PASS syntax workspace full suite"

    Write-Host "== 783 touched families (--lib / admission_ / dep_audit_) =="
    $probe = Join-Path $root "experimental/ace/crates/artifacts/Cargo.toml"
    foreach ($t in @(@("--lib"), @("--test", "admission_"), @("--test", "dep_audit_"))) {
        $cargoArgs = @("test", "--manifest-path", $probe) + $t + @("--", "--test-threads=1")
        & Invoke-AceCargo @cargoArgs | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Write-Host "FAIL probe $($t -join ' ')"
            exit 1
        }
        Write-Host "PASS probe $($t -join ' ')"
    }
}

Write-Host "PLAN-787 $Scope verify: ALL PASS"
exit 0
