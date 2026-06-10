!define MERCURY_SHORTCUT_ICON_NAME "mercury-shortcut-icon-v0.1.10.ico"

!macro MERCURY_INSTALL_SHORTCUT_ICON
  SetOutPath "$INSTDIR"
  Delete "$INSTDIR\mercury-shortcut-icon-v*.ico"
  File /oname=$INSTDIR\${MERCURY_SHORTCUT_ICON_NAME} "${INSTALLERICON}"
!macroend

!macro MERCURY_CREATE_SHORTCUT_WITH_ICON LINKPATH
  CreateShortcut "${LINKPATH}" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MERCURY_SHORTCUT_ICON_NAME}" 0
  !insertmacro SetLnkAppUserModelId "${LINKPATH}"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Tauri's default shortcuts omit IconLocation, which lets Windows keep stale icon-cache entries
  ; across upgrades. Rewrite shortcuts with an explicit icon source after files are installed.
  !insertmacro MERCURY_INSTALL_SHORTCUT_ICON

  !if "${STARTMENUFOLDER}" != ""
    CreateDirectory "$SMPROGRAMS\$AppStartMenuFolder"
    !insertmacro MERCURY_CREATE_SHORTCUT_WITH_ICON "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
  !else
    !insertmacro MERCURY_CREATE_SHORTCUT_WITH_ICON "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  !endif

  ${If} ${FileExists} "$DESKTOP\${PRODUCTNAME}.lnk"
    !insertmacro MERCURY_CREATE_SHORTCUT_WITH_ICON "$DESKTOP\${PRODUCTNAME}.lnk"
    StrCpy $NoShortcutMode 1
  ${EndIf}

  ${If} ${FileExists} "$APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\${PRODUCTNAME}.lnk"
    !insertmacro MERCURY_CREATE_SHORTCUT_WITH_ICON "$APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\${PRODUCTNAME}.lnk"
  ${EndIf}

  ExecWait '"$SYSDIR\ie4uinit.exe" -show'
!macroend
