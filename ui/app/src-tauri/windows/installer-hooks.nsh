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

; On uninstall, ask whether to KEEP the encrypted account data (so reinstalling restores the account)
; or ERASE it now. Default is KEEP, so a silent/automated uninstall never destroys data. "No" removes
; the encrypted snapshot AND its durable mirror from this PC. The OS-keychain device key is left as a
; harmless orphan (useless without the ciphertext) — the in-app "Delete my data" is what removes the
; key as well, for a full cryptographic erase.
!macro NSIS_HOOK_PREUNINSTALL
  MessageBox MB_YESNO|MB_ICONQUESTION \
    "Keep your encrypted Mercury data on this PC, so reinstalling restores your account?$\n$\nYes = keep it (recommended)$\nNo = erase your account data now (cannot be undone)" \
    /SD IDYES IDYES mercury_keep_data
    RMDir /r "$APPDATA\com.mercury.messaging"
    RMDir /r "$LOCALAPPDATA\com.mercury.messaging"
  mercury_keep_data:
!macroend
