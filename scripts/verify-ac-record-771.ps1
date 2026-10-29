# PLAN-771 one-shot record-profile verification script (ASCII only for
# Windows PowerShell 5.1). Serial gate: full ac-core check/fmt/tests (all
# domains), record_native PE suite single-threaded, the record case
# walkthrough (positive source flagship + eval-order trace 'ba', aliased-
# destination value-semantics trio, positional construction [F-771-R1-01/
# 02] and ctor-operand snapshot order [F-771-R2-01], positive HIR fixtures
# r01/r03/r04 with distinct exit codes plus the forced-failure copy control
# [F-771-R1-04], 10 negative fixtures with pinned codes, profile isolation,
# SENTINEL transaction preservation, receipt identity), then the three
# old-profile gates (741 -SkipMainGates / 745 / 767) to prove the old
# surfaces are byte-for-byte unaffected. Raw receipts land in
# experimental/ac-core/target/ac-verify-receipts/record-771/ (gitignored).
# The contained executor is VERBATIM the 745/741/767 runner.
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-record-771.ps1
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$AcCore = Join-Path $RepoRoot 'experimental\ac-core'
# PLAN-871 T-05: build legs (HIR `build` and `build-source`) run through the
# shared-MIR `ac-native` CLI (the receiver of ac-core's retiring
# build/build-source). Old assertion mapping: every exit / stdout / stderr
# oracle is UNCHANGED; the old `app.obj` product assertions become "receipt
# exists / survives the failed-build transaction with backend+mir axes" —
# the shared-MIR closure has no single object.
$AcMn = Join-Path $RepoRoot 'experimental\ac-mir-native'
$ReceiptsDir = Join-Path $AcCore 'target\ac-verify-receipts\record-771'
New-Item -ItemType Directory -Force -Path $ReceiptsDir | Out-Null

$script:Results = New-Object System.Collections.Generic.List[object]

function Invoke-Step {
    param([string]$Name, [scriptblock]$Body, [int]$TimeoutSec = 600)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        & $Body
        $script:Results.Add([pscustomobject]@{ Step = $Name; Ok = $true; Seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1); Detail = '' })
        Write-Host ("[PASS] {0} ({1}s)" -f $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1)) -ForegroundColor Green
    } catch {
        $script:Results.Add([pscustomobject]@{ Step = $Name; Ok = $false; Seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1); Detail = $_.Exception.Message })
        Write-Host ("[FAIL] {0} ({1}s): {2}" -f $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1), $_.Exception.Message) -ForegroundColor Red
        throw
    }
}

$script:AcRunnerCs = @'
using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

// PLAN-745 R3-QA-01: a PS-hostable contained executor with the SAME
// lifecycle contract as the 741 Rust executor (link.rs): suspended start,
// Job-Object subtree containment before the first instruction, one
// deadline covering process exit AND pipe-EOF collection, byte draining
// with lossy UTF-8 conversion, and full tree + reader reclamation on every
// exit path. No quiet-window heuristics, no detached readers.
public static class AcContainedRunner
{
    [StructLayout(LayoutKind.Sequential)]
    struct SECURITY_ATTRIBUTES
    {
        public int nLength;
        public IntPtr lpSecurityDescriptor;
        public bool bInheritHandle;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct STARTUPINFO
    {
        public int cb;
        public IntPtr lpReserved;
        public IntPtr lpDesktop;
        public IntPtr lpTitle;
        public int dwX;
        public int dwY;
        public int dwXSize;
        public int dwYSize;
        public int dwXCountChars;
        public int dwYCountChars;
        public int dwFillAttribute;
        public int dwFlags;
        public short wShowWindow;
        public short cbReserved2;
        public IntPtr lpReserved2;
        public IntPtr hStdInput;
        public IntPtr hStdOutput;
        public IntPtr hStdError;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct PROCESS_INFORMATION
    {
        public IntPtr hProcess;
        public IntPtr hThread;
        public int dwProcessId;
        public int dwThreadId;
    }

    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    static extern bool CreateProcess(
        string lpApplicationName,
        string lpCommandLine,
        IntPtr lpProcessAttributes,
        IntPtr lpThreadAttributes,
        bool bInheritHandles,
        uint dwCreationFlags,
        IntPtr lpEnvironment,
        string lpCurrentDirectory,
        ref STARTUPINFO lpStartupInfo,
        out PROCESS_INFORMATION lpProcessInformation);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool CreatePipe(out IntPtr hReadPipe, out IntPtr hWritePipe, ref SECURITY_ATTRIBUTES lpPipeAttributes, uint nSize);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool CloseHandle(IntPtr hObject);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr CreateJobObject(IntPtr lpJobAttributes, string lpName);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool AssignProcessToJobObject(IntPtr hJob, IntPtr hProcess);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool TerminateJobObject(IntPtr hJob, uint uExitCode);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern uint ResumeThread(IntPtr hThread);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool TerminateProcess(IntPtr hProcess, uint uExitCode);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern int WaitForSingleObject(IntPtr hHandle, uint dwMilliseconds);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool GetExitCodeProcess(IntPtr hProcess, out uint lpExitCode);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool ReadFile(IntPtr hFile, byte[] lpBuffer, uint nNumberOfBytesToRead, out uint lpNumberOfBytesRead, IntPtr lpOverlapped);

    const uint CREATE_SUSPENDED = 0x4;
    const uint STARTF_USESTDHANDLES = 0x100;
    const int STILL_ACTIVE = 259;
    const int WAIT_TIMEOUT = 258;
    const int WAIT_OBJECT_0 = 0;

    static string Escape(string arg)
    {
        if (arg == null) return "\"\"";
        if (arg.Length > 0 && arg.IndexOfAny(new[] { ' ', '\t', '"' }) < 0) return arg;
        var sb = new StringBuilder();
        sb.Append('"');
        int bs = 0;
        foreach (char c in arg)
        {
            if (c == '\\') { bs++; continue; }
            if (c == '"') { sb.Append('\\', bs + 1); sb.Append('"'); bs = 0; continue; }
            sb.Append('\\', bs); bs = 0; sb.Append(c);
        }
        sb.Append('\\', bs);
        sb.Append('"');
        return sb.ToString();
    }

    class Reader
    {
        const int ERROR_BROKEN_PIPE = 109;
        const int ERROR_HANDLE_EOF = 38;
        readonly IntPtr _handle;
        public readonly MemoryStream Buf = new MemoryStream();
        public readonly Thread Thr;
        public bool Done;
        // R4-QA-01: a failed ReadFile is NOT an EOF. Broken-pipe/handle-EOF
        // are the pipe's normal end; anything else is a real read error
        // (recorded, surfaced through RunResult.Error — never a silent
        // truncated success).
        public int ReadError;
        public Reader(IntPtr handle)
        {
            _handle = handle;
            Thr = new Thread(Drain) { IsBackground = true };
        }
        void Drain()
        {
            var b = new byte[65536];
            while (true)
            {
                uint n;
                if (!ReadFile(_handle, b, (uint)b.Length, out n, IntPtr.Zero))
                {
                    int e = Marshal.GetLastWin32Error();
                    if (e != ERROR_BROKEN_PIPE && e != ERROR_HANDLE_EOF) ReadError = e;
                    break;
                }
                if (n == 0) break;
                Buf.Write(b, 0, (int)n);
            }
            lock (this) { Done = true; Monitor.Pulse(this); }
        }
        public bool WaitDone(int ms)
        {
            lock (this)
            {
                if (Done) return true;
                return Monitor.Wait(this, ms) && Done;
            }
        }
    }

    public sealed class RunResult
    {
        public int Code;
        public string Stdout;
        public string Stderr;
        public bool TimedOut;
        public string Error;
        public int Pid;
        public int StdoutReadError;
        public int StderrReadError;
    }

    public static RunResult Run(string exe, string[] args, string workDir, int timeoutMs)
    {
        return RunCore(exe, args, workDir, timeoutMs, false, false);
    }

    // R4-QA-01 failure-exit controls (741-style injectable determinism, no
    // product fault switch): simulateAssignFailure forces the REAL
    // AssignProcessToJobObject call to fail (job=zero handle), simulateRead
    // Failure forces the REAL ReadFile call to fail (handle=zero). Both
    // native calls still execute and return genuine Win32 errors.
    public static RunResult RunControl(
        string exe, string[] args, string workDir, int timeoutMs,
        bool simulateAssignFailure, bool simulateReadFailure)
    {
        return RunCore(exe, args, workDir, timeoutMs, simulateAssignFailure, simulateReadFailure);
    }

    static RunResult RunCore(
        string exe, string[] args, string workDir, int timeoutMs,
        bool breakAssign, bool breakRead)
    {
        var res = new RunResult();
        var sa = new SECURITY_ATTRIBUTES { nLength = Marshal.SizeOf(typeof(SECURITY_ATTRIBUTES)), bInheritHandle = true };
        IntPtr inR = IntPtr.Zero, inW = IntPtr.Zero, outR = IntPtr.Zero, outW = IntPtr.Zero, errR = IntPtr.Zero, errW = IntPtr.Zero;
        IntPtr job = IntPtr.Zero;
        IntPtr hProc = IntPtr.Zero, hThread = IntPtr.Zero;
        Reader outReader = null, errReader = null;
        try
        {
            if (!CreatePipe(out inR, out inW, ref sa, 0) ||
                !CreatePipe(out outR, out outW, ref sa, 0) ||
                !CreatePipe(out errR, out errW, ref sa, 0))
            {
                res.Error = "pipe creation failed";
                return res;
            }
            // stdin write end stays ours; the child sees a quiet stdin.
            var si = new STARTUPINFO
            {
                cb = Marshal.SizeOf(typeof(STARTUPINFO)),
                dwFlags = (int)STARTF_USESTDHANDLES,
                hStdInput = inR,
                hStdOutput = outW,
                hStdError = errW,
            };
            var cmd = new StringBuilder();
            cmd.Append(Escape(exe));
            foreach (var a in args) { cmd.Append(' '); cmd.Append(Escape(a)); }
            PROCESS_INFORMATION pi;
            if (!CreateProcess(null, cmd.ToString(), IntPtr.Zero, IntPtr.Zero, true, CREATE_SUSPENDED,
                IntPtr.Zero, workDir, ref si, out pi))
            {
                res.Error = "CreateProcess failed: " + Marshal.GetLastWin32Error();
                return res;
            }
            hProc = pi.hProcess; hThread = pi.hThread;
            res.Pid = pi.dwProcessId;

            job = CreateJobObject(IntPtr.Zero, null);
            IntPtr jobToAssign = (breakAssign && job != IntPtr.Zero) ? IntPtr.Zero : job;
            if (job == IntPtr.Zero || !AssignProcessToJobObject(jobToAssign, hProc))
            {
                // R4-QA-01: the suspended child is NOT in the job here — a
                // job terminate cannot reach it. Terminate the process
                // directly and WAIT for it before releasing any handle, so
                // a containment failure never leaks a suspended process.
                TerminateProcess(hProc, 1);
                WaitForSingleObject(hProc, 2000);
                CloseHandle(hThread); hThread = IntPtr.Zero;
                CloseHandle(hProc); hProc = IntPtr.Zero;
                res.Error = "containment unavailable";
                return res;
            }
            if (ResumeThread(hThread) == uint.MaxValue)
            {
                TerminateJobObject(job, 1);
                TerminateProcess(hProc, 1);
                WaitForSingleObject(hProc, 2000);
                CloseHandle(hThread); hThread = IntPtr.Zero;
                CloseHandle(hProc); hProc = IntPtr.Zero;
                res.Error = "resume failed";
                return res;
            }
            res.Pid = pi.dwProcessId;
            // parent-side write ends close so EOF reflects the CHILD tree
            CloseHandle(outW); outW = IntPtr.Zero;
            CloseHandle(errW); errW = IntPtr.Zero;
            outReader = new Reader(breakRead ? IntPtr.Zero : outR); outReader.Thr.Start();
            errReader = new Reader(errR); errReader.Thr.Start();

            var sw = Stopwatch.StartNew();
            // C#5 (PS 5.1 compiler): no local functions — use a lambda
            Func<int> remaining = () => (int)Math.Max(0, timeoutMs - sw.ElapsedMilliseconds);

            while (true)
            {
                int w = WaitForSingleObject(hProc, (uint)Math.Max(1, Math.Min(remaining(), 50)));
                if (w == WAIT_OBJECT_0)
                {
                    // process exited; collection bound by the REMAINING budget
                    int rem = remaining();
                    bool okOut = outReader.WaitDone(rem);
                    bool okErr = errReader.WaitDone(okOut ? remaining() : 0);
                    if (okOut && okErr && remaining() >= 0)
                    {
                        // R4-QA-01: a reader error is a FAILURE, not an
                        // EOF-flavored success with truncated output.
                        if (outReader.ReadError != 0 || errReader.ReadError != 0)
                        {
                            res.StdoutReadError = outReader.ReadError;
                            res.StderrReadError = errReader.ReadError;
                            res.Error = "output read failed (stdout err " + outReader.ReadError
                                + ", stderr err " + errReader.ReadError + ")";
                            return res;
                        }
                        uint code;
                        GetExitCodeProcess(hProc, out code);
                        res.Code = (int)code;
                        res.Stdout = Encoding.UTF8.GetString(outReader.Buf.ToArray(), 0, (int)outReader.Buf.Length);
                        res.Stderr = Encoding.UTF8.GetString(errReader.Buf.ToArray(), 0, (int)errReader.Buf.Length);
                        return res;
                    }
                    // a descendant still holds the pipes past the deadline:
                    // terminate the tree; this is a timeout, never a
                    // truncated success
                    TerminateJobObject(job, 1);
                    outReader.WaitDone(2000);
                    errReader.WaitDone(2000);
                    res.TimedOut = true;
                    res.Stdout = Encoding.UTF8.GetString(outReader.Buf.ToArray(), 0, (int)outReader.Buf.Length);
                    res.Stderr = Encoding.UTF8.GetString(errReader.Buf.ToArray(), 0, (int)errReader.Buf.Length);
                    return res;
                }
                if (remaining() <= 0)
                {
                    TerminateJobObject(job, 1);
                    WaitForSingleObject(hProc, 2000);
                    outReader.WaitDone(2000);
                    errReader.WaitDone(2000);
                    res.TimedOut = true;
                    res.Stdout = Encoding.UTF8.GetString(outReader.Buf.ToArray(), 0, (int)outReader.Buf.Length);
                    res.Stderr = Encoding.UTF8.GetString(errReader.Buf.ToArray(), 0, (int)errReader.Buf.Length);
                    return res;
                }
            }
        }
        finally
        {
            // full reclamation on EVERY path: terminate the tree (idempotent
            // for an exited job), wait the readers out (pipes are closed by
            // then), release handles.
            if (job != IntPtr.Zero) { TerminateJobObject(job, 1); CloseHandle(job); }
            if (outReader != null) outReader.WaitDone(3000);
            if (errReader != null) errReader.WaitDone(3000);
            if (hThread != IntPtr.Zero) CloseHandle(hThread);
            if (hProc != IntPtr.Zero) CloseHandle(hProc);
            foreach (var h in new[] { inR, inW, outR, outW, errR, errW })
                if (h != IntPtr.Zero) CloseHandle(h);
        }
    }
}
'@
$script:AcRunnerLoaded = $false

function Invoke-Captured {
    # The 741/745 contained executor (verbatim runner above).
    param([string]$Exe, [string[]]$ArgList, [string]$WorkDir, [int]$TimeoutSec = 600)
    if (-not $script:AcRunnerLoaded) {
        Add-Type -TypeDefinition $script:AcRunnerCs -Language CSharp
        $script:AcRunnerLoaded = $true
    }
    $r = [AcContainedRunner]::Run($Exe, $ArgList, $WorkDir, ($TimeoutSec * 1000))
    if ($r.Error) { throw "contained executor refused '$Exe': $($r.Error)" }
    if ($r.TimedOut) {
        throw "subprocess '$Exe $($ArgList -join ' ')' exceeded ${TimeoutSec}s deadline (process or output collection)"
    }
    return [pscustomobject]@{
        Code = $r.Code
        Stdout = $r.Stdout
        Stderr = $r.Stderr
        TimedOut = $r.TimedOut
    }
}

function Get-Sha256 {
    param([string]$Path)
    (Get-FileHash -Algorithm SHA256 $Path).Hash.ToLower()
}


$Record = 'auto.source.record.draft'
$Trace = 'hir.test.trace.v1'

# ---- 1. context receipt ----
Invoke-Step 'context (HEAD + toolchain)' {
    $lines = @()
    $lines += 'git-head: ' + (Invoke-Captured 'git' @('rev-parse', 'HEAD') $RepoRoot -TimeoutSec 60).Stdout.Trim()
    $lines += 'git-branch: ' + (Invoke-Captured 'git' @('branch', '--show-current') $RepoRoot -TimeoutSec 60).Stdout.Trim()
    $lines += (Invoke-Captured 'rustc' @('-Vv') $RepoRoot -TimeoutSec 60).Stdout
    $lines += (Invoke-Captured 'cargo' @('-V') $RepoRoot -TimeoutSec 60).Stdout
    $lines | Set-Content (Join-Path $ReceiptsDir 'context.txt')
}

# ---- 2. ac-core scoped gates ----
Invoke-Step 'cargo check (ac-core)' {
    $r = Invoke-Captured 'cargo' @('check', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot
    if ($r.Code -ne 0) { throw "cargo check exited $($r.Code): $($r.Stderr)" }
}
Invoke-Step 'cargo fmt --check (ac-core)' {
    $r = Invoke-Captured 'cargo' @('fmt', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--', '--check') $RepoRoot
    if ($r.Code -ne 0) { throw "fmt --check failed: $($r.Stdout)$($r.Stderr)" }
}
Invoke-Step 'cargo check --all-targets, zero warnings (ac-core)' {
    $r = Invoke-Captured 'cargo' @('check', '--locked', '--all-targets', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot
    if ($r.Code -ne 0) { throw "check --all-targets exited $($r.Code): $($r.Stderr)" }
    if (($r.Stdout + $r.Stderr) -match '(?m)^warning:') { throw "unhandled compiler warnings in ac-core all-targets build" }
}
Invoke-Step 'cargo test --locked (full ac-core)' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot -TimeoutSec 900
    if ($r.Code -ne 0) { throw $r.Stderr }
}
# PLAN-871 T-06 mapping: record_native (retired lowering PE suite) → this
# script's ac-native SOURCE/HIR record legs below (same frozen exits,
# including the SP05 ctor-snapshot rows) plus
# `ac-mir-native/tests/record_method_tier.rs` and `gap_closure_871.rs`.

# ---- 3. serial case walkthrough ----
Invoke-Step 'build tool binaries once' {
    $r = Invoke-Captured 'cargo' @('build', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot -TimeoutSec 600
    if ($r.Code -ne 0) { throw $r.Stderr }
    $m = Invoke-Captured 'cargo' @('build', '--locked', '--manifest-path', (Join-Path $AcMn 'Cargo.toml'), '--bin', 'ac-native') $RepoRoot -TimeoutSec 900
    if ($m.Code -ne 0) { throw $m.Stderr }
    $script:Bin = Join-Path $AcMn 'target\debug\ac-native.exe'
    $script:CheckBin = Join-Path $AcCore 'target\debug\auto-ac-prototype.exe'
    if (-not (Test-Path $script:Bin)) { throw "binary missing: $script:Bin" }
    ("binary: " + $script:Bin) | Add-Content (Join-Path $ReceiptsDir 'context.txt')
    ("binary-sha256: " + (Get-Sha256 $script:Bin)) | Add-Content (Join-Path $ReceiptsDir 'context.txt')
}

Invoke-Step 'build trace support library once' {
    $r = Invoke-Captured 'cargo' @('build', '--locked') (Join-Path $AcCore 'test-support') -TimeoutSec 600
    if ($r.Code -ne 0) { throw $r.Stderr }
    $script:SupportLib = Join-Path $AcCore 'test-support\target\debug\ac_trace_support.lib'
    if (-not (Test-Path $script:SupportLib)) { throw "support lib missing: $script:SupportLib" }
    ("support-lib-sha256: " + (Get-Sha256 $script:SupportLib)) | Add-Content (Join-Path $ReceiptsDir 'context.txt')
}

$caseSummary = New-Object System.Collections.Generic.List[object]

Invoke-Step 'positive SOURCE case: flagship build + run (V-02/V-06/V-08)' {
    $dir = Join-Path $ReceiptsDir 'source-positive\SP01'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $srcp = Join-Path $RepoRoot 'experimental\ac-core\fixtures\record-source\valid\p01-records.at'
    ("input: " + $srcp) | Set-Content (Join-Path $dir 'case.txt')
    ("input-sha256: " + (Get-Sha256 $srcp)) | Add-Content (Join-Path $dir 'case.txt')
    $exe = Join-Path $dir 'app.exe'
    $build = Invoke-Captured $script:Bin @('build-source', $srcp, '--source-profile', $Record, '--entry', 'main', '--output', $exe) $RepoRoot -TimeoutSec 120
    $build.Stdout | Set-Content (Join-Path $dir 'build.stdout.txt')
    $build.Stderr | Set-Content (Join-Path $dir 'build.stderr.txt')
    if ($build.Code -ne 0) { throw "flagship build exited $($build.Code)" }
    if (-not (Test-Path $exe)) { throw "exe missing" }
    $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
    $run.Stdout | Set-Content (Join-Path $dir 'run.stdout.txt')
    $run.Stderr | Set-Content (Join-Path $dir 'run.stderr.txt')
    ("exit: " + $run.Code) | Add-Content (Join-Path $dir 'case.txt')
    ("exe-sha256: " + (Get-Sha256 $exe)) | Add-Content (Join-Path $dir 'case.txt')
    # Old assertion: app.obj + app.ac-link.txt. PLAN-871: the shared-MIR
    # closure links per-unit COFFs, so no single object is published — the
    # receipt (with backend/mir axes) is the second product.
    if (-not (Test-Path (Join-Path $dir 'app.ac-link.txt'))) { throw "receipt missing" }
    $receiptText = Get-Content (Join-Path $dir 'app.ac-link.txt') -Raw
    if ($receiptText -notmatch 'backend: mn:') { throw "receipt lacks the mn backend axis" }
    if ($receiptText -notmatch 'mir-lowering: ') { throw "receipt lacks the mir-lowering axis" }
    if ($run.Code -ne 0) { throw "flagship run exited $($run.Code)" }
    if ($run.Stderr -ne '') { throw "flagship stderr '$($run.Stderr)'" }
    $caseSummary.Add([pscustomobject]@{ Case = 'SP01'; Exit = $run.Code; Stderr = ''; Ok = $true })
}

Invoke-Step 'positive SOURCE case: eval-order trace build + run (V-05/V-07)' {
    $dir = Join-Path $ReceiptsDir 'source-positive\SP02'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    # ctor fields declared left/right; source writes right then left with
    # mark_b/mark_a — the trace must be `ba` (source order), never `ab`
    Set-Content (Join-Path $dir 'order.at') "type Q {`n    a int`n    b int`n}`nfn main() int {`n    let q = Q { b: mark_b(), a: mark_a() }`n    if q.a != 1 { return 2 }`n    if q.b != 2 { return 3 }`n    return 0`n}`n"
    $exe = Join-Path $dir 'order.exe'
    $build = Invoke-Captured $script:Bin @('build-source', (Join-Path $dir 'order.at'), '--source-profile', $Record, '--entry', 'main', '--capability', $Trace, '--support-lib', $script:SupportLib, '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($build.Code -ne 0) { throw "order build exited $($build.Code): $($build.Stderr)" }
    $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
    ("exit: " + $run.Code + " stderr: " + $run.Stderr) | Set-Content (Join-Path $dir 'case.txt')
    if ($run.Code -ne 0) { throw "order run exited $($run.Code)" }
    if ($run.Stderr -ne 'ba') { throw "ctor order trace '$($run.Stderr)' expected 'ba'" }
    $caseSummary.Add([pscustomobject]@{ Case = 'SP02'; Exit = $run.Code; Stderr = $run.Stderr; Ok = $true })
}

Invoke-Step 'positive HIR case: record fixture build + run (V-02 HIR leg)' {
    $dir = Join-Path $ReceiptsDir 'hir-positive\HP01'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $srcp = Join-Path $RepoRoot 'experimental\ac-core\fixtures\record-hir\valid\r01-records.atom'
    ("input: " + $srcp) | Set-Content (Join-Path $dir 'case.txt')
    ("input-sha256: " + (Get-Sha256 $srcp)) | Add-Content (Join-Path $dir 'case.txt')
    $exe = Join-Path $dir 'app.exe'
    $build = Invoke-Captured $script:Bin @('build', $srcp, '--entry', 'd_main', '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($build.Code -ne 0) { throw "HIR build exited $($build.Code): $($build.Stderr)" }
    $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
    ("exit: " + $run.Code) | Add-Content (Join-Path $dir 'case.txt')
    if ($run.Code -ne 0) { throw "HIR run exited $($run.Code), expected 0 (happy path; 11/12 are failure codes)" }
    $caseSummary.Add([pscustomobject]@{ Case = 'HP01'; Exit = $run.Code; Stderr = ''; Ok = $true })
}

Invoke-Step 'HIR failure control: forced copy-check exits 11 (F-771-R1-04)' {
    $dir = Join-Path $ReceiptsDir 'hir-positive\HP02'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $srcp = Join-Path $RepoRoot 'experimental\ac-core\fixtures\record-hir\controls\r02-broken-copy-check.atom'
    ("input: " + $srcp) | Set-Content (Join-Path $dir 'case.txt')
    ("input-sha256: " + (Get-Sha256 $srcp)) | Add-Content (Join-Path $dir 'case.txt')
    $exe = Join-Path $dir 'app.exe'
    $build = Invoke-Captured $script:Bin @('build', $srcp, '--entry', 'd_main', '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($build.Code -ne 0) { throw "broken control build exited $($build.Code): $($build.Stderr)" }
    $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
    ("exit: " + $run.Code) | Add-Content (Join-Path $dir 'case.txt')
    if ($run.Code -ne 11) { throw "broken control exited $($run.Code), expected 11 (distinct from success 0)" }
    $caseSummary.Add([pscustomobject]@{ Case = 'HP02'; Exit = $run.Code; Stderr = ''; Ok = $true })
}

Invoke-Step 'positive HIR case: nested copy / aggregate return / aliasing (F-771-R1-01/04)' {
    $dir = Join-Path $ReceiptsDir 'hir-positive\HP03'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $srcp = Join-Path $RepoRoot 'experimental\ac-core\fixtures\record-hir\valid\r03-copy-return-alias.atom'
    ("input: " + $srcp) | Set-Content (Join-Path $dir 'case.txt')
    ("input-sha256: " + (Get-Sha256 $srcp)) | Add-Content (Join-Path $dir 'case.txt')
    $exe = Join-Path $dir 'app.exe'
    $build = Invoke-Captured $script:Bin @('build', $srcp, '--entry', 'd_main', '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($build.Code -ne 0) { throw "r03 build exited $($build.Code): $($build.Stderr)" }
    $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
    ("exit: " + $run.Code) | Add-Content (Join-Path $dir 'case.txt')
    if ($run.Code -ne 0) { throw "r03 exited $($run.Code), expected 0 (failure branches are 21..29)" }
    $caseSummary.Add([pscustomobject]@{ Case = 'HP03'; Exit = $run.Code; Stderr = ''; Ok = $true })
}

Invoke-Step 'positive SOURCE cases: aliased assignment value semantics (F-771-R1-01)' {
    # the assignment destination may alias storage the RHS still reads:
    # ctor self-assign and record-returning call self-assign must keep the
    # original reads intact; fresh destination is the accepted control
    $cases = @(
        @{ Name = 'alias-ctor'; Src = "type Q {`n    v int`n}`ntype P {`n    a Q`n    b Q`n}`nfn main() int {`n    var p = P { a: Q { v: 1 }, b: Q { v: 2 } }`n    p = P { a: p.b, b: p.a }`n    if p.a.v != 2 { return 11 }`n    if p.b.v != 1 { return 12 }`n    return 0`n}`n" },
        @{ Name = 'alias-sret'; Src = "type Q {`n    v int`n}`ntype P {`n    a Q`n    b Q`n}`nfn swap(p P) P { return P { a: p.b, b: p.a } }`nfn main() int {`n    var p = P { a: Q { v: 1 }, b: Q { v: 2 } }`n    p = swap(p)`n    if p.a.v != 2 { return 11 }`n    if p.b.v != 1 { return 12 }`n    return 0`n}`n" },
        @{ Name = 'fresh-control'; Src = "type Q {`n    v int`n}`ntype P {`n    a Q`n    b Q`n}`nfn swap(p P) P { return P { a: p.b, b: p.a } }`nfn main() int {`n    let p = P { a: Q { v: 1 }, b: Q { v: 2 } }`n    let q = swap(p)`n    if q.a.v != 2 { return 11 }`n    if q.b.v != 1 { return 12 }`n    if p.a.v != 1 { return 13 }`n    return 0`n}`n" }
    )
    foreach ($c in $cases) {
        $dir = Join-Path $ReceiptsDir ('source-positive\SP03-' + $c.Name)
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $srcp = Join-Path $dir 'case.at'
        Set-Content $srcp $c.Src
        $exe = Join-Path $dir 'case.exe'
        $build = Invoke-Captured $script:Bin @('build-source', $srcp, '--source-profile', $Record, '--entry', 'main', '--output', $exe) $RepoRoot -TimeoutSec 120
        if ($build.Code -ne 0) { throw "$($c.Name) build exited $($build.Code): $($build.Stderr)" }
        $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
        ("exit: " + $run.Code) | Set-Content (Join-Path $dir 'case.txt')
        if ($run.Code -ne 0) { throw "$($c.Name) exited $($run.Code), expected 0" }
        $caseSummary.Add([pscustomobject]@{ Case = 'SP03-' + $c.Name; Exit = $run.Code; Stderr = ''; Ok = $true })
    }
}

Invoke-Step 'positive SOURCE case: positional construction (F-771-R1-02)' {
    $dir = Join-Path $ReceiptsDir 'source-positive\SP04'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Set-Content (Join-Path $dir 'pos.at') "type Q {`n    v int`n}`ntype P {`n    a Q`n    b Q`n}`nfn main() int {`n    let q = Q(7)`n    if q.v != 7 { return 1 }`n    let p = P(Q(1), Q(2))`n    if p.a.v != 1 { return 2 }`n    if p.b.v != 2 { return 3 }`n    var t = P(Q(1), Q(2))`n    t = P(t.b, t.a)`n    if t.a.v != 2 { return 4 }`n    if t.b.v != 1 { return 5 }`n    return 0`n}`n"
    $exe = Join-Path $dir 'pos.exe'
    $build = Invoke-Captured $script:Bin @('build-source', (Join-Path $dir 'pos.at'), '--source-profile', $Record, '--entry', 'main', '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($build.Code -ne 0) { throw "positional build exited $($build.Code): $($build.Stderr)" }
    $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
    ("exit: " + $run.Code) | Set-Content (Join-Path $dir 'case.txt')
    if ($run.Code -ne 0) { throw "positional run exited $($run.Code)" }
    $caseSummary.Add([pscustomobject]@{ Case = 'SP04'; Exit = $run.Code; Stderr = ''; Ok = $true })
}

Invoke-Step 'positive SOURCE cases: ctor operand snapshot order (F-771-R2-01)' {
    # a record ctor operand is captured at its own source-order evaluation
    # point: the later `.mut` sibling must not retroactively change the
    # captured field; the explicit-save control distinguishes the oracle
    $cases = @(
        @{ Name = 'named-then-mut'; Src = "type Q {`n    v int`n}`ntype P {`n    a Q`n    b int`n}`nfn bump(mut q Q) int {`n    q.v = 2`n    return 0`n}`nfn main() int {`n    var q = Q(1)`n    let p = P { a: q, b: bump(q.mut) }`n    if q.v != 2 { return 10 }`n    if p.a.v != 1 { return 11 }`n    return 0`n}`n" },
        @{ Name = 'positional-then-mut'; Src = "type Q {`n    v int`n}`ntype P {`n    a Q`n    b int`n}`nfn bump(mut q Q) int {`n    q.v = 2`n    return 0`n}`nfn main() int {`n    var q = Q(1)`n    let p = P(q, bump(q.mut))`n    if q.v != 2 { return 10 }`n    if p.a.v != 1 { return 11 }`n    return 0`n}`n" },
        @{ Name = 'fresh-snapshot-control'; Src = "type Q {`n    v int`n}`ntype P {`n    a Q`n    b int`n}`nfn bump(mut q Q) int {`n    q.v = 2`n    return 0`n}`nfn main() int {`n    var q = Q(1)`n    let saved = q`n    let p = P { a: saved, b: bump(q.mut) }`n    if p.a.v != 1 { return 11 }`n    return 0`n}`n" }
    )
    foreach ($c in $cases) {
        $dir = Join-Path $ReceiptsDir ('source-positive\SP05-' + $c.Name)
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $srcp = Join-Path $dir 'case.at'
        Set-Content $srcp $c.Src
        $exe = Join-Path $dir 'case.exe'
        $build = Invoke-Captured $script:Bin @('build-source', $srcp, '--source-profile', $Record, '--entry', 'main', '--output', $exe) $RepoRoot -TimeoutSec 120
        if ($build.Code -ne 0) { throw "$($c.Name) build exited $($build.Code): $($build.Stderr)" }
        $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
        ("exit: " + $run.Code) | Set-Content (Join-Path $dir 'case.txt')
        if ($run.Code -ne 0) { throw "$($c.Name) exited $($run.Code), expected 0" }
        $caseSummary.Add([pscustomobject]@{ Case = 'SP05-' + $c.Name; Exit = $run.Code; Stderr = ''; Ok = $true })
    }
}

Invoke-Step 'positive HIR case: ctor operand snapshot order (F-771-R2-01)' {
    $dir = Join-Path $ReceiptsDir 'hir-positive\HP04'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $srcp = Join-Path $RepoRoot 'experimental\ac-core\fixtures\record-hir\valid\r04-ctor-snapshot-order.atom'
    ("input: " + $srcp) | Set-Content (Join-Path $dir 'case.txt')
    ("input-sha256: " + (Get-Sha256 $srcp)) | Add-Content (Join-Path $dir 'case.txt')
    $exe = Join-Path $dir 'app.exe'
    $build = Invoke-Captured $script:Bin @('build', $srcp, '--entry', 'd_main', '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($build.Code -ne 0) { throw "r04 build exited $($build.Code): $($build.Stderr)" }
    $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
    ("exit: " + $run.Code) | Add-Content (Join-Path $dir 'case.txt')
    if ($run.Code -ne 0) { throw "r04 exited $($run.Code), expected 0 (failure branches are 31..34)" }
    $caseSummary.Add([pscustomobject]@{ Case = 'HP04'; Exit = $run.Code; Stderr = ''; Ok = $true })
}

# Negative SOURCE corpus (V-03/V-06/V-07/V-08 + F-771-R1-02): id, fixture,
# code marker.
$SourceNegative = @(
    @{ Id = 'SN01'; File = 'i01-let-field-write.at'; Code = 'verify.immutable-write' },
    @{ Id = 'SN02'; File = 'i02-missing-dotmut.at'; Code = 'source.type-mismatch' },
    @{ Id = 'SN03'; File = 'i03-mut-fn.at'; Code = 'source.unsupported' },
    @{ Id = 'SN04'; File = 'i04-same-structure-swap.at'; Code = 'source.type-mismatch' },
    @{ Id = 'SN05'; File = 'i05-cycle.at'; Code = 'verify.record-cycle' },
    @{ Id = 'SN06'; File = 'i06-ctor-order-and-fields.at'; Code = 'source.duplicate' },
    @{ Id = 'SN07'; File = 'i07-shadows-type.at'; Code = 'source.duplicate' },
    @{ Id = 'SN08'; File = 'i08-positional-arity.at'; Code = 'source.argument' },
    @{ Id = 'SN09'; File = 'i09-positional-type.at'; Code = 'source.type-mismatch' },
    @{ Id = 'SN10'; File = 'i10-positional-named.at'; Code = 'source.argument' }
)

Invoke-Step 'negative SOURCE cases: check-source rejects (10)' {
    foreach ($c in $SourceNegative) {
        $srcp = Join-Path $RepoRoot ("experimental\ac-core\fixtures\record-source\invalid\" + $c.File)
        $r = Invoke-Captured $script:CheckBin @('check-source', $srcp, '--source-profile', $Record) $RepoRoot -TimeoutSec 60
        if ($r.Code -ne 1) { throw "$($c.Id): exited $($r.Code), expected 1" }
        if ($r.Stderr -notmatch [regex]::Escape($c.Code)) { throw "$($c.Id): expected $($c.Code), got $($r.Stderr)" }
        $caseSummary.Add([pscustomobject]@{ Case = $c.Id; Exit = $r.Code; Stderr = $c.Code; Ok = $true })
    }
}

Invoke-Step 'profile isolation (V-01: old profiles reject the record surface)' {
    $rec = Join-Path $RepoRoot 'experimental\ac-core\fixtures\record-source\valid\p01-records.at'
    foreach ($prof in @('auto.source.core-i32.draft', 'auto.source.scalar-i32.draft')) {
        $r = Invoke-Captured $script:CheckBin @('check-source', $rec, '--source-profile', $prof) $RepoRoot -TimeoutSec 60
        if ($r.Code -ne 1) { throw "$prof on record source exited $($r.Code), expected 1" }
        if ($r.Stderr -notmatch [regex]::Escape('text/source.unsupported')) { throw "expected source.unsupported: $($r.Stderr)" }
    }
    # unknown profile value stays a usage error (exit 2)
    $r = Invoke-Captured $script:CheckBin @('check-source', $rec, '--source-profile', 'auto.source.future.draft') $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 2) { throw "unknown profile exited $($r.Code), expected 2" }
    # record HIR document checks OK and reports the record domain
    $r = Invoke-Captured $script:CheckBin @('check', (Join-Path $RepoRoot 'experimental\ac-core\fixtures\record-hir\valid\r01-records.atom')) $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 0) { throw "record check exited $($r.Code): $($r.Stderr)" }
    if ($r.Stdout -notmatch 'hir-profile record-i32-draft') { throw "domain missing in: $($r.Stdout)" }
}

Invoke-Step 'SENTINEL transaction (V-10: failed build preserves artifacts)' {
    $dir = Join-Path $ReceiptsDir 'sentinel'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $srcp = Join-Path $RepoRoot 'experimental\ac-core\fixtures\record-source\valid\p01-records.at'
    $exe = Join-Path $dir 'app.exe'
    $rcpt = Join-Path $dir 'app.ac-link.txt'
    $build = Invoke-Captured $script:Bin @('build-source', $srcp, '--source-profile', $Record, '--entry', 'main', '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($build.Code -ne 0) { throw "sentinel seed build failed: $($build.Stderr)" }
    # PLAN-871: the transaction's products are exe + receipt (no single obj).
    foreach ($f in @($exe, $rcpt)) { if (-not (Test-Path $f)) { throw "sentinel missing $f" } }
    $h1 = Get-Sha256 $exe; $h3 = Get-Sha256 $rcpt
    # now a failing build into the SAME output: artifacts must survive
    $bad = Join-Path $dir 'bad.at'
    Set-Content $bad "type P { v int }`nfn main() int {`n    let p = P { v: 1 }`n    p.v = 2`n    return p.v`n}`n"
    $fail = Invoke-Captured $script:Bin @('build-source', $bad, '--source-profile', $Record, '--entry', 'main', '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($fail.Code -ne 1) { throw "bad build exited $($fail.Code), expected 1" }
    if ((Get-Sha256 $exe) -ne $h1) { throw "exe hash changed after failed build" }
    if ((Get-Sha256 $rcpt) -ne $h3) { throw "receipt hash changed after failed build" }
    @("exe: $h1", "receipt: $h3") | Set-Content (Join-Path $dir 'sentinel.txt')
}

Invoke-Step 'receipt identity lines (V-10)' {
    $dir = Join-Path $ReceiptsDir 'identity'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $srcp = Join-Path $RepoRoot 'experimental\ac-core\fixtures\record-source\valid\p01-records.at'
    $exe = Join-Path $dir 'app.exe'
    $r = Invoke-Captured $script:Bin @('build-source', $srcp, '--source-profile', $Record, '--entry', 'main', '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($r.Code -ne 0) { throw "build exited $($r.Code): $($r.Stderr)" }
    $receipt = Get-Content (Join-Path $dir 'app.ac-link.txt') -Raw
    foreach ($needle in @(
        'source-profile: auto.source.record.draft',
        'hir-schema: auto.hir.record.draft',
        'hir-revision: 1u32',
        'hir-profile: record-i32-draft',
        'source-input-hash: '
    )) {
        if ($receipt -notmatch [regex]::Escape($needle)) { throw "receipt missing '$needle': $receipt" }
    }
    $receipt | Set-Content (Join-Path $dir 'receipt.txt')
}

# ---- 4. old-profile regression gates (V-10: old surfaces unchanged) ----
Invoke-Step 'old gate 741 (core-i32)' {
    $r = Invoke-Captured 'powershell' @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $RepoRoot 'scripts\verify-ac-741.ps1'), '-SkipMainGates') $RepoRoot -TimeoutSec 900
    if ($r.Code -ne 0) { throw "741 gate exited $($r.Code): $($r.Stdout)$($r.Stderr)" }
}
Invoke-Step 'old gate 745 (source core-i32)' {
    $r = Invoke-Captured 'powershell' @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $RepoRoot 'scripts\verify-ac-source-745.ps1')) $RepoRoot -TimeoutSec 900
    if ($r.Code -ne 0) { throw "745 gate exited $($r.Code): $($r.Stdout)$($r.Stderr)" }
}
Invoke-Step 'old gate 767 (scalar)' {
    $r = Invoke-Captured 'powershell' @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $RepoRoot 'scripts\verify-ac-scalar-767.ps1')) $RepoRoot -TimeoutSec 1200
    if ($r.Code -ne 0) { throw "767 gate exited $($r.Code): $($r.Stdout)$($r.Stderr)" }
}

# ---- 5. summary ----
$script:Results | Format-Table -AutoSize | Out-String | Set-Content (Join-Path $ReceiptsDir 'summary.txt')
$caseSummary | Format-Table -AutoSize | Out-String | Set-Content (Join-Path $ReceiptsDir 'cases.txt')
$total = $script:Results.Count
$failed = @($script:Results | Where-Object { -not $_.Ok }).Count
Write-Host ("record-771 gate: {0} steps, {1} failed; {2} cases" -f $total, $failed, $caseSummary.Count)
if ($failed -gt 0) { exit 1 }
exit 0
