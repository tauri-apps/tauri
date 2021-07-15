# Copyright 2019-2021 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT
# Adapted from https://superuser.com/a/532109
param([string]$ChangeDir, [switch]$Elevated)

function Test-Admin {
    $currentUser = New-Object Security.Principal.WindowsPrincipal $([Security.Principal.WindowsIdentity]::GetCurrent())
    $currentUser.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
}

if ((Test-Admin) -eq $false) {
    if ($elevated) {
        # tried to elevate, did not work, aborting
    }
    else {
        $InstallDirectory = Get-Location
        $ArgList = ('-File "{0}" -ChangeDir "{1}" -Elevated' -f ($myinvocation.MyCommand.Definition, $InstallDirectory))
        Start-Process powershell.exe -WindowStyle hidden -Verb RunAs -ArgumentList $ArgList
    }
    exit
}

if ($ChangeDir -ne "") {
    # Change directories to the install path
    Set-Location -Path $ChangeDir
}
SCHTASKS.EXE /CREATE /XML update.xml /TN "Update {{{product_name}}} - Skip UAC" /F