@echo off
rem Plan 595 (004 SS5-2): build the a2c autoterm-ctrlc replica from its
rem snapshot. The snapshot pair (autoterm_ctrlc.expected.c/.h) is gated
rem byte-exact by tests/a2c_tests.at, so compiling it directly is building
rem the transpiler output -- regenerate happens via the test suite.
rem
rem Usage: scripts\build-ctrlc-a2c.cmd [out-dir]
rem   out-dir defaults to target/ctrlc-a2c (gitignored).
rem Requires MSVC 2022 (vcvars64). PATH without cl is fine -- this script
rem calls vcvars itself. Pure ASCII on purpose (cmd parses OEM codepage).

setlocal
set "SRC=%~dp0..\crates\auto-lang\test\a2c\18_c_interop\003_autoterm_ctrlc"
set "OUT=%~1"
if "%OUT%"=="" set "OUT=%~dp0..\target\ctrlc-a2c"

set "VCVARS=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if not exist "%VCVARS%" (
    echo ERROR: vcvars64.bat not found: %VCVARS%
    exit /b 1
)

if not exist "%SRC%\autoterm_ctrlc.expected.c" (
    echo ERROR: snapshot missing under %SRC%
    exit /b 1
)

call "%VCVARS%" >nul || exit /b 1
if not exist "%OUT%" mkdir "%OUT%"

copy /y "%SRC%\autoterm_ctrlc.expected.c" "%OUT%\autoterm_ctrlc.c" >nul
copy /y "%SRC%\autoterm_ctrlc.expected.h" "%OUT%\autoterm_ctrlc.h" >nul

pushd "%OUT%"
cl /nologo /W4 /TC autoterm_ctrlc.c /Fe:ctrlc-a2c.exe /link kernel32.lib
set "RC=%ERRORLEVEL%"
popd
if not "%RC%"=="0" (
    echo ERROR: cl failed (%RC%)
    exit /b %RC%
)
if not exist "%OUT%\ctrlc-a2c.exe" (
    echo ERROR: ctrlc-a2c.exe not produced
    exit /b 1
)
echo OK: %OUT%\ctrlc-a2c.exe
exit /b 0
