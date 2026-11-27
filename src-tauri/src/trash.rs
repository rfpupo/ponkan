//! Managed recycle bin. Legacy archives under deleted/ are never touched.
use crate::profiles::{self, Profile};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
type R<T> = Result<T, String>;
pub static LOCK: Mutex<()> = Mutex::new(());
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub profile: Profile,
    #[serde(with = "time::serde::rfc3339")]
    pub deleted_at: OffsetDateTime,
    pub avatar: Option<String>,
    pub favorite: bool,
}
fn io<T>(r: std::io::Result<T>) -> R<T> {
    r.map_err(|e| format!("Falha ao acessar arquivos: {e}"))
}
fn id_ok(id: &str) -> R<()> {
    if Uuid::parse_str(id)
        .ok()
        .is_some_and(|x| x.to_string() == id)
    {
        Ok(())
    } else {
        Err("Identificador inválido".into())
    }
}
fn safe_dir(p: &Path) -> R<()> {
    let m = io(fs::symlink_metadata(p))?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err("Pasta de lixeira insegura.".into());
        }
    }
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err("Pasta de lixeira insegura.".into());
    }
    Ok(())
}
pub(crate) fn atomic_json<T: Serialize + ?Sized>(p: &Path, value: &T) -> R<()> {
    let parent = p.parent().ok_or("Caminho inválido")?;
    io(fs::create_dir_all(parent))?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let tmp = parent.join(format!(".ponkan-{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let mut f = io(fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp))?;
        io(f.write_all(&bytes))?;
        io(f.sync_all())?;
        drop(f);
        io(fs::rename(&tmp, p))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}
fn read_active(root: &Path) -> R<Vec<Profile>> {
    let p = root.join("index.json");
    if !p.exists() {
        return Ok(vec![]);
    }
    serde_json::from_slice(&io(fs::read(p))?).map_err(|_| "Lista de perfis danificada".into())
}
fn write_active(root: &Path, active: &[Profile]) -> R<()> {
    atomic_json(&root.join("index.json"), active)
}
fn entry_dir(root: &Path, id: &str) -> R<PathBuf> {
    id_ok(id)?;
    safe_dir(&root.join("trash-v1"))?;
    let p = root.join("trash-v1").join(id);
    safe_dir(&p)?;
    Ok(p)
}
fn read_entry(root: &Path, id: &str) -> R<Entry> {
    let p = entry_dir(root, id)?.join("entry.json");
    let m = io(fs::symlink_metadata(&p))?;
    if m.file_type().is_symlink() || !m.is_file() || m.len() > 512 * 1024 {
        return Err("Registro da lixeira inválido.".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err("Registro da lixeira inválido.".into());
        }
    }
    let e: Entry =
        serde_json::from_slice(&io(fs::read(p))?).map_err(|_| "Registro da lixeira inválido.")?;
    if e.profile.id != id || profiles::validate_name(&e.profile.name).is_err() {
        return Err("Registro da lixeira inválido.".into());
    }
    Ok(e)
}
fn list_at(root: &Path) -> R<Vec<Entry>> {
    let base = root.join("trash-v1");
    if !base.exists() {
        return Ok(vec![]);
    }
    safe_dir(&base)?;
    let active = read_active(root)?;
    let mut out = vec![];
    for d in io(fs::read_dir(base))? {
        let d = io(d)?;
        let id = d.file_name().to_string_lossy().into_owned();
        if id_ok(&id).is_err()
            || active.iter().any(|p| p.id == id)
            || root.join("profiles").join(&id).exists()
        {
            continue;
        }
        out.push(read_entry(root, &id)?);
    }
    out.sort_by(|a, b| {
        b.deleted_at
            .cmp(&a.deleted_at)
            .then(a.profile.id.cmp(&b.profile.id))
    });
    Ok(out)
}
fn move_to_trash(
    root: &Path,
    id: &str,
    avatar: Option<String>,
    favorite: bool,
    now: OffsetDateTime,
) -> R<Entry> {
    id_ok(id)?;
    let mut active = read_active(root)?;
    let profile = active
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or("Perfil não encontrado")?;
    let original = root.join("profiles").join(id);
    safe_dir(&root.join("profiles"))?;
    safe_dir(&original)?;
    let history = root.join("history").join(id);
    if history.exists() {
        safe_dir(&root.join("history"))?;
        safe_dir(&history)?
    }
    let base = root.join("trash-v1");
    io(fs::create_dir_all(&base))?;
    safe_dir(&base)?;
    let dir = base.join(id);
    io(fs::create_dir(&dir))?;
    let avatar =
        avatar.filter(|x| x.starts_with("data:image/jpeg;base64,") && x.len() <= 256 * 1024);
    let e = Entry {
        profile,
        deleted_at: now,
        avatar,
        favorite,
    };
    let result = (|| {
        atomic_json(&dir.join("entry.json"), &e)?;
        io(fs::rename(&original, dir.join("profile")))?;
        if history.exists() {
            io(fs::rename(&history, dir.join("history")))?
        }
        active.retain(|p| p.id != id);
        write_active(root, &active)
    })();
    if let Err(error) = result {
        if dir.join("profile").exists() && !original.exists() {
            let _ = fs::rename(dir.join("profile"), &original);
        }
        if dir.join("history").exists() && !history.exists() {
            let _ = fs::rename(dir.join("history"), &history);
        }
        if !dir.join("profile").exists() && !dir.join("history").exists() {
            let _ = fs::remove_file(dir.join("entry.json"));
            let _ = fs::remove_dir(&dir);
        }
        return Err(error);
    }
    Ok(e)
}
fn unique_name(name: &str, active: &[Profile]) -> R<String> {
    if !active.iter().any(|p| p.name.eq_ignore_ascii_case(name)) {
        return Ok(name.into());
    }
    for i in 2..10000 {
        let suffix = format!(" ({i})");
        let prefix: String = name.chars().take(48 - suffix.len()).collect();
        let n = format!("{}{suffix}", prefix.trim_end());
        if !active.iter().any(|p| p.name.eq_ignore_ascii_case(&n)) {
            return Ok(n);
        }
    }
    Err("Não foi possível escolher um nome para o perfil.".into())
}
fn restore_at(root: &Path, id: &str) -> R<Entry> {
    let mut e = read_entry(root, id)?;
    let dir = entry_dir(root, id)?;
    let mut active = read_active(root)?;
    let original = root.join("profiles").join(id);
    let history = root.join("history").join(id);
    if active.iter().any(|p| p.id == id) || original.exists() || history.exists() {
        return Err("Perfil já existente.".into());
    }
    safe_dir(&dir.join("profile"))?;
    io(fs::create_dir_all(root.join("profiles")))?;
    safe_dir(&root.join("profiles"))?;
    if dir.join("history").exists() {
        safe_dir(&dir.join("history"))?;
        io(fs::create_dir_all(root.join("history")))?;
        safe_dir(&root.join("history"))?
    }
    e.profile.name = unique_name(&e.profile.name, &active)?;
    let result = (|| {
        io(fs::rename(dir.join("profile"), &original))?;
        if dir.join("history").exists() {
            io(fs::rename(dir.join("history"), &history))?
        }
        active.push(e.profile.clone());
        write_active(root, &active)
    })();
    if let Err(error) = result {
        if original.exists() && !dir.join("profile").exists() {
            let _ = fs::rename(&original, dir.join("profile"));
        }
        if history.exists() && !dir.join("history").exists() {
            let _ = fs::rename(&history, dir.join("history"));
        }
        return Err(error);
    }
    let _ = fs::remove_file(dir.join("entry.json"));
    let _ = fs::remove_dir(dir);
    Ok(e)
}
fn safe_tree(p: &Path) -> R<()> {
    safe_dir(p)?;
    for e in walkdir::WalkDir::new(p).follow_links(false) {
        let e = e.map_err(|e| e.to_string())?;
        let m = io(fs::symlink_metadata(e.path()))?;
        if m.file_type().is_symlink() {
            return Err("Pasta de lixeira insegura.".into());
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if m.file_attributes() & 0x400 != 0 {
                return Err("Pasta de lixeira insegura.".into());
            }
        }
    }
    Ok(())
}
fn purge_at(root: &Path, id: &str) -> R<()> {
    read_entry(root, id)?;
    let dir = entry_dir(root, id)?;
    if read_active(root)?.iter().any(|p| p.id == id) || root.join("profiles").join(id).exists() {
        return Err("Perfil ainda ativo.".into());
    }
    safe_tree(&dir)?;
    // No unknown files are expected. Check before making any irreversible change.
    if io(fs::read_dir(&dir))?.any(|e| {
        e.map(|d| {
            !["entry.json", "profile", "history"]
                .iter()
                .any(|n| d.file_name() == *n)
        })
        .unwrap_or(true)
    }) {
        return Err("Registro da lixeira inválido.".into());
    }
    // Keep metadata until the data folders are removed, so a failed deletion can be retried.
    for name in ["profile", "history"] {
        let p = dir.join(name);
        if p.exists() {
            io(fs::remove_dir_all(p))?
        }
    }
    io(fs::remove_file(dir.join("entry.json")))?;
    io(fs::remove_dir(dir))
}
fn expire_at(root: &Path, days: u16, now: OffsetDateTime) -> R<Vec<String>> {
    if days == 0 {
        return Ok(vec![]);
    }
    if ![1, 7, 15, 30, 60, 90].contains(&days) {
        return Err("Prazo inválido.".into());
    }
    let mut ids = vec![];
    for e in list_at(root)? {
        if (now - e.deleted_at) >= Duration::days(days.into()) {
            purge_at(root, &e.profile.id)?;
            ids.push(e.profile.id)
        }
    }
    Ok(ids)
}
// Recover interrupted moves against the authoritative active index. Never purge ambiguous data.
pub fn recover() -> R<()> {
    let _g = LOCK.lock().map_err(|_| "Lixeira indisponível.")?;
    let root = profiles::data_root()?;
    let base = root.join("trash-v1");
    if !base.exists() {
        return Ok(());
    }
    safe_dir(&base)?;
    let active = read_active(&root)?;
    for d in io(fs::read_dir(&base))? {
        let d = io(d)?;
        let id = d.file_name().to_string_lossy().into_owned();
        if id_ok(&id).is_err() {
            continue;
        }
        read_entry(&root, &id)?;
        let dir = d.path();
        let original = root.join("profiles").join(&id);
        let history = root.join("history").join(&id);
        let is_active = active.iter().any(|p| p.id == id);
        if is_active {
            if !original.exists() && dir.join("profile").exists() {
                safe_dir(&root.join("profiles"))?;
                safe_dir(&dir.join("profile"))?;
                io(fs::rename(dir.join("profile"), &original))?
            }
            if !history.exists() && dir.join("history").exists() {
                io(fs::create_dir_all(root.join("history")))?;
                safe_dir(&root.join("history"))?;
                safe_dir(&dir.join("history"))?;
                io(fs::rename(dir.join("history"), &history))?
            }
            if original.exists() && !dir.join("profile").exists() && !dir.join("history").exists() {
                let _ = fs::remove_file(dir.join("entry.json"));
                let _ = fs::remove_dir(dir);
            }
        } else if original.exists() && !dir.join("profile").exists() {
            safe_dir(&root.join("profiles"))?;
            safe_dir(&original)?;
            io(fs::rename(&original, dir.join("profile")))?;
            if history.exists() && !dir.join("history").exists() {
                safe_dir(&root.join("history"))?;
                safe_dir(&history)?;
                io(fs::rename(history, dir.join("history")))?
            }
        }
    }
    Ok(())
}
pub fn list() -> R<Vec<Entry>> {
    let _g = LOCK.lock().map_err(|_| "Lixeira indisponível.")?;
    list_at(&profiles::data_root()?)
}
pub fn delete(id: &str, avatar: Option<String>, favorite: bool) -> R<Entry> {
    let _g = LOCK.lock().map_err(|_| "Lixeira indisponível.")?;
    move_to_trash(
        &profiles::data_root()?,
        id,
        avatar,
        favorite,
        OffsetDateTime::now_utc(),
    )
}
pub fn restore(id: &str) -> R<Entry> {
    let _g = LOCK.lock().map_err(|_| "Lixeira indisponível.")?;
    restore_at(&profiles::data_root()?, id)
}
pub fn purge(id: &str) -> R<()> {
    let _g = LOCK.lock().map_err(|_| "Lixeira indisponível.")?;
    purge_at(&profiles::data_root()?, id)
}
pub fn expire(days: u16) -> R<Vec<String>> {
    let _g = LOCK.lock().map_err(|_| "Lixeira indisponível.")?;
    expire_at(&profiles::data_root()?, days, OffsetDateTime::now_utc())
}

