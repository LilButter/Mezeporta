use crate::manifest::Manifest;
use crate::{server::PatcherResponse, LogPayload};
use log::{info, warn};
use reqwest;
use serde::Serialize;
use serde_repr::Serialize_repr;
use sha2::Digest;
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
use tauri::Window;
use tokio::select;
use tokio_util::sync::CancellationToken;

pub const NETWORK_ERROR: &str = "patcher-network-error";
pub const FILE_ERROR: &str = "patcher-file-error";
const ACTIVE_SERVER_FILE: &str = "Mezeporta/active_server";
const CLIENT_ETAG_FILE: &str = "patch.etag";
const SERVER_CACHE_DIR: &str = "Mezeporta/Servers";
const PRIMARY_BACKUP_SUFFIX: &str = ".mezeb";
const LEGACY_BACKUP_SUFFIXES: &[&str] = &[".butterold", ".buttold"];

fn sanitize_server_key(server: &str) -> String {
    server
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// foo/bar.txt -> foo/bar.txt.mezeb
fn backup_name(p: &Path) -> PathBuf {
    if let Some(name) = p.file_name() {
        let mut f = name.to_os_string();
        f.push(PRIMARY_BACKUP_SUFFIX);
        p.with_file_name(f)
    } else {
        p.with_extension(PRIMARY_BACKUP_SUFFIX.trim_start_matches('.'))
    }
}

fn backup_path_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    if let Some(name) = path.file_name() {
        let mut file_name = name.to_os_string();
        file_name.push(suffix);
        path.with_file_name(file_name)
    } else {
        path.with_extension(suffix.trim_start_matches('.'))
    }
}

fn legacy_backup_names(path: &Path) -> [PathBuf; 2] {
    [
        backup_path_with_suffix(path, LEGACY_BACKUP_SUFFIXES[0]),
        backup_path_with_suffix(path, LEGACY_BACKUP_SUFFIXES[1]),
    ]
}

fn clear_readonly_if_needed(path: &Path) -> io::Result<()> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    let permissions = metadata.permissions();
    if !permissions.readonly() {
        return Ok(());
    }
    let mut writable = permissions;
    writable.set_readonly(false);
    fs::set_permissions(path, writable)
}

fn remove_file_force(path: &Path) -> io::Result<()> {
    clear_readonly_if_needed(path)?;
    fs::remove_file(path)
}

fn rename_force(source: &Path, destination: &Path) -> io::Result<()> {
    clear_readonly_if_needed(source)?;
    if destination.exists() {
        clear_readonly_if_needed(destination)?;
        fs::remove_file(destination)?;
    }
    fs::rename(source, destination)
}

fn cleanup_legacy_backups(original: &Path) -> io::Result<()> {
    for backup in legacy_backup_names(original) {
        if backup.exists() {
            remove_file_force(&backup)?;
        }
    }
    Ok(())
}

fn preferred_backup_name(original: &Path) -> Option<PathBuf> {
    let primary = backup_name(original);
    if primary.exists() {
        return Some(primary);
    }

    legacy_backup_names(original)
        .into_iter()
        .find(|candidate| candidate.exists())
}

fn promote_legacy_backup(original: &Path) -> io::Result<bool> {
    let primary = backup_name(original);
    if primary.exists() {
        cleanup_legacy_backups(original)?;
        return Ok(true);
    }

    for legacy in legacy_backup_names(original) {
        if legacy.exists() {
            rename_force(&legacy, &primary)?;
            cleanup_legacy_backups(original)?;
            return Ok(true);
        }
    }

    Ok(false)
}

fn restore_preferred_backup(original: &Path) -> io::Result<bool> {
    let Some(backup) = preferred_backup_name(original) else {
        return Ok(false);
    };

    if original.exists() {
        remove_file_force(original)?;
    }
    rename_force(&backup, original)?;
    cleanup_legacy_backups(original)?;
    Ok(true)
}

fn original_path_from_backup(path: &Path) -> Option<PathBuf> {
    let file_name = path.file_name()?.to_str()?;
    for suffix in
        std::iter::once(PRIMARY_BACKUP_SUFFIX).chain(LEGACY_BACKUP_SUFFIXES.iter().copied())
    {
        if let Some(stripped) = file_name.strip_suffix(suffix) {
            return Some(path.with_file_name(stripped));
        }
    }
    None
}

/// If `target` exists, rename to `*.mezeb`.
/// Returns **true** if backed up an existing file.
fn backup_original(target: &Path) -> io::Result<bool> {
    if !target.exists() {
        return Ok(false);
    }

    let backup = backup_name(target);
    if backup.exists() || promote_legacy_backup(target)? {
        // Original backup already exists from a prior patch for this server.
        // Keep the oldest backup and replace the current target in-place.
        remove_file_force(target)?;
    } else {
        rename_force(target, &backup)?;
    }

    Ok(true)
}

#[derive(Debug, Serialize_repr, Clone)]
#[repr(u8)]
enum State {
    Checking,
    Downloading,
    Restoring,
    Patching,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PatcherEvent {
    total: usize,
    current: usize,
    state: State,
    queue_position: usize,
}

fn send_event(window: &Window, total: usize, current: usize, state: State, queue_position: usize) {
    window
        .emit(
            "patcher",
            PatcherEvent {
                total,
                current,
                state,
                queue_position,
            },
        )
        .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
}

fn send_error(window: &Window, msg: &str) {
    warn!("patcher error: {}", msg);
    window
        .emit("log", LogPayload::error(msg))
        .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
    window
        .emit(
            "patcher",
            PatcherEvent {
                total: 0,
                current: 0,
                state: State::Error,
                queue_position: 0,
            },
        )
        .unwrap_or_else(|e| warn!("failed to emit message: {}", e));
}

fn parse_queue_position_from_headers(headers: &reqwest::header::HeaderMap) -> usize {
    headers
        .get("X-AltClient-Patch-Queue-Position")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

fn get_changed_paths<'a>(
    patcher_content: &'a str,
    game_folder: &Path,
) -> Result<Vec<&'a str>, &'static str> {
    patcher_content
        .lines()
        .filter_map(|line| {
            let Some((patcher_hash, mut patcher_path)) = line.split_once('\t') else {
                return Some(Err(NETWORK_ERROR));
            };
            patcher_path = patcher_path.trim_start_matches('/');
            let client_path = game_folder.join(patcher_path);

            info!(
                "files: {} {} {}",
                game_folder.display(),
                &patcher_path,
                client_path.display()
            );

            if let Ok(mut file) = fs::File::open(&client_path) {
                let mut hasher = sha2::Sha256::new();
                if io::copy(&mut file, &mut hasher).is_ok() {
                    let client_hash = format!("{:x}", hasher.finalize());
                    info!("hashes: {} {}", patcher_hash, client_hash);
                    if patcher_hash == client_hash {
                        return None;
                    }
                };
            };
            Some(Ok(patcher_path))
        })
        .collect::<Result<Vec<_>, _>>()
        .or(Err(NETWORK_ERROR))
}

async fn download_changed_paths(
    window: &Window,
    client: &reqwest::Client,
    patcher_url: &str,
    changed_paths: &[&str],
    patcher_folder: &Path,
    cancel: CancellationToken,
) -> Result<(), &'static str> {
    let total = changed_paths.len();
    let mut current = 0;
    for changed_path in changed_paths {
        let req = client
            .get(format!("{}/{}", patcher_url, changed_path))
            .send();
        let mut resp = select! {
            _ = cancel.cancelled() => return Ok(()),
            resp = req => resp.or(Err(NETWORK_ERROR))?,
        };
        let queue_position = parse_queue_position_from_headers(resp.headers());
        if queue_position > 0 {
            send_event(window, total, current, State::Downloading, queue_position);
        }
        let patcher_path = patcher_folder.join(changed_path);
        fs::create_dir_all(patcher_path.parent().ok_or(FILE_ERROR)?).or(Err(FILE_ERROR))?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(patcher_path)
            .or(Err(FILE_ERROR))?;
        while let Some(chunk) = select! {
            _ = cancel.cancelled() => return Ok(()),
            chunk = resp.chunk() => chunk.or(Err(NETWORK_ERROR))?
        } {
            file.write_all(&chunk).or(Err(NETWORK_ERROR))?;
        }
        current += 1;
        send_event(window, total, current, State::Downloading, 0);
    }
    Ok(())
}

fn move_changed_paths(
    changed_paths: &[&str],
    source_folder: &Path,
    target_folder: &Path,
    manifest: &mut Manifest,
) -> Result<(), &'static str> {
    for rel in changed_paths {
        let source = source_folder.join(rel);
        let target = target_folder.join(rel);

        fs::create_dir_all(target.parent().ok_or(FILE_ERROR)?).or(Err(FILE_ERROR))?;

        let was_added = manifest.added_files.iter().any(|item| item == rel);
        if was_added {
            if target.exists() {
                remove_file_force(&target).or(Err(FILE_ERROR))?;
            }
            // Clean up any stale backup from older buggy runs.
            let stale_backup = backup_name(&target);
            if stale_backup.exists() {
                let _ = remove_file_force(&stale_backup);
            }
            let _ = cleanup_legacy_backups(&target);
            record_manifest_entry(manifest, rel, false);
        } else {
            match backup_original(&target) {
                Ok(true) => record_manifest_entry(manifest, rel, true),
                Ok(false) => record_manifest_entry(manifest, rel, false),
                Err(_) => return Err(FILE_ERROR),
            }
        }

        rename_force(&source, &target).or(Err(FILE_ERROR))?;
    }
    Ok(())
}

fn record_manifest_entry(manifest: &mut Manifest, rel: &str, modified: bool) {
    if modified {
        manifest.added_files.retain(|item| item != rel);
        if !manifest.modified_files.iter().any(|item| item == rel) {
            manifest.modified_files.push(rel.to_string());
        }
    } else {
        manifest.modified_files.retain(|item| item != rel);
        if !manifest.added_files.iter().any(|item| item == rel) {
            manifest.added_files.push(rel.to_string());
        }
    }
}

fn restore_backup_files(root: &Path) -> io::Result<()> {
    let mut stack = vec![root.to_path_buf()];
    let mut originals = BTreeSet::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if let Some(original) = original_path_from_backup(&path) {
                originals.insert(original);
            }
        }
    }
    for original in originals {
        restore_preferred_backup(&original)?;
    }
    Ok(())
}

fn server_cache_root(root: &Path, server: &str) -> PathBuf {
    root.join(SERVER_CACHE_DIR)
        .join(sanitize_server_key(server))
}

fn read_trimmed_file(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn cached_server_etag(root: &Path, server: &str) -> Option<String> {
    let etag_path = server_cache_root(root, server).join(CLIENT_ETAG_FILE);
    read_trimmed_file(&etag_path)
}

fn is_cache_complete_for_server(root: &Path, server: &str, target_has_manifest: bool) -> bool {
    let cache_root = server_cache_root(root, server);
    if !cache_root.exists() {
        return false;
    }
    if cached_server_etag(root, server).is_none() {
        return false;
    }

    if !target_has_manifest {
        return true;
    }

    let manifest = Manifest::load(root, server);
    let mut required_files = BTreeSet::new();
    for rel in manifest
        .modified_files
        .iter()
        .chain(manifest.added_files.iter())
    {
        required_files.insert(rel.to_string());
    }

    required_files
        .iter()
        .all(|rel| cache_root.join(rel).exists())
}

fn cache_server_files(
    root: &Path,
    server: &str,
    changed_paths: &[&str],
) -> Result<(), &'static str> {
    let cache_root = server_cache_root(root, server);
    for rel in changed_paths {
        let source = root.join(rel);
        if !source.exists() {
            continue;
        }
        let target = cache_root.join(rel);
        fs::create_dir_all(target.parent().ok_or(FILE_ERROR)?).or(Err(FILE_ERROR))?;
        fs::copy(&source, &target).or(Err(FILE_ERROR))?;
    }
    Ok(())
}

fn write_server_etag(root: &Path, server: &str, etag: &str) -> Result<(), &'static str> {
    let trimmed = etag.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let target = server_cache_root(root, server).join(CLIENT_ETAG_FILE);
    fs::create_dir_all(target.parent().ok_or(FILE_ERROR)?).or(Err(FILE_ERROR))?;
    fs::write(target, trimmed).or(Err(FILE_ERROR))?;
    Ok(())
}
fn clear_server_patch_state(root: &Path, server: &str) {
    let cache_root = server_cache_root(root, server);
    if let Err(e) = fs::remove_dir_all(&cache_root) {
        if e.kind() != io::ErrorKind::NotFound {
            warn!(
                "failed to remove cache root {}: {}",
                cache_root.display(),
                e
            );
        }
    }

    let manifest_path = Manifest::path(root, server);
    if let Err(e) = remove_file_force(&manifest_path) {
        if e.kind() != io::ErrorKind::NotFound {
            warn!(
                "failed to remove manifest {}: {}",
                manifest_path.display(),
                e
            );
        }
    }
}
fn restore_cached_server_files(root: &Path, server: &str) -> Result<(), String> {
    let cache_root = server_cache_root(root, server);
    if !cache_root.exists() {
        return Err("patcher-file-error".to_string());
    }

    let manifest_path = Manifest::path(root, server);
    if !manifest_path.exists() {
        return Err("patcher-file-error".to_string());
    }

    let manifest = Manifest::load(root, server);
    let mut required_files = BTreeSet::new();
    for rel in manifest
        .modified_files
        .iter()
        .chain(manifest.added_files.iter())
    {
        required_files.insert(rel.to_string());
    }

    for rel in &manifest.modified_files {
        let target = root.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|_| "patcher-file-error")?;
        }
        backup_original(&target).map_err(|_| "patcher-file-error")?;
    }

    for rel in &manifest.added_files {
        let target = root.join(rel);
        if target.exists() {
            remove_file_force(&target).map_err(|_| "patcher-file-error")?;
        }
    }

    for rel in required_files {
        let source = cache_root.join(&rel);
        if !source.exists() {
            return Err("patcher-file-error".to_string());
        }
        let target = root.join(&rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|_| "patcher-file-error")?;
        }
        fs::copy(&source, &target).map_err(|_| "patcher-file-error")?;
    }

    Ok(())
}

/// Restore every file that was changed by `server`.
pub fn restore_server(root: &Path, server: &str) -> io::Result<()> {
    let manifest = Manifest::load(root, server);

    for rel in manifest.modified_files {
        let orig = root.join(&rel);
        let _ = restore_preferred_backup(&orig)?;
    }
    for rel in manifest.added_files {
        let _ = remove_file_force(&root.join(&rel));
    }

    restore_backup_files(root)?;
    Ok(())
}

fn reset_game_files_internal(root: &Path) -> Result<(), String> {
    let active_file = root.join(ACTIVE_SERVER_FILE);
    let server = fs::read_to_string(&active_file)
        .unwrap_or_default()
        .trim()
        .to_string();

    if server.is_empty() {
        return Ok(());
    }

    restore_server(root, &server).map_err(|e| format!("restore failed: {e}"))?;
    let _ = remove_file_force(&active_file);
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub active_server: Option<String>,
    pub active_has_manifest: bool,
    pub target_has_manifest: bool,
    pub target_has_cache: bool,
}

pub fn swap_info(root: &Path, target_server: &str) -> SwapInfo {
    let active_file = root.join(ACTIVE_SERVER_FILE);
    let active_server = fs::read_to_string(&active_file)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let active_has_manifest = active_server
        .as_deref()
        .is_some_and(|server| Manifest::path(root, server).exists());
    let target_has_manifest = Manifest::path(root, target_server).exists();
    let target_has_cache = is_cache_complete_for_server(root, target_server, target_has_manifest);

    SwapInfo {
        active_server,
        active_has_manifest,
        target_has_manifest,
        target_has_cache,
    }
}

pub fn swap_to_cached_server(root: &Path, target_server: &str) -> Result<(), String> {
    let info = swap_info(root, target_server);
    if info
        .active_server
        .as_deref()
        .is_some_and(|active| active == target_server)
    {
        return Ok(());
    }

    if !info.target_has_manifest || !info.target_has_cache {
        return Err("patcher-file-error".to_string());
    }

    reset_game_files_internal(root)?;
    restore_cached_server_files(root, target_server)?;

    let active_file = root.join(ACTIVE_SERVER_FILE);
    if let Some(dir) = active_file.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(&active_file, target_server);
    Ok(())
}

async fn patch_internal(
    window: &Window,
    client: reqwest::Client,
    patcher_url: String,
    patcher_resp: PatcherResponse,
    game_folder: &Path,
    patcher_folder: &Path,
    cancel: CancellationToken,
) -> Result<(), &'static str> {
    // Compare hashes before downloading only the changed paths.
    send_event(window, 0, 0, State::Checking, patcher_resp.queue_position);
    let changed_paths = get_changed_paths(&patcher_resp.content, game_folder)?;
    send_event(window, changed_paths.len(), 0, State::Downloading, 0);

    // Download the changed files into the patcher temp folder.
    download_changed_paths(
        window,
        &client,
        &patcher_url,
        &changed_paths,
        patcher_folder,
        cancel.clone(),
    )
    .await?;

    // Patch files in place and persist the manifest.
    send_event(window, 0, 0, State::Patching, 0);

    let mut manifest = Manifest::load(game_folder, &patcher_resp.server_name);
    move_changed_paths(&changed_paths, patcher_folder, game_folder, &mut manifest)?;
    cache_server_files(game_folder, &patcher_resp.server_name, &changed_paths)?;

    manifest
        .save(game_folder, &patcher_resp.server_name)
        .unwrap_or_else(|e| warn!("manifest save failed: {}", e));

    // Signal completion to the frontend.
    send_event(window, 0, 0, State::Done, 0);
    Ok(())
}

/// Main patch entrypoint for the hash-based patch flow.
pub async fn patch(
    window: Window,
    client: reqwest::Client,
    patcher_url: String,
    patcher_resp: PatcherResponse,
    game_folder: PathBuf,
    cancel: CancellationToken,
) {
    let active_file = game_folder.join(ACTIVE_SERVER_FILE);
    let target_server = patcher_resp.server_name.clone();
    let target_etag = patcher_resp.etag.trim().to_string();
    let prev_server = fs::read_to_string(&active_file)
        .unwrap_or_default()
        .trim()
        .to_string();

    let local_etag = cached_server_etag(&game_folder, &target_server).unwrap_or_default();

    let target_has_manifest = Manifest::path(&game_folder, &target_server).exists();
    let target_has_cache =
        is_cache_complete_for_server(&game_folder, &target_server, target_has_manifest);
    let same_server = prev_server == target_server;
    let etag_matches = !local_etag.is_empty() && local_etag == target_etag;

    if same_server && etag_matches {
        if let Some(dir) = active_file.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(&active_file, &target_server);
        if let Err(e) = write_server_etag(&game_folder, &target_server, &target_etag) {
            warn!("failed to update {}: {}", CLIENT_ETAG_FILE, e);
        }
        send_event(&window, 0, 0, State::Done, 0);
        return;
    }

    if !same_server && etag_matches && target_has_manifest && target_has_cache {
        if !prev_server.is_empty() {
            send_event(&window, 0, 0, State::Restoring, 0);
            if let Err(e) = restore_server(&game_folder, &prev_server) {
                warn!("failed to restore {prev_server}: {e}");
                send_error(&window, FILE_ERROR);
                return;
            }
        }

        send_event(&window, 0, 0, State::Restoring, 0);
        send_event(&window, 1, 0, State::Downloading, 0);
        if let Err(e) = restore_cached_server_files(&game_folder, &target_server) {
            warn!("failed to restore cached files for {target_server}: {e}");
            send_error(&window, FILE_ERROR);
            return;
        }
        send_event(&window, 1, 1, State::Downloading, 0);
        send_event(&window, 0, 0, State::Patching, 0);

        if let Some(dir) = active_file.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(&active_file, &target_server);
        if let Err(e) = write_server_etag(&game_folder, &target_server, &target_etag) {
            warn!("failed to update {}: {}", CLIENT_ETAG_FILE, e);
        }
        send_event(&window, 0, 0, State::Done, 0);
        return;
    }

    if !same_server && !prev_server.is_empty() {
        send_event(&window, 0, 0, State::Restoring, 0);
        if let Err(e) = restore_server(&game_folder, &prev_server) {
            warn!("failed to restore {prev_server}: {e}");
            send_error(&window, FILE_ERROR);
            return;
        }
    } else if same_server && !prev_server.is_empty() {
        send_event(&window, 0, 0, State::Restoring, 0);
        if let Err(e) = restore_server(&game_folder, &target_server) {
            warn!("failed to restore {target_server} before update: {e}");
            send_error(&window, FILE_ERROR);
            return;
        }
    }

    if !local_etag.is_empty() && local_etag != target_etag {
        clear_server_patch_state(&game_folder, &target_server);
    }

    let tmp_folder = game_folder.join("tmp");
    if let Err(e) = fs::create_dir_all(&tmp_folder) {
        warn!("error creating patcher dir: {}", e);
        send_error(&window, FILE_ERROR);
        return;
    }

    if let Err(e) = patch_internal(
        &window,
        client,
        patcher_url,
        patcher_resp.clone(),
        &game_folder,
        &tmp_folder,
        cancel,
    )
    .await
    {
        send_error(&window, e);
        if let Err(cleanup_err) = fs::remove_dir_all(&tmp_folder) {
            warn!("error deleting patcher dir: {}", cleanup_err);
        }
        return;
    }

    if let Err(e) = fs::remove_dir_all(&tmp_folder) {
        warn!("error deleting patcher dir: {}", e);
        send_error(&window, FILE_ERROR);
    }

    if let Some(dir) = active_file.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Err(e) = fs::write(&active_file, &target_server) {
        warn!("failed to write active-server file: {}", e);
    }

    if let Err(e) = write_server_etag(&game_folder, &target_server, &target_etag) {
        warn!("failed to cache {}: {}", CLIENT_ETAG_FILE, e);
    }
}
#[tauri::command]
pub async fn reset_game_files(game_folder: String) -> Result<(), String> {
    let root = PathBuf::from(&game_folder);
    reset_game_files_internal(&root)
}
