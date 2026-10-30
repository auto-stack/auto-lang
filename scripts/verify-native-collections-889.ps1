# PLAN-889 scoped verification. Formal timing is an explicit serial window;
# implementation scopes never invoke whole repository test tiers.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("source", "resources", "incremental", "compat", "formal", "acceptance")]
    [string]$Scope
)
$ErrorActionPreference = "Continue"
$repo = Split-Path -Parent $PSScriptRoot
$nativeManifest = Join-Path $repo "experimental/ac-mir-native/Cargo.toml"
function Native-Test([string]$Name, [string]$Filter = "") {
    $testArgs = @("test", "--release", "--manifest-path", $nativeManifest, "--test", $Name)
    if ($Filter) { $testArgs += $Filter }
    & cargo @testArgs -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
function SDK-Test([string]$Filter, [switch]$Formal) {
    $testArgs = @("test", "-p", "auto-lang", "--features", "test-cases", "--lib", $Filter, "--", "--test-threads=1", "--nocapture")
    if ($Formal) { $testArgs += "--ignored" }
    & cargo @testArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
Push-Location $repo
try {
    switch ($Scope) {
        "source" {
            Native-Test "collections_source_tier"
            SDK-Test "ace_m1_native_collections_889_source_sdk"
            SDK-Test "ace_m1_native_collections_889_wrong_oracle_and_wrong_type_turn_red"
        }
        "resources" {
            Native-Test "collections_resources_tier"
            Native-Test "collections_typed_tier"
            & cargo test --release --manifest-path $nativeManifest --bin mn-collections-worker allocation_controls -- --test-threads=1
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            SDK-Test "ace_m1_native_collections_889_actual_missing_free_turns_parent_red"
        }
        "incremental" {
            Native-Test "collections_incremental_tier"
            SDK-Test "ace_m1_native_collections_889_incremental_rounds_and_restore_sdk"
            SDK-Test "ace_m1_native_collections_889_real_whole_work_and_code_copy_turn_sdk_red"
        }
        "compat" {
            Native-Test "cache_tier"
            Native-Test "cli_871"
        }
        "formal" {
            SDK-Test "ace_m1_native_collections_889_formal_performance_and_churn" -Formal
        }
        "acceptance" {
            # Per-plan affected files only. This is a final frozen acceptance
            # replay, not a development/individual-finding repair shortcut.
            Native-Test "collections_driver_tier"
            Native-Test "collections_incremental_tier"
            Native-Test "collections_typed_tier"
            & cargo test -p auto-lang-core --features test-cases --lib native_collections_889_ -- --test-threads=1
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            SDK-Test "ace_m1_native_collections_889_"
            SDK-Test "ace_m1_native_collections_889_formal_performance_and_churn" -Formal
        }
    }
} finally { Pop-Location }
