//! Versioned, self-contained profile links. No network or Riot calls in import/export.
pub(crate) mod short;
use crate::{
    portable,
    profiles::{self, Profile},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    io::{Read, Write},
    path::Path,
    sync::Mutex,
};
use uuid::Uuid;
type R<T> = Result<T, String>;
const MAX_CODE: usize = 8192;
const MAX_JSON: usize = 32 * 1024;
const INVALID: &str = "Link de perfil inválido ou incompatível.";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Share {
    pub v: u8,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub settings: BTreeMap<String, Value>,
    #[serde(default)]
    pub video: BTreeMap<String, Value>,
}
pub(crate) fn scheme() -> &'static str {
    if cfg!(debug_assertions) {
        "ponkan-dev"
    } else {
        "ponkan"
    }
}
pub(crate) fn validate(p: &Share) -> R<()> {
    if p.v != 1
        || profiles::validate_name(&p.name).is_err()
        || p.author
            .as_ref()
            .is_some_and(|x| x.chars().count() > 80 || x.chars().any(char::is_control))
        || p.description.as_ref().is_some_and(|x| {
            x.chars().count() > 512 || x.chars().any(|c| c.is_control() && c != '\n')
        })
        || p.settings.len() + p.video.len() == 0
        || p.settings.len() > 64
        || p.video.len() > 32
    {
        return Err(INVALID.into());
    }
    portable::share_cloud(&p.settings)?;
    portable::validate_share_video(&p.video)?;
    Ok(())
}
fn digest(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn encode(p: &Share) -> R<String> {
    validate(p)?;
    let raw = serde_json::to_vec(p).map_err(|_| INVALID)?;
    if raw.len() > MAX_JSON {
        return Err(INVALID.into());
    }
    let mut z = DeflateEncoder::new(Vec::new(), Compression::best());
    z.write_all(&raw).map_err(|_| INVALID)?;
    let packed = z.finish().map_err(|_| INVALID)?;
    let c = format!(
        "PKN1.{}.{}",
        URL_SAFE_NO_PAD.encode(&packed),
        digest(&packed)
    );
    if c.len() > MAX_CODE {
        return Err("Perfil grande demais para compartilhar por link.".into());
    }
    Ok(c)
}
fn decode(code: &str) -> R<Share> {
    if code.len() > MAX_CODE {
        return Err(INVALID.into());
    }
    let mut parts = code.split('.');
    if parts.next() != Some("PKN1") {
        return Err(INVALID.into());
    }
    let b = parts.next().ok_or(INVALID)?;
    let h = parts.next().ok_or(INVALID)?;
    if parts.next().is_some()
        || b.is_empty()
        || h.len() != 64
        || !b
            .bytes()
            .all(|x| x.is_ascii_alphanumeric() || x == b'-' || x == b'_')
        || !h.bytes().all(|x| x.is_ascii_hexdigit())
    {
        return Err(INVALID.into());
    }
    let packed = URL_SAFE_NO_PAD.decode(b).map_err(|_| INVALID)?;
    if digest(&packed) != h {
        return Err(INVALID.into());
    }
    let mut raw = Vec::new();
    DeflateDecoder::new(packed.as_slice())
        .take(MAX_JSON as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|_| INVALID)?;
    if raw.len() > MAX_JSON {
        return Err(INVALID.into());
    }
    let p: Share = serde_json::from_slice(&raw).map_err(|_| INVALID)?;
    validate(&p)?;
    Ok(p)
}
fn decode_uri(uri: &str) -> R<Share> {
    let prefix = format!("{}://import/", scheme());
    decode(uri.strip_prefix(&prefix).ok_or(INVALID)?)
}
pub(crate) fn load(id: &str) -> R<Share> {
    profiles::find(id)?;
    Uuid::parse_str(id).map_err(|_| INVALID)?;
    let path = profiles::data_root()?
        .join("profiles")
        .join(id)
        .join("share.json");
    let raw = fs::read(path).map_err(|_| INVALID)?;
    if raw.len() > MAX_JSON {
        return Err(INVALID.into());
    }
    let p = serde_json::from_slice(&raw).map_err(|_| INVALID)?;
    validate(&p)?;
    Ok(p)
}
pub(crate) fn make(id: &str) -> R<String> {
    let p = profiles::find(id)?;
    let mut share = if p.shared == Some(true) {
        load(id)?
    } else {
        let cloud = portable::decode_share_cloud(&profiles::cloud_for(id)?)?;
        let settings = portable::share_fields(&cloud)?;
        let video = portable::share_video_fields(&profiles::config_for(id)?)?;
        Share {
            v: 1,
            name: p.name.clone(),
            author: p.author.clone(),
            description: p.description.clone(),
            settings,
            video,
        }
    };
    share.name = p.name;
    share.author = p.author;
    share.description = p.description;
    encode(&share)
}
fn import_at(root: &Path, p: &Share) -> R<Profile> {
    validate(p)?;
    let idx = root.join("index.json");
    let existing: Vec<Profile> = if idx.exists() {
        serde_json::from_slice(&fs::read(idx).map_err(|e| e.to_string())?)
            .map_err(|_| "Lista de perfis danificada")?
    } else {
        vec![]
    };
    for item in &existing {
        if item.shared != Some(true) || Uuid::parse_str(&item.id).is_err() {
            continue;
        }
        if let Ok(bytes) = fs::read(root.join("profiles").join(&item.id).join("share.json")) {
            if bytes.len() <= MAX_JSON
                && serde_json::from_slice::<Share>(&bytes).ok().as_ref() == Some(p)
            {
                return Ok(item.clone());
            }
        }
    }
    let name = if !existing
        .iter()
        .any(|x| x.name.eq_ignore_ascii_case(&p.name))
    {
        p.name.clone()
    } else {
        (2..10000)
            .find_map(|i| {
                let suffix = format!(" ({i})");
                let prefix: String = p.name.chars().take(48 - suffix.len()).collect();
                let n = format!("{prefix}{suffix}");
                (!existing.iter().any(|x| x.name.eq_ignore_ascii_case(&n))).then_some(n)
            })
            .ok_or("Não foi possível escolher um nome para o perfil.")?
    };
    let manifest = portable::Manifest {
        format: "ponkan".into(),
        format_version: 1,
        id: Uuid::new_v4().to_string(),
        name: name.clone(),
        author: p.author.clone(),
        description: p.description.clone(),
        avatar: None,
        created_at: None,
        updated_at: profiles::timestamp_now()?,
        app_version: env!("CARGO_PKG_VERSION").into(),
    };
    let cloud = portable::encode_share_cloud(&portable::share_cloud(&p.settings)?)?;
    let files = BTreeMap::from([
        (
            "raw/share.json".into(),
            serde_json::to_vec(p).map_err(|_| INVALID)?,
        ),
        ("raw/cloud.txt".into(), cloud.into_bytes()),
    ]);
    profiles::import_portable_at(root, name, manifest, files)
}
pub(crate) fn import_uri(uri: &str) -> R<Profile> {
    import_at(&profiles::data_root()?, &decode_uri(uri)?)
}
#[derive(Default)]
pub(crate) struct Queue(Mutex<VecDeque<String>>);
impl Queue {
    pub(crate) fn push_args(&self, args: impl IntoIterator<Item = String>) -> bool {
        #[cfg(debug_assertions)]
        {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::env::temp_dir().join("ponkan-dev-deeplink.log"))
            {
                let _ = writeln!(f, "QUEUE pid={}", std::process::id());
            }
        }
        let prefix = format!("{}://import/", scheme());
        let Ok(mut q) = self.0.lock() else {
            return false;
        };
        let mut received = false;
        for s in args {
            if s.starts_with(&prefix)
                && s.len() <= MAX_CODE + prefix.len()
                && q.len() < 8
                && !q.contains(&s)
            {
                q.push_back(s);
                received = true
            }
        }
        received
    }
    pub(crate) fn has_pending(&self) -> R<bool> {
        Ok(!self.0.lock().map_err(|_| INVALID)?.is_empty())
    }
    pub(crate) fn pop(&self) -> R<Option<String>> {
        Ok(self.0.lock().map_err(|_| INVALID)?.pop_front())
    }
}
