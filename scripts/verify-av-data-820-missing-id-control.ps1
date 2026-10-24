# PLAN-820 F820-R2-02/R4 close — END-TO-END negative control (never a
# product run): a mocked `cargo` whose listing AND execution both show
# only ONE first-case id (the other twelve contract-required cases are
# "deleted"). The R3 guard shape (--list derivation alone) stayed GREEN
# on this input because the missing cases vanished from listing and
# execution together; the independent CONTRACT manifest must now make
# verify-av-data-820.ps1 exit NON-ZERO. This control asserts exactly
# that; if the verify script exits 0, the guard is blind again.
#
# Usage: pwsh -File scripts/verify-av-data-820-missing-id-control.ps1
# Exit 0 = the control PASSED (the verify script correctly went red).

$ErrorActionPreference = "Stop"
$verify = Join-Path $PSScriptRoot "verify-av-data-820.ps1"

# The mock replaces cargo ONLY inside this control session (the verify
# script resolves `cargo` through the session's function table). It never
# touches the real toolchain, the tree, or any store.
function cargo {
    if ($args -contains "--list") {
        Write-Output "av_data_820_first_case_whole_path: test"
        Write-Output ""
        Write-Output "1 test, 0 benchmarks"
    } else {
        Write-Output "running 1 test"
        Write-Output "test av_data_820_first_case_whole_path ... ok"
        Write-Output ""
        Write-Output "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.0s"
    }
    $global:LASTEXITCODE = 0
}

Write-Host "[missing-id-control] mocked cargo: listing+execution hold only av_data_820_first_case_whole_path"
& $verify -Scope first-case | Out-Null
$code = $LASTEXITCODE
if ($code -eq 0) {
    Write-Host "[missing-id-control] FAIL: the verify script stayed GREEN with 12 contract cases deleted — the guard is blind." -ForegroundColor Red
    exit 1
}
Write-Host "[missing-id-control] PASS: verify script exited $code (contract manifest caught the deleted required cases)" -ForegroundColor Green
exit 0
