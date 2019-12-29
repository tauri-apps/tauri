@echo OFF
echo "Setting up enviromental Variables"

rem check script execution directory vs script directory. 

IF "%cd%\"=="%~dp0" (
    GOTO exitnodir
)

rem setup relative paths from root folder
set "TAURI_DIST_DIR=%~1tauri\test\fixture\dist"
set "TAURI_DIR=%~1tauri\test\fixture\src-tauri"
rem convert relative path to absolute path and re-set it into the enviroment var
for /F "delims=" %%F IN ("%TAURI_DIST_DIR%") DO SET "TAURI_DIST_DIR=%%~fF"
for /F "delims=" %%F IN ("%TAURI_DIR%") DO SET "TAURI_DIR=%%~fF"

if NOT EXIST %TAURI_DIR% GOTO exitnodir
if NOT EXIST %TAURI_DIST_DIR% GOTO exitnodir

GOTO exitfine

:exitnodir
echo "Variables are not setup properly. Please run from Tauri Root directory"
@EXIT /B 1

:exitfine
echo "Variables set, ready to work!"
@EXIT /B 0