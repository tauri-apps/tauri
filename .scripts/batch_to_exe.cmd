@ECHO OFF
ECHO Make EXE From BAT
ECHO.
ECHO.

REM Usage:
REM MakeExeFromBat BatFileToConvert [IncludeFile1] [IncludeFile2] [...]
REM
REM Required Parameters:
REM  BatFileToConvert
REM      Source batch file to use to produce the output Exe file.
REM
REM Optional Parameters:
REM  IncludeFile
REM      Additional files to include in the Exe file.
REM      You can include external tools used by the batch file so they are available on the executing machine.

SETLOCAL

REM Configuration (no quotes needed):
SET PathTo7Zip=


REM ---- Do not modify anything below this line ----

SET OutputFile="%~n1.exe"
SET SourceFiles="%TEMP%MakeEXE_files.txt"
SET Config="%TEMP%MakeEXE_config.txt"
SET Source7ZFile="%Temp%MakeEXE.7z"

REM Remove existing files
IF EXIST %OutputFile% DEL %OutputFile%

REM Build source archive
ECHO "%~dpnx1" > %SourceFiles%
:AddInclude
IF {%2}=={} GOTO EndInclude
ECHO "%~dpnx2" >> %SourceFiles%
SHIFT /2
GOTO AddInclude
:EndInclude
"%PathTo7Zip%7za.exe" a %Source7ZFile% @%SourceFiles%

REM Build config file
ECHO ;!@Install@!UTF-8! > %Config%
ECHO RunProgram="%~nx1" >> %Config%
ECHO ;!@InstallEnd@! >> %Config%

REM Build EXE
COPY /B "%PathTo7Zip%7zsd.sfx" + %Config% + %Source7ZFile% %OutputFile%

REM Clean up
IF EXIST %SourceFiles% DEL %SourceFiles%
IF EXIST %Config% DEL %Config%
IF EXIST %Source7ZFile% DEL %Source7ZFile%

ENDLOCAL