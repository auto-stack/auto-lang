# PLAN-891 scoped verification. Formal timing is an explicit serial window;
# implementation scopes never invoke whole repository test tiers.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("abi", "transport", "incremental", "performance", "compat", "sdk", "acceptance")]
    [string]$Scope
)
. (Join-Path $PSScriptRoot 'test-governance/ace-cargo.ps1')

$ErrorActionPreference = "Continue"
$repo = Split-Path -Parent $PSScriptRoot
$nativeManifest = Join-Path $repo "experimental/ace/crates/native/Cargo.toml"
function Native-Test([string]$Name, [string]$Filter = "") {
    $testArgs = @("test", "--release", "--manifest-path", $nativeManifest, "--test", $Name)
    if ($Filter) { $testArgs += $Filter }
    & Invoke-AceCargo @testArgs '--' --test-threads=1
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
function SDK-Test([string]$Filter) {
    $testArgs = @("test", "-p", 'ace-conformance', "--features", "test-cases", '--test', 'conformance', $Filter, "--", "--test-threads=1", "--nocapture")
    & Invoke-AceCargo @testArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
Push-Location $repo
try {
    switch ($Scope) {
        "abi" {
            # Six-scalar entry/ABI/frames: const-return, identity, mixed
            # selections, cross-object calls, records, value70 vs traps,
            # codec negatives, fingerprints, budgets.
            Native-Test "typed_entry_tier" "typed_probe"
            Native-Test "typed_entry_tier" "typed_frame_codec"
            Native-Test "typed_entry_tier" "typed_fingerprints"
            Native-Test "typed_entry_tier" "typed_budget"
        }
        "transport" {
            Native-Test "typed_entry_tier" "typed_cache"
        }
        "incremental" {
            Native-Test "typed_entry_incremental_tier"
        }
        "performance" {
            # Formal four-state x 20-sample turnaround window (serial).
            Native-Test "typed_entry_tier" "performance::"
        }
        "compat" {
            # The legacy scalar surface must stay byte-identical in
            # behavior: the full pre-891 conformance suite.
            Native-Test "conformance"
        }
        "sdk" {
            SDK-Test "ace_m1_native_typed_891"
        }
        "acceptance" {
            Native-Test "typed_entry_tier" "typed_probe"
            Native-Test "typed_entry_tier" "typed_frame_codec"
            Native-Test "typed_entry_tier" "typed_fingerprints"
            Native-Test "typed_entry_tier" "typed_budget"
            Native-Test "typed_entry_tier" "typed_cache"
            Native-Test "typed_entry_incremental_tier"
            Native-Test "typed_entry_tier" "performance::"
            Native-Test "conformance"
            SDK-Test "ace_m1_native_typed_891"
        }
    }
}
finally {
    Pop-Location
}
