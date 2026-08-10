@echo off
cd /d D:\autostack\auto-lang\.worktree\plan-407
C:\Users\zhaop\.cargo\bin\cargo.exe build -p auto-lang 2> D:\autostack\auto-lang\.worktree\plan-407\err.txt
echo exitcode=%errorlevel%
