use windows::{
    core::HSTRING,
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL},
        System::{
            Memory::{GlobalAlloc, GLOBAL_ALLOC_FLAGS},
            Threading::{CreateMutexW, OpenMutexW, SYNCHRONIZATION_ACCESS_RIGHTS},
        },
    },
};

use crate::{Error, MhfVersion, Result};

pub fn bufcopy<T: Copy>(s: &mut [T], v: &[T]) {
    let l = s.len().min(v.len());
    s[..l].copy_from_slice(&v[..l])
}

pub fn get_mutex_name(version: MhfVersion, s: &str) -> String {
    let pid = std::process::id();
    let title_prefix = match version {
        MhfVersion::S6 | MhfVersion::S7K | MhfVersion::F4 | MhfVersion::F5 => {
            "MONSTER HUNTER FRONTIER ONLINE"
        }
        MhfVersion::G1
        | MhfVersion::G2
        | MhfVersion::G3
        | MhfVersion::G3_1
        | MhfVersion::G3_2
        | MhfVersion::GG
        | MhfVersion::G5
        | MhfVersion::G5_1
        | MhfVersion::G5_2
        | MhfVersion::G6
        | MhfVersion::G7
        | MhfVersion::G9_1
        | MhfVersion::G10_1 => "MONSTER HUNTER FRONTIER G",
        MhfVersion::Z1 | MhfVersion::Z2 | MhfVersion::Z2T | MhfVersion::ZZ => "MONSTER HUNTER FRONTIER Z",
    };

    format!("{title_prefix} {s} {pid}")
}

pub fn create_mutex(name: impl Into<HSTRING>) -> Result<HANDLE> {
    unsafe { CreateMutexW(None, false, &name.into()) }.or(Err(Error::Mutex))
}

pub fn get_or_create_mutex(name: impl Into<HSTRING> + Copy) -> Result<HANDLE> {
    unsafe { OpenMutexW(SYNCHRONIZATION_ACCESS_RIGHTS(0x1F0001), false, &name.into()) }
        .or_else(|_| create_mutex(name))
        .or(Err(Error::Mutex))
}

pub fn create_global_alloc() -> Result<HGLOBAL> {
    unsafe { GlobalAlloc(GLOBAL_ALLOC_FLAGS(0x42), 0x8ae0) }.or(Err(Error::GlobalAlloc))
}

pub fn release_global_alloc(handle: HGLOBAL) -> Result<HGLOBAL> {
    unsafe { GlobalFree(handle) }.or(Err(Error::GlobalAlloc))
}



