use crate::mhf::{
    boxed_zeroed, drop_boxed, gg_proc_addr, init_global_alloc, load_layout_entry,
    mock_proc_addr, resolve_layout_dll_name, LaunchContext,
    PreparedLayout, INI_BASENAME,
};
use crate::utils::bufcopy;
use crate::{Error, MhfConfig, Result};
use windows::core::s;
use windows::Win32::Foundation::{FARPROC, HANDLE, HGLOBAL, HMODULE};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalUnlock, GLOBAL_ALLOC_FLAGS, VirtualProtect,
    PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
};
use windows::Win32::System::WindowsProgramming::{
    GetPrivateProfileIntA, GetPrivateProfileStringA,
};
use windows::Win32::UI::TextServices::HKL;

const S7K_FULLSCREEN_MODE_OFFSET: usize = 0x1D24;
const S7K_FULLSCREEN_RESOLUTION_W_OFFSET: usize = 0x1D30;
const S7K_FULLSCREEN_RESOLUTION_H_OFFSET: usize = 0x1D34;
const S7K_WINDOW_RESOLUTION_W_OFFSET: usize = 0x1D28;
const S7K_WINDOW_RESOLUTION_H_OFFSET: usize = 0x1D2C;
const S7K_TEXTURE_DXT_USE_OFFSET: usize = 0x1D3C;
const S7K_NOW_MONITOR_WH_OFFSET: usize = 0x1D40;
const S7K_SOUND_NOTUSE_OFFSET: usize = 0x1D44;
const S7K_SOUND_VOLUME_OFFSET: usize = 0x1D48;
const S7K_SOUND_VOLUME_INACTIVITY_OFFSET: usize = 0x1D4C;
const S7K_SOUND_VOLUME_MINIMIZE_OFFSET: usize = 0x1D50;
const S7K_SOUND_FREQUENCY_OFFSET: usize = 0x1D54;
const S7K_SOUND_BUFFERNUM_OFFSET: usize = 0x1D58;
const S7K_FONT_QUALITY_OFFSET: usize = 0x1D5C;
const S7K_FONT_WEIGHT_OFFSET: usize = 0x1D60;
const S7K_FONT_NAME_OFFSET: usize = 0x1D64;
const S7K_FONT_NAME_LEN: usize = 0x64;
const S7K_CLOGDIS_OFFSET: usize = 0x1DC8;
const S7K_PROXY_USE_OFFSET: usize = 0x1DCC;
const S7K_PROXY_IE_OFFSET: usize = 0x1DD0;
const S7K_PROXY_SET_OFFSET: usize = 0x1DD4;
const S7K_PROXY_ADDR_OFFSET: usize = 0x1DD8;
const S7K_PROXY_ADDR_LEN: usize = 0x28;
const S7K_PROXY_PORT_OFFSET: usize = 0x1E00;
const S7K_SERVER_SEL_OFFSET: usize = 0x1E04;
const S7K_FALLBACK_IP_OFFSET: usize = 0x839360;
const S7K_FALLBACK_IP_LEN: usize = 0x10;
const S7K_FONT_STARTUP_PATCH_OFFSET: usize = 0x630BBF;
const S7K_FONT_STARTUP_PATCH_LEN: usize = 0x36;
const S7K_FONT_SHUTDOWN_PATCH_OFFSET: usize = 0x630E00;
const S7K_FONT_SHUTDOWN_PATCH_LEN: usize = 0x1E;
const S7K_LATE_STATE_PAD_LEN: usize = 0x3D4;
const S7K_LATE_STATE_BLOCK_LEN: usize = 0x1000;
const S7K_GAME_GLOBAL_ALLOC_SIZE: usize = 0x20_000;
const S7K_RUNTIME_TAIL_LEN: usize = 0x9000;
const S7K_FONT_FAMILY_NAME: &[u8] = b"CreGothic_NHN M\0";
#[derive(Debug)]
#[repr(C)]
pub(crate) struct Data {
    main_module: HMODULE,
    _pad_44717c: [u8; 0x8],
    cmd_flags_1: u32,
    cmd_flags_2: u32,
    path1: [u8; 0x400],
    path2: [u8; 0x400],
    user_name: [u8; 0x800],
    user_password: [u8; 0x800],
    cmd_number: u32,
    cmd_netfcup: u32,
    s7k_url_list_mode: u32,
    s7k_keyconfig_flags: u32,
    mutex_master: HANDLE,
    mutex_master_ready: HANDLE,
    mutex_master_name: [u8; 0x40],
    ini_file: [u8; 0x40],
    proc_1: usize,
    proc_2: usize,
    proc_3: usize,
    _pad_448a30: [u8; 0x4],
    s7k_state_18bc: u32,
    s7k_state_18c0: u32,
    selected_char_id_1: u32,
    selected_char_id_2: u32,
    user_token_id: u32,
    user_token: [u8; 0x10],
    _pad_448a58: [u8; 0x8],
    server_current_ts: u32,
    fixed_448a64_0x0: u32,
    _pad_448a68: [u8; 0x200],
    remote_addr: [u8; 0x100],
    remote_host: [u8; 0x100],
    remote_patch_count: u32,
    server_entrance_count: u32,
    selected_char_status: u32,
    user_rights: u32,
    selected_char_hr: u32,
    season_text_global_alloc: HGLOBAL,
    state_1d08: u32,
    _pad_1d0c_to_1d2b: [u8; 0x20],
    preset_level: u32,
    custom: u32,
    fullscreen_mode: u32,
    window_resolution_w: u32,
    window_resolution_h: u32,
    fullscreen_resolution_w: u32,
    fullscreen_resolution_h: u32,
    disp_max_char: u32,
    texture_dxt_use: u32,
    now_monitor_wh: u32,
    sound_notuse: u32,
    sound_volume: u32,
    sound_volume_inactivity: u32,
    sound_volume_minimize: u32,
    sound_frequency: u32,
    sound_buffernum: u32,
    language: u32,
    font_quality: u32,
    font_weight: u32,
    font_name: [u8; 0x60],
    unk_setting_448f94: u32,
    drawskip: u32,
    clogdis: u32,
    proxy_use: u32,
    proxy_ie: u32,
    proxy_set: u32,
    proxy_addr: [u8; 0x28],
    proxy_port: u32,
    server_sel: u32,
    inner_ptr_1_4491a8: usize,
    _pad_448ffc: [u8; 0x40],
    _pad_44903c: [u8; 0x40],
    alt_ip_address: [u8; 0xC0],
    _pad_44913c: [u8; 0x40],
    server_expiry_ts: u32,
    remote_16e: u32,
    fixed_449184_0x1: u32,
    _pad_449188: [u8; 0x8],
    data_ptr: usize,
    keyboard_layout: HKL,
    inner_3: (),
    _pad_449198: [u8; 0x10],
    inner_1: (),
    _pad_4491a8: [u8; 0x4],
    fixed_4491ac_0x10: u32,
    inner_ptr_2_4491d4: usize,
    _pad_4491b4: [u8; 4],
    fixed_4491b8_0x10: u32,
    inner_ptr_3_449198: usize,
    proc_4: usize,
    _pad_4491c4: [u8; 0x4],
    proc_5: usize,
    _pad_4491cc: [u8; 0x8],
    inner_2: (),
    _pad_4491d4: [u8; 0x14],
    mhfo_module: HMODULE,
    _pad_4491ec: [u8; 0x4],
    _pad_4491f0: [u8; 0x520],
    mutex_master_ready_name: [u8; 0x100],
    _pad_449810: [u8; 0x414],
    mhddl_main: FARPROC,
    _pad_s7k_2a50_to_2e24: [u8; S7K_LATE_STATE_PAD_LEN],
    late_state_2e24: [u8; S7K_LATE_STATE_BLOCK_LEN],
    late_state_ptr_3e24: usize,
    _pad_s7_runtime: [u8; S7K_RUNTIME_TAIL_LEN],
}

const _: [(); 0x1D08] = [(); std::mem::offset_of!(Data, state_1d08)];
const _: [(); 0x2E24] = [(); std::mem::offset_of!(Data, late_state_2e24)];
const _: [(); 0x3E24] = [(); std::mem::offset_of!(Data, late_state_ptr_3e24)];

fn init_ptrs(data: &mut Box<Data>) {
    data.data_ptr = Box::as_ref(data) as *const _ as usize;
    data.inner_ptr_1_4491a8 = &data.inner_1 as *const _ as usize;
    data.inner_ptr_2_4491d4 = &data.inner_2 as *const _ as usize;
    data.inner_ptr_3_449198 = &data.inner_3 as *const _ as usize;
}

fn write_s7k_u32(data: &mut Box<Data>, offset: usize, value: u32) {
    unsafe {
        (Box::as_mut(data) as *mut Data)
            .cast::<u8>()
            .add(offset)
            .cast::<u32>()
            .write_unaligned(value);
    }
}

fn write_s7k_bytes(data: &mut Box<Data>, offset: usize, len: usize, value: &[u8]) {
    unsafe {
        let dst = std::slice::from_raw_parts_mut(
            (Box::as_mut(data) as *mut Data).cast::<u8>().add(offset),
            len,
        );
        dst.fill(0);
        bufcopy(dst, value);
    }
}

fn mirror_s7k_settings(data: &mut Box<Data>) {
    let font_name = data.font_name;
    let proxy_addr = data.proxy_addr;
    write_s7k_u32(data, S7K_FULLSCREEN_MODE_OFFSET, data.fullscreen_mode);
    write_s7k_u32(
        data,
        S7K_FULLSCREEN_RESOLUTION_W_OFFSET,
        data.fullscreen_resolution_w,
    );
    write_s7k_u32(
        data,
        S7K_FULLSCREEN_RESOLUTION_H_OFFSET,
        data.fullscreen_resolution_h,
    );
    write_s7k_u32(
        data,
        S7K_WINDOW_RESOLUTION_W_OFFSET,
        data.window_resolution_w,
    );
    write_s7k_u32(
        data,
        S7K_WINDOW_RESOLUTION_H_OFFSET,
        data.window_resolution_h,
    );
    write_s7k_u32(data, S7K_TEXTURE_DXT_USE_OFFSET, data.texture_dxt_use);
    write_s7k_u32(data, S7K_NOW_MONITOR_WH_OFFSET, data.now_monitor_wh);
    write_s7k_u32(data, S7K_SOUND_NOTUSE_OFFSET, data.sound_notuse);
    write_s7k_u32(data, S7K_SOUND_VOLUME_OFFSET, data.sound_volume);
    write_s7k_u32(
        data,
        S7K_SOUND_VOLUME_INACTIVITY_OFFSET,
        data.sound_volume_inactivity,
    );
    write_s7k_u32(
        data,
        S7K_SOUND_VOLUME_MINIMIZE_OFFSET,
        data.sound_volume_minimize,
    );
    write_s7k_u32(data, S7K_SOUND_FREQUENCY_OFFSET, data.sound_frequency);
    write_s7k_u32(data, S7K_SOUND_BUFFERNUM_OFFSET, data.sound_buffernum);
    write_s7k_u32(data, S7K_FONT_QUALITY_OFFSET, data.font_quality);
    write_s7k_u32(data, S7K_FONT_WEIGHT_OFFSET, data.font_weight);
    write_s7k_bytes(data, S7K_FONT_NAME_OFFSET, S7K_FONT_NAME_LEN, &font_name);
    write_s7k_u32(data, S7K_CLOGDIS_OFFSET, data.clogdis);
    write_s7k_u32(data, S7K_PROXY_USE_OFFSET, data.proxy_use);
    write_s7k_u32(data, S7K_PROXY_IE_OFFSET, data.proxy_ie);
    write_s7k_u32(data, S7K_PROXY_SET_OFFSET, data.proxy_set);
    write_s7k_bytes(data, S7K_PROXY_ADDR_OFFSET, S7K_PROXY_ADDR_LEN, &proxy_addr);
    write_s7k_u32(data, S7K_PROXY_PORT_OFFSET, data.proxy_port);
    write_s7k_u32(data, S7K_SERVER_SEL_OFFSET, data.server_sel);
}

unsafe fn patch_bytes(module: HMODULE, offset: usize, bytes: &[u8], target_len: usize) {
    if module.0 == 0 || bytes.len() > target_len {
        return;
    }

    let target = (module.0 as *mut u8).add(offset);
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    if VirtualProtect(target.cast(), target_len, PAGE_READWRITE, &mut old_protect).is_err() {
        return;
    }

    let dst = std::slice::from_raw_parts_mut(target, target_len);
    dst.fill(0);
    bufcopy(dst, bytes);

    let mut restored = PAGE_PROTECTION_FLAGS(0);
    let _ = VirtualProtect(target.cast(), target_len, old_protect, &mut restored);
}

fn s7k_font_startup_patch() -> [u8; S7K_FONT_STARTUP_PATCH_LEN] {
    let mut patch = [0x90; S7K_FONT_STARTUP_PATCH_LEN];
    patch[..5].copy_from_slice(&[0xB8, 0x01, 0x00, 0x00, 0x00]);
    patch
}

unsafe fn patch_s7k_fallback_host(module: HMODULE, host: &[u8]) {
    if host.len() >= S7K_FALLBACK_IP_LEN {
        return;
    }

    patch_bytes(module, S7K_FALLBACK_IP_OFFSET, host, S7K_FALLBACK_IP_LEN);
}

unsafe fn patch_s7k_font_bootstrap(module: HMODULE) {
    let startup_patch = s7k_font_startup_patch();
    patch_bytes(
        module,
        S7K_FONT_STARTUP_PATCH_OFFSET,
        &startup_patch,
        S7K_FONT_STARTUP_PATCH_LEN,
    );
    patch_bytes(
        module,
        S7K_FONT_SHUTDOWN_PATCH_OFFSET,
        &[0x90; S7K_FONT_SHUTDOWN_PATCH_LEN],
        S7K_FONT_SHUTDOWN_PATCH_LEN,
    );
}
fn create_s7k_game_global_alloc() -> Result<HGLOBAL> {
    unsafe { GlobalAlloc(GLOBAL_ALLOC_FLAGS(0x42), S7K_GAME_GLOBAL_ALLOC_SIZE) }
        .or(Err(Error::GlobalAlloc))
}

fn init_s7k_game_global_alloc(global_alloc: HGLOBAL, cfg: &MhfConfig) {
    let p = unsafe { GlobalLock(global_alloc) };
    unsafe { p.write_bytes(0, S7K_GAME_GLOBAL_ALLOC_SIZE) };
    unsafe {
        let _ = GlobalUnlock(global_alloc)
            .or_else(|e| if e.code().0 == 0 { Ok(()) } else { Err(e) });
    }
    init_global_alloc(global_alloc, cfg);
}

fn init_data(data: &mut Box<Data>, ctx: &LaunchContext, config: &MhfConfig) -> Result<()> {
    data.main_module = ctx.main_module;
    data.mutex_master = ctx.mutex_master;
    data.mutex_master_ready = ctx.mutex_master_ready;
    data.keyboard_layout = ctx.keyboard_layout;
    data.fixed_448a64_0x0 = 0x0;
    data.fixed_4491ac_0x10 = 0x10;
    data.fixed_4491b8_0x10 = 0x10;
    data.proc_1 = mock_proc_addr();
    data.proc_2 = gg_proc_addr();
    data.proc_3 = mock_proc_addr();
    data.proc_4 = mock_proc_addr();
    data.proc_5 = mock_proc_addr();

    unsafe {
        data.preset_level = GetPrivateProfileIntA(s!("SET"), s!("PRESET_LEVEL"), 0, ctx.ini_file);
        data.custom = GetPrivateProfileIntA(s!("SET"), s!("CUSTOM"), 1, ctx.ini_file);
        data.fullscreen_mode =
            GetPrivateProfileIntA(s!("SCREEN"), s!("FULLSCREEN_MODE"), 1, ctx.ini_file);
        data.window_resolution_w =
            GetPrivateProfileIntA(s!("SCREEN"), s!("WINDOW_RESOLUTION_W"), 1920, ctx.ini_file);
        data.window_resolution_h =
            GetPrivateProfileIntA(s!("SCREEN"), s!("WINDOW_RESOLUTION_H"), 1080, ctx.ini_file);
        data.fullscreen_resolution_w = GetPrivateProfileIntA(
            s!("SCREEN"),
            s!("FULLSCREEN_RESOLUTION_W"),
            1920,
            ctx.ini_file,
        );
        data.fullscreen_resolution_h = GetPrivateProfileIntA(
            s!("SCREEN"),
            s!("FULLSCREEN_RESOLUTION_H"),
            1080,
            ctx.ini_file,
        );
        data.disp_max_char =
            GetPrivateProfileIntA(s!("VIDEO"), s!("DISP_MAX_CHAR"), 100, ctx.ini_file);
        data.texture_dxt_use =
            GetPrivateProfileIntA(s!("VIDEO"), s!("TEXTURE_DXT_USE"), 0, ctx.ini_file);
        data.now_monitor_wh =
            GetPrivateProfileIntA(s!("VIDEO"), s!("NOW_MONITOR_WH"), 0, ctx.ini_file);
        data.sound_notuse =
            GetPrivateProfileIntA(s!("SOUND"), s!("SOUND_NOTUSE"), 0, ctx.ini_file);
        data.sound_volume =
            GetPrivateProfileIntA(s!("SOUND"), s!("SOUND_VOLUME"), 0, ctx.ini_file);
        data.sound_volume_inactivity = GetPrivateProfileIntA(
            s!("SOUND"),
            s!("SOUND_VOLUME_INACTIVITY"),
            0,
            ctx.ini_file,
        );
        data.sound_volume_minimize = GetPrivateProfileIntA(
            s!("SOUND"),
            s!("SOUND_VOLUME_MINIMIZE"),
            0,
            ctx.ini_file,
        );
        data.sound_frequency =
            GetPrivateProfileIntA(s!("SOUND"), s!("SOUND_FREQUENCY"), 48000, ctx.ini_file);
        data.sound_buffernum =
            GetPrivateProfileIntA(s!("SOUND"), s!("SOUND_BUFFERNUM"), 2048, ctx.ini_file);
        data.language =
            GetPrivateProfileIntA(s!("LOCALIZATION"), s!("LANGUAGE"), 0, ctx.ini_file);
        data.font_quality = GetPrivateProfileIntA(s!("FONT"), s!("QUALITY"), 4, ctx.ini_file);
        data.font_weight = GetPrivateProfileIntA(s!("FONT"), s!("WEIGHT"), 0x2bc, ctx.ini_file);
        GetPrivateProfileStringA(
            s!("FONT"),
            s!("NAME"),
            s!("CreGothic_NHN M"),
            Some(&mut data.font_name),
            ctx.ini_file,
        );
        if config.enable_font_registration && data.font_name[0] == 0 {
            bufcopy(&mut data.font_name, S7K_FONT_FAMILY_NAME);
        }
        data.drawskip = GetPrivateProfileIntA(s!("OPTION"), s!("DRAWSKIP"), 1, ctx.ini_file);
        data.clogdis = GetPrivateProfileIntA(s!("OPTION"), s!("CLOGDIS"), 0, ctx.ini_file);
        data.proxy_use = GetPrivateProfileIntA(s!("LAUNCH"), s!("PROXY_USE"), 0, ctx.ini_file);
        data.proxy_ie = GetPrivateProfileIntA(s!("LAUNCH"), s!("PROXY_IE"), 0, ctx.ini_file);
        data.proxy_set = GetPrivateProfileIntA(s!("LAUNCH"), s!("PROXY_SET"), 1, ctx.ini_file);
        GetPrivateProfileStringA(
            s!("LAUNCH"),
            s!("PROXY_ADDR"),
            s!("127.0.0.1"),
            Some(&mut data.proxy_addr),
            ctx.ini_file,
        );
        data.proxy_port =
            GetPrivateProfileIntA(s!("LAUNCH"), s!("PROXY_PORT"), 8888, ctx.ini_file);
        data.server_sel = GetPrivateProfileIntA(s!("LAUNCH"), s!("SERVER_SEL"), 1, ctx.ini_file);
    }

    init_global_alloc(ctx.global_alloc, config);

    data.selected_char_id_1 = config.char_id;
    data.selected_char_id_2 = config.char_id;
    data.season_text_global_alloc = create_s7k_game_global_alloc()?;
    init_s7k_game_global_alloc(data.season_text_global_alloc, config);
    data.selected_char_hr = config.char_hr;
    data.selected_char_status = if config.char_new { 2 } else { 0 };

    bufcopy(&mut data.user_name, config.user_name.as_bytes());
    bufcopy(&mut data.user_password, config.user_password.as_bytes());
    data.user_token_id = config.user_token_id;
    bufcopy(&mut data.user_token, config.user_token.as_bytes());
    data.user_rights = config.user_rights;

    data.server_entrance_count = config.entrance_count;
    data.server_current_ts = config.current_ts;

    data.cmd_flags_1 = ctx.cmd_data.cmd_flags_1;
    data.cmd_flags_2 = ctx.cmd_data.cmd_flags_2;
    data.cmd_number = if ctx.cmd_data.cmd_flags_1 == 0 && ctx.cmd_data.cmd_flags_2 == 0 {
        2
    } else {
        ctx.cmd_data.cmd_flags_1
    };
    data.s7k_url_list_mode = 1;
    data.s7k_keyconfig_flags = 0;

    bufcopy(&mut data.mutex_master_name, ctx.mutex_master_name.as_bytes());
    bufcopy(
        &mut data.mutex_master_ready_name,
        ctx.mutex_master_ready_name.as_bytes(),
    );
    bufcopy(&mut data.path1, ctx.mhf_folder_name.as_bytes());
    bufcopy(&mut data.path2, ctx.mhf_folder_name.as_bytes());
    data.ini_file.fill(0);
    bufcopy(&mut data.ini_file, INI_BASENAME);
    bufcopy(
        &mut data.remote_addr,
        format!("{}:{}", config.server_host, config.server_port).as_bytes(),
    );
    bufcopy(&mut data.remote_host, config.server_host.as_bytes());
    mirror_s7k_settings(data);

    Ok(())
}

unsafe fn cleanup_s7k(ptr: *mut usize) {
    drop_boxed::<Data>(ptr);
}

pub(crate) fn prepare(ctx: &LaunchContext, config: &MhfConfig) -> Result<PreparedLayout> {
    let mut data: Box<Data> = unsafe { boxed_zeroed() };
    std::env::set_var("JKR", "1");

    let (dll_name, layout_dll_name) = resolve_layout_dll_name(0);

    init_data(&mut data, ctx, config)?;
    bufcopy(
        &mut data.alt_ip_address,
        format!("{}:8080", config.server_host).as_bytes(),
    );
    data.server_expiry_ts = config.expiry_ts;
    data.fixed_449184_0x1 = 0x1;

    let (module_handle, mhddl_main, entry_proc) = load_layout_entry(dll_name)?;
    data.mhfo_module = module_handle;
    data.mhddl_main = mhddl_main;
    unsafe {
        patch_s7k_fallback_host(module_handle, config.server_host.as_bytes());
        patch_s7k_font_bootstrap(module_handle);
    }
    init_ptrs(&mut data);

    Ok(PreparedLayout {
        data_ptr: Box::into_raw(data) as *mut usize,
        entry_proc,
        mhfo_module: module_handle,
        friend_layout_dll_name: layout_dll_name,
        cleanup: cleanup_s7k,
    })
}







