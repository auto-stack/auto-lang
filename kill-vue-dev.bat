@echo off
REM Kill all Node.js/Vite dev server processes
echo Stopping all npm/node/vite processes...

REM Kill node processes running vite
taskkill /F /IM node.exe 2>nul

REM Kill any remaining vite processes
taskkill /F /FI "WINDOWTITLE eq *vite*" 2>nul

echo Done.
