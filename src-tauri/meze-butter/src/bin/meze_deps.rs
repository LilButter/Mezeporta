#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::engine::general_purpose;
use base64::Engine;
use meze_butter::MhfConfig;
use std::io::Read;
use std::process::exit;

fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_UNAWARE_GDISCALED,
        };

        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_UNAWARE_GDISCALED);
    }

    let stdin_b64 = std::env::args().any(|arg| arg == "--stdin-b64");
    if !stdin_b64 {
        eprintln!("usage: meze-deps.exe --stdin-b64");
        exit(1);
    }

    let mut stdin = String::new();
    std::io::stdin().read_to_string(&mut stdin).unwrap_or_else(|e| {
        eprintln!("error reading stdin: {e}");
        exit(2);
    });

    let decoded = general_purpose::STANDARD
        .decode(stdin.trim())
        .unwrap_or_else(|e| {
            eprintln!("error decoding base64 stdin: {e}");
            exit(3);
        });

    let config_text = String::from_utf8(decoded).unwrap_or_else(|e| {
        eprintln!("error parsing config text: {e}");
        exit(4);
    });

    let config: MhfConfig = serde_json::from_str(&config_text).unwrap_or_else(|e| {
        eprintln!("error parsing config data: {e}");
        exit(5);
    });

    if let Err(e) = meze_butter::run(config) {
        eprintln!("error running meze-deps: {e}");
        exit(6);
    }
}
