use crate::profiles::{copy_tree, file_map};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
type R<T> = Result<T, String>;
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn account_dirs(root: &Path) -> R<Vec<String>> {
    let mut v = vec![];
    for e in fs::read_dir(root).map_err(error)? {
        let e = e.map_err(error)?;
        if e.file_type().map_err(error)?.is_symlink() {
            return Err("Link simbólico em configurações".into());
        }
        let n = e.file_name().to_string_lossy().to_string();
        if e.path().is_dir()
            && n.get(..36)
                .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
            && (n.len() == 36 || n.as_bytes().get(36) == Some(&b'-'))
        {
            v.push(n);
        }
    }
    Ok(v)
}
fn select_account(root: &Path, subject: &str, source: bool) -> R<String> {
    let all = account_dirs(root)?;
    if source {
        if all.len() == 1 {
            return Ok(all[0].clone());
        }
        return Err("Não foi possível identificar a conta de origem no perfil".into());
    }
    let matches: Vec<_> = all
        .iter()
        .filter(|s| {
            s.get(..36)
                .is_some_and(|id| id.eq_ignore_ascii_case(subject))
        })
        .collect();
    if matches.len() == 1 {
        return Ok(matches[0].clone());
    }
    Err(if source{"Não foi possível identificar a conta de origem no perfil"}else{"Abra o VALORANT nesta conta ao menos uma vez e feche-o antes de carregar o perfil importado"}.into())
}
fn protected(key: &str) -> bool {
    let k = key.trim().trim_start_matches('+').to_ascii_lowercase();
    [
        "monitor",
        "device",
        "hardware",
        "benchmark",
        "windowpos",
        "lastrecommended",
        "adapter",
        "machine",
        "account",
        "puuid",
        "token",
        "password",
        "credential",
        "secret",
        "authorization",
        "hasever",
        "hasseen",
        "hasaccepted",
        "lastaccepted",
        "lastseen",
        "lastpatchstory",
        "contextawaremodulecomplete",
        "hasshown",
        "lastknownuser",
    ]
    .iter()
    .any(|p| k.contains(p))
}
fn ini_merge(source: &str, target: &str) -> String {
    // Preserve destination-specific fields. Copy every other line in the source verbatim.
    // Preserve duplicate/array entries and unknown settings.
    let mut output = String::new();
    let mut protected_lines = std::collections::BTreeMap::<String, Vec<String>>::new();
    let mut section = String::new();
    for line in target.lines() {
        if line.starts_with('[') {
            section = line.to_owned();
        } else if line.split_once('=').is_some_and(|(k, _)| protected(k)) {
            protected_lines
                .entry(section.clone())
                .or_default()
                .push(line.to_owned());
        }
    }
    let mut seen = BTreeSet::new();
    section.clear();
    let flush = |out: &mut String, sec: &str, seen: &mut BTreeSet<String>| {
        if seen.insert(sec.to_owned()) {
            if let Some(lines) = protected_lines.get(sec) {
                for l in lines {
                    out.push_str(l);
                    out.push_str("\r\n");
                }
            }
        }
    };
    for line in source.lines() {
        if line.starts_with('[') {
            flush(&mut output, &section, &mut seen);
            section = line.to_owned();
        }
        if line.split_once('=').is_some_and(|(k, _)| protected(k)) {
            continue;
        }
        output.push_str(line);
        output.push_str("\r\n");
    }
    flush(&mut output, &section, &mut seen);
    for sec in protected_lines.keys() {
        if !seen.contains(sec) {
            output.push_str("\r\n");
            output.push_str(sec);
            output.push_str("\r\n");
            flush(&mut output, sec, &mut seen);
        }
    }
    output
}
#[cfg(target_os = "windows")]
fn resolution_supported(w: u32, h: u32) -> bool {
    use windows_sys::Win32::Graphics::Gdi::{EnumDisplaySettingsW, DEVMODEW};
    unsafe {
        for i in 0..4096 {
            let mut mode: DEVMODEW = std::mem::zeroed();
            mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
            if EnumDisplaySettingsW(std::ptr::null(), i, &mut mode) == 0 {
                break;
            }
            if mode.dmPelsWidth == w && mode.dmPelsHeight == h {
                return true;
            }
        }
    }
    false
}
#[cfg(not(target_os = "windows"))]
fn resolution_supported(_w: u32, _h: u32) -> bool {
    false
}
fn dimensions(s: &str) -> Option<(u32, u32)> {
    let mut w = None;
    let mut h = None;
    for line in s.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k == "ResolutionSizeX" {
                w = v.trim().parse().ok()
            }
            if k == "ResolutionSizeY" {
                h = v.trim().parse().ok()
            }
        }
    }
    Some((w?, h?))
}
fn transfer(source: &Path, target: &Path, check_display: bool) -> R<()> {
    if !source.is_file() {
        return Ok(());
    }
    if let Some(p) = target.parent() {
        fs::create_dir_all(p).map_err(error)?;
    }
    if source
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("ini"))
    {
        let s = fs::read_to_string(source).map_err(error)?;
        let t = if target.exists() {
            fs::read_to_string(target).map_err(error)?
        } else {
            String::new()
        };
        if check_display
            && source.file_name().and_then(|n| n.to_str()) == Some("GameUserSettings.ini")
        {
            if let Some((w, h)) = dimensions(&s) {
                if dimensions(&t) != Some((w, h)) && !resolution_supported(w, h) {
                    return Err(format!(
                        "Resolução não disponível neste computador: {w}x{h}"
                    ));
                }
            }
        }
        fs::write(target, ini_merge(&s, &t)).map_err(error)?;
    } else {
        fs::copy(source, target).map_err(error)?;
    }
    Ok(())
}
pub struct Plan {
    target: PathBuf,
    stage: PathBuf,
    original: PathBuf,
    before: std::collections::BTreeMap<PathBuf, (u64, String)>,
    expected: std::collections::BTreeMap<PathBuf, (u64, String)>,
}
impl Drop for Plan {
    fn drop(&mut self) {
        if self.stage.exists() {
            let _ = fs::remove_dir_all(&self.stage);
        }
    }
}
impl Plan {
    pub fn prepare_full(source: &Path, target: &Path) -> R<Self> {
        let parent = target.parent().ok_or("Caminho inválido")?;
        let id = uuid::Uuid::new_v4();
        let mut plan = Self {
            target: target.into(),
            stage: parent.join(format!("Config-stage-{id}")),
            original: parent.join(format!("Config-old-{id}")),
            before: file_map(target)?,
            expected: Default::default(),
        };
        copy_tree(source, &plan.stage)?;
        plan.expected = file_map(&plan.stage)?;
        Ok(plan)
    }
    pub(crate) fn maps(
        &self,
    ) -> (
        &crate::apply_recovery::FileMap,
        &crate::apply_recovery::FileMap,
    ) {
        (&self.before, &self.expected)
    }
    pub(crate) fn verify_before(&self) -> R<()> {
        if file_map(&self.target)? != self.before || file_map(&self.stage)? != self.expected {
            return Err("As configurações locais mudaram durante a operação".into());
        }
        Ok(())
    }
    pub(crate) fn verify_after(&self) -> R<()> {
        if file_map(&self.target)? != self.expected {
            return Err("Falha na verificação das configurações locais".into());
        }
        Ok(())
    }
    pub fn prepare(source: &Path, target: &Path, subject: &str) -> R<Self> {
        Self::prepare_inner(source, target, subject, true)
    }
    fn prepare_inner(source: &Path, target: &Path, subject: &str, check_display: bool) -> R<Self> {
        uuid::Uuid::parse_str(subject).map_err(error)?;
        let src = select_account(source, subject, true)?;
        let dst = select_account(target, subject, false)?;
        let parent = target.parent().ok_or("Caminho inválido")?;
        let id = uuid::Uuid::new_v4();
        let mut plan = Self {
            target: target.into(),
            stage: parent.join(format!("Config-stage-{id}")),
            original: parent.join(format!("Config-old-{id}")),
            before: file_map(target)?,
            expected: Default::default(),
        };
        copy_tree(target, &plan.stage)?;
        // Copy the known game settings files, not arbitrary foreign machine files.
        let mut count = 0;
        for rel in [
            "WindowsClient/GameUserSettings.ini",
            "WindowsClient/BackupKeybinds.json",
            "Windows/RiotUserSettings.ini",
        ] {
            let from = source.join(&src).join(rel);
            if from.is_file() {
                transfer(&from, &plan.stage.join(&dst).join(rel), check_display)?;
                count += 1;
            }
        }
        for rel in [
            "WindowsClient/GameUserSettings.ini",
            "WindowsClient/Engine.ini",
        ] {
            let from = source.join(rel);
            if from.is_file() {
                transfer(&from, &plan.stage.join(rel), check_display)?;
                count += 1;
            }
        }
        if count == 0 {
            return Err("Nenhuma configuração local portátil encontrada".into());
        }
        plan.expected = file_map(&plan.stage)?;
        Ok(plan)
    }

    pub fn prepare_shared(
        target: &Path,
        subject: &str,
        cloud: &serde_json::Value,
        video: &std::collections::BTreeMap<String, serde_json::Value>,
    ) -> R<Self> {
        uuid::Uuid::parse_str(subject).map_err(error)?;
        let dst = select_account(target, subject, false)?;
        let parent = target.parent().ok_or("Pasta de configuração inválida")?;
        let id = uuid::Uuid::new_v4();
        let mut plan = Self {
            target: target.into(),
            stage: parent.join(format!("Config-stage-{id}")),
            original: parent.join(format!("Config-old-{id}")),
            before: file_map(target)?,
            expected: Default::default(),
        };
        copy_tree(target, &plan.stage)?;
        crate::portable::patch_shared_local(&plan.stage, &dst, cloud, video)?;
        plan.expected = file_map(&plan.stage)?;
        Ok(plan)
    }
    pub fn commit(&self) -> R<()> {
        self.verify_before()?;
        fs::rename(&self.target, &self.original).map_err(error)?;
        if let Err(e) = fs::rename(&self.stage, &self.target) {
            if matches!(fs::symlink_metadata(&self.target), Err(ref err) if err.kind() == std::io::ErrorKind::NotFound)
                && file_map(&self.original).ok().as_ref() == Some(&self.before)
            {
                fs::rename(&self.original, &self.target).map_err(error)?;
            }
            return Err(error(e));
        }
        if self.verify_after().is_err() {
            self.rollback()?;
            return Err("Falha na verificação das configurações locais".into());
        }
        Ok(())
    }
    pub fn rollback(&self) -> R<()> {
        const CONFLICT: &str =
            "Configurações ou backup alterados; dados preservados para recuperação";
        match fs::symlink_metadata(&self.original) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(error(e)),
            Ok(_) => {}
        }
        if file_map(&self.original)? != self.before || file_map(&self.target)? != self.expected {
            return Err(CONFLICT.into());
        }
        // Do not reuse the staging directory: Drop could delete displaced data.
        let displaced = self
            .target
            .parent()
            .ok_or(CONFLICT)?
            .join(format!("Config-rollback-{}", uuid::Uuid::new_v4()));
        fs::rename(&self.target, &displaced).map_err(error)?;
        let restore_displaced = || {
            if matches!(fs::symlink_metadata(&self.target), Err(ref e) if e.kind() == std::io::ErrorKind::NotFound)
            {
                let _ = fs::rename(&displaced, &self.target);
            }
        };
        if file_map(&displaced).ok().as_ref() != Some(&self.expected)
            || file_map(&self.original).ok().as_ref() != Some(&self.before)
        {
            restore_displaced();
            return Err(CONFLICT.into());
        }
        if let Err(e) = fs::rename(&self.original, &self.target) {
            restore_displaced();
            return Err(error(e));
        }
        if file_map(&self.target)? != self.before {
            return Err(CONFLICT.into());
        }
        if file_map(&displaced).ok().as_ref() == Some(&self.expected) {
            let _ = fs::remove_dir_all(&displaced);
        }
        Ok(())
    }
    pub fn finish(&self) {
        if file_map(&self.original).ok().as_ref() == Some(&self.before) {
            let _ = fs::remove_dir_all(&self.original);
        }
    }
}
