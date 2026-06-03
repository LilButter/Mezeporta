use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    GamePath,
    Mutex,
    GlobalAlloc,
    Dll,
    FontMissing,
    FontBootstrap,
    ProcNotFound,
    ThreadJoin,
    TokenLength,
    IniMissing,
    UnsupportedPlatform,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GamePath => write!(f, "unable to find path to game"),
            Self::Mutex => write!(f, "unable to create or free game mutexes"),
            Self::GlobalAlloc => write!(f, "unable to create or free game global alloc"),
            Self::Dll => write!(f, "unable to load or free mhfo dll"),
            Self::FontMissing => write!(f, "required S7K font file is missing"),
            Self::FontBootstrap => write!(f, "unable to prepare or register the S7K font"),
            Self::ProcNotFound => write!(f, "unable to find mhDLL_Main proc in mhfo-hd.dll"),
            Self::ThreadJoin => write!(f, "game or injector thread panicked"),
            Self::TokenLength => write!(f, "user token must have a length of 16"),
            Self::IniMissing => write!(f, "ini file not found in the game folder"),
            Self::UnsupportedPlatform => {
                write!(f, "meze-butter runtime is only supported on Windows targets")
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
