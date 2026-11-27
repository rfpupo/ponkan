//! Native code sharing. The is.gd destination is data, never navigated to.
use super::{decode, digest, import_at, make, profiles, Profile, MAX_CODE, R};
use reqwest::{redirect::Policy, Client, Url};
use serde::Deserialize;
use std::{
    fs,
    sync::Mutex,
    time::{Duration, Instant},
};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;
const CARRIER: &str = "https://github.com/";
const SYMBOLS: &[u8] = b"!@#$%*+-=";
const NETWORK: &str = "Não foi possível conectar ao is.gd. Tente novamente.";
const RATE: &str = "Limite do is.gd atingido. Aguarde um minuto.";
const BAD: &str = "Código de perfil inválido ou incompatível.";
const MISSING: &str = "Código não encontrado ou indisponível.";
static COOLDOWN: Mutex<Option<Instant>> = Mutex::new(None);
static CACHE: Mutex<Option<(String, String)>> = Mutex::new(None);
pub(super) fn name_part(name: &str) -> String {
    name.nfd()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(5)
        .collect()
}
fn valid_alias(a: &str) -> bool {
    (5..=30).contains(&a.len()) && a.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
pub(super) fn format_code(name: &str, alias: &str, symbol: char) -> R<String> {
    if !valid_alias(alias) || !symbol.is_ascii() || !SYMBOLS.contains(&(symbol as u8)) {
        return Err(BAD.into());
    }
    let n = name_part(name);
    Ok(if n.is_empty() {
        format!("pkn!{alias}{symbol}")
    } else {
        format!("pkn.{n}!{alias}{symbol}")
    })
}
pub(super) fn parse_code(input: &str) -> R<&str> {
    if input.len() > 43 || !input.is_ascii() {
        return Err(BAD.into());
    }
    let (head, tail) = input.split_once('!').ok_or(BAD)?;
    if head != "pkn" {
        let n = head.strip_prefix("pkn.").ok_or(BAD)?;
        if n.is_empty() || n.len() > 5 || !n.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            return Err(BAD.into());
        }
    }
    let (alias, last) = tail.split_at(tail.len().checked_sub(1).ok_or(BAD)?);
    if !valid_alias(alias) || !SYMBOLS.contains(&last.as_bytes()[0]) {
        return Err(BAD.into());
    }
    Ok(alias)
}
pub(super) fn destination(payload: &str) -> R<String> {
    decode(payload)?;
    let url = format!("{CARRIER}#{payload}");
    if url.len() > 5000 {
        return Err("Perfil grande demais para compartilhar por código.".into());
    }
    Ok(url)
}
pub(super) fn destination_payload(url: &str) -> R<String> {
    // A carrier is not an authentication signal. The fragment is validated data,
    // never navigated to. Accept old one/two-component GitHub carriers without
    // embedding the original maintainer account in distributed code.
    let (carrier, payload) = url.split_once('#').ok_or(BAD)?;
    let path = carrier.strip_prefix("https://github.com/").ok_or(BAD)?;
    if !path.is_empty() {
        let parts: Vec<_> = path.split('/').collect();
        if parts.len() > 2
            || parts.iter().any(|p| {
                p.is_empty()
                    || p.len() > 100
                    || *p == "."
                    || *p == ".."
                    || !p
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            })
        {
            return Err(BAD.into());
        }
    }
    decode(payload)?;
    Ok(payload.to_owned())
}
pub(super) fn local_payload(input: &str) -> R<Option<String>> {
    if input.starts_with("PKN1.") {
        decode(input)?;
        return Ok(Some(input.to_owned()));
    }
    if input.starts_with("ponkan://import/") || input.starts_with("ponkan-dev://import/") {
        let p = input.split_once("://import/").ok_or(BAD)?.1;
        decode(p)?;
        return Ok(Some(p.to_owned()));
    }
    if input.starts_with("https://") || input.starts_with("http://") {
        let u = Url::parse(input).map_err(|_| BAD)?;
        if u.host_str() == Some("is.gd")
            && u.scheme() == "https"
            && u.username().is_empty()
            && u.password().is_none()
            && u.port().is_none()
            && u.query().is_none()
            && u.fragment().is_none()
        {
            return Ok(None);
        }
        let p = u.fragment().ok_or(BAD)?;
        decode(p)?;
        return Ok(Some(p.to_owned()));
    }
    Ok(None)
}
fn extract_alias(input: &str) -> R<String> {
    if let Some(a) = input.strip_prefix("https://is.gd/") {
        if valid_alias(a) {
            return Ok(a.to_owned());
        }
        return Err(BAD.into());
    }
    if input.starts_with("pkn.") || input.starts_with("pkn!") {
        return Ok(parse_code(input)?.to_owned());
    }
    if valid_alias(input) {
        return Ok(input.to_owned());
    }
    Err(BAD.into())
}
fn check_cooldown() -> R<()> {
    if COOLDOWN
        .lock()
        .map_err(|_| NETWORK)?
        .is_some_and(|t| t.elapsed() < Duration::from_secs(60))
    {
        return Err(RATE.into());
    }
    Ok(())
}
fn rate_limit() -> String {
    if let Ok(mut t) = COOLDOWN.lock() {
        *t = Some(Instant::now());
    }
    RATE.into()
}
fn client() -> R<Client> {
    Client::builder()
        .https_only(true)
        .redirect(Policy::none())
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(8))
        .user_agent("ponkan-profile-sharing/1")
        .build()
        .map_err(|_| NETWORK.into())
}
#[derive(Deserialize)]
struct Api {
    shorturl: Option<String>,
    url: Option<String>,
    errorcode: Option<u32>,
}
async fn response(mut r: reqwest::Response) -> R<Api> {
    let status = r.status();
    let mut raw = Vec::new();
    while let Some(chunk) = r.chunk().await.map_err(|_| NETWORK)? {
        if raw.len() + chunk.len() > 16384 {
            return Err(NETWORK.into());
        }
        raw.extend_from_slice(&chunk);
    }
    let data: Api = serde_json::from_slice(&raw).map_err(|_| {
        if status.as_u16() == 429 || status.as_u16() == 502 {
            rate_limit()
        } else {
            NETWORK.into()
        }
    })?;
    match data.errorcode {
        Some(3) => Err(rate_limit()),
        Some(1 | 2) => Err(MISSING.into()),
        Some(_) => Err(NETWORK.into()),
        None if !status.is_success() => Err(NETWORK.into()),
        None => Ok(data),
    }
}
async fn lookup(c: &Client, alias: &str) -> R<String> {
    if !valid_alias(alias) {
        return Err(BAD.into());
    }
    check_cooldown()?;
    let r = c
        .get("https://is.gd/forward.php")
        .query(&[("format", "json"), ("shorturl", alias)])
        .send()
        .await
        .map_err(|_| NETWORK)?;
    let u = response(r).await?.url.ok_or(BAD)?;
    destination_payload(&u)
}
fn cache_file(payload: &str) -> R<std::path::PathBuf> {
    Ok(profiles::data_root()?
        .join("share-codes-v2")
        .join(format!("{}.txt", digest(payload.as_bytes()))))
}
fn cached(payload: &str, name: &str) -> Option<String> {
    let file = cache_file(payload).ok()?;
    let s = fs::read_to_string(file).ok()?;
    let a = parse_code(&s).ok()?;
    let symbol = s.chars().last()?;
    (format_code(name, a, symbol).ok()?.as_str() == s).then_some(s)
}
fn remember(payload: &str, code: &str) {
    if let Ok(file) = cache_file(payload) {
        if let Some(parent) = file.parent() {
            if fs::create_dir_all(parent).is_ok() {
                let tmp = parent.join(format!("{}.tmp", Uuid::new_v4()));
                if fs::write(&tmp, code).is_ok() {
                    let _ = fs::rename(&tmp, &file);
                }
                let _ = fs::remove_file(tmp);
            }
        }
    }
}
pub(crate) async fn generate(id: &str) -> R<String> {
    let payload = make(id)?;
    let profile = decode(&payload)?;
    if let Ok(g) = CACHE.lock() {
        if let Some((p, c)) = &*g {
            if p == &payload {
                return Ok(c.clone());
            }
        }
    }
    if let Some(code) = cached(&payload, &profile.name) {
        return Ok(code);
    }
    let url = destination(&payload)?;
    check_cooldown()?;
    let c = client()?;
    // POST keeps the profile out of the request URL and preserves its fragment.
    let mut form = Url::parse("https://is.gd/create.php").map_err(|_| NETWORK)?;
    form.query_pairs_mut()
        .append_pair("format", "json")
        .append_pair("url", &url);
    let body = form.query().ok_or(NETWORK)?.to_owned();
    form.set_query(None);
    let r = c
        .post(form)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|_| NETWORK)?;
    let short = response(r).await?.shorturl.ok_or(BAD)?;
    let alias = short.strip_prefix("https://is.gd/").ok_or(BAD)?;
    if !valid_alias(alias) {
        return Err(BAD.into());
    }
    // is.gd assigns a globally unique alias; check exact data before copying.
    if lookup(&c, alias).await? != payload {
        return Err(BAD.into());
    }
    let symbol = SYMBOLS[(Uuid::new_v4().as_bytes()[0] as usize) % SYMBOLS.len()] as char;
    let code = format_code(&profile.name, alias, symbol)?;
    remember(&payload, &code);
    if let Ok(mut g) = CACHE.lock() {
        *g = Some((payload, code.clone()));
    }
    Ok(code)
}
pub(crate) async fn import(input: &str) -> R<Profile> {
    let input = input.trim();
    if input.is_empty() || input.len() > MAX_CODE + 512 {
        return Err(BAD.into());
    }
    let p = match local_payload(input)? {
        Some(p) => p,
        None => {
            let alias = extract_alias(input)?;
            let c = client()?;
            lookup(&c, &alias).await?
        }
    };
    // Same validated, deduplicating local import as the legacy protocol.
    import_at(&profiles::data_root()?, &decode(&p)?)
}
