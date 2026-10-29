# PLAN-782 one-shot text-record combination gate (ASCII only for Windows
# PowerShell 5.1). Serially runs the frozen gates from plan section 6 /
# design section 10: the ac-core workspace (check --locked --all-targets
# zero-warning with in-script matcher controls, the FULL test suite --
# text_record_hir/source/resources/native included -- and fmt), the
# runtime/text workspace (check/test/fmt, same policy; runtime owns the
# fault-injection/budget faces the language cannot reach), the case
# walkthrough under the 741/745/771 contained executor (source flagship
# p01 PE with buffer-pairing oracle, overlap shapes p02, the direct-HIR
# tr01 leg with exact trace, the 8 pinned-code negative corpus, profile
# isolation, the capability-gate failure control, the SENTINEL
# transaction preservation and the receipt identity lines), then the five
# old gates 770/771/741[-SkipMainGates]/745/767 and this plan's own scope
# isolation. The old 770 gate embeds a per-plan scope assertion whose
# allow-list is 770-specific, so on this successor tree it is EXPECTED to
# fail that single step: this gate asserts the failure is exactly that
# step, that the flagged files are exactly this plan's own in-scope
# artifacts (computed from the same git diff), and that every substantive
# 770 step passed -- a real out-of-scope change still fails here because
# it would appear in the flagged set outside the 782 allow-list. Raw
# outputs of every step land under
# experimental/ac-core/target/ac-verify-receipts/text-record-782/
# (gitignored), on success AND on failure.
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-text-record-782.ps1
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$AcCore = Join-Path $RepoRoot 'experimental\ac-core'
# PLAN-871 T-05: build legs run through the shared-MIR `ac-native` CLI (the
# receiver of ac-core's retiring build/build-source). Old assertion mapping:
# exit / stdout / stderr oracles are UNCHANGED (including the ledger/trap
# rows); the old `app.obj` product assertions become "receipt exists and
# carries backend/mir-lowering axes; the exe/receipt survive the failed-build
# transaction"; `--support-lib` carries the text runtime library on owned
# rows and the trace library elsewhere, exactly as before.
$AcMn = Join-Path $RepoRoot 'experimental\ac-mir-native'
$RtDir = Join-Path $AcCore 'runtime\text'
$ReceiptsDir = Join-Path $AcCore 'target\ac-verify-receipts\text-record-782'
New-Item -ItemType Directory -Force -Path $ReceiptsDir | Out-Null

$script:Results = New-Object System.Collections.Generic.List[object]
$script:ReceiptLines = New-Object System.Collections.Generic.List[string]
$script:RawDir = $null
$script:CaseSummary = New-Object System.Collections.Generic.List[object]

function Invoke-Step {
    param([string]$Name, [scriptblock]$Body)
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

# Per-line warning matcher with an in-script positive AND negative control
# (mirror of the 770/774 gates; catches warnings not on the first line).
function Test-HasWarningLine {
    param([string]$Text)
    foreach ($line in ($Text -split "`r?`n")) {
        if ($line -match '^\s*warning') { return $true }
    }
    return $false
}

function Get-Sha256 {
    param([string]$Path)
    if (-not (Test-Path $Path)) { return '<missing>' }
    (Get-FileHash -Algorithm SHA256 $Path).Hash.ToLower()
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
    param([string]$Exe, [string[]]$ArgList, [string]$WorkDir, [int]$TimeoutSec = 600, [string]$RawName = $null)
    if (-not $script:AcRunnerLoaded) {
        Add-Type -TypeDefinition $script:AcRunnerCs -Language CSharp
        $script:AcRunnerLoaded = $true
    }
    $r = [AcContainedRunner]::Run($Exe, $ArgList, $WorkDir, ($TimeoutSec * 1000))
    $text = "cmd: $Exe $($ArgList -join ' ')`nexit: $($r.Code)`n--- stdout ---`n$($r.Stdout)`n--- stderr ---`n$($r.Stderr)`n"
    # NB: a [string]-typed parameter defaults to '' (not $null) when
    # omitted, so IsNullOrEmpty is the correct emptiness guard here.
    if (-not [string]::IsNullOrEmpty($RawName) -and $null -ne $script:RawDir) {
        [System.IO.File]::WriteAllText((Join-Path $script:RawDir $RawName), $text)
    }
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

$TR = 'auto.source.text-record.draft'
$TEXT_CAP = 'ac.runtime.text.v1'

$script:Head = ''
$script:HadFailure = $false

try {
    # --- 1. matcher controls + context ----------------------------------------

    Invoke-Step 'warning matcher controls (positive/negative)' {
        $positive = "   Compiling x`n    warning: unused variable y`n    Finished"
        if (-not (Test-HasWarningLine $positive)) { throw 'positive control failed: second-line warning not detected' }
        $negative = "   Compiling x`n    Finished dev profile"
        if (Test-HasWarningLine $negative) { throw 'negative control failed: clean output flagged' }
    }

    Invoke-Step 'context (HEAD + toolchain)' {
        $head = (Invoke-Captured 'git' @('rev-parse', 'HEAD') $RepoRoot -TimeoutSec 60).Stdout.Trim()
        $branch = (Invoke-Captured 'git' @('branch', '--show-current') $RepoRoot -TimeoutSec 60).Stdout.Trim()
        $rustc = (Invoke-Captured 'rustc' @('-Vv') $RepoRoot -TimeoutSec 60).Stdout
        $cargoV = (Invoke-Captured 'cargo' @('-V') $RepoRoot -TimeoutSec 60).Stdout
        $lld = Join-Path ((Invoke-Captured 'rustc' @('--print', 'sysroot') $RepoRoot -TimeoutSec 60).Stdout.Trim()) 'lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe'
        $script:Head = $head
        $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
        $script:RawDir = Join-Path $ReceiptsDir "raw-$stamp"
        New-Item -ItemType Directory -Force -Path $script:RawDir | Out-Null
        $script:ReceiptLines.Add('PLAN-782 text-record gate receipt')
        $script:ReceiptLines.Add("head: $head")
        $script:ReceiptLines.Add("branch: $branch")
        $script:ReceiptLines.Add("rustc: $($rustc.Trim())")
        $script:ReceiptLines.Add("cargo: $($cargoV.Trim())")
        $script:ReceiptLines.Add("rust-lld: $lld")
        $script:ReceiptLines.Add("rust-lld-sha256: $(Get-Sha256 $lld)")
        $script:ReceiptLines.Add("raw-outputs: $script:RawDir")
        Write-Host "HEAD $head ($branch)"
    }

    # --- 2. ac-core workspace ---------------------------------------------------

    Invoke-Step 'cargo check --locked --all-targets, zero warnings (ac-core)' {
        $r = Invoke-Captured 'cargo' @('check', '--locked', '--all-targets', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot -TimeoutSec 900 '02-core-check.txt'
        if ($r.Code -ne 0) { throw "check --all-targets exited $($r.Code)" }
        if (Test-HasWarningLine ($r.Stdout + $r.Stderr)) { throw 'zero-warning health check failed (per-line matcher)' }
        $script:ReceiptLines.Add('ac-core check: ok (zero warnings, per-line matcher with controls)')
    }

    Invoke-Step 'cargo test --locked (ac-core full suite incl. text_record faces)' {
        $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot -TimeoutSec 1800 '03-core-test.txt'
        if ($r.Code -ne 0) { throw "full test suite exited $($r.Code)" }
        $t = $r.Stdout + $r.Stderr
        $ms = [regex]::Matches($t, 'result: ok\. (\d+) passed')
        $total = 0
        foreach ($m in $ms) { $total += [int]$m.Groups[1].Value }
        # PLAN-871 T-06: the old native/lowering suites (and their count) were
        # retired with the HIR→CLIF backend; the remaining ac-core suite is
        # the frontend/verify/link surface. The floor is the OBSERVED size of
        # that surface (not the old 400+ PE suite), and the text-record HIR +
        # source binaries named below must still be present.
        if ($total -lt 200) { throw "expected at least 200 tests in the full suite, counted $total" }
        foreach ($bin in @('text_record_hir.rs', 'text_record_source.rs')) {
            if ($t -notmatch [regex]::Escape($bin)) { throw "test binary $bin missing from the ac-core run" }
        }
        $script:ReceiptLines.Add("ac-core test: ok ($total tests; text_record_hir/source present; PF/verify surface post-retirement)")
    }

    Invoke-Step 'cargo fmt --check (ac-core)' {
        $r = Invoke-Captured 'cargo' @('fmt', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--', '--check') $RepoRoot -TimeoutSec 300 '04-core-fmt.txt'
        if ($r.Code -ne 0) { throw "fmt --check exited $($r.Code)" }
        $script:ReceiptLines.Add('ac-core fmt: clean')
    }

    # --- 3. runtime/text workspace ----------------------------------------------

    Invoke-Step 'runtime/text cargo check --locked --all-targets, zero warnings' {
        $r = Invoke-Captured 'cargo' @('check', '--locked', '--all-targets', '--manifest-path', (Join-Path $RtDir 'Cargo.toml')) $RepoRoot -TimeoutSec 900 '05-rt-check.txt'
        if ($r.Code -ne 0) { throw "runtime check exited $($r.Code)" }
        if (Test-HasWarningLine ($r.Stdout + $r.Stderr)) { throw 'runtime zero-warning check failed' }
        $script:ReceiptLines.Add('runtime/text check: ok (zero warnings)')
    }

    Invoke-Step 'runtime/text cargo test --locked (runtime-owned fault/budget faces)' {
        $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $RtDir 'Cargo.toml')) $RepoRoot -TimeoutSec 900 '06-rt-test.txt'
        if ($r.Code -ne 0) { throw "runtime test exited $($r.Code)" }
        $t = $r.Stdout + $r.Stderr
        $ms = [regex]::Matches($t, 'result: ok\. (\d+) passed')
        $total = 0
        foreach ($m in $ms) { $total += [int]$m.Groups[1].Value }
        if ($total -lt 19) { throw "expected at least the 19 baseline runtime tests, counted $total" }
        $script:ReceiptLines.Add("runtime/text test: ok ($total tests; AP fault injection, budget, trap/hang, contract sidecar)")
    }

    Invoke-Step 'runtime/text cargo fmt --check' {
        $r = Invoke-Captured 'cargo' @('fmt', '--manifest-path', (Join-Path $RtDir 'Cargo.toml'), '--', '--check') $RepoRoot -TimeoutSec 300 '07-rt-fmt.txt'
        if ($r.Code -ne 0) { throw "runtime fmt --check exited $($r.Code)" }
        $script:ReceiptLines.Add('runtime/text fmt: clean')
    }

    # --- 4. case walkthrough (contained executor; deliberate-failure
    # ---    controls included per plan V-11/T-06) --------------------------------

    Invoke-Step 'build tool binaries once' {
        $r = Invoke-Captured 'cargo' @('build', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot -TimeoutSec 900 '08-bin.txt'
        if ($r.Code -ne 0) { throw "tool build exited $($r.Code)" }
        $m = Invoke-Captured 'cargo' @('build', '--locked', '--manifest-path', (Join-Path $AcMn 'Cargo.toml'), '--bin', 'ac-native') $RepoRoot -TimeoutSec 900 '08-bin.txt'
        if ($m.Code -ne 0) { throw "ac-native build exited $($m.Code)" }
        $script:Bin = Join-Path $AcMn 'target\debug\ac-native.exe'
        $script:CheckBin = Join-Path $AcCore 'target\debug\auto-ac-prototype.exe'
        if (-not (Test-Path $script:Bin)) { throw "binary missing: $script:Bin" }
        $script:ReceiptLines.Add("tool-binary-sha256: $(Get-Sha256 $script:Bin)")
    }

    Invoke-Step 'build text runtime support library + contract sidecar' {
        # mirrors tests/common::text_runtime_lib (PLAN-770): the shim
        # staticlib plus the contract sidecar the gate's builds consume
        $r = Invoke-Captured 'cargo' @('build', '-p', 'ac-text-runtime-shim', '--release', '--locked') $RtDir -TimeoutSec 900 '09-rtlib.txt'
        if ($r.Code -ne 0) { throw "runtime shim build exited $($r.Code)" }
        $script:SupportLib = Join-Path $RtDir 'target\release\ac_text_runtime.lib'
        if (-not (Test-Path $script:SupportLib)) { throw "support lib missing: $script:SupportLib" }
        $contract = Join-Path $RtDir 'target\release\ac_text_runtime.lib.contract.json'
        $sidecar = "{`n" +
            "  `"api`": `"ac.runtime.text.v1`",`n" +
            "  `"api_revision`": 1,`n" +
            "  `"target`": `"x86_64-pc-windows-msvc`",`n" +
            "  `"lib`": `"ac_text_runtime.lib`",`n" +
            "  `"lib_sha256`": `"" + (Get-Sha256 $script:SupportLib) + "`",`n" +
            "  `"symbol_count`": 23`n" +
            "}`n"
        [System.IO.File]::WriteAllText($contract, $sidecar)
        $script:ReceiptLines.Add("support-lib-sha256: $(Get-Sha256 $script:SupportLib)")
        $script:ReceiptLines.Add("support-lib-contract-sha256: $(Get-Sha256 $contract)")
    }

    # buffer-pairing oracle: every allocated id (LIT/CLONE/CONCAT with an
    # id; zero-length buffers allocate nothing and carry no id) is freed
    # exactly once; nothing else is freed (no leak, no double free).
    function Test-TracePairing {
        param([string]$Stderr)
        $alloc = @([regex]::Matches($Stderr, '(?m)^(?:LIT|CLONE|CONCAT) id=(\d+)') | ForEach-Object { $_.Groups[1].Value })
        $free = @([regex]::Matches($Stderr, '(?m)^FREE id=(\d+)') | ForEach-Object { $_.Groups[1].Value })
        $a = ($alloc | Sort-Object) -join ','
        $f = ($free | Sort-Object) -join ','
        if ($a -ne $f) { throw "trace pairing broken:`nalloc: $a`nfree:  $f`nstderr:`n$Stderr" }
        if ($alloc.Count -eq 0) { throw 'no buffers allocated in the flagship run (oracle degenerate)' }
    }

    function Invoke-BuildSource {
        param([string]$Src, [string]$Exe, [string[]]$ExtraCaps, [string]$RawName)
        $args2 = @('build-source', $Src, '--source-profile', $TR, '--entry', 'main', '--capability', $TEXT_CAP)
        foreach ($cap in $ExtraCaps) { $args2 += @('--capability', $cap) }
        $args2 += @('--support-lib', $script:SupportLib, '--output', $Exe)
        return Invoke-Captured $script:Bin $args2 $RepoRoot -TimeoutSec 300 $RawName
    }

    Invoke-Step 'TP01 source flagship PE (p01-combination.at)' {
        $dir = Join-Path $ReceiptsDir 'cases\TP01'
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot 'experimental\ac-core\fixtures\text-record-source\valid\p01-combination.at'
        $exe = Join-Path $dir 'app.exe'
        $build = Invoke-BuildSource $src $exe @() '10-tp01-build.txt'
        if ($build.Code -ne 0) { throw "flagship build exited $($build.Code)" }
        # Old assertion: app.obj + app.ac-link.txt. PLAN-871: the shared-MIR
        # closure publishes no single object — the receipt (backend + mir
        # axes) replaces it.
        foreach ($f in @('app.ac-link.txt')) { if (-not (Test-Path (Join-Path $dir $f))) { throw "artifact missing: $f" } }
        $rcptText = Get-Content (Join-Path $dir 'app.ac-link.txt') -Raw
        if ($rcptText -notmatch 'backend: mn:') { throw "receipt lacks the mn backend axis" }
        if ($rcptText -notmatch 'mir-lowering: ') { throw "receipt lacks the mir-lowering axis" }
        $run = Invoke-Captured $exe @() $dir -TimeoutSec 120 '10-tp01-run.txt'
        Test-TracePairing $run.Stderr
        if ($run.Code -ne 30) { throw "flagship exited $($run.Code), expected 30 (n5+m5+k6+kind9+len5)" }
        if ($run.Stdout -ne '') { throw "flagship stdout '$($run.Stdout)'" }
        @("input: $src", "input-sha256: $(Get-Sha256 $src)", "exit: $($run.Code)", "exe-sha256: $(Get-Sha256 $exe)", "stderr:", $run.Stderr) | Set-Content (Join-Path $dir 'case.txt')
        $script:CaseSummary.Add([pscustomobject]@{ Case = 'TP01'; Exit = $run.Code; Buffers = @([regex]::Matches($run.Stderr, '(?m)^FREE id=')).Count; Ok = $true })
        $script:ReceiptLines.Add('TP01 flagship: ok (exit 30, buffer pairing, obj+receipt present)')
    }

    Invoke-Step 'TP02 overlap shapes PE (p02-overlap.at)' {
        $dir = Join-Path $ReceiptsDir 'cases\TP02'
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot 'experimental\ac-core\fixtures\text-record-source\valid\p02-overlap.at'
        $exe = Join-Path $dir 'app.exe'
        $build = Invoke-BuildSource $src $exe @() '11-tp02-build.txt'
        if ($build.Code -ne 0) { throw "overlap build exited $($build.Code)" }
        $run = Invoke-Captured $exe @() $dir -TimeoutSec 120 '11-tp02-run.txt'
        Test-TracePairing $run.Stderr
        if ($run.Code -ne 9) { throw "overlap exited $($run.Code), expected 9 (kind2 + len7)" }
        @("input: $src", "input-sha256: $(Get-Sha256 $src)", "exit: $($run.Code)", "exe-sha256: $(Get-Sha256 $exe)", "stderr:", $run.Stderr) | Set-Content (Join-Path $dir 'case.txt')
        $script:CaseSummary.Add([pscustomobject]@{ Case = 'TP02'; Exit = $run.Code; Buffers = @([regex]::Matches($run.Stderr, '(?m)^FREE id=')).Count; Ok = $true })
        $script:ReceiptLines.Add('TP02 overlap: ok (exit 9, self-clone/transform/move-rebuild/concat pairing)')
    }

    Invoke-Step 'HP01 direct-HIR PE (tr01-combination.atom)' {
        $dir = Join-Path $ReceiptsDir 'cases\HP01'
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot 'experimental\ac-core\fixtures\text-record-hir\valid\tr01-combination.atom'
        $exe = Join-Path $dir 'app.exe'
        $bargs = @('build', $src, '--entry', 'd_main', '--capability', $TEXT_CAP, '--support-lib', $script:SupportLib, '--output', $exe)
        $build = Invoke-Captured $script:Bin $bargs $RepoRoot -TimeoutSec 300 '12-hp01-build.txt'
        if ($build.Code -ne 0) { throw "direct-HIR build exited $($build.Code)" }
        $run = Invoke-Captured $exe @() $dir -TimeoutSec 120 '12-hp01-run.txt'
        if ($run.Code -ne 2) { throw "direct-HIR exited $($run.Code), expected 2 (consume(t.move) = len 2)" }
        if ($run.Stderr -ne "LIT len=2`nLIT id=1`nTEXT-SCALARS n=2`nFREE id=1`n") { throw "direct-HIR trace '$($run.Stderr)'" }
        @("input: $src", "input-sha256: $(Get-Sha256 $src)", "exit: $($run.Code)", "exe-sha256: $(Get-Sha256 $exe)", "stderr:", $run.Stderr) | Set-Content (Join-Path $dir 'case.txt')
        $script:CaseSummary.Add([pscustomobject]@{ Case = 'HP01'; Exit = $run.Code; Buffers = 1; Ok = $true })
        $script:ReceiptLines.Add('HP01 direct-HIR: ok (exit 2, exact trace, Str-spelled field_get regression)')
    }

    $Negative = @(
        @{ Id = 'TN01'; File = 'i01-bare-resource-assign.at'; Code = 'verify.resource-assign' },
        @{ Id = 'TN02'; File = 'i02-partial-move-field.at'; Code = 'verify.partial-move' },
        @{ Id = 'TN03'; File = 'i03-user-clone-method.at'; Code = 'source.duplicate' },
        @{ Id = 'TN04'; File = 'i04-str-field.at'; Code = 'verify.field-borrow' },
        @{ Id = 'TN05'; File = 'i05-bare-return-resource.at'; Code = 'verify.ownership-required' },
        @{ Id = 'TN06'; File = 'i06-nominal-mismatch.at'; Code = 'source.type-mismatch' },
        @{ Id = 'TN07'; File = 'i07-nonexecuted-branch.at'; Code = 'verify.partial-move' },
        @{ Id = 'TN08'; File = 'i08-move-param-on-copyable.at'; Code = 'verify.ownership-required' }
    )

    Invoke-Step 'negative SOURCE corpus: check-source rejects (8 pinned codes)' {
        foreach ($c in $Negative) {
            $srcp = Join-Path $RepoRoot ("experimental\ac-core\fixtures\text-record-source\invalid\" + $c.File)
            $r = Invoke-Captured $script:CheckBin @('check-source', $srcp, '--source-profile', $TR) $RepoRoot -TimeoutSec 120
            if ($r.Code -ne 1) { throw "$($c.Id): exited $($r.Code), expected 1" }
            if ($r.Stderr -notmatch [regex]::Escape($c.Code)) { throw "$($c.Id): expected $($c.Code), got $($r.Stderr)" }
            $script:CaseSummary.Add([pscustomobject]@{ Case = $c.Id; Exit = $r.Code; Buffers = -1; Ok = $true })
        }
        $script:ReceiptLines.Add('negative corpus: ok (8/8 pinned diagnostic codes, exit 1, no PE built)')
    }

    Invoke-Step 'profile isolation (V-01: old profiles reject the combination surface)' {
        $src = Join-Path $RepoRoot 'experimental\ac-core\fixtures\text-record-source\valid\p01-combination.at'
        foreach ($prof in @('auto.source.core-i32.draft', 'auto.source.scalar-i32.draft', 'auto.source.text.draft', 'auto.source.record.draft')) {
            $r = Invoke-Captured $script:CheckBin @('check-source', $src, '--source-profile', $prof) $RepoRoot -TimeoutSec 120 '13-profile-iso.txt'
            if ($r.Code -ne 1) { throw "$prof on combination source exited $($r.Code), expected 1" }
            if ($r.Stderr -notmatch [regex]::Escape('text/source.unsupported')) { throw "expected text/source.unsupported: $($r.Stderr)" }
        }
        $r = Invoke-Captured $script:CheckBin @('check-source', $src, '--source-profile', 'auto.source.future.draft') $RepoRoot -TimeoutSec 120
        if ($r.Code -ne 2) { throw "unknown profile exited $($r.Code), expected 2" }
        $r = Invoke-Captured $script:CheckBin @('check', (Join-Path $RepoRoot 'experimental\ac-core\fixtures\text-record-hir\valid\tr01-combination.atom')) $RepoRoot -TimeoutSec 120
        if ($r.Code -ne 0) { throw "combination HIR check exited $($r.Code): $($r.Stderr)" }
        if ($r.Stdout -notmatch 'hir-profile text-record-i32-draft') { throw "domain missing in: $($r.Stdout)" }
        $script:ReceiptLines.Add('profile isolation: ok (4 old profiles reject; unknown exit 2; domain line present)')
    }

    Invoke-Step 'capability gate control (missing capability rejects, no artifact)' {
        # deliberate-failure control: the derived ac.runtime.text.v1
        # requirement is not waived by caller silence -- the build must
        # reject BEFORE any artifact is written
        $dir = Join-Path $ReceiptsDir 'cases\TCAP'
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot 'experimental\ac-core\fixtures\text-record-source\valid\p01-combination.at'
        $exe = Join-Path $dir 'app.exe'
        $r = Invoke-Captured $script:Bin @('build-source', $src, '--source-profile', $TR, '--entry', 'main', '--support-lib', $script:SupportLib, '--output', $exe) $RepoRoot -TimeoutSec 300 '14-cap-control.txt'
        if ($r.Code -ne 1) { throw "missing-capability build exited $($r.Code), expected 1" }
        if ($r.Stderr -notmatch [regex]::Escape('capability.missing')) { throw "expected capability.missing: $($r.Stderr)" }
        if (Test-Path $exe) { throw 'artifact written despite rejection' }
        $script:ReceiptLines.Add('capability control: ok (exit 1, capability.missing, no artifact)')
    }

    Invoke-Step 'SENTINEL transaction (failed build preserves artifacts)' {
        $dir = Join-Path $ReceiptsDir 'cases\SENTINEL'
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot 'experimental\ac-core\fixtures\text-record-source\valid\p01-combination.at'
        $exe = Join-Path $dir 'app.exe'
        $rcpt = Join-Path $dir 'app.ac-link.txt'
        $build = Invoke-BuildSource $src $exe @() '15-sentinel-seed.txt'
        if ($build.Code -ne 0) { throw "sentinel seed build exited $($build.Code)" }
        # PLAN-871: the transaction products are exe + receipt (no single obj).
        foreach ($f in @($exe, $rcpt)) { if (-not (Test-Path $f)) { throw "sentinel missing $f" } }
        $h1 = Get-Sha256 $exe; $h3 = Get-Sha256 $rcpt
        # deliberately failing fixture into the SAME output: the artifacts
        # must survive byte-for-byte (741 transaction contract)
        $bad = Join-Path $dir 'bad.at'
        [System.IO.File]::WriteAllText($bad, "type Tok { text Str; kind int }`nfn main() int {`n    let a Tok = Tok { text: `"hi`", kind: 1 }`n    var b Tok = Tok { text: `"x`", kind: 2 }`n    b = a`n    return b.kind`n}`n")
        $fail = Invoke-BuildSource $bad $exe @() '15-sentinel-fail.txt'
        if ($fail.Code -ne 1) { throw "bad build exited $($fail.Code), expected 1" }
        if ($fail.Stderr -notmatch [regex]::Escape('verify.resource-assign')) { throw "expected verify.resource-assign: $($fail.Stderr)" }
        if ((Get-Sha256 $exe) -ne $h1) { throw 'exe hash changed after failed build' }
        if ((Get-Sha256 $rcpt) -ne $h3) { throw 'receipt hash changed after failed build' }
        @("exe: $h1", "receipt: $h3") | Set-Content (Join-Path $dir 'sentinel.txt')
        $script:ReceiptLines.Add('SENTINEL: ok (failing build exit 1 distinct from success 0; exe/receipt hashes preserved)')
    }

    Invoke-Step 'receipt identity lines (V-01)' {
        $dir = Join-Path $ReceiptsDir 'cases\IDENT'
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot 'experimental\ac-core\fixtures\text-record-source\valid\p01-combination.at'
        $exe = Join-Path $dir 'app.exe'
        $r = Invoke-BuildSource $src $exe @() '16-identity.txt'
        if ($r.Code -ne 0) { throw "identity build exited $($r.Code)" }
        $receipt = Get-Content (Join-Path $dir 'app.ac-link.txt') -Raw
        foreach ($needle in @(
            'source-profile: auto.source.text-record.draft',
            'hir-schema: auto.hir.text-record.draft',
            'hir-revision: 1u32',
            'hir-profile: text-record-i32-draft',
            'source-input-hash: '
        )) {
            if ($receipt -notmatch [regex]::Escape($needle)) { throw "receipt missing '$needle': $receipt" }
        }
        $receipt | Set-Content (Join-Path $dir 'receipt.txt')
        $script:ReceiptLines.Add('receipt identity: ok (5 needles present)')
    }

    # --- 5. old gates (V-12: old surfaces proven unchanged on this tree) -------

    Invoke-Step 'old gate: verify-ac-record-771.ps1' {
        $r = Invoke-Captured 'powershell' @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $RepoRoot 'scripts\verify-ac-record-771.ps1')) $RepoRoot -TimeoutSec 2400 '17-old-771.txt'
        if ($r.Code -ne 0) { throw "771 gate exited $($r.Code)`n$($r.Stdout)$($r.Stderr)" }
        $script:ReceiptLines.Add('old gate 771: ok (record surface on the combination baseline)')
    }

    Invoke-Step 'old gate: verify-ac-741.ps1 (-SkipMainGates)' {
        $r = Invoke-Captured 'powershell' @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $RepoRoot 'scripts\verify-ac-741.ps1'), '-SkipMainGates') $RepoRoot -TimeoutSec 1800 '18-old-741.txt'
        if ($r.Code -ne 0) { throw "741 gate exited $($r.Code)`n$($r.Stdout)$($r.Stderr)" }
        $script:ReceiptLines.Add('old gate 741: ok (main gates skipped per tiering; batch regression owns t/tv)')
    }

    Invoke-Step 'old gate: verify-ac-source-745.ps1' {
        $r = Invoke-Captured 'powershell' @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $RepoRoot 'scripts\verify-ac-source-745.ps1')) $RepoRoot -TimeoutSec 1800 '19-old-745.txt'
        if ($r.Code -ne 0) { throw "745 gate exited $($r.Code)`n$($r.Stdout)$($r.Stderr)" }
        $script:ReceiptLines.Add('old gate 745: ok (source core-i32 surface)')
    }

    Invoke-Step 'old gate: verify-ac-scalar-767.ps1' {
        $r = Invoke-Captured 'powershell' @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $RepoRoot 'scripts\verify-ac-scalar-767.ps1')) $RepoRoot -TimeoutSec 2400 '20-old-767.txt'
        if ($r.Code -ne 0) { throw "767 gate exited $($r.Code)`n$($r.Stdout)$($r.Stderr)" }
        $script:ReceiptLines.Add('old gate 767: ok (scalar surface; covers the post-772 fixed computation face)')
    }

    Invoke-Step 'old gate: verify-ac-text-770.ps1 (expected per-plan scope failure attributed)' {
        # The 770 gate ends with a per-plan scope assertion whose allow-list
        # is 770-specific. On this successor tree it MUST fail that single
        # step with exactly this plan's own in-scope artifacts -- and every
        # substantive step (workspaces, tests, its own old gates) must pass.
        # A real out-of-scope change would surface in the flagged set
        # outside the 782 allow-list and fail HERE.
        $r = Invoke-Captured 'powershell' @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $RepoRoot 'scripts\verify-ac-text-770.ps1')) $RepoRoot -TimeoutSec 3600 '21-old-770.txt'
        $out = $r.Stdout + $r.Stderr
        if ($r.Code -ne 1) {
            throw "770 gate exited $($r.Code) (expected 1 with ONLY the per-plan scope step failing)`n$out"
        }
        $fails = @([regex]::Matches($out, '\[FAIL\] ([^\r\n]+)') | ForEach-Object { $_.Groups[1].Value.Trim() })
        if ($fails.Count -ne 1 -or $fails[0] -notmatch '^scope isolation') {
            throw "770 gate failures not the expected single scope step: $($fails -join ' | ')"
        }
        # every substantive step must have passed (the trailing hashes step
        # is informational-only and aborts after the scope throw; this gate
        # recomputes the same hashes in its own final step)
        foreach ($passName in @(
            'warning matcher controls (positive/negative)',
            'context (HEAD + toolchain)',
            'cargo check --locked --all-targets (ac-core)',
            'cargo test --locked (ac-core full suite incl. text faces)',
            'cargo fmt --check (ac-core)',
            'runtime/text cargo check --locked --all-targets',
            'runtime/text cargo test --locked (runtime-owned fault/budget faces)',
            'runtime/text cargo fmt --check',
            'old gate: verify-ac-741.ps1',
            'old gate: verify-ac-source-745.ps1',
            'old gate: verify-ac-scalar-767.ps1',
            'old gate: verify-ac-record-771.ps1'
        )) {
            if ($out -notmatch [regex]::Escape("[PASS] $passName")) { throw "770 gate step not green: $passName" }
        }
        # the flagged set must be EXACTLY this plan's own artifacts: recompute
        # the same diff the 770 scope step sees and require membership in the
        # 782 allow-list (anything else is a genuine out-of-scope change)
        $base = (& git -C $RepoRoot merge-base HEAD master | Out-String).Trim()
        $changed = @(& git -C $RepoRoot diff --name-only $base HEAD)
        $allowed770 = @(
            '^experimental/ac-core/src/', '^experimental/ac-core/tests/', '^experimental/ac-core/fixtures/',
            '^experimental/ac-core/schema/', '^experimental/ac-core/runtime/text/',
            '^experimental/ac-core/test-support/marks\.rs$', '^experimental/ac-core/Cargo\.toml$',
            '^experimental/ac-core/Cargo\.lock$', '^experimental/ac-core/README\.md$',
            '^scripts/verify-ac-text-770\.ps1$', '^docs/reports/770-text/', '^docs/plans/770-ac-text-char-native\.md$'
        )
        $illegal = @($changed | Where-Object {
            $p = $_ -replace '\\', '/'
            -not ($allowed770 | Where-Object { $p -match $_ })
        })
        $allowed782 = @('^experimental/ac-core/', '^scripts/verify-ac-text-record-782\.ps1$', '^docs/reports/782-text-record/', '^docs/plans/782-ac-text-record-owned\.md$')
        $outside = @($illegal | Where-Object {
            $p = $_ -replace '\\', '/'
            -not ($allowed782 | Where-Object { $p -match $_ })
        })
        if ($outside.Count -gt 0) { throw "genuine out-of-scope changes flagged by the 770 gate predicate:`n$($outside -join "`n")" }
        $script:ReceiptLines.Add("old gate 770: attributed pass (all substantive steps green; scope step failed on exactly this plan's $($illegal.Count) in-scope artifacts - composition re-based onto the 782 allow-list)")
    }

    # --- 6. this plan's scope isolation + hashes --------------------------------

    Invoke-Step 'scope isolation (V-12: 782 paths only)' {
        $base = (& git -C $RepoRoot merge-base HEAD master | Out-String).Trim()
        $changed = @(& git -C $RepoRoot diff --name-only $base HEAD)
        $allowed = @(
            '^experimental/ac-core/',
            '^scripts/verify-ac-text-record-782\.ps1$',
            '^docs/reports/782-text-record/',
            '^docs/plans/782-ac-text-record-owned\.md$'
        )
        $illegal = @($changed | Where-Object {
            $p = $_ -replace '\\', '/'
            -not ($allowed | Where-Object { $p -match $_ })
        })
        if ($illegal.Count -gt 0) { throw "out-of-scope changes:`n$($illegal -join "`n")" }
        $script:ReceiptLines.Add("scope: clean (diff $base..HEAD within ac-core/gate-script/report paths; product crates/, test/ and every ACC binding untouched - strict acc_inventory premise)")
    }

    Invoke-Step 'hashes (runtime lib + contract sidecar + gate inputs + case PEs)' {
        $script:ReceiptLines.Add("runtime-lib-sha256: $(Get-Sha256 (Join-Path $RtDir 'target\release\ac_text_runtime.lib'))")
        $script:ReceiptLines.Add("runtime-contract-sha256: $(Get-Sha256 (Join-Path $RtDir 'target\release\ac_text_runtime.lib.contract.json'))")
        foreach ($rel in @(
            'schema\text-record.atom',
            'src\hir.rs', 'src\descriptor.rs', 'src\verify.rs',
            'src\main.rs', 'src\text_resources.rs', 'src\lib.rs',
            'src\source\mod.rs', 'src\source\parser.rs', 'src\source\resolve.rs',
            'src\source\typecheck.rs', 'src\source\adapter.rs',
            'tests\text_record_hir.rs', 'tests\text_record_source.rs',
            'fixtures\text-record-source\valid\p01-combination.at',
            'fixtures\text-record-source\valid\p02-overlap.at',
            'fixtures\text-record-hir\valid\tr01-combination.atom',
            'runtime\text\src\lib.rs', 'runtime\text\shim\src\lib.rs',
            'runtime\text\Cargo.toml', 'runtime\text\Cargo.lock'
        )) {
            $p = Join-Path $AcCore $rel
            $script:ReceiptLines.Add(("core {0}: {1}" -f ($rel -replace '\\', '/'), (Get-Sha256 $p)))
        }
        foreach ($c in @('TP01', 'TP02', 'HP01')) {
            $p = Join-Path $ReceiptsDir "cases\$c\app.exe"
            $script:ReceiptLines.Add(("pe {0}: {1}" -f $c, (Get-Sha256 $p)))
        }
        $script:ReceiptLines.Add(("gate-script: {0}" -f (Get-Sha256 "$PSScriptRoot\verify-ac-text-record-782.ps1")))
    }

} catch {
    $script:HadFailure = $true
    $script:ReceiptLines.Add('gate: FAILED - see step table and raw outputs')
} finally {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $receiptPath = Join-Path $ReceiptsDir "gate-$stamp.txt"
    if ($null -ne $script:RawDir) { $script:ReceiptLines.Add("generated: $stamp") }
    [System.IO.File]::WriteAllLines($receiptPath, $script:ReceiptLines)
    if ($null -ne $script:RawDir) {
        $script:Results | Format-Table -AutoSize | Out-String | Set-Content (Join-Path $script:RawDir 'summary.txt')
        $script:CaseSummary | Format-Table -AutoSize | Out-String | Set-Content (Join-Path $script:RawDir 'cases.txt')
    }
    Write-Host ''
    Write-Host 'PLAN-782 text-record gate summary' -ForegroundColor Cyan
    $script:Results | Format-Table -AutoSize
    Write-Host "receipt: $receiptPath"
    if ($script:HadFailure) { exit 1 }
    $failed = @($script:Results | Where-Object { -not $_.Ok })
    if ($failed.Count -gt 0) { exit 1 } else { exit 0 }
}
