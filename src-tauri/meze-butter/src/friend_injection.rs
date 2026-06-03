use std::thread;
use std::time::Duration;

use crate::utils::bufcopy;
use crate::{FriendData, MhfVersion};
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

#[derive(Copy, Clone)]
struct FriendLayout {
    dll_name: &'static str,
    base_off: usize,
    id0_off: usize,
}

#[derive(Copy, Clone)]
struct FriendLayoutSignature {
    signature: &'static str,
    layout: FriendLayout,
}

const SEASON_FRIEND_LAYOUTS: &[FriendLayoutSignature] = &[
    FriendLayoutSignature {
        signature: "v1.13.3246",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x136502A0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v7.0.14_2",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x14A163E0,
            id0_off: 0x18,
        },
    },
];

const FORWARD_FRIEND_LAYOUTS: &[FriendLayoutSignature] = &[
    FriendLayoutSignature {
        signature: "v1.20_107869",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15711F40,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.20_125635",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x157B1480,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.20_133710",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x157CCA00,
            id0_off: 0x18,
        },
    },
];

const G_FRIEND_LAYOUTS: &[FriendLayoutSignature] = &[
    FriendLayoutSignature {
        signature: "v1.22_153077",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15B108A0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.22_156129",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15B079A0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.23_187828",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15B537C0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.27_211402",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x1548E960,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.27_212295",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x1548E960,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.27_213258",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15496CC0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.27_215335",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15496CC0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.27_217155",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15496CC0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.27_222273",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x154A0F20,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.27_223087",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x154A0F20,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.28_246880",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x1568DE00,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.30.283838",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x157216C0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.32_302094",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15782C80,
            id0_off: 0x18,
        },
    },
];

const LATE_FRIEND_LAYOUTS: &[FriendLayoutSignature] = &[
    FriendLayoutSignature {
        signature: "v1.33_325336",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x1581A680,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.33_326088",
        layout: FriendLayout {
            dll_name: "mhfo-hd.dll",
            base_off: 0x67EE2240,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.36.05_936940dd",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x67F173A0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.36.05_a924ce4d",
        layout: FriendLayout {
            dll_name: "mhfo-hd.dll",
            base_off: 0x67EC2C40,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.38.19_e8966870",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x67EED520,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.38.19_47c90390",
        layout: FriendLayout {
            dll_name: "mhfo-hd.dll",
            base_off: 0x67E99CE0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.41.30_c730c673",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15DCB840,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.41.30_f5ed3a6a",
        layout: FriendLayout {
            dll_name: "mhfo-hd.dll",
            base_off: 0x1EA07FE0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.41.32_8acc3715",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15DCD860,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.41.32_5c06b547",
        layout: FriendLayout {
            dll_name: "mhfo-hd.dll",
            base_off: 0x1EA0A020,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.44.45_15a73eb7",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x15F691A0,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.44.45_dca95f5f",
        layout: FriendLayout {
            dll_name: "mhfo-hd.dll",
            base_off: 0x1EBA3940,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.52.79_04d16dc4",
        layout: FriendLayout {
            dll_name: "mhfo.dll",
            base_off: 0x16142F20,
            id0_off: 0x18,
        },
    },
    FriendLayoutSignature {
        signature: "v1.52.79_73c49f52",
        layout: FriendLayout {
            dll_name: "mhfo-hd.dll",
            base_off: 0x1ED7D6C0,
            id0_off: 0x18,
        },
    },
];

const FRIEND_TABLE_SIZE: usize = 0x1000;
const FRIEND_ENTRY_SIZE: usize = 0x30;
const MAX_FRIENDS: usize = 50;
const LEAD_STEP: usize = 4;

const BASE32_CHARS: &[u8; 32] = b"123456789ABCDEFGHJKLMNPQRTUVWXYZ";
const BASE32_CAP: u32 = 32u32.pow(6);

fn friend_layout_signatures_for_version(version: MhfVersion) -> &'static [FriendLayoutSignature] {
    const EMPTY_FRIEND_LAYOUTS: &[FriendLayoutSignature] = &[];
    match version {
        MhfVersion::S6 | MhfVersion::S7K => SEASON_FRIEND_LAYOUTS,
        MhfVersion::F4 | MhfVersion::F5 => FORWARD_FRIEND_LAYOUTS,
        MhfVersion::G1
        | MhfVersion::G2
        | MhfVersion::G3
        | MhfVersion::G3_1
        | MhfVersion::G3_2
        | MhfVersion::GG
        | MhfVersion::G5_1
        | MhfVersion::G5_2 => G_FRIEND_LAYOUTS,
        MhfVersion::G5 => EMPTY_FRIEND_LAYOUTS,
        MhfVersion::G6
        | MhfVersion::G7
        | MhfVersion::G9_1
        | MhfVersion::G10_1
        | MhfVersion::Z1
        | MhfVersion::ZZ => LATE_FRIEND_LAYOUTS,
        MhfVersion::Z2 | MhfVersion::Z2T => EMPTY_FRIEND_LAYOUTS,
    }
}

#[inline]
fn make_ext_id(mut id: u32) -> String {
    debug_assert!(id < BASE32_CAP, "ext_id overflow: {}", id);
    let mut out = [b'1'; 6];
    for byte in &mut out {
        *byte = BASE32_CHARS[(id % 32) as usize];
        id /= 32;
        if id == 0 {
            break;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn resolve(l: FriendLayout) -> Option<usize> {
    if l.base_off >= 0x1000_0000 {
        return Some(l.base_off);
    }
    unsafe {
        let c = std::ffi::CString::new(l.dll_name).ok()?;
        GetModuleHandleA(PCSTR(c.as_ptr() as _))
            .ok()
            .map(|h| h.0 as usize + l.base_off)
    }
}

fn find_layout_for_signature(
    signature: &str,
    dll_name: Option<&str>,
    layouts: &[FriendLayoutSignature],
) -> Option<FriendLayout> {
    layouts
        .iter()
        .find(|candidate| {
            dll_name
                .map(|name| candidate.layout.dll_name == name)
                .unwrap_or(true)
                && candidate.signature.eq_ignore_ascii_case(signature)
        })
        .map(|candidate| candidate.layout)
}

#[inline]
unsafe fn inject_blob(buf: &mut [u8], id0_off: usize, friends: &[FriendData]) {
    let slots = friends
        .len()
        .min(MAX_FRIENDS)
        .min(buf.len() / FRIEND_ENTRY_SIZE);

    let header_sz = if id0_off == 0x20 { 0x08 } else { 0x00 };

    for (i, f) in friends.iter().take(slots).enumerate() {
        let base = i * FRIEND_ENTRY_SIZE;
        let lead = header_sz + i * LEAD_STEP;
        let id_off = header_sz + id0_off + i * LEAD_STEP;
        let tz_end = base + id_off;
        let mut cur = base + lead;

        let clear_len = (id_off + 4) - lead;
        std::ptr::write_bytes(buf.as_mut_ptr().add(base + lead), 0, clear_len);

        if header_sz != 0 && i != 0 {
            buf[base + 3] = 0x01;
            buf[base + 4] = 0x01;
        }

        let ext = make_ext_id(f.id);
        let n = ext.len().min(tz_end - cur);
        bufcopy(&mut buf[cur..cur + n], ext.as_bytes());
        cur += n;

        if cur < tz_end {
            buf[cur] = 0;
            cur += 1;
        }
        if cur < tz_end {
            buf[cur] = 0;
            cur += 1;
        }

        if cur < tz_end {
            let name = f.name.as_bytes();
            let m = name.len().min(tz_end - cur - 1);
            bufcopy(&mut buf[cur..cur + m], &name[..m]);
            cur += m;
            if cur < tz_end {
                buf[cur] = 0;
            }
        }

        let id_pos = base + id_off;
        bufcopy(&mut buf[id_pos..id_pos + 4], &f.id.to_le_bytes());
    }
}

fn wait_and_inject(layout: FriendLayout, friends: &[FriendData]) -> bool {
    let base = match resolve(layout) {
        Some(p) => p,
        None => {
            return false;
        }
    };
    let hdr = if layout.id0_off == 0x20 { 8 } else { 0 };

    let table_ready;
    unsafe {
        let mut tries = 0;
        while {
            let blk = std::slice::from_raw_parts((base + hdr) as *const u8, 0x20);
            blk.chunks(4)
                .take(8)
                .any(|c| c.len() == 4 && u32::from_le_bytes([c[0], c[1], c[2], c[3]]) == 0)
                && {
                    tries += 1;
                    tries <= 800
                }
        } {
            thread::sleep(Duration::from_millis(10));
        }

        table_ready = {
            let blk = std::slice::from_raw_parts((base + hdr) as *const u8, 0x20);
            !blk.chunks(4)
                .take(8)
                .any(|c| c.len() == 4 && u32::from_le_bytes([c[0], c[1], c[2], c[3]]) == 0)
        };

        let mut old = PAGE_PROTECTION_FLAGS(0);
        let _ = VirtualProtect(
            base as _,
            FRIEND_TABLE_SIZE,
            PAGE_EXECUTE_READWRITE,
            &mut old,
        );

        inject_blob(
            std::slice::from_raw_parts_mut(base as *mut u8, FRIEND_TABLE_SIZE),
            layout.id0_off,
            friends,
        );

        let _ = VirtualProtect(
            base as _,
            FRIEND_TABLE_SIZE,
            old,
            &mut PAGE_PROTECTION_FLAGS(0),
        );
    }

    table_ready
}

pub(crate) fn maybe_inject_friends(
    version: MhfVersion,
    friend_layout_dll_name: &'static str,
    selected_friend_signature: Option<&str>,
    friends: &[FriendData],
) -> bool {
    let Some(signature) = selected_friend_signature else {
        return false;
    };
    let friend_layout_signatures = friend_layout_signatures_for_version(version);
    if let Some(layout) = find_layout_for_signature(
        signature,
        Some(friend_layout_dll_name),
        friend_layout_signatures,
    )
    .or_else(|| find_layout_for_signature(signature, None, friend_layout_signatures))
    {
        return wait_and_inject(layout, friends);
    }

    false
}


