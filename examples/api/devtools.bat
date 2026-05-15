@echo off
setlocal enabledelayedexpansion

:: Initialize port number (fixed to 9222)
set PORT=9222
set PID_LIST=
set MAPPED_PID=

:: Get the list of all forwarded ports and PIDs
for /f "tokens=2,5 delims=:_" %%a in ('hdc fport ls') do (
    for /f "tokens=1 delims= " %%c in ("%%b") do (
        set PID_LIST=!PID_LIST! %%c
    )
)

:: Get the domain socket name of devtools
set SOCKET_COUNT=0
set SOCKET_NAME=

for /f "tokens=*" %%a in ('hdc shell "cat /proc/net/unix | grep devtools"') do (
    set /a SOCKET_COUNT=SOCKET_COUNT+1
    set SOCKET_NAME=%%a
)

:: Check if multiple WebView processes exist
if !SOCKET_COUNT! GTR 1 (
    echo Error: Multiple WebView DevTools processes detected !SOCKET_COUNT! processes.
    echo This script only supports a single WebView process.
    echo Please close other applications using WebView and try again.
    pause
    exit /b 1
)

:: Check if any WebView process exists
if !SOCKET_COUNT! EQU 0 (
    echo No WebView DevTools process found. Please open debugging in your application code.
    pause
    exit /b 0
)

:: Extract process ID from socket name
for /f "delims=_ tokens=4" %%b in ("!SOCKET_NAME!") do set PID=%%b

echo Detected WebView process with PID: !PID!

:: Check if PID already has a mapping
echo !PID_LIST! | findstr /C:"!PID!" >nul
if not errorlevel 1 (
    echo PID !PID! is already mapped to a port.
) else (
    :: PID not mapped, clean up port 9222 and add new mapping
    echo PID !PID! is not mapped yet.
    
    :: Check if port 9222 is already in use
    hdc fport ls | findstr "tcp:!PORT!" >nul
    if not errorlevel 1 (
        echo Port !PORT! is currently in use. Cleaning up existing mapping...
        hdc fport rm tcp:!PORT!
        if errorlevel 1 (
            echo Error: Failed to remove existing mapping on port !PORT!.
            pause
            exit /b 1
        )
        echo Existing mapping on port !PORT! removed.
    )
    
    :: Add new mapping for the PID to port 9222
    echo Mapping PID !PID! to port !PORT!...
    hdc fport tcp:!PORT! localabstract:webview_devtools_remote_!PID!
    if errorlevel 1 (
        echo Error: Failed to add mapping.
        pause
        exit /b 1
    )
    echo Successfully mapped PID !PID! to port !PORT!.
)

:: Check current port forwarding status
echo.
echo Current port forwarding rules:
hdc fport ls

echo.
echo Script executed successfully.
pause

:: Try to open the page in Edge
start msedge chrome://inspect/#devices
if errorlevel 1 (
    start chrome chrome://inspect/#devices
)

endlocal