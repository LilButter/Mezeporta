use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::{endpoint::Endpoint, server};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientMezFes {
    pub id: u32,
    pub start: u32,
    pub end: u32,
    pub solo_tickets: u32,
    pub group_tickets: u32,
    #[serde(default)]
    pub stalls: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientFeaturedWeapon {
    pub start_time: u32,
    pub active_features: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientEvents {
    pub festa_active: bool,
    pub diva_active: bool,
    #[serde(default)]
    pub tournament_active: bool,
    #[serde(default)]
    pub special_events: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientUserStats {
    #[serde(default)]
    pub gacha_premium: u32,
    #[serde(default)]
    pub gacha_trial: u32,
    #[serde(default)]
    pub frontier_points: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientDistributionItemInfo {
    #[serde(default)]
    pub id: u32,
    #[serde(default)]
    pub item_type: u8,
    #[serde(default)]
    pub item_id: u32,
    #[serde(default)]
    pub quantity: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientDistributionInfo {
    pub id: u32,
    #[serde(default)]
    pub event_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub distribution_type: i32,
    #[serde(default)]
    pub type_label: String,
    #[serde(default)]
    pub deadline: i64,
    #[serde(default)]
    pub times_acceptable: u32,
    #[serde(default)]
    pub min_hr: Option<i32>,
    #[serde(default)]
    pub max_hr: Option<i32>,
    #[serde(default)]
    pub min_sr: Option<i32>,
    #[serde(default)]
    pub max_sr: Option<i32>,
    #[serde(default)]
    pub min_gr: Option<i32>,
    #[serde(default)]
    pub max_gr: Option<i32>,
    #[serde(default)]
    pub items: Vec<AltClientDistributionItemInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientDistributionPage {
    #[serde(default)]
    pub character_id: u32,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub limit: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub entries: Vec<AltClientDistributionInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientMailInfo {
    #[serde(default)]
    pub sender_id: u32,
    #[serde(default)]
    pub sender_name: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub has_item: bool,
    #[serde(default)]
    pub attached_item: u32,
    #[serde(default)]
    pub attached_item_amount: u32,
    #[serde(default)]
    pub item_amount: u32,
    #[serde(default)]
    pub is_guild_invite: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub is_system_message: bool,
    #[serde(default)]
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientCharacterStats {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub returning: bool,
    #[serde(default)]
    pub courses: Vec<server::CourseData>,
    #[serde(default)]
    pub time_played: u32,
    #[serde(default)]
    pub unread_mail: u32,
    #[serde(default)]
    pub unread_mail_entries: Vec<AltClientMailInfo>,
    #[serde(default)]
    pub unclaimed_distributions: u32,
    #[serde(default)]
    pub unclaimed_distribution_names: Vec<String>,
    #[serde(default)]
    pub unclaimed_distribution_details: Vec<AltClientDistributionInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientOnlineFriend {
    pub cid: u32,
    pub id: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub server_id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct ServerStatusResponse {
    #[serde(default)]
    pub mez_fes: Option<AltClientMezFes>,
    #[serde(default)]
    pub featured_weapon: Option<AltClientFeaturedWeapon>,
    #[serde(default)]
    pub events: AltClientEvents,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct DashboardStatsResponse {
    #[serde(default)]
    pub online_players: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct AltClientSessionCharacterStats {
    pub id: u32,
    #[serde(default)]
    pub time_played: u32,
    #[serde(default)]
    pub unread_mail: u32,
    #[serde(default)]
    pub unread_mail_entries: Vec<AltClientMailInfo>,
    #[serde(default)]
    pub unclaimed_distributions: u32,
    #[serde(default)]
    pub unclaimed_distribution_names: Vec<String>,
    #[serde(default)]
    pub unclaimed_distribution_details: Vec<AltClientDistributionInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct AltClientSessionStatsResponse {
    #[serde(default)]
    pub user: AltClientUserStats,
    #[serde(default)]
    pub characters: Vec<AltClientSessionCharacterStats>,
    #[serde(default)]
    pub online_friends: Vec<AltClientOnlineFriend>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AltClientStats {
    #[serde(default)]
    pub mez_fes: Option<AltClientMezFes>,
    #[serde(default)]
    pub featured_weapon: Option<AltClientFeaturedWeapon>,
    #[serde(default)]
    pub events: AltClientEvents,
    #[serde(default)]
    pub enabled_special_events: Vec<String>,
    #[serde(default)]
    pub online_players: u32,
    #[serde(default)]
    pub user: AltClientUserStats,
    #[serde(default)]
    pub characters: Vec<AltClientCharacterStats>,
    #[serde(default)]
    pub online_friends: Vec<AltClientOnlineFriend>,
}

pub fn build_character_stats(
    characters: Vec<server::CharacterData>,
    courses: Vec<server::CourseData>,
) -> Vec<AltClientCharacterStats> {
    characters
        .into_iter()
        .map(|character| AltClientCharacterStats {
            id: character.id,
            name: character.name,
            returning: character.returning,
            courses: courses.clone(),
            ..Default::default()
        })
        .collect()
}

fn apply_session_stats(out: &mut AltClientStats, stats: AltClientSessionStatsResponse) {
    out.user = stats.user;
    out.online_friends = stats.online_friends;

    let by_char_id: HashMap<u32, AltClientSessionCharacterStats> = stats
        .characters
        .into_iter()
        .map(|character| (character.id, character))
        .collect();

    for character in &mut out.characters {
        if let Some(extra) = by_char_id.get(&character.id) {
            character.time_played = extra.time_played;
            character.unread_mail = extra.unread_mail;
            character.unread_mail_entries = extra.unread_mail_entries.clone();
            character.unclaimed_distributions = extra.unclaimed_distributions;
            character.unclaimed_distribution_names = extra.unclaimed_distribution_names.clone();
            character.unclaimed_distribution_details = extra.unclaimed_distribution_details.clone();
        }
    }
}

pub async fn fetch(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
    characters: Vec<server::CharacterData>,
    courses: Vec<server::CourseData>,
) -> AltClientStats {
    let mut out = AltClientStats {
        characters: build_character_stats(characters, courses),
        ..Default::default()
    };

    let status_req = server::simple_request::<ServerStatusResponse>(
        client,
        cancel.clone(),
        &endpoint.get_url("/v2/server/status"),
    );
    if let Ok(status) = status_req.send().await {
        out.mez_fes = status.mez_fes;
        out.featured_weapon = status.featured_weapon;
        out.events = status.events;
    }
    out.enabled_special_events = out.events.special_events.clone();

    let dashboard_req = server::simple_request::<DashboardStatsResponse>(
        client,
        cancel.clone(),
        &endpoint.get_url("/api/dashboard/stats"),
    );
    if let Ok(stats) = dashboard_req.send().await {
        out.online_players = stats.online_players;
    }

    if !token.trim().is_empty() {
        let request = client
            .get(endpoint.get_url("/v2/altclient/stats"))
            .bearer_auth(token);
        let stats_req = server::JsonRequest::<AltClientSessionStatsResponse>::new(request, cancel);
        if let Ok(stats) = stats_req.send().await {
            apply_session_stats(&mut out, stats);
        }
    }

    out
}

pub async fn fetch_distribution_page(
    client: &reqwest::Client,
    cancel: CancellationToken,
    endpoint: &Endpoint,
    token: &str,
    character_id: u32,
    offset: u32,
    limit: u32,
) -> Result<AltClientDistributionPage, server::Error> {
    if token.trim().is_empty() || character_id == 0 {
        return Ok(AltClientDistributionPage {
            character_id,
            offset,
            limit,
            ..Default::default()
        });
    }

    let path = format!(
        "/v2/altclient/characters/{}/distributions?offset={}&limit={}",
        character_id, offset, limit
    );
    let request = client.get(endpoint.get_url(&path)).bearer_auth(token);
    server::JsonRequest::<AltClientDistributionPage>::new(request, cancel)
        .send()
        .await
}
