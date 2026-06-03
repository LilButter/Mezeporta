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
