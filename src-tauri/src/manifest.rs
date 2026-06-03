//! Handles per-server manifest bookkeeping.
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub const MANIFEST_DIR: &str = "Mezeporta/Manifests";

fn sanitize_server_key(server: &str) -> String {
    server
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

#[derive(Default, Serialize, Deserialize)]
pub struct Manifest {
    pub modified_files: Vec<String>,
    pub added_files: Vec<String>,
}

impl Manifest {
    pub fn path(root: &Path, server: &str) -> PathBuf {
        root.join(MANIFEST_DIR)
            .join(format!("{}.json", sanitize_server_key(server)))
    }

    pub fn load(root: &Path, server: &str) -> Self {
        fs::read_to_string(Self::path(root, server))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, root: &Path, server: &str) -> io::Result<()> {
        let dir = root.join(MANIFEST_DIR);
        fs::create_dir_all(&dir)?;

        let server_key = sanitize_server_key(server);
        let tmp = dir.join(format!("{server_key}.json.tmp"));
        let final_ = dir.join(format!("{server_key}.json"));

        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        if final_.exists() {
            fs::remove_file(&final_)?;
        }
        fs::rename(tmp, final_)
    }
}
