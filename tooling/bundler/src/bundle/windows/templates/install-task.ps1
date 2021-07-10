# Adapted from https://superuser.com/a/532109

param([switch]$Elevated)

function Test-Admin {
    $currentUser = New-Object Security.Principal.WindowsPrincipal $([Security.Principal.WindowsIdentity]::GetCurrent())
    $currentUser.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
}

if ((Test-Admin) -eq $false) {
    if ($elevated) {
        # tried to elevate, did not work, aborting
    }
    else {
        Start-Process powershell.exe -Verb RunAs -ArgumentList ('-file "{0}" -elevated' -f ($myinvocation.MyCommand.Definition)) -WorkingDirectory Get-Location
        pause
    }
    exit
}

SCHTASKS.EXE /CREATE /XML ".\update.xml" /TN "Update {{{product_name}}} - Skip UAC" /F
pause