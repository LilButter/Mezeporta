use crate::mhf::{
    boxed_zeroed, drop_boxed, gg_proc_addr, init_global_alloc, load_layout_entry,
    mock_proc_addr, resolve_layout_dll_name, LaunchContext,
    PreparedLayout, INI_BASENAME,
};
use crate::utils::bufcopy;
use crate::{MhfConfig, Result};
use windows::core::s;
use windows::Win32::Foundation::{FARPROC, HANDLE, HGLOBAL, HMODULE};
use windows::Win32::System::WindowsProgramming::{
    GetPrivateProfileIntA, GetPrivateProfileStringA,
};
use windows::Win32::UI::TextServices::HKL;

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
    cmd_dmm: u32,
    _pad_448998: [u8; 0x4],
    mutex_master: HANDLE,
    mutex_master_ready: HANDLE,
    mutex_master_name: [u8; 0x40],
    ini_file: [u8; 0x40],
    proc_1: usize,
    proc_2: usize,
    proc_3: usize,
    _pad_448a30: [u8; 0xc],
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
    selected_char_name: [u8; 0x10],
    char_ids: [u32; 0x10],
    global_alloc: HGLOBAL,
    fixed_448ed0_0x1: u32,
    unk_448ed4: u32,
    selected_char_gr: u32,
    _pad_448edc: [u8; 0x8],
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
    font_name: [u8; 0x68],
    drawskip: u32,
    clogdis: u32,
    proxy_use: u32,
    proxy_ie: u32,
    proxy_set: u32,
    proxy_addr: [u8; 0x40],
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
}

fn init_ptrs(data: &mut Box<Data>) {
    data.data_ptr = Box::as_ref(data) as *const _ as usize;
    data.inner_ptr_1_4491a8 = &data.inner_1 as *const _ as usize;
    data.inner_ptr_2_4491d4 = &data.inner_2 as *const _ as usize;
    data.inner_ptr_3_449198 = &data.inner_3 as *const _ as usize;
}

fn init_data(data: &mut Box<Data>, ctx: &LaunchContext, config: &MhfConfig) {
    data.main_module = ctx.main_module;
    data.mutex_master = ctx.mutex_master;
    data.mutex_master_ready = ctx.mutex_master_ready;
    data.global_alloc = ctx.global_alloc;
    data.keyboard_layout = ctx.keyboard_layout;
    data.fixed_448a64_0x0 = 0x0;
    data.fixed_448ed0_0x1 = 0x1;
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
            s!("MS ????"),
            Some(&mut data.font_name),
            ctx.ini_file,
        );
        if config.enable_font_registration {
            bufcopy(&mut data.font_name, b"MS Gothic\0");
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

    init_global_alloc(data.global_alloc, config);

    data.selected_char_id_1 = config.char_id;
    data.selected_char_id_2 = config.char_id;
    bufcopy(&mut data.selected_char_name, config.char_name.as_bytes());
    data.selected_char_hr = config.char_hr;
    data.selected_char_gr = config.char_gr;
    data.selected_char_status = if config.char_new { 2 } else { 0 };

    bufcopy(&mut data.user_name, config.user_name.as_bytes());
    bufcopy(&mut data.user_password, config.user_password.as_bytes());
    data.user_token_id = config.user_token_id;
    bufcopy(&mut data.user_token, config.user_token.as_bytes());
    data.user_rights = config.user_rights;
    bufcopy(&mut data.char_ids, &config.char_ids);

    data.server_entrance_count = config.entrance_count;
    data.server_current_ts = config.current_ts;

    data.cmd_flags_1 = ctx.cmd_data.cmd_flags_1;
    data.cmd_flags_2 = ctx.cmd_data.cmd_flags_2;
    data.cmd_dmm = ctx.cmd_data.cmd_dmm;
    data.cmd_number = if ctx.cmd_data.cmd_flags_1 == 0 && ctx.cmd_data.cmd_flags_2 == 0 {
        1
    } else {
        ctx.cmd_data.cmd_flags_1
    };

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
}

pub(crate) fn prepare(ctx: &LaunchContext, config: &MhfConfig) -> Result<PreparedLayout> {
    let mut data: Box<Data> = unsafe { boxed_zeroed() };
    std::env::set_var("JKR", "1");

    let (dll_name, layout_dll_name) = resolve_layout_dll_name(0);

    init_data(&mut data, ctx, config);
    bufcopy(
        &mut data.alt_ip_address,
        format!("{}:8080", config.server_host).as_bytes(),
    );
    data.server_expiry_ts = config.expiry_ts;
    data.fixed_449184_0x1 = 0x1;

    let (module_handle, mhddl_main, entry_proc) = load_layout_entry(dll_name)?;
    data.mhfo_module = module_handle;
    data.mhddl_main = mhddl_main;
    init_ptrs(&mut data);

    Ok(PreparedLayout {
        data_ptr: Box::into_raw(data) as *mut usize,
        entry_proc,
        mhfo_module: module_handle,
        friend_layout_dll_name: layout_dll_name,
        cleanup: drop_boxed::<Data>,
    })
}




