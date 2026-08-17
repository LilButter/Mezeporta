use std::path::{Path, PathBuf};

use log::{info, warn};
use meze_butter as mhf_iel;
use meze_butter::MhfConfig;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::Value;

use crate::endpoint::Endpoint;
use crate::run_mhf;
use crate::settings;

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub username: String,
    pub password: String,
    pub server_host: String,
    pub launcher_port: u16,
    pub game_port: u16,
    pub version: String,
    pub character_slot: usize,
    pub hd: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CliAuthResponse {
    current_ts: u32,
    expiry_ts: u32,
    entrance_count: u32,
    notices: Vec<String>,
    user: CliUserData,
    characters: Vec<CliCharacterData>,
    #[serde(rename = "mezFes")]
    mez_fez: Option<CliMezFesData>,
    #[serde(default)]
    friends: Vec<CliFriendData>,
    patch_server: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CliUserData {
    token_id: u32,
    token: String,
    rights: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CliCharacterData {
    id: u32,
    name: String,
    is_female: bool,
    weapon: u32,
    hr: u32,
    gr: u32,
    last_login: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CliMezFesData {
    id: u32,
    start: u32,
    end: u32,
    solo_tickets: u32,
    group_tickets: u32,
    stalls: Vec<u32>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CliFriendData {
    cid: u32,
    id: u32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CliApiError {
    #[serde(default)]
    error: String,
    #[serde(default)]
    message: String,
}

fn print_usage() {
    println!("Mezeporta CLI - Meze-butter backend launcher");
    println!();
    println!("Usage: Mezeporta [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -v, --version <VERSION>   Game version (e.g., ZZ, G10.1, S7K, G1, F5, S6)");
    println!("  -u, --username <USER>     Login username");
    println!("  -pw, --password <PASS>    Login password");
    println!("  -s, --server <SERVER>     Server hostname or IP");
    println!("  -p1, --launcher-port <PORT>  Launcher port (default: 8080)");
    println!("  -p2, --game-port <PORT>      Game/entrance port (default: 53310)");
    println!("  -c, --character <SLOT>    Character slot index (0-based)");
    println!("  -HD                       Enable HD mode (default is SD)");
    println!("  --help                    Show this help message");
    println!();
    println!("Examples:");
    println!("  Mezeporta -v ZZ -u player -pw secret -s 192.168.1.100 -c 0");
    println!("  Mezeporta -v G1 -u player -pw secret -s mezeporta.example.com -p1 8080 -p2 53310 -c 1 -HD");
}

fn parse_args() -> Result<CliArgs, String> {
    let args: Vec<String> = std::env::args().collect();

    let mut username: Option<String> = None;
    let mut password: Option<String> = None;
    let mut server: Option<String> = None;
    let mut launcher_port: Option<u16> = None;
    let mut game_port: Option<u16> = None;
    let mut version: Option<String> = None;
    let mut character_slot: Option<usize> = None;
    let mut hd = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {

            "-v" | "--version" => {
                i += 1;
                if i >= args.len() {
                    return Err("--version requires a value".to_string());
                }
                version = Some(args[i].clone());
            }
            "-u" | "--username" => {
                i += 1;
                if i >= args.len() {
                    return Err("--username requires a value".to_string());
                }
                username = Some(args[i].clone());
            }
            "-pw" | "--password" => {
                i += 1;
                if i >= args.len() {
                    return Err("--password requires a value".to_string());
                }
                password = Some(args[i].clone());
            }
            "-s" | "--server" => {
                i += 1;
                if i >= args.len() {
                    return Err("--server requires a value".to_string());
                }
                server = Some(args[i].clone());
            }
            "-p1" | "--launcher-port" => {
                i += 1;
                if i >= args.len() {
                    return Err("--launcher-port requires a value".to_string());
                }
                launcher_port = Some(
                    args[i]
                        .parse::<u16>()
                        .map_err(|_| format!("invalid launcher port: {}", args[i]))?,
                );
            }
            "-p2" | "--game-port" => {
                i += 1;
                if i >= args.len() {
                    return Err("--game-port requires a value".to_string());
                }
                game_port = Some(
                    args[i]
                        .parse::<u16>()
                        .map_err(|_| format!("invalid game port: {}", args[i]))?,
                );
            }
            "-c" | "--character" => {
                i += 1;
                if i >= args.len() {
                    return Err("--character requires a value".to_string());
                }
                character_slot = Some(
                    args[i]
                        .parse::<usize>()
                        .map_err(|_| format!("invalid character slot: {}", args[i]))?,
                );
            }
            "-HD" | "--hd" => {
                hd = true;
            }
            other => {
                return Err(format!("unknown option: {}", other));
            }
        }
        i += 1;
    }

    let username = username.ok_or("-u/--username is required")?;
    let password = password.ok_or("-pw/--password is required")?;
    let server_host = server.ok_or("-s/--server is required")?;
    let version = version.ok_or("-v/--version is required")?;
    let character_slot = character_slot.ok_or("-c/--character is required")?;

    Ok(CliArgs {
        username,
        password,
        server_host,
        launcher_port: launcher_port.unwrap_or(8080),
        game_port: game_port.unwrap_or(53310),
        version,
        character_slot,
        hd,
    })
}

fn resolve_game_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."))
    })
}

async fn cli_auth(
    client: &Client,
    endpoint: &Endpoint,
    username: &str,
    password: &str,
) -> Result<CliAuthResponse, String> {
    let auth_req = serde_json::json!({
        "username": username,
        "password": password
    });

    let url = endpoint.get_url("/v2/login");

    let resp = client
        .post(&url)
        .json(&auth_req)
        .send()
        .await
        .map_err(|e| format!("failed to connect to server: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("failed to read response: {}", e))?;

    if status == StatusCode::UNAUTHORIZED {
        return Err("username-password-empty-error".to_string());
    }

    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<CliApiError>(&body) {
            if !err.error.is_empty() {
                return Err(err.error);
            }
            if !err.message.is_empty() {
                return Err(err.message);
            }
        }
        return Err(format!("server error: {} {}", status, body));
    }

    let auth_resp: CliAuthResponse =
        serde_json::from_str(&body).map_err(|e| format!("failed to parse auth response: {}", e))?;

    Ok(auth_resp)
}

fn build_cli_launch_config(
    auth_resp: &CliAuthResponse,
    args: &CliArgs,
    game_root: &PathBuf,
) -> Result<MhfConfig, String> {
    let version = crate::label_to_version(&args.version)
        .ok_or_else(|| format!("unknown version: {}", args.version))?;

    let char = auth_resp
        .characters
        .get(args.character_slot)
        .ok_or_else(|| {
            format!(
                "character slot {} out of range (has {} characters)",
                args.character_slot,
                auth_resp.characters.len()
            )
        })?
        .clone();

    let char_ids: Vec<u32> = auth_resp.characters.iter().map(|c| c.id).collect();

    let notices: Vec<mhf_iel::Notice> = auth_resp
        .notices
        .iter()
        .map(|n| mhf_iel::Notice {
            flags: 0,
            data: n.clone(),
        })
        .collect();

    let friends: Vec<mhf_iel::FriendData> = auth_resp
        .friends
        .iter()
        .map(|f| mhf_iel::FriendData {
            cid: f.cid,
            id: f.id,
            name: f.name.clone(),
        })
        .collect();

    let mut mez_stalls: Vec<mhf_iel::MezFesStall> = vec![];
    if let Some(mez_fes) = auth_resp.mez_fez.as_ref() {
        mez_stalls = mez_fes
            .stalls
            .iter()
            .filter_map(|&s| match mhf_iel::MezFesStall::try_from(s) {
                Ok(stall) => Some(stall),
                Err(e) => {
                    warn!("invalid mez stall value {}: {:?}", s, e);
                    None
                }
            })
            .collect();
    }

    let font_path = resolve_launcher_font_path(game_root, version);

    Ok(MhfConfig {
        char_id: char.id,
        char_name: char.name.clone(),
        char_gr: char.gr,
        char_hr: char.hr,
        char_ids,
        char_new: false,
        user_token_id: auth_resp.user.token_id,
        user_token: auth_resp.user.token.clone(),
        user_name: args.username.clone(),
        user_password: args.password.clone(),
        user_rights: auth_resp.user.rights,
        server_host: args.server_host.clone(),
        server_port: args.game_port as u32,
        entrance_count: auth_resp.entrance_count,
        current_ts: auth_resp.current_ts,
        expiry_ts: auth_resp.expiry_ts,
        notices,
        friends,
        mez_event_id: 0,
        mez_start: 0,
        mez_end: 0,
        mez_solo_tickets: 0,
        mez_group_tickets: 0,
        mez_stalls,
        version,
        mutex_version: version,
        mutex_fallback_version: None,
        preload_controller_dlls: false,
        friend_signature: Some("none".to_string()),
        enable_font_registration: true,
        mhf_folder: None,
        font_path,
        mhf_flags: None,
    })
}

fn resolve_launcher_font_path(game_root: &Path, version: mhf_iel::MhfVersion) -> Option<PathBuf> {
    let font_name = match version {
        mhf_iel::MhfVersion::Z2T => "dft_0.ttc",
        _ => "MS Gothic.ttf",
    };

    let primary = game_root.join("Mezeporta/fonts").join(font_name);
    if primary.exists() {
        return Some(primary);
    }

    let fallback = game_root.join("fonts").join(font_name);
    if fallback.exists() {
        return Some(fallback);
    }

    None
}

pub fn try_cli_launch() -> bool {
    let args = std::env::args().collect::<Vec<String>>();

    if args.iter().any(|a| a.as_str() == "--help" || a.as_str() == "-h") {
        print_usage();
        return true;
    }

    let has_cli_flags = args.iter().any(|a| {
        matches!(
            a.as_str(),
            "-v" | "--version" | "-u" | "--username" | "-pw" | "--password"
                | "-s" | "--server" | "-p1" | "--launcher-port" | "-p2" | "--game-port"
                | "-c" | "--character" | "-HD" | "--hd"
        )
    });

    if !has_cli_flags {
        return false;
    }

    let cli_args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            print_usage();
            return true;
        }
    };

    info!(
        "CLI launch: user={}, host={}, port1={}, port2={}, version={}, char={}, hd={}",
        cli_args.username,
        cli_args.server_host,
        cli_args.launcher_port,
        cli_args.game_port,
        cli_args.version,
        cli_args.character_slot,
        cli_args.hd,
    );

    // Build endpoint
    let endpoint_url = if cli_args.launcher_port == 8080 {
        cli_args.server_host.clone()
    } else {
        format!(
            "{}:{}",
            cli_args.server_host, cli_args.launcher_port
        )
    };

    let endpoint = Endpoint {
        url: endpoint_url,
        name: cli_args.server_host.clone(),
        launcher_port: Some(cli_args.launcher_port),
        game_port: Some(cli_args.game_port),
        game_folder: None,
        version: crate::label_to_version(&cli_args.version)
            .unwrap_or(mhf_iel::MhfVersion::ZZ),
        is_remote: true,
    };

    // Resolve game root
    let game_root = resolve_game_root();
    info!("Game root: {}", game_root.display());

    if cli_args.hd {
        if let Err(e) = settings::set_setting(&game_root, "hdVersion", Value::Bool(true)) {
            warn!("Failed to set HD version: {}", e);
        } else {
            info!("HD mode enabled");
        }
    }

    // Ensure support dirs exist
    if let Err(e) = crate::ensure_mezeporta_support_dirs(&game_root) {
        warn!("Failed to create support dirs: {}", e);
    }

    // Perform auth
    let client = Client::builder()
        .gzip(true)
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new());

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Failed to create tokio runtime: {}", e);
            return true;
        }
    };

    let auth_resp = match rt.block_on(cli_auth(
        &client,
        &endpoint,
        &cli_args.username,
        &cli_args.password,
    )) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Auth error: {}", e);
            return true;
        }
    };

    info!(
        "Auth successful: user={}, chars={}, entrance_count={}",
        cli_args.username,
        auth_resp.characters.len(),
        auth_resp.entrance_count,
    );

    // Build launch config
    let config = match build_cli_launch_config(&auth_resp, &cli_args, &game_root) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to build launch config: {}", e);
            return true;
        }
    };

    info!(
        "Launching game: char_id={}, char_name={}, version={}",
        config.char_id, config.char_name, config.version as u8
    );

    match run_mhf(
        config,
        game_root.clone(),
        Default::default(),
        true,
        None,
    ) {
        Ok(code) => {
            info!("Game exited with code: {}", code);
            true
        }
        Err(e) => {
            eprintln!("Failed to launch game: {}", e);
            true
        }
    }
}
