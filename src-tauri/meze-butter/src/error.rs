use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    GamePath,
    Mutex,
    GlobalAlloc,
    Dll,
    FontMissing,
    FontBootstrap,
    S6ReceiveBufferSetup,
    S6ConsoleOutputFiltering,
    S7KReceiveBufferSetup,
    S7KConsoleOutputFiltering,
    Z2TNameCheck,
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
            Self::S6ReceiveBufferSetup => {
                write!(f, "unable to prepare the S6")
            }
            Self::S6ConsoleOutputFiltering => {
                write!(f, "unable to configure S6")
            }
            Self::S7KReceiveBufferSetup => {
                write!(f, "unable to prepare the S7K")
            }
            Self::S7KConsoleOutputFiltering => {
                write!(f, "unable to configure S7K")
            }
            Self::Z2TNameCheck => {
                write!(f, "unable to prepare the Z2TW character-name")
            }
            Self::ProcNotFound => write!(f, "unable to find mhDLL_Main proc in mhfo-hd.dll"),
            Self::ThreadJoin => write!(f, "game thread panicked"),
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
