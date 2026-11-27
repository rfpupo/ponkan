// Adapted from ValoUtils v1.0.17 (MIT), windowsedd.
// https://github.com/windowsedd/ValoUtils/blob/v1.0.17/src-tauri/src/riot/settings.rs
// Local client credentials are used in memory only, never logged or saved.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};
use std::time::Duration;

const GET_URL: &str = "https://player-preferences-usw2.pp.sgp.pvp.net/playerPref/v3/getPreference/Ares.PlayerSettings";
const PUT_URL: &str = "https://player-preferences-usw2.pp.sgp.pvp.net/playerPref/v3/savePreference";

struct ClientCredentials {
    port: u16,
    password: String,
    protocol: String,
}

fn lockfile() -> Result<ClientCredentials, String> {
    let base = std::env::var("LOCALAPPDATA").map_err(|_| "AppData não encontrado")?;
    let path = std::path::Path::new(&base)
        .join("Riot Games")
        .join("Riot Client")
        .join("Config")
        .join("lockfile");
    let content = std::fs::read_to_string(path).map_err(|_| "Abra e entre no Riot Client")?;
    let parts: Vec<&str> = content.trim().split(':').collect();
    if parts.len() != 5 {
        return Err("Sessão Riot inválida".into());
    }
    let port = parts[2].parse::<u16>().map_err(|_| "Porta Riot inválida")?;
    if parts[3].is_empty() || !["http", "https"].contains(&parts[4]) {
        return Err("Sessão Riot inválida".into());
    }
    Ok(ClientCredentials {
        port,
        password: parts[3].to_owned(),
        protocol: parts[4].to_owned(),
    })
}

fn http_error(status: u16, session: bool) -> String {
    let message = match status {
        401 => "Não foi possível autenticar no Riot Client. Verifique sua conta e tente novamente.",
        403 => "O acesso ao serviço da Riot foi recusado. Verifique sua conta.",
        404 if session => {
            "Não foi possível obter a sessão da Riot. Verifique se você está conectado."
        }
        404 => "Não foi possível encontrar as configurações na nuvem Riot. Tente novamente.",
        429 => "Muitas solicitações à Riot. Aguarde um momento e tente novamente.",
        500..=599 => "O serviço da Riot está indisponível no momento. Tente novamente.",
        400..=499 => "O serviço da Riot não aceitou a solicitação. Tente novamente.",
        _ => "Não foi possível concluir a comunicação com a Riot. Tente novamente.",
    };
    format!("{message} (HTTP {status})")
}

pub struct Session {
    access: String,
    entitlement: String,
    pub(crate) subject: Option<String>,
}
pub async fn session() -> Result<Session, String> {
    let lc = lockfile()?;
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // Only for the local Riot Client HTTPS certificate.
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|_| "Falha ao conectar ao Riot Client")?;
    let basic = STANDARD.encode(format!("riot:{}", lc.password));
    let url = format!(
        "{}://127.0.0.1:{}/entitlements/v1/token",
        lc.protocol, lc.port
    );
    let response = client
        .get(url)
        .header("Authorization", format!("Basic {basic}"))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "A Riot demorou para responder. Tente novamente."
            } else {
                "Não foi possível conectar ao Riot Client. Abra o cliente e verifique sua conexão."
            }
        })?;
    if !response.status().is_success() {
        return Err(http_error(response.status().as_u16(), true));
    }
    let data: Value = response.json().await.map_err(|_| "Sessão Riot inválida")?;
    let access = data
        .get("accessToken")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or("Token de acesso não disponível")?;
    let entitlement = data
        .get("token")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or("Token Riot não disponível")?;
    Ok(Session {
        access: access.to_owned(),
        entitlement: entitlement.to_owned(),
        subject: data
            .get("subject")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn remote_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| "Falha na comunicação com a Riot".into())
}

pub async fn read_preferences() -> Result<String, String> {
    read_for(&session().await?).await
}
pub async fn read_for(session: &Session) -> Result<String, String> {
    let reply = remote_client()?
        .get(GET_URL)
        .bearer_auth(&session.access)
        .header("X-Riot-Entitlements-JWT", &session.entitlement)
        .send()
        .await
        .map_err(|_| "Não foi possível consultar a nuvem Riot")?;
    if !reply.status().is_success() {
        return Err(http_error(reply.status().as_u16(), false));
    }
    let json: Value = reply
        .json()
        .await
        .map_err(|_| "Resposta da Riot inválida")?;
    let data = json
        .get("data")
        .and_then(Value::as_str)
        .filter(|x| !x.is_empty())
        .ok_or("Nuvem Riot retornou configurações vazias")?;
    Ok(data.to_owned())
}

fn decoded_cloud(blob: &str) -> Option<Value> {
    use std::io::Read;
    let bytes = STANDARD.decode(blob).ok()?;
    let mut decoder = flate2::read::DeflateDecoder::new(bytes.as_slice());
    let mut body = Vec::new();
    decoder
        .by_ref()
        .take(16 * 1024 * 1024)
        .read_to_end(&mut body)
        .ok()?;
    serde_json::from_slice(&body).ok()
}

pub(crate) fn cloud_equals(left: &str, right: &str) -> bool {
    left == right
        || decoded_cloud(left)
            .zip(decoded_cloud(right))
            .is_some_and(|(a, b)| a == b)
}

fn same_subject(old: Option<&str>, current: Option<&str>) -> bool {
    old.is_some_and(|subject| !subject.is_empty() && Some(subject) == current)
}
pub async fn assert_current_session(old: &Session) -> Result<(), String> {
    let current = session().await?;
    if !same_subject(old.subject.as_deref(), current.subject.as_deref()) {
        return Err("A conta Riot mudou ou não pôde ser confirmada".into());
    }
    Ok(())
}
// Never overwrite an unrelated change or an unconfirmed account while rolling back.
fn rollback_decision(before: &str, intended: &str, current: &str) -> Result<bool, String> {
    if cloud_equals(before, current) {
        return Ok(false);
    }
    if cloud_equals(intended, current) {
        return Ok(true);
    }
    Err("A nuvem mudou de forma inesperada. Recuperação automática interrompida.".into())
}

pub(crate) async fn rollback_for(
    session: &Session,
    before: &str,
    intended: &str,
) -> Result<(), String> {
    crate::profiles::require_game_closed()?;
    assert_current_session(session).await?;
    let current = read_for(session).await?;
    if rollback_decision(before, intended, &current)? {
        assert_current_session(session).await?;
        crate::profiles::require_game_closed()?;
        write_for(session, before).await?;
    }
    Ok(())
}

pub async fn write_for(session: &Session, data: &str) -> Result<(), String> {
    if data.is_empty() {
        return Err("Backup da nuvem vazio".into());
    }
    assert_current_session(session).await?;
    crate::profiles::require_game_closed()?;
    let reply = remote_client()?
        .put(PUT_URL)
        .bearer_auth(&session.access)
        .header("X-Riot-Entitlements-JWT", &session.entitlement)
        .json(&json!({ "type": "Ares.PlayerSettings", "data": data }))
        .send()
        .await
        .map_err(|_| "Não foi possível salvar na nuvem Riot")?;
    if !reply.status().is_success() {
        return Err(http_error(reply.status().as_u16(), false));
    }
    // Compare against the server, rather than trusting a successful HTTP response.
    let persisted = read_for(session).await?;
    if !cloud_equals(&persisted, data) {
        return Err("A nuvem não confirmou o perfil restaurado".into());
    }
    Ok(())
}


