# PLAN-745 one-shot source-profile verification script (ASCII only for
# Windows PowerShell 5.1). Consumes the full corpus case table serially:
# every positive case is compiled from .at source, linked and EXECUTED as a
# real PE (60s hard deadline per run) with exit/stderr oracles; every
# negative case is checked through check-source expecting exit 1 and the
# pinned stage/code marker. Raw receipts land in
# experimental/ac-core/target/ac-verify-receipts/source-745/ (gitignored).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-source-745.ps1
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$AcCore = Join-Path $RepoRoot 'experimental\ac-core'
$ReceiptsDir = Join-Path $AcCore 'target\ac-verify-receipts\source-745'
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
        readonly IntPtr _handle;
        public readonly MemoryStream Buf = new MemoryStream();
        public readonly Thread Thr;
        public bool Done;
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
                if (!ReadFile(_handle, b, (uint)b.Length, out n, IntPtr.Zero) || n == 0) break;
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
    }

    public static RunResult Run(string exe, string[] args, string workDir, int timeoutMs)
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
            job = CreateJobObject(IntPtr.Zero, null);
            if (job == IntPtr.Zero || !AssignProcessToJobObject(job, hProc))
            {
                // refuse to run what cannot be bounded: kill before the
                // first instruction (the child is still suspended)
                TerminateJobObject(job, 1);
                res.Error = "containment unavailable";
                return res;
            }
            if (ResumeThread(hThread) == uint.MaxValue)
            {
                TerminateJobObject(job, 1);
                res.Error = "resume failed";
                return res;
            }
            // parent-side write ends close so EOF reflects the CHILD tree
            CloseHandle(outW); outW = IntPtr.Zero;
            CloseHandle(errW); errW = IntPtr.Zero;
            outReader = new Reader(outR); outReader.Thr.Start();
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
    # Run an external command through the contained C# executor below
    # (R3-QA-01): suspended start, Job-Object subtree containment before the
    # first instruction, ONE deadline covering process exit AND pipe-EOF
    # collection (no quiet-window heuristics), byte draining with lossy
    # UTF-8, and full tree + reader reclamation on every exit path. A
    # descendant holding the pipes past the deadline is a TimedOut result —
    # never a truncated success.
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

$Profile = 'auto.source.core-i32.draft'
$Trace = 'hir.test.trace.v1'

# Positive corpus: id, fixture, exit, stderr, capability, support-lib.
$Positive = @(
    @{ Id = 'P01'; File = 'p01-precedence.at'; Exit = 14; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P02'; File = 'p02-signed-positional.at'; Exit = 1; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P03'; File = 'p03-bool-param-constant-encoding.at'; Exit = 7; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P04'; File = 'p04-while-retest.at'; Exit = 3; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P05'; File = 'p05-while-zero.at'; Exit = 1; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P06'; File = 'p06-continue-retests-condition.at'; Exit = 3; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P07'; File = 'p07-break-nearest-loop.at'; Exit = 5; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P08'; File = 'p08-recursion.at'; Exit = 3; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P09'; File = 'p09-forward-signature.at'; Exit = 8; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P10'; File = 'p10-shadow-rhs-outer.at'; Exit = 5; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P11'; File = 'p11-literal-min-max.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P12'; File = 'p12-unexecuted-trap-not-lifted-x1.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P13'; File = 'p13-add-overflow-trap.at'; Exit = 70; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P14'; File = 'p14-mul-overflow-trap.at'; Exit = 70; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P15'; File = 'p15-named-eval-order-x2.at'; Exit = 12; Stderr = 'ba'; Cap = $true; Lib = $true },
    @{ Id = 'P16'; File = 'p16-observable-before-trap-x9.at'; Exit = 70; Stderr = 'a'; Cap = $true; Lib = $true },
    @{ Id = 'P17'; File = 'p17-heterogeneous-binding-map.at'; Exit = 6; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P18'; File = 'p18-bool-return-local-branch.at'; Exit = 4; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P19'; File = 'p19-user-function-not-intrinsic.at'; Exit = 6; Stderr = ''; Cap = $true; Lib = $false },
    @{ Id = 'P20'; File = 'p20-utf8-crlf-negative-min.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P21'; File = 'p21-mutual-recursion.at'; Exit = 4; Stderr = ''; Cap = $false; Lib = $false }
)

# Negative corpus: id, fixture, stage, code.
$Negative = @(
    @{ Id = 'N01'; File = 'n01-literal-range-max.at'; Stage = 'text'; Code = 'source.literal-range' },
    @{ Id = 'N02'; File = 'n02-literal-range-min.at'; Stage = 'text'; Code = 'source.literal-range' },
    @{ Id = 'N03'; File = 'n03-undefined-name.at'; Stage = 'bind'; Code = 'source.undefined' },
    @{ Id = 'N04'; File = 'n04-type-mismatch-return.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N05'; File = 'n05-type-mismatch-if-cond.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N06'; File = 'n06-type-mismatch-bool-add.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N07'; File = 'n07-immutable-write.at'; Stage = 'verify'; Code = 'verify.immutable-write' },
    @{ Id = 'N08'; File = 'n08-duplicate-local.at'; Stage = 'bind'; Code = 'source.duplicate' },
    @{ Id = 'N09'; File = 'n09-undefined-scope-escape.at'; Stage = 'bind'; Code = 'source.undefined' },
    @{ Id = 'N10'; File = 'n10-arg-missing.at'; Stage = 'bind'; Code = 'source.argument' },
    @{ Id = 'N11'; File = 'n11-arg-duplicate.at'; Stage = 'bind'; Code = 'source.argument' },
    @{ Id = 'N12'; File = 'n12-arg-unknown-name.at'; Stage = 'bind'; Code = 'source.argument' },
    @{ Id = 'N13'; File = 'n13-arg-mixed.at'; Stage = 'bind'; Code = 'source.argument' },
    @{ Id = 'N14'; File = 'n14-type-mismatch-arg.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N15'; File = 'n15-break-outside-loop.at'; Stage = 'verify'; Code = 'verify.loop-target-scope' },
    @{ Id = 'N16'; File = 'n16-continue-outside-loop.at'; Stage = 'verify'; Code = 'verify.loop-target-scope' },
    @{ Id = 'N17'; File = 'n17-no-return-path.at'; Stage = 'verify'; Code = 'verify.no-return-path' },
    @{ Id = 'N18'; File = 'n18-stmt-after-return.at'; Stage = 'verify'; Code = 'verify.block-structure' },
    @{ Id = 'N19'; File = 'n19-var-uninitialized.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N20'; File = 'n20-char-literal.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N21'; File = 'n21-str-type.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N22'; File = 'n22-i64-type.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N23'; File = 'n23-sub-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N24'; File = 'n24-div-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N25'; File = 'n25-and-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N26'; File = 'n26-checked-marker.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N27'; File = 'n27-not-callable-local.at'; Stage = 'bind'; Code = 'source.not-callable' },
    @{ Id = 'N28'; File = 'n28-not-callable-param.at'; Stage = 'bind'; Code = 'source.not-callable' },
    @{ Id = 'N29'; File = 'n29-not-callable-shadow.at'; Stage = 'bind'; Code = 'source.not-callable' },
    @{ Id = 'N30'; File = 'n30-eof-block.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'N31'; File = 'n31-lex-at.at'; Stage = 'text'; Code = 'source.lex' },
    @{ Id = 'N32'; File = 'n32-duplicate-fn.at'; Stage = 'bind'; Code = 'source.duplicate' },
    @{ Id = 'N33'; File = 'n33-duplicate-param.at'; Stage = 'bind'; Code = 'source.duplicate' },
    @{ Id = 'N34'; File = 'n34-mod-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N35'; File = 'n35-eq-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N36'; File = 'n36-unary-minus-ident.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N37'; File = 'n37-suffixed-literal.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N38'; File = 'n38-hex-literal.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N39'; File = 'n39-toplevel-use.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N40'; File = 'n40-toplevel-let.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N41'; File = 'n41-call-statement.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N42'; File = 'n42-missing-ret-ann.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N43'; File = 'n43-list-type.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N44'; File = 'n44-toplevel-type.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N45'; File = 'n45-toplevel-enum.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N46'; File = 'n46-for-loop.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N47'; File = 'n47-float-literal.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N48'; File = 'n48-block-comment.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N49'; File = 'n49-unicode-ident.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N50'; File = 'n50-undefined-callee.at'; Stage = 'bind'; Code = 'source.undefined' },
    @{ Id = 'N51'; File = 'n51-type-mismatch-lt-bool.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N52'; File = 'n52-tailing-garbage.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'N53'; File = 'n53-empty-file.at'; Stage = 'text'; Code = 'source.syntax' }
)

# ---- 1. context receipt ----
Invoke-Step 'context (HEAD + toolchain)' {
    $lines = @()
    $lines += 'git-head: ' + (Invoke-Captured 'git' @('rev-parse', 'HEAD') $RepoRoot -TimeoutSec 60).Stdout.Trim()
    $lines += 'git-branch: ' + (Invoke-Captured 'git' @('branch', '--show-current') $RepoRoot -TimeoutSec 60).Stdout.Trim()
    $status = (Invoke-Captured 'git' @('status', '--short') $RepoRoot -TimeoutSec 60).Stdout.Trim()
    $lines += 'git-status-short-lines: ' + (@($status -split "`n").Count)
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
Invoke-Step 'cargo test source_frontend' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'source_frontend') $RepoRoot -TimeoutSec 300
    if ($r.Code -ne 0) { throw $r.Stderr }
}
Invoke-Step 'cargo test source_hir' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'source_hir') $RepoRoot -TimeoutSec 300
    if ($r.Code -ne 0) { throw $r.Stderr }
}
Invoke-Step 'cargo test source_native (--test-threads=1)' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'source_native', '--', '--test-threads=1') $RepoRoot -TimeoutSec 900
    if ($r.Code -ne 0) { throw $r.Stderr }
}
Invoke-Step 'cargo test source_cli (--test-threads=1)' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'source_cli', '--', '--test-threads=1') $RepoRoot -TimeoutSec 900
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

Invoke-Step 'positive cases: build + run (21)' {
    foreach ($c in $Positive) {
        $dir = Join-Path $ReceiptsDir ('positive\' + $c.Id)
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot ("experimental\ac-core\fixtures\source\valid\" + $c.File)
        ("input: " + $src) | Set-Content (Join-Path $dir 'case.txt')
        ("input-sha256: " + (Get-Sha256 $src)) | Add-Content (Join-Path $dir 'case.txt')
        $exe = Join-Path $dir 'app.exe'
        $args = @('build-source', $src, '--source-profile', $Profile, '--entry', 'main', '--output', $exe)
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

Invoke-Step 'negative cases: check-source rejects (53)' {
    foreach ($c in $Negative) {
        $dir = Join-Path $ReceiptsDir ('negative\' + $c.Id)
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot ("experimental\ac-core\fixtures\source\invalid\" + $c.File)
        ("input: " + $src) | Set-Content (Join-Path $dir 'case.txt')
        ("input-sha256: " + (Get-Sha256 $src)) | Add-Content (Join-Path $dir 'case.txt')
        $extra = @()
        if ($c.Id -in @('N28', 'N29')) { $extra = @('--capability', $Trace) }
        $r = Invoke-Captured $script:Bin (@('check-source', $src, '--source-profile', $Profile) + $extra) $RepoRoot -TimeoutSec 60
        $r.Stdout | Set-Content (Join-Path $dir 'stdout.txt')
        $r.Stderr | Set-Content (Join-Path $dir 'stderr.txt')
        ("exit: " + $r.Code) | Add-Content (Join-Path $dir 'case.txt')
        if ($r.Code -ne 1) { throw "$($c.Id): exited $($r.Code), expected 1" }
        $marker = "error[$($c.Stage)/$($c.Code)]"
        if ($r.Stderr -notmatch [regex]::Escape($marker)) { throw "$($c.Id): marker '$marker' missing in: $($r.Stderr)" }
        $caseSummary.Add([pscustomobject]@{ Case = $c.Id; Exit = $r.Code; Stderr = ''; Ok = $true })
    }
}

# ---- 4. summary ----
$caseSummary | Format-Table -AutoSize | Tee-Object -FilePath (Join-Path $ReceiptsDir 'cases.txt')
$Results | Format-Table -AutoSize | Tee-Object -FilePath (Join-Path $ReceiptsDir 'summary.txt')
$failed = @($Results | Where-Object { -not $_.Ok })
if ($failed.Count -gt 0) {
    Write-Host ("VERIFY-AC-SOURCE-745: {0}/{1} steps FAILED" -f $failed.Count, $Results.Count) -ForegroundColor Red
    exit 1
}
Write-Host ("VERIFY-AC-SOURCE-745: all {0} steps PASS ({1} cases)" -f $Results.Count, $caseSummary.Count) -ForegroundColor Green
exit 0
