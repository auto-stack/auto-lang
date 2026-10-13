# PLAN-767 one-shot scalar-profile verification script (ASCII only for
# Windows PowerShell 5.1). Serially consumes the scalar case manifests:
# every positive case is compiled (source via build-source under
# auto.source.scalar-i32.draft, HIR via build) and EXECUTED as a real PE
# (60s hard deadline per run) with exit/stderr oracles; every negative case
# is checked expecting exit 1 and the pinned stage/code marker; CLI usage
# errors expect exit 2. The contained executor is VERBATIM the 745/741
# runner (suspended start, Job-Object containment, one deadline for exit +
# pipe-EOF collection, full reclamation). Raw receipts land in
# experimental/ac-core/target/ac-verify-receipts/scalar-767/ (gitignored).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-scalar-767.ps1
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$AcCore = Join-Path $RepoRoot 'experimental\ac-core'
$ReceiptsDir = Join-Path $AcCore 'target\ac-verify-receipts\scalar-767'
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

$Scalar = 'auto.source.scalar-i32.draft'
$Core = 'auto.source.core-i32.draft'
$Trace = 'hir.test.trace.v1'

# Positive SOURCE corpus (V-05/V-11/V-12 + trap polarity): id, fixture,
# exit, stderr, capability, support-lib.
$SourcePositive = @(
    @{ Id = 'SP01'; File = 'p-gcd-euclid.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'SP02'; File = 'p-count-digits.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'SP03'; File = 'p-is-digit.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'SP04'; File = 'p-cmp-matrix.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'SP05'; File = 'p-a-minus-1.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'SP06'; File = 'p-for-loops.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'SP07'; File = 'p-binop-args.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'SP08'; File = 'p-crlf.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'SP09'; File = 'p-short-circuit-trap.at'; Exit = 70; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'SP10'; File = 'p-short-circuit-trace.at'; Exit = 0; Stderr = 'aaaba'; Cap = $true; Lib = $true },
    @{ Id = 'SP11'; File = 'p-short-circuit-contexts.at'; Exit = 0; Stderr = 'abababaaaabab'; Cap = $true; Lib = $true }
)

# Positive HIR corpus: id, fixture, entry, exit, stderr, capability, lib.
$HirPositive = @(
    @{ Id = 'HP01'; File = 'n01-divrem-table.atom'; Entry = 'd_main'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'HP02'; File = 'n04-unexecuted-trap.atom'; Entry = 'd_main'; Exit = 3; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'HP03'; File = 'n02-trap-matrix.atom'; Entry = 'd_t_maxadd'; Exit = 70; Stderr = 'a'; Cap = $true; Lib = $true },
    @{ Id = 'HP04'; File = 'n02-trap-matrix.atom'; Entry = 'd_t_minsub'; Exit = 70; Stderr = 'a'; Cap = $true; Lib = $true },
    @{ Id = 'HP05'; File = 'n02-trap-matrix.atom'; Entry = 'd_t_minneg'; Exit = 70; Stderr = 'a'; Cap = $true; Lib = $true },
    @{ Id = 'HP06'; File = 'n02-trap-matrix.atom'; Entry = 'd_t_divm1'; Exit = 70; Stderr = 'a'; Cap = $true; Lib = $true },
    @{ Id = 'HP07'; File = 'n02-trap-matrix.atom'; Entry = 'd_t_remm1'; Exit = 70; Stderr = 'a'; Cap = $true; Lib = $true },
    @{ Id = 'HP08'; File = 'n02-trap-matrix.atom'; Entry = 'd_t_div0'; Exit = 70; Stderr = 'a'; Cap = $true; Lib = $true },
    @{ Id = 'HP09'; File = 'n02-trap-matrix.atom'; Entry = 'd_t_rem0'; Exit = 70; Stderr = 'a'; Cap = $true; Lib = $true },
    @{ Id = 'HP10'; File = 'n03-shortcircuit.atom'; Entry = 'd_sc1'; Exit = 1; Stderr = ''; Cap = $true; Lib = $true },
    @{ Id = 'HP11'; File = 'n03-shortcircuit.atom'; Entry = 'd_sc2'; Exit = 0; Stderr = ''; Cap = $true; Lib = $true },
    @{ Id = 'HP12'; File = 'n03-shortcircuit.atom'; Entry = 'd_sc3'; Exit = 0; Stderr = 'b'; Cap = $true; Lib = $true },
    @{ Id = 'HP13'; File = 'n03-shortcircuit.atom'; Entry = 'd_sc4'; Exit = 0; Stderr = 'b'; Cap = $true; Lib = $true },
    @{ Id = 'HP14'; File = 'n03-shortcircuit.atom'; Entry = 'd_sc5'; Exit = 1; Stderr = ''; Cap = $true; Lib = $true },
    @{ Id = 'HP15'; File = 'n03-shortcircuit.atom'; Entry = 'd_sc6'; Exit = 70; Stderr = ''; Cap = $true; Lib = $true }
)

# Negative SOURCE corpus (V-06/V-08/V-12): id, fixture, stage, code.
$SourceNegative = @(
    @{ Id = 'SN01'; File = 'e01-mixed-eq.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'SN02'; File = 'e02-not-on-int.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'SN03'; File = 'e03-neg-on-bool.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'SN04'; File = 'e04-or-unknown-rhs.at'; Stage = 'bind'; Code = 'source.undefined' },
    @{ Id = 'SN05'; File = 'e05-and-int-rhs.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'SN06'; File = 'e06-and-mixed-types.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'SN07'; File = 'e07-for-nonbool-cond.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'SN08'; File = 'e08-for-in.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'SN09'; File = 'e09-for-init-clause.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'SN10'; File = 'e10-for-unclosed.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'SN11'; File = 'e11-while-in-scalar.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'SN12'; File = 'e12-missing-operand.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'SN13'; File = 'e13-missing-rparen.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'SN14'; File = 'e14-unclosed-call.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'SN15'; File = 'e15-literal-range.at'; Stage = 'text'; Code = 'source.literal-range' },
    @{ Id = 'SN16'; File = 'e16-neg-paren-min.at'; Stage = 'text'; Code = 'source.literal-range' },
    @{ Id = 'SN17'; File = 'e17-unary-plus.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'SN18'; File = 'e18-or-newline.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'SN19'; File = 'e19-neg-newline.at'; Stage = 'text'; Code = 'source.syntax' }
)

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
Invoke-Step 'cargo test scalar_hir' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'scalar_hir') $RepoRoot -TimeoutSec 300
    if ($r.Code -ne 0) { throw $r.Stderr }
}
Invoke-Step 'cargo test scalar_source' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'scalar_source') $RepoRoot -TimeoutSec 300
    if ($r.Code -ne 0) { throw $r.Stderr }
}
Invoke-Step 'cargo test scalar_native (--test-threads=1)' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'scalar_native', '--', '--test-threads=1') $RepoRoot -TimeoutSec 900
    if ($r.Code -ne 0) { throw $r.Stderr }
}

# ---- 3. serial case-table walkthrough ----
Invoke-Step 'build tool binary once' {
    $r = Invoke-Captured 'cargo' @('build', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot -TimeoutSec 600
    if ($r.Code -ne 0) { throw $r.Stderr }
    $script:Bin = Join-Path $AcCore 'target\debug\auto-ac-prototype.exe'
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

Invoke-Step 'positive SOURCE cases: build + run (11)' {
    foreach ($c in $SourcePositive) {
        $dir = Join-Path $ReceiptsDir ('source-positive\' + $c.Id)
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot ("experimental\ac-core\fixtures\scalar-source\valid\" + $c.File)
        ("input: " + $src) | Set-Content (Join-Path $dir 'case.txt')
        ("input-sha256: " + (Get-Sha256 $src)) | Add-Content (Join-Path $dir 'case.txt')
        $exe = Join-Path $dir 'app.exe'
        $args = @('build-source', $src, '--source-profile', $Scalar, '--entry', 'main', '--output', $exe)
        if ($c.Cap) { $args += @('--capability', $Trace) }
        if ($c.Lib) { $args += @('--support-lib', $script:SupportLib) }
        $build = Invoke-Captured $script:Bin $args $RepoRoot -TimeoutSec 120
        $build.Stdout | Set-Content (Join-Path $dir 'build.stdout.txt')
        $build.Stderr | Set-Content (Join-Path $dir 'build.stderr.txt')
        if ($build.Code -ne 0) { throw "$($c.Id): build exited $($build.Code)" }
        if (-not (Test-Path $exe)) { throw "$($c.Id): exe missing" }
        $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
        $run.Stdout | Set-Content (Join-Path $dir 'run.stdout.txt')
        $run.Stderr | Set-Content (Join-Path $dir 'run.stderr.txt')
        ("exit: " + $run.Code) | Add-Content (Join-Path $dir 'case.txt')
        ("exe-sha256: " + (Get-Sha256 $exe)) | Add-Content (Join-Path $dir 'case.txt')
        if (-not (Test-Path (Join-Path $dir 'app.obj'))) { throw "$($c.Id): obj missing" }
        if (-not (Test-Path (Join-Path $dir 'app.ac-link.txt'))) { throw "$($c.Id): receipt missing" }
        if ($run.Code -ne $c.Exit) { throw "$($c.Id): run exited $($run.Code), expected $($c.Exit)" }
        if ($run.Stderr -ne $c.Stderr) { throw "$($c.Id): stderr '$($run.Stderr)' expected '$($c.Stderr)'" }
        $caseSummary.Add([pscustomobject]@{ Case = $c.Id; Exit = $run.Code; Stderr = $run.Stderr; Ok = $true })
    }
}

Invoke-Step 'positive HIR cases: build + run (15)' {
    foreach ($c in $HirPositive) {
        $dir = Join-Path $ReceiptsDir ('hir-positive\' + $c.Id)
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot ("experimental\ac-core\fixtures\scalar-hir\valid\" + $c.File)
        ("input: " + $src) | Set-Content (Join-Path $dir 'case.txt')
        ("input-sha256: " + (Get-Sha256 $src)) | Add-Content (Join-Path $dir 'case.txt')
        ("entry: " + $c.Entry) | Add-Content (Join-Path $dir 'case.txt')
        $exe = Join-Path $dir 'app.exe'
        $args = @('build', $src, '--entry', $c.Entry, '--output', $exe)
        if ($c.Cap) { $args += @('--capability', $Trace) }
        if ($c.Lib) { $args += @('--support-lib', $script:SupportLib) }
        $build = Invoke-Captured $script:Bin $args $RepoRoot -TimeoutSec 120
        $build.Stdout | Set-Content (Join-Path $dir 'build.stdout.txt')
        $build.Stderr | Set-Content (Join-Path $dir 'build.stderr.txt')
        if ($build.Code -ne 0) { throw "$($c.Id): build exited $($build.Code)" }
        $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
        $run.Stdout | Set-Content (Join-Path $dir 'run.stdout.txt')
        $run.Stderr | Set-Content (Join-Path $dir 'run.stderr.txt')
        ("exit: " + $run.Code) | Add-Content (Join-Path $dir 'case.txt')
        ("exe-sha256: " + (Get-Sha256 $exe)) | Add-Content (Join-Path $dir 'case.txt')
        if ($run.Code -ne $c.Exit) { throw "$($c.Id): run exited $($run.Code), expected $($c.Exit)" }
        if ($run.Stderr -ne $c.Stderr) { throw "$($c.Id): stderr '$($run.Stderr)' expected '$($c.Stderr)'" }
        $caseSummary.Add([pscustomobject]@{ Case = $c.Id; Exit = $run.Code; Stderr = $run.Stderr; Ok = $true })
    }
}

Invoke-Step 'negative SOURCE cases: check-source rejects (19)' {
    foreach ($c in $SourceNegative) {
        $dir = Join-Path $ReceiptsDir ('source-negative\' + $c.Id)
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot ("experimental\ac-core\fixtures\scalar-source\invalid\" + $c.File)
        ("input: " + $src) | Set-Content (Join-Path $dir 'case.txt')
        ("input-sha256: " + (Get-Sha256 $src)) | Add-Content (Join-Path $dir 'case.txt')
        $r = Invoke-Captured $script:Bin @('check-source', $src, '--source-profile', $Scalar) $RepoRoot -TimeoutSec 60
        $r.Stdout | Set-Content (Join-Path $dir 'stdout.txt')
        $r.Stderr | Set-Content (Join-Path $dir 'stderr.txt')
        ("exit: " + $r.Code) | Add-Content (Join-Path $dir 'case.txt')
        if ($r.Code -ne 1) { throw "$($c.Id): exited $($r.Code), expected 1" }
        $marker = "error[$($c.Stage)/$($c.Code)]"
        if ($r.Stderr -notmatch [regex]::Escape($marker)) { throw "$($c.Id): marker '$marker' missing in: $($r.Stderr)" }
        $caseSummary.Add([pscustomobject]@{ Case = $c.Id; Exit = $r.Code; Stderr = ''; Ok = $true })
    }
}

Invoke-Step 'profile isolation cases (V-01/V-12 CLI)' {
    $scalarMatrix = Join-Path $RepoRoot 'experimental\ac-core\fixtures\scalar-source\valid\p-cmp-matrix.at'
    # old source profile rejects the SAME scalar source at the operator
    $r = Invoke-Captured $script:Bin @('check-source', $scalarMatrix, '--source-profile', $Core) $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 1) { throw "core on scalar source exited $($r.Code), expected 1" }
    if ($r.Stderr -notmatch [regex]::Escape('text/source.unsupported')) { throw "expected source.unsupported: $($r.Stderr)" }
    # unknown profile value stays a usage error
    $r = Invoke-Captured $script:Bin @('check-source', $scalarMatrix, '--source-profile', 'auto.source.future.draft') $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 2) { throw "unknown profile exited $($r.Code), expected 2" }
    # unknown HIR triple is a controlled bind diagnostic
    $unknown = Join-Path $RepoRoot 'experimental\ac-core\fixtures\scalar-hir\invalid\i09-unknown-triple.atom'
    $r = Invoke-Captured $script:Bin @('check', $unknown) $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 1) { throw "unknown triple exited $($r.Code), expected 1" }
    if ($r.Stderr -notmatch [regex]::Escape('bind/bind.descriptor-unknown')) { throw "expected bind.descriptor-unknown: $($r.Stderr)" }
    # scalar HIR document checks OK and reports the scalar domain
    $r = Invoke-Captured $script:Bin @('check', (Join-Path $RepoRoot 'experimental\ac-core\fixtures\scalar-hir\valid\n01-divrem-table.atom')) $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 0) { throw "scalar check exited $($r.Code): $($r.Stderr)" }
    if ($r.Stdout -notmatch 'hir-profile scalar-i32-draft') { throw "domain missing in: $($r.Stdout)" }
    # while/for isolation both ways (temp sources)
    $tmp = Join-Path $ReceiptsDir 'isolation'
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    $w = Join-Path $tmp 'w.at'
    Set-Content $w "fn main() int {`n    var i = 0`n    while i < 1 { i = i + 1 }`n    return i`n}`n"
    $f = Join-Path $tmp 'f.at'
    Set-Content $f "fn main() int {`n    var i = 0`n    for i < 1 { i = i + 1 }`n    return i`n}`n"
    $r = Invoke-Captured $script:Bin @('check-source', $w, '--source-profile', $Core) $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 0) { throw "core while must pass: $($r.Stderr)" }
    $r = Invoke-Captured $script:Bin @('check-source', $w, '--source-profile', $Scalar) $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 1 -or $r.Stderr -notmatch [regex]::Escape('text/source.unsupported')) { throw "scalar while must reject: $($r.Stderr)" }
    $r = Invoke-Captured $script:Bin @('check-source', $f, '--source-profile', $Scalar) $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 0) { throw "scalar for must pass: $($r.Stderr)" }
    $r = Invoke-Captured $script:Bin @('check-source', $f, '--source-profile', $Core) $RepoRoot -TimeoutSec 60
    if ($r.Code -ne 1 -or $r.Stderr -notmatch [regex]::Escape('text/source.unsupported')) { throw "core for must reject: $($r.Stderr)" }
}

Invoke-Step 'receipt identity lines (V-10)' {
    $dir = Join-Path $ReceiptsDir 'identity'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $src = Join-Path $RepoRoot 'experimental\ac-core\fixtures\scalar-source\valid\p-gcd-euclid.at'
    $exe = Join-Path $dir 'app.exe'
    $r = Invoke-Captured $script:Bin @('build-source', $src, '--source-profile', $Scalar, '--entry', 'main', '--output', $exe) $RepoRoot -TimeoutSec 120
    if ($r.Code -ne 0) { throw "build exited $($r.Code): $($r.Stderr)" }
    $receipt = Get-Content (Join-Path $dir 'app.ac-link.txt') -Raw
    foreach ($needle in @(
        'source-profile: auto.source.scalar-i32.draft',
        'hir-schema: auto.hir.scalar.draft',
        'hir-revision: 1u32',
        'hir-profile: scalar-i32-draft',
        'source-input-hash: '
    )) {
        if ($receipt -notmatch [regex]::Escape($needle)) { throw "receipt missing '$needle': $receipt" }
    }
    $receipt | Set-Content (Join-Path $dir 'receipt.txt')
}

# ---- 4. summary ----
$script:Results | Format-Table -AutoSize | Out-String | Set-Content (Join-Path $ReceiptsDir 'summary.txt')
$caseSummary | Format-Table -AutoSize | Out-String | Set-Content (Join-Path $ReceiptsDir 'cases.txt')
$total = $script:Results.Count
$failed = @($script:Results | Where-Object { -not $_.Ok }).Count
Write-Host ("scalar-767 gate: {0} steps, {1} failed; {2} cases" -f $total, $failed, $caseSummary.Count)
if ($failed -gt 0) { exit 1 }
exit 0
