use std::path::{Path, PathBuf};

use log::{info, warn};
use meze_butter as mhf_iel;
use meze_butter::MhfConfig;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::Value;

use crate::endpoint::Endpoint;
use crate::run_mhf;
use crate::SERVER_MODE_SIGNV1;
use crate::settings;
use crate::sign_server;

/// Reads a line from stdin.
/// Works correctly when the executable is console-subsystem and launched from cmd,
/// since cmd waits for the process and stdin is properly inherited.
fn read_console_line() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}

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
    pub sign_server: bool,
    pub friend_signature: Option<String>,
    pub list_versions: bool,
    pub list_signatures: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CliAuthResponse {
    pub current_ts: u32,
    pub expiry_ts: u32,
    pub entrance_count: u32,
    pub notices: Vec<String>,
    pub user: CliUserData,
    pub characters: Vec<CliCharacterData>,
    #[serde(rename = "mezFes")]
    pub mez_fez: Option<CliMezFesData>,
    #[serde(default)]
    pub friends: Vec<CliFriendData>,
    pub patch_server: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CliUserData {
    pub token_id: u32,
    pub token: String,
    pub rights: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CliCharacterData {
    pub id: u32,
    pub name: String,
    pub is_female: bool,
    pub weapon: u32,
    pub hr: u32,
    pub gr: u32,
    pub last_login: u32,
    #[serde(default)]
    pub is_new: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CliMezFesData {
    pub id: u32,
    pub start: u32,
    pub end: u32,
    pub solo_tickets: u32,
    pub group_tickets: u32,
    pub stalls: Vec<u32>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CliFriendData {
    pub cid: u32,
    pub id: u32,
    pub name: String,
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
    println!("  -p1, --launcher-port <PORT>  Launcher port (default: 8080 for API, 53312 for sign server)");
    println!("  -p2, --game-port <PORT>      Game/entrance port (default: 53310)");
    println!("  -t, --sign-server           Use Signv1 (sign server) auth instead of API");
    println!("  -c, --character <SLOT>    Character slot index (0-based)");
    println!("  -HD                       Enable HD mode (default is SD)");
    println!("  -fs, --friend-signature <SIG>  Version signature (e.g., v1.52.79_04d16dc4)");
    println!("  --list-versions, --list-version  List all available game versions");
    println!("  --list-signatures         List all version signatures for a version (requires -v)");
    println!("  --help                    Show this help message");
    println!();
    println!("Examples:");
    println!("  Mezeporta -v ZZ -u player -pw secret -s 192.168.1.100 -c 0");
    println!("  Mezeporta -v S7K -u player -pw secret -s mezeporta.example.com -p1 8080 -p2 53310 -c 1 -HD");
    println!("  Mezeporta -v ZZ -u player -pw secret -s 192.168.1.100 -t -c 0");
    println!("  Mezeporta -v ZZ -u player -pw secret -s 192.168.1.100 -t -p1 53312 -c 0");
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
    let mut sign_server = false;
    let mut friend_signature: Option<String> = None;
    let mut list_versions = false;
    let mut list_signatures = false;

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
            "-t" | "--sign-server" => {
                sign_server = true;
            }
            "-fs" | "--friend-signature" => {
                i += 1;
                if i >= args.len() {
                    return Err("--friend-signature requires a value".to_string());
                }
                friend_signature = Some(args[i].clone());
            }
            "--list-versions" | "--list-version" => {
                list_versions = true;
            }
            "--list-signatures" => {
                list_signatures = true;
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
        launcher_port: launcher_port.unwrap_or(if sign_server { 53312 } else { 8080 }),
        game_port: game_port.unwrap_or(53310),
        version,
        character_slot,
        hd,
        sign_server,
        friend_signature,
        list_versions,
        list_signatures,
    })
}

fn resolve_game_root() -> PathBuf {
    // Try current working directory first, then exe parent directory
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
        // Try to extract error message
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

    // Find the character by slot
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
        char_new: char.is_new,
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
        friend_signature: args.friend_signature.clone(),
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

    // Check for help first
    if args.iter().any(|a| a.as_str() == "--help" || a.as_str() == "-h") {
        print_usage();
        return true;
    }

    // --list-versions / --list-version (no other args needed)
    if args.iter().any(|a| a.as_str() == "--list-versions" || a.as_str() == "--list-version") {
        println!("Available game versions:");
        for v in mhf_iel::MhfVersion::all_versions().iter() {
            println!("  {}", crate::version_to_label(*v));
        }
        return true;
    }

    // --list-signatures (requires -v)
    if args.iter().any(|a| a.as_str() == "--list-signatures") {
        let version_arg = args.iter().position(|a| a == "-v" || a == "--version").map(|p| args[p + 1].clone());
        match version_arg {
            Some(version) => {
                let version = match crate::label_to_version(&version) {
                    Some(v) => v,
                    None => {
                        eprintln!("Error: invalid version '{}' for --list-signatures", version);
                        return true;
                    }
                };
                let sigs = mhf_iel::available_friend_signatures(version, false);
                if sigs.is_empty() {
                    println!("No version signatures available for version '{}'", crate::version_to_label(version));
                } else {
                    println!("Available version signatures for '{}':", crate::version_to_label(version));
                    for (i, sig) in sigs.iter().enumerate() {
                        println!("  {}. {}", i + 1, sig);
                    }
                }
            }
            None => {
                eprintln!("Error: --list-signatures requires -v/--version");
                return true;
            }
        }
        return true;
    }

    // Check if any CLI flags are present
    let has_cli_flags = args.iter().any(|a| {
        matches!(
            a.as_str(),
            "-v" | "--version" | "-u" | "--username" | "-pw" | "--password"
                | "-s" | "--server" | "-p1" | "--launcher-port" | "-p2" | "--game-port"
                | "-c" | "--character" | "-HD" | "--hd" | "-t" | "--sign-server"
                | "-fs" | "--friend-signature" | "--list-versions" | "--list-version" | "--list-signatures"
        )
    });

    if !has_cli_flags {
        return false;
    }

    // Parse args
    let mut cli_args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            print_usage();
            return true;
        }
    };

    // --list-versions
    if cli_args.list_versions {
        println!("Available game versions:");
        for v in mhf_iel::MhfVersion::all_versions().iter() {
            println!("  {}", crate::version_to_label(*v));
        }
        return true;
    }

    // --list-signatures (requires -v)
    if cli_args.list_signatures {
        let version = match crate::label_to_version(&cli_args.version) {
            Some(v) => v,
            None => {
                eprintln!("Error: invalid version '{}' for --list-signatures", cli_args.version);
                return true;
            }
        };
        let sigs = mhf_iel::available_friend_signatures(version, cli_args.hd);
        if sigs.is_empty() {
            println!("No version signatures available for version '{}'", cli_args.version);
        } else {
            println!("Available version signatures for '{}':", cli_args.version);
            for (i, sig) in sigs.iter().enumerate() {
                println!("  {}. {}", i + 1, sig);
            }
        }
        return true;
    }

    // Interactive version signature prompt (no -fs provided)
    if cli_args.friend_signature.is_none() {
        let version = match crate::label_to_version(&cli_args.version) {
            Some(v) => v,
            None => {
                eprintln!("Error: invalid version '{}'", cli_args.version);
                return true;
            }
        };
        let sigs = mhf_iel::available_friend_signatures(version, cli_args.hd);
        if sigs.is_empty() {
            // No signatures for this version — default to none, skip prompt
            cli_args.friend_signature = Some("none".to_string());
        } else {
            println!("Available version signatures:");
            println!("  1. none (disabled)");
            for (i, sig) in sigs.iter().enumerate() {
                println!("  {}. {}", i + 2, sig);
            }
            print!("Select signature [1-{}]: ", sigs.len() + 1);
            std::io::Write::flush(&mut std::io::stdout()).ok();
            let input = read_console_line();
            let choice = input.trim().parse::<usize>().unwrap_or(1);
            match choice {
                1 => cli_args.friend_signature = Some("none".to_string()),
                n if n >= 2 && n <= sigs.len() + 1 => {
                    cli_args.friend_signature = Some(sigs[n - 2].clone());
                }
                _ => {
                    eprintln!("Invalid choice, using none");
                    cli_args.friend_signature = Some("none".to_string());
                }
            }
            println!("Using signature: {}", cli_args.friend_signature.as_deref().unwrap());
        }
    } else if cli_args.friend_signature.is_none() {
        cli_args.friend_signature = Some("none".to_string());
    }

    info!(
        "CLI launch: user={}, host={}, port1={}, port2={}, version={}, char={}, hd={}, sign_server={}",
        cli_args.username,
        cli_args.server_host,
        cli_args.launcher_port,
        cli_args.game_port,
        cli_args.version,
        cli_args.character_slot,
        cli_args.hd,
        cli_args.sign_server,
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
        server_mode: if cli_args.sign_server { SERVER_MODE_SIGNV1.to_string() } else { String::new() },
    };

    // Resolve game root
    let game_root = resolve_game_root();
    info!("Game root: {}", game_root.display());

    // Apply HD/SD setting: -HD enables HD, no -HD forces SD
    if cli_args.hd {
        if let Err(e) = settings::set_setting(&game_root, "hdVersion", Value::Bool(true)) {
            warn!("Failed to set HD version: {}", e);
        } else {
            info!("HD mode enabled");
        }
    } else {
        if let Err(e) = settings::set_setting(&game_root, "hdVersion", Value::Bool(false)) {
            warn!("Failed to set SD version: {}", e);
        } else {
            info!("SD mode enabled");
        }
    }

    // Ensure support dirs exist
    if let Err(e) = crate::ensure_mezeporta_support_dirs(&game_root) {
        warn!("Failed to create support dirs: {}", e);
    }

    // Perform auth
    let auth_resp = if cli_args.sign_server {
        // Signv1 (sign server) auth — TCP + Blowfish crypto
        let version = crate::label_to_version(&cli_args.version)
            .unwrap_or(mhf_iel::MhfVersion::ZZ);
        match sign_server::sign_auth(
            &cli_args.server_host,
            cli_args.launcher_port,
            &cli_args.username,
            &cli_args.password,
            version,
        ) {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Sign server auth error: {}", e);
                return true;
            }
        }
    } else {
        // API server auth — HTTP/JSON
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

        match rt.block_on(cli_auth(
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
        }
    };

    eprintln!("[CLI] auth successful: user={}, chars={}, entrance_count={}, sign_server={}",
        cli_args.username,
        auth_resp.characters.len(),
        auth_resp.entrance_count,
        cli_args.sign_server,
    );
    eprintln!("[CLI] char[{}]: id={}, name={}, hr={}, gr={}, weapon={}",
        cli_args.character_slot,
        auth_resp.characters.get(cli_args.character_slot).map(|c| c.id).unwrap_or(0),
        auth_resp.characters.get(cli_args.character_slot).map(|c| &c.name).unwrap_or(&"".to_string()),
        auth_resp.characters.get(cli_args.character_slot).map(|c| c.hr).unwrap_or(0),
        auth_resp.characters.get(cli_args.character_slot).map(|c| c.gr).unwrap_or(0),
        auth_resp.characters.get(cli_args.character_slot).map(|c| c.weapon).unwrap_or(0),
    );
    eprintln!("[CLI] user_rights: {}, token: {}", auth_resp.user.rights, auth_resp.user.token);
    eprintln!("[CLI] friends: {}", auth_resp.friends.len());
    for (i, f) in auth_resp.friends.iter().enumerate() {
        let ext_id = mhf_iel::make_ext_id(f.id);
        eprintln!("[CLI] friend[{}]: cid={}, id={}, ext_id={}, name={}", i, f.cid, f.id, ext_id, f.name);
    }

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

    // Launch!
    // On Windows, run_mhf ignores game_root, launcher_prefs, wait_for_exit, and app_handle
    // On Linux, minimal values are passed since CLI mode has no Tauri app handle.
    // On success the game runs in-process; always return true so main() exits cleanly.
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
