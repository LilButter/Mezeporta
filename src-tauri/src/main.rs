#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod altclient_stats;
mod config;
mod endpoint;
mod manifest;
mod patcher;
mod server;
mod settings;
mod store;
mod user;

use crate::config::{CLASSIC_STYLE, DEFAULT_MESSAGELIST_URL, DEFAULT_SERVERLIST_URL, PS4_STYLE};
use crate::endpoint::{Endpoint, EndpointConfig, EndpointVecExt};
use base64::Engine;
use log::{error, info, warn};
use meze_butter as mhf_iel;
use meze_butter::MhfConfig;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use server::{
    AuthResponse, JsonRequest, LauncherHeaders, LauncherPs4Assets, LauncherResponse, MessageData,
    PatcherResponse,
};
use settings::Settings;
#[cfg(windows)]
use std::collections::hash_map::DefaultHasher;
use std::fs;
#[cfg(windows)]
use std::hash::{Hash, Hasher};
#[cfg(not(windows))]
use std::io::Write;
#[cfg(not(windows))]
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
#[cfg(not(windows))]
use std::sync::OnceLock;
use std::{
    collections::HashMap,
    fs::File,
    path::{self, Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};
use store::StoreHelper;
use tauri::utils::config::WindowConfig;
use tauri::{async_runtime::Mutex, AppHandle, Manager, PhysicalSize, Window, WindowBuilder};
use tauri_plugin_log::LogTarget;
use tauri_plugin_store::StoreBuilder;
use tokio_util::sync::CancellationToken;
use user::{UserData, UserManager};
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, LRESULT, RECT, WPARAM};
#[cfg(windows)]
use windows::Win32::System::Threading::{CreateMutexW, OpenMutexW, MUTEX_ALL_ACCESS};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WMSZ_BOTTOM, WMSZ_BOTTOMLEFT,
    WMSZ_LEFT, WMSZ_RIGHT, WMSZ_TOP, WMSZ_TOPLEFT, WMSZ_TOPRIGHT, WM_SIZING, WNDPROC,
};

#[cfg_attr(not(windows), allow(dead_code))]
enum ExitSignal {
    RunGame(u32, bool),
}

const CONFIG_VERSION: &str = "1.5.2";
const ALT_CHARACTER_CACHE_ROOT: &str = "Mezeporta/Servers";
const ALT_CHARACTER_CACHE_FILE: &str = "savedata";
const ALT_CHARACTER_VERSION_FILE: &str = "GSV.json";
const CLASSIC_ASPECT_NUMERATOR: i32 = 281;
const CLASSIC_ASPECT_DENOMINATOR: i32 = 150;
const WIDESCREEN_ASPECT_NUMERATOR: i32 = 16;
const WIDESCREEN_ASPECT_DENOMINATOR: i32 = 9;
const CLASSIC_DEFAULT_WINDOW_WIDTH: u32 = 1124;
const CLASSIC_DEFAULT_WINDOW_HEIGHT: u32 = 600;
const PS4_DEFAULT_WINDOW_WIDTH: u32 = 1280;
const PS4_DEFAULT_WINDOW_HEIGHT: u32 = 720;
const WINDOW_MAIN_CONFIG_JSON: &str = include_str!("../window.main.json");
#[cfg(not(windows))]
const MEZE_DEPS_RESOURCE_PATH: &str = "bin/meze-deps.exe";
#[cfg(not(windows))]
const MEZE_DEPS_STAGED_PATH: &str = "Mezeporta/bin/meze-deps.exe";
#[cfg(not(windows))]
const LINUX_UI_SFX_RESOURCE_DIR: &str = "audio";
const WEBVIEW_DATA_DIR: &str = "Mezeporta/WebView";
const CONFIG_STORE_REL_PATH: &str = "Mezeporta/config.json";
#[cfg(not(windows))]
const WINE_PREFIX_DIR: &str = "Mezeporta/WinePrefix";
const WINE_PREFIX_MODE_PORTABLE: &str = "portable";
const WINE_PREFIX_MODE_SYSTEM: &str = "system";
const WINE_PREFIX_MODE_CUSTOM: &str = "custom";
const WINE_PREFIX_MODE_PROTON: &str = "proton";
#[cfg(not(windows))]
const WINE_DLL_OVERRIDES_KEY: &str = r"HKCU\Software\Wine\DllOverrides";
#[cfg(not(windows))]
const CONTROLLER_DLL_OVERRIDE_VALUE: &str = "native,builtin";
#[cfg(not(windows))]
const CONTROLLER_DLL_OVERRIDE_NAMES: [&str; 3] = ["xinput1_3", "dinput", "dinput8"];
#[cfg(windows)]
const CONTROLLER_DLL_FILE_NAMES: [&str; 3] = ["XInput1_3.dll", "Dinput.dll", "Dinput8.dll"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LauncherWindowSize {
    width: u32,
    height: u32,
}

impl LauncherWindowSize {
    fn to_physical(self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.width, self.height)
    }
}

#[cfg(windows)]
static PS4_ASPECT_LOCK_ENABLED: AtomicBool = AtomicBool::new(false);
#[cfg(windows)]
static PS4_ASPECT_LOCK_INSTALLED: AtomicBool = AtomicBool::new(false);
#[cfg(windows)]
static PS4_ASPECT_NUMERATOR: AtomicI32 = AtomicI32::new(WIDESCREEN_ASPECT_NUMERATOR);
#[cfg(windows)]
static PS4_ASPECT_DENOMINATOR: AtomicI32 = AtomicI32::new(WIDESCREEN_ASPECT_DENOMINATOR);
#[cfg(windows)]
static mut PS4_ASPECT_OLD_WNDPROC: WNDPROC = None;

fn ensure_endpoint_shape(value: &mut Value, is_remote_default: bool) -> bool {
    let Some(obj) = value.as_object_mut() else {
        return false;
    };

    let mut changed = false;

    if obj
        .get("url")
        .and_then(Value::as_str)
        .map_or(true, |v| v.trim().is_empty())
    {
        if let Some(host) = obj.get("host").and_then(Value::as_str).map(str::trim) {
            if !host.is_empty() {
                obj.insert("url".to_string(), Value::String(host.to_string()));
                changed = true;
            }
        }
    }

    if let Some(url) = obj.get("url").and_then(Value::as_str) {
        let trimmed = url.trim();
        if trimmed != url {
            obj.insert("url".to_string(), Value::String(trimmed.to_string()));
            changed = true;
        }
    }

    if obj
        .get("name")
        .and_then(Value::as_str)
        .map_or(true, |v| v.trim().is_empty())
    {
        if let Some(url) = obj.get("url").and_then(Value::as_str).map(str::trim) {
            if !url.is_empty() {
                obj.insert("name".to_string(), Value::String(url.to_string()));
                changed = true;
            }
        }
    }

    let normalize_port = |obj: &mut serde_json::Map<String, Value>, key: &str| -> bool {
        let Some(current) = obj.get(key).cloned() else {
            obj.insert(key.to_string(), Value::Null);
            return true;
        };

        match current {
            Value::Null | Value::Number(_) => false,
            Value::String(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    obj.insert(key.to_string(), Value::Null);
                    true
                } else if let Ok(parsed) = trimmed.parse::<u16>() {
                    obj.insert(key.to_string(), Value::from(parsed));
                    true
                } else {
                    obj.insert(key.to_string(), Value::Null);
                    true
                }
            }
            _ => {
                obj.insert(key.to_string(), Value::Null);
                true
            }
        }
    };

    changed |= normalize_port(obj, "launcherPort");
    changed |= normalize_port(obj, "gamePort");

    if !obj.contains_key("gameFolder") {
        if let Some(game_path) = obj.remove("gamePath") {
            obj.insert("gameFolder".to_string(), game_path);
            changed = true;
        } else {
            obj.insert("gameFolder".to_string(), Value::Null);
            changed = true;
        }
    }

    if !obj.contains_key("version")
        || !obj
            .get("version")
            .is_some_and(|value| matches!(value, Value::String(text) if !text.trim().is_empty()))
    {
        obj.insert("version".to_string(), Value::String("ZZ".to_string()));
        changed = true;
    }

    if !obj.contains_key("isRemote") {
        obj.insert("isRemote".to_string(), Value::Bool(is_remote_default));
        changed = true;
    }

    if obj.remove("legacy").is_some() {
        changed = true;
    }

    changed
}

#[cfg(not(windows))]
fn appimage_parent_dir() -> Option<PathBuf> {
    std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

#[cfg(not(windows))]
fn argv0_parent_dir() -> Option<PathBuf> {
    std::env::args_os()
        .next()
        .map(PathBuf::from)
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case("AppImage"))
        })
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

#[cfg(not(windows))]
fn executable_parent_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

#[cfg(not(windows))]
fn is_ephemeral_appimage_runtime_dir(path: &Path) -> bool {
    let display = path.to_string_lossy();
    display.contains("/.mount_")
        || display.contains("/squashfs-root")
        || display.starts_with("/tmp/.mount_")
        || display.starts_with("/run/user/")
}

fn default_effective_folder() -> PathBuf {
    #[cfg(all(not(windows), debug_assertions))]
    {
        if let Ok(current_dir) = std::env::current_dir() {
            return current_dir;
        }
    }

    #[cfg(not(windows))]
    {
        if let Some(appimage_root) = appimage_parent_dir() {
            return appimage_root;
        }

        if let Some(argv0_root) = argv0_parent_dir() {
            return argv0_root;
        }

        if let Some(exe_root) = executable_parent_dir() {
            if !is_ephemeral_appimage_runtime_dir(&exe_root) {
                return exe_root;
            }
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn launcher_style_is_resizable(style: u32) -> bool {
    matches!(style, CLASSIC_STYLE | PS4_STYLE)
}

fn normalize_launcher_style(style: u32) -> u32 {
    if style == PS4_STYLE {
        PS4_STYLE
    } else {
        CLASSIC_STYLE
    }
}

fn default_launcher_window_size(style: u32) -> LauncherWindowSize {
    match normalize_launcher_style(style) {
        CLASSIC_STYLE => LauncherWindowSize {
            width: CLASSIC_DEFAULT_WINDOW_WIDTH,
            height: CLASSIC_DEFAULT_WINDOW_HEIGHT,
        },
        PS4_STYLE => LauncherWindowSize {
            width: PS4_DEFAULT_WINDOW_WIDTH,
            height: PS4_DEFAULT_WINDOW_HEIGHT,
        },
        _ => LauncherWindowSize {
            width: CLASSIC_DEFAULT_WINDOW_WIDTH,
            height: CLASSIC_DEFAULT_WINDOW_HEIGHT,
        },
    }
}

fn normalize_launcher_window_dimension(value: u32, fallback: u32) -> u32 {
    if value == 0 {
        return fallback;
    }
    value.clamp(320, 8192)
}

fn format_resolution_value(width: u32, height: u32) -> String {
    format!("{}x{}", width, height)
}

fn normalize_recent_resolution_entry(value: &str) -> Option<String> {
    let (width_text, height_text) = value.trim().split_once('x')?;
    let width = width_text.trim().parse::<u32>().ok()?;
    let height = height_text.trim().parse::<u32>().ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    Some(format_resolution_value(width, height))
}

fn normalize_recent_resolution_list(values: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in values {
        let Some(entry) = normalize_recent_resolution_entry(value) else {
            continue;
        };
        if normalized.iter().any(|current| current == &entry) {
            continue;
        }
        normalized.push(entry);
        if normalized.len() >= 5 {
            break;
        }
    }
    normalized
}

fn migrate_endpoint_collection(value: &mut Value, is_remote_default: bool) -> bool {
    let Some(entries) = value.as_array_mut() else {
        return false;
    };

    let mut changed = false;
    for entry in entries {
        changed |= ensure_endpoint_shape(entry, is_remote_default);
    }
    changed
}

fn migrate_store_value<F>(
    store: &mut tauri_plugin_store::Store<tauri::Wry>,
    key: &str,
    mutator: F,
) -> bool
where
    F: FnOnce(&mut Value) -> bool,
{
    let Some(mut current) = store.get(key).cloned() else {
        return false;
    };

    if !mutator(&mut current) {
        return false;
    }

    if let Err(e) = store.insert(key.to_string(), current) {
        warn!("unable to migrate store key '{}': {}", key, e);
        return false;
    }

    true
}

fn migrate_config_store(store: &mut tauri_plugin_store::Store<tauri::Wry>) -> bool {
    let current_version = store
        .get("config_version")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();

    if current_version == CONFIG_VERSION {
        return false;
    }

    let mut changed = false;

    changed |= migrate_store_value(store, "endpoints", |value| {
        migrate_endpoint_collection(value, false)
    });
    changed |= migrate_store_value(store, "remote_endpoints", |value| {
        migrate_endpoint_collection(value, true)
    });
    changed |= migrate_store_value(store, "current_endpoint", |value| {
        ensure_endpoint_shape(value, false)
    });

    changed |= migrate_store_value(store, "launcher_prefs", |value| {
        let Some(obj) = value.as_object_mut() else {
            return false;
        };

        let mut touched = false;

        if !obj.contains_key("preloadControllerDlls") {
            let preload = obj
                .remove("preload_controller_dlls")
                .unwrap_or(Value::Bool(false));
            obj.insert("preloadControllerDlls".to_string(), preload);
            touched = true;
        }

        if !obj.contains_key("friendSignature") {
            let normalized = obj
                .remove("friend_signature")
                .and_then(|value| value.as_str().map(normalize_friend_signature))
                .unwrap_or_else(default_friend_signature);
            obj.insert("friendSignature".to_string(), Value::String(normalized));
            touched = true;
        } else if let Some(current) = obj.get("friendSignature").and_then(Value::as_str) {
            let normalized = normalize_friend_signature(current);
            if normalized != current {
                obj.insert("friendSignature".to_string(), Value::String(normalized));
                touched = true;
            }
        }

        if !obj.contains_key("winePrefixMode") {
            let normalized = obj
                .remove("wine_prefix_mode")
                .and_then(|value| value.as_str().map(normalize_wine_prefix_mode))
                .unwrap_or_else(default_wine_prefix_mode);
            obj.insert("winePrefixMode".to_string(), Value::String(normalized));
            touched = true;
        } else if let Some(current) = obj.get("winePrefixMode").and_then(Value::as_str) {
            let normalized = normalize_wine_prefix_mode(current);
            if normalized != current {
                obj.insert("winePrefixMode".to_string(), Value::String(normalized));
                touched = true;
            }
        }

        if !obj.contains_key("winePrefixCustomPath") {
            let normalized = obj
                .remove("wine_prefix_custom_path")
                .and_then(|value| value.as_str().map(|text| text.to_string()));
            let normalized = normalize_wine_prefix_custom_path(normalized.as_deref());
            obj.insert(
                "winePrefixCustomPath".to_string(),
                normalized.map(Value::String).unwrap_or(Value::Null),
            );
            touched = true;
        } else {
            let normalized = normalize_wine_prefix_custom_path(
                obj.get("winePrefixCustomPath").and_then(Value::as_str),
            );
            let current = obj
                .get("winePrefixCustomPath")
                .cloned()
                .unwrap_or(Value::Null);
            let next = normalized.map(Value::String).unwrap_or(Value::Null);
            if current != next {
                obj.insert("winePrefixCustomPath".to_string(), next);
                touched = true;
            }
        }

        touched
    });

    changed |= migrate_store_value(store, "ui_prefs", |value| {
        let Some(obj) = value.as_object_mut() else {
            return false;
        };

        let mut touched = false;
        if !obj.contains_key("fontPreset") {
            obj.insert(
                "fontPreset".to_string(),
                Value::String("default".to_string()),
            );
            touched = true;
        }
        if !obj.contains_key("sfxVolume") {
            obj.insert("sfxVolume".to_string(), Value::from(30));
            touched = true;
        }
        if !obj.contains_key("gameVersion") {
            obj.insert("gameVersion".to_string(), Value::String("ZZ".to_string()));
            touched = true;
        }
        if !obj.contains_key("forceGameVersion") {
            obj.insert("forceGameVersion".to_string(), Value::Bool(false));
            touched = true;
        }
        if !obj.contains_key("devMode") {
            obj.insert("devMode".to_string(), Value::Bool(false));
            touched = true;
        }
        if !obj.contains_key("serverVersionPromptDisabled") {
            obj.insert(
                "serverVersionPromptDisabled".to_string(),
                Value::Bool(false),
            );
            touched = true;
        }
        if !obj.contains_key("launcherController") {
            obj.insert("launcherController".to_string(), Value::Bool(true));
            touched = true;
        }
        if !obj.contains_key("linuxHardwareAcceleration") {
            obj.insert("linuxHardwareAcceleration".to_string(), Value::Bool(true));
            touched = true;
        }
        if !obj.contains_key("offlineImages") {
            obj.insert("offlineImages".to_string(), Value::Bool(false));
            touched = true;
        }
        if !obj.contains_key("classicLauncherRecentResolutions") {
            obj.insert(
                "classicLauncherRecentResolutions".to_string(),
                Value::Array(Vec::new()),
            );
            touched = true;
        }
        if !obj.contains_key("classicLauncherCustomResolution") {
            obj.insert(
                "classicLauncherCustomResolution".to_string(),
                Value::Bool(false),
            );
            touched = true;
        }
        if !obj.contains_key("ps4LauncherRecentResolutions") {
            obj.insert(
                "ps4LauncherRecentResolutions".to_string(),
                Value::Array(Vec::new()),
            );
            touched = true;
        }
        if !obj.contains_key("ps4LauncherCustomResolution") {
            obj.insert(
                "ps4LauncherCustomResolution".to_string(),
                Value::Bool(false),
            );
            touched = true;
        }
        touched
    });

    if store.get("config_version").and_then(Value::as_str) != Some(CONFIG_VERSION) {
        if let Err(e) = store.insert(
            "config_version".to_string(),
            Value::String(CONFIG_VERSION.to_string()),
        ) {
            warn!("unable to set config_version: {}", e);
        } else {
            changed = true;
        }
    }

    if changed {
        if let Err(e) = store.save() {
            warn!("unable to save migrated config store: {}", e);
            return false;
        }
        info!("migrated Mezeporta config schema to {}", CONFIG_VERSION);
    }

    changed
}

fn normalize_remote_url(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = Url::parse(trimmed).ok()?;
    match parsed.scheme() {
        "http" | "https" => Some(trimmed.to_string()),
        _ => None,
    }
}

fn version_to_label(version: mhf_iel::MhfVersion) -> &'static str {
    match version {
        mhf_iel::MhfVersion::S6 => "S6",
        mhf_iel::MhfVersion::S7K => "S7K",
        mhf_iel::MhfVersion::F4 => "F4",
        mhf_iel::MhfVersion::F5 => "F5",
        mhf_iel::MhfVersion::G1 => "G1",
        mhf_iel::MhfVersion::G2 => "G2",
        mhf_iel::MhfVersion::G3 => "G3",
        mhf_iel::MhfVersion::G3_1 => "G3.1",
        mhf_iel::MhfVersion::G3_2 => "G3.2",
        mhf_iel::MhfVersion::GG => "GG",
        mhf_iel::MhfVersion::G5 => "G5",
        mhf_iel::MhfVersion::G5_1 => "G5.1",
        mhf_iel::MhfVersion::G5_2 => "G5.2",
        mhf_iel::MhfVersion::G6 => "G6",
        mhf_iel::MhfVersion::G7 => "G7",
        mhf_iel::MhfVersion::G9_1 => "G9.1",
        mhf_iel::MhfVersion::G10_1 => "G10.1",
        mhf_iel::MhfVersion::Z1 => "Z1",
        mhf_iel::MhfVersion::Z2 => "Z2",
        mhf_iel::MhfVersion::Z2T => "Z2T",
        mhf_iel::MhfVersion::ZZ => "ZZ",
    }
}

fn version_to_savedata_token(version: mhf_iel::MhfVersion) -> &'static str {
    match version {
        mhf_iel::MhfVersion::Z2T => "Z2.2",
        _ => version_to_label(version),
    }
}

fn label_to_version(label: &str) -> Option<mhf_iel::MhfVersion> {
    serde_json::from_value::<mhf_iel::MhfVersion>(Value::String(label.trim().to_string())).ok()
}

fn detect_game_version_from_folder(game_root: &Path) -> Option<mhf_iel::MhfVersion> {
    const VERSION_SIGNATURES: &[(&str, mhf_iel::MhfVersion)] = &[
        (
            "MONSTER HUNTER FRONTIER SEASON 6.0",
            mhf_iel::MhfVersion::S6,
        ),
        (
            "MONSTER HUNTER FRONTIER SEASON 7.0",
            mhf_iel::MhfVersion::S7K,
        ),
        (
            "MONSTER HUNTER FRONTIER ONLINE v1.20_107869",
            mhf_iel::MhfVersion::F4,
        ),
        ("v1.20_107869", mhf_iel::MhfVersion::F4),
        ("MONSTER HUNTER FRONTIER FORWARD.4", mhf_iel::MhfVersion::F4),
        (
            "MONSTER HUNTER FRONTIER ONLINE v1.20_125635",
            mhf_iel::MhfVersion::F5,
        ),
        (
            "MONSTER HUNTER FRONTIER ONLINE v1.20_133710",
            mhf_iel::MhfVersion::F5,
        ),
        ("v1.20_125635", mhf_iel::MhfVersion::F5),
        ("v1.20_133710", mhf_iel::MhfVersion::F5),
        ("MONSTER HUNTER FRONTIER FORWARD.5", mhf_iel::MhfVersion::F5),
        (
            "MONSTER HUNTER FRONTIER G v1.22_153077",
            mhf_iel::MhfVersion::G1,
        ),
        ("v1.22_153077", mhf_iel::MhfVersion::G1),
        (
            "MONSTER HUNTER FRONTIER G v1.22_156129",
            mhf_iel::MhfVersion::G1,
        ),
        ("v1.22_156129", mhf_iel::MhfVersion::G1),
        (
            "MONSTER HUNTER FRONTIER G v1.23_187828",
            mhf_iel::MhfVersion::G2,
        ),
        ("v1.23_187828", mhf_iel::MhfVersion::G2),
        ("MONSTER HUNTER FRONTIER G3", mhf_iel::MhfVersion::G3),
        ("MONSTER HUNTER FRONTIER G3.1", mhf_iel::MhfVersion::G3_1),
        ("MONSTER HUNTER FRONTIER G3.2", mhf_iel::MhfVersion::G3_2),
        ("MONSTER HUNTER FRONTIER GG", mhf_iel::MhfVersion::GG),
        ("MONSTER HUNTER FRONTIER G4", mhf_iel::MhfVersion::GG),
        ("MONSTER HUNTER FRONTIER G5", mhf_iel::MhfVersion::G5),
        ("MONSTER HUNTER FRONTIER G5.1", mhf_iel::MhfVersion::G5_1),
        ("MONSTER HUNTER FRONTIER G5.2", mhf_iel::MhfVersion::G5_2),
        ("MONSTER HUNTER FRONTIER G5.3", mhf_iel::MhfVersion::G5_2),
        (
            "MONSTER HUNTER FRONTIER G v1.33_325336",
            mhf_iel::MhfVersion::G6,
        ),
        (
            "MONSTER HUNTER FRONTIER G v1.33_326088",
            mhf_iel::MhfVersion::G6,
        ),
        ("v1.33_325336", mhf_iel::MhfVersion::G6),
        ("v1.33_326088", mhf_iel::MhfVersion::G6),
        ("MONSTER HUNTER FRONTIER G6", mhf_iel::MhfVersion::G6),
        ("MONSTER HUNTER FRONTIER G6.1", mhf_iel::MhfVersion::G6),
        ("MONSTER HUNTER FRONTIER G7", mhf_iel::MhfVersion::G7),
        (
            "MONSTER HUNTER FRONTIER G v1.38.19_e8966870",
            mhf_iel::MhfVersion::G9_1,
        ),
        (
            "MONSTER HUNTER FRONTIER G v1.38.19_47c90390",
            mhf_iel::MhfVersion::G9_1,
        ),
        ("v1.38.19_e8966870", mhf_iel::MhfVersion::G9_1),
        ("v1.38.19_47c90390", mhf_iel::MhfVersion::G9_1),
        ("MONSTER HUNTER FRONTIER G9", mhf_iel::MhfVersion::G9_1),
        ("MONSTER HUNTER FRONTIER G9.1", mhf_iel::MhfVersion::G9_1),
        (
            "MONSTER HUNTER FRONTIER G v1.41.30_c730c673",
            mhf_iel::MhfVersion::G10_1,
        ),
        (
            "MONSTER HUNTER FRONTIER G v1.41.30_f5ed3a6a",
            mhf_iel::MhfVersion::G10_1,
        ),
        (
            "MONSTER HUNTER FRONTIER G v1.41.32_8acc3715",
            mhf_iel::MhfVersion::G10_1,
        ),
        (
            "MONSTER HUNTER FRONTIER G v1.41.32_5c06b547",
            mhf_iel::MhfVersion::G10_1,
        ),
        ("v1.41.30_c730c673", mhf_iel::MhfVersion::G10_1),
        ("v1.41.30_f5ed3a6a", mhf_iel::MhfVersion::G10_1),
        ("v1.41.32_8acc3715", mhf_iel::MhfVersion::G10_1),
        ("v1.41.32_5c06b547", mhf_iel::MhfVersion::G10_1),
        ("MONSTER HUNTER FRONTIER G10", mhf_iel::MhfVersion::G10_1),
        ("MONSTER HUNTER FRONTIER G10.1", mhf_iel::MhfVersion::G10_1),
        (
            "MONSTER HUNTER FRONTIER Z v1.44.45_15a73eb7",
            mhf_iel::MhfVersion::Z1,
        ),
        (
            "MONSTER HUNTER FRONTIER Z v1.44.45_dca95f5f",
            mhf_iel::MhfVersion::Z1,
        ),
        ("v1.44.45_15a73eb7", mhf_iel::MhfVersion::Z1),
        ("v1.44.45_dca95f5f", mhf_iel::MhfVersion::Z1),
        ("MONSTER HUNTER FRONTIER Z1", mhf_iel::MhfVersion::Z1),
        ("MONSTER HUNTER FRONTIER Z1.1", mhf_iel::MhfVersion::Z1),
        ("MONSTER HUNTER FRONTIER Z1.2", mhf_iel::MhfVersion::Z1),
        ("MONSTER HUNTER FRONTIER Z2", mhf_iel::MhfVersion::Z2),
        ("MONSTER HUNTER FRONTIER Z2.1", mhf_iel::MhfVersion::Z1),
        ("%c %s G Z2.2.%d:%d", mhf_iel::MhfVersion::Z2T),
        ("MONSTER HUNTER FRONTIER Z2.2", mhf_iel::MhfVersion::Z2T),
        ("MONSTER HUNTER FRONTIER Z2.3", mhf_iel::MhfVersion::Z1),
        (
            "MONSTER HUNTER FRONTIER Z v1.52.79_73c49f52",
            mhf_iel::MhfVersion::ZZ,
        ),
        (
            "MONSTER HUNTER FRONTIER Z v1.52.79_04d16dc4",
            mhf_iel::MhfVersion::ZZ,
        ),
        ("v1.52.79_73c49f52", mhf_iel::MhfVersion::ZZ),
        ("v1.52.79_04d16dc4", mhf_iel::MhfVersion::ZZ),
        ("MONSTER HUNTER FRONTIER Z3", mhf_iel::MhfVersion::ZZ),
        ("MONSTER HUNTER FRONTIER Z3.1", mhf_iel::MhfVersion::ZZ),
        ("MONSTER HUNTER FRONTIER ZZ", mhf_iel::MhfVersion::ZZ),
    ];

    let mhfo_bytes = std::fs::read(game_root.join("mhfo.dll")).ok()?;
    let hd_bytes = std::fs::read(game_root.join("mhfo-hd.dll")).ok();

    for bytes in hd_bytes.iter().chain(std::iter::once(&mhfo_bytes)) {
        for &(signature, version) in VERSION_SIGNATURES {
            let ascii = signature.as_bytes();
            if bytes.windows(ascii.len()).any(|window| window == ascii) {
                return Some(version);
            }

            let utf16le: Vec<u8> = signature
                .encode_utf16()
                .flat_map(|ch| ch.to_le_bytes())
                .collect();
            if bytes
                .windows(utf16le.len())
                .any(|window| window == utf16le.as_slice())
            {
                return Some(version);
            }

            let utf16be: Vec<u8> = signature
                .encode_utf16()
                .flat_map(|ch| ch.to_be_bytes())
                .collect();
            if bytes
                .windows(utf16be.len())
                .any(|window| window == utf16be.as_slice())
            {
                return Some(version);
            }
        }
    }

    None
}

fn resolve_effective_game_version(state_sync: &TauriStateSync) -> mhf_iel::MhfVersion {
    state_sync.ui_prefs.game_version
}

#[derive(Clone, Copy)]
struct MutexVersionSelection {
    primary: mhf_iel::MhfVersion,
    fallback: Option<mhf_iel::MhfVersion>,
}

fn map_game_version_to_mutex_version(version: mhf_iel::MhfVersion) -> mhf_iel::MhfVersion {
    version
}

fn resolve_effective_mutex_versions(state_sync: &TauriStateSync) -> MutexVersionSelection {
    let primary = map_game_version_to_mutex_version(resolve_effective_game_version(state_sync));

    MutexVersionSelection {
        primary,
        fallback: None,
    }
}

fn default_friend_signature() -> String {
    "none".to_string()
}

fn default_wine_prefix_mode() -> String {
    WINE_PREFIX_MODE_PORTABLE.to_string()
}

fn normalize_wine_prefix_mode(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        WINE_PREFIX_MODE_SYSTEM => WINE_PREFIX_MODE_SYSTEM.to_string(),
        WINE_PREFIX_MODE_CUSTOM => WINE_PREFIX_MODE_CUSTOM.to_string(),
        WINE_PREFIX_MODE_PROTON => WINE_PREFIX_MODE_PROTON.to_string(),
        _ => WINE_PREFIX_MODE_PORTABLE.to_string(),
    }
}

fn normalize_wine_prefix_custom_path(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_friend_signature(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        "none".to_string()
    } else if trimmed.eq_ignore_ascii_case("detect") || trimmed.eq_ignore_ascii_case("detect-beta")
    {
        "none".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(windows)]
fn single_instance_mutex_name() -> String {
    let normalized_path = std::env::current_exe()
        .ok()
        .map(|path| path.canonicalize().unwrap_or(path))
        .map(|path| {
            path.to_string_lossy()
                .replace('/', "\\")
                .to_ascii_lowercase()
        })
        .unwrap_or_else(|| "unknown-exe".to_string());

    let mut hasher = DefaultHasher::new();
    normalized_path.hash(&mut hasher);

    format!(
        "Local\\MezeportaLauncherSingleInstance-{:016x}",
        hasher.finish()
    )
}

#[cfg(windows)]
fn acquire_single_instance_mutex() -> Option<HANDLE> {
    let mutex_name = single_instance_mutex_name();
    let mutex_name_wide: Vec<u16> = mutex_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mutex_name_ptr = PCWSTR(mutex_name_wide.as_ptr());

    if let Ok(handle) = unsafe { OpenMutexW(MUTEX_ALL_ACCESS, false, mutex_name_ptr) } {
        let _ = unsafe { CloseHandle(handle) };
        info!(
            "launcher instance already running for this executable path, exiting duplicate process"
        );
        std::process::exit(0);
    }

    match unsafe { CreateMutexW(None, false, mutex_name_ptr) } {
        Ok(handle) => Some(handle),
        Err(err) => {
            warn!("failed to create single-instance mutex: {}", err);
            None
        }
    }
}

#[cfg(not(windows))]
fn acquire_single_instance_mutex() {}

#[cfg(windows)]
fn set_ps4_aspect_lock_enabled(enabled: bool, numerator: i32, denominator: i32) {
    PS4_ASPECT_NUMERATOR.store(numerator.max(1), Ordering::Relaxed);
    PS4_ASPECT_DENOMINATOR.store(denominator.max(1), Ordering::Relaxed);
    PS4_ASPECT_LOCK_ENABLED.store(enabled, Ordering::Relaxed);
}

#[cfg(not(windows))]
fn set_ps4_aspect_lock_enabled(_enabled: bool, _numerator: i32, _denominator: i32) {}

#[cfg(windows)]
fn ps4_rect_has_top(edge: u32) -> bool {
    matches!(edge, WMSZ_TOP | WMSZ_TOPLEFT | WMSZ_TOPRIGHT)
}

#[cfg(windows)]
fn ps4_rect_has_left(edge: u32) -> bool {
    matches!(edge, WMSZ_LEFT | WMSZ_TOPLEFT | WMSZ_BOTTOMLEFT)
}

#[cfg(windows)]
fn ps4_scale_dimension(value: i32, numerator: i32, denominator: i32) -> i32 {
    ((value * numerator) + (denominator / 2)) / denominator
}

#[cfg(windows)]
fn ps4_enforce_aspect(rect: &mut RECT, edge: u32) {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return;
    }

    let numerator = PS4_ASPECT_NUMERATOR.load(Ordering::Relaxed).max(1);
    let denominator = PS4_ASPECT_DENOMINATOR.load(Ordering::Relaxed).max(1);

    let use_width = match edge {
        WMSZ_LEFT | WMSZ_RIGHT => true,
        WMSZ_TOP | WMSZ_BOTTOM => false,
        _ => width * denominator >= height * numerator,
    };

    if use_width {
        let target_height = ps4_scale_dimension(width, denominator, numerator);
        if ps4_rect_has_top(edge) {
            rect.top = rect.bottom - target_height;
        } else {
            rect.bottom = rect.top + target_height;
        }
    } else {
        let target_width = ps4_scale_dimension(height, numerator, denominator);
        if ps4_rect_has_left(edge) {
            rect.left = rect.right - target_width;
        } else {
            rect.right = rect.left + target_width;
        }
    }
}

#[cfg(windows)]
unsafe extern "system" fn ps4_aspect_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_SIZING && PS4_ASPECT_LOCK_ENABLED.load(Ordering::Relaxed) {
        let rect_ptr = lparam.0 as *mut RECT;
        if !rect_ptr.is_null() {
            ps4_enforce_aspect(&mut *rect_ptr, wparam.0 as u32);
            return LRESULT(1);
        }
    }

    if let Some(previous) = PS4_ASPECT_OLD_WNDPROC {
        CallWindowProcW(Some(previous), hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

#[cfg(windows)]
fn install_ps4_aspect_lock(window: &Window) {
    if PS4_ASPECT_LOCK_INSTALLED.load(Ordering::Relaxed) {
        return;
    }

    let hwnd = match window.hwnd() {
        Ok(value) => HWND(value.0),
        Err(err) => {
            warn!("failed to get window handle for PS4 aspect lock: {}", err);
            return;
        }
    };

    let previous = unsafe {
        SetWindowLongPtrW(
            hwnd,
            GWLP_WNDPROC,
            ps4_aspect_window_proc as *const () as i32,
        )
    };
    if previous == 0 {
        warn!("failed to install PS4 aspect lock window procedure");
        return;
    }

    unsafe {
        PS4_ASPECT_OLD_WNDPROC = Some(std::mem::transmute(previous));
    }
    PS4_ASPECT_LOCK_INSTALLED.store(true, Ordering::Relaxed);
}

#[cfg(not(windows))]
fn install_ps4_aspect_lock(_window: &Window) {}

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

const BUNDLED_LAUNCHER_FONTS: &[(&str, &[u8])] = &[
    (
        "CreGothic_NHN M.ttf",
        include_bytes!("../../public/fonts/CreGothic_NHN M.ttf"),
    ),
    ("dft_0.ttc", include_bytes!("../../public/fonts/dft_0.ttc")),
    (
        "MS Gothic.ttf",
        include_bytes!("../../public/fonts/MS Gothic.ttf"),
    ),
    (
        "ZenAntique-Regular.ttf",
        include_bytes!("../../public/fonts/ZenAntique-Regular.ttf"),
    ),
];

fn offline_images_readme(style_label: &str) -> String {
    format!(
        "# {style_label} Offline Image Overrides\n\n\
Place replacement launcher images in this folder to override the bundled fallback assets.\n\
Accepted extensions: .png, .webp, .jpg, .jpeg\n\n\
Supported files:\n\
- header\n\
- background\n\
- cog\n\
- capcom\n\
- server-patch\n\
- dialogue\n\n\
Examples:\n\
- header.png\n\
- background.webp\n\
- server-patch.jpg\n\n\
Only these launcher-side images are overridden here for now.\n\
Server-driven banners, messages, and unit cards still come from the server.\n"
    )
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|err| format!("failed to create {}: {}", path.display(), err))
}

fn ensure_config_store_parent(game_root: &Path) -> Result<(), String> {
    if let Some(parent) = config_store_path(game_root).parent() {
        ensure_dir(parent)?;
    }
    Ok(())
}

fn ensure_webview_data_dir(game_root: &Path) -> Result<(), String> {
    ensure_dir(&webview_data_path(game_root))
}

fn write_bundled_file_if_needed(target: &Path, bytes: &[u8]) -> Result<(), String> {
    let needs_write = match fs::metadata(target) {
        Ok(metadata) => metadata.len() != bytes.len() as u64,
        Err(_) => true,
    };
    if !needs_write {
        return Ok(());
    }
    fs::write(target, bytes)
        .map_err(|err| format!("failed to copy bundled file {}: {}", target.display(), err))
}

fn ensure_mezeporta_support_dirs(game_root: &Path) -> Result<(), String> {
    let mezeporta_root = game_root.join("Mezeporta");
    let webview_data_dir = mezeporta_root.join("WebView");
    let fonts_dir = mezeporta_root.join("fonts");
    let custom_fonts_dir = fonts_dir.join("Custom");
    let offline_images_root = mezeporta_root.join("Offline-Images");
    let classic_offline_dir = offline_images_root.join("Classic");
    let ps4_offline_dir = offline_images_root.join("PS4");

    ensure_dir(&fonts_dir)?;
    ensure_dir(&custom_fonts_dir)?;
    ensure_dir(&classic_offline_dir)?;
    ensure_dir(&ps4_offline_dir)?;
    ensure_dir(&webview_data_dir)?;

    #[cfg(not(windows))]
    {
        let bin_dir = mezeporta_root.join("bin");
        let wine_prefix_dir = game_root.join(WINE_PREFIX_DIR);
        for dir in [&bin_dir, &wine_prefix_dir] {
            ensure_dir(dir)?;
        }
    }

    for (file_name, bytes) in BUNDLED_LAUNCHER_FONTS {
        let target = fonts_dir.join(file_name);
        write_bundled_file_if_needed(&target, bytes)?;
    }

    for (folder, readme) in [
        (&classic_offline_dir, offline_images_readme("Classic")),
        (&ps4_offline_dir, offline_images_readme("PS4")),
    ] {
        let readme_path = folder.join("README.md");
        if !readme_path.exists() {
            fs::write(&readme_path, readme)
                .map_err(|err| format!("failed to write {}: {}", readme_path.display(), err))?;
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn staged_meze_deps_path(game_root: &Path) -> PathBuf {
    game_root.join(MEZE_DEPS_STAGED_PATH)
}

fn webview_data_path(game_root: &Path) -> PathBuf {
    game_root.join(WEBVIEW_DATA_DIR)
}

fn config_store_path(game_root: &Path) -> PathBuf {
    game_root.join(CONFIG_STORE_REL_PATH)
}

#[cfg(not(windows))]
fn ensure_linux_path_defaults() {
    use std::env;

    if env::var_os("ALSOFT_DRIVERS").is_none() {
        env::set_var("ALSOFT_DRIVERS", "pulse,null");
    }
    if env::var_os("SDL_AUDIODRIVER").is_none() {
        env::set_var("SDL_AUDIODRIVER", "pulseaudio");
    }
    if let Some(pulse_server) = resolved_linux_pulse_server() {
        env::set_var("PULSE_SERVER", pulse_server);
    }
    if env::var_os("XDG_RUNTIME_DIR").is_none() {
        if let Some(runtime_dir) = resolved_linux_runtime_dir() {
            env::set_var("XDG_RUNTIME_DIR", runtime_dir);
        }
    }

    let path_value = env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = env::split_paths(&path_value).collect();
    let defaults = [
        "/usr/local/sbin",
        "/usr/local/bin",
        "/usr/sbin",
        "/usr/bin",
        "/sbin",
        "/bin",
    ];

    let mut changed = false;
    for default in defaults {
        let default_path = PathBuf::from(default);
        if !paths.iter().any(|existing| existing == &default_path) {
            paths.push(default_path);
            changed = true;
        }
    }

    if changed {
        if let Ok(joined) = env::join_paths(paths) {
            env::set_var("PATH", joined);
        }
    }

    let plugin_dirs: Vec<PathBuf> = [
        "/usr/lib/x86_64-linux-gnu/gstreamer-1.0",
        "/usr/lib/gstreamer-1.0",
        "/usr/lib64/gstreamer-1.0",
        "/usr/local/lib/gstreamer-1.0",
        "/usr/local/lib/x86_64-linux-gnu/gstreamer-1.0",
    ]
    .into_iter()
    .map(PathBuf::from)
    .filter(|path| path.is_dir())
    .collect();

    if !plugin_dirs.is_empty() {
        if let Ok(joined) = env::join_paths(&plugin_dirs) {
            env::set_var("GST_PLUGIN_PATH_1_0", &joined);
            env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", &joined);
            env::set_var("GST_PLUGIN_PATH", &joined);
            env::set_var("GST_PLUGIN_SYSTEM_PATH", &joined);
        }
    }

    let scanner_path = [
        "/usr/lib/x86_64-linux-gnu/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner",
        "/usr/lib/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner",
        "/usr/libexec/gstreamer-1.0/gst-plugin-scanner",
        "/usr/lib64/gstreamer-1.0/gst-plugin-scanner",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|path| path.is_file());

    if let Some(scanner_path) = scanner_path {
        env::set_var("GST_PLUGIN_SCANNER", &scanner_path);
        env::set_var("GST_PLUGIN_SCANNER_1_0", &scanner_path);
    }

    let cache_root = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")));
    if let Some(cache_root) = cache_root {
        let registry_dir = cache_root.join("gstreamer-1.0");
        let _ = fs::create_dir_all(&registry_dir);
        env::set_var("GST_REGISTRY", registry_dir.join("registry-mezeporta.bin"));
    }
}

#[cfg(not(windows))]
fn copy_if_changed(source: &Path, destination: &Path) -> Result<(), String> {
    let needs_copy = if destination.exists() {
        fs::read(source).map_err(|err| format!("failed to read {}: {}", source.display(), err))?
            != fs::read(destination)
                .map_err(|err| format!("failed to read {}: {}", destination.display(), err))?
    } else {
        true
    };

    if needs_copy {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {}", parent.display(), err))?;
        }
        fs::copy(source, destination).map_err(|err| {
            format!(
                "failed to stage helper from {} to {}: {}",
                source.display(),
                destination.display(),
                err
            )
        })?;
    }

    Ok(())
}

#[cfg(not(windows))]
fn resolve_meze_deps_source(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Some(resource_path) = app_handle
        .path_resolver()
        .resolve_resource(MEZE_DEPS_RESOURCE_PATH)
    {
        candidates.push(resource_path);
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            candidates.push(exe_dir.join("bin").join("meze-deps.exe"));
            candidates.push(exe_dir.join("Mezeporta").join("bin").join("meze-deps.exe"));
            if let Some(prefix_dir) = exe_dir.parent() {
                candidates.push(
                    prefix_dir
                        .join("lib")
                        .join("mezeporta")
                        .join("bin")
                        .join("meze-deps.exe"),
                );
                candidates.push(
                    prefix_dir
                        .join("lib")
                        .join("Mezeporta")
                        .join("bin")
                        .join("meze-deps.exe"),
                );
            }
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("bin").join("meze-deps.exe"));
        candidates.push(current_dir.join("Mezeporta").join("bin").join("meze-deps.exe"));
    }

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(
        "meze-deps.exe was not found. Use the AppImage/.deb build or place it at Mezeporta/bin/meze-deps.exe in the portable layout."
            .to_string(),
    )
}

#[cfg(not(windows))]
fn stage_meze_deps(app_handle: &AppHandle, game_root: &Path) -> Result<PathBuf, String> {
    let source = resolve_meze_deps_source(app_handle)?;
    let destination = staged_meze_deps_path(game_root);
    copy_if_changed(&source, &destination)?;
    Ok(destination)
}

fn load_main_window_config() -> Result<WindowConfig, String> {
    serde_json::from_str(WINDOW_MAIN_CONFIG_JSON)
        .map_err(|err| format!("failed to parse window.main.json: {}", err))
}

#[cfg(not(windows))]
fn expand_home_path(path: &str) -> Option<PathBuf> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed == "~" {
        return std::env::var_os("HOME").map(PathBuf::from);
    }

    if let Some(remainder) = trimmed.strip_prefix("~/") {
        return std::env::var_os("HOME").map(|home| PathBuf::from(home).join(remainder));
    }

    Some(PathBuf::from(trimmed))
}

#[cfg(not(windows))]
fn resolve_wine_command() -> PathBuf {
    PathBuf::from(
        std::env::var("MEZEPORTA_WINE_CMD")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                std::env::var("MHF_WINE_CMD")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })
            .unwrap_or_else(|| "wine".to_string()),
    )
}

#[cfg(not(windows))]
fn resolve_proton_command() -> PathBuf {
    PathBuf::from(
        std::env::var("MEZEPORTA_PROTON_CMD")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                std::env::var("PROTON_CMD")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })
            .unwrap_or_else(|| "proton".to_string()),
    )
}

#[cfg(not(windows))]
fn command_exists(command: &Path) -> bool {
    if command.is_absolute() || command.components().count() > 1 {
        return command.exists();
    }

    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| {
                let candidate = dir.join(command);
                candidate.exists()
            })
        })
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn resolve_wine_prefix(
    game_root: &Path,
    launcher_prefs: &LauncherPrefs,
) -> Result<Option<PathBuf>, String> {
    match normalize_wine_prefix_mode(&launcher_prefs.wine_prefix_mode).as_str() {
        WINE_PREFIX_MODE_SYSTEM => Ok(None),
        WINE_PREFIX_MODE_PROTON => Ok(None),
        WINE_PREFIX_MODE_CUSTOM => {
            let custom_path = launcher_prefs
                .wine_prefix_custom_path
                .as_deref()
                .ok_or_else(|| "custom Wine prefix path is empty".to_string())?;
            expand_home_path(custom_path)
                .ok_or_else(|| "custom Wine prefix path is empty".to_string())
                .map(Some)
        }
        _ => Ok(Some(game_root.join(WINE_PREFIX_DIR))),
    }
}

#[cfg(not(windows))]
fn default_system_wine_prefix() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".wine"))
}

#[cfg(not(windows))]
fn wine_prefix_initialized(path: &Path) -> bool {
    path.join("system.reg").is_file()
}

#[cfg(not(windows))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WineRuntimeKind {
    Wine,
    Proton,
}

#[cfg(not(windows))]
#[derive(Clone, Debug)]
struct ResolvedWineRuntime {
    kind: WineRuntimeKind,
    wine_command: PathBuf,
    wineserver_command: PathBuf,
    wine_prefix: Option<PathBuf>,
    apply_esync_fallback: bool,
    apply_fsync_fallback: bool,
}

#[cfg(not(windows))]
impl ResolvedWineRuntime {
    fn apply_env(&self, command: &mut Command) {
        if let Some(prefix) = self.wine_prefix.as_ref() {
            command.env("WINEPREFIX", prefix);
        } else {
            command.env_remove("WINEPREFIX");
        }
        if std::env::var_os("DISPLAY").is_some() {
            command.env("GDK_BACKEND", "x11");
            command.env("WINIT_UNIX_BACKEND", "x11");
            command.env("SDL_VIDEODRIVER", "x11");
            command.env_remove("WAYLAND_DISPLAY");
        }
        if self.apply_esync_fallback {
            command.env("WINEESYNC", "0");
        }
        if self.apply_fsync_fallback {
            command.env("WINEFSYNC", "0");
        }
    }

    fn program_name(&self) -> &'static str {
        match self.kind {
            WineRuntimeKind::Wine => "wine",
            WineRuntimeKind::Proton => "proton",
        }
    }

    fn append_runtime_prefix_args(&self, command: &mut Command) {
        if self.kind == WineRuntimeKind::Proton {
            command.arg("run");
        }
    }
}

#[cfg(not(windows))]
fn run_wine_reg_command(
    runtime: &ResolvedWineRuntime,
    args: &[&str],
) -> Result<std::process::Output, String> {
    let mut command = Command::new(&runtime.wine_command);
    runtime.apply_env(&mut command);
    runtime.append_runtime_prefix_args(&mut command);
    command
        .arg("reg")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| format!("failed to run wine reg: {}", err))
}

#[cfg(not(windows))]
fn run_runtime_command(
    program: &Path,
    runtime: &ResolvedWineRuntime,
    args: &[&str],
    description: &str,
) -> Result<(), String> {
    let mut command = Command::new(program);
    runtime.apply_env(&mut command);
    if program == runtime.wine_command.as_path() {
        runtime.append_runtime_prefix_args(&mut command);
    }
    let output = command
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| format!("failed to run {}: {}", description, err))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let detail = if stderr.is_empty() {
        format!("exit status {}", output.status)
    } else {
        stderr
    };
    Err(format!("{} failed: {}", description, detail))
}

#[cfg(not(windows))]
fn sync_wine_controller_overrides(
    runtime: &ResolvedWineRuntime,
    enabled: bool,
) -> Result<(), String> {
    if enabled {
        for dll_name in CONTROLLER_DLL_OVERRIDE_NAMES {
            let output = run_wine_reg_command(
                runtime,
                &[
                    "add",
                    WINE_DLL_OVERRIDES_KEY,
                    "/v",
                    dll_name,
                    "/t",
                    "REG_SZ",
                    "/d",
                    CONTROLLER_DLL_OVERRIDE_VALUE,
                    "/f",
                ],
            )?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let detail = if stderr.is_empty() {
                    format!("exit status {}", output.status)
                } else {
                    stderr
                };
                return Err(format!(
                    "failed to apply Wine controller override for {}: {}",
                    dll_name, detail
                ));
            }
        }
        return Ok(());
    }

    let prefix_initialized = runtime
        .wine_prefix
        .as_deref()
        .map(wine_prefix_initialized)
        .or_else(|| {
            default_system_wine_prefix()
                .as_deref()
                .map(wine_prefix_initialized)
        })
        .unwrap_or(false);

    if !prefix_initialized {
        return Ok(());
    }

    for dll_name in CONTROLLER_DLL_OVERRIDE_NAMES {
        match run_wine_reg_command(
            runtime,
            &["delete", WINE_DLL_OVERRIDES_KEY, "/v", dll_name, "/f"],
        ) {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if !stderr.is_empty() {
                    warn!(
                        "failed to clear Wine controller override for {}: {}",
                        dll_name, stderr
                    );
                }
            }
            Err(err) => warn!(
                "failed to clear Wine controller override for {}: {}",
                dll_name, err
            ),
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn resolve_wineserver_command(wine_command: &Path) -> PathBuf {
    let has_explicit_path =
        wine_command.is_absolute() || wine_command.components().nth(1).is_some();
    if has_explicit_path {
        if let Some(parent) = wine_command.parent() {
            let sibling = parent.join("wineserver");
            if sibling.exists() {
                return sibling;
            }
        }
    }
    PathBuf::from("wineserver")
}

#[cfg(not(windows))]
fn resolve_wine_runtime(
    game_root: &Path,
    launcher_prefs: &LauncherPrefs,
) -> Result<ResolvedWineRuntime, String> {
    let mode = normalize_wine_prefix_mode(&launcher_prefs.wine_prefix_mode);
    let is_proton = mode == WINE_PREFIX_MODE_PROTON;
    let wine_command = if is_proton {
        resolve_proton_command()
    } else {
        resolve_wine_command()
    };
    Ok(ResolvedWineRuntime {
        kind: if is_proton {
            WineRuntimeKind::Proton
        } else {
            WineRuntimeKind::Wine
        },
        wineserver_command: if is_proton {
            PathBuf::from("wineserver")
        } else {
            resolve_wineserver_command(&wine_command)
        },
        wine_command,
        wine_prefix: resolve_wine_prefix(game_root, launcher_prefs)?,
        apply_esync_fallback: std::env::var_os("WINEESYNC").is_none(),
        apply_fsync_fallback: std::env::var_os("WINEFSYNC").is_none(),
    })
}

#[cfg(not(windows))]
fn find_missing_wine_tools(runtime: &ResolvedWineRuntime) -> Vec<String> {
    let mut missing = Vec::new();
    if !command_exists(&runtime.wine_command) {
        missing.push(runtime.program_name().to_string());
    }
    if runtime.kind == WineRuntimeKind::Proton {
        return missing;
    }
    if !command_exists(&runtime.wineserver_command) {
        missing.push("wineserver".to_string());
    }
    if !command_exists(Path::new("winetricks")) {
        missing.push("winetricks".to_string());
    }
    missing
}

#[cfg(not(windows))]
fn gstreamer_component_available(component: &str) -> bool {
    command_exists(Path::new("gst-inspect-1.0"))
        && Command::new("gst-inspect-1.0")
            .arg(component)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
}

#[cfg(not(windows))]
fn find_missing_linux_audio_runtime() -> Vec<String> {
    if !command_exists(Path::new("gst-inspect-1.0")) {
        return vec!["gst-inspect-1.0".to_string()];
    }

    let mut missing = Vec::new();
    for component in ["uridecodebin", "audioconvert", "audioresample", "volume"] {
        if !gstreamer_component_available(component) {
            missing.push(component.to_string());
        }
    }
    if is_wsl_environment() {
        if !gstreamer_component_available("pulsesink") {
            missing.push("pulsesink".to_string());
        }
    } else if !["pulsesink", "pipewiresink"]
        .iter()
        .any(|component| gstreamer_component_available(component))
    {
        missing.push("audio-sink".to_string());
    }

    missing
}

#[cfg(not(windows))]
fn cached_missing_linux_audio_runtime() -> Vec<String> {
    static MISSING: OnceLock<Vec<String>> = OnceLock::new();
    MISSING.get_or_init(find_missing_linux_audio_runtime).clone()
}

#[cfg(not(windows))]
fn is_wsl_environment() -> bool {
    if std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::env::var_os("WSL_INTEROP").is_some()
        || Path::new("/mnt/wslg").exists()
        || Path::new("/mnt/c/Windows").exists()
    {
        return true;
    }

    ["/proc/sys/kernel/osrelease", "/proc/version"]
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .map(|contents| contents.to_ascii_lowercase())
        .any(|contents| contents.contains("microsoft") || contents.contains("wsl"))
}

#[cfg(not(windows))]
fn unix_audio_path_exists(value: &str) -> bool {
    let trimmed = value.trim();
    let path = trimmed.strip_prefix("unix:").unwrap_or(trimmed);
    !path.is_empty()
        && Path::new(path).exists()
        && std::os::unix::net::UnixStream::connect(path).is_ok()
}

#[cfg(not(windows))]
fn resolved_linux_pulse_server() -> Option<String> {
    if let Ok(server) = std::env::var("PULSE_SERVER") {
        let server = server.trim();
        if server.starts_with("unix:") || server.starts_with('/') {
            if unix_audio_path_exists(server) {
                return Some(server.to_string());
            }
        } else if !server.is_empty() {
            return Some(server.to_string());
        }
    }

    if unix_audio_path_exists("/mnt/wslg/PulseServer") {
        return Some("/mnt/wslg/PulseServer".to_string());
    }

    for candidate in [
        "/run/user/1000/pulse/native",
        "/mnt/wslg/run/user/1000/pulse/native",
    ] {
        if unix_audio_path_exists(candidate) {
            return Some(candidate.to_string());
        }
    }

    for root in ["/run/user", "/mnt/wslg/run/user"] {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("pulse").join("native");
                let candidate = candidate.display().to_string();
                if unix_audio_path_exists(&candidate) {
                    return Some(candidate);
                }
            }
        }
    }

    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .map(|path| path.join("pulse").join("native"))
        .filter(|path| unix_audio_path_exists(&path.display().to_string()))
        .map(|path| path.display().to_string())
}

#[cfg(not(windows))]
fn resolved_linux_runtime_dir() -> Option<String> {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let runtime_dir = runtime_dir.trim();
        if !runtime_dir.is_empty() && Path::new(runtime_dir).exists() {
            return Some(runtime_dir.to_string());
        }
    }

    for candidate in ["/run/user/1000", "/mnt/wslg/run/user/1000"] {
        if Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }

    for root in ["/run/user", "/mnt/wslg/run/user"] {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let candidate = entry.path();
                if candidate.is_dir() {
                    return Some(candidate.display().to_string());
                }
            }
        }
    }

    if Path::new("/mnt/wslg/runtime-dir").exists() {
        return Some("/mnt/wslg/runtime-dir".to_string());
    }

    None
}

#[cfg(not(windows))]
fn pulse_server_available() -> bool {
    resolved_linux_pulse_server().is_some()
}

#[cfg(not(windows))]
fn pipewire_server_available() -> bool {
    resolved_linux_runtime_dir()
        .map(PathBuf::from)
        .map(|path| path.join("pipewire-0").exists())
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn preferred_linux_ui_audio_sink() -> Option<&'static str> {
    static SINK: OnceLock<Option<&'static str>> = OnceLock::new();
    *SINK.get_or_init(|| {
        let is_wsl = is_wsl_environment();
        if is_wsl && gstreamer_component_available("pulsesink") && pulse_server_available() {
            Some("pulsesink")
        } else if gstreamer_component_available("pulsesink") && pulse_server_available() {
            Some("pulsesink")
        } else if !is_wsl
            && gstreamer_component_available("pipewiresink")
            && pipewire_server_available()
        {
            Some("pipewiresink")
        } else if std::env::var("MEZEPORTA_LINUX_SFX_SILENT")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
            && gstreamer_component_available("fakesink")
        {
            Some("fakesink")
        } else {
            None
        }
    })
}

#[cfg(not(windows))]
fn resolve_linux_ui_sfx_source(app_handle: &AppHandle, name: &str) -> Result<PathBuf, String> {
    let safe_name = match name.trim() {
        "hover" | "select" | "confirm" | "start" | "login" => name.trim(),
        _ => return Err(format!("unsupported ui sfx '{}'", name)),
    };

    let mut candidates = Vec::new();
    let mut search_roots = Vec::new();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    search_roots.push(manifest_dir.clone());
    if let Some(project_root) = manifest_dir.parent() {
        search_roots.push(project_root.to_path_buf());
    }
    if let Ok(current_dir) = std::env::current_dir() {
        for ancestor in current_dir.ancestors().take(8) {
            search_roots.push(ancestor.to_path_buf());
        }
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            for ancestor in exe_dir.ancestors().take(10) {
                search_roots.push(ancestor.to_path_buf());
            }
        }
    }

    for extension in ["mp3", "ogg"] {
        let file_name = format!("{}.{}", safe_name, extension);
        if let Some(resource_path) = app_handle
            .path_resolver()
            .resolve_resource(&format!("{}/{}", LINUX_UI_SFX_RESOURCE_DIR, file_name))
        {
            candidates.push(resource_path);
        }
        if let Some(resource_path) = app_handle
            .path_resolver()
            .resolve_resource(&format!("../public/audio/{}", file_name))
        {
            candidates.push(resource_path);
        }
        if let Some(resource_path) = app_handle
            .path_resolver()
            .resolve_resource(&format!("_up_/public/audio/{}", file_name))
        {
            candidates.push(resource_path);
        }

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                candidates.push(exe_dir.join("audio").join(&file_name));
                candidates.push(exe_dir.join("Mezeporta").join("audio").join(&file_name));
                if let Some(prefix_dir) = exe_dir.parent() {
                    candidates.push(
                        prefix_dir
                            .join("lib")
                            .join("mezeporta")
                            .join("audio")
                            .join(&file_name),
                    );
                    candidates.push(
                        prefix_dir
                            .join("lib")
                            .join("mezeporta")
                            .join("_up_")
                            .join("public")
                            .join("audio")
                            .join(&file_name),
                    );
                    candidates.push(
                        prefix_dir
                            .join("lib")
                            .join("Mezeporta")
                            .join("audio")
                            .join(&file_name),
                    );
                    candidates.push(
                        prefix_dir
                            .join("lib")
                            .join("Mezeporta")
                            .join("_up_")
                            .join("public")
                            .join("audio")
                            .join(&file_name),
                    );
                }
            }
        }

        for root in &search_roots {
            candidates.push(root.join("audio").join(&file_name));
            candidates.push(root.join("Mezeporta").join("audio").join(&file_name));
            candidates.push(root.join("public").join("audio").join(&file_name));
        }
    }

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "ui sfx asset '{}' was not found in the Linux bundle layout",
        safe_name
    ))
}

#[cfg(not(windows))]
fn resolve_linux_ui_sfx_source_cached(
    app_handle: &AppHandle,
    name: &str,
) -> Result<PathBuf, String> {
    static CACHE: OnceLock<std::sync::Mutex<HashMap<String, PathBuf>>> = OnceLock::new();
    let safe_name = name.trim().to_string();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));

    if let Ok(cache) = cache.lock() {
        if let Some(path) = cache.get(&safe_name) {
            return Ok(path.clone());
        }
    }

    let resolved = resolve_linux_ui_sfx_source(app_handle, name)?;
    if let Ok(mut cache) = cache.lock() {
        cache.insert(safe_name, resolved.clone());
    }
    Ok(resolved)
}

#[cfg(not(windows))]
fn normalize_sfx_volume(volume: Option<f32>) -> f32 {
    volume.unwrap_or(0.7).clamp(0.0, 1.0)
}

#[cfg(not(windows))]
fn create_linux_ui_sfx_command(audio_path: &Path, volume: f32) -> Result<Command, String> {
    let Some(audio_sink) = preferred_linux_ui_audio_sink() else {
        return Err("no usable Linux ui audio sink is available".to_string());
    };
    let uri = Url::from_file_path(audio_path)
        .map_err(|_| format!("failed to build file URI for {}", audio_path.display()))?
        .to_string();
    let mut command = Command::new("gst-launch-1.0");
    command
        .arg("-q")
        .arg("uridecodebin")
        .arg(format!("uri={}", uri))
        .arg("!")
        .arg("audioconvert")
        .arg("!")
        .arg("audioresample")
        .arg("!")
        .arg("volume")
        .arg(format!("volume={:.3}", volume))
        .arg("!")
        .arg(audio_sink)
        .env("ALSOFT_DRIVERS", "pulse,null")
        .env("SDL_AUDIODRIVER", "pulseaudio")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(runtime_dir) = resolved_linux_runtime_dir() {
        command.env("XDG_RUNTIME_DIR", runtime_dir);
    }
    if audio_sink == "pulsesink" {
        if let Some(pulse_server) = resolved_linux_pulse_server() {
            command.env("PULSE_SERVER", pulse_server);
        }
    }
    if audio_sink == "pulsesink" {
        if let Some(pulse_server) = resolved_linux_pulse_server() {
            command.arg(format!("server={}", pulse_server));
        }
    }
    command.arg("sync=false");
    Ok(command)
}

#[cfg(not(windows))]
fn bootstrap_portable_wine_prefix(
    runtime: &ResolvedWineRuntime,
    apply_controller_fix: bool,
) -> Result<(), String> {
    let prefix = runtime
        .wine_prefix
        .as_ref()
        .ok_or_else(|| "portable Wine prefix could not be resolved".to_string())?;
    fs::create_dir_all(prefix)
        .map_err(|err| format!("failed to create {}: {}", prefix.display(), err))?;

    run_runtime_command(
        &runtime.wine_command,
        runtime,
        &["wineboot", "-u"],
        "wineboot",
    )?;
    run_runtime_command(
        Path::new("winetricks"),
        runtime,
        &["-q", "d3dcompiler_47", "dxvk", "vcrun2022"],
        "winetricks",
    )?;
    if apply_controller_fix {
        sync_wine_controller_overrides(runtime, true)?;
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinuxPrefixStatus {
    prefix_path: String,
    ready: bool,
    missing_tools: Vec<String>,
    error: String,
    audio_ready: bool,
    audio_missing: Vec<String>,
}

#[cfg(not(windows))]
fn kill_wineserver_with_env(runtime: &ResolvedWineRuntime) {
    let mut command = Command::new(&runtime.wineserver_command);
    runtime.apply_env(&mut command);
    let _ = command.arg("-k").status();
}

fn create_store_builder(
    app_handle: &AppHandle,
    store_path: PathBuf,
) -> tauri_plugin_store::Store<tauri::Wry> {
    StoreBuilder::new(app_handle.clone(), store_path).build()
}

fn initialize_config_store(
    store: &mut tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    store
        .insert(
            "config_version".to_string(),
            Value::String(CONFIG_VERSION.to_string()),
        )
        .map_err(|err| format!("unable to initialize config_version: {}", err))?;
    store
        .save()
        .map_err(|err| format!("unable to save initialized config store: {}", err))
}

fn reset_state_sync_store_defaults(state_sync: &mut TauriStateSync) {
    let remote_endpoints = config::get_default_endpoints();
    let current_endpoint = remote_endpoints
        .first()
        .cloned()
        .unwrap_or_else(Endpoint::default);

    state_sync.style = CLASSIC_STYLE;
    state_sync.locale = "en".to_string();
    state_sync.endpoints.clear();
    state_sync.remote_endpoints = remote_endpoints;
    state_sync.remote_endpoints_config.clear();
    state_sync.current_endpoint = current_endpoint;
    state_sync.user_manager = UserManager::default();
    state_sync.game_folder = None;
    state_sync.last_char_id = None;
    state_sync.serverlist_url = DEFAULT_SERVERLIST_URL.to_string();
    state_sync.messagelist_url = DEFAULT_MESSAGELIST_URL.to_string();
    state_sync.launcher_prefs = LauncherPrefs::default();
    state_sync.ui_prefs = UiPrefs::default();
}

fn load_state_sync_from_store(
    store: &mut tauri_plugin_store::Store<tauri::Wry>,
    state_sync: &mut TauriStateSync,
) -> Result<bool, String> {
    match store.load() {
        Ok(_) => {}
        Err(err) => {
            info!("unable to load config from disk: {}", err);
            return Ok(false);
        }
    }

    let _ = migrate_config_store(store);
    reset_state_sync_store_defaults(state_sync);

    store::get(store, "style", &mut state_sync.style);
    store::get(store, "locale", &mut state_sync.locale);
    store::get(store, "endpoints", &mut state_sync.endpoints);
    store::get(
        store,
        "remote_endpoints_config",
        &mut state_sync.remote_endpoints_config,
    );
    store::get(store, "current_endpoint", &mut state_sync.current_endpoint);
    store::get(store, "user_manager", &mut state_sync.user_manager);
    store::get(store, "game_folder", &mut state_sync.game_folder);
    store::get(store, "last_char_id", &mut state_sync.last_char_id);
    store::get(store, "serverlist_url", &mut state_sync.serverlist_url);
    store::get(store, "messagelist_url", &mut state_sync.messagelist_url);
    state_sync.serverlist_url =
        normalize_remote_url(&state_sync.serverlist_url).unwrap_or_default();
    state_sync.messagelist_url =
        normalize_remote_url(&state_sync.messagelist_url).unwrap_or_default();
    store::get(store, "launcher_prefs", &mut state_sync.launcher_prefs);
    state_sync.launcher_prefs.friend_signature =
        normalize_friend_signature(&state_sync.launcher_prefs.friend_signature);
    state_sync.launcher_prefs.wine_prefix_mode =
        normalize_wine_prefix_mode(&state_sync.launcher_prefs.wine_prefix_mode);
    state_sync.launcher_prefs.wine_prefix_custom_path = normalize_wine_prefix_custom_path(
        state_sync.launcher_prefs.wine_prefix_custom_path.as_deref(),
    );
    store::get(store, "ui_prefs", &mut state_sync.ui_prefs);
    state_sync.ui_prefs.classic_launcher_recent_resolutions =
        normalize_recent_resolution_list(&state_sync.ui_prefs.classic_launcher_recent_resolutions);
    state_sync.ui_prefs.ps4_launcher_recent_resolutions =
        normalize_recent_resolution_list(&state_sync.ui_prefs.ps4_launcher_recent_resolutions);

    let mut dirty = false;
    let normalized_style = normalize_launcher_style(state_sync.style);
    if state_sync.style != normalized_style {
        state_sync.style = normalized_style;
        store
            .insert("style".to_string(), Value::from(state_sync.style))
            .map_err(|err| format!("failed to normalize stored launcher style: {}", err))?;
        dirty = true;
    }

    state_sync.current_endpoint.version = resolve_effective_game_version(state_sync);
    state_sync
        .remote_endpoints
        .apply_config(&state_sync.remote_endpoints_config);

    if dirty {
        store
            .save()
            .map_err(|err| format!("failed to save normalized config store: {}", err))?;
    }

    Ok(true)
}

fn create_main_window(
    app_handle: &AppHandle,
    game_root: &Path,
    ui_prefs: &UiPrefs,
) -> Result<Window, String> {
    let webview_data_dir = webview_data_path(game_root);
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &webview_data_dir);
    #[cfg(windows)]
    let _ = ui_prefs;
    #[cfg(not(windows))]
    {
        if ui_prefs.linux_hardware_acceleration {
            std::env::remove_var("WEBKIT_DISABLE_COMPOSITING_MODE");
            std::env::remove_var("WEBKIT_DISABLE_DMABUF_RENDERER");
        } else {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    let config = load_main_window_config()?;
    WindowBuilder::from_config(app_handle, config)
        .data_directory(webview_data_dir)
        .build()
        .map_err(|err| format!("failed to build main window: {}", err))
}

fn runtime_error(message: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, message.into())
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LauncherPrefs {
    preload_controller_dlls: bool,
    #[serde(default = "default_friend_signature")]
    friend_signature: String,
    #[serde(default = "default_wine_prefix_mode")]
    wine_prefix_mode: String,
    #[serde(default)]
    wine_prefix_custom_path: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LauncherPrefUpdate {
    preload_controller_dlls: Option<bool>,
    friend_signature: Option<String>,
    wine_prefix_mode: Option<String>,
    wine_prefix_custom_path: Option<Option<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct UiPrefs {
    sfx_enabled: bool,
    sfx_volume: u8,
    font_preset: String,
    classic_launcher_width: u32,
    classic_launcher_height: u32,
    #[serde(default)]
    classic_launcher_custom_resolution: bool,
    #[serde(default)]
    classic_launcher_recent_resolutions: Vec<String>,
    ps4_launcher_width: u32,
    ps4_launcher_height: u32,
    #[serde(default)]
    ps4_launcher_custom_resolution: bool,
    #[serde(default)]
    ps4_launcher_recent_resolutions: Vec<String>,
    game_version: mhf_iel::MhfVersion,
    force_game_version: bool,
    dev_mode: bool,
    server_version_prompt_disabled: bool,
    launcher_controller: bool,
    linux_hardware_acceleration: bool,
    offline_images: bool,
}

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            sfx_enabled: false,
            sfx_volume: 30,
            font_preset: "default".to_string(),
            classic_launcher_width: CLASSIC_DEFAULT_WINDOW_WIDTH,
            classic_launcher_height: CLASSIC_DEFAULT_WINDOW_HEIGHT,
            classic_launcher_custom_resolution: false,
            classic_launcher_recent_resolutions: Vec::new(),
            ps4_launcher_width: PS4_DEFAULT_WINDOW_WIDTH,
            ps4_launcher_height: PS4_DEFAULT_WINDOW_HEIGHT,
            ps4_launcher_custom_resolution: false,
            ps4_launcher_recent_resolutions: Vec::new(),
            game_version: mhf_iel::MhfVersion::ZZ,
            force_game_version: false,
            dev_mode: false,
            server_version_prompt_disabled: false,
            launcher_controller: true,
            linux_hardware_acceleration: true,
            offline_images: false,
        }
    }
}

impl Default for LauncherPrefs {
    fn default() -> Self {
        Self {
            preload_controller_dlls: false,
            friend_signature: "none".to_string(),
            wine_prefix_mode: default_wine_prefix_mode(),
            wine_prefix_custom_path: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ControllerDllFileState {
    name: String,
    active_path: Option<String>,
    disabled_path: Option<String>,
    active: bool,
    disabled: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ControllerDllStateResponse {
    available: bool,
    files: Vec<ControllerDllFileState>,
}

#[cfg(windows)]
fn controller_dll_path_text(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(windows)]
fn find_case_insensitive_file(folder: &Path, target_name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(folder).ok()?;
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        if file_name.to_string_lossy().eq_ignore_ascii_case(target_name) {
            return Some(entry.path());
        }
    }
    None
}

#[cfg(windows)]
fn controller_dll_file_state(folder: &Path, name: &str) -> ControllerDllFileState {
    let disabled_name = format!("{name}.disabled");
    let active_path = find_case_insensitive_file(folder, name).unwrap_or_else(|| folder.join(name));
    let disabled_path = find_case_insensitive_file(folder, &disabled_name)
        .unwrap_or_else(|| folder.join(&disabled_name));
    let active = active_path.is_file();
    let disabled = disabled_path.is_file();

    ControllerDllFileState {
        name: name.to_string(),
        active_path: Some(controller_dll_path_text(&active_path)),
        disabled_path: Some(controller_dll_path_text(&disabled_path)),
        active,
        disabled,
    }
}

#[cfg(windows)]
fn collect_controller_dll_state(folder: &Path) -> ControllerDllStateResponse {
    let files: Vec<_> = CONTROLLER_DLL_FILE_NAMES
        .iter()
        .map(|name| controller_dll_file_state(folder, name))
        .collect();
    let available = files.iter().all(|file| file.active || file.disabled);

    ControllerDllStateResponse { available, files }
}

#[cfg(windows)]
fn rename_controller_dll_files(folder: &Path, enabled: bool) -> Result<(), String> {
    let state = collect_controller_dll_state(folder);
    if !state.available {
        return Ok(());
    }

    for file in state.files {
        let active_path = file
            .active_path
            .as_ref()
            .map(PathBuf::from)
            .ok_or_else(|| format!("missing active path for {}", file.name))?;
        let disabled_path = file
            .disabled_path
            .as_ref()
            .map(PathBuf::from)
            .ok_or_else(|| format!("missing disabled path for {}", file.name))?;

        if enabled {
            if file.disabled && !file.active {
                fs::rename(&disabled_path, &active_path).map_err(|err| {
                    format!(
                        "failed to enable {} from {} to {}: {}",
                        file.name,
                        disabled_path.display(),
                        active_path.display(),
                        err
                    )
                })?;
            }
        } else if file.active && !file.disabled {
            fs::rename(&active_path, &disabled_path).map_err(|err| {
                format!(
                    "failed to disable {} from {} to {}: {}",
                    file.name,
                    active_path.display(),
                    disabled_path.display(),
                    err
                )
            })?;
        }
    }

    Ok(())
}

#[tauri::command]
async fn sync_controller_dll_files(
    game_folder: Option<String>,
    enabled: bool,
    apply_changes: Option<bool>,
) -> Result<ControllerDllStateResponse, String> {
    #[cfg(not(windows))]
    {
        let _ = (game_folder, enabled, apply_changes);
        return Ok(ControllerDllStateResponse {
            available: true,
            files: Vec::new(),
        });
    }

    #[cfg(windows)]
    {
        let folder_value = game_folder.unwrap_or_default();
        let folder_text = folder_value.trim();
        if folder_text.is_empty() {
            return Ok(ControllerDllStateResponse {
                available: false,
                files: Vec::new(),
            });
        }

        let folder = PathBuf::from(folder_text);
        if !folder.is_dir() {
            return Ok(ControllerDllStateResponse {
                available: false,
                files: Vec::new(),
            });
        }

        let mut state = collect_controller_dll_state(&folder);
        if apply_changes.unwrap_or(false) && state.available {
            rename_controller_dll_files(&folder, enabled)?;
            state = collect_controller_dll_state(&folder);
        }

        Ok(state)
    }
}

impl UiPrefs {
    fn launcher_window_size_for_style(&self, style: u32) -> LauncherWindowSize {
        match style {
            CLASSIC_STYLE => LauncherWindowSize {
                width: normalize_launcher_window_dimension(
                    self.classic_launcher_width,
                    CLASSIC_DEFAULT_WINDOW_WIDTH,
                ),
                height: normalize_launcher_window_dimension(
                    self.classic_launcher_height,
                    CLASSIC_DEFAULT_WINDOW_HEIGHT,
                ),
            },
            PS4_STYLE => LauncherWindowSize {
                width: normalize_launcher_window_dimension(
                    self.ps4_launcher_width,
                    PS4_DEFAULT_WINDOW_WIDTH,
                ),
                height: normalize_launcher_window_dimension(
                    self.ps4_launcher_height,
                    PS4_DEFAULT_WINDOW_HEIGHT,
                ),
            },
            _ => default_launcher_window_size(style),
        }
    }
}

struct TauriState {
    client: reqwest::Client,
    state_sync: Arc<Mutex<TauriStateSync>>,
}

#[derive(Default)]
struct TauriStateSync {
    style: u32,
    locale: String,
    store: StoreHelper,
    endpoints: Vec<Endpoint>,
    remote_endpoints: Vec<Endpoint>,
    remote_endpoints_config: HashMap<String, EndpointConfig>,
    current_endpoint: Endpoint,
    launcher_ts: Option<SystemTime>,
    remote_messages: Vec<MessageData>,
    user_manager: UserManager,
    session_username: String,
    game_folder: Option<PathBuf>,
    last_char_id: Option<u32>,
    serverlist_url: String,
    messagelist_url: String,

    exit_reason: Option<ExitSignal>,
    skip_child_cleanup_once: bool,
    launcher_prefs: LauncherPrefs,
    ui_prefs: UiPrefs,

    auth_resp: Option<AuthResponse>,
    launcher_resp: Option<LauncherResponse>,
    patcher_resp: Option<PatcherResponse>,
    alt_client_stats: Option<altclient_stats::AltClientStats>,

    cancel_shared: CancellationToken,
    cancel_launcher: CancellationToken,
    cancel_serverlist: CancellationToken,
    cancel_messagelist: CancellationToken,
}

impl TauriStateSync {
    fn first_endpoint(&self) -> Option<&Endpoint> {
        self.remote_endpoints
            .first()
            .or_else(|| self.endpoints.first())
    }

    fn contains_endpoint(&self, endpoint: &Endpoint) -> bool {
        if self.current_endpoint.is_remote {
            self.remote_endpoints.contains(endpoint)
        } else {
            self.endpoints.contains(endpoint)
        }
    }

    fn ensure_current_endpoint(&mut self) -> Result<(), &'static str> {
        let endpoints = if self.current_endpoint.is_remote {
            &self.remote_endpoints
        } else {
            &self.endpoints
        };

        self.current_endpoint = endpoints
            .iter()
            .find(|&e| e == &self.current_endpoint)
            .or_else(|| self.first_endpoint())
            .ok_or("internal-error")?
            .clone();
        Ok(())
    }

    fn auth_resp_err(&self) -> Result<&AuthResponse, &str> {
        self.auth_resp.as_ref().ok_or("internal-error")
    }

    fn effective_folder(&self) -> PathBuf {
        self.current_endpoint
            .game_folder
            .as_ref()
            .or(self.game_folder.as_ref())
            .cloned()
            .unwrap_or_else(default_effective_folder)
    }
}

fn run_mhf(
    config: MhfConfig,
    game_root: PathBuf,
    launcher_prefs: LauncherPrefs,
    wait_for_exit: bool,
    app_handle: AppHandle,
) -> Result<isize, String> {
    #[cfg(windows)]
    {
        let _ = (game_root, launcher_prefs, wait_for_exit, app_handle);
        return mhf_iel::run(config).map_err(|e| format!("meze-butter error: {e}"));
    }

    #[cfg(not(windows))]
    {
        ensure_linux_path_defaults();
        ensure_mezeporta_support_dirs(&game_root)?;
        let helper_path = stage_meze_deps(&app_handle, &game_root)?;
        let runtime = resolve_wine_runtime(&game_root, &launcher_prefs)?;
        sync_wine_controller_overrides(&runtime, launcher_prefs.preload_controller_dlls)?;
        let payload = serde_json::to_vec(&config)
            .map_err(|e| format!("failed to serialize launch config: {e}"))?;
        let payload_b64 = base64::engine::general_purpose::STANDARD.encode(payload);

        let mut command = Command::new(&runtime.wine_command);
        runtime.apply_env(&mut command);
        runtime.append_runtime_prefix_args(&mut command);

        let mut child = command
            .arg(&helper_path)
            .arg("--stdin-b64")
            .stdin(Stdio::piped())
            .stdout(if wait_for_exit {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .stderr(if wait_for_exit {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .current_dir(config.mhf_folder.as_ref().unwrap_or(&game_root))
            .spawn()
            .map_err(|e| format!("failed to launch wine: {e}"))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(payload_b64.as_bytes())
                .map_err(|e| format!("failed to write config to helper stdin: {e}"))?;
        }
        drop(child.stdin.take());

        if !wait_for_exit {
            return Ok(0);
        }

        let status = child
            .wait()
            .map_err(|e| format!("failed to wait on helper: {e}"))?;
        kill_wineserver_with_env(&runtime);
        return Ok(status.code().unwrap_or(1) as isize);
    }
}

#[cfg(not(windows))]
fn build_launch_bundle(
    state_sync: &TauriStateSync,
    char_id: u32,
    char_new: bool,
) -> Result<(MhfConfig, PathBuf, LauncherPrefs), String> {
    let preload_controller_dlls = if cfg!(windows) {
        state_sync.launcher_prefs.preload_controller_dlls
    } else {
        false
    };
    let friend_signature = state_sync.launcher_prefs.friend_signature.clone();
    let launcher_prefs = state_sync.launcher_prefs.clone();
    let auth_resp = state_sync
        .auth_resp
        .as_ref()
        .ok_or_else(|| "missing auth response when preparing game launch".to_string())?;
    let char = auth_resp
        .characters
        .iter()
        .find(|c| c.id == char_id)
        .ok_or_else(|| format!("selected character {} not found in auth response", char_id))?;
    let char_ids = auth_resp.characters.iter().map(|c| c.id).collect();
    let notices = auth_resp
        .notices
        .iter()
        .map(|n| mhf_iel::Notice {
            flags: 0,
            data: n.clone(),
        })
        .collect();
    let (userdata, password) = state_sync.user_manager.get(&state_sync.current_endpoint);
    let mutex_selection = resolve_effective_mutex_versions(state_sync);
    let effective_version = resolve_effective_game_version(state_sync);
    let game_root = state_sync.effective_folder();

    let mut config = MhfConfig {
        char_id,
        char_name: char.name.clone(),
        char_gr: char.gr,
        char_hr: char.hr,
        char_ids,
        char_new,
        user_token_id: auth_resp.user.token_id,
        user_token: auth_resp.user.token.clone(),
        user_name: userdata.username,
        user_password: password,
        user_rights: auth_resp.user.rights,
        friends: auth_resp.friends.iter().map(Into::into).collect(),
        server_host: state_sync.current_endpoint.host(),
        server_port: state_sync.current_endpoint.game_port.unwrap_or(53310) as u32,
        entrance_count: auth_resp.entrance_count,
        current_ts: auth_resp.current_ts,
        expiry_ts: auth_resp.expiry_ts,
        notices,
        mez_event_id: 0,
        mez_start: 0,
        mez_end: 0,
        mez_solo_tickets: 0,
        mez_group_tickets: 0,
        mez_stalls: vec![],
        mhf_flags: None,
        version: effective_version,
        mutex_version: mutex_selection.primary,
        mutex_fallback_version: mutex_selection.fallback,
        preload_controller_dlls,
        friend_signature: Some(friend_signature),
        enable_font_registration: true,
        mhf_folder: state_sync
            .current_endpoint
            .game_folder
            .as_ref()
            .or_else(|| state_sync.game_folder.as_ref())
            .cloned(),
        font_path: state_sync
            .current_endpoint
            .game_folder
            .as_ref()
            .or_else(|| state_sync.game_folder.as_ref())
            .and_then(|root| resolve_launcher_font_path(root, effective_version)),
    };

    if let Some(mez_fes) = auth_resp.mez_fez.as_ref() {
        config.mez_event_id = mez_fes.id;
        config.mez_start = mez_fes.start;
        config.mez_end = mez_fes.end;
        config.mez_solo_tickets = mez_fes.solo_tickets;
        config.mez_group_tickets = mez_fes.group_tickets;
        config.mez_stalls = mez_fes
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

    Ok((config, game_root, launcher_prefs))
}

#[derive(Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointsPayload {
    endpoints: Option<Vec<Endpoint>>,
    remote_endpoints: Option<Vec<Endpoint>>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthPayload {
    response: AuthResponse,
    has_patch: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AltClientSavedataResponse {
    character_id: u32,
    savedata: String,
    #[serde(default)]
    client_mode: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AltCharacterSavedataCacheResponse {
    savedata: String,
    gsv: String,
}
#[derive(Serialize, Clone)]
pub struct LogPayload {
    level: String,
    message: String,
}

impl LogPayload {
    fn error(message: impl Into<String>) -> Self {
        Self {
            level: "error".into(),
            message: message.into(),
        }
    }

    fn warning(message: impl Into<String>) -> Self {
        Self {
            level: "warning".into(),
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitialDataPayload {
    style: u32,
    locale: String,
    endpoints: Vec<Endpoint>,
    remote_endpoints: Vec<Endpoint>,
    current_endpoint: Endpoint,
    remote_messages: Vec<MessageData>,
    username: String,
    password: String,
    remember_me: bool,
    game_folder: Option<PathBuf>,
    current_folder: PathBuf,
    last_char_id: Option<u32>,
    serverlist_url: String,
    messagelist_url: String,
    settings: Settings,
    launcher_prefs: LauncherPrefs,
    ui_prefs: UiPrefs,
    background: Option<String>,
    cog: Option<String>,
    capcom: Option<String>,
    button: Option<String>,
    headers: Option<LauncherHeaders>,
    dialog: Option<String>,
    #[serde(rename = "server_patch")]
    server_patch: Option<String>,
    ps4: Option<LauncherPs4Assets>,
    launcher_tag: Option<String>,
    footer_tag: Option<String>,
    tag: Option<String>,
    server_tag: Option<String>,
    detected_game_version: String,
}

#[tauri::command]
async fn initial_data(state: tauri::State<'_, TauriState>) -> Result<InitialDataPayload, ()> {
    let state_sync = state.state_sync.lock().await;
    let (userdata, password) = state_sync.user_manager.get(&state_sync.current_endpoint);
    let launcher_prefs = state_sync.launcher_prefs.clone();
    let launcher_resp = state_sync.launcher_resp.clone();
    Ok(InitialDataPayload {
        style: state_sync.style,
        endpoints: state_sync.endpoints.clone(),
        remote_endpoints: state_sync.remote_endpoints.clone(),
        current_endpoint: state_sync.current_endpoint.clone(),
        remote_messages: state_sync.remote_messages.clone(),
        username: userdata.username,
        password,
        remember_me: userdata.remember_me,
        game_folder: state_sync.game_folder.clone(),
        current_folder: state_sync.effective_folder(),
        locale: state_sync.locale.clone(),
        last_char_id: state_sync.last_char_id,
        serverlist_url: state_sync.serverlist_url.clone(),
        messagelist_url: state_sync.messagelist_url.clone(),
        settings: settings::get_settings(&state_sync.effective_folder()),
        launcher_prefs,
        ui_prefs: state_sync.ui_prefs.clone(),
        background: launcher_resp
            .as_ref()
            .and_then(|resp| resp.background.clone()),
        cog: launcher_resp.as_ref().and_then(|resp| resp.cog.clone()),
        capcom: launcher_resp.as_ref().and_then(|resp| resp.capcom.clone()),
        button: launcher_resp.as_ref().and_then(|resp| resp.button.clone()),
        headers: launcher_resp.as_ref().and_then(|resp| resp.headers.clone()),
        dialog: launcher_resp.as_ref().and_then(|resp| resp.dialog.clone()),
        server_patch: launcher_resp
            .as_ref()
            .and_then(|resp| resp.server_patch.clone()),
        ps4: launcher_resp.as_ref().and_then(|resp| resp.ps4.clone()),
        launcher_tag: launcher_resp
            .as_ref()
            .and_then(|resp| resp.launcher_tag.clone()),
        footer_tag: launcher_resp
            .as_ref()
            .and_then(|resp| resp.footer_tag.clone()),
        tag: launcher_resp.as_ref().and_then(|resp| resp.tag.clone()),
        server_tag: launcher_resp.as_ref().and_then(|resp| resp.server_tag.clone()),
        detected_game_version: version_to_label(resolve_effective_game_version(&state_sync))
            .to_string(),
    })
}

#[tauri::command]
async fn set_style(
    mut window: Window,
    state: tauri::State<'_, TauriState>,
    style: u32,
) -> Result<(), String> {
    let mut state_sync = state.state_sync.lock().await;
    let style = normalize_launcher_style(style);
    state_sync.style = style;
    state_sync.store.with(|s| s.set("style", style));
    handle_style(&mut window, style, &state_sync.ui_prefs);
    Ok(())
}

#[tauri::command]
async fn set_locale(state: tauri::State<'_, TauriState>, locale: String) -> Result<(), String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.locale = locale.clone();
    state_sync.store.with(|s| s.set("locale", locale));
    Ok(())
}

#[tauri::command]
async fn set_launcher_pref(
    state: tauri::State<'_, TauriState>,
    prefs: Option<LauncherPrefUpdate>,
    preload_controller_dlls: Option<bool>,
) -> Result<(), String> {
    let mut payload = prefs.unwrap_or_default();
    if payload.preload_controller_dlls.is_none() {
        payload.preload_controller_dlls = preload_controller_dlls;
    }

    let mut state_sync = state.state_sync.lock().await;
    if let Some(preload_controller_dlls) = payload.preload_controller_dlls {
        state_sync.launcher_prefs.preload_controller_dlls = preload_controller_dlls;
    }
    if let Some(friend_signature) = payload.friend_signature {
        state_sync.launcher_prefs.friend_signature = normalize_friend_signature(&friend_signature);
    }
    if let Some(wine_prefix_mode) = payload.wine_prefix_mode {
        state_sync.launcher_prefs.wine_prefix_mode = normalize_wine_prefix_mode(&wine_prefix_mode);
    }
    if let Some(wine_prefix_custom_path) = payload.wine_prefix_custom_path {
        state_sync.launcher_prefs.wine_prefix_custom_path =
            normalize_wine_prefix_custom_path(wine_prefix_custom_path.as_deref());
    }
    let launcher_prefs = state_sync.launcher_prefs.clone();
    state_sync
        .store
        .with(|s| s.set("launcher_prefs", launcher_prefs));
    Ok(())
}

#[tauri::command]
async fn set_setting(
    state: tauri::State<'_, TauriState>,
    setting: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let state_sync = state.state_sync.lock().await;
    settings::set_setting(&state_sync.effective_folder(), &setting, value)
}

#[tauri::command]
async fn set_ui_pref(
    state: tauri::State<'_, TauriState>,
    name: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let mut state_sync = state.state_sync.lock().await;
    match (name.as_str(), value) {
        ("sfxEnabled", serde_json::Value::Bool(v)) => state_sync.ui_prefs.sfx_enabled = v,
        ("sfxVolume", serde_json::Value::Number(n)) => {
            if let Some(v) = n.as_u64() {
                state_sync.ui_prefs.sfx_volume = v.min(u8::MAX as u64) as u8;
            }
        }
        ("fontPreset", serde_json::Value::String(v)) => {
            if (v == "default" || v == "classic")
                || (v.starts_with("custom:") && !v.contains('/') && !v.contains('\\'))
            {
                state_sync.ui_prefs.font_preset = v;
            }
        }
        ("classicLauncherWidth", serde_json::Value::Number(n)) => {
            if let Some(v) = n.as_u64() {
                state_sync.ui_prefs.classic_launcher_width =
                    normalize_launcher_window_dimension(v as u32, CLASSIC_DEFAULT_WINDOW_WIDTH);
            }
        }
        ("classicLauncherHeight", serde_json::Value::Number(n)) => {
            if let Some(v) = n.as_u64() {
                state_sync.ui_prefs.classic_launcher_height =
                    normalize_launcher_window_dimension(v as u32, CLASSIC_DEFAULT_WINDOW_HEIGHT);
            }
        }
        ("classicLauncherRecentResolutions", serde_json::Value::Array(entries)) => {
            let values = entries
                .iter()
                .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>();
            state_sync.ui_prefs.classic_launcher_recent_resolutions =
                normalize_recent_resolution_list(&values);
        }
        ("classicLauncherCustomResolution", serde_json::Value::Bool(v)) => {
            state_sync.ui_prefs.classic_launcher_custom_resolution = v;
        }
        ("ps4LauncherWidth", serde_json::Value::Number(n)) => {
            if let Some(v) = n.as_u64() {
                state_sync.ui_prefs.ps4_launcher_width =
                    normalize_launcher_window_dimension(v as u32, PS4_DEFAULT_WINDOW_WIDTH);
            }
        }
        ("ps4LauncherHeight", serde_json::Value::Number(n)) => {
            if let Some(v) = n.as_u64() {
                state_sync.ui_prefs.ps4_launcher_height =
                    normalize_launcher_window_dimension(v as u32, PS4_DEFAULT_WINDOW_HEIGHT);
            }
        }
        ("ps4LauncherRecentResolutions", serde_json::Value::Array(entries)) => {
            let values = entries
                .iter()
                .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>();
            state_sync.ui_prefs.ps4_launcher_recent_resolutions =
                normalize_recent_resolution_list(&values);
        }
        ("ps4LauncherCustomResolution", serde_json::Value::Bool(v)) => {
            state_sync.ui_prefs.ps4_launcher_custom_resolution = v;
        }
        ("gameVersion", serde_json::Value::String(v)) => {
            if let Some(version) = label_to_version(&v) {
                state_sync.ui_prefs.game_version = version;
                state_sync.current_endpoint.version = version;
            }
        }
        ("forceGameVersion", serde_json::Value::Bool(v)) => {
            state_sync.ui_prefs.force_game_version = v;
        }
        ("devMode", serde_json::Value::Bool(v)) => {
            state_sync.ui_prefs.dev_mode = v;
        }
        ("serverVersionPromptDisabled", serde_json::Value::Bool(v)) => {
            state_sync.ui_prefs.server_version_prompt_disabled = v;
        }
        ("launcherController", serde_json::Value::Bool(v)) => {
            state_sync.ui_prefs.launcher_controller = v;
        }
        ("linuxHardwareAcceleration", serde_json::Value::Bool(v)) => {
            state_sync.ui_prefs.linux_hardware_acceleration = v;
        }
        ("offlineImages", serde_json::Value::Bool(v)) => {
            state_sync.ui_prefs.offline_images = v;
        }
        _ => return Ok(()),
    }
    let ui_prefs = state_sync.ui_prefs.clone();
    let current_endpoint = state_sync.current_endpoint.clone();
    state_sync.store.with(|s| {
        s.set("ui_prefs", ui_prefs);
        s.set("current_endpoint", current_endpoint);
    });
    Ok(())
}

#[tauri::command]
async fn get_linux_prefix_status(
    state: tauri::State<'_, TauriState>,
) -> Result<LinuxPrefixStatus, String> {
    #[cfg(not(windows))]
    {
        ensure_linux_path_defaults();
        let state_sync = state.state_sync.lock().await;
        let game_root = state_sync.effective_folder();
        let launcher_prefs = state_sync.launcher_prefs.clone();
        drop(state_sync);

        let audio_missing = find_missing_linux_audio_runtime();
        let mut error = String::new();
        let mut ready = false;
        let mut missing_tools = Vec::new();
        let mut prefix_path = game_root.join(WINE_PREFIX_DIR);

        match resolve_wine_runtime(&game_root, &launcher_prefs) {
            Ok(runtime) => {
                if let Some(runtime_prefix) = runtime.wine_prefix.clone() {
                    prefix_path = runtime_prefix;
                }
                ready = wine_prefix_initialized(&prefix_path);
                missing_tools = find_missing_wine_tools(&runtime);
            }
            Err(err) => {
                error = err;
            }
        }

        return Ok(LinuxPrefixStatus {
            prefix_path: prefix_path.display().to_string(),
            ready,
            missing_tools,
            error,
            audio_ready: audio_missing.is_empty(),
            audio_missing,
        });
    }

    #[cfg(windows)]
    {
        let _ = state;
        Ok(LinuxPrefixStatus {
            prefix_path: String::new(),
            ready: false,
            missing_tools: Vec::new(),
            error: String::new(),
            audio_ready: true,
            audio_missing: Vec::new(),
        })
    }
}

#[tauri::command]
async fn install_linux_portable_prefix(
    state: tauri::State<'_, TauriState>,
) -> Result<LinuxPrefixStatus, String> {
    #[cfg(not(windows))]
    {
        ensure_linux_path_defaults();
        let (game_root, apply_controller_fix) = {
            let state_sync = state.state_sync.lock().await;
            (
                state_sync.effective_folder(),
                state_sync.launcher_prefs.preload_controller_dlls,
            )
        };
        ensure_mezeporta_support_dirs(&game_root)?;

        let launcher_prefs = LauncherPrefs {
            wine_prefix_mode: WINE_PREFIX_MODE_PORTABLE.to_string(),
            ..LauncherPrefs::default()
        };
        let runtime = resolve_wine_runtime(&game_root, &launcher_prefs)?;
        let missing_tools = find_missing_wine_tools(&runtime);
        if !missing_tools.is_empty() {
            return Err(format!(
                "linux-prefix-missing-tools:{}",
                missing_tools.join(",")
            ));
        }

        let result = bootstrap_portable_wine_prefix(&runtime, apply_controller_fix);
        kill_wineserver_with_env(&runtime);
        result?;

        let prefix_path = game_root.join(WINE_PREFIX_DIR);
        let audio_missing = find_missing_linux_audio_runtime();
        return Ok(LinuxPrefixStatus {
            prefix_path: prefix_path.display().to_string(),
            ready: wine_prefix_initialized(&prefix_path),
            missing_tools: Vec::new(),
            error: String::new(),
            audio_ready: audio_missing.is_empty(),
            audio_missing,
        });
    }

    #[cfg(windows)]
    {
        let _ = state;
        Err("linux-only-command".to_string())
    }
}

#[tauri::command]
async fn play_linux_ui_sfx(
    app_handle: AppHandle,
    name: String,
    wait_for_end: Option<bool>,
    _volume: Option<f32>,
) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        ensure_linux_path_defaults();
        let missing = cached_missing_linux_audio_runtime();
        if !missing.is_empty() {
            return Err(format!(
                "linux ui audio runtime is missing: {}",
                missing.join(", ")
            ));
        }

        let audio_path = resolve_linux_ui_sfx_source_cached(&app_handle, &name)?;
        let volume = normalize_sfx_volume(_volume);
        let mut command = create_linux_ui_sfx_command(&audio_path, volume)?;
        if wait_for_end.unwrap_or(false) {
            let output = command
                .output()
                .map_err(|err| format!("failed to play Linux ui sfx '{}': {}", name, err))?;
            if output.status.success() {
                return Ok(());
            }
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(format!(
                "Linux ui sfx '{}' exited with status {}{}",
                name,
                output.status,
                if stderr.is_empty() {
                    String::new()
                } else {
                    format!(": {}", stderr)
                }
            ));
        }

        command.stderr(Stdio::null());
        command
            .spawn()
            .map_err(|err| format!("failed to start Linux ui sfx '{}': {}", name, err))?;
        return Ok(());
    }

    #[cfg(windows)]
    {
        let _ = (app_handle, name, wait_for_end);
        Ok(())
    }
}

type GameVersionDetection = String;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ServerVersionInfoPayload {
    name: String,
    client_mode: String,
}

fn version_payload_field(payload: &Value, keys: &[&str]) -> Option<String> {
    let object = payload.as_object()?;
    for key in keys {
        let value = object.get(*key)?;
        let text = match value {
            Value::String(v) => v.trim().to_string(),
            Value::Number(v) => v.to_string(),
            Value::Bool(v) => v.to_string(),
            _ => continue,
        };
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}

fn parse_server_version_payload(payload: &Value) -> Option<ServerVersionInfoPayload> {
    let client_mode = version_payload_field(
        payload,
        &[
            "clientMode",
            "client_mode",
            "clientModeId",
            "clientModeID",
            "client_mode_id",
            "ClientModeId",
            "ClientModeID",
            "version",
            "Version",
        ],
    )?;

    let name = version_payload_field(payload, &["name", "Name"]).unwrap_or_default();

    Some(ServerVersionInfoPayload { name, client_mode })
}

async fn request_server_version_info(
    client: &reqwest::Client,
    endpoint: &Endpoint,
    path: &str,
) -> Option<ServerVersionInfoPayload> {
    let response = client
        .get(endpoint.get_url(path))
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let payload: Value = response.json().await.ok()?;
    parse_server_version_payload(&payload)
}

#[tauri::command]
async fn detect_game_version(
    state: tauri::State<'_, TauriState>,
    game_folder: Option<PathBuf>,
) -> Result<GameVersionDetection, String> {
    let state_sync = state.state_sync.lock().await;
    let folder = game_folder.unwrap_or_else(|| state_sync.effective_folder());
    let detected =
        detect_game_version_from_folder(&folder).unwrap_or(state_sync.ui_prefs.game_version);
    Ok(version_to_label(detected).to_string())
}

#[tauri::command]
async fn get_server_version_info(
    state: tauri::State<'_, TauriState>,
    endpoint: Endpoint,
) -> Result<Option<ServerVersionInfoPayload>, String> {
    if endpoint.url.trim().is_empty() || endpoint.url == "OFFLINEMODE" {
        return Ok(None);
    }

    for path in ["/v2/version", "/version"] {
        if let Some(payload) = request_server_version_info(&state.client, &endpoint, path).await {
            return Ok(Some(payload));
        }
    }

    Ok(None)
}
#[tauri::command]
async fn set_endpoints(
    state: tauri::State<'_, TauriState>,
    endpoints: Vec<Endpoint>,
) -> Result<Endpoint, String> {
    endpoints.check_valid()?;
    let mut state_sync = state.state_sync.lock().await;
    state_sync.endpoints = endpoints;
    if !state_sync.current_endpoint.is_remote {
        state_sync.ensure_current_endpoint()?;
    }
    let endpoints = state_sync.endpoints.clone();
    let current_endpoint = state_sync.current_endpoint.clone();
    state_sync.store.with(|s| {
        s.set("endpoints", endpoints);
        s.set("current_endpoint", current_endpoint);
    });
    Ok(state_sync.current_endpoint.clone())
}

#[tauri::command]
async fn set_remote_endpoints(
    state: tauri::State<'_, TauriState>,
    endpoints: Vec<Endpoint>,
) -> Result<Endpoint, String> {
    endpoints.check_valid()?;
    let state_sync = &mut *state.state_sync.lock().await;
    state_sync.remote_endpoints = endpoints;
    if state_sync.current_endpoint.is_remote {
        state_sync.ensure_current_endpoint()?;
    }
    state_sync
        .remote_endpoints
        .update_config(&mut state_sync.remote_endpoints_config);
    let current_endpoint = state_sync.current_endpoint.clone();
    let remote_endpoints_config = state_sync.remote_endpoints_config.clone();
    state_sync.store.with(|s| {
        s.set("remote_endpoints_config", remote_endpoints_config);
        s.set("current_endpoint", current_endpoint);
    });
    Ok(state_sync.current_endpoint.clone())
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UserDataPayload {
    userdata: UserData,
    password: String,
}

fn launcher_request_for_endpoint(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
) -> JsonRequest<LauncherResponse> {
    server::launcher_request(client, cancel, endpoint)
}

fn login_request_for_endpoint(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    username: &str,
    password: &str,
) -> JsonRequest<AuthResponse> {
    server::login_request(client, cancel, endpoint, username, password)
}

fn register_request_for_endpoint(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    username: &str,
    password: &str,
) -> JsonRequest<AuthResponse> {
    server::register_request(client, cancel, endpoint, username, password)
}

fn create_character_request_for_endpoint(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
) -> JsonRequest<server::CharacterData> {
    server::create_character_request(client, cancel, endpoint, token)
}

fn delete_character_request_for_endpoint(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
    character_id: i32,
) -> JsonRequest<server::EmptyResponse> {
    server::delete_character_request(client, cancel, endpoint, token, character_id)
}

fn export_character_request_for_endpoint(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
    character_id: i32,
) -> JsonRequest<Value> {
    server::export_save_request(client, cancel, endpoint, token, character_id)
}

#[derive(Clone)]
struct AltCharacterCacheIdentity {
    server_name: String,
    version_label: String,
    username: String,
    character_name: String,
}

fn sanitize_cache_path_segment(value: &str, fallback: &str) -> String {
    let mut sanitized: String = value
        .trim()
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            _ => c,
        })
        .collect();

    while sanitized.ends_with(' ') || sanitized.ends_with('.') {
        sanitized.pop();
    }

    if sanitized.is_empty() {
        sanitized = fallback.to_string();
    }

    let upper = sanitized.to_ascii_uppercase();
    const WINDOWS_RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if WINDOWS_RESERVED.contains(&upper.as_str()) {
        sanitized = format!("_{}", sanitized);
    }

    sanitized
}

fn resolve_alt_character_cache_identity(
    state_sync: &TauriStateSync,
    character_id: u32,
) -> Result<AltCharacterCacheIdentity, String> {
    let auth = state_sync
        .auth_resp
        .as_ref()
        .ok_or_else(|| "internal-error".to_string())?;

    let character = auth
        .characters
        .iter()
        .find(|entry| entry.id == character_id)
        .ok_or_else(|| "internal-error".to_string())?;

    let mut username = state_sync.session_username.trim().to_string();
    if username.is_empty() {
        let (user_data, _) = state_sync.user_manager.get(&state_sync.current_endpoint);
        username = user_data.username.trim().to_string();
    }
    if username.is_empty() {
        username = format!("User{}", auth.user.token_id);
    }

    let character_name = if !character.name.trim().is_empty() {
        character.name.trim().to_string()
    } else {
        format!("Character{}", character_id)
    };

    let server_name = {
        let server_key = state_sync.current_endpoint.server_key();
        if server_key.trim().is_empty() {
            "UnknownServer".to_string()
        } else {
            server_key
        }
    };

    let version_label = version_to_label(state_sync.current_endpoint.version).to_string();

    Ok(AltCharacterCacheIdentity {
        server_name,
        version_label,
        username,
        character_name,
    })
}

fn alt_character_user_cache_dir(game_root: &Path, identity: &AltCharacterCacheIdentity) -> PathBuf {
    game_root
        .join(ALT_CHARACTER_CACHE_ROOT)
        .join(sanitize_cache_path_segment(
            &identity.server_name,
            "UnknownServer",
        ))
        .join(sanitize_cache_path_segment(
            &identity.version_label,
            "UnknownVersion",
        ))
        .join("Users")
        .join(sanitize_cache_path_segment(
            &identity.username,
            "UnknownUser",
        ))
        .join(sanitize_cache_path_segment(
            &identity.character_name,
            "UnknownCharacter",
        ))
}

fn alt_character_cache_path(game_root: &Path, identity: &AltCharacterCacheIdentity) -> PathBuf {
    alt_character_user_cache_dir(game_root, identity).join(ALT_CHARACTER_CACHE_FILE)
}

fn alt_character_version_path(game_root: &Path, identity: &AltCharacterCacheIdentity) -> PathBuf {
    alt_character_user_cache_dir(game_root, identity).join(ALT_CHARACTER_VERSION_FILE)
}

fn normalize_savedata_version_token(value: Option<&str>) -> Option<String> {
    let token = value?.trim();
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

fn savedata_token_to_cache_version_label(token: &str) -> String {
    let normalized = token.trim();
    if normalized.eq_ignore_ascii_case("Z2.2") {
        "Z2T".to_string()
    } else {
        normalized.to_string()
    }
}

fn identity_with_savedata_version(
    mut identity: AltCharacterCacheIdentity,
    savedata_version: Option<&str>,
) -> AltCharacterCacheIdentity {
    if let Some(version_token) = normalize_savedata_version_token(savedata_version) {
        identity.version_label = savedata_token_to_cache_version_label(&version_token);
    }
    identity
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AltCharacterSavedataVersionPayload {
    client_mode: String,
}

fn ensure_alt_character_savedata_version(
    game_root: &Path,
    identity: &AltCharacterCacheIdentity,
    savedata_version: Option<&str>,
) {
    if let Ok(existing_token) = read_alt_character_savedata_version(game_root, identity) {
        if !existing_token.trim().is_empty() {
            return;
        }
    }

    let Some(version_token) = normalize_savedata_version_token(savedata_version) else {
        return;
    };

    let version_path = alt_character_version_path(game_root, identity);

    if let Some(parent) = version_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            warn!(
                "failed to create alt character version dir {}: {}",
                parent.display(),
                e
            );
            return;
        }
    }

    let payload = AltCharacterSavedataVersionPayload {
        client_mode: version_token.clone(),
    };

    let serialized = serde_json::to_string_pretty(&payload)
        .unwrap_or_else(|_| format!("{{\"clientMode\":\"{}\"}}", version_token));

    if let Err(e) = fs::write(&version_path, format!("{}\n", serialized)) {
        warn!(
            "failed to write alt character version file {}: {}",
            version_path.display(),
            e
        );
    }
}

fn read_alt_character_savedata_version(
    game_root: &Path,
    identity: &AltCharacterCacheIdentity,
) -> Result<String, String> {
    let version_path = alt_character_version_path(game_root, identity);
    let raw = fs::read_to_string(&version_path).map_err(|_| "file-error".to_string())?;

    if let Ok(payload) = serde_json::from_str::<AltCharacterSavedataVersionPayload>(&raw) {
        if let Some(token) = normalize_savedata_version_token(Some(payload.client_mode.as_str())) {
            return Ok(token);
        }
    }

    normalize_savedata_version_token(Some(raw.as_str())).ok_or_else(|| "file-error".to_string())
}

fn clear_alt_character_cache_for_character(game_root: &Path, identity: &AltCharacterCacheIdentity) {
    let cache_path = alt_character_cache_path(game_root, identity);
    if let Err(e) = fs::remove_file(&cache_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            warn!(
                "failed to remove alt character cache file {}: {}",
                cache_path.display(),
                e
            );
        }
    }

    if let Some(parent) = cache_path.parent() {
        if let Err(e) = fs::remove_dir(parent) {
            let kind = e.kind();
            if kind != std::io::ErrorKind::NotFound && kind != std::io::ErrorKind::DirectoryNotEmpty
            {
                warn!(
                    "failed to remove alt character cache dir {}: {}",
                    parent.display(),
                    e
                );
            }
        }
    }
}

async fn fetch_alt_character_savedata_for_endpoint(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
    character_id: u32,
) -> Result<(Vec<u8>, Option<String>), String> {
    let request = client
        .get(endpoint.get_url(&format!(
            "/v2/altclient/characters/{}/savedata",
            character_id
        )))
        .bearer_auth(token);
    let response = server::JsonRequest::<AltClientSavedataResponse>::new(request, cancel)
        .send()
        .await
        .map_err(|e| e.into_frontend())?;

    if response.character_id != 0 && response.character_id != character_id {
        warn!(
            "savedata response character mismatch: requested {}, got {}",
            character_id, response.character_id
        );
    }

    let encoded = response.savedata.trim();
    if encoded.is_empty() {
        return Err("internal-error".into());
    }

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .map_err(|e| {
            warn!(
                "failed to decode alt character savedata for {}: {}",
                character_id, e
            );
            "internal-error".to_string()
        })?;

    Ok((
        decoded,
        normalize_savedata_version_token(Some(response.client_mode.as_str())),
    ))
}

#[tauri::command]
async fn set_current_endpoint(
    window: Window,
    state: tauri::State<'_, TauriState>,
    current_endpoint: Endpoint,
) -> Result<LauncherResponse, String> {
    let req = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_shared.cancel();
        state_sync.cancel_launcher.cancel();
        state_sync.cancel_launcher = CancellationToken::new();

        // lightweight cache TTL (e.g. 5 s):
        let stale = state_sync
            .launcher_ts
            .map(|t| {
                SystemTime::now()
                    .duration_since(t)
                    .map(|d| d.as_secs() > 5)
                    .unwrap_or(true)
            })
            .unwrap_or(true);

        if state_sync.current_endpoint == current_endpoint && !stale {
            if let Some(launcher_resp) = &state_sync.launcher_resp {
                return Ok(launcher_resp.clone());
            }
        }
        state_sync.launcher_resp = None;
        state_sync.current_endpoint = current_endpoint.clone();
        state_sync.current_endpoint.version = resolve_effective_game_version(&state_sync);
        let (userdata, password) = state_sync.user_manager.get(&state_sync.current_endpoint);
        state_sync.session_username = userdata.username.clone();
        window
            .emit("userdata", UserDataPayload { userdata, password })
            .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
        if !state_sync.contains_endpoint(&current_endpoint) {
            let payload = if current_endpoint.is_remote {
                state_sync
                    .remote_endpoints
                    .insert(0, current_endpoint.clone());
                EndpointsPayload {
                    remote_endpoints: Some(state_sync.remote_endpoints.clone()),
                    ..Default::default()
                }
            } else {
                state_sync.endpoints.insert(0, current_endpoint.clone());
                let endpoints = state_sync.endpoints.clone();
                state_sync.store.with(|s| s.set("endpoints", endpoints));
                EndpointsPayload {
                    endpoints: Some(state_sync.endpoints.clone()),
                    ..Default::default()
                }
            };
            window
                .emit("endpoints", payload)
                .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
        }
        state_sync
            .store
            .with(|s| s.set("current_endpoint", current_endpoint.clone()));
        launcher_request_for_endpoint(
            &state.client,
            state_sync.cancel_launcher.clone(),
            &state_sync.current_endpoint,
        )
    };
    let launcher_resp = req.send().await.map_err(|e| e.into_frontend())?;
    let mut state_sync = state.state_sync.lock().await;
    state_sync.launcher_resp = Some(launcher_resp.clone());
    state_sync.launcher_ts = Some(SystemTime::now());
    Ok(launcher_resp)
}

#[tauri::command]
async fn set_game_folder(
    _app_handle: AppHandle,
    state: tauri::State<'_, TauriState>,
    game_folder: Option<String>,
) -> Result<(), String> {
    let mut state_sync = state.state_sync.lock().await;
    let game_folder = game_folder.map(PathBuf::from);
    if let Some(f) = game_folder.as_ref() {
        if !f.is_dir() {
            return Err("path-folder-error".into());
        } else if !f.exists() {
            return Err("path-exists-error".into());
        }
        ensure_mezeporta_support_dirs(f)?;
    }
    state_sync.game_folder = game_folder.clone();
    state_sync.store.with(|s| s.set("game_folder", game_folder));
    Ok(())
}

#[tauri::command]
async fn set_serverlist_url(
    window: Window,
    state: tauri::State<'_, TauriState>,
    serverlist_url: String,
) -> Result<(), String> {
    let normalized = normalize_remote_url(&serverlist_url).unwrap_or_default();
    if normalized.is_empty() {
        let state_sync = &mut *state.state_sync.lock().await;
        state_sync.remote_endpoints = config::get_default_endpoints();
        if state_sync.current_endpoint.is_remote
            && !state_sync
                .remote_endpoints
                .contains(&state_sync.current_endpoint)
        {
            let current_endpoint = state_sync.current_endpoint.clone();
            state_sync.remote_endpoints.push(current_endpoint);
        }
        state_sync
            .remote_endpoints
            .apply_config(&state_sync.remote_endpoints_config);
        let payload = EndpointsPayload {
            remote_endpoints: Some(state_sync.remote_endpoints.clone()),
            ..Default::default()
        };
        window
            .emit("endpoints", payload)
            .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
    } else {
        let req = {
            let mut state_sync = state.state_sync.lock().await;
            if normalized == state_sync.serverlist_url {
                return Ok(());
            }
            state_sync.cancel_serverlist.cancel();
            state_sync.cancel_serverlist = CancellationToken::new();
            server::simple_request(
                &state.client,
                state_sync.cancel_serverlist.clone(),
                &normalized,
            )
        };
        handle_remote_endpoints(&window, req, state.state_sync.clone()).await;
    }
    let mut state_sync = state.state_sync.lock().await;
    state_sync.serverlist_url = normalized.clone();
    state_sync
        .store
        .with(|s| s.set("serverlist_url", normalized));
    Ok(())
}

#[tauri::command]
async fn set_messagelist_url(
    window: Window,
    state: tauri::State<'_, TauriState>,
    messagelist_url: String,
) -> Result<(), String> {
    let normalized = normalize_remote_url(&messagelist_url).unwrap_or_default();
    if !normalized.is_empty() {
        let req = {
            let mut state_sync = state.state_sync.lock().await;
            if normalized == state_sync.messagelist_url {
                return Ok(());
            }
            state_sync.messagelist_url = normalized.clone();
            state_sync.cancel_messagelist.cancel();
            state_sync.cancel_messagelist = CancellationToken::new();
            server::simple_request(
                &state.client,
                state_sync.cancel_messagelist.clone(),
                &normalized,
            )
        };
        let state_sync_mutex = state.state_sync.clone();
        handle_remote_messages(&window, req, state_sync_mutex).await;
    }
    let mut state_sync = state.state_sync.lock().await;
    state_sync.messagelist_url = normalized.clone();
    state_sync
        .store
        .with(|s| s.set("messagelist_url", normalized));
    Ok(())
}

#[tauri::command]
async fn auth(
    state: tauri::State<'_, TauriState>,
    username: String,
    password: String,
    remember_me: bool,
    auth_req: JsonRequest<AuthResponse>,
) -> Result<AuthPayload, String> {
    // -- 1) perform login -----------------------------------------------------
    let mut auth_resp = auth_req.send().await.map_err(|e| e.into_frontend())?;

    let (game_root, is_offline) = {
        let state_sync = state.state_sync.lock().await;
        (
            state_sync.effective_folder(),
            state_sync
                .current_endpoint
                .url
                .eq_ignore_ascii_case("OFFLINEMODE")
                || state_sync.current_endpoint.url.is_empty(),
        )
    };

    // -- 2) identify active + selected server --------------------------------
    let active_server = std::fs::read_to_string(game_root.join("Mezeporta/active_server"))
        .unwrap_or_default()
        .trim()
        .to_string();

    // canonical server key for swap/manifest/cache bookkeeping (host:port)
    let (server_name, endpoint_patch_url) = {
        let state_sync = state.state_sync.lock().await;
        (
            state_sync.current_endpoint.server_key(),
            state_sync.current_endpoint.base_url(),
        )
    };

    // Prefer patchServer from API response when present; otherwise use current endpoint base URL.
    let patcher_base_url = if auth_resp.patch_server.trim().is_empty() {
        if is_offline {
            String::new()
        } else {
            normalize_patcher_base_url(&endpoint_patch_url)
        }
    } else {
        normalize_patcher_base_url(auth_resp.patch_server.trim())
    };
    auth_resp.patch_server = patcher_base_url.clone();
    // Reuse the selected server ETag as If-None-Match only when
    // the currently active server matches.
    let local_etag = patcher::cached_server_etag(&game_root, &server_name).unwrap_or_default();
    let etag_for_header: &str = if active_server == server_name {
        &local_etag
    } else {
        ""
    };

    // -- 3) fetch patch list if patch_server is set ---------------------------
    let mut raw_patcher_resp: Option<PatcherResponse> = if !patcher_base_url.is_empty() {
        let state_sync = state.state_sync.lock().await;
        server::patcher_request(
            &state.client,
            state_sync.cancel_shared.clone(),
            &patcher_base_url,
            etag_for_header,
        )
        .send()
        .await
        .map_err(|e| e.into_frontend())?
    } else {
        None
    };
    if let Some(resp) = raw_patcher_resp.as_mut() {
        resp.server_name = server_name.clone();
    }
    // Keep the active server marker in sync for non-patching flows.
    if !is_offline && !server_name.is_empty() && raw_patcher_resp.is_none() {
        let active_file = game_root.join("Mezeporta/active_server");
        if let Some(dir) = active_file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&active_file, &server_name);
    }

    let (endpoint_for_stats, cancel_for_stats) = {
        let state_sync = state.state_sync.lock().await;
        (
            state_sync.current_endpoint.clone(),
            state_sync.cancel_shared.clone(),
        )
    };
    let alt_client_stats = if is_offline {
        altclient_stats::AltClientStats {
            characters: altclient_stats::build_character_stats(
                auth_resp.characters.clone(),
                auth_resp.courses.clone(),
            ),
            ..Default::default()
        }
    } else {
        altclient_stats::fetch(
            &state.client,
            cancel_for_stats,
            &endpoint_for_stats,
            &auth_resp.user.token,
            auth_resp.characters.clone(),
            auth_resp.courses.clone(),
        )
        .await
    };

    // -- 4) lock and store everything -----------------------------------------
    let mut state_sync = state.state_sync.lock().await;
    state_sync.auth_resp = Some(auth_resp.clone());
    state_sync.patcher_resp = raw_patcher_resp;
    state_sync.alt_client_stats = Some(alt_client_stats);
    let has_patch = state_sync.patcher_resp.is_some();

    let endpoint_snapshot = state_sync.current_endpoint.clone();
    state_sync.user_manager.set(
        &endpoint_snapshot,
        UserData {
            username: username.clone(),
            remember_me,
        },
        password.clone(),
    );
    state_sync.session_username = username.clone();
    let um_snapshot = state_sync.user_manager.clone();
    state_sync
        .store
        .with(|s| s.set("user_manager", &um_snapshot));

    Ok(AuthPayload {
        response: auth_resp,
        has_patch,
    })
}

#[tauri::command]
async fn get_alt_client_stats(
    state: tauri::State<'_, TauriState>,
) -> Result<altclient_stats::AltClientStats, String> {
    let state_sync = state.state_sync.lock().await;
    Ok(state_sync.alt_client_stats.clone().unwrap_or_default())
}

#[tauri::command]
async fn get_alt_client_distributions(
    state: tauri::State<'_, TauriState>,
    character_id: u32,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<altclient_stats::AltClientDistributionPage, String> {
    if character_id == 0 {
        return Ok(altclient_stats::AltClientDistributionPage {
            character_id,
            ..Default::default()
        });
    }

    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(6).clamp(1, 6);
    let (endpoint, token, cancel) = {
        let state_sync = state.state_sync.lock().await;
        if state_sync
            .current_endpoint
            .url
            .eq_ignore_ascii_case("OFFLINEMODE")
            || state_sync.current_endpoint.url.trim().is_empty()
        {
            return Ok(altclient_stats::AltClientDistributionPage {
                character_id,
                offset,
                limit,
                ..Default::default()
            });
        }
        let auth = match state_sync.auth_resp.as_ref() {
            Some(value) => value,
            None => {
                return Ok(altclient_stats::AltClientDistributionPage {
                    character_id,
                    offset,
                    limit,
                    ..Default::default()
                });
            }
        };
        if auth.user.token.trim().is_empty() {
            return Ok(altclient_stats::AltClientDistributionPage {
                character_id,
                offset,
                limit,
                ..Default::default()
            });
        }

        (
            state_sync.current_endpoint.clone(),
            auth.user.token.clone(),
            state_sync.cancel_shared.clone(),
        )
    };

    altclient_stats::fetch_distribution_page(
        &state.client,
        cancel,
        &endpoint,
        &token,
        character_id,
        offset,
        limit,
    )
    .await
    .map_err(|err| err.into_frontend())
}

#[tauri::command]
async fn prefetch_alt_character_savedata(
    state: tauri::State<'_, TauriState>,
    character_id: u32,
    savedata_version: Option<String>,
) -> Result<bool, String> {
    if character_id == 0 {
        return Ok(false);
    }

    let requested_savedata_version = normalize_savedata_version_token(savedata_version.as_deref());
    let (endpoint, token, cancel, game_root, identity) = {
        let state_sync = state.state_sync.lock().await;
        if state_sync
            .current_endpoint
            .url
            .eq_ignore_ascii_case("OFFLINEMODE")
            || state_sync.current_endpoint.url.trim().is_empty()
        {
            return Ok(false);
        }
        let auth = match state_sync.auth_resp.as_ref() {
            Some(value) => value,
            None => return Ok(false),
        };
        if !auth.alt_savedata_enabled {
            return Ok(false);
        }
        if auth.user.token.trim().is_empty() {
            return Ok(false);
        }

        let identity = match resolve_alt_character_cache_identity(&state_sync, character_id) {
            Ok(value) => value,
            Err(_) => return Ok(false),
        };
        let identity =
            identity_with_savedata_version(identity, requested_savedata_version.as_deref());

        (
            state_sync.current_endpoint.clone(),
            auth.user.token.clone(),
            state_sync.cancel_shared.clone(),
            state_sync.effective_folder(),
            identity,
        )
    };

    let cache_path = alt_character_cache_path(&game_root, &identity);
    let has_savedata_version = read_alt_character_savedata_version(&game_root, &identity)
        .map(|token| !token.trim().is_empty())
        .unwrap_or(false);

    if let Ok(meta) = fs::metadata(&cache_path) {
        if meta.is_file() && meta.len() > 0 && has_savedata_version {
            return Ok(true);
        }
    }

    let (savedata, response_savedata_version) = fetch_alt_character_savedata_for_endpoint(
        &state.client,
        cancel,
        &endpoint,
        &token,
        character_id,
    )
    .await?;

    if savedata.is_empty() {
        return Ok(false);
    }

    let fallback_savedata_version = version_to_savedata_token(endpoint.version).to_string();
    let effective_savedata_version = requested_savedata_version
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or(response_savedata_version.as_deref())
        .or(Some(fallback_savedata_version.as_str()));

    let mut write_identity = identity.clone();
    if let Some(version_token) = effective_savedata_version {
        write_identity.version_label = savedata_token_to_cache_version_label(version_token);
    }
    let cache_path = alt_character_cache_path(&game_root, &write_identity);

    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|_| "file-error".to_string())?;
    }
    fs::write(&cache_path, savedata).map_err(|_| "file-error".to_string())?;
    ensure_alt_character_savedata_version(&game_root, &write_identity, effective_savedata_version);
    Ok(true)
}

#[tauri::command]
async fn has_alt_character_savedata_version(
    state: tauri::State<'_, TauriState>,
    character_id: u32,
    savedata_version: Option<String>,
) -> Result<bool, String> {
    if character_id == 0 {
        return Ok(false);
    }

    let (game_root, identity) = {
        let state_sync = state.state_sync.lock().await;

        let auth = match state_sync.auth_resp.as_ref() {
            Some(value) => value,
            None => return Ok(false),
        };
        if !auth.alt_savedata_enabled {
            return Ok(false);
        }

        let identity = match resolve_alt_character_cache_identity(&state_sync, character_id) {
            Ok(value) => value,
            Err(_) => return Ok(false),
        };
        let identity = identity_with_savedata_version(identity, savedata_version.as_deref());

        (state_sync.effective_folder(), identity)
    };

    match read_alt_character_savedata_version(&game_root, &identity) {
        Ok(token) => Ok(!token.trim().is_empty()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
async fn read_alt_character_savedata_cache(
    state: tauri::State<'_, TauriState>,
    character_id: u32,
    savedata_version: Option<String>,
) -> Result<AltCharacterSavedataCacheResponse, String> {
    if character_id == 0 {
        return Err("internal-error".into());
    }

    let (game_root, identity) = {
        let state_sync = state.state_sync.lock().await;

        let auth = state_sync
            .auth_resp
            .as_ref()
            .ok_or_else(|| "internal-error".to_string())?;
        if !auth.alt_savedata_enabled {
            return Err("internal-error".into());
        }

        let identity = resolve_alt_character_cache_identity(&state_sync, character_id)
            .map_err(|_| "internal-error".to_string())?;
        let identity = identity_with_savedata_version(identity, savedata_version.as_deref());

        (state_sync.effective_folder(), identity)
    };

    let cache_path = alt_character_cache_path(&game_root, &identity);
    let data = fs::read(&cache_path).map_err(|_| "file-error".to_string())?;
    if data.is_empty() {
        return Err("file-error".into());
    }

    let gsv = read_alt_character_savedata_version(&game_root, &identity)?;

    Ok(AltCharacterSavedataCacheResponse {
        savedata: base64::engine::general_purpose::STANDARD.encode(data),
        gsv,
    })
}

#[tauri::command]
async fn login(
    state: tauri::State<'_, TauriState>,
    username: String,
    password: String,
    remember_me: bool,
) -> Result<AuthPayload, String> {
    let auth_req = {
        let mut state_sync = state.state_sync.lock().await;
        if username.is_empty() || password.is_empty() {
            return Err("username-password-empty-error".into());
        }
        state_sync.cancel_shared.cancel();
        state_sync.cancel_shared = CancellationToken::new();
        login_request_for_endpoint(
            &state.client,
            state_sync.cancel_shared.clone(),
            &state_sync.current_endpoint,
            &username,
            &password,
        )
    };
    auth(state, username, password, remember_me, auth_req).await
}

#[tauri::command]
async fn register(
    state: tauri::State<'_, TauriState>,
    username: String,
    password: String,
    remember_me: bool,
) -> Result<AuthPayload, String> {
    let auth_req = {
        let mut state_sync = state.state_sync.lock().await;
        if username.is_empty() || password.is_empty() {
            return Err("username-password-empty-error".into());
        }
        state_sync.cancel_shared.cancel();
        state_sync.cancel_shared = CancellationToken::new();
        register_request_for_endpoint(
            &state.client,
            state_sync.cancel_shared.clone(),
            &state_sync.current_endpoint,
            &username,
            &password,
        )
    };
    auth(state, username, password, remember_me, auth_req).await
}

async fn reauth(state: &mut tauri::State<'_, TauriState>) -> Result<(), String> {
    let req = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_shared.cancel();
        state_sync.cancel_shared = CancellationToken::new();
        let (userdata, password) = state_sync.user_manager.get(&state_sync.current_endpoint);
        login_request_for_endpoint(
            &state.client,
            state_sync.cancel_shared.clone(),
            &state_sync.current_endpoint,
            &userdata.username,
            &password,
        )
    };
    let data = req.send().await.map_err(|e| e.into_frontend())?;
    {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.auth_resp = Some(data);
    }
    Ok(())
}

async fn get_create_character_request(
    state: &mut tauri::State<'_, TauriState>,
) -> Result<server::JsonRequest<server::CharacterData>, String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.cancel_shared.cancel();
    state_sync.cancel_shared = CancellationToken::new();
    let req = create_character_request_for_endpoint(
        &state.client,
        state_sync.cancel_shared.clone(),
        &state_sync.current_endpoint,
        &state_sync.auth_resp_err()?.user.token,
    );
    Ok(req)
}

#[tauri::command]
async fn create_character(
    window: Window,
    mut state: tauri::State<'_, TauriState>,
) -> Result<(), String> {
    let req = get_create_character_request(&mut state).await?;
    let character = match req.send().await {
        Ok(data) => data,
        Err(server::Error::Server(401, _)) => {
            reauth(&mut state).await?;
            let req = get_create_character_request(&mut state).await?;
            req.send().await.map_err(|e| e.into_frontend())?
        }
        Err(e) => return Err(e.into_frontend()),
    };
    #[cfg(not(windows))]
    {
        let app_handle = window.app_handle();
        let (config, game_root, launcher_prefs) = {
            let mut state_sync = state.state_sync.lock().await;
            state_sync
                .auth_resp
                .as_mut()
                .ok_or("Auth data was not set")?
                .characters
                .push(character.clone());
            state_sync.store.with(|s| {
                s.set("last_char_id", character.id);
            });
            build_launch_bundle(&state_sync, character.id, true)?
        };

        run_mhf(config, game_root, launcher_prefs, false, app_handle)?;

        let mut state_sync = state.state_sync.lock().await;
        state_sync.skip_child_cleanup_once = true;
        cancel_all_requests(&mut state_sync);
        drop(state_sync);
        window.app_handle().exit(0);
        return Ok(());
    }

    #[cfg(windows)]
    {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.exit_reason = Some(ExitSignal::RunGame(character.id, true));
        state_sync
            .auth_resp
            .as_mut()
            .ok_or("Auth data was not set")?
            .characters
            .push(character.clone());
        state_sync.store.with(|s| {
            s.set("last_char_id", character.id);
        });
        cancel_all_requests(&mut state_sync);
        drop(state_sync);
        window.close().map_err(|e| {
            error!("failed to close window: {}", e);
            "internal-error"
        })?;
        Ok(())
    }
}

#[tauri::command]
async fn select_character(
    window: Window,
    state: tauri::State<'_, TauriState>,
    character_id: u32,
    friend_signature: Option<String>,
    savedata_version: Option<String>,
) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        let app_handle = window.app_handle();
        let (config, game_root, launcher_prefs) = {
            let mut state_sync = state.state_sync.lock().await;
            if let Some(friend_signature) = friend_signature {
                state_sync.launcher_prefs.friend_signature =
                    normalize_friend_signature(&friend_signature);
                let launcher_prefs = state_sync.launcher_prefs.clone();
                state_sync
                    .store
                    .with(|s| s.set("launcher_prefs", launcher_prefs));
            }
            state_sync.store.with(|s| {
                s.set("last_char_id", character_id);
            });
            if let Ok(identity) = resolve_alt_character_cache_identity(&state_sync, character_id) {
                let identity =
                    identity_with_savedata_version(identity, savedata_version.as_deref());
                ensure_alt_character_savedata_version(
                    &state_sync.effective_folder(),
                    &identity,
                    savedata_version.as_deref(),
                );
                clear_alt_character_cache_for_character(&state_sync.effective_folder(), &identity);
            }
            build_launch_bundle(&state_sync, character_id, false)?
        };

        run_mhf(config, game_root, launcher_prefs, false, app_handle)?;

        let mut state_sync = state.state_sync.lock().await;
        state_sync.skip_child_cleanup_once = true;
        cancel_all_requests(&mut state_sync);
        drop(state_sync);
        window.app_handle().exit(0);
        return Ok(());
    }

    #[cfg(windows)]
    {
        let mut state_sync = state.state_sync.lock().await;
        if let Some(friend_signature) = friend_signature {
            state_sync.launcher_prefs.friend_signature =
                normalize_friend_signature(&friend_signature);
            let launcher_prefs = state_sync.launcher_prefs.clone();
            state_sync
                .store
                .with(|s| s.set("launcher_prefs", launcher_prefs));
        }
        state_sync.exit_reason = Some(ExitSignal::RunGame(character_id, false));
        state_sync.store.with(|s| {
            s.set("last_char_id", character_id);
        });
        if let Ok(identity) = resolve_alt_character_cache_identity(&state_sync, character_id) {
            let identity = identity_with_savedata_version(identity, savedata_version.as_deref());
            ensure_alt_character_savedata_version(
                &state_sync.effective_folder(),
                &identity,
                savedata_version.as_deref(),
            );
            clear_alt_character_cache_for_character(&state_sync.effective_folder(), &identity);
        }
        cancel_all_requests(&mut state_sync);
        drop(state_sync);
        window.close().map_err(|e| {
            error!("failed to close window: {}", e);
            "internal-error"
        })?;
        Ok(())
    }
}

async fn get_delete_character_request(
    state: &mut tauri::State<'_, TauriState>,
    character_id: i32,
) -> Result<server::JsonRequest<server::EmptyResponse>, String> {
    let mut state_sync = state.state_sync.lock().await;
    state_sync.cancel_shared.cancel();
    state_sync.cancel_shared = CancellationToken::new();
    let req = delete_character_request_for_endpoint(
        &state.client,
        state_sync.cancel_shared.clone(),
        &state_sync.current_endpoint,
        &state_sync.auth_resp_err()?.user.token,
        character_id,
    );
    Ok(req)
}

#[tauri::command]
async fn delete_character(
    mut state: tauri::State<'_, TauriState>,
    character_id: i32,
) -> Result<(), String> {
    let req = get_delete_character_request(&mut state, character_id).await?;
    let _ = match req.send().await {
        Ok(data) => data,
        Err(server::Error::Server(401, _)) => {
            reauth(&mut state).await?;
            let req = get_delete_character_request(&mut state, character_id).await?;
            req.send().await.map_err(|e| e.into_frontend())?
        }
        Err(e) => return Err(e.into_frontend()),
    };
    Ok(())
}

async fn get_export_character_request(
    state: &mut tauri::State<'_, TauriState>,
    character_id: i32,
) -> Result<server::JsonRequest<Value>, String> {
    let state_sync = state.state_sync.lock().await;
    let req = export_character_request_for_endpoint(
        &state.client,
        CancellationToken::new(),
        &state_sync.current_endpoint,
        &state_sync.auth_resp_err()?.user.token,
        character_id,
    );
    Ok(req)
}

#[tauri::command]
async fn export_character(
    mut state: tauri::State<'_, TauriState>,
    character_id: i32,
) -> Result<PathBuf, String> {
    let req = get_export_character_request(&mut state, character_id).await?;
    let data = match req.send().await {
        Ok(data) => data,
        Err(server::Error::Server(401, _)) => {
            reauth(&mut state).await?;
            let req = get_export_character_request(&mut state, character_id).await?;
            req.send().await.map_err(|e| e.into_frontend())?
        }
        Err(e) => return Err(e.into_frontend()),
    };
    let id = data.get("id").and_then(Value::as_i64).unwrap_or_default();
    let name = data.get("name").and_then(Value::as_str).unwrap_or_default();
    let folder_name = format!("./saves/{}-{}.json", id, name);
    let path = Path::new(&folder_name);
    path.parent()
        .and_then(|p| std::fs::create_dir_all(p).ok())
        .ok_or("file-error")?;
    File::options()
        .write(true)
        .create(true)
        .open(path)
        .ok()
        .and_then(|f| serde_json::to_writer_pretty(f, &data).ok())
        .ok_or("file-error")?;
    path::absolute(path).or(Err("file-error".into()))
}

#[tauri::command]
async fn patcher_start(window: Window, state: tauri::State<'_, TauriState>) -> Result<(), String> {
    let (patcher_url, patcher_resp, game_folder, cancel) = {
        let mut state_sync = state.state_sync.lock().await;
        state_sync.cancel_shared.cancel();
        state_sync.cancel_shared = CancellationToken::new();
        let patcher_url = {
            let auth_patch = state_sync.auth_resp_err()?.patch_server.clone();
            if auth_patch.trim().is_empty() {
                normalize_patcher_base_url(&state_sync.current_endpoint.base_url())
            } else {
                normalize_patcher_base_url(auth_patch.trim())
            }
        };
        (
            patcher_url,
            state_sync.patcher_resp.take(),
            state_sync.effective_folder(),
            state_sync.cancel_shared.clone(),
        )
    };
    let Some(patcher_resp) = patcher_resp else {
        return Err("internal-error".into());
    };
    if patcher_url.is_empty() {
        return Err(patcher::NETWORK_ERROR.into());
    }
    let _client = state.client.clone();
    tauri::async_runtime::spawn(patcher::patch(
        window,
        _client,
        patcher_url,
        patcher_resp,
        game_folder,
        cancel,
    ));
    Ok(())
}

#[tauri::command]
async fn patcher_stop(state: tauri::State<'_, TauriState>) -> Result<(), String> {
    let state_sync = state.state_sync.lock().await;
    state_sync.cancel_shared.cancel();
    Ok(())
}

#[tauri::command]
async fn patcher_swap_info(
    state: tauri::State<'_, TauriState>,
    target_server: String,
) -> Result<patcher::SwapInfo, String> {
    let state_sync = state.state_sync.lock().await;
    let root = state_sync.effective_folder();
    Ok(patcher::swap_info(&root, &target_server))
}

#[tauri::command]
async fn patcher_swap_to_cached(
    state: tauri::State<'_, TauriState>,
    target_server: String,
) -> Result<(), String> {
    let state_sync = state.state_sync.lock().await;
    let root = state_sync.effective_folder();
    patcher::swap_to_cached_server(&root, &target_server)
}

fn cancel_all_requests(state_sync: &mut TauriStateSync) {
    state_sync.cancel_shared.cancel();
    state_sync.cancel_launcher.cancel();
    state_sync.cancel_serverlist.cancel();
    state_sync.cancel_messagelist.cancel();
}

fn handle_style(window: &mut Window, style: u32, ui_prefs: &UiPrefs) {
    let style = normalize_launcher_style(style);
    let (lock_aspect, numerator, denominator) = match style {
        CLASSIC_STYLE => (true, CLASSIC_ASPECT_NUMERATOR, CLASSIC_ASPECT_DENOMINATOR),
        PS4_STYLE => (
            true,
            WIDESCREEN_ASPECT_NUMERATOR,
            WIDESCREEN_ASPECT_DENOMINATOR,
        ),
        _ => (
            false,
            WIDESCREEN_ASPECT_NUMERATOR,
            WIDESCREEN_ASPECT_DENOMINATOR,
        ),
    };
    let target_size = ui_prefs.launcher_window_size_for_style(style).to_physical();
    set_ps4_aspect_lock_enabled(lock_aspect, numerator, denominator);
    match style {
        CLASSIC_STYLE => {
            window
                .set_decorations(false)
                .unwrap_or_else(|e| warn!("failed to set window decorations: {}", e));
            window
                .set_resizable(launcher_style_is_resizable(style))
                .unwrap_or_else(|e| warn!("failed to set window resizable state: {}", e));
            window
                .set_size(target_size)
                .unwrap_or_else(|e| warn!("failed to set window size: {}", e));
        }
        PS4_STYLE => {
            window
                .set_decorations(false)
                .unwrap_or_else(|e| warn!("failed to set window decorations: {}", e));
            window
                .set_resizable(launcher_style_is_resizable(style))
                .unwrap_or_else(|e| warn!("failed to set window resizable state: {}", e));
            window
                .set_size(target_size)
                .unwrap_or_else(|e| warn!("failed to set window size: {}", e));
        }
        _ => {}
    }
}

#[tauri::command]
async fn shutdown_launcher(
    window: Window,
    state: tauri::State<'_, TauriState>,
) -> Result<(), String> {
    let app_handle = window.app_handle();
    let mut state_sync = state.state_sync.lock().await;
    state_sync.exit_reason = None;
    cancel_all_requests(&mut state_sync);
    drop(state_sync);
    window.close().map_err(|e| {
        error!("failed to close window: {}", e);
        "internal-error"
    })?;
    app_handle.exit(0);
    Ok(())
}

async fn handle_remote_endpoints(
    window: &Window,
    req: server::JsonRequest<Vec<Endpoint>>,
    state_sync_mutex: Arc<Mutex<TauriStateSync>>,
) {
    let mut serverlist_endpoints = match req.send().await {
        Ok(endpoints) => endpoints,
        Err(e) => {
            warn!("failed to fetch remote servers: {}", e);
            window
                .emit("log", LogPayload::warning("remote-endpoint-error"))
                .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
            return;
        }
    };
    for endpoint in &mut serverlist_endpoints {
        endpoint.is_remote = true;
    }
    let mut remote_endpoints = config::get_default_endpoints();
    let default_len = remote_endpoints.len();
    remote_endpoints.extend_valid(serverlist_endpoints);
    let state_sync = &mut *state_sync_mutex.lock().await;
    if state_sync.current_endpoint.is_remote
        && !remote_endpoints.contains(&state_sync.current_endpoint)
    {
        remote_endpoints.insert(default_len, state_sync.current_endpoint.clone())
    }
    remote_endpoints.apply_config(&state_sync.remote_endpoints_config);
    state_sync.remote_endpoints = remote_endpoints;
    let payload = EndpointsPayload {
        remote_endpoints: Some(state_sync.remote_endpoints.clone()),
        ..Default::default()
    };
    window
        .emit("endpoints", payload)
        .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
}

async fn handle_remote_messages(
    window: &Window,
    req: server::JsonRequest<Vec<MessageData>>,
    state_sync_mutex: Arc<Mutex<TauriStateSync>>,
) {
    match req.send().await {
        Ok(messages) => {
            let r = window.emit("remote_messages", messages.clone());
            let mut state_sync = state_sync_mutex.lock().await;
            state_sync.remote_messages = messages;
            r
        }
        Err(e) => {
            warn!("failed to fetch global messages: {}", e);
            window.emit("log", LogPayload::warning("remote-messages-error"))
        }
    }
    .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
}

impl From<server::FriendData> for mhf_iel::FriendData {
    fn from(f: server::FriendData) -> Self {
        mhf_iel::FriendData {
            cid: f.cid,
            id: f.id,
            name: f.name,
        }
    }
}

// also support mapping &server::FriendData
impl From<&server::FriendData> for mhf_iel::FriendData {
    fn from(f: &server::FriendData) -> Self {
        mhf_iel::FriendData {
            cid: f.cid,
            id: f.id,
            name: f.name.clone(),
        }
    }
}

fn main() {
    let _single_instance_guard = acquire_single_instance_mutex();
    // Log plugin has an issue where it cannot be initialized twice.
    let mut log_plugin_initial = None;
    loop {
        let (config, run, game_root, launcher_prefs_for_launch, app_handle) = {
            let default_endpoints = config::get_default_endpoints();
            let current_endpoint = default_endpoints[0].clone();
            let state_sync = Arc::new(Mutex::new(TauriStateSync {
                style: CLASSIC_STYLE,
                remote_endpoints: default_endpoints,
                current_endpoint,
                locale: "en".into(),
                serverlist_url: DEFAULT_SERVERLIST_URL.into(),
                messagelist_url: DEFAULT_MESSAGELIST_URL.into(),
                ..Default::default()
            }));
            // resolve <game>/Mezeporta/config.json
            let game_root = {
                // lock once in this thread, grab the folder, then drop the guard
                let guard = state_sync.blocking_lock();
                guard.effective_folder()
            };

            if log_plugin_initial.is_none() {
                let targets = vec![LogTarget::Stdout, LogTarget::Webview];
                log_plugin_initial = Some(
                    tauri_plugin_log::Builder::default()
                        .targets(targets)
                        .build(),
                );
            }

            // initialize the plugin; we'll open the actual Store in `setup`
            let store_plugin = tauri_plugin_store::Builder::default().build();

            let mut builder = tauri::Builder::default().plugin(store_plugin);
            if let Some(log_plugin) = log_plugin_initial.take() {
                builder = builder.plugin(log_plugin);
            }
            let mut app = match builder
                .manage(TauriState {
                    client: reqwest::ClientBuilder::new()
                        .gzip(true)
                        .connect_timeout(Duration::from_secs(5))
                        .timeout(Duration::from_secs(10))
                        .build()
                        .unwrap_or_else(|e| {
                            error!(
                                "failed to build reqwest client, falling back to default: {}",
                                e
                            );
                            reqwest::Client::new()
                        }),
                    state_sync: state_sync.clone(),
                })
                .setup(|app| -> Result<(), Box<dyn std::error::Error>> {
                    let state: tauri::State<'_, TauriState> = app.state();
                    let app_handle = app.handle();

                    let initial_game_root = {
                        let guard = state.state_sync.blocking_lock();
                        guard.effective_folder()
                    };
                    ensure_config_store_parent(&initial_game_root).map_err(runtime_error)?;
                    let initial_store_path = config_store_path(&initial_game_root);
                    let mut bootstrap_store =
                        create_store_builder(&app_handle, initial_store_path.clone());
                    let bootstrap_loaded = {
                        let state_sync = &mut *state.state_sync.blocking_lock();
                        load_state_sync_from_store(&mut bootstrap_store, state_sync)
                            .map_err(runtime_error)?
                    };

                    let final_game_root = {
                        let guard = state.state_sync.blocking_lock();
                        guard.effective_folder()
                    };
                    ensure_config_store_parent(&final_game_root).map_err(runtime_error)?;
                    ensure_webview_data_dir(&final_game_root).map_err(runtime_error)?;
                    let final_store_path = config_store_path(&final_game_root);

                    let mut store = if final_store_path == initial_store_path {
                        bootstrap_store
                    } else {
                        if !final_store_path.exists()
                            && bootstrap_loaded
                            && initial_store_path.exists()
                        {
                            if let Some(parent) = final_store_path.parent() {
                                fs::create_dir_all(parent).map_err(|err| {
                                    runtime_error(format!(
                                        "failed to create {}: {}",
                                        parent.display(),
                                        err
                                    ))
                                })?;
                            }
                            fs::copy(&initial_store_path, &final_store_path).map_err(|err| {
                                runtime_error(format!(
                                    "failed to seed {} from {}: {}",
                                    final_store_path.display(),
                                    initial_store_path.display(),
                                    err
                                ))
                            })?;
                        }
                        create_store_builder(&app_handle, final_store_path.clone())
                    };

                    let final_loaded = if final_store_path == initial_store_path {
                        bootstrap_loaded
                    } else {
                        let state_sync = &mut *state.state_sync.blocking_lock();
                        load_state_sync_from_store(&mut store, state_sync).map_err(runtime_error)?
                    };

                    if !final_loaded {
                        initialize_config_store(&mut store).map_err(runtime_error)?;
                        let state_sync = &mut *state.state_sync.blocking_lock();
                        reset_state_sync_store_defaults(state_sync);
                    }

                    let (
                        style,
                        ui_prefs,
                        serverlist_url,
                        messagelist_url,
                        cancel_serverlist,
                        cancel_messagelist,
                    ) = {
                        let state_sync = &mut *state.state_sync.blocking_lock();
                        state_sync.store = StoreHelper::new(store);
                        (
                            state_sync.style,
                            state_sync.ui_prefs.clone(),
                            state_sync.serverlist_url.clone(),
                            state_sync.messagelist_url.clone(),
                            state_sync.cancel_serverlist.clone(),
                            state_sync.cancel_messagelist.clone(),
                        )
                    };
                    #[cfg(not(windows))]
                    ensure_linux_path_defaults();
                    let mut window = create_main_window(&app_handle, &final_game_root, &ui_prefs)
                        .map_err(runtime_error)?;
                    install_ps4_aspect_lock(&window);
                    handle_style(&mut window, style, &ui_prefs);
                    let delayed_show_window = window.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(1200));
                        delayed_show_window
                            .show()
                            .unwrap_or_else(|e| warn!("failed to show main window: {}", e));
                    });
                    let support_game_root = final_game_root.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(1800));
                        if let Err(error) = ensure_mezeporta_support_dirs(&support_game_root) {
                            warn!("failed to prepare Mezeporta support folders: {}", error);
                        }
                    });
                    if !serverlist_url.is_empty() {
                        let endpoints_req = server::simple_request(
                            &state.client,
                            cancel_serverlist,
                            &serverlist_url,
                        );
                        let state_sync_mutex = state.state_sync.clone();
                        let window = window.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_remote_endpoints(&window, endpoints_req, state_sync_mutex).await
                        });
                    }
                    if !messagelist_url.is_empty() {
                        let messages_req = server::simple_request(
                            &state.client,
                            cancel_messagelist,
                            &messagelist_url,
                        );
                        let state_sync_mutex = state.state_sync.clone();
                        let window = window.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_remote_messages(&window, messages_req, state_sync_mutex).await
                        });
                    }
                    Ok(())
                })
                .invoke_handler(tauri::generate_handler![
                    initial_data,
                    set_style,
                    set_locale,
                    set_launcher_pref,
                    set_ui_pref,
                    get_linux_prefix_status,
                    install_linux_portable_prefix,
                    play_linux_ui_sfx,
                    sync_controller_dll_files,
                    detect_game_version,
                    get_server_version_info,
                    set_setting,
                    set_endpoints,
                    set_remote_endpoints,
                    set_current_endpoint,
                    set_game_folder,
                    set_serverlist_url,
                    set_messagelist_url,
                    shutdown_launcher,
                    get_alt_client_stats,
                    get_alt_client_distributions,
                    has_alt_character_savedata_version,
                    prefetch_alt_character_savedata,
                    read_alt_character_savedata_cache,
                    login,
                    register,
                    create_character,
                    select_character,
                    delete_character,
                    export_character,
                    patcher_start,
                    patcher_stop,
                    patcher::reset_game_files,
                    patcher_swap_info,
                    patcher_swap_to_cached
                ])
                .build(tauri::generate_context!())
            {
                Ok(app) => app,
                Err(e) => {
                    error!("error while building tauri application: {}", e);
                    break;
                }
            };
            let app_handle = app.handle();
            loop {
                let iteration = app.run_iteration();
                if iteration.window_count == 0 {
                    break;
                }
            }
            let skip_child_cleanup = {
                let mut state_sync = state_sync.blocking_lock();
                let skip = state_sync.skip_child_cleanup_once;
                state_sync.skip_child_cleanup_once = false;
                skip
            };
            if !skip_child_cleanup {
                tauri::api::process::kill_children();
            }

            let state_sync = state_sync.blocking_lock();
            if let Some(ExitSignal::RunGame(char_id, char_new)) = state_sync.exit_reason {
                let preload_controller_dlls = if cfg!(windows) {
                    state_sync.launcher_prefs.preload_controller_dlls
                } else {
                    false
                };
                let friend_signature = state_sync.launcher_prefs.friend_signature.clone();
                let launcher_prefs = state_sync.launcher_prefs.clone();

                if let Some(auth_resp) = state_sync.auth_resp.as_ref() {
                    if let Some(char) = auth_resp.characters.iter().find(|c| c.id == char_id) {
                        let char_ids = auth_resp.characters.iter().map(|c| c.id).collect();
                        let notices = auth_resp
                            .notices
                            .iter()
                            .map(|n| mhf_iel::Notice {
                                flags: 0,
                                data: n.clone(),
                            })
                            .collect();
                        let (userdata, password) =
                            state_sync.user_manager.get(&state_sync.current_endpoint);

                        let mutex_selection = resolve_effective_mutex_versions(&state_sync);
                        let effective_version = resolve_effective_game_version(&state_sync);

                        let mut config = MhfConfig {
                            char_id,
                            char_name: char.name.clone(),
                            char_gr: char.gr,
                            char_hr: char.hr,
                            char_ids,
                            char_new,
                            user_token_id: auth_resp.user.token_id,
                            user_token: auth_resp.user.token.clone(),
                            user_name: userdata.username,
                            user_password: password,
                            user_rights: auth_resp.user.rights,
                            friends: auth_resp.friends.iter().map(Into::into).collect(),
                            server_host: state_sync.current_endpoint.host(),
                            server_port: state_sync.current_endpoint.game_port.unwrap_or(53310)
                                as u32,
                            entrance_count: auth_resp.entrance_count,
                            current_ts: auth_resp.current_ts,
                            expiry_ts: auth_resp.expiry_ts,
                            notices,
                            mez_event_id: 0,
                            mez_start: 0,
                            mez_end: 0,
                            mez_solo_tickets: 0,
                            mez_group_tickets: 0,
                            mez_stalls: vec![],
                            mhf_flags: None,
                            version: effective_version,
                            mutex_version: mutex_selection.primary,
                            mutex_fallback_version: mutex_selection.fallback,
                            preload_controller_dlls,
                            friend_signature: Some(friend_signature),
                            enable_font_registration: true,
                            mhf_folder: state_sync
                                .current_endpoint
                                .game_folder
                                .as_ref()
                                .or_else(|| state_sync.game_folder.as_ref())
                                .cloned(),
                            font_path: state_sync
                                .current_endpoint
                                .game_folder
                                .as_ref()
                                .or_else(|| state_sync.game_folder.as_ref())
                                .and_then(|root| {
                                    resolve_launcher_font_path(root, effective_version)
                                }),
                        };
                        if let Some(mez_fes) = auth_resp.mez_fez.as_ref() {
                            config.mez_event_id = mez_fes.id;
                            config.mez_start = mez_fes.start;
                            config.mez_end = mez_fes.end;
                            config.mez_solo_tickets = mez_fes.solo_tickets;
                            config.mez_group_tickets = mez_fes.group_tickets;
                            config.mez_stalls = mez_fes
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
                        (config, true, game_root, launcher_prefs, app_handle)
                    } else {
                        error!("selected character {} not found in auth response", char_id);
                        (
                            MhfConfig::default(),
                            false,
                            game_root,
                            launcher_prefs,
                            app_handle,
                        )
                    }
                } else {
                    error!("missing auth response when preparing game launch");
                    (
                        MhfConfig::default(),
                        false,
                        game_root,
                        launcher_prefs,
                        app_handle,
                    )
                }
            } else {
                (
                    MhfConfig::default(),
                    false,
                    game_root,
                    LauncherPrefs::default(),
                    app_handle,
                )
            }
        };
        if run {
            match run_mhf(
                config,
                game_root,
                launcher_prefs_for_launch,
                true,
                app_handle,
            ) {
                Ok(code) => {
                    info!("exited with code {}", code);
                    info!("launcher exiting after game exit");
                    std::process::exit(0);
                }
                Err(err) => {
                    error!("failed to launch mhf: {}", err);
                    break;
                }
            };
        } else {
            break;
        }
    }
    info!("app exit");
    std::process::exit(0);
}
