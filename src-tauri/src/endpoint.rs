use std::{collections::HashMap, path::PathBuf};

use meze_butter as mhf_iel;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub game_folder: Option<PathBuf>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub url: String,
    pub name: String,
    pub launcher_port: Option<u16>,
    pub game_port: Option<u16>,
    pub game_folder: Option<PathBuf>,
    pub version: mhf_iel::MhfVersion,
    #[serde(default)]
    pub is_remote: bool,
}

impl PartialEq for Endpoint {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.is_remote == other.is_remote
    }
}

impl Endpoint {
    fn parsed_url(&self) -> Option<reqwest::Url> {
        if self.url.trim().is_empty() {
            return None;
        }

        let raw = if self.url.contains("://") {
            self.url.clone()
        } else {
            format!("http://{}", self.url)
        };

        reqwest::Url::parse(&raw).ok()
    }

    pub fn host(&self) -> String {
        self.parsed_url()
            .and_then(|url| url.host_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| self.url.to_owned())
    }

    fn resolved_launcher_port(&self) -> Option<u16> {
        self.launcher_port.or_else(|| {
            self.parsed_url()
                .and_then(|url| url.port_or_known_default())
        })
    }

    pub fn base_url(&self) -> String {
        if let Some(mut url) = self.parsed_url() {
            if let Some(port) = self.launcher_port {
                let _ = url.set_port(Some(port));
            }
            return url.origin().ascii_serialization();
        }

        if self.url.contains("://") {
            self.url.clone()
        } else {
            format!("http://{}", self.url)
        }
    }

    pub fn server_key(&self) -> String {
        let host = self.host();
        if host.is_empty() {
            return String::new();
        }

        if let Some(port) = self.resolved_launcher_port() {
            return format!("{}:{}", host, port);
        }

        host
    }

    pub fn get_url(&self, path: &str) -> String {
        let mut base = self.base_url();
        if path.is_empty() {
            return base;
        }

        if path.starts_with('/') {
            base.push_str(path);
        } else {
            base.push('/');
            base.push_str(path);
        }

        base
    }
}

pub trait EndpointVecExt {
    fn check_valid(&self) -> Result<(), &'static str>;
    fn extend_valid(&mut self, other: Self);
    fn apply_config(&mut self, configs: &HashMap<String, EndpointConfig>);
    fn update_config(&self, configs: &mut HashMap<String, EndpointConfig>);
}

impl EndpointVecExt for Vec<Endpoint> {
    fn check_valid(&self) -> Result<(), &'static str> {
        for endpoint in self {
            if endpoint.name.is_empty() {
                return Err("endpoint-name-empty");
            } else if endpoint.url.is_empty() {
                return Err("endpoint-host-empty");
            } else if self.iter().filter(|e| e.name == endpoint.name).count() > 1 {
                return Err("endpoint-unique");
            }
            if let Some(game_folder) = endpoint.game_folder.as_ref() {
                if !game_folder.exists() {
                    return Err("path-exists-error");
                }
            }
        }
        Ok(())
    }

    fn extend_valid(&mut self, other: Self) {
        self.reserve(other.len());
        for endpoint in other {
            if !endpoint.name.is_empty() && !endpoint.url.is_empty() && !self.contains(&endpoint) {
                self.push(endpoint)
            }
        }
    }

    fn apply_config(&mut self, configs: &HashMap<String, EndpointConfig>) {
        for endpoint in self {
            if let Some(config) = configs.get(&endpoint.name) {
                endpoint.game_folder = config.game_folder.clone();
            }
        }
    }

    fn update_config(&self, configs: &mut HashMap<String, EndpointConfig>) {
        for endpoint in self {
            if endpoint.game_folder.is_some() {
                configs.insert(
                    endpoint.name.clone(),
                    EndpointConfig {
                        game_folder: endpoint.game_folder.clone(),
                    },
                );
            } else {
                configs.remove(&endpoint.name);
            }
        }
    }
}
