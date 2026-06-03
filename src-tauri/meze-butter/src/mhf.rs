use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use crate::friend_injection;
use crate::utils;
use crate::{CliFlags, Error, MhfConfig, MhfVersion, Result};
use windows::core::{s, PCSTR, PCWSTR};
use windows::Win32::Foundation::{FreeLibrary, FARPROC, HANDLE, HGLOBAL, HMODULE};
use windows::Win32::Graphics::Gdi::{
    AddFontResourceExW, CreateScalableFontResourceW, RemoveFontResourceExW, FR_PRIVATE,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::Win32::System::WindowsProgramming::GetPrivateProfileIntA;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows::Win32::UI::TextServices::HKL;

pub(crate) const INI_BASENAME: &[u8] = b"mhf.ini\0";
pub(crate) type RawEntry = unsafe extern "system" fn() -> isize;
const S7K_FONT_FOT_NAME: &str = "CreGothic_NHN M.fot";
const S7K_FONT_TTF_NAME: &str = "CreGothic_NHN M.ttf";

#[derive(Clone, Copy, Default)]
pub(crate) struct CmdData {
    pub cmd_flags_1: u32,
    pub cmd_flags_2: u32,
    pub cmd_dmm: u32,
}

pub(crate) struct LaunchContext {
    pub main_module: HMODULE,
    pub keyboard_layout: HKL,
    pub mutex_master: HANDLE,
    pub mutex_master_ready: HANDLE,
    pub mutex_master_name: String,
    pub mutex_master_ready_name: String,
    pub global_alloc: HGLOBAL,
    pub cmd_data: CmdData,
    pub ini_file: PCSTR,
    pub mhf_folder_name: String,
}

pub(crate) struct PreparedLayout {
    pub data_ptr: *mut usize,
    pub entry_proc: RawEntry,
    pub mhfo_module: HMODULE,
    pub friend_layout_dll_name: &'static str,
    pub cleanup: unsafe fn(*mut usize),
}

#[repr(C)]
pub(crate) struct GlobalData {
    _pad_0x0000: [u8; 0xa00],
    _pad_0x0a00: [u8; 0xc],
    notices_count: [u32; 0x4],
    _pad_0x0a10: [u8; 0x8],
    notices_flags: [u16; 0x4],
    notices: [[u8; 0x1000]; 0x4],
    _filter: [u8; 0x3000],
    _pad_0x4a2c: [u8; 0x1080],
    mez_event_id: u32,
    mez_start: u32,
    mez_end: u32,
    mez_solo_tickets: u32,
    mez_group_tickets: u32,
    mez_stalls: [u32; 8],
}

unsafe extern "C" fn mock_proc(_v: u32) -> u32 {
    0
}

unsafe extern "C" fn gg_proc() -> u32 {
    1
}

#[inline]
pub(crate) unsafe fn farproc_or(p: FARPROC) -> Result<RawEntry> {
    p.ok_or(Error::ProcNotFound)
}

#[inline]
pub(crate) unsafe fn boxed_zeroed<T>() -> Box<T> {
    Box::from_raw(Box::into_raw(Box::<T>::new(std::mem::zeroed())))
}

#[inline]
pub(crate) unsafe fn drop_boxed<T>(ptr: *mut usize) {
    drop(Box::from_raw(ptr as *mut T));
}

#[inline]
pub(crate) fn mock_proc_addr() -> usize {
    mock_proc as *const () as usize
}

#[inline]
pub(crate) fn gg_proc_addr() -> usize {
    gg_proc as *const () as usize
}

pub(crate) fn read_graphics_ver(ini_file: PCSTR) -> u32 {
    unsafe { GetPrivateProfileIntA(s!("VIDEO"), s!("GRAPHICS_VER"), 1, ini_file) }
}

pub(crate) fn resolve_layout_dll_name(graphics_ver: u32) -> (PCSTR, &'static str) {
    if graphics_ver == 1 {
        (s!("mhfo-hd.dll"), "mhfo-hd.dll")
    } else {
        (s!("mhfo.dll"), "mhfo.dll")
    }
}

pub(crate) fn load_layout_entry(dll_name: PCSTR) -> Result<(HMODULE, FARPROC, RawEntry)> {
    let module = unsafe { LoadLibraryA(dll_name) }.or(Err(Error::Dll))?;
    let proc = unsafe { GetProcAddress(module, s!("mhDLL_Main")) };
    let entry = unsafe { farproc_or(proc)? };
    Ok((module, proc, entry))
}

fn path_to_wide_null(path: &Path) -> Vec<u16> {
    let mut wide: Vec<u16> = path.to_string_lossy().encode_utf16().collect();
    wide.push(0);
    wide
}

fn register_font(path: &Path) {
    if !path.exists() {
        return;
    }
    let wide = path_to_wide_null(path);
    unsafe {
        let _ = AddFontResourceExW(PCWSTR(wide.as_ptr()), FR_PRIVATE, None);
    }
}

fn resolve_s7k_fot_path(mhf_folder: &Path) -> PathBuf {
    mhf_folder.join(S7K_FONT_FOT_NAME)
}

fn resolve_s7k_ttf_path(mhf_folder: &Path) -> PathBuf {
    mhf_folder
        .join("Mezeporta")
        .join("fonts")
        .join(S7K_FONT_TTF_NAME)
}

fn path_to_ascii_bytes(path: &Path) -> Vec<u8> {
    path.to_string_lossy().as_bytes().to_vec()
}

fn file_contains_bytes(path: &Path, needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }

    match std::fs::read(path) {
        Ok(bytes) => bytes.windows(needle.len()).any(|window| window == needle),
        Err(_) => false,
    }
}

fn s7k_fot_needs_refresh(fot_path: &Path, ttf_path: &Path) -> Result<bool> {
    if !fot_path.exists() {
        return Ok(true);
    }

    let fot_meta = std::fs::metadata(fot_path).map_err(|_| Error::FontBootstrap)?;
    let ttf_meta = std::fs::metadata(ttf_path).map_err(|_| Error::FontMissing)?;

    let ttf_newer = match (fot_meta.modified(), ttf_meta.modified()) {
        (Ok(fot_mtime), Ok(ttf_mtime)) => ttf_mtime > fot_mtime,
        _ => false,
    };
    if ttf_newer {
        return Ok(true);
    }

    Ok(!file_contains_bytes(fot_path, &path_to_ascii_bytes(ttf_path)))
}

fn ensure_s7k_font_resource(mhf_folder: &Path) -> Result<()> {
    let fot_path = resolve_s7k_fot_path(mhf_folder);
    let ttf_path = resolve_s7k_ttf_path(mhf_folder);

    if !ttf_path.exists() {
        return Err(Error::FontMissing);
    }

    if s7k_fot_needs_refresh(&fot_path, &ttf_path)? {
        if fot_path.exists() {
            std::fs::remove_file(&fot_path).map_err(|_| Error::FontBootstrap)?;
        }

        let fot_wide = path_to_wide_null(&fot_path);
        let ttf_wide = path_to_wide_null(&ttf_path);
        unsafe {
            CreateScalableFontResourceW(
                0,
                PCWSTR(fot_wide.as_ptr()),
                PCWSTR(ttf_wide.as_ptr()),
                PCWSTR::null(),
            )
            .map_err(|_| Error::FontBootstrap)?;
        }
    }

    let fot_wide = path_to_wide_null(&fot_path);
    unsafe {
        let _ = RemoveFontResourceExW(PCWSTR(fot_wide.as_ptr()), FR_PRIVATE.0, None);
        if AddFontResourceExW(PCWSTR(fot_wide.as_ptr()), FR_PRIVATE, None) == 0 {
            return Err(Error::FontBootstrap);
        }
    }

    Ok(())
}

fn resolve_launcher_font_path(
    mhf_folder: &Path,
    version: MhfVersion,
    config_font_path: Option<&PathBuf>,
) -> Option<PathBuf> {
    if let Some(path) = config_font_path {
        if path.exists() {
            return Some(path.clone());
        }
    }

    let font_name = match version {
        MhfVersion::Z2T => "dft_0.ttc",
        _ => "MS Gothic.ttf",
    };

    let primary = mhf_folder.join("Mezeporta").join("fonts").join(font_name);
    if primary.exists() {
        return Some(primary);
    }

    let fallback = mhf_folder.join("fonts").join(font_name);
    if fallback.exists() {
        return Some(fallback);
    }

    None
}
fn preload_controller_dlls(mhf_folder: &Path) {
    for dll_name in ["xinput1_3.dll", "dinput8.dll", "dinput.dll"] {
        let dll_path = mhf_folder.join(dll_name);
        if !dll_path.exists() {
            continue;
        }
        let Ok(dll_path_cstr) = std::ffi::CString::new(dll_path.to_string_lossy().as_bytes()) else {
            continue;
        };
        unsafe {
            let _ = LoadLibraryA(PCSTR(dll_path_cstr.as_ptr() as _));
        }
    }
}

fn find_ini_file(folder: &Path) -> Result<std::ffi::CString> {
    use std::ffi::CString;

    let ini_path = folder.join("mhf.ini");
    if !ini_path.exists() {
        return Err(Error::GamePath);
    }

    CString::new(ini_path.to_str().ok_or(Error::IniMissing)?).map_err(|_| Error::IniMissing)
}

pub(crate) fn init_global_alloc(global_alloc: HGLOBAL, cfg: &MhfConfig) {
    let p = unsafe { GlobalLock(global_alloc) };
    unsafe { p.write_bytes(0, 0x8ae0) };

    unsafe {
        let g = &mut *(p as *mut GlobalData);
        for (i, n) in cfg.notices.iter().enumerate() {
            g.notices_count[i] = n.data.len() as u32;
            g.notices_flags[i] = n.flags;
            utils::bufcopy(&mut g.notices[i], n.data.as_bytes());
        }

        g.mez_event_id = cfg.mez_event_id;
        g.mez_start = cfg.mez_start;
        g.mez_end = cfg.mez_end;
        g.mez_solo_tickets = cfg.mez_solo_tickets;
        g.mez_group_tickets = cfg.mez_group_tickets;

        for (i, stall) in cfg.mez_stalls.iter().enumerate().take(8) {
            g.mez_stalls[i] = *stall as u32;
        }
        for i in cfg.mez_stalls.len()..8 {
            g.mez_stalls[i] = 0;
        }
    }

    unsafe {
        let _ = GlobalUnlock(global_alloc)
            .or_else(|e| if e.code().0 == 0 { Ok(()) } else { Err(e) });
    }
}

pub(crate) fn init_cli(mhf_flags: &[CliFlags]) -> CmdData {
    let mut cmd_data = CmdData::default();
    for flag in mhf_flags {
        match flag {
            CliFlags::Selfup => cmd_data.cmd_flags_1 = 1,
            CliFlags::Restat => cmd_data.cmd_flags_1 = 2,
            CliFlags::Autolc => cmd_data.cmd_flags_1 = 3,
            CliFlags::Hanres => cmd_data.cmd_flags_1 = 4,
            CliFlags::DmmBoot => {
                cmd_data.cmd_flags_1 = 5;
                cmd_data.cmd_dmm = 1;
            }
            CliFlags::DmmSelfup => {
                cmd_data.cmd_flags_1 = 6;
                cmd_data.cmd_dmm = 1;
            }
            CliFlags::DmmAutolc => {
                cmd_data.cmd_flags_1 = 7;
                cmd_data.cmd_dmm = 1;
            }
            CliFlags::DmmReboot => {
                cmd_data.cmd_flags_1 = 8;
                cmd_data.cmd_dmm = 1;
            }
            CliFlags::Npge => {
                cmd_data.cmd_flags_1 = 9;
                cmd_data.cmd_flags_2 |= 6;
            }
            CliFlags::NpMhfoTest => cmd_data.cmd_flags_2 |= 4,
        }
    }
    cmd_data
}

#[allow(clippy::too_many_lines)]
pub fn run_mhf(config: MhfConfig) -> Result<isize> {
    let mhf_folder = match &config.mhf_folder {
        Some(dir) => {
            std::env::set_current_dir(dir).or(Err(Error::GamePath))?;
            dir.clone()
        }
        None => std::env::current_dir().or(Err(Error::GamePath))?,
    };

    let mut mhf_folder_name = mhf_folder.to_str().ok_or(Error::GamePath)?.to_owned();
    if !mhf_folder_name.ends_with(['/', '\\']) {
        mhf_folder_name.push('/');
    }

    if config.version == MhfVersion::S7K {
        ensure_s7k_font_resource(&mhf_folder)?;
    } else if let Some(font_path) = resolve_launcher_font_path(&mhf_folder, config.version, config.font_path.as_ref()) {
        register_font(&font_path);
    }

    let main_module = unsafe { GetModuleHandleA(None) }.map_err(|_| Error::Dll)?;
    let keyboard_layout = unsafe { GetKeyboardLayout(0) };

    let mutex_master_name = utils::get_mutex_name(config.mutex_version, "MHF_MASTER");
    let mutex_master = utils::get_or_create_mutex(&mutex_master_name)?;
    let mutex_master_ready_name = utils::get_mutex_name(config.mutex_version, "MHF_MASTER_READY");
    let mutex_master_ready = utils::get_or_create_mutex(&mutex_master_ready_name)?;

    if let Some(fallback_version) = config.mutex_fallback_version {
        if fallback_version != config.mutex_version {
            let fallback_master_name = utils::get_mutex_name(fallback_version, "MHF_MASTER");
            let fallback_ready_name = utils::get_mutex_name(fallback_version, "MHF_MASTER_READY");
            let _ = utils::get_or_create_mutex(&fallback_master_name);
            let _ = utils::get_or_create_mutex(&fallback_ready_name);
        }
    }

    let global_alloc = utils::create_global_alloc()?;
    let cmd_data = config
        .mhf_flags
        .as_deref()
        .map(init_cli)
        .unwrap_or_default();

    let ini_path_cstr = find_ini_file(&mhf_folder)?;
    let ini_file = PCSTR(ini_path_cstr.as_ptr() as _);

    let launch_ctx = LaunchContext {
        main_module,
        keyboard_layout,
        mutex_master,
        mutex_master_ready,
        mutex_master_name,
        mutex_master_ready_name,
        global_alloc,
        cmd_data,
        ini_file,
        mhf_folder_name,
    };

    let selected_friend_signature = config
        .friend_signature
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("none"));

    let prepared = match config.version {
        MhfVersion::S6 => crate::layouts::layout_s6::prepare(&launch_ctx, &config)?,
        MhfVersion::S7K => crate::layouts::layout_s7k::prepare(&launch_ctx, &config)?,
        MhfVersion::F4 => crate::layouts::layout_f4::prepare(&launch_ctx, &config)?,
        MhfVersion::F5 => crate::layouts::layout_f5::prepare(&launch_ctx, &config)?,
        MhfVersion::G1 => crate::layouts::layout_g1::prepare(&launch_ctx, &config)?,
        MhfVersion::G2 => crate::layouts::layout_g2::prepare(&launch_ctx, &config)?,
        MhfVersion::G3 => crate::layouts::layout_g3::prepare(&launch_ctx, &config)?,
        MhfVersion::G3_1 => crate::layouts::layout_g3_1::prepare(&launch_ctx, &config)?,
        MhfVersion::G3_2 => crate::layouts::layout_g3_2::prepare(&launch_ctx, &config)?,
        MhfVersion::GG => crate::layouts::layout_gg::prepare(&launch_ctx, &config)?,
        MhfVersion::G5 => crate::layouts::layout_g5::prepare(&launch_ctx, &config)?,
        MhfVersion::G5_1 => crate::layouts::layout_g5_1::prepare(&launch_ctx, &config)?,
        MhfVersion::G5_2 => crate::layouts::layout_g5_2::prepare(&launch_ctx, &config)?,
        MhfVersion::G6 => crate::layouts::layout_g6::prepare(&launch_ctx, &config)?,
        MhfVersion::G7 => crate::layouts::layout_g7::prepare(&launch_ctx, &config)?,
        MhfVersion::G9_1 => crate::layouts::layout_g9_1::prepare(&launch_ctx, &config)?,
        MhfVersion::G10_1 => crate::layouts::layout_g10_1::prepare(&launch_ctx, &config)?,
        MhfVersion::Z1 => crate::layouts::layout_z1::prepare(&launch_ctx, &config)?,
        MhfVersion::Z2 => crate::layouts::layout_z2::prepare(&launch_ctx, &config)?,
        MhfVersion::Z2T => crate::layouts::layout_z2t::prepare(&launch_ctx, &config)?,
        MhfVersion::ZZ => crate::layouts::layout_zz::prepare(&launch_ctx, &config)?,
    };

    let proc_addr = prepared.entry_proc as usize;
    let data_ptr_val = prepared.data_ptr as usize;

    let friends_copy: Vec<_> = config
        .friends
        .iter()
        .cloned()
        .filter(|f| f.cid == config.char_id)
        .collect();

    let game_running = Arc::new(AtomicBool::new(true));
    let game_running_for_game = Arc::clone(&game_running);
    let game_handle = thread::spawn(move || {
        let entry: unsafe extern "C" fn(*const usize) -> isize =
            unsafe { std::mem::transmute(proc_addr) };
        let result = unsafe { entry(data_ptr_val as *const usize) };
        game_running_for_game.store(false, Ordering::Relaxed);
        result
    });

    let inject_signature = selected_friend_signature.map(ToOwned::to_owned);
    let friends_for_inject = friends_copy.clone();
    let game_running_for_inject = Arc::clone(&game_running);
    let friend_layout_dll_name = prepared.friend_layout_dll_name;
    let game_version = config.version;
    let inj_handle = thread::spawn(move || {
        while game_running_for_inject.load(Ordering::Relaxed) {
            if friend_injection::maybe_inject_friends(
                game_version,
                friend_layout_dll_name,
                inject_signature.as_deref(),
                &friends_for_inject,
            ) {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    let preload_controller = config.preload_controller_dlls;
    let mhf_folder_for_controller = mhf_folder.clone();
    let game_running_for_controller = Arc::clone(&game_running);
    let controller_handle = thread::spawn(move || {
        if !preload_controller {
            return;
        }
        thread::sleep(Duration::from_millis(1200));
        if game_running_for_controller.load(Ordering::Relaxed) {
            preload_controller_dlls(&mhf_folder_for_controller);
        }
    });

    let result = game_handle.join().map_err(|_| Error::ThreadJoin)?;
    inj_handle.join().map_err(|_| Error::ThreadJoin)?;
    controller_handle.join().map_err(|_| Error::ThreadJoin)?;

    unsafe { FreeLibrary(prepared.mhfo_module) }.or(Err(Error::Dll))?;
    utils::release_global_alloc(global_alloc)?;
    unsafe { (prepared.cleanup)(prepared.data_ptr) };

    Ok(result)
}



