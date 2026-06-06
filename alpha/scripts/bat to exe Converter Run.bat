@echo off
setlocal EnableExtensions
title EdgeBeacon — BAT to EXE Builder

REM Always run from this BAT's folder (scripts/)
cd /d "%~dp0"

set "PS=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
set "PS1=%~dp0bat2exe.ps1"
if not exist "%PS1%" (
  echo ERROR: %PS1% not found.
  pause & exit /b 1
)

echo ===== BAT -> EXE (interactive) =====
set /p IN=Input BAT path (e.g. Auto Reload Until Website is Up.bat): 
if "%IN%"=="" echo Need input BAT & pause & exit /b 1

REM If user mistakenly typed "scripts\...", strip it
if /I "%IN:~0,8%"=="scripts\" set "IN=%IN:~8%"

REM Make absolute path
for %%I in ("%IN%") do set "IN=%%~fI"
if not exist "%IN%" echo ERROR: BAT not found: %IN% & pause & exit /b 1

set /p OUT=Output EXE path (e.g. ..\EdgeBeacon.exe): 
if "%OUT%"=="" set "OUT=%~dp0..\EdgeBeacon.exe"
for %%I in ("%OUT%") do set "OUT=%%~fI"
if /I not "%OUT:~-4%"==".exe" set "OUT=%OUT%.exe"

set /p ICO=Icon ICO path (optional, e.g. icon\edgebeacon.ico): 
if defined ICO (
  if /I "%ICO:~0,8%"=="scripts\" set "ICO=%ICO:~8%"
  for %%I in ("%ICO%") do set "ICO=%%~fI"
)

set /p GUI=Hide console window? (Y/N): 
set "GUISW="
if /I "%GUI%"=="Y" set "GUISW=-WinExe"

echo.
echo Running compiler...
if defined ICO (
  "%PS%" -NoProfile -ExecutionPolicy Bypass -File "%PS1%" -InputBat "%IN%" -OutputExe "%OUT%" -Icon "%ICO%" %GUISW%
) else (
  "%PS%" -NoProfile -ExecutionPolicy Bypass -File "%PS1%" -InputBat "%IN%" -OutputExe "%OUT%" %GUISW%
)
echo.
pause
