$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r4'
$w='D:/autostack/.wt/lang-745/auto-lang'
$full=[IO.File]::ReadAllText((Join-Path $w 'scripts/verify-ac-source-745.ps1'))
$a=$full.IndexOf('$script:AcRunnerCs =');$b=$full.IndexOf('function Get-Sha256 {');if($a -lt 0 -or $b -lt $a){throw 'runner seam absent'}
$exact=$full.Substring($a,$b-$a);[IO.File]::WriteAllText((Join-Path $r 'csharp-runner-exact.ps1.txt'),$exact)
Invoke-Expression $exact
Add-Type -TypeDefinition $script:AcRunnerCs -Language CSharp
$script:AcRunnerLoaded=$true
$env:AC745_REVIEW_PID_PATH=Join-Path $r 'csharp-descendant-pid.txt'
$probeExe=Join-Path $w 'experimental/ac-core/target/review-r4/helper_probe.exe'
$rows=@()
foreach($c in @(
 @{id='hold';timeout=500;mode='--root-exits'},
 @{id='late';timeout=3000;mode='--root-late'},
 @{id='running-parent';timeout=500;mode='--root-waits'},
 @{id='invalid-utf8';timeout=1000;mode='--invalid-utf8'}
)){
 $sw=[Diagnostics.Stopwatch]::StartNew();$res=[AcContainedRunner]::Run($probeExe,@($c.mode),$w,$c.timeout)
 $alive=$null;if($c.mode -ne '--invalid-utf8'){$taskPid=[int]([IO.File]::ReadAllText($env:AC745_REVIEW_PID_PATH));$alive=$null -ne (Get-Process -Id $taskPid -ErrorAction SilentlyContinue);if($alive){Stop-Process -Id $taskPid}}
 $rows+=@{id=$c.id;deadline_ms=$c.timeout;elapsed_ms=$sw.ElapsedMilliseconds;result=$res;descendant_alive=$alive}
}
# Only the containment call is mutated: use a null Job handle to induce an
# actual Win32 assignment failure; record child PID for survivor checking.
$fault=$script:AcRunnerCs.Replace('AcContainedRunner','AcRejectedRunner').Replace('public int Code;','public int Code; public int Pid;').Replace('hProc = pi.hProcess; hThread = pi.hThread;','hProc = pi.hProcess; hThread = pi.hThread; res.Pid = pi.dwProcessId;').Replace('!AssignProcessToJobObject(job, hProc)','!AssignProcessToJobObject(IntPtr.Zero, hProc)')
if($fault -eq $script:AcRunnerCs -or -not $fault.Contains('!AssignProcessToJobObject(IntPtr.Zero, hProc)')){throw 'fault did not change containment input'}
[IO.File]::WriteAllText((Join-Path $r 'csharp-containment-control.cs.txt'),$fault)
Add-Type -TypeDefinition $fault -Language CSharp
$sw=[Diagnostics.Stopwatch]::StartNew();$res=[AcRejectedRunner]::Run('cmd.exe',@('/c','exit 0'),$w,500)
$alive=$null -ne (Get-Process -Id $res.Pid -ErrorAction SilentlyContinue)
$rows+=@{id='actual-assignment-failure';deadline_ms=500;elapsed_ms=$sw.ElapsedMilliseconds;result=$res;child_alive_after_return=$alive;mutation='AssignProcessToJobObject receives IntPtr.Zero; all cleanup unchanged'}
if($alive){Stop-Process -Id $res.Pid}
$accepted=$fault.Replace('AcRejectedRunner','AcAcceptedRunner').Replace('!AssignProcessToJobObject(IntPtr.Zero, hProc)','!AssignProcessToJobObject(job, hProc)')
Add-Type -TypeDefinition $accepted -Language CSharp
$res=[AcAcceptedRunner]::Run('cmd.exe',@('/c','echo READ_CONTROL'),$w,1000)
$rows+=@{id='valid-assignment-and-read-control';result=$res;child_alive_after_return=($null -ne (Get-Process -Id $res.Pid -ErrorAction SilentlyContinue))}
# Force a real native ReadFile error (ERROR_INVALID_HANDLE), with every
# caller/result/cleanup branch unchanged. It must not become empty success.
$badRead=$accepted.Replace('AcAcceptedRunner','AcReadFailureRunner').Replace('ReadFile(_handle, b,','ReadFile(IntPtr.Zero, b,')
$badRead=$badRead.Replace('    class Reader','    public static int LastNativeReadError;'+"`n"+'    class Reader')
$badRead=$badRead.Replace('if (!ReadFile(IntPtr.Zero, b, (uint)b.Length, out n, IntPtr.Zero) || n == 0) break;','if (!ReadFile(IntPtr.Zero, b, (uint)b.Length, out n, IntPtr.Zero) || n == 0) { LastNativeReadError = Marshal.GetLastWin32Error(); break; }')
if($badRead -eq $accepted -or -not $badRead.Contains('ReadFile(IntPtr.Zero, b,')){throw 'read-error injection absent'}
[IO.File]::WriteAllText((Join-Path $r 'csharp-read-control.cs.txt'),$badRead)
Add-Type -TypeDefinition $badRead -Language CSharp
$res=[AcReadFailureRunner]::Run('cmd.exe',@('/c','echo READ_CONTROL'),$w,1000)
$rows+=@{id='actual-read-failure';result=$res;native_error=[AcReadFailureRunner]::LastNativeReadError;mutation='ReadFile receives IntPtr.Zero; native error recorded; result/cleanup handling unchanged'}
$rows|ConvertTo-Json -Depth 7|Set-Content (Join-Path $r 'csharp-control-results.json') -Encoding utf8
$rows|ConvertTo-Json -Depth 7|Write-Output
