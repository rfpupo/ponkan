use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared: Option<bool>,
    pub id: String,
    pub name: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn io<T>(value: std::io::Result<T>) -> Result<T, String> {
    value.map_err(|e| format!("Falha ao acessar arquivos: {e}"))
}

pub fn data_root() -> Result<PathBuf, String> {
    let base = PathBuf::from(std::env::var("LOCALAPPDATA").map_err(|_| "AppData não encontrado")?);
    Ok(crate::migration::storage_root_at(
        &base,
        cfg!(debug_assertions),
    ))
}
pub fn config_root() -> Result<PathBuf, String> {
    Ok(
        PathBuf::from(std::env::var("LOCALAPPDATA").map_err(|_| "AppData não encontrado")?)
            .join("VALORANT")
            .join("Saved")
            .join("Config"),
    )
}
pub(crate) fn timestamp_now() -> Result<String, String> {
    // Keep the old UTC offset notation, preserving the RFC3339 profile format.
    let value = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| "Data inválida")?;
    Ok(value
        .strip_suffix('Z')
        .map(|v| format!("{v}+00:00"))
        .unwrap_or(value))
}
pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.trim() != name
        || name.chars().count() > 48
        || name
            .chars()
            .any(|c| c.is_control() || "\\/:*?\"<>|".contains(c))
    {
        return Err("Nome de perfil inválido".into());
    }
    Ok(())
}
pub fn require_game_closed() -> Result<(), String> {
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    for process in system.processes().values() {
        let n = process.name().to_string_lossy().to_ascii_lowercase();
        if n == "valorant-win64-shipping.exe" || n == "valorant.exe" {
            return Err("Feche o VALORANT antes de continuar".into());
        }
    }
    Ok(())
}
pub(crate) fn file_map(root: &Path) -> Result<BTreeMap<PathBuf, (u64, String)>, String> {
    crate::storage_lock::reject_reparse(root, true)?;
    if !root.is_dir() {
        return Err("Configurações locais não encontradas".into());
    }
    let mut map = BTreeMap::new();
    for entry in WalkDir::new(root) {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type();
        crate::storage_lock::reject_reparse(entry.path(), kind.is_dir())?;
        let rel = entry
            .path()
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_path_buf();
        if kind.is_dir() {
            if !rel.as_os_str().is_empty() {
                map.insert(rel, (0, "directory".into()));
            }
        } else if kind.is_file() {
            let data = io(fs::read(entry.path()))?;
            let hash = format!("{:x}", Sha256::digest(&data));
            map.insert(rel, (data.len() as u64, hash));
        } else {
            return Err("Tipo de arquivo não suportado".into());
        }
    }
    Ok(map)
}
pub fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    crate::storage_lock::reject_reparse(source, true)?;
    crate::storage_lock::reject_reparse(target, true)?;
    if !source.is_dir() {
        return Err("Configurações locais não encontradas".into());
    }
    if target.exists() {
        return Err("Pasta de destino já existe".into());
    }
    for entry in WalkDir::new(source) {
        let entry = entry.map_err(|e| e.to_string())?;
        crate::storage_lock::reject_reparse(entry.path(), entry.file_type().is_dir())?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|e| e.to_string())?;
        let out = target.join(relative);
        if entry.file_type().is_dir() {
            io(fs::create_dir_all(out))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = out.parent() {
                io(fs::create_dir_all(parent))?;
            }
            io(fs::copy(entry.path(), out))?;
        }
    }
    verify_tree(source, target)
}
pub fn verify_tree(expected: &Path, actual: &Path) -> Result<(), String> {
    if file_map(expected)? != file_map(actual)? {
        return Err("Arquivos restaurados diferem do perfil".into());
    }
    Ok(())
}
fn index_path() -> Result<PathBuf, String> {
    Ok(data_root()?.join("index.json"))
}
pub fn list() -> Result<Vec<Profile>, String> {
    let path = index_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let bytes = io(fs::read(path))?;
    serde_json::from_slice(&bytes).map_err(|_| "Lista de perfis danificada".into())
}
fn save_index(profiles: &[Profile]) -> Result<(), String> {
    crate::trash::atomic_json(&index_path()?, profiles)
}
fn profile_path(id: &str) -> Result<PathBuf, String> {
    Uuid::parse_str(id).map_err(|_| "Identificador inválido")?;
    Ok(data_root()?.join("profiles").join(id))
}
pub fn find(id: &str) -> Result<Profile, String> {
    list()?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or("Perfil não encontrado".into())
}
pub fn cloud_for(id: &str) -> Result<String, String> {
    find(id)?;
    let data = io(fs::read_to_string(profile_path(id)?.join("cloud.txt")))?;
    if data.is_empty() {
        return Err("Perfil da nuvem vazio".into());
    }
    Ok(data)
}
pub fn config_for(id: &str) -> Result<PathBuf, String> {
    find(id)?;
    let path = profile_path(id)?.join("Config");
    if !path.is_dir() {
        return Err("Arquivos locais ausentes".into());
    }
    Ok(path)
}
pub async fn save(
    name: String,
    id: Option<String>,
    cloud: String,
    session: &crate::riot::Session,
) -> Result<Vec<Profile>, String> {
    validate_name(&name)?;
    if cloud.is_empty() {
        return Err("Configurações da nuvem vazias".into());
    }
    let mut profiles = list()?;
    if profiles
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case(&name) && Some(&p.id) != id.as_ref())
    {
        return Err("Já existe um perfil com esse nome".into());
    }
    let actual_id = match id {
        Some(ref existing) => find(existing)?.id,
        None => Uuid::new_v4().to_string(),
    };
    let root = data_root()?;
    let stage_id = Uuid::new_v4().to_string();
    let stage = root.join("staging").join(&stage_id);
    let dest = profile_path(&actual_id)?;
    io(fs::create_dir_all(&stage))?;
    if let Err(error) = (|| {
        io(fs::write(stage.join("cloud.txt"), &cloud))?;
        let source = config_root()?;
        let before = file_map(&source)?;
        let target = stage.join("Config");
        copy_tree(&source, &target).map_err(|error| {
            if error == "Arquivos restaurados diferem do perfil" {
                "As configurações mudaram durante o salvamento. Tente novamente.".into()
            } else {
                error
            }
        })?;
        if before != file_map(&target)? || before != file_map(&source)? {
            return Err("As configurações mudaram durante o salvamento. Tente novamente.".into());
        }
        Ok::<(), String>(())
    })() {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    let prior = profiles.iter().find(|p| p.id == actual_id).cloned();
    if dest.exists() != prior.is_some() {
        return Err("Perfil inconsistente. Os dados foram preservados.".into());
    }
    let history_id = prior.as_ref().map(|_| Uuid::new_v4().to_string());
    let history = history_id
        .as_ref()
        .map(|id| root.join("history").join(&actual_id).join(id));
    let item = Profile {
        id: actual_id,
        name,
        updated_at: timestamp_now()?,
        portable: None,
        shared: None,
        author: prior.as_ref().and_then(|p| p.author.clone()),
        description: prior.as_ref().and_then(|p| p.description.clone()),
    };
    let journal = SaveJournal {
        stage_id,
        history_id,
        previous: prior,
        next: item.clone(),
    };
    if let Some(current) = profiles.iter_mut().find(|p| p.id == item.id) {
        *current = item;
    } else {
        profiles.push(item);
    }
    // Saving is allowed while the game is open, but never publish a capture
    // when either the local files or the pinned account's cloud data changed.
    let consistent = async {
        crate::riot::assert_current_session(session).await?;
        let current_cloud = crate::riot::read_for(session).await?;
        verify_capture(
            &config_root()?,
            &stage.join("Config"),
            &cloud,
            &current_cloud,
        )?;
        crate::riot::assert_current_session(session).await
    }
    .await;
    if let Err(error) = consistent {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    let journal_path = write_save_journal(&root, &journal)?;
    commit_snapshot(&stage, &dest, history.as_deref(), || save_index(&profiles))?;
    if let Err(error) = fs::remove_file(journal_path) {
        eprintln!("Save journal cleanup: {error}");
    }
    Ok(profiles)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveJournal {
    stage_id: String,
    history_id: Option<String>,
    previous: Option<Profile>,
    next: Profile,
}
fn canonical_id(id: &str) -> bool {
    Uuid::parse_str(id)
        .ok()
        .is_some_and(|v| v.to_string() == id)
}
fn validate_journal(j: &SaveJournal) -> Result<(), String> {
    if !canonical_id(&j.next.id)
        || !canonical_id(&j.stage_id)
        || j.history_id.as_ref().is_some_and(|id| !canonical_id(id))
        || j.previous.is_some() != j.history_id.is_some()
        || j.previous
            .as_ref()
            .is_some_and(|old| old.id != j.next.id || old == &j.next)
    {
        return Err("Registro de recuperação inválido.".into());
    }
    Ok(())
}
fn write_save_journal(root: &Path, j: &SaveJournal) -> Result<PathBuf, String> {
    validate_journal(j)?;
    let path = root
        .join("save-journal")
        .join(format!("{}.json", j.stage_id));
    if path.exists() {
        return Err("Registro de recuperação já existe.".into());
    }
    crate::trash::atomic_json(&path, j)?;
    Ok(path)
}
fn safe_recovery_path(path: &Path, directory: bool) -> Result<bool, String> {
    let m = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err("Caminho de recuperação inseguro.".into());
        }
    }
    if m.file_type().is_symlink() || if directory { !m.is_dir() } else { !m.is_file() } {
        return Err("Caminho de recuperação inseguro.".into());
    }
    Ok(true)
}
pub(crate) fn recover_saves() -> Result<(), String> {
    recover_saves_at(&data_root()?)
}
fn recover_saves_at(root: &Path) -> Result<(), String> {
    let base = root.join("save-journal");
    if !safe_recovery_path(root, true)? || !safe_recovery_path(&base, true)? {
        return Ok(());
    }
    for entry in io(fs::read_dir(&base))? {
        let entry = io(entry)?;
        let path = entry.path();
        let file = entry.file_name();
        let name = file.to_str().ok_or("Registro de recuperação inválido.")?;
        // Atomic JSON may leave an uncommitted temporary file after interruption.
        if name
            .strip_prefix(".ponkan-")
            .and_then(|x| x.strip_suffix(".tmp"))
            .is_some_and(canonical_id)
        {
            continue;
        }
        let stage_id = name
            .strip_suffix(".json")
            .filter(|id| canonical_id(id))
            .ok_or("Registro de recuperação inválido.")?;
        if !safe_recovery_path(&path, false)? || io(fs::metadata(&path))?.len() > 64 * 1024 {
            return Err("Registro de recuperação inválido.".into());
        }
        let j: SaveJournal = serde_json::from_slice(&io(fs::read(&path))?)
            .map_err(|_| "Registro de recuperação inválido.")?;
        validate_journal(&j)?;
        if j.stage_id != stage_id {
            return Err("Registro de recuperação inválido.".into());
        }
        let active: Vec<Profile> = if root.join("index.json").exists() {
            serde_json::from_slice(&io(fs::read(root.join("index.json")))?)
                .map_err(|_| "Lista de perfis danificada")?
        } else {
            vec![]
        };
        let mut matching = active.iter().filter(|p| p.id == j.next.id);
        let current = matching.next();
        if matching.next().is_some() {
            return Err("Registro de recuperação ambíguo.".into());
        }
        let dest = root.join("profiles").join(&j.next.id);
        let stage = root.join("staging").join(&j.stage_id);
        let history = j
            .history_id
            .as_ref()
            .map(|id| root.join("history").join(&j.next.id).join(id));
        for parent in [
            root.join("profiles"),
            root.join("staging"),
            root.join("history"),
            root.join("history").join(&j.next.id),
        ] {
            safe_recovery_path(&parent, true)?;
        }
        let (has_dest, has_stage) = (
            safe_recovery_path(&dest, true)?,
            safe_recovery_path(&stage, true)?,
        );
        let has_history = match &history {
            Some(p) => safe_recovery_path(p, true)?,
            None => false,
        };
        let ambiguous =
            || "Registro de recuperação ambíguo. Os dados foram preservados.".to_string();
        if current == Some(&j.next) {
            if !has_dest || has_stage || has_history != j.previous.is_some() {
                return Err(ambiguous());
            }
        } else if current == j.previous.as_ref() {
            if let Some(old) = history.as_ref() {
                match (has_dest, has_stage, has_history) {
                    (true, true, false) => {}
                    (false, true, true) => {
                        io(fs::rename(old, &dest))?;
                    }
                    (true, false, true) => {
                        io(fs::create_dir_all(
                            stage.parent().ok_or("Caminho inválido")?,
                        ))?;
                        io(fs::rename(&dest, &stage))?;
                        io(fs::rename(old, &dest))?;
                    }
                    _ => return Err(ambiguous()),
                }
            } else {
                match (has_dest, has_stage) {
                    (false, true) => {}
                    (true, false) => {
                        io(fs::create_dir_all(
                            stage.parent().ok_or("Caminho inválido")?,
                        ))?;
                        io(fs::rename(&dest, &stage))?;
                    }
                    _ => return Err(ambiguous()),
                }
            }
        } else {
            return Err(ambiguous());
        }
        io(fs::remove_file(path))?;
    }
    Ok(())
}

// Publish only after the snapshot is complete. On an ordinary I/O failure, retain
// the staged data and restore the old snapshot; report any failed rollback.
fn commit_snapshot(
    stage: &Path,
    dest: &Path,
    history: Option<&Path>,
    commit_index: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    io(fs::create_dir_all(dest.parent().ok_or("Caminho inválido")?))?;
    if let Some(old) = history {
        if old.exists() {
            return Err("Pasta de destino já existe".into());
        }
        io(fs::create_dir_all(old.parent().ok_or("Caminho inválido")?))?;
        io(fs::rename(dest, old))?;
    } else if dest.exists() {
        return Err("Pasta de destino já existe".into());
    }
    if let Err(error) = io(fs::rename(stage, dest)) {
        if let Some(old) = history {
            if let Err(rollback) = io(fs::rename(old, dest)) {
                return Err(format!("{error}. Recuperação falhou: {rollback}"));
            }
        }
        return Err(error);
    }
    if let Err(error) = commit_index() {
        if let Err(rollback) = io(fs::rename(dest, stage)) {
            return Err(format!("{error}. Recuperação falhou: {rollback}"));
        }
        if let Some(old) = history {
            if let Err(rollback) = io(fs::rename(old, dest)) {
                return Err(format!("{error}. Recuperação falhou: {rollback}"));
            }
        }
        return Err(error);
    }
    Ok(())
}
pub fn rename(id: &str, name: String) -> Result<Vec<Profile>, String> {
    validate_name(&name)?;
    let mut profiles = list()?;
    if profiles
        .iter()
        .any(|p| p.id != id && p.name.eq_ignore_ascii_case(&name))
    {
        return Err("Nome já utilizado".into());
    }
    let item = profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("Perfil não encontrado")?;
    item.name = name;
    save_index(&profiles)?;
    Ok(profiles)
}
pub fn emergency_backup(cloud: &str) -> Result<PathBuf, String> {
    let dir = data_root()?
        .join("backups")
        .join(Uuid::new_v4().to_string());
    io(fs::create_dir_all(&dir))?;
    io(fs::write(dir.join("cloud.txt"), cloud))?;
    copy_tree(&config_root()?, &dir.join("Config"))?;
    Ok(dir)
}

pub fn import_portable(
    name: String,
    manifest: crate::portable::Manifest,
    files: std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<Profile, String> {
    import_portable_at(&data_root()?, name, manifest, files)
}
pub(crate) fn import_portable_at(
    root: &Path,
    name: String,
    manifest: crate::portable::Manifest,
    files: std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<Profile, String> {
    validate_name(&name)?;
    let idx = root.join("index.json");
    let mut list: Vec<Profile> = if idx.exists() {
        serde_json::from_slice(&io(fs::read(&idx))?).map_err(|_| "Lista de perfis danificada")?
    } else {
        vec![]
    };
    if list.iter().any(|p| p.name.eq_ignore_ascii_case(&name)) {
        return Err("Já existe um perfil com esse nome".into());
    }
    let id = Uuid::new_v4().to_string();
    let stage = root.join("staging").join(&id);
    let dest = root.join("profiles").join(&id);
    let item = Profile {
        id: id.clone(),
        name,
        updated_at: manifest.updated_at.clone(),
        portable: Some(true),
        shared: files.contains_key("raw/share.json").then_some(true),
        author: manifest.author.clone(),
        description: manifest.description.clone(),
    };
    let prepared = (|| {
        io(fs::create_dir_all(&stage))?;
        for (p, b) in files {
            if p.starts_with("assets/avatar.") {
                io(fs::create_dir_all(stage.join("assets")))?;
                io(fs::write(stage.join(&p), &b))?;
            }
            if let Some(rel) = p.strip_prefix("raw/") {
                let path = stage.join(rel);
                io(fs::create_dir_all(path.parent().ok_or("Caminho inválido")?))?;
                io(fs::write(path, b))?;
            }
        }
        io(fs::write(
            stage.join("portable.json"),
            serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
        ))?;
        Ok::<(), String>(())
    })();
    if let Err(error) = prepared {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }

    // An imported profile is a new snapshot, so the same recovery rules apply.
    let journal = SaveJournal {
        stage_id: id,
        history_id: None,
        previous: None,
        next: item.clone(),
    };
    let journal_path = match write_save_journal(root, &journal) {
        Ok(path) => path,
        Err(error) => {
            let _ = fs::remove_dir_all(&stage);
            return Err(error);
        }
    };
    list.push(item.clone());
    // Once a journal is durable, preserve staging on failure for recovery.
    commit_snapshot(&stage, &dest, None, || {
        crate::trash::atomic_json(&idx, &list)
    })?;
    if let Err(error) = fs::remove_file(journal_path) {
        eprintln!("Import journal cleanup: {error}");
    }
    Ok(item)
}





fn verify_capture(source: &Path, staged: &Path, before: &str, after: &str) -> Result<(), String> {
    if !crate::riot::cloud_equals(before, after) || file_map(source)? != file_map(staged)? {
        return Err("As configurações mudaram durante o salvamento. Tente novamente.".into());
    }
    Ok(())
}
