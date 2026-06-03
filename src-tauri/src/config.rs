#![allow(clippy::needless_update)]
use crate::Endpoint;

pub const CLASSIC_STYLE: u32 = 1;
pub const PS4_STYLE: u32 = 3;

pub const DEFAULT_SERVERLIST_URL: &str = "";
pub const DEFAULT_MESSAGELIST_URL: &str = "";

pub fn get_default_endpoints() -> Vec<Endpoint> {
    vec![Endpoint {
        name: "Offline-Mode".into(),
        url: "OFFLINEMODE".into(),
        is_remote: true,
        ..Default::default()
    }]
}
