; Remove only registry entries created by the retired Ponkan File Editor.
!include "${__FILEDIR__}\release-migration-guard.nsh"
!macro NSIS_HOOK_POSTINSTALL
  ReadRegStr $0 SHCTX "Software\Classes\Ponkan File\shell\open\command" ""
  ${If} $0 == '$\"$INSTDIR\ponkan.exe$\" $\"%1$\"'
    ReadRegStr $1 SHCTX "Software\Classes\.pkn" ""
    ${If} $1 == "Ponkan File"
      DeleteRegValue SHCTX "Software\Classes\.pkn" ""
    ${EndIf}
    ReadRegStr $1 SHCTX "Software\Classes\.pkn" "Ponkan File_backup"
    ${If} $1 == "Ponkan File"
      DeleteRegValue SHCTX "Software\Classes\.pkn" "Ponkan File_backup"
    ${EndIf}
    DeleteRegValue SHCTX "Software\Classes\.pkn\OpenWithProgids" "Ponkan File"
    DeleteRegKey /ifempty SHCTX "Software\Classes\.pkn\OpenWithProgids"
    DeleteRegKey /ifempty SHCTX "Software\Classes\.pkn"
    DeleteRegKey SHCTX "Software\Classes\Ponkan File"
  ${EndIf}
  ReadRegStr $0 SHCTX "Software\Classes\Applications\ponkan.exe\shell\open\command" ""
  ReadRegStr $1 SHCTX "Software\Classes\Applications\ponkan.exe" "FriendlyAppName"
  ${If} $0 == '$\"$INSTDIR\ponkan.exe$\" $\"%1$\"'
  ${AndIf} $1 == "Ponkan File Editor"
    DeleteRegKey SHCTX "Software\Classes\Applications\ponkan.exe"
  ${EndIf}
  ReadRegStr $0 SHCTX "Software\RegisteredApplications" "Ponkan File Editor"
  ${If} $0 == "Software\ponkan\Capabilities"
    DeleteRegValue SHCTX "Software\RegisteredApplications" "Ponkan File Editor"
  ${EndIf}
  ReadRegStr $0 SHCTX "Software\ponkan\Capabilities" "ApplicationName"
  ${If} $0 == "Ponkan File Editor"
    DeleteRegKey SHCTX "Software\ponkan\Capabilities"
  ${EndIf}
  Delete "$INSTDIR\ponkan-file.ico"
  !insertmacro UPDATEFILEASSOC
!macroend
