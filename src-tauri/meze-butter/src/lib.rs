mod error;
#[cfg(windows)]
mod friend_injection;
#[cfg(windows)]
mod layouts;
#[cfg(windows)]
mod mhf;
#[cfg(windows)]
mod utils;

pub use error::Error;
pub use error::Result;
use serde::Serialize;

use std::path::PathBuf;

use num_enum::TryFromPrimitive;
use serde::Deserialize;

fn default_true() -> bool {
    true
}

#[repr(u8)]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    TryFromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum MhfVersion {
    #[default]
    #[serde(rename = "ZZ", alias = "Z3", alias = "Z3.1")]
    ZZ = 1,
    #[serde(rename = "Z1", alias = "Z1.1", alias = "Z1.2", alias = "Z2.1", alias = "Z2.3")]
    Z1 = 2,
    #[serde(rename = "G10.1", alias = "G10_1", alias = "G10")]
    G10_1 = 3,
    #[serde(rename = "G9.1", alias = "G9_1", alias = "G9")]
    G9_1 = 4,
    #[serde(rename = "G7")]
    G7 = 5,
    #[serde(rename = "G6", alias = "G6.1")]
    G6 = 6,
    #[serde(rename = "G5.2", alias = "G5_2")]
    G5_2 = 7,
    #[serde(rename = "GG", alias = "G4")]
    GG = 8,
    #[serde(rename = "G3.2", alias = "G3_2")]
    G3_2 = 9,
    #[serde(rename = "G3.1", alias = "G3_1")]
    G3_1 = 10,
    #[serde(rename = "G2")]
    G2 = 11,
    #[serde(rename = "G1")]
    G1 = 12,
    #[serde(rename = "F5")]
    F5 = 13,
    #[serde(rename = "F4")]
    F4 = 14,
    #[serde(rename = "S7K", alias = "Season 7.0")]
    S7K = 15,
    #[serde(rename = "S6", alias = "Season 6.0")]
    S6 = 16,
    #[serde(rename = "Z2T", alias = "Z2.2")]
    Z2T = 17,
    #[serde(rename = "G5.1", alias = "G5_1")]
    G5_1 = 18,
    #[serde(rename = "G3")]
    G3 = 19,
    #[serde(rename = "G5")]
    G5 = 20,
    #[serde(rename = "Z2")]
    Z2 = 21,
}

impl MhfVersion {
    pub fn all_versions() -> Vec<MhfVersion> {
        vec![
            MhfVersion::ZZ,
            MhfVersion::Z1,
            MhfVersion::G10_1,
            MhfVersion::G9_1,
            MhfVersion::G7,
            MhfVersion::G6,
            MhfVersion::G5_2,
            MhfVersion::GG,
            MhfVersion::G3_2,
            MhfVersion::G3_1,
            MhfVersion::G2,
            MhfVersion::G1,
            MhfVersion::F5,
            MhfVersion::F4,
            MhfVersion::S7K,
            MhfVersion::S6,
            MhfVersion::Z2T,
            MhfVersion::G5_1,
            MhfVersion::G3,
            MhfVersion::G5,
            MhfVersion::Z2,
        ]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TryFromPrimitive)]
pub enum CliFlags {
    Selfup = 1,
    Restat = 2,
    Autolc = 3,
    Hanres = 4,
    DmmBoot = 5,
    DmmSelfup = 6,
    DmmAutolc = 7,
    DmmReboot = 8,
    Npge = 9,
    NpMhfoTest = 10,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TryFromPrimitive)]
pub enum MezFesStall {
    TokotokoPartnya = 2,
    Pachinko = 3,
    VolpakkunTogether = 4,
    GoocooScoop = 5,
    Nyanrendo = 6,
    HoneyPanic = 7,
    DokkanBattleCats = 8,
    PointStall = 9,
    StallMap = 10,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notice {
    pub flags: u16,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FriendData {
    pub cid: u32,
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MhfConfig {
    pub char_id: u32,
    pub char_name: String,
    pub char_gr: u32,
    pub char_hr: u32,
    pub char_ids: Vec<u32>,
    pub char_new: bool,
    pub user_token_id: u32,
    pub user_token: String,
    pub user_name: String,
    pub user_password: String,
    pub user_rights: u32,
    pub server_host: String,
    pub server_port: u32,
    pub entrance_count: u32,
    pub current_ts: u32,
    pub expiry_ts: u32,
    pub notices: Vec<Notice>,
    pub friends: Vec<FriendData>,
    pub mez_event_id: u32,
    pub mez_start: u32,
    pub mez_end: u32,
    pub mez_solo_tickets: u32,
    pub mez_group_tickets: u32,
    pub mez_stalls: Vec<MezFesStall>,
    pub version: MhfVersion,
    #[serde(default)]
    pub mutex_version: MhfVersion,
    #[serde(default)]
    pub mutex_fallback_version: Option<MhfVersion>,
    pub preload_controller_dlls: bool,
    #[serde(default)]
    pub friend_signature: Option<String>,
    #[serde(default = "default_true")]
    pub enable_font_registration: bool,

    pub mhf_folder: Option<PathBuf>,
    pub font_path: Option<PathBuf>,
    pub mhf_flags: Option<Vec<CliFlags>>,
}

pub fn available_friend_signatures(version: MhfVersion, hd: bool) -> Vec<String> {
    const EMPTY: &[&str] = &[];
    const S6: &[&str] = &["v1.13.3246"];
    const S7K: &[&str] = &["v7.0.14_2"];
    const F4: &[&str] = &["v1.20_107869"];
    const F5: &[&str] = &["v1.20_125635", "v1.20_133710"];
    const G1: &[&str] = &["v1.22_153077", "v1.22_156129"];
    const G2: &[&str] = &["v1.23_187828"];
    const G3: &[&str] = &["v1.27_211402", "v1.27_212295"];
    const G3_1: &[&str] = &["v1.27_213258", "v1.27_215335", "v1.27_217155"];
    const G3_2: &[&str] = &["v1.27_222273", "v1.27_223087"];
    const GG: &[&str] = &["v1.28_246880"];
    const G5_1: &[&str] = &["v1.30.283838"];
    const G5_2: &[&str] = &["v1.32_302094"];
    const G6_SD: &[&str] = &["v1.33_325336"];
    const G6_HD: &[&str] = &["v1.33_326088"];
    const G7_SD: &[&str] = &["v1.36.05_936940dd"];
    const G7_HD: &[&str] = &["v1.36.05_a924ce4d"];
    const G9_1_SD: &[&str] = &["v1.38.19_e8966870"];
    const G9_1_HD: &[&str] = &["v1.38.19_47c90390"];
    const G10_1_SD: &[&str] = &["v1.41.30_c730c673", "v1.41.32_8acc3715"];
    const G10_1_HD: &[&str] = &["v1.41.30_f5ed3a6a", "v1.41.32_5c06b547"];
    const Z1_SD: &[&str] = &["v1.44.45_15a73eb7"];
    const Z1_HD: &[&str] = &["v1.44.45_dca95f5f"];
    const ZZ_SD: &[&str] = &["v1.52.79_04d16dc4"];
    const ZZ_HD: &[&str] = &["v1.52.79_73c49f52"];

    let sigs: &[&str] = match version {
        MhfVersion::S6 => S6,
        MhfVersion::S7K => S7K,
        MhfVersion::F4 => F4,
        MhfVersion::F5 => F5,
        MhfVersion::G1 => G1,
        MhfVersion::G2 => G2,
        MhfVersion::G3 => G3,
        MhfVersion::G3_1 => G3_1,
        MhfVersion::G3_2 => G3_2,
        MhfVersion::GG => GG,
        MhfVersion::G5_1 => G5_1,
        MhfVersion::G5_2 => G5_2,
        MhfVersion::G6 => if hd { G6_HD } else { G6_SD },
        MhfVersion::G7 => if hd { G7_HD } else { G7_SD },
        MhfVersion::G9_1 => if hd { G9_1_HD } else { G9_1_SD },
        MhfVersion::G10_1 => if hd { G10_1_HD } else { G10_1_SD },
        MhfVersion::Z1 => if hd { Z1_HD } else { Z1_SD },
        MhfVersion::ZZ => if hd { ZZ_HD } else { ZZ_SD },
        MhfVersion::G5 | MhfVersion::Z2 | MhfVersion::Z2T => EMPTY,
    };

    sigs.iter().map(|s| (*s).to_string()).collect()
}

const BASE32_CHARS: &[u8; 32] = b"123456789ABCDEFGHJKLMNPQRTUVWXYZ";
const BASE32_CAP: u32 = 32u32.pow(6);

#[inline]
pub fn make_ext_id(mut id: u32) -> String {
    debug_assert!(id < BASE32_CAP, "ext_id overflow: {}", id);
    let mut out = [b'1'; 6];
    for byte in &mut out {
        *byte = BASE32_CHARS[(id % 32) as usize];
        id /= 32;
        if id == 0 {
            break;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn run(config: MhfConfig) -> Result<isize> {
    if config.user_token.len() != 16 {
        return Err(Error::TokenLength);
    }
    #[cfg(windows)]
    {
        return mhf::run_mhf(config);
    }

    #[cfg(not(windows))]
    {
        let _ = config;
        Err(Error::UnsupportedPlatform)
    }
}
