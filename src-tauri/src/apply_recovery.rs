//! Persistent, cross-process apply recovery. Never auto-write Riot or local data.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
type R<T> = Result<T, String>;
pub(crate) type FileMap = BTreeMap<PathBuf, (u64, String)>;
const PENDING: &str = "apply-pending";
const BLOCKED: &str = "Existe uma aplicação interrompida. Os dados foram preservados; a recuperação precisa ser verificada.";

#[derive(Debug, PartialEq, Eq)]
enum State {
    Before,
    After,
    Ambiguous,
}

fn classify(record: &Record, cloud: &str, local: &FileMap) -> State {
    if crate::riot::cloud_equals(&record.before_cloud, cloud) && &record.before_local == local {
        State::Before
    } else if crate::riot::cloud_equals(&record.after_cloud, cloud) && &record.after_local == local
    {
        State::After
    } else {
        State::Ambiguous
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u8,
    subject: String,
    before_cloud: String,
    after_cloud: String,
    before_local: FileMap,
    after_local: FileMap,
}

fn io<T>(result: std::io::Result<T>) -> R<T> {
    result.map_err(|e| format!("Falha no registro de recuperação: {e}"))
}
fn pending_exists(root: &Path) -> R<bool> {
    crate::storage_lock::reject_reparse(root, true)?;
    let pending = root.join(PENDING);
    crate::storage_lock::reject_reparse(&pending, true)?;
    match fs::symlink_metadata(pending) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

fn lock_root() -> R<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA").ok_or("AppData não encontrado")?;
    Ok(Path::new(&base).join("PonkanLocks"))
}

fn begin_at(
    root: &Path,
    config: &Path,
    subject: &str,
    before_cloud: &str,
    after_cloud: &str,
    before: &FileMap,
    after: &FileMap,
) -> R<()> {
    if pending_exists(root)? {
        return Err(BLOCKED.into());
    }
    uuid::Uuid::parse_str(subject).map_err(|_| "Identidade Riot inválida")?;
    if before_cloud.is_empty() || after_cloud.is_empty() {
        return Err(BLOCKED.into());
    }
    let stage = root.join(format!("apply-stage-{}", uuid::Uuid::new_v4()));
    io(fs::create_dir(&stage))?;
    // No game write is permitted until the entire backup is durable and published.
    let backup = stage.join("Config");
    crate::profiles::copy_tree(config, &backup)?;
    if crate::profiles::file_map(&backup)? != *before
        || crate::profiles::file_map(config)? != *before
    {
        return Err("As configurações mudaram durante a preparação".into());
    }
    for entry in walkdir::WalkDir::new(&backup) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_file() {
            io(fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(entry.path()))?
            .sync_all()
            .map_err(|e| e.to_string())?;
        }
    }
    let record = Record {
        version: 2,
        subject: subject.into(),
        before_cloud: before_cloud.into(),
        after_cloud: after_cloud.into(),
        before_local: before.clone(),
        after_local: after.clone(),
    };
    let bytes = serde_json::to_vec(&record).map_err(|e| e.to_string())?;
    let mut file = io(fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(stage.join("record.json")))?;
    io(file.write_all(&bytes))?;
    io(file.sync_all())?;
    drop(file);
    if pending_exists(root)? {
        return Err(BLOCKED.into());
    }
    io(fs::rename(&stage, root.join(PENDING)))?;
    Ok(())
}

fn reconcile_at(root: &Path, config: &Path, subject: &str, cloud: &str, retain: bool) -> R<()> {
    if !pending_exists(root)? {
        return Ok(());
    }
    let pending = root.join(PENDING);
    let record_path = pending.join("record.json");
    crate::storage_lock::reject_reparse(&record_path, false)?;
    crate::storage_lock::reject_reparse(&pending.join("Config"), true)?;
    let record: Record =
        serde_json::from_slice(&io(fs::read(record_path))?).map_err(|_| BLOCKED)?;
    if record.version != 2
        || record.subject != subject
        || uuid::Uuid::parse_str(subject).is_err()
        || crate::profiles::file_map(&pending.join("Config"))? != record.before_local
    {
        return Err(BLOCKED.into());
    }
    let local = crate::profiles::file_map(config).map_err(|_| BLOCKED)?;
    if classify(&record, cloud, &local) == State::Ambiguous {
        return Err(BLOCKED.into());
    }
    // Resolve atomically; retain interrupted-operation evidence, not routine duplicates.
    let resolved = root.join(format!("apply-resolved-{}", uuid::Uuid::new_v4()));
    io(fs::rename(pending, &resolved))?;
    if !retain {
        let _ = fs::remove_dir_all(&resolved);
    }
    Ok(())
}

pub(crate) fn begin(
    session: &crate::riot::Session,
    before_cloud: &str,
    after_cloud: &str,
    before: &FileMap,
    after: &FileMap,
) -> R<()> {
    let subject = session
        .subject
        .as_deref()
        .ok_or("Identidade Riot indisponível")?;
    begin_at(
        &lock_root()?,
        &crate::profiles::config_root()?,
        subject,
        before_cloud,
        after_cloud,
        before,
        after,
    )
}

pub(crate) async fn reconcile(session: &crate::riot::Session, retain: bool) -> R<()> {
    let root = lock_root()?;
    if !pending_exists(&root)? {
        return Ok(());
    }
    crate::profiles::require_game_closed()?;
    crate::riot::assert_current_session(session).await?;
    let cloud = crate::riot::read_for(session).await?;
    crate::riot::assert_current_session(session).await?;
    crate::profiles::require_game_closed()?;
    reconcile_at(
        &root,
        &crate::profiles::config_root()?,
        session.subject.as_deref().ok_or(BLOCKED)?,
        &cloud,
        retain,
    )
}

pub(crate) async fn reconcile_pending() -> R<()> {
    if pending_exists(&lock_root()?)? {
        reconcile(&crate::riot::session().await?, true).await?;
    }
    Ok(())
}

