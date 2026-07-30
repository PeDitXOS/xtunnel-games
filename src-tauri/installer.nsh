!macro NSIS_HOOK_POSTINSTALL
  ; Copy DLLs from resources dir to install dir (next to exe)
  ; so Windows can find them via DLL search path
  CopyFiles "$INSTDIR\resources\WinDivert.dll" "$INSTDIR\"
  CopyFiles "$INSTDIR\resources\WinDivert64.sys" "$INSTDIR\"
  CopyFiles "$INSTDIR\resources\wintun.dll" "$INSTDIR\"
  CopyFiles "$INSTDIR\resources\aether.exe" "$INSTDIR\"
  CopyFiles "$INSTDIR\resources\sing-box.exe" "$INSTDIR\"
  CopyFiles "$INSTDIR\resources\xray.exe" "$INSTDIR\"
!macroend
