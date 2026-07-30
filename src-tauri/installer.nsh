!macro NSIS_HOOK_POSTINSTALL
  ; Copy DLLs from resources dir to install dir (next to exe)
  ; Only copy if source exists — skip silently if not
  IfFileExists "$INSTDIR\resources\WinDivert.dll" 0 +2
    CopyFiles "$INSTDIR\resources\WinDivert.dll" "$INSTDIR\"
  IfFileExists "$INSTDIR\resources\WinDivert64.sys" 0 +2
    CopyFiles "$INSTDIR\resources\WinDivert64.sys" "$INSTDIR\"
  IfFileExists "$INSTDIR\resources\wintun.dll" 0 +2
    CopyFiles "$INSTDIR\resources\wintun.dll" "$INSTDIR\"
!macroend
