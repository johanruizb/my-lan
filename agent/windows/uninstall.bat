@echo off
REM Uninstall the MyLAN agent Windows service.
REM Run from an elevated command prompt.

sc.exe stop MylanAgent
sc.exe delete MylanAgent

echo MylanAgent service stopped and removed.