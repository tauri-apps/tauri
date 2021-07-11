# Adapted from https://superuser.com/a/532109

param([switch]$Elevated)

function Test-Admin {
    $currentUser = New-Object Security.Principal.WindowsPrincipal $([Security.Principal.WindowsIdentity]::GetCurrent())
    $currentUser.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
}

if ((Test-Admin) -eq $false) {
    if ($elevated) {
        # tried to elevate, did not work, aborting
        Write-Host "Failed to elevate"
    }
    else {
        Start-Process powershell.exe -Verb RunAs -ArgumentList ('-file "{0}" -elevated' -f ($myinvocation.MyCommand.Definition))
        Write-Host "Attempted to elevate"
    }
    PAUSE
    exit
}

Write-Host "Elevated"
SCHTASKS.EXE /CREATE /XML "C:\Program Files\updater-example\update.xml" /TN "Update {{{product_name}}} - Skip UAC" /F
PAUSE