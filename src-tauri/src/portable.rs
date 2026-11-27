
use base64::{engine::general_purpose::STANDARD, Engine as _};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};
use uuid::Uuid;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const MAX_ARCHIVE: u64 = 32 * 1024 * 1024;
const MAX_FILE: u64 = 16 * 1024 * 1024;
const MAX_TOTAL: u64 = 64 * 1024 * 1024;
const MAX_FILES: usize = 1024;
type Files = BTreeMap<String, Vec<u8>>;
type R<T> = Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn sha(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn pretty(v: &impl Serialize) -> R<Vec<u8>> {
    serde_json::to_vec_pretty(v).map_err(err)
}
fn parse<T: serde::de::DeserializeOwned>(b: &[u8]) -> R<T> {
    serde_json::from_slice(b).map_err(err)
}
fn safe_path(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 512
        && s.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.ends_with(['.', ' '])
                && !p
                    .chars()
                    .any(|c| c.is_control() || "\\:*?\"<>|".contains(c))
                && {
                    let stem = p.split('.').next().unwrap_or("").to_ascii_uppercase();
                    !matches!(
                        stem.as_str(),
                        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
                    ) && !(stem.len() == 4
                        && (stem.starts_with("COM") || stem.starts_with("LPT"))
                        && stem.as_bytes()[3].is_ascii_digit())
                }
        })
}
fn allowed(s: &str) -> bool {
    matches!(
        s,
        "editor-draft.json"
            | "profile.json"
            | "settings.json"
            | "integrity.json"
            | "assets/avatar.webp"
            | "assets/avatar.jpeg"
            | "assets/avatar.png"
            | "raw/cloud.txt"
            | "raw/share.json"
    ) || (s.starts_with("raw/Config/")
        && s.rsplit('/').next().is_some_and(|n| {
            let n = n.to_ascii_lowercase();
            (n.ends_with(".ini") || n.ends_with(".json"))
                && !n.contains("credential")
                && !n.contains("lockfile")
                && !n.contains("token")
                && !n.contains("password")
        }))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub format_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub updated_at: String,
    pub app_version: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Integrity {
    algorithm: String,
    files: BTreeMap<String, String>,
}
#[derive(Serialize)]
pub struct Preview {
    pub manifest: Manifest,
    pub edited: bool,
    pub duplicate: bool,
    pub suggested_name: String,
    pub avatar: Option<String>,
}
pub struct Package {
    pub manifest: Manifest,
    pub files: Files,
    pub edited: bool,
}
fn decode(b: &[u8]) -> R<Value> {
    let raw = STANDARD
        .decode(b)
        .map_err(|_| "Configurações Riot inválidas")?;
    let mut plain = Vec::new();
    DeflateDecoder::new(raw.as_slice())
        .take(MAX_FILE + 1)
        .read_to_end(&mut plain)
        .map_err(err)?;
    if plain.len() as u64 > MAX_FILE {
        return Err("Configurações Riot excedem o limite".into());
    }
    let value: Value = parse(&plain)?;
    if !value.is_object()
        || !value.get("floatSettings").is_some_and(Value::is_array)
        || !value.get("boolSettings").is_some_and(Value::is_array)
        || !value.get("actionMappings").is_some_and(Value::is_array)
    {
        return Err("Estrutura de configurações Riot não reconhecida".into());
    }
    Ok(value)
}
fn encode(v: &Value) -> R<Vec<u8>> {
    let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
    enc.write_all(&serde_json::to_vec(v).map_err(err)?)
        .map_err(err)?;
    Ok(STANDARD.encode(enc.finish().map_err(err)?).into_bytes())
}
// Each binding is grounded in observed Riot enums, not guessed default values.
struct Field {
    group: &'static str,
    category: &'static str,
    section: &'static str,
    key: &'static str,
    game: &'static str,
    min: f64,
    max: f64,
}
const FIELDS: &[Field] = &[
    Field {
        group: "floatSettings",
        category: "general",
        section: "mouse",
        key: "sensitivity",
        game: "EAresFloatSettingName::MouseSensitivity",
        min: 0.01,
        max: 10.0,
    },
    Field {
        group: "floatSettings",
        category: "general",
        section: "mouse",
        key: "ads_multiplier",
        game: "EAresFloatSettingName::MouseSensitivityADS",
        min: 0.01,
        max: 10.0,
    },
    Field {
        group: "boolSettings",
        category: "general",
        section: "other",
        key: "left_handed",
        game: "EAresBoolSettingName::LeftHanded",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "boolSettings",
        category: "general",
        section: "minimap",
        key: "center_on_player",
        game: "EAresBoolSettingName::MinimapTranslates",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "floatSettings",
        category: "general",
        section: "minimap",
        key: "size",
        game: "EAresFloatSettingName::MinimapSize",
        min: 0.5,
        max: 2.0,
    },
    Field {
        group: "boolSettings",
        category: "general",
        section: "other",
        key: "show_blood",
        game: "EAresBoolSettingName::ShowBlood",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "boolSettings",
        category: "general",
        section: "other",
        key: "show_blood_visual",
        game: "EAresBoolSettingName::ShowBloodVisual",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "boolSettings",
        category: "audio",
        section: "voice_chat",
        key: "custom_party_voice",
        game: "EAresBoolSettingName::CustomPartyVoiceChatEnabled",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "boolSettings",
        category: "crosshair",
        section: "primary",
        key: "custom_color_enabled",
        game: "EAresBoolSettingName::CrosshairUseCustomColor",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "stringSettings",
        category: "crosshair",
        section: "primary",
        key: "color",
        game: "EAresStringSettingName::CrosshairColor",
        min: 0.,
        max: 0.,
    },
    Field {
        group: "boolSettings",
        category: "crosshair",
        section: "primary",
        key: "center_dot_enabled",
        game: "EAresBoolSettingName::CrosshairDisplayCenterDot",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "floatSettings",
        category: "crosshair",
        section: "primary",
        key: "center_dot_size",
        game: "EAresFloatSettingName::CrosshairCenterDotSize",
        min: 1.,
        max: 6.,
    },
    Field {
        group: "floatSettings",
        category: "crosshair",
        section: "inner_lines",
        key: "length",
        game: "EAresFloatSettingName::CrosshairInnerLinesLineLength",
        min: 0.,
        max: 20.,
    },
    Field {
        group: "floatSettings",
        category: "crosshair",
        section: "inner_lines",
        key: "vertical_length",
        game: "EAresFloatSettingName::CrosshairInnerLinesLineLengthVertical",
        min: 0.,
        max: 20.,
    },
    Field {
        group: "floatSettings",
        category: "crosshair",
        section: "inner_lines",
        key: "offset",
        game: "EAresFloatSettingName::CrosshairInnerLinesLineOffset",
        min: 0.,
        max: 20.,
    },
    Field {
        group: "floatSettings",
        category: "crosshair",
        section: "inner_lines",
        key: "thickness",
        game: "EAresFloatSettingName::CrosshairInnerLinesLineThickness",
        min: 0.,
        max: 10.,
    },
    Field {
        group: "floatSettings",
        category: "crosshair",
        section: "inner_lines",
        key: "opacity",
        game: "EAresFloatSettingName::CrosshairInnerLinesOpacity",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "boolSettings",
        category: "crosshair",
        section: "inner_lines",
        key: "shooting_error",
        game: "EAresBoolSettingName::CrosshairInnerLinesShowShootingError",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "floatSettings",
        category: "crosshair",
        section: "outline",
        key: "opacity",
        game: "EAresFloatSettingName::CrosshairOutlineOpacity",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "boolSettings",
        category: "crosshair",
        section: "outer_lines",
        key: "enabled",
        game: "EAresBoolSettingName::CrosshairOuterLinesShowLines",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "floatSettings",
        category: "audio",
        section: "volume",
        key: "master",
        game: "EAresFloatSettingName::OverallVolume",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "floatSettings",
        category: "audio",
        section: "volume",
        key: "music",
        game: "EAresFloatSettingName::AllMusicOverallVolume",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "floatSettings",
        category: "audio",
        section: "volume",
        key: "effects",
        game: "EAresFloatSettingName::SoundEffectsVolume",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "floatSettings",
        category: "audio",
        section: "volume",
        key: "voice_over",
        game: "EAresFloatSettingName::VoiceOverVolume",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "intSettings",
        category: "audio",
        section: "voice_chat",
        key: "microphone_volume",
        game: "EAresIntSettingName::MicVolume",
        min: 0.,
        max: 100.,
    },
    Field {
        group: "intSettings",
        category: "audio",
        section: "voice_chat",
        key: "voice_volume",
        game: "EAresIntSettingName::VoiceVolume",
        min: 0.,
        max: 100.,
    },
    Field {
        group: "boolSettings",
        category: "audio",
        section: "voice_chat",
        key: "push_to_talk",
        game: "EAresBoolSettingName::PushToTalkEnabled",
        min: 0.,
        max: 1.,
    },
    Field {
        group: "boolSettings",
        category: "audio",
        section: "spatial",
        key: "hrtf",
        game: "EAresBoolSettingName::EnableHRTF",
        min: 0.,
        max: 1.,
    },
];
fn locate<'a>(v: &'a Value, f: &Field) -> Option<&'a Value> {
    let mut it = v
        .get(f.group)?
        .as_array()?
        .iter()
        .filter(|x| x.get("settingEnum").and_then(Value::as_str) == Some(f.game));
    let first = it.next()?;
    if it.next().is_some() {
        return None;
    }
    first.get("value")
}
fn view(v: &Value) -> R<Value> {
    let mut out = json!({"general":{},"controls":{},"crosshair":{},"video":{},"audio":{}});
    for f in FIELDS {
        if let Some(x) = locate(v, f) {
            if out[f.category][f.section].is_null() {
                out[f.category][f.section] = json!({});
            }
            out[f.category][f.section][f.key] = if f.key == "color" {
                color_to_hex(x).unwrap_or_else(|| x.clone())
            } else {
                x.clone()
            };
        }
    }
    // Raw bindings are retained; the readable list is informational until the full mapping is proven.
    if let Some(a) = v.get("actionMappings").and_then(Value::as_array) {
        out["controls"]["bindings"]=Value::Array(a.iter().map(|x|json!({"action":x.get("name"),"key":x.get("key"),"bind_index":x.get("bindIndex"),"shift":x.get("shift"),"ctrl":x.get("ctrl"),"alt":x.get("alt"),"character":x.get("characterName")})).collect());
    }
    Ok(out)
}
fn color_to_hex(v: &Value) -> Option<Value> {
    let s = v.as_str()?.strip_prefix('(')?.strip_suffix(')')?;
    let values: Vec<_> = s.split(',').collect();
    if values.len() != 4 {
        return None;
    }
    let mut nums = [0u8; 4];
    for (i, (k, n)) in values.iter().filter_map(|p| p.split_once('=')).enumerate() {
        if i >= 4 || k != ["R", "G", "B", "A"][i] {
            return None;
        }
        nums[i] = n.parse().ok()?;
    }
    // Require all four well-formed components, not partial values.
    if values.iter().any(|p| p.split_once('=').is_none()) {
        return None;
    }
    Some(json!(format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        nums[0], nums[1], nums[2], nums[3]
    )))
}
fn hex_to_color(v: &Value) -> Option<Value> {
    let s = v.as_str()?.strip_prefix('#')?;
    if !matches!(s.len(), 6 | 8) || !s.bytes().all(|x| x.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    let a = if s.len() == 8 {
        u8::from_str_radix(&s[6..8], 16).ok()?
    } else {
        255
    };
    Some(json!(format!("(R={r},G={g},B={b},A={a})")))
}
fn valid_value(f: &Field, v: &Value) -> bool {
    if f.group == "boolSettings" {
        return v.is_boolean();
    }
    if f.group == "stringSettings" {
        return color_to_hex(v).is_some();
    }
    if f.group == "intSettings" && !v.is_i64() {
        return false;
    }
    v.as_f64()
        .is_some_and(|n| n.is_finite() && n >= f.min && n <= f.max)
}
fn first_diff(a: &Value, b: &Value, path: &str) -> Option<String> {
    if a == b {
        return None;
    }
    if let (Some(x), Some(y)) = (a.as_object(), b.as_object()) {
        for k in x.keys().chain(y.keys()) {
            let p = if path.is_empty() {
                k.clone()
            } else {
                format!("{path}.{k}")
            };
            if let Some(diff) = first_diff(
                &x.get(k).cloned().unwrap_or(Value::Null),
                &y.get(k).cloned().unwrap_or(Value::Null),
                &p,
            ) {
                return Some(diff);
            }
            if x.contains_key(k) != y.contains_key(k) {
                return Some(p);
            }
        }
    }
    if let (Some(x), Some(y)) = (a.as_array(), b.as_array()) {
        for i in 0..x.len().max(y.len()) {
            let p = format!("{path}[{i}]");
            if let Some(diff) = first_diff(
                x.get(i).unwrap_or(&Value::Null),
                y.get(i).unwrap_or(&Value::Null),
                &p,
            ) {
                return Some(diff);
            }
        }
    }
    Some(path.to_owned())
}
fn patch(v: &Value, edited: &Value) -> R<Value> {
    let base = view(v)?;
    let mut next = v.clone();
    for f in FIELDS {
        let before = &base[f.category][f.section][f.key];
        let after = &edited[f.category][f.section][f.key];
        if before != after {
            let encoded = if f.key == "color" {
                hex_to_color(after).ok_or("Cor de mira inválida: use #RRGGBB ou #RRGGBBAA")?
            } else {
                after.clone()
            };
            if before.is_null() || !valid_value(f, &encoded) {
                return Err(format!(
                    "Valor inválido ou não suportado: {}.{}.{}",
                    f.category, f.section, f.key
                ));
            }
            let a = next[f.group].as_array_mut().ok_or("Configuração ausente")?;
            for x in a {
                if x.get("settingEnum").and_then(Value::as_str) == Some(f.game) {
                    x["value"] = encoded.clone();
                }
            }
        }
    }
    let mut expected = edited.clone();
    if base["crosshair"]["primary"]["color"] != edited["crosshair"]["primary"]["color"] {
        if let Some(c) =
            hex_to_color(&edited["crosshair"]["primary"]["color"]).and_then(|v| color_to_hex(&v))
        {
            expected["crosshair"]["primary"]["color"] = c;
        }
    }
    if let Some(path) = first_diff(&view(&next)?, &expected, "") {
        return Err(format!(
            "settings.json: campo sem mapeamento validado: {path}"
        ));
    }
    Ok(next)
}
const VIDEO_FIELDS: &[(&str, &str, &str)] = &[
    ("ResolutionSizeX", "display", "width"),
    ("ResolutionSizeY", "display", "height"),
    ("FrameRateLimit", "display", "fps_limit"),
    ("PreferredFullscreenMode", "display", "display_mode"),
    ("bUseVSync", "display", "vsync"),
    ("sg.ViewDistanceQuality", "graphics", "view_distance"),
    ("sg.AntiAliasingQuality", "graphics", "anti_aliasing"),
    ("sg.ShadowQuality", "graphics", "shadows"),
    (
        "sg.GlobalIlluminationQuality",
        "graphics",
        "global_illumination",
    ),
    ("sg.ReflectionQuality", "graphics", "reflections"),
    ("sg.PostProcessQuality", "graphics", "post_processing"),
    ("sg.TextureQuality", "graphics", "textures"),
    ("sg.EffectsQuality", "graphics", "effects"),
    ("sg.FoliageQuality", "graphics", "foliage"),
    ("sg.ShadingQuality", "graphics", "shading"),
];
fn video_path(files: &Files) -> Option<String> {
    let mut accounts = files.keys().filter(|p| {
        p.starts_with("raw/Config/")
            && p.ends_with("/WindowsClient/GameUserSettings.ini")
            && p.split('/').count() == 5
    });
    let first = accounts.next();
    if accounts.next().is_none() {
        if let Some(p) = first {
            return Some(p.clone());
        }
    }
    let p = "raw/Config/WindowsClient/GameUserSettings.ini";
    files.contains_key(p).then(|| p.to_string())
}
fn parse_video(key: &str, value: &str) -> Option<Value> {
    let v = value.trim();
    if key == "bUseVSync" {
        return match v.to_ascii_lowercase().as_str() {
            "true" => Some(json!(true)),
            "false" => Some(json!(false)),
            _ => None,
        };
    }
    if key == "PreferredFullscreenMode" {
        return match v {
            "0" => Some(json!("fullscreen")),
            "1" => Some(json!("windowed_fullscreen")),
            "2" => Some(json!("windowed")),
            _ => None,
        };
    }
    if key == "FrameRateLimit" {
        let n = v.parse::<f64>().ok()?;
        return if n.is_finite() {
            Some(if n == 0.0 {
                json!("unlimited")
            } else {
                json!(n)
            })
        } else {
            None
        };
    }
    v.parse::<u32>().ok().map(|n| json!(n))
}
fn video_view(files: &Files) -> Value {
    let mut v = json!({"display":{},"graphics":{}});
    if let Some(p) = video_path(files) {
        if let Ok(s) = std::str::from_utf8(&files[&p]) {
            let mut section = "";
            for line in s.lines() {
                if line.starts_with('[') {
                    section = line;
                    continue;
                }
                if let Some((k, x)) = line.split_once('=') {
                    for &(raw, group, key) in VIDEO_FIELDS {
                        if k == raw
                            && (section.is_empty()
                                || section == "[/Script/ShooterGame.ShooterGameUserSettings]"
                                || section == "[/Script/Engine.GameUserSettings]"
                                || section == "[ScalabilityGroups]")
                        {
                            if let Some(value) = parse_video(raw, x) {
                                v[group][key] = value;
                            }
                        }
                    }
                }
            }
        }
    }
    v
}
fn safe_video_value(raw: &str, value: &Value) -> R<String> {
    if raw == "bUseVSync" {
        return value
            .as_bool()
            .map(|b| if b { "True" } else { "False" }.into())
            .ok_or("VSync inválido".into());
    }
    if raw == "FrameRateLimit" {
        if value.as_str() == Some("unlimited") {
            return Ok("0.000000".into());
        }
        return value
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 30.0 && *n <= 1000.0)
            .map(|n| format!("{n:.6}"))
            .ok_or("Limite de FPS inválido".into());
    }
    if raw.starts_with("sg.") {
        return value
            .as_u64()
            .filter(|n| *n <= 4)
            .map(|n| n.to_string())
            .ok_or("Qualidade gráfica inválida".into());
    }
    Err("Resolução e modo de exibição são dependentes do computador e permanecem somente para leitura".into())
}
fn patch_video(files: &mut Files, desired: &Value) -> R<()> {
    let base = video_view(files);
    if base == *desired {
        return Ok(());
    }
    let path = video_path(files).ok_or("Configurações de vídeo não encontradas")?;
    let mut changes = BTreeMap::new();
    for &(raw, group, key) in VIDEO_FIELDS {
        if base[group][key] != desired[group][key] {
            if base[group][key].is_null() {
                return Err("Campo de vídeo ausente no original".into());
            }
            changes.insert(
                raw,
                safe_video_value(raw, &desired[group][key])
                    .map_err(|e| format!("settings.json: video.{group}.{key}: {e}"))?,
            );
        }
    }
    let s = std::str::from_utf8(&files[&path]).map_err(err)?;
    let mut next = String::new();
    let mut count = BTreeMap::new();
    for line in s.split_inclusive('\n') {
        let body = line.trim_end_matches(['\r', '\n']);
        if let Some((k, _)) = body.split_once('=') {
            if let Some(value) = changes.get(k) {
                next.push_str(k);
                next.push('=');
                next.push_str(value);
                next.push_str(&line[body.len()..]);
                *count.entry(k).or_insert(0) += 1;
                continue;
            }
        }
        next.push_str(line);
    }
    if changes.keys().any(|k| count.get(k) != Some(&1)) {
        return Err("Configuração de vídeo ambígua".into());
    }
    files.insert(path, next.into_bytes());
    if video_view(files) != *desired {
        return Err("settings.json contém configuração de vídeo não suportada".into());
    }
    Ok(())
}
fn patch_local_cloud_fields(files: &mut Files, original: &Value, next: &Value) -> R<()> {
    let mut paths: Vec<_> = files
        .keys()
        .filter(|p| {
            p.starts_with("raw/Config/")
                && p.ends_with("/Windows/RiotUserSettings.ini")
                && p.split('/').count() == 5
        })
        .cloned()
        .collect();
    if paths.len() > 1 {
        return Err("O perfil contém várias contas; não é possível editar a configuração local com segurança".into());
    }
    let Some(path) = paths.pop() else {
        return Ok(());
    };
    let mut edits = BTreeMap::new();
    for f in FIELDS {
        if locate(original, f) != locate(next, f) {
            if let Some(v) = locate(next, f) {
                let val = if let Some(b) = v.as_bool() {
                    if b {
                        "True".into()
                    } else {
                        "False".into()
                    }
                } else if let Some(s) = v.as_str() {
                    s.to_owned()
                } else {
                    v.to_string()
                };
                edits.insert(f.game, val);
            }
        }
    }
    let s = std::str::from_utf8(&files[&path]).map_err(err)?;
    let mut out = String::new();
    for line in s.split_inclusive('\n') {
        let body = line.trim_end_matches(['\r', '\n']);
        if let Some((key, _)) = body.split_once('=') {
            if let Some(v) = edits.get(key) {
                out.push_str(key);
                out.push('=');
                out.push_str(v);
                out.push_str(&line[body.len()..]);
                continue;
            }
        }
        out.push_str(line);
    }
    files.insert(path, out.into_bytes());
    Ok(())
}
fn shared_files(s: &crate::share::Share) -> R<Files> {
    crate::share::validate(s)?;
    let cloud = share_cloud(&s.settings)?;
    Ok(Files::from([
        ("raw/share.json".into(), pretty(s)?),
        ("raw/cloud.txt".into(), encode(&cloud)?),
    ]))
}
fn settings(files: &Files) -> R<Value> {
    let mut v = view(&decode(
        files.get("raw/cloud.txt").ok_or("Dados Riot ausentes")?,
    )?)?;
    v["video"] = video_view(files);
    if let Some(raw) = files.get("raw/share.json") {
        if raw.len() > 32768 {
            return Err("Dados compartilháveis excedem o limite".into());
        }
        let s: crate::share::Share = parse(raw)?;
        crate::share::validate(&s)?;
        for (key, value) in &s.video {
            let (group, field) = key.split_once('.').ok_or("Campo de vídeo inválido")?;
            v["video"][group][field] = value.clone();
        }
    }
    Ok(v)
}
fn avatar_data(files: &Files, m: &Manifest) -> R<Option<String>> {
    let Some(p) = &m.avatar else { return Ok(None) };
    let b = files.get(p).ok_or("Avatar ausente")?;
    if b.len() > 1024 * 1024 {
        return Err("Avatar excede o limite".into());
    }
    let mime = match p.as_str() {
        "assets/avatar.jpeg" if b.starts_with(&[0xff, 0xd8, 0xff]) => "jpeg",
        "assets/avatar.png" if b.starts_with(b"\x89PNG\r\n\x1a\n") => "png",
        "assets/avatar.webp" if b.starts_with(b"RIFF") && b.get(8..12) == Some(b"WEBP") => "webp",
        _ => return Err("Imagem de perfil inválida".into()),
    };
    Ok(Some(format!(
        "data:image/{mime};base64,{}",
        STANDARD.encode(b)
    )))
}
fn metadata(m: &Manifest) -> R<()> {
    if m.format != "ponkan" || m.format_version != 1 {
        return Err("Formato .pkn incompatível".into());
    }
    Uuid::parse_str(&m.id).map_err(|_| "Identificador inválido")?;
    crate::profiles::validate_name(&m.name)?;
    for (v, max) in [(&m.author, 80), (&m.description, 512)] {
        if v.as_ref().is_some_and(|s| {
            s.chars().count() > max || s.chars().any(|c| c.is_control() && c != '\n')
        }) {
            return Err("Metadados inválidos".into());
        }
    }
    time::OffsetDateTime::parse(
        &m.updated_at,
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| "Data inválida")?;
    if let Some(s) = &m.created_at {
        time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
            .map_err(|_| "Data inválida")?;
    }
    if m.app_version.len() > 64 {
        return Err("Versão inválida".into());
    }
    Ok(())
}
fn write_zip(files: &Files) -> R<Vec<u8>> {
    let mut w = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, b) in files {
        w.start_file(name, opts).map_err(err)?;
        w.write_all(b).map_err(err)?;
    }
    Ok(w.finish().map_err(err)?.into_inner())
}
fn read_zip(bytes: &[u8]) -> R<Files> {
    if bytes.len() as u64 > MAX_ARCHIVE {
        return Err("Arquivo .pkn muito grande".into());
    }
    let mut z = ZipArchive::new(Cursor::new(bytes)).map_err(|_| "Arquivo .pkn inválido")?;
    if z.len() > MAX_FILES {
        return Err("Quantidade excessiva de arquivos".into());
    }
    let mut out = Files::new();
    let mut names = BTreeSet::new();
    let mut total = 0u64;
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(err)?;
        let name = f.name().map_err(err)?.into_owned();
        if f.is_dir() {
            if !safe_path(name.trim_end_matches('/')) {
                return Err("Caminho inválido".into());
            }
            continue;
        }
        if !safe_path(&name)
            || !allowed(&name)
            || !names.insert(name.to_ascii_lowercase())
            || f.enclosed_name().is_none()
            || f.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000)
        {
            return Err(format!("Arquivo não permitido: {name}"));
        }
        if f.size() > MAX_FILE || total.saturating_add(f.size()) > MAX_TOTAL {
            return Err("Limite de descompressão excedido".into());
        }
        let mut b = Vec::new();
        (&mut f)
            .take(MAX_FILE + 1)
            .read_to_end(&mut b)
            .map_err(err)?;
        if b.len() as u64 != f.size() {
            return Err("Arquivo incompleto".into());
        }
        total += b.len() as u64;
        if total > MAX_TOTAL {
            return Err("Limite de descompressão excedido".into());
        }
        out.insert(name, b);
    }
    Ok(out)
}
fn with_integrity(files: &mut Files) -> R<()> {
    files.remove("integrity.json");
    let entries = files.iter().map(|(k, v)| (k.clone(), sha(v))).collect();
    files.insert(
        "integrity.json".into(),
        pretty(&Integrity {
            algorithm: "sha256".into(),
            files: entries,
        })?,
    );
    Ok(())
}
pub fn inspect(bytes: &[u8]) -> R<Package> {
    let mut files = read_zip(bytes)?;
    if files.contains_key("editor-draft.json") {
        return Err(
            "Este arquivo contém um rascunho não validado e não pode ser importado.".into(),
        );
    }
    let manifest: Manifest = parse(files.get("profile.json").ok_or("profile.json ausente")?)?;
    metadata(&manifest)?;
    avatar_data(&files, &manifest)?;
    if files
        .keys()
        .filter(|p| p.starts_with("assets/"))
        .any(|p| Some(p) != manifest.avatar.as_ref())
    {
        return Err("Avatar não referenciado".into());
    }
    let integrity: Integrity = parse(
        files
            .get("integrity.json")
            .ok_or("integrity.json ausente")?,
    )?;
    if integrity.algorithm != "sha256" || integrity.files.len() + 1 != files.len() {
        return Err("Manifesto de integridade inválido".into());
    }
    let mut edited = false;
    for (p, b) in &files {
        if p == "integrity.json" {
            continue;
        }
        let old = integrity
            .files
            .get(p)
            .ok_or("Arquivo não listado na integridade")?;
        if old.len() != 64 || !old.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("Hash inválido".into());
        }
        if !old.eq_ignore_ascii_case(&sha(b)) {
            if p == "settings.json" || p == "profile.json" {
                edited = true
            } else {
                return Err(format!("Arquivo modificado ou corrompido: {p}"));
            }
        }
    }
    if let Some(raw) = files.get("raw/share.json") {
        if raw.len() > 32768 || files.keys().any(|p| p.starts_with("raw/Config/")) {
            return Err("Arquivo de perfil incompatível".into());
        }
        let s: crate::share::Share = parse(raw)?;
        crate::share::validate(&s)?;
        if decode(files.get("raw/cloud.txt").ok_or("Dados Riot ausentes")?)?
            != share_cloud(&s.settings)?
        {
            return Err("Configurações do perfil inconsistentes".into());
        }
        let desired: Value = parse(files.get("settings.json").ok_or("settings.json ausente")?)?;
        if desired != settings(&files)? || edited {
            return Err("Arquivo de perfil modificado ou inconsistente".into());
        }
        return Ok(Package {
            manifest,
            files,
            edited: false,
        });
    }
    if !files.keys().any(|p| p.starts_with("raw/Config/")) {
        return Err("Configurações locais ausentes".into());
    }
    let original = settings(&files)?;
    let desired: Value = parse(files.get("settings.json").ok_or("settings.json ausente")?)?;
    if desired != original {
        patch_video(&mut files, &desired["video"])?;
        let cloud = decode(&files["raw/cloud.txt"])?;
        let mut cloud_view = desired.clone();
        cloud_view["video"] = json!({});
        let next = patch(&cloud, &cloud_view)?;
        if next != cloud {
            patch_local_cloud_fields(&mut files, &cloud, &next)?;
            files.insert("raw/cloud.txt".into(), encode(&next)?);
        }
        edited = true;
    }
    Ok(Package {
        manifest,
        files,
        edited,
    })
}
fn read_limited(p: &Path) -> R<Vec<u8>> {
    let f = fs::File::open(p).map_err(err)?;
    let mut b = Vec::new();
    f.take(MAX_ARCHIVE + 1).read_to_end(&mut b).map_err(err)?;
    if b.len() as u64 > MAX_ARCHIVE {
        return Err("Arquivo muito grande".into());
    }
    Ok(b)
}
pub fn export(
    id: &str,
    avatar: Option<String>,
    author: Option<String>,
    description: Option<String>,
) -> R<Vec<u8>> {
    let profile = crate::profiles::find(id)?;
    let mut files = if profile.shared == Some(true) {
        let mut s = crate::share::load(id)?;
        s.name = profile.name.clone();
        s.author = author.clone();
        s.description = description.clone();
        shared_files(&s)?
    } else {
        let root = crate::profiles::config_for(id)?;
        let mut files = Files::new();
        files.insert(
            "raw/cloud.txt".into(),
            crate::profiles::cloud_for(id)?.into_bytes(),
        );
        for e in walkdir::WalkDir::new(&root) {
            let e = e.map_err(err)?;
            if e.file_type().is_symlink() {
                return Err("Link simbólico nas configurações".into());
            }
            if !e.file_type().is_file() {
                continue;
            }
            let path = format!(
                "raw/Config/{}",
                e.path()
                    .strip_prefix(&root)
                    .map_err(err)?
                    .to_string_lossy()
                    .replace('\\', "/")
            );
            if !safe_path(&path) || !allowed(&path) {
                return Err("Arquivo de configuração não suportado".into());
            }
            if e.metadata().map_err(err)?.len() > MAX_FILE {
                return Err("Arquivo de configuração muito grande".into());
            }
            files.insert(path, fs::read(e.path()).map_err(err)?);
        }
        files
    };
    let avatar_path = if let Some(a) = avatar {
        let (h, b) = a.split_once(',').ok_or("Avatar inválido")?;
        let path = match h {
            "data:image/jpeg;base64" => "assets/avatar.jpeg",
            "data:image/webp;base64" => "assets/avatar.webp",
            "data:image/png;base64" => "assets/avatar.png",
            _ => return Err("Avatar inválido".into()),
        };
        if b.len() > 2 * 1024 * 1024 {
            return Err("Avatar muito grande".into());
        }
        files.insert(path.into(), STANDARD.decode(b).map_err(err)?);
        Some(path.into())
    } else {
        None
    };
    let old = stored_manifest(id)?;
    let manifest = Manifest {
        format: "ponkan".into(),
        format_version: 1,
        id: profile.id,
        name: profile.name,
        author,
        description,
        avatar: avatar_path,
        created_at: old.as_ref().and_then(|m| m.created_at.clone()),
        updated_at: profile.updated_at,
        app_version: env!("CARGO_PKG_VERSION").into(),
    };
    metadata(&manifest)?;
    avatar_data(&files, &manifest)?;
    files.insert("profile.json".into(), pretty(&manifest)?);
    let readable = settings(&files)?;
    files.insert("settings.json".into(), pretty(&readable)?);
    with_integrity(&mut files)?;
    let bytes = write_zip(&files)?;
    inspect(&bytes)?;
    Ok(bytes)
}
fn stored_manifest(id: &str) -> R<Option<Manifest>> {
    Uuid::parse_str(id).map_err(err)?;
    let p = crate::profiles::data_root()?
        .join("profiles")
        .join(id)
        .join("portable.json");
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(parse(&fs::read(p).map_err(err)?)?))
}
pub fn is_imported(id: &str) -> R<bool> {
    Ok(stored_manifest(id)?.is_some())
}
fn unique_name(name: &str, existing: &[crate::profiles::Profile]) -> String {
    if !existing.iter().any(|p| p.name.eq_ignore_ascii_case(name)) {
        return name.into();
    }
    for i in 2..10000 {
        let suffix = format!(" ({i})");
        let prefix: String = name.chars().take(48 - suffix.len()).collect();
        let n = format!("{prefix}{suffix}");
        if !existing.iter().any(|p| p.name.eq_ignore_ascii_case(&n)) {
            return n;
        }
    }
    Uuid::new_v4().to_string()
}
pub fn preview(bytes: &[u8]) -> R<Preview> {
    let p = inspect(bytes)?;
    let existing = crate::profiles::list()?;
    Ok(Preview {
        duplicate: existing
            .iter()
            .any(|x| x.id == p.manifest.id || x.name.eq_ignore_ascii_case(&p.manifest.name)),
        suggested_name: unique_name(&p.manifest.name, &existing),
        avatar: avatar_data(&p.files, &p.manifest)?,
        edited: p.edited,
        manifest: p.manifest,
    })
}
pub fn import(bytes: &[u8], name: String) -> R<crate::profiles::Profile> {
    let package = inspect(bytes)?;
    crate::profiles::import_portable(name, package.manifest, package.files)
}
// Safe portable cloud application: merge only proven fields into the destination.
// Unknown and account-specific destination values remain unchanged.
pub fn merge_cloud(source: &str, dest: &str) -> R<String> {
    let source = decode(source.as_bytes())?;
    let mut target = decode(dest.as_bytes())?;
    let mut applied = 0;
    for f in FIELDS {
        if let Some(v) = locate(&source, f) {
            if !valid_value(f, v) {
                continue;
            }
            if locate(&target, f).is_some() {
                for x in target[f.group].as_array_mut().unwrap() {
                    if x.get("settingEnum").and_then(Value::as_str) == Some(f.game) {
                        x["value"] = v.clone();
                    }
                }
            } else if let Some(a) = target.get_mut(f.group).and_then(Value::as_array_mut) {
                // The enum is proven; insert only into an existing typed settings list.
                if a.iter()
                    .any(|x| x.get("settingEnum").and_then(Value::as_str) == Some(f.game))
                {
                    continue;
                }
                a.push(json!({"settingEnum":f.game,"value":v}));
            } else {
                continue;
            }
            applied += 1;
        }
    }
    if applied == 0 {
        return Err("Nenhuma configuração portátil compatível encontrada".into());
    }
    String::from_utf8(encode(&target)?).map_err(err)
}
pub fn choose_import() -> R<Option<Vec<u8>>> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Ponkan Profile", &["pkn"])
        .pick_file()
    else {
        return Ok(None);
    };
    Ok(Some(read_limited(&path)?))
}
pub fn choose_export(name: &str, bytes: &[u8]) -> R<bool> {
    let Some(mut p) = rfd::FileDialog::new()
        .add_filter("Ponkan Profile", &["pkn"])
        .set_file_name(format!("{name}.pkn"))
        .save_file()
    else {
        return Ok(false);
    };
    if p.extension().and_then(|v| v.to_str()) != Some("pkn") {
        p.set_extension("pkn");
    }
    // Refuse overwrite after extension normalization; no silent truncation.
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&p)
        .map_err(|e| format!("Não foi possível criar o arquivo (use um nome novo): {e}"))?;
    if let Err(e) = f.write_all(bytes).and_then(|_| f.sync_all()) {
        drop(f);
        let _ = fs::remove_file(p);
        return Err(err(e));
    }
    Ok(true)
}



// Share links carry only existing, typed and validated portable settings; no raw account data.
fn share_key(f: &Field) -> String {
    format!("{}.{}.{}", f.category, f.section, f.key)
}
pub(crate) fn decode_share_cloud(s: &str) -> R<Value> {
    decode(s.as_bytes())
}
pub(crate) fn encode_share_cloud(v: &Value) -> R<String> {
    String::from_utf8(encode(v)?).map_err(err)
}
pub(crate) fn share_fields(v: &Value) -> R<BTreeMap<String, Value>> {
    let mut out = BTreeMap::new();
    for f in FIELDS {
        if let Some(x) = locate(v, f) {
            if valid_value(f, x) {
                out.insert(
                    share_key(f),
                    if f.key == "color" {
                        color_to_hex(x).ok_or("Cor inválida")?
                    } else {
                        x.clone()
                    },
                );
            }
        }
    }
    Ok(out)
}
pub(crate) fn share_cloud(fields: &BTreeMap<String, Value>) -> R<Value> {
    let mut out = json!({"floatSettings":[],"boolSettings":[],"intSettings":[],"stringSettings":[],"actionMappings":[]});
    for (key, v) in fields {
        let f = FIELDS
            .iter()
            .find(|f| share_key(f) == *key)
            .ok_or("Configuração do link não suportada.")?;
        let value = if f.key == "color" {
            hex_to_color(v).ok_or("Cor inválida")?
        } else {
            v.clone()
        };
        if !valid_value(f, &value) {
            return Err("Valor de configuração inválido.".into());
        }
        out[f.group]
            .as_array_mut()
            .unwrap()
            .push(json!({"settingEnum":f.game,"value":value}));
    }
    Ok(out)
}
pub(crate) fn validate_share_video(fields: &BTreeMap<String, Value>) -> R<()> {
    for (key, v) in fields {
        let &(raw, _, _) = VIDEO_FIELDS
            .iter()
            .find(|&&(_, group, k)| format!("{group}.{k}") == *key)
            .ok_or("Configuração de vídeo não suportada.")?;
        safe_video_value(raw, v)?;
    }
    Ok(())
}
pub(crate) fn share_video_fields(root: &Path) -> R<BTreeMap<String, Value>> {
    let mut files = Files::new();
    for e in walkdir::WalkDir::new(root).max_depth(3) {
        let e = e.map_err(err)?;
        if e.file_type().is_symlink() {
            return Err("Link simbólico nas configurações".into());
        }
        if !e.file_type().is_file() || e.file_name() != "GameUserSettings.ini" {
            continue;
        }
        let rel = e
            .path()
            .strip_prefix(root)
            .map_err(err)?
            .to_string_lossy()
            .replace('\\', "/");
        if !rel.ends_with("WindowsClient/GameUserSettings.ini") {
            continue;
        }
        if e.metadata().map_err(err)?.len() > MAX_FILE {
            return Err("Arquivo de configuração muito grande".into());
        }
        files.insert(
            format!("raw/Config/{rel}"),
            fs::read(e.path()).map_err(err)?,
        );
    }
    let view = video_view(&files);
    let mut out = BTreeMap::new();
    for &(raw, group, key) in VIDEO_FIELDS {
        let v = &view[group][key];
        if !v.is_null() && safe_video_value(raw, v).is_ok() {
            out.insert(format!("{group}.{key}"), v.clone());
        }
    }
    Ok(out)
}
pub(crate) fn patch_shared_local(
    root: &Path,
    account: &str,
    cloud: &Value,
    video: &BTreeMap<String, Value>,
) -> R<()> {
    let mut files = Files::new();
    for rel in [
        format!("{account}/WindowsClient/GameUserSettings.ini"),
        format!("{account}/Windows/RiotUserSettings.ini"),
        "WindowsClient/GameUserSettings.ini".into(),
    ] {
        let path = root.join(&rel);
        if !path.is_file() {
            continue;
        }
        if fs::metadata(&path).map_err(err)?.len() > MAX_FILE {
            return Err("Arquivo de configuração muito grande".into());
        }
        files.insert(format!("raw/Config/{rel}"), fs::read(path).map_err(err)?);
    }
    if !video.is_empty() {
        validate_share_video(video)?;
        let mut desired = video_view(&files);
        for (k, v) in video {
            let (group, key) = k.split_once('.').ok_or("Configuração inválida")?;
            if desired[group][key].is_null() {
                return Err("Configuração de vídeo ausente neste computador.".into());
            }
            desired[group][key] = v.clone();
        }
        patch_video(&mut files, &desired)?;
    }
    patch_local_cloud_fields(&mut files, &json!({}), cloud)?;
    for (p, b) in files {
        let rel = p
            .strip_prefix("raw/Config/")
            .ok_or("Configuração inválida")?;
        fs::write(root.join(rel), b).map_err(err)?;
    }
    Ok(())
}
