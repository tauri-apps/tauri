@echo OFF
echo "Setting up enviromental Variables"
rem setup relative paths
set "TAURI_DIST_DIR=%~1..\dist"
set "TAURI_DIR=%~1..\src-tauri"
rem convert relative path to absolute path and re-set it into the enviroment var
for /F "delims=" %%F IN ("%TAURI_DIST_DIR%") DO SET "TAURI_DIST_DIR=%%~fF"
for /F "delims=" %%F IN ("%TAURI_DIR%") DO SET "TAURI_DIR=%%~fF"

echo "Ready to Work"