; QuantPilot Windows NSIS Installer
; v2.0.0

!define APP_NAME "QuantPilot"
!define APP_VERSION "${VERSION}"
!define APP_PUBLISHER "QuantPilot"
!define APP_EXE "quantpilot.exe"

Name "${APP_NAME} ${APP_VERSION}"
OutFile "..\..\dist\${APP_NAME}-${APP_VERSION}-win-x64.exe"
InstallDir "$LOCALAPPDATA\${APP_NAME}"
RequestExecutionLevel user

Page directory
Page instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  File /r "..\..\dist\package\*"
  CreateShortCut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  RMDir /r "$INSTDIR"
SectionEnd
