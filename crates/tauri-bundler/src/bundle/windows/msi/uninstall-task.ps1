# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT
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
        $ArgList = ('-File "{0}" -Elevated' -f $myinvocation.MyCommand.Definition)
        Start-Process "$env:SYSTEMROOT\System32\WindowsPowerShell\v1.0\powershell.exe" -WindowStyle hidden -Verb RunAs -ArgumentList $ArgList
    }
    exit
}

SCHTASKS.EXE /DELETE /TN 'Update {{product_name}} - Skip UAC' /F
