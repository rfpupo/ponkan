; Guard only. This does not perform or authorize a migration.
; The legacy installations are not yet supported by the 0.3.0 upgrade path.
!macro PONKAN_ABORT_MIGRATION
  ${IfNot} ${Silent}
    MessageBox MB_OK|MB_ICONSTOP "Ponkan cannot install over legacy or incomplete migration data. Your data and current installation have been preserved. Complete the verified migration first."
  ${EndIf}
  SetErrorLevel 2
  Abort
!macroend

!macro PONKAN_CHECK_ENTRY BASE ENTRY
  System::Call 'kernel32::GetFileAttributesW(w "${BASE}\${ENTRY}") i.r0 ?e'
  Pop $1
  ${If} $0 != -1
    !insertmacro PONKAN_ABORT_MIGRATION
  ${ElseIf} $1 != 2
  ${AndIf} $1 != 3
    ; Access denied, invalid path, or another unverified state.
    !insertmacro PONKAN_ABORT_MIGRATION
  ${EndIf}
!macroend

!macro PONKAN_CHECK_LEGACY BASE
  Push $0
  Push $1
  Push $2
  System::Call 'kernel32::GetFileAttributesW(w "${BASE}") i.r0'
  ${If} $0 == -1
    !insertmacro PONKAN_ABORT_MIGRATION
  ${EndIf}
  IntOp $2 $0 & 16
  ${If} $2 == 0
    !insertmacro PONKAN_ABORT_MIGRATION
  ${EndIf}
  !insertmacro PONKAN_CHECK_ENTRY "${BASE}" "CFGValorant"
  !insertmacro PONKAN_CHECK_ENTRY "${BASE}" "dev.cfgvalorant.desktop"
  !insertmacro PONKAN_CHECK_ENTRY "${BASE}" "PonkanMigration"
  Pop $2
  Pop $1
  Pop $0
!macroend
