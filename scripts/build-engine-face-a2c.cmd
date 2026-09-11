@echo off
rem Plan 597 (004 SS5-3): build the a2c engine-face driver from its
rem snapshot and stage a self-contained layout. The snapshot pair
rem (engine_face.expected.c/.h) is gated byte-exact by tests/a2c_tests.at,
rem so compiling it directly builds the transpiler output.
rem
rem Link form (Plan 597 T-01 evidence): MSVC rejects a DLL as direct link
rem input (LNK1107); rust cdylib ships an import lib alongside the DLL
rem (<name>.dll.lib) -- we link that and copy the DLL next to the exe
rem (same-dir distribution contract, auto-term 003 SS5).
rem
rem Engine DLL resolution: %AUTOTERM_ENGINE_DLL% -> auto-term main checkout
rem (override the env for custom layouts). Pure ASCII (cmd parses OEM).
rem
rem Usage: scripts\build-engine-face-a2c.cmd [out-dir]

setlocal
set "SRC=%~dp0..\crates\auto-lang\test\a2c\18_c_interop\004_engine_face"
set "OUT=%~1"
if "%OUT%"=="" set "OUT=%~dp0..\target\engine-face-a2c"

set "DLL=%AUTOTERM_ENGINE_DLL%"
if "%DLL%"=="" set "DLL=D:\autostack\auto-term\target\debug\autoterm_core.dll"
set "IMPLIB=%DLL%.lib"

if not exist "%DLL%" (
    echo ERROR: engine dll not found: %DLL%  ^(build it: cargo build -p autoterm-core, or set AUTOTERM_ENGINE_DLL^)
    exit /b 1
)
if not exist "%IMPLIB%" (
    echo ERROR: import lib not found: %IMPLIB%
    exit /b 1
)
if not exist "%SRC%\engine_face.expected.c" (
    echo ERROR: snapshot missing under %SRC%
    exit /b 1
)

set "VCVARS=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if not exist "%VCVARS%" (
    echo ERROR: vcvars64.bat not found: %VCVARS%
    exit /b 1
)

call "%VCVARS%" >nul || exit /b 1
if not exist "%OUT%" mkdir "%OUT%"

copy /y "%SRC%\engine_face.expected.c" "%OUT%\engine_face.c" >nul
copy /y "%SRC%\engine_face.expected.h" "%OUT%\engine_face.h" >nul
copy /y "%SRC%\engine_abi.h" "%OUT%\engine_abi.h" >nul
copy /y "%DLL%" "%OUT%\" >nul

pushd "%OUT%"
cl /nologo /W4 /TC engine_face.c /Fe:engine-face-a2c.exe /link "%IMPLIB%"
set "RC=%ERRORLEVEL%"
popd
if not "%RC%"=="0" (
    echo ERROR: cl failed (%RC%)
    exit /b %RC%
)
if not exist "%OUT%\engine-face-a2c.exe" (
    echo ERROR: engine-face-a2c.exe not produced
    exit /b 1
)
echo OK: %OUT%\engine-face-a2c.exe  ^(engine dll staged same-dir^)
exit /b 0
