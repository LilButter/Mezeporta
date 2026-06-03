use core::fmt;
use log::warn;
use reqwest::{RequestBuilder, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{endpoint::Endpoint, patcher};

const NETWORK_ERROR: &str = "launcher-network-error";
const USERNAME_ERROR: &str = "username-error";
const PASSWORD_ERROR: &str = "password-error";
const USERNAME_EXISTS_ERROR: &str = "username-exists-error";

pub enum Error {
    Cancellation,
    Server(u16, String),
    Backend(String),
}

impl Error {
    pub fn into_frontend(self) -> String {
        match self {
            Self::Cancellation => "".into(),
            Self::Server(_, msg) | Self::Backend(msg) => msg,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancellation => write!(f, "request cancelled"),
            Self::Server(status, msg) => write!(f, "server error {}: {}", status, msg),
            Self::Backend(msg) => write!(f, "backend error: {}", msg),
        }
    }
}

fn null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let option = Option::<T>::deserialize(deserializer)?;
    Ok(option.unwrap_or_default())
}

fn bool_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BannerData {
    pub src: String,
    pub link: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageData {
    pub message: String,
    pub date: i32,
    pub link: String,
    pub kind: MessageKind,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum MessageKind {
    Default,
    New,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkData {
    pub name: String,
    pub link: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherPs4Assets {
    pub background: Option<String>,
    pub button: Option<String>,
    pub add_server_button: Option<String>,
    pub capcom: Option<String>,
    pub cog: Option<String>,
    pub emblem: Option<String>,
    pub headers: Option<LauncherPs4Headers>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherPs4Headers {
    pub online: Option<String>,
    pub forward: Option<String>,
    pub g: Option<String>,
    pub z: Option<String>,
    pub zz: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherHeaders {
    pub online: Option<String>,
    pub forward: Option<String>,
    pub g: Option<String>,
    pub z: Option<String>,
    pub zz: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherResponse {
    pub banners: Vec<BannerData>,
    pub messages: Vec<MessageData>,
    pub links: Vec<LinkData>,
    #[serde(alias = "launcher_tag", alias = "LauncherTag")]
    pub launcher_tag: Option<String>,
    #[serde(alias = "footer_tag", alias = "FooterTag")]
    pub footer_tag: Option<String>,
    #[serde(alias = "Tag")]
    pub tag: Option<String>,
    #[serde(alias = "server_tag", alias = "ServerTag")]
    pub server_tag: Option<String>,
    pub background: Option<String>,
    pub cog: Option<String>,
    pub capcom: Option<String>,
    pub button: Option<String>,
    pub classic_add_server_button: Option<String>,
    pub headers: Option<LauncherHeaders>,
    pub dialog: Option<String>,
    #[serde(rename = "server_patch")]
    pub server_patch: Option<String>,
    pub ps4: Option<LauncherPs4Assets>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub token_id: u32,
    pub token: String,
    pub rights: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterData {
    pub id: u32,
    pub name: String,
    pub is_female: bool,
    pub weapon: u32,
    pub hr: u32,
    pub gr: u32,
    pub last_login: u32,
    #[serde(default)]
    pub returning: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MezFesData {
    pub id: u32,
    pub start: u32,
    pub end: u32,
    pub solo_tickets: u32,
    pub group_tickets: u32,
    pub stalls: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FriendData {
    pub cid: u32,
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CourseData {
    pub id: u16,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub current_ts: u32,
    pub expiry_ts: u32,
    pub entrance_count: u32,
    pub notices: Vec<String>,
    pub user: UserData,
    pub characters: Vec<CharacterData>,
    #[serde(rename = "mezFes")]
    pub mez_fez: Option<MezFesData>,
    #[serde(default, deserialize_with = "null_default")]
    pub friends: Vec<FriendData>,
    #[serde(default)]
    pub courses: Vec<CourseData>,
    pub patch_server: String,
    #[serde(default = "bool_true")]
    pub alt_savedata_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmptyResponse {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatcherResponse {
    pub etag: String,
    pub content: String,
    pub server_name: String,
    pub queue_position: usize,
}

#[derive(Debug, Deserialize)]
struct APIErrorResponse {
    #[serde(default)]
    error: String,
    #[serde(default)]
    message: String,
}

fn map_api_error_code_to_frontend(code: &str) -> Option<&'static str> {
    match code {
        "invalid_username" => Some(USERNAME_ERROR),
        "invalid_password" => Some(PASSWORD_ERROR),
        "username_exists" => Some(USERNAME_EXISTS_ERROR),
        _ => None,
    }
}

fn extract_api_error_message(body: &str) -> Option<String> {
    let parsed: APIErrorResponse = serde_json::from_str(body).ok()?;
    if let Some(mapped) = map_api_error_code_to_frontend(parsed.error.trim()) {
        return Some(mapped.to_string());
    }
    if !parsed.message.trim().is_empty() {
        return Some(parsed.message);
    }
    if !parsed.error.trim().is_empty() {
        return Some(parsed.error);
    }
    None
}

async fn send(request: RequestBuilder, cancel: CancellationToken) -> Result<Response, Error> {
    let resp = select! {
        _ = cancel.cancelled() => return Err(Error::Cancellation),
        resp = request.send() => resp,
    };
    let resp = resp.map_err(|e| {
        warn!("request connection failed: {}", e);
        Error::Backend(NETWORK_ERROR.into())
    })?;
    let status = resp.status().as_u16();
    if status >= 400 {
        warn!("request status error: {}", status);
        let content_type = resp
            .headers()
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_ascii_lowercase())
            .unwrap_or_default();
        let body = resp.text().await.unwrap_or_default();
        let message = if content_type.starts_with("text/plain") {
            if body.trim().is_empty() {
                NETWORK_ERROR.into()
            } else {
                body
            }
        } else if content_type.contains("json") {
            extract_api_error_message(&body).unwrap_or_else(|| NETWORK_ERROR.into())
        } else if let Some(mapped) = extract_api_error_message(&body) {
            mapped
        } else {
            NETWORK_ERROR.into()
        };
        return Err(Error::Server(status, message));
    }
    Ok(resp)
}

pub struct JsonRequest<T: DeserializeOwned> {
    request: RequestBuilder,
    cancel: CancellationToken,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> JsonRequest<T> {
    pub(crate) fn new(request: RequestBuilder, cancel: CancellationToken) -> Self {
        Self {
            request,
            cancel,
            _phantom: PhantomData,
        }
    }

    pub async fn send(self) -> Result<T, Error> {
        let resp = send(self.request, self.cancel).await?;
        let text = resp.text().await.map_err(|e| {
            warn!("failed to read body: {}", e);
            Error::Backend(NETWORK_ERROR.into())
        })?;
        serde_json::from_str(&text).map_err(|e| {
            warn!("parsing failed: {}", e);
            Error::Backend(NETWORK_ERROR.into())
        })
    }
}

pub struct PatcherRequest {
    request: RequestBuilder,
    cancel: CancellationToken,
}

impl PatcherRequest {
    pub async fn send(self) -> Result<Option<PatcherResponse>, Error> {
        let resp = send(self.request, self.cancel).await?;
        let status = resp.status().as_u16();

        if status == 304 {
            return Ok(None);
        }

        let etag_header = resp
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);
        let queue_position = resp
            .headers()
            .get("X-AltClient-Patch-Queue-Position")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let server_name = {
            let url = resp.url();
            let host = url.host_str().unwrap_or_default();
            if host.is_empty() {
                String::new()
            } else if let Some(port) = url.port_or_known_default() {
                format!("{}:{}", host, port)
            } else {
                host.to_string()
            }
        };

        let content = resp.text().await.map_err(|e| {
            warn!("failed to read body of patcher request {}", e);
            Error::Server(status, patcher::NETWORK_ERROR.into())
        })?;

        let etag = etag_header.unwrap_or_else(|| {
            warn!("patcher response missing ETag, deriving fallback hash");
            fallback_patcher_etag(&content)
        });

        Ok(Some(PatcherResponse {
            etag,
            content,
            server_name,
            queue_position,
        }))
    }
}

fn fallback_patcher_etag(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("\"{:x}\"", hasher.finish())
}

fn normalize_patcher_base_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.to_ascii_lowercase().ends_with("/check") {
        return trimmed[..trimmed.len() - "/check".len()]
            .trim_end_matches('/')
            .to_string();
    }

    trimmed.to_string()
}
pub fn simple_request<T: DeserializeOwned>(
    client: &reqwest::Client,
    cancel: CancellationToken,
    url: &str,
) -> JsonRequest<T> {
    let req = client.get(url);
    JsonRequest::new(req, cancel)
}

pub fn launcher_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
) -> JsonRequest<LauncherResponse> {
    let req = client.get(endpoint.get_url("/v2/launcher"));
    JsonRequest::new(req, cancel)
}

pub fn login_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    username: &str,
    password: &str,
) -> JsonRequest<AuthResponse> {
    let auth_req = AuthRequest { username, password };
    let req = client.post(endpoint.get_url("/v2/login")).json(&auth_req);
    JsonRequest::new(req, cancel)
}

pub fn register_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    username: &str,
    password: &str,
) -> JsonRequest<AuthResponse> {
    let auth_req = AuthRequest { username, password };
    let req = client
        .post(endpoint.get_url("/v2/register"))
        .json(&auth_req);
    JsonRequest::new(req, cancel)
}

pub fn delete_character_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
    character_id: i32,
) -> JsonRequest<EmptyResponse> {
    let req = client
        .delete(endpoint.get_url(&format!("/v2/characters/{}", character_id)))
        .bearer_auth(token);
    JsonRequest::new(req, cancel)
}

pub fn create_character_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
) -> JsonRequest<CharacterData> {
    let req = client
        .post(endpoint.get_url("/v2/characters"))
        .bearer_auth(token);
    JsonRequest::new(req, cancel)
}

pub fn export_save_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
    character_id: i32,
) -> JsonRequest<Value> {
    let req = client
        .get(endpoint.get_url(&format!("/v2/characters/{}/export", character_id)))
        .bearer_auth(token);
    JsonRequest::new(req, cancel)
}

pub fn patcher_request(
    client: &reqwest::Client,
    cancel: CancellationToken,
    url: &str,
    client_etag: &str,
) -> PatcherRequest {
    let normalized = normalize_patcher_base_url(url);
    let check_url = if normalized.is_empty() {
        format!("{}/check", url.trim().trim_end_matches('/'))
    } else {
        format!("{}/check", normalized)
    };

    let mut request = client.get(check_url);

    if !client_etag.is_empty() {
        request = request.header("If-None-Match", client_etag);
    }

    PatcherRequest { request, cancel }
}
