use std::io::{Read, Write};
use std::net::TcpStream;

use log::{info, warn};
use crate::cli::{CliAuthResponse, CliCharacterData, CliFriendData, CliMezFesData, CliUserData};

// ─── MHF Blowfish Crypto ────────────────────────────────────────────────────

const ENCRYPT_KEY: [u8; 256] = [
    0x90, 0x51, 0x26, 0x25, 0x04, 0xBF, 0xCF, 0x4C, 0x92, 0x02, 0x52, 0x7A, 0x70, 0x1A, 0x41,
    0x88, 0x8C, 0xC2, 0xCE, 0xB8, 0xF6, 0x57, 0x7E, 0xBA, 0x83, 0x63, 0x2C, 0x24, 0x9A, 0x67,
    0x86, 0x0C, 0xBE, 0x72, 0xFD, 0xB6, 0x7B, 0x79, 0xB0, 0x22, 0x5A, 0x60, 0x5C, 0x4F, 0x49,
    0xE2, 0x0E, 0xF5, 0x3A, 0x81, 0xAE, 0x11, 0x6B, 0xF0, 0xA1, 0x01, 0xE8, 0x65, 0x8D, 0x5B,
    0xDC, 0xCC, 0x93, 0x18, 0xB3, 0xAB, 0x77, 0xF7, 0x8E, 0xEC, 0xEF, 0x05, 0x00, 0xCA, 0x4E,
    0xA7, 0xBC, 0xB5, 0x10, 0xC6, 0x6C, 0xC0, 0xC4, 0xE5, 0x87, 0x3F, 0xC1, 0x82, 0x29, 0x96,
    0x45, 0x73, 0x07, 0xCB, 0x43, 0xF9, 0xF3, 0x08, 0x89, 0xD0, 0x99, 0x6A, 0x3B, 0x37, 0x19,
    0xD4, 0x40, 0xEA, 0xD7, 0x85, 0x16, 0x66, 0x1E, 0x9C, 0x39, 0xBB, 0xEE, 0x4A, 0x03, 0x8A,
    0x36, 0x2D, 0x13, 0x1D, 0x56, 0x48, 0xC7, 0x0D, 0x59, 0xB2, 0x44, 0xA3, 0xFE, 0x8B, 0x32,
    0x1B, 0x84, 0xA0, 0x2E, 0x62, 0x17, 0x42, 0xB9, 0x9B, 0x2B, 0x75, 0xD8, 0x1C, 0x3C, 0x4D,
    0x76, 0x27, 0x6E, 0x28, 0xD3, 0x33, 0xC3, 0x21, 0xAF, 0x34, 0x23, 0xDD, 0x68, 0x9F, 0xF1,
    0xAD, 0xE1, 0xB4, 0xE7, 0xA6, 0x74, 0x15, 0x4B, 0xFA, 0x3D, 0x5F, 0x7C, 0xDA, 0x2F, 0x0A,
    0xE3, 0x7D, 0xC8, 0xB7, 0x12, 0x6F, 0x9E, 0xA9, 0x14, 0x53, 0x97, 0x8F, 0x64, 0xF4, 0xF8,
    0xA2, 0xA4, 0x2A, 0xD2, 0x47, 0x9D, 0x71, 0xC5, 0xE9, 0x06, 0x98, 0x20, 0x54, 0x80, 0xAA,
    0xF2, 0xAC, 0x50, 0xD6, 0x7F, 0xD9, 0xC9, 0xCD, 0x69, 0x46, 0x6D, 0x30, 0xB1, 0x58, 0x0B,
    0x55, 0xD1, 0x5D, 0xD5, 0xBD, 0x31, 0xDE, 0xA5, 0xE4, 0x91, 0x0F, 0x61, 0x38, 0xDF, 0xA8,
    0xE6, 0x3E, 0x1F, 0x35, 0xED, 0xDB, 0x94, 0xEB, 0x09, 0x5E, 0x95, 0xFB, 0xFC, 0xE0, 0x78,
    0xFF,
];

const DECRYPT_KEY: [u8; 256] = [
    0x48, 0x37, 0x09, 0x76, 0x04, 0x47, 0xCC, 0x5C, 0x61, 0xF8, 0xB3, 0xE0, 0x1F, 0x7F, 0x2E,
    0xEB, 0x4E, 0x33, 0xB8, 0x7A, 0xBC, 0xAB, 0x6E, 0x8C, 0x3F, 0x68, 0x0D, 0x87, 0x93, 0x7B,
    0x70, 0xF2, 0xCE, 0x9D, 0x27, 0xA0, 0x1B, 0x03, 0x02, 0x97, 0x99, 0x58, 0xC5, 0x90, 0x1A,
    0x79, 0x8A, 0xB2, 0xDD, 0xE6, 0x86, 0x9B, 0x9F, 0xF3, 0x78, 0x67, 0xED, 0x72, 0x30, 0x66,
    0x94, 0xAE, 0xF1, 0x55, 0x6A, 0x0E, 0x8D, 0x5E, 0x82, 0x5A, 0xDB, 0xC7, 0x7D, 0x2C, 0x75,
    0xAC, 0x07, 0x95, 0x4A, 0x2B, 0xD4, 0x01, 0x0A, 0xBD, 0xCF, 0xE1, 0x7C, 0x15, 0xDF, 0x80,
    0x28, 0x3B, 0x2A, 0xE3, 0xF9, 0xAF, 0x29, 0xEC, 0x8B, 0x19, 0xC0, 0x39, 0x6F, 0x1D, 0xA2,
    0xDA, 0x65, 0x34, 0x50, 0xDC, 0x98, 0xB9, 0x0C, 0xC9, 0x21, 0x5B, 0xAA, 0x91, 0x96, 0x42,
    0xFE, 0x25, 0x0B, 0x24, 0xB0, 0xB5, 0x16, 0xD6, 0xD0, 0x31, 0x57, 0x18, 0x88, 0x6D, 0x1E,
    0x54, 0x0F, 0x62, 0x77, 0x85, 0x10, 0x3A, 0x44, 0xBF, 0x00, 0xEA, 0x08, 0x3E, 0xF6, 0xFA,
    0x59, 0xBE, 0xCD, 0x64, 0x1C, 0x8F, 0x71, 0xC8, 0xBA, 0xA3, 0x89, 0x36, 0xC3, 0x83, 0xC4,
    0xE8, 0xA9, 0x4B, 0xEF, 0xBB, 0xD1, 0x41, 0xD3, 0xA5, 0x32, 0x9E, 0x26, 0xDE, 0x81, 0x40,
    0xA7, 0x4D, 0x23, 0xB7, 0x13, 0x8E, 0x17, 0x73, 0x4C, 0xE5, 0x20, 0x05, 0x51, 0x56, 0x11,
    0x9C, 0x52, 0xCA, 0x4F, 0x7E, 0xB6, 0xD8, 0x49, 0x5D, 0x3D, 0xD9, 0x12, 0x06, 0x63, 0xE2,
    0xC6, 0x9A, 0x69, 0xE4, 0xD5, 0x6C, 0x92, 0xD7, 0xB1, 0xF5, 0x3C, 0xA1, 0xE7, 0xEE, 0xFD,
    0xA6, 0x2D, 0xB4, 0xE9, 0x53, 0xF0, 0xA8, 0x38, 0xCB, 0x6B, 0xF7, 0x45, 0xF4, 0x74, 0x46,
    0x35, 0xA4, 0xD2, 0x60, 0xC1, 0x2F, 0x14, 0x43, 0xC2, 0x5F, 0xAD, 0xFB, 0xFC, 0x22, 0x84,
    0xFF,
];

const SHARED_CRYPT_KEY: [u8; 256] = [
    0xDD, 0xA8, 0x5F, 0x1E, 0x57, 0xAF, 0xC0, 0xCC, 0x43, 0x35, 0x8F, 0xBB, 0x6F, 0xE6, 0xA1,
    0xD6, 0x60, 0xB9, 0x1A, 0xAE, 0x20, 0x49, 0x24, 0x81, 0x21, 0xFE, 0x86, 0x2B, 0x98, 0xB7,
    0xB3, 0xD2, 0x91, 0x01, 0x3A, 0x4C, 0x65, 0x92, 0x1C, 0xF4, 0xBE, 0xDD, 0xD9, 0x08, 0xE6,
    0x81, 0x98, 0x1B, 0x8D, 0x60, 0xF3, 0x6F, 0xA1, 0x47, 0x24, 0xF1, 0x53, 0x45, 0xC8, 0x7B,
    0x88, 0x80, 0x4E, 0x36, 0xC3, 0x0D, 0xC9, 0xD6, 0x8B, 0x08, 0x19, 0x0B, 0xA5, 0xC1, 0x11,
    0x4C, 0x60, 0xF8, 0x5D, 0xFC, 0x15, 0x68, 0x7E, 0x32, 0xC0, 0x50, 0xAB, 0x64, 0x1F, 0x8A,
    0xD4, 0x08, 0x39, 0x7F, 0xC2, 0xFB, 0xBA, 0x6C, 0xF0, 0xE6, 0xB0, 0x31, 0x10, 0xC1, 0xBF,
    0x75, 0x43, 0xBB, 0x18, 0x04, 0x0D, 0xD1, 0x97, 0xF7, 0x23, 0x21, 0x83, 0x8B, 0xCA, 0x25,
    0x2B, 0xA3, 0x03, 0x13, 0xEA, 0xAE, 0xFE, 0xF0, 0xEB, 0xFD, 0x85, 0x57, 0x53, 0x65, 0x41,
    0x2A, 0x40, 0x99, 0xC0, 0x94, 0x65, 0x7E, 0x7C, 0x93, 0x82, 0xB0, 0xB3, 0xE5, 0xC0, 0x21,
    0x09, 0x84, 0xD5, 0xEF, 0x9F, 0xD1, 0x7E, 0xDC, 0x4D, 0xF5, 0x7E, 0xCD, 0x45, 0x3C, 0x7F,
    0xF5, 0x59, 0x98, 0xC6, 0x55, 0xFC, 0x9F, 0xA3, 0xB7, 0x74, 0xEE, 0x31, 0x98, 0xE6, 0xB7,
    0xBE, 0x26, 0xF4, 0x3C, 0x76, 0xF1, 0x23, 0x7E, 0x02, 0x4E, 0x3C, 0xD1, 0xC7, 0x28, 0x23,
    0x73, 0xC4, 0xD9, 0x5E, 0x0D, 0xA1, 0x80, 0xA5, 0xAA, 0x26, 0x0A, 0xA3, 0x44, 0x82, 0x74,
    0xE6, 0x3C, 0x44, 0x27, 0x51, 0x0D, 0x5F, 0xC7, 0x9C, 0xD6, 0x63, 0x67, 0xA5, 0x27, 0x97,
    0x38, 0xFB, 0x2D, 0xD3, 0xD6, 0x60, 0x25, 0x83, 0x4D, 0x37, 0x5B, 0x40, 0x59, 0x11, 0x77,
    0x51, 0x11, 0x14, 0x18, 0x07, 0x63, 0xB1, 0x34, 0x3D, 0xB8, 0x60, 0x13, 0xC2, 0xE8, 0x13,
    0x82,
];

fn crypto(data: &[u8], rot_key: u32, encrypt: bool, override_byte: Option<u8>) -> (Vec<u8>, u16, u16, u16, u16) {
    let crypt_key_trunc_byte = if let Some(b) = override_byte {
        b
    } else {
        (((rot_key >> 1) % 999983) & 0xFF) as u8
    };

    let mut derived_crypt_key = ((data.len() as u32) * ((crypt_key_trunc_byte as u32) + 1)) & 0xFFFFFFFF;
    let mut shared_buf_idx: u8 = 1;
    let mut acc0: u32 = 0;
    let mut acc1: u32 = 0;
    let mut acc2: u32 = 0;

    let mut output: Vec<u8> = Vec::with_capacity(data.len());

    if encrypt {
        for i in 0..data.len() {
            let enc_key_idx = ((derived_crypt_key >> 10) ^ (data[i] as u32)) & 0xFF;
            derived_crypt_key = 1277u32.wrapping_mul(derived_crypt_key) + 1277;
            let enc_key_byte = ENCRYPT_KEY[enc_key_idx as usize];

            acc2 = acc2.wrapping_add((shared_buf_idx as u32) * (data[i] as u32));
            acc1 = acc1.wrapping_add(enc_key_idx);
            acc0 = acc0.wrapping_add((enc_key_byte as u32) << (i & 7));

            output.push(SHARED_CRYPT_KEY[shared_buf_idx as usize] ^ enc_key_byte);
            shared_buf_idx = data[i];
        }
    } else {
        for i in 0..data.len() {
            let old_shared_buf_idx = shared_buf_idx;
            let t_idx = data[i] ^ SHARED_CRYPT_KEY[shared_buf_idx as usize];
            let dec_key_byte = DECRYPT_KEY[t_idx as usize];
            shared_buf_idx = (((derived_crypt_key >> 10) ^ (dec_key_byte as u32)) & 0xFF) as u8;

            acc0 = acc0.wrapping_add((t_idx as u32) << (i & 7));
            acc1 = acc1.wrapping_add(dec_key_byte as u32);
            acc2 = acc2.wrapping_add((old_shared_buf_idx as u32) * (shared_buf_idx as u32));

            output.push(shared_buf_idx);

            derived_crypt_key = 1277u32.wrapping_mul(derived_crypt_key) + 1277;
        }
    }

    let combined_check = (acc1 + (acc0 >> 1) + (acc2 >> 2)) as u16;
    let check0 = (acc0 ^ ((acc0 & 0xFFFF_0000) >> 16)) as u16;
    let check1 = (acc1 ^ ((acc1 & 0xFFFF_0000) >> 16)) as u16;
    let check2 = (acc2 ^ ((acc2 & 0xFFFF_0000) >> 16)) as u16;

    (output, combined_check, check0, check1, check2)
}

// ─── Packet Header ──────────────────────────────────────────────────────────

const HEADER_LEN: usize = 14;

#[derive(Debug)]
#[allow(dead_code)]
struct PacketHeader {
    pf0: u8,
    key_rot_delta: u8,
    packet_num: u16,
    data_size: u16,
    prev_combined_check: u16,
    check0: u16,
    check1: u16,
    check2: u16,
}

fn read_header(stream: &mut TcpStream) -> Result<PacketHeader, String> {
    let mut buf = [0u8; HEADER_LEN];
    stream.read_exact(&mut buf).map_err(|e| format!("failed to read packet header: {e}"))?;

    Ok(PacketHeader {
        pf0: buf[0],
        key_rot_delta: buf[1],
        packet_num: u16::from_be_bytes([buf[2], buf[3]]),
        data_size: u16::from_be_bytes([buf[4], buf[5]]),
        prev_combined_check: u16::from_be_bytes([buf[6], buf[7]]),
        check0: u16::from_be_bytes([buf[8], buf[9]]),
        check1: u16::from_be_bytes([buf[10], buf[11]]),
        check2: u16::from_be_bytes([buf[12], buf[13]]),
    })
}

fn payload_size(header: &PacketHeader) -> usize {
    header.data_size as usize + ((header.pf0 as usize).wrapping_sub(0x03)) * 0x1000
}

fn read_packet(stream: &mut TcpStream, read_key_rot: &mut u32) -> Result<Vec<u8>, String> {
    let header = read_header(stream)?;
    let size = payload_size(&header);

    let mut encrypted = vec![0u8; size];
    stream.read_exact(&mut encrypted).map_err(|e| format!("failed to read packet body: {e}"))?;

    if header.key_rot_delta != 0 {
        *read_key_rot = (header.key_rot_delta as u32).wrapping_mul(read_key_rot.wrapping_add(1));
    }

    let (decrypted, _combined_check, c0, c1, c2) = crypto(&encrypted, *read_key_rot, false, None);

    if header.check0 != c0 || header.check1 != c1 || header.check2 != c2 {
        warn!(
            "crypto checksum mismatch: want c0={:04X} c1={:04X} c2={:04X}, got c0={:04X} c1={:04X} c2={:04X}",
            header.check0, header.check1, header.check2, c0, c1, c2
        );
        for key in 0u8..255 {
            let (alt, alt_c0, alt_c1, alt_c2, _) = crypto(&encrypted, 0, false, Some(key));
            if header.check0 == alt_c0 && header.check1 == alt_c1 && header.check2 == alt_c2 {
                info!("crypto bruteforce successful with override key: 0x{:02X}", key);
                return Ok(alt);
            }
        }
        return Err("crypto checksum mismatch — data out of sync".to_string());
    }

    Ok(decrypted)
}

fn send_packet(stream: &mut TcpStream, data: &[u8], send_key_rot: &mut u32) -> Result<(), String> {
    let key_rot_delta: u8 = 3;
    if key_rot_delta != 0 {
        *send_key_rot = (key_rot_delta as u32).wrapping_mul(send_key_rot.wrapping_add(1));
    }

    let (enc_data, _, c0, c1, c2) = crypto(data, *send_key_rot, true, None);

    let pf0 = (((enc_data.len() as usize) >> 12) & 0xF3) | 0x03 as usize;
    let data_size = enc_data.len() as u16;

    let header: [u8; HEADER_LEN] = [
        pf0 as u8,
        key_rot_delta,
        0, 0,
        (data_size >> 8) as u8, data_size as u8,
        0, 0,
        (c0 >> 8) as u8, c0 as u8,
        (c1 >> 8) as u8, c1 as u8,
        (c2 >> 8) as u8, c2 as u8,
    ];

    stream.write_all(&header).map_err(|e| format!("failed to write packet header: {e}"))?;
    stream.write_all(&enc_data).map_err(|e| format!("failed to write packet body: {e}"))?;
    stream.flush().map_err(|e| format!("failed to flush: {e}"))?;

    Ok(())
}

fn connect_and_init(addr: &str) -> Result<TcpStream, String> {
    let mut stream = TcpStream::connect(addr).map_err(|e| format!("failed to connect to {addr}: {e}"))?;
    stream.write_all(&[0u8; 8]).map_err(|e| format!("failed to write init bytes: {e}"))?;
    stream.flush().map_err(|e| format!("failed to flush init: {e}"))?;
    Ok(stream)
}

// ─── Binary Helpers (sign server uses BIG-ENDIAN) ───────────────────────────

fn read_u8<R: Read>(r: &mut R) -> Result<u8, String> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf).map_err(|_| "truncated data".to_string())?;
    Ok(buf[0])
}

fn read_u16_be<R: Read>(r: &mut R) -> Result<u16, String> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf).map_err(|_| "truncated data".to_string())?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32_be<R: Read>(r: &mut R) -> Result<u32, String> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf).map_err(|_| "truncated data".to_string())?;
    Ok(u32::from_be_bytes(buf))
}

fn read_bytes<R: Read>(r: &mut R, n: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; n];
    r.read_exact(&mut buf).map_err(|_| "truncated data".to_string())?;
    Ok(buf)
}

fn skip_bytes<R: Read>(r: &mut R, n: usize) -> Result<(), String> {
    let mut buf = vec![0u8; n];
    r.read_exact(&mut buf).map_err(|_| "truncated data".to_string())?;
    Ok(())
}

fn read_pascal_string(cursor: &mut std::io::Cursor<&[u8]>) -> Result<String, String> {
    let slen = read_u8(cursor)?;
    if slen == 0 {
        return Ok(String::new());
    }
    let data_len = (slen as usize).saturating_sub(1);
    let bytes = read_bytes(cursor, data_len)?;
    let _null_term = read_u8(cursor)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn skip_pascal_string(cursor: &mut std::io::Cursor<&[u8]>) {
    let slen = read_u8(cursor).unwrap();
    if slen > 0 {
        skip_bytes(cursor, (slen as usize).saturating_sub(1)).unwrap();
        skip_bytes(cursor, 1).unwrap();
    }
}

fn read_padded_string(cursor: &mut std::io::Cursor<&[u8]>, width: usize) -> Result<String, String> {
    let bytes = read_bytes(cursor, width)?;
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(width);
    let raw = &bytes[..len];
    Ok(String::from_utf8_lossy(&raw).to_string())
}

fn decode_sjis(bytes: &[u8]) -> String {
    let (s, _, was_malformed) = encoding_rs::SHIFT_JIS.decode(bytes);
    if was_malformed {
        String::from_utf8_lossy(bytes).to_string()
    } else {
        s.into_owned()
    }
}

fn encode_sjis(input: &str) -> Vec<u8> {
    let (cow, _, _) = encoding_rs::SHIFT_JIS.encode(input);
    cow.into_owned()
}

// ─── Sign Response Parser ───────────────────────────────────────────────────

/// Parse the binary sign server response into a CliAuthResponse.
/// Only parses what the launcher needs — the rest (caplink, PSN, filters) is for meze-deps.
fn parse_sign_response(data: &[u8]) -> Result<CliAuthResponse, String> {
    if data.is_empty() {
        return Err("empty sign response".to_string());
    }

    let mut cursor = std::io::Cursor::new(data);

    // Result code
    let result_code = read_u8(&mut cursor)?;
    if result_code != 1 {
        return Err(format!("sign failed with code {}", result_code));
    }

    // Server counts
    let _patch_count: u8 = read_u8(&mut cursor)?;
    let entrance_count: u8 = read_u8(&mut cursor)?;
    let char_count: u8 = read_u8(&mut cursor)?;

    // Token and timestamp
    let token_id = read_u32_be(&mut cursor)?;
    let mut token_bytes = [0u8; 16];
    cursor.read_exact(&mut token_bytes).map_err(|_| "truncated token")?;
    let token = String::from_utf8(token_bytes.to_vec()).unwrap_or_default();
    let current_ts = read_u32_be(&mut cursor)?;

    // Skip patch server URLs
    for _ in 0.._patch_count {
        skip_pascal_string(&mut cursor);
    }

    // Read entrance address (consumed for offset, not used by launcher)
    let _entrance_str = read_pascal_string(&mut cursor)?;

    // Read character entries
    let mut characters = Vec::with_capacity(char_count as usize);
    for _ in 0..char_count {
        let id = read_u32_be(&mut cursor)?;
        let hr = read_u16_be(&mut cursor)?;
        let weapon = read_u16_be(&mut cursor)?;
        let last_login = read_u32_be(&mut cursor)?;
        let is_female: bool = read_u8(&mut cursor)? != 0;
        let _is_new: u8 = read_u8(&mut cursor)?;
        let _old_gr: u8 = read_u8(&mut cursor)?;
        let _use_u16_gr: u8 = read_u8(&mut cursor)?;
        let name = read_padded_string(&mut cursor, 16)?;
        skip_bytes(&mut cursor, 32)?; // 32-byte unk desc
        let gr = read_u16_be(&mut cursor)?;
        let _unk1: u8 = read_u8(&mut cursor)?;
        let _unk2: u8 = read_u8(&mut cursor)?;

        characters.push(CliCharacterData {
            id,
            name,
            is_female,
            weapon: weapon.into(),
            hr: hr.into(),
            gr: gr.into(),
            last_login,
        });
    }

    // Read friends
    let mut friends = Vec::new();
    let friend_count_byte = read_u8(&mut cursor)?;
    if friend_count_byte == 255 {
        let _extended: u16 = read_u16_be(&mut cursor)?;
        warn!("sign response has >255 friends, skipping");
    } else {
        for _ in 0..friend_count_byte {
            let cid = read_u32_be(&mut cursor)?;
            let id = read_u32_be(&mut cursor)?;
            let name = read_pascal_string(&mut cursor)?;
            friends.push(CliFriendData { cid, id, name });
        }
    }
    info!("sign response: friend_count={}", friends.len());

    // Skip guildmates
    let _guildmate_count_byte = read_u8(&mut cursor)?;
    if _guildmate_count_byte == 255 {
        let _extended: u16 = read_u16_be(&mut cursor)?;
        let _ = _extended;
    } else {
        for _ in 0.._guildmate_count_byte {
            let _cid = read_u32_be(&mut cursor)?;
            let _id = read_u32_be(&mut cursor)?;
            let _name = read_pascal_string(&mut cursor)?;
        }
    }

    // Notices
    let mut notices = Vec::new();
    let has_notice: bool = read_u8(&mut cursor)? != 0;
    if has_notice {
        let _pad1: u8 = read_u8(&mut cursor)?;
        let _pad2: u8 = read_u8(&mut cursor)?;
        let notice_len = read_u16_be(&mut cursor)?;
        if notice_len > 0 {
            let notice_bytes = read_bytes(&mut cursor, (notice_len as usize).saturating_sub(1))?;
            notices.push(decode_sjis(&notice_bytes));
        }
    }

    // Last CID and rights
    let last_cid: u32 = read_u32_be(&mut cursor)?;
    let rights = read_u32_be(&mut cursor)?;
    let _ = (last_cid, rights);

    // Skip filters blob (uint16 BE length)
    let filter_start = cursor.position();
    let filter_len = read_u16_be(&mut cursor)?;
    let filter_end = filter_start + 2 + filter_len as u64;
    if filter_end > data.len() as u64 {
        return Err(format!("truncated filters: filter ends at {} but data is only {} bytes", filter_end, data.len()));
    }
    cursor.set_position(filter_end);

    // Skip everything between filters and mezFes (caplink, PSN, etc.)
    // mezFes is always at the end: expiry_ts + extra_zero + event_id + start + end + ticket_count + tickets + stall_count + stalls = 38 bytes
    let mez_fes_offset = data.len() as u64 - 38;
    cursor.set_position(mez_fes_offset);

    // expiry_ts and extra_zero
    let expiry_ts = read_u32_be(&mut cursor)?;
    let _extra_zero: u32 = read_u32_be(&mut cursor)?;

    // mezFes
    let mez_event_id = read_u32_be(&mut cursor)?;
    let mez_start = read_u32_be(&mut cursor)?;
    let mez_end = read_u32_be(&mut cursor)?;
    let ticket_count: u8 = read_u8(&mut cursor)?;
    let mut solo_tickets = 0u32;
    let mut group_tickets = 0u32;
    for i in 0..ticket_count {
        let val = read_u32_be(&mut cursor)?;
        if i == 0 {
            solo_tickets = val;
        } else if i == 1 {
            group_tickets = val;
        }
    }
    let _stall_count: u8 = read_u8(&mut cursor)?;
    let mut stalls = Vec::new();
    for _ in 0.._stall_count {
        stalls.push(read_u8(&mut cursor)? as u32);
    }

    Ok(CliAuthResponse {
        current_ts,
        expiry_ts,
        entrance_count: entrance_count as u32,
        notices,
        user: CliUserData {
            token_id,
            token,
            rights,
        },
        characters,
        mez_fez: Some(CliMezFesData {
            id: mez_event_id,
            start: mez_start,
            end: mez_end,
            solo_tickets,
            group_tickets,
            stalls,
        }),
        friends,
        patch_server: String::new(),
    })
}

// ─── Public API ─────────────────────────────────────────────────────────────

pub fn sign_auth(host: &str, port: u16, username: &str, password: &str) -> Result<CliAuthResponse, String> {
    let addr = format!("{}:{}", host, port);
    info!("sign server: connecting to {}", addr);

    let mut stream = connect_and_init(&addr)?;

    let mut send_key_rot: u32 = 995117;
    let mut read_key_rot: u32 = 995117;

    let req_type = "DSGN:041";
    let user_sjis = encode_sjis(username);
    let pass_sjis = encode_sjis(password);

    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(req_type.as_bytes());
    payload.push(0);
    payload.extend_from_slice(&user_sjis);
    payload.push(0);
    payload.extend_from_slice(&pass_sjis);
    payload.push(0);
    payload.push(0);

    send_packet(&mut stream, &payload, &mut send_key_rot)?;
    info!("sign server: DSGN packet sent");

    let resp_data = read_packet(&mut stream, &mut read_key_rot)?;

    parse_sign_response(&resp_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_encrypt_key_0() {
        let data = vec![0x74, 0x65, 0x73, 0x74];
        let (out, cc, c0, c1, c2) = crypto(&data, 0, true, None);
        assert_eq!(out, vec![0x46, 0x53, 0x28, 0x5E]);
        assert_eq!(cc, 0x2976);
        assert_eq!(c0, 0x06ea);
        assert_eq!(c1, 0x0215);
        assert_eq!(c2, 0x8fb3);
    }

    #[test]
    fn crypto_encrypt_key_3() {
        let data = vec![0x74, 0x65, 0x73, 0x74];
        let (out, cc, c0, c1, c2) = crypto(&data, 3, true, None);
        assert_eq!(out, vec![0x46, 0x95, 0x88, 0xEA]);
        assert_eq!(cc, 0x2ae4);
        assert_eq!(c0, 0x0a56);
        assert_eq!(c1, 0x01cd);
        assert_eq!(c2, 0x8fb3);
    }

    #[test]
    fn crypto_encrypt_key_max() {
        let data = vec![0x74, 0x65, 0x73, 0x74];
        let (out, cc, c0, c1, c2) = crypto(&data, 0xFFFFFFFF, true, None);
        assert_eq!(out, vec![0x46, 0xB5, 0xDC, 0xB2]);
        assert_eq!(cc, 0x2add);
        assert_eq!(c0, 0x09a6);
        assert_eq!(c1, 0x021e);
        assert_eq!(c2, 0x8fb3);
    }

    #[test]
    fn crypto_roundtrip_dsgn() {
        let data = b"DSGN:041\x00test\x00test\x00\x00";
        let (enc, _, _, _, _) = crypto(data, 995117, true, None);
        let (dec, _, _, _, _) = crypto(&enc, 995117, false, None);
        assert_eq!(dec, data.to_vec());
    }

    #[test]
    fn crypto_roundtrip_key_rotation() {
        let data = b"DSGN:041\x00test\x00test\x00\x00";
        let rotated_key = 3u32.wrapping_mul(995117u32.wrapping_add(1));
        let (enc, _, _, _, _) = crypto(data, rotated_key, true, None);
        let (dec, _, _, _, _) = crypto(&enc, rotated_key, false, None);
        assert_eq!(dec, data.to_vec());
    }

    #[test]
    fn encode_sjis_basic() {
        assert_eq!(encode_sjis("test"), vec![0x74, 0x65, 0x73, 0x74]);
    }

    #[test]
    fn encode_sjis_japanese() {
        let sjis = encode_sjis("こんにちは");
        assert!(!sjis.is_empty());
        assert_eq!(decode_sjis(&sjis), "こんにちは");
    }
}
