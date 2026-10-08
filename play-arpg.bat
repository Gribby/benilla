@echo off
rem Start the benilla ARPG client. Double-click this file, or make a desktop shortcut to it.
rem   play-arpg.bat          start the client (builds it first if it has never been built)
rem   play-arpg.bat update   pull the latest code from GitHub, rebuild, then start
rem
rem WOW_DATA is the WoW 1.12.1 client's Data folder. Change the line below if yours moves, or
rem set WOW_DATA in Windows' environment variables to override it.
setlocal
cd /d "%~dp0"
if not defined WOW_DATA set "WOW_DATA=D:\WoWclient\World of Warcraft\Data"
set "WOW_ARPG=1"

if not exist "%WOW_DATA%" (
    echo The WoW Data folder was not found: %WOW_DATA%
    echo Edit WOW_DATA near the top of play-arpg.bat.
    goto :fail
)

if /i "%~1"=="update" (
    echo Pulling the latest code...
    git pull || goto :fail
    echo Building...
    cargo build --release -p benilla || goto :fail
)

if not exist "target\release\benilla.exe" (
    echo Building the client for the first time, this takes a while...
    cargo build --release -p benilla || goto :fail
)

"target\release\benilla.exe"
if errorlevel 1 goto :fail
goto :eof

:fail
echo.
echo Something went wrong; see above.
pause
exit /b 1
