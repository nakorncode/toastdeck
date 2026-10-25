; ToastDesk v3 per-user installer (NSIS).
; Build with scripts\build-installers.ps1, or directly:
;   makensis /DVERSION=3.0.0 /DSRCDIR=<folder with ToastDeskV3.exe, sounds\> /DOUTFILE=<setup.exe> toastdesk-v3.nsi

Unicode true
SetCompressor /SOLID lzma

!ifndef VERSION
  !define VERSION "3.0.0"
!endif
!ifndef SRCDIR
  !error "SRCDIR is required"
!endif
!ifndef OUTFILE
  !error "OUTFILE is required"
!endif

!define PRODUCT "ToastDesk v3"
!define APP_ID "ToastDesk.v3"
!define EXE "ToastDeskV3.exe"
!define UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_ID}"
!define RUN_KEY "Software\Microsoft\Windows\CurrentVersion\Run"

Name "${PRODUCT}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\${APP_ID}"
RequestExecutionLevel user
ShowInstDetails show
ShowUninstDetails show

Section "Install"
  ; Stop only this app. The v2 build uses ToastDesk.exe and is left alone.
  nsExec::Exec 'taskkill /F /IM ${EXE} /T'
  Sleep 1500

  SetOutPath "$INSTDIR"
  File "${SRCDIR}\${EXE}"
  File "${SRCDIR}\AOSP-LICENSE.md"
  File "${SRCDIR}\icon.png"
  File "${SRCDIR}\icon.ico"

  SetOutPath "$INSTDIR\sounds"
  File "${SRCDIR}\sounds\*.wav"

  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateDirectory "$SMPROGRAMS\${PRODUCT}"
  CreateShortCut "$SMPROGRAMS\${PRODUCT}\${PRODUCT}.lnk" "$INSTDIR\${EXE}"
  CreateShortCut "$SMPROGRAMS\${PRODUCT}\Uninstall ${PRODUCT}.lnk" "$INSTDIR\Uninstall.exe"

  WriteRegStr HKCU "${UNINST_KEY}" "DisplayName" "${PRODUCT}"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINST_KEY}" "Publisher" "NakornCode"
  WriteRegStr HKCU "${UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayIcon" "$INSTDIR\${EXE}"
  WriteRegStr HKCU "${UNINST_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegDWORD HKCU "${UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINST_KEY}" "NoRepair" 1
SectionEnd

Section "un.Uninstall"
  nsExec::Exec 'taskkill /F /IM ${EXE} /T'
  Sleep 1500

  ; Startup entry written by the app. Settings in %LOCALAPPDATA%\ToastDesk.v3 are kept.
  DeleteRegValue HKCU "${RUN_KEY}" "${APP_ID}"

  Delete "$INSTDIR\${EXE}"
  Delete "$INSTDIR\AOSP-LICENSE.md"
  Delete "$INSTDIR\icon.png"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\sounds\*.wav"
  RMDir "$INSTDIR\sounds"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\${PRODUCT}\${PRODUCT}.lnk"
  Delete "$SMPROGRAMS\${PRODUCT}\Uninstall ${PRODUCT}.lnk"
  RMDir "$SMPROGRAMS\${PRODUCT}"

  DeleteRegKey HKCU "${UNINST_KEY}"
SectionEnd
