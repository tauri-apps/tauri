; Installs or updates the WebView2 runtime.
;
; Sections run in the order they are declared, so this file has to be
; included between `Section EarlyChecks` and `Section Install`.

!define WEBVIEW2APPGUID "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

; Version of the installed runtime, empty if it is not installed
Var Webview2Version

; Values interpolated into the `webview2DownloadError` and `webview2InstallError`
; language strings, declared whatever the install mode because NSIS prints the
; variable name instead of its value when a language string interpolates a
; variable that the build never declares
Var Webview2DownloadResult
Var Webview2InstallExitCode

; An empty install mode means the runtime is shipped next to the app or skipped,
; so there is no installer to run
!if "${INSTALLWEBVIEW2MODE}" != ""
  ; Path of the bootstrapper or offline installer to run
  Var Webview2InstallerPath
!endif

!if "${MINIMUMWEBVIEW2VERSION}" != ""
  ; Whether the installed runtime is older than ${MINIMUMWEBVIEW2VERSION}
  Var Webview2VersionCompare

  ; Path of the Edge updater, used to update an outdated runtime
  Var EdgeUpdatePath
!endif

Section WebView2
  ; The `webview2DownloadError` and `webview2InstallError` language strings
  ; interpolate the variables below, but only some install modes set them.
  ; NSIS does not count language string interpolation as a reference, so without
  ; a value it warns about them never being written (warning 6001)
  StrCpy $Webview2DownloadResult ""
  StrCpy $Webview2InstallExitCode ""

  ; Check if Webview2 is already installed and skip this section
  ${If} ${RunningX64}
    ReadRegStr $Webview2Version HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${Else}
    ReadRegStr $Webview2Version HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${EndIf}
  ${If} $Webview2Version == ""
    ReadRegStr $Webview2Version HKCU "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${EndIf}

  ${If} $Webview2Version == ""
    ; Webview2 installation
    ;
    ; Skip if updating
    ${If} $UpdateMode <> 1
      !if "${INSTALLWEBVIEW2MODE}" == "downloadBootstrapper"
        Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        DetailPrint "$(webview2Downloading)"
        NSISdl::download "https://go.microsoft.com/fwlink/p/?LinkId=2124703" "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Pop $Webview2DownloadResult
        ; TODO(v3): only kept for custom language files that interpolate `$0`
        StrCpy $0 "$Webview2DownloadResult"
        ${If} $Webview2DownloadResult == "success"
          DetailPrint "$(webview2DownloadSuccess)"
        ${Else}
          DetailPrint "$(webview2DownloadError)"
          Abort "$(webview2AbortError)"
        ${EndIf}
        StrCpy $Webview2InstallerPath "$TEMP\MicrosoftEdgeWebview2Setup.exe"
      !else if "${INSTALLWEBVIEW2MODE}" == "embedBootstrapper"
        Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        File "/oname=$TEMP\MicrosoftEdgeWebview2Setup.exe" "${WEBVIEW2BOOTSTRAPPERPATH}"
        StrCpy $Webview2InstallerPath "$TEMP\MicrosoftEdgeWebview2Setup.exe"
      !else if "${INSTALLWEBVIEW2MODE}" == "offlineInstaller"
        Delete "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
        File "/oname=$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe" "${WEBVIEW2INSTALLERPATH}"
        StrCpy $Webview2InstallerPath "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
      !endif

      ; Run the installer the branch above prepared
      !if "${INSTALLWEBVIEW2MODE}" != ""
        DetailPrint "$(installingWebview2)"
        ExecWait "$Webview2InstallerPath ${WEBVIEW2INSTALLERARGS} /install" $Webview2InstallExitCode
        ; TODO(v3): only kept for custom language files that interpolate `$1`
        StrCpy $1 "$Webview2InstallExitCode"
        ${If} $Webview2InstallExitCode = 0
          DetailPrint "$(webview2InstallSuccess)"
        ${Else}
          DetailPrint "$(webview2InstallError)"
          Abort "$(webview2AbortError)"
        ${EndIf}
      !endif
    ${EndIf}
  ${Else}
    ; Webview2 update
    ;
    ; The runtime is installed, but it may be older than the version the app
    ; needs, so bring it up to ${MINIMUMWEBVIEW2VERSION} through the Edge updater
    !if "${MINIMUMWEBVIEW2VERSION}" != ""
      ${VersionCompare} "${MINIMUMWEBVIEW2VERSION}" "$Webview2Version" $Webview2VersionCompare
      ${If} $Webview2VersionCompare = 1
        update_webview:
          DetailPrint "$(installingWebview2)"
          ${If} ${RunningX64}
            ReadRegStr $EdgeUpdatePath HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate" "path"
          ${Else}
            ReadRegStr $EdgeUpdatePath HKLM "SOFTWARE\Microsoft\EdgeUpdate" "path"
          ${EndIf}
          ${If} $EdgeUpdatePath == ""
            ReadRegStr $EdgeUpdatePath HKCU "SOFTWARE\Microsoft\EdgeUpdate" "path"
          ${EndIf}
          ${If} $EdgeUpdatePath != ""
            ; Chromium updater docs: https://source.chromium.org/chromium/chromium/src/+/main:docs/updater/user_manual.md
            ; Modified from "HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Microsoft EdgeWebView\ModifyPath"
            ExecWait `"$EdgeUpdatePath" /install appguid=${WEBVIEW2APPGUID}&needsadmin=true` $Webview2InstallExitCode
            ; TODO(v3): only kept for custom language files that interpolate `$1`
            StrCpy $1 "$Webview2InstallExitCode"
            ${If} $Webview2InstallExitCode = 0
              DetailPrint "$(webview2InstallSuccess)"
            ${Else}
              MessageBox MB_ICONEXCLAMATION|MB_ABORTRETRYIGNORE "$(webview2InstallError)" IDIGNORE ignore IDRETRY update_webview
              Quit
              ignore:
            ${EndIf}
          ${EndIf}
      ${EndIf}
    !endif
  ${EndIf}
SectionEnd
