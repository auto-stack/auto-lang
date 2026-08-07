@echo off
rem Cargo `rustc-wrapper` for Windows: proxy to sccache when available,
rem otherwise pass through to rustc directly. Activated by setting
rem RUSTC_WRAPPER to this script's path (see scripts/README-sccache.md).
rem
rem Cargo invokes:  sccache-wrap.cmd  <rustc>  <rustc-args...>
rem so %* already starts with the rustc executable, which is exactly what
rem sccache expects as its first argument.
rem
rem NOTE: we use `goto` rather than an `if (...) ( ... )` block because cmd
rem expands %errorlevel% when the whole block is parsed, so `exit /b
rem %errorlevel%` inside a block would capture the PRE-block value (the `where`
rem result), not the child's exit code. Statements after a label are parsed
rem individually, so %errorlevel% is current at exit time.

where sccache >nul 2>nul
if errorlevel 1 goto passthrough
sccache %*
exit /b %errorlevel%

:passthrough
%*
exit /b %errorlevel%
