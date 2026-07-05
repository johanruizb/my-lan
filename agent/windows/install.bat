@echo off
REM Install the MyLAN agent as a Windows service.
REM Run from an elevated command prompt. Requires mylan-agent.exe in PATH
REM or in the same directory as this script.

setlocal
set "BIN=%~dp0mylan-agent.exe"
if not exist "%BIN%" set "BIN=mylan-agent.exe"

if not exist "%PROGRAMDATA%\mylan" mkdir "%PROGRAMDATA%\mylan"
if not exist "%PROGRAMDATA%\mylan\mylan-agent.toml" (
    echo [mylan] NOTE: create %%PROGRAMDATA%%\mylan\mylan-agent.toml before starting.
)

sc.exe create MylanAgent binPath= "\"%BIN%\" --serve-api --config \"%PROGRAMDATA%\mylan\mylan-agent.toml\"" start= auto
sc.exe description MylanAgent "MyLAN network discovery + monitoring + local API"
sc.exe start MylanAgent

echo MylanAgent service installed and started.
endlocal