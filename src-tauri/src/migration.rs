//! Storage identity and upgrade guard. Never silently create an empty replacement
//! for a previous installation whose data has not been migrated.
use std::path::{Path, PathBuf};

pub(crate) fn storage_root_at(base: &Path, development: bool) -> PathBuf {
    base.join(if development {
        "PonkanDevData"
    } else {
        "PonkanData"
    })
}
#[cfg(not(debug_assertions))]
pub(crate) fn ensure_release_ready_at(base: &Path) -> Result<(), String> {
    const BLOCKED: &str = "A migração dos dados da versão anterior precisa ser concluída antes de iniciar esta versão. Seus dados foram preservados.";
    // An inaccessible or invalid base is not evidence of a clean installation.
    if !std::fs::metadata(base).map_err(|_| BLOCKED)?.is_dir() {
        return Err(BLOCKED.into());
    }
    // Inspect entries without following links; dangling reparse points and
    // interrupted staging are not safe to interpret as missing legacy data.
    for name in ["CFGValorant", "dev.cfgvalorant.desktop", "PonkanMigration"] {
        match std::fs::symlink_metadata(base.join(name)) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err(BLOCKED.into()),
        }
    }
    Ok(())
}
#[cfg(not(debug_assertions))]
pub(crate) fn ensure_release_ready() -> Result<(), String> {
    let base = std::env::var("LOCALAPPDATA").map_err(|_| "AppData não encontrado")?;
    ensure_release_ready_at(Path::new(&base))
}

