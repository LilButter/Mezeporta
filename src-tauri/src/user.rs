use std::collections::HashMap;
use std::env;

use crate::endpoint::Endpoint;
use log::warn;
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "Mezeporta 1.5.2";

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub username: String,
    pub remember_me: bool,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone)]
pub struct UserManager {
    #[serde(default)]
    data: [HashMap<String, UserData>; 2],
    #[serde(default)]
    passwords: [HashMap<String, String>; 2],
}

impl UserManager {
    fn use_keyring() -> bool {
        if let Ok(value) = env::var("MEZEPORTA_USE_KEYRING") {
            let value = value.to_lowercase();
            return value == "1" || value == "true" || value == "yes";
        }
        false
    }

    fn get_target(&self, endpoint: &'_ Endpoint) -> String {
        format!("{}:{}", endpoint.name, endpoint.is_remote)
    }

    pub fn get(&self, endpoint: &'_ Endpoint) -> (UserData, String) {
        let target = self.get_target(endpoint);
        let data = &self.data[endpoint.is_remote as usize];
        let userdata = data
            .get(&endpoint.name)
            .cloned()
            .unwrap_or_else(|| UserData {
                username: "".into(),
                remember_me: true,
            });
        let password = if !userdata.username.is_empty() && Self::use_keyring() {
            keyring::Entry::new_with_target(&target, APP_NAME, &userdata.username)
                .and_then(|entry| entry.get_password())
                .unwrap_or_else(|e| {
                    warn!("failed to get user password: {}", e);
                    "".to_owned()
                })
        } else {
            self.passwords[endpoint.is_remote as usize]
                .get(&endpoint.name)
                .cloned()
                .unwrap_or_default()
        };
        (userdata, password)
    }

    pub fn set(&mut self, endpoint: &'_ Endpoint, userdata: UserData, password: String) {
        let target = self.get_target(endpoint);
        let data = &mut self.data[endpoint.is_remote as usize];
        let passwords = &mut self.passwords[endpoint.is_remote as usize];
        let entry = keyring::Entry::new_with_target(&target, APP_NAME, &userdata.username);
        if userdata.remember_me {
            if Self::use_keyring() {
                entry
                    .and_then(|entry| entry.set_password(&password))
                    .unwrap_or_else(|e| warn!("failed to save password: {}", e));
            } else {
                passwords.insert(endpoint.name.to_owned(), password);
            }
            data.insert(endpoint.name.to_owned(), userdata);
        } else {
            if Self::use_keyring() {
                entry
                    .and_then(|entry| entry.delete_password())
                    .unwrap_or_else(|e| warn!("failed to save password: {}", e));
            }
            passwords.remove(&endpoint.name);
            data.remove(&endpoint.name);
        }
    }
}
