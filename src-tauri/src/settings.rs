use std::fs;
use std::path::{Path, PathBuf};

#[cfg(not(target_os = "windows"))]
use configparser::ini::Ini;
use log::warn;
use serde::Serialize;
use serde_json::Value;
#[cfg(target_os = "windows")]
use windows::core::{w, HSTRING, PCWSTR};
#[cfg(target_os = "windows")]
use windows::Win32::System::WindowsProgramming::{
    GetPrivateProfileIntW, WritePrivateProfileStringW,
};

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    hd_version: bool,
    fullscreen: bool,
    fullscreen_w: i32,
    fullscreen_h: i32,
    window_w: i32,
    window_h: i32,
    brightness: i32,
    max_char_display: i32,
    texture_compression: bool,
    match_monitor_resolution: bool,
    disable_sound_output: bool,
    sound: i32,
    sound_unfocused: i32,
    sound_minimized: i32,
    sound_frequency: i32,
    sound_buffer_num: i32,
    game_bgm_volume: i32,
    game_se_volume: i32,
    controller_vibration: bool,
    hd_graphic_shadow_quest: bool,
    hd_graphic_shadow_lobby: bool,
    hd_graphic_dof: bool,
    hd_graphic_bloom: bool,
    hd_graphic_ssao: bool,
    hd_graphic_godray: bool,
    hd_graphic_anti_aliasing: bool,
    hd_graphic_soft_particle: bool,
    hd_graphic_dof_far_blur_size: i32,
    hd_graphic_bloom_dispersion: i32,
    hd_graphic_bloom_threshold: i32,
    hd_graphic_bloom_color: i32,
    hd_graphic_gaussian_blur_dispersion: i32,
    hd_graphic_gaussian_blur_blend_rate: i32,
    hd_graphic_ssao_density: i32,
    hd_graphic_shadowmap_color: i32,
    hd_graphic_pl_light_shadow_attenuation: i32,
    hd_graphic_bg_light_shadow_attenuation: i32,
    hd_graphic_anti_aliasing_weight_scale: i32,
}

struct SettingEntry {
    section: &'static str,
    key: &'static str,
    value: String,
}

fn bool_string(value: bool) -> String {
    if value {
        "1".to_string()
    } else {
        "0".to_string()
    }
}

fn number_string(value: Value) -> Option<String> {
    match value {
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn setting_entry(name: &str, value: Value) -> Option<SettingEntry> {
    match (name, value) {
        ("hdVersion", Value::Bool(v)) => Some(SettingEntry {
            section: "VIDEO",
            key: "GRAPHICS_VER",
            value: bool_string(v),
        }),
        ("fullscreen", Value::Bool(v)) => Some(SettingEntry {
            section: "SCREEN",
            key: "FULLSCREEN_MODE",
            value: bool_string(v),
        }),
        ("brightness", value) => number_string(value).map(|value| SettingEntry {
            section: "SCREEN",
            key: "BRIGHT",
            value,
        }),
        ("fullscreenW", value) => number_string(value).map(|value| SettingEntry {
            section: "SCREEN",
            key: "FULLSCREEN_RESOLUTION_W",
            value,
        }),
        ("fullscreenH", value) => number_string(value).map(|value| SettingEntry {
            section: "SCREEN",
            key: "FULLSCREEN_RESOLUTION_H",
            value,
        }),
        ("windowW", value) => number_string(value).map(|value| SettingEntry {
            section: "SCREEN",
            key: "WINDOW_RESOLUTION_W",
            value,
        }),
        ("windowH", value) => number_string(value).map(|value| SettingEntry {
            section: "SCREEN",
            key: "WINDOW_RESOLUTION_H",
            value,
        }),
        ("maxCharDisplay", value) => number_string(value).map(|value| SettingEntry {
            section: "VIDEO",
            key: "DISP_MAX_CHAR",
            value,
        }),
        ("textureCompression", Value::Bool(v)) => Some(SettingEntry {
            section: "VIDEO",
            key: "TEXTURE_DXT_USE",
            value: bool_string(v),
        }),
        ("matchMonitorResolution", Value::Bool(v)) => Some(SettingEntry {
            section: "VIDEO",
            key: "NOW_MONITOR_WH",
            value: bool_string(v),
        }),
        ("disableSoundOutput", Value::Bool(v)) => Some(SettingEntry {
            section: "SOUND",
            key: "SOUND_NOTUSE",
            value: bool_string(v),
        }),
        ("sound", value) => number_string(value).map(|value| SettingEntry {
            section: "SOUND",
            key: "SOUND_VOLUME",
            value,
        }),
        ("soundUnfocused", value) => number_string(value).map(|value| SettingEntry {
            section: "SOUND",
            key: "SOUND_VOLUME_INACTIVITY",
            value,
        }),
        ("soundMinimized", value) => number_string(value).map(|value| SettingEntry {
            section: "SOUND",
            key: "SOUND_VOLUME_MINIMIZE",
            value,
        }),
        ("soundFrequency", value) => number_string(value).map(|value| SettingEntry {
            section: "SOUND",
            key: "SOUND_FREQUENCY",
            value,
        }),
        ("soundBufferNum", value) => number_string(value).map(|value| SettingEntry {
            section: "SOUND",
            key: "SOUND_BUFFERNUM",
            value,
        }),
        ("gameBgmVolume", value) => number_string(value).map(|value| SettingEntry {
            section: "SOUND",
            key: "SOUND_BGM",
            value,
        }),
        ("gameSeVolume", value) => number_string(value).map(|value| SettingEntry {
            section: "SOUND",
            key: "SOUND_SE",
            value,
        }),
        ("controllerVibration", Value::Bool(v)) => Some(SettingEntry {
            section: "OPTION",
            key: "VIBRATION",
            value: bool_string(v),
        }),
        ("hdGraphicShadowQuest", Value::Bool(v)) => Some(SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_SHADOW_QUEST",
            value: bool_string(v),
        }),
        ("hdGraphicShadowLobby", Value::Bool(v)) => Some(SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_SHADOW_LOBBY",
            value: bool_string(v),
        }),
        ("hdGraphicDof", Value::Bool(v)) => Some(SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_DOF",
            value: bool_string(v),
        }),
        ("hdGraphicBloom", Value::Bool(v)) => Some(SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_BLOOM",
            value: bool_string(v),
        }),
        ("hdGraphicSsao", Value::Bool(v)) => Some(SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_SSAO",
            value: bool_string(v),
        }),
        ("hdGraphicGodray", Value::Bool(v)) => Some(SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_GODRAY",
            value: bool_string(v),
        }),
        ("hdGraphicAntiAliasing", Value::Bool(v)) => Some(SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_ANTI_ALIASING",
            value: bool_string(v),
        }),
        ("hdGraphicSoftParticle", Value::Bool(v)) => Some(SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_SOFTPARTICLE",
            value: bool_string(v),
        }),
        ("hdGraphicDofFarBlurSize", value) => number_string(value).map(|value| SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_DOF_FARBLURSIZE",
            value,
        }),
        ("hdGraphicBloomDispersion", value) => number_string(value).map(|value| SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_BLOOM_DISPERSION",
            value,
        }),
        ("hdGraphicBloomThreshold", value) => number_string(value).map(|value| SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_BLOOM_THRESHOLD",
            value,
        }),
        ("hdGraphicBloomColor", value) => number_string(value).map(|value| SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_BLOOM_COLOR",
            value,
        }),
        ("hdGraphicGaussianBlurDispersion", value) => {
            number_string(value).map(|value| SettingEntry {
                section: "OPTION",
                key: "90C_GRAPHIC_GAUSSIANBLUR_DISPERSION",
                value,
            })
        }
        ("hdGraphicGaussianBlurBlendRate", value) => {
            number_string(value).map(|value| SettingEntry {
                section: "OPTION",
                key: "90C_GRAPHIC_GAUSSIANBLUR_BLENDRATE",
                value,
            })
        }
        ("hdGraphicSsaoDensity", value) => number_string(value).map(|value| SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_SSAO_DENSITY",
            value,
        }),
        ("hdGraphicShadowmapColor", value) => number_string(value).map(|value| SettingEntry {
            section: "OPTION",
            key: "90C_GRAPHIC_SHADOWMAP_COLOR",
            value,
        }),
        ("hdGraphicPlLightShadowAttenuation", value) => {
            number_string(value).map(|value| SettingEntry {
                section: "OPTION",
                key: "90C_GRAPHIC_PLLIGHT_SHADOWATTENUATION",
                value,
            })
        }
        ("hdGraphicBgLightShadowAttenuation", value) => {
            number_string(value).map(|value| SettingEntry {
                section: "OPTION",
                key: "90C_GRAPHIC_BGLIGHT_SHADOWATTENUATION",
                value,
            })
        }
        ("hdGraphicAntiAliasingWeightScale", value) => {
            number_string(value).map(|value| SettingEntry {
                section: "OPTION",
                key: "90C_GRAPHIC_ANTI_ALIASING_WEIGHTSCALE",
                value,
            })
        }
        _ => None,
    }
}

fn find_ini_file(folder: &Path) -> PathBuf {
    let default = folder.join("mhf.ini");
    if default.exists() {
        return default;
    }
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map_or(false, |ext| ext == "ini") {
                return p;
            }
        }
    }
    default
}

fn ensure_ini_file(folder: &Path) -> PathBuf {
    let ini_path = find_ini_file(folder);
    if !ini_path.exists() {
        if let Some(parent) = ini_path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                warn!("failed to create config folder {}: {}", parent.display(), err);
            }
        }
        if let Err(err) = fs::OpenOptions::new().create(true).append(true).open(&ini_path) {
            warn!("failed to create config file {}: {}", ini_path.display(), err);
        }
    }
    ini_path
}

#[cfg(target_os = "windows")]
fn write_profile_string(
    section: &str,
    key: &str,
    value: &str,
    ini_file: PCWSTR,
) -> windows::core::Result<()> {
    let section_w = HSTRING::from(section);
    let key_w = HSTRING::from(key);
    let value_w = HSTRING::from(value);
    unsafe {
        WritePrivateProfileStringW(
            PCWSTR(section_w.as_ptr()),
            PCWSTR(key_w.as_ptr()),
            PCWSTR(value_w.as_ptr()),
            ini_file,
        )
    }
}

#[cfg(target_os = "windows")]
pub fn get_settings(path: &Path) -> Settings {
    let ini_path = ensure_ini_file(path);
    let ini_file = HSTRING::from(ini_path.as_os_str());
    let ini_file = PCWSTR(ini_file.as_ptr());
    unsafe {
        Settings {
            hd_version: GetPrivateProfileIntW(w!("VIDEO"), w!("GRAPHICS_VER"), 1, ini_file) > 0,
            fullscreen: GetPrivateProfileIntW(w!("SCREEN"), w!("FULLSCREEN_MODE"), 1, ini_file) > 0,
            fullscreen_w: GetPrivateProfileIntW(
                w!("SCREEN"),
                w!("FULLSCREEN_RESOLUTION_W"),
                1920,
                ini_file,
            ),
            fullscreen_h: GetPrivateProfileIntW(
                w!("SCREEN"),
                w!("FULLSCREEN_RESOLUTION_H"),
                1080,
                ini_file,
            ),
            window_w: GetPrivateProfileIntW(
                w!("SCREEN"),
                w!("WINDOW_RESOLUTION_W"),
                1920,
                ini_file,
            ),
            window_h: GetPrivateProfileIntW(
                w!("SCREEN"),
                w!("WINDOW_RESOLUTION_H"),
                1080,
                ini_file,
            ),
            brightness: GetPrivateProfileIntW(w!("SCREEN"), w!("BRIGHT"), -128, ini_file),
            max_char_display: GetPrivateProfileIntW(
                w!("VIDEO"),
                w!("DISP_MAX_CHAR"),
                100,
                ini_file,
            ),
            texture_compression: GetPrivateProfileIntW(
                w!("VIDEO"),
                w!("TEXTURE_DXT_USE"),
                0,
                ini_file,
            ) > 0,
            match_monitor_resolution: GetPrivateProfileIntW(
                w!("VIDEO"),
                w!("NOW_MONITOR_WH"),
                1,
                ini_file,
            ) > 0,
            disable_sound_output: GetPrivateProfileIntW(
                w!("SOUND"),
                w!("SOUND_NOTUSE"),
                0,
                ini_file,
            ) > 0,
            sound: GetPrivateProfileIntW(w!("SOUND"), w!("SOUND_VOLUME"), 0, ini_file),
            sound_unfocused: GetPrivateProfileIntW(
                w!("SOUND"),
                w!("SOUND_VOLUME_INACTIVITY"),
                0,
                ini_file,
            ),
            sound_minimized: GetPrivateProfileIntW(
                w!("SOUND"),
                w!("SOUND_VOLUME_MINIMIZE"),
                0,
                ini_file,
            ),
            sound_frequency: GetPrivateProfileIntW(
                w!("SOUND"),
                w!("SOUND_FREQUENCY"),
                48000,
                ini_file,
            ),
            sound_buffer_num: GetPrivateProfileIntW(
                w!("SOUND"),
                w!("SOUND_BUFFERNUM"),
                2048,
                ini_file,
            ),
            game_bgm_volume: GetPrivateProfileIntW(w!("SOUND"), w!("SOUND_BGM"), 5, ini_file),
            game_se_volume: GetPrivateProfileIntW(w!("SOUND"), w!("SOUND_SE"), 7, ini_file),
            controller_vibration: GetPrivateProfileIntW(w!("OPTION"), w!("VIBRATION"), 0, ini_file)
                > 0,
            hd_graphic_shadow_quest: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_SHADOW_QUEST"),
                1,
                ini_file,
            ) > 0,
            hd_graphic_shadow_lobby: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_SHADOW_LOBBY"),
                0,
                ini_file,
            ) > 0,
            hd_graphic_dof: GetPrivateProfileIntW(w!("OPTION"), w!("90C_GRAPHIC_DOF"), 1, ini_file)
                > 0,
            hd_graphic_bloom: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_BLOOM"),
                1,
                ini_file,
            ) > 0,
            hd_graphic_ssao: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_SSAO"),
                1,
                ini_file,
            ) > 0,
            hd_graphic_godray: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_GODRAY"),
                1,
                ini_file,
            ) > 0,
            hd_graphic_anti_aliasing: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_ANTI_ALIASING"),
                1,
                ini_file,
            ) > 0,
            hd_graphic_soft_particle: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_SOFTPARTICLE"),
                1,
                ini_file,
            ) > 0,
            hd_graphic_dof_far_blur_size: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_DOF_FARBLURSIZE"),
                100,
                ini_file,
            ),
            hd_graphic_bloom_dispersion: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_BLOOM_DISPERSION"),
                100,
                ini_file,
            ),
            hd_graphic_bloom_threshold: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_BLOOM_THRESHOLD"),
                100,
                ini_file,
            ),
            hd_graphic_bloom_color: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_BLOOM_COLOR"),
                100,
                ini_file,
            ),
            hd_graphic_gaussian_blur_dispersion: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_GAUSSIANBLUR_DISPERSION"),
                100,
                ini_file,
            ),
            hd_graphic_gaussian_blur_blend_rate: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_GAUSSIANBLUR_BLENDRATE"),
                100,
                ini_file,
            ),
            hd_graphic_ssao_density: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_SSAO_DENSITY"),
                100,
                ini_file,
            ),
            hd_graphic_shadowmap_color: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_SHADOWMAP_COLOR"),
                100,
                ini_file,
            ),
            hd_graphic_pl_light_shadow_attenuation: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_PLLIGHT_SHADOWATTENUATION"),
                100,
                ini_file,
            ),
            hd_graphic_bg_light_shadow_attenuation: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_BGLIGHT_SHADOWATTENUATION"),
                100,
                ini_file,
            ),
            hd_graphic_anti_aliasing_weight_scale: GetPrivateProfileIntW(
                w!("OPTION"),
                w!("90C_GRAPHIC_ANTI_ALIASING_WEIGHTSCALE"),
                100,
                ini_file,
            ),
        }
    }
}

#[cfg(target_os = "windows")]
pub fn set_setting(path: &Path, name: &str, value: Value) -> Result<(), String> {
    let Some(entry) = setting_entry(name, value) else {
        warn!("unknown setting: {}", name);
        return Ok(());
    };

    let ini_path = ensure_ini_file(path);
    let ini_file = HSTRING::from(ini_path.as_os_str());
    let ini_file = PCWSTR(ini_file.as_ptr());
    write_profile_string(entry.section, entry.key, &entry.value, ini_file).map_err(|e| {
        warn!("failed to write to config: {}, {}", name, e);
        "settings-error".to_owned()
    })
}

#[cfg(not(target_os = "windows"))]
pub fn get_settings(path: &Path) -> Settings {
    let ini_path = ensure_ini_file(path);
    let mut ini = Ini::new();
    let _ = ini.load(ini_path.to_string_lossy().as_ref());
    Settings {
        hd_version: ini
            .getint("VIDEO", "GRAPHICS_VER")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        fullscreen: ini
            .getint("SCREEN", "FULLSCREEN_MODE")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        fullscreen_w: ini
            .getint("SCREEN", "FULLSCREEN_RESOLUTION_W")
            .ok()
            .flatten()
            .unwrap_or(1920) as i32,
        fullscreen_h: ini
            .getint("SCREEN", "FULLSCREEN_RESOLUTION_H")
            .ok()
            .flatten()
            .unwrap_or(1080) as i32,
        window_w: ini
            .getint("SCREEN", "WINDOW_RESOLUTION_W")
            .ok()
            .flatten()
            .unwrap_or(1920) as i32,
        window_h: ini
            .getint("SCREEN", "WINDOW_RESOLUTION_H")
            .ok()
            .flatten()
            .unwrap_or(1080) as i32,
        brightness: ini
            .getint("SCREEN", "BRIGHT")
            .ok()
            .flatten()
            .unwrap_or(-128) as i32,
        max_char_display: ini
            .getint("VIDEO", "DISP_MAX_CHAR")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        texture_compression: ini
            .getint("VIDEO", "TEXTURE_DXT_USE")
            .ok()
            .flatten()
            .unwrap_or(0)
            > 0,
        match_monitor_resolution: ini
            .getint("VIDEO", "NOW_MONITOR_WH")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        disable_sound_output: ini
            .getint("SOUND", "SOUND_NOTUSE")
            .ok()
            .flatten()
            .unwrap_or(0)
            > 0,
        sound: ini
            .getint("SOUND", "SOUND_VOLUME")
            .ok()
            .flatten()
            .unwrap_or(0) as i32,
        sound_unfocused: ini
            .getint("SOUND", "SOUND_VOLUME_INACTIVITY")
            .ok()
            .flatten()
            .unwrap_or(0) as i32,
        sound_minimized: ini
            .getint("SOUND", "SOUND_VOLUME_MINIMIZE")
            .ok()
            .flatten()
            .unwrap_or(0) as i32,
        sound_frequency: ini
            .getint("SOUND", "SOUND_FREQUENCY")
            .ok()
            .flatten()
            .unwrap_or(48000) as i32,
        sound_buffer_num: ini
            .getint("SOUND", "SOUND_BUFFERNUM")
            .ok()
            .flatten()
            .unwrap_or(2048) as i32,
        game_bgm_volume: ini.getint("SOUND", "SOUND_BGM").ok().flatten().unwrap_or(5) as i32,
        game_se_volume: ini.getint("SOUND", "SOUND_SE").ok().flatten().unwrap_or(7) as i32,
        controller_vibration: ini
            .getint("OPTION", "VIBRATION")
            .ok()
            .flatten()
            .unwrap_or(0)
            > 0,
        hd_graphic_shadow_quest: ini
            .getint("OPTION", "90C_GRAPHIC_SHADOW_QUEST")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        hd_graphic_shadow_lobby: ini
            .getint("OPTION", "90C_GRAPHIC_SHADOW_LOBBY")
            .ok()
            .flatten()
            .unwrap_or(0)
            > 0,
        hd_graphic_dof: ini
            .getint("OPTION", "90C_GRAPHIC_DOF")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        hd_graphic_bloom: ini
            .getint("OPTION", "90C_GRAPHIC_BLOOM")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        hd_graphic_ssao: ini
            .getint("OPTION", "90C_GRAPHIC_SSAO")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        hd_graphic_godray: ini
            .getint("OPTION", "90C_GRAPHIC_GODRAY")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        hd_graphic_anti_aliasing: ini
            .getint("OPTION", "90C_GRAPHIC_ANTI_ALIASING")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        hd_graphic_soft_particle: ini
            .getint("OPTION", "90C_GRAPHIC_SOFTPARTICLE")
            .ok()
            .flatten()
            .unwrap_or(1)
            > 0,
        hd_graphic_dof_far_blur_size: ini
            .getint("OPTION", "90C_GRAPHIC_DOF_FARBLURSIZE")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_bloom_dispersion: ini
            .getint("OPTION", "90C_GRAPHIC_BLOOM_DISPERSION")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_bloom_threshold: ini
            .getint("OPTION", "90C_GRAPHIC_BLOOM_THRESHOLD")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_bloom_color: ini
            .getint("OPTION", "90C_GRAPHIC_BLOOM_COLOR")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_gaussian_blur_dispersion: ini
            .getint("OPTION", "90C_GRAPHIC_GAUSSIANBLUR_DISPERSION")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_gaussian_blur_blend_rate: ini
            .getint("OPTION", "90C_GRAPHIC_GAUSSIANBLUR_BLENDRATE")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_ssao_density: ini
            .getint("OPTION", "90C_GRAPHIC_SSAO_DENSITY")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_shadowmap_color: ini
            .getint("OPTION", "90C_GRAPHIC_SHADOWMAP_COLOR")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_pl_light_shadow_attenuation: ini
            .getint("OPTION", "90C_GRAPHIC_PLLIGHT_SHADOWATTENUATION")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_bg_light_shadow_attenuation: ini
            .getint("OPTION", "90C_GRAPHIC_BGLIGHT_SHADOWATTENUATION")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
        hd_graphic_anti_aliasing_weight_scale: ini
            .getint("OPTION", "90C_GRAPHIC_ANTI_ALIASING_WEIGHTSCALE")
            .ok()
            .flatten()
            .unwrap_or(100) as i32,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_setting(path: &Path, name: &str, value: Value) -> Result<(), String> {
    let Some(entry) = setting_entry(name, value) else {
        warn!("unknown setting: {}", name);
        return Ok(());
    };

    let ini_path = ensure_ini_file(path);
    let mut ini = Ini::new();
    let _ = ini.load(ini_path.to_string_lossy().as_ref());
    ini.set(entry.section, entry.key, Some(entry.value));
    ini.write(ini_path.to_string_lossy().as_ref()).map_err(|e| {
        warn!("failed to write to config: {}, {}", name, e);
        "settings-error".to_owned()
    })
}
