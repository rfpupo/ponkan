//! Cross-process exclusion. The file is never removed while locked.
use std::{
    fs::{self, File, OpenOptions},
    path::Path,
};

pub(crate) fn reject_reparse(path: &Path, directory: bool) -> Result<(), String> {
    let meta = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.to_string()),
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("Caminho de bloqueio inseguro.".into());
        }
    }
    if meta.file_type().is_symlink()
        || if directory {
            !meta.is_dir()
        } else {
            !meta.is_file()
        }
    {
        return Err("Caminho de bloqueio inseguro.".into());
    }
    Ok(())
}

pub(crate) fn acquire_at(root: &Path, name: &str) -> Result<File, String> {
    reject_reparse(root, true)?;
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let path = root.join(name);
    reject_reparse(&path, false)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.try_lock().map_err(|e| {
        format!("Outra operação está em andamento ou o bloqueio está indisponível: {e}")
    })?;
    Ok(file)
}

pub(crate) fn acquire_game() -> Result<File, String> {
    let base = std::env::var_os("LOCALAPPDATA").ok_or("AppData não encontrado")?;
    acquire_at(&Path::new(&base).join("PonkanLocks"), "riot.lock")
}

