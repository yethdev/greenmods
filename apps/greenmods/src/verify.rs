use bytes::Bytes;
use std::{
    ffi::OsStr,
    io::{Cursor, Read, Write},
    path::Path,
    process::Command,
};
use zip::ZipArchive;

const MIN_UPLOAD_BYTES: usize = 32;
const MAX_UPLOAD_BYTES: usize = 512 * 1024 * 1024;
const MAX_ARCHIVE_DEPTH: usize = 4;
const MAX_ARCHIVE_ENTRIES: usize = 512;
const MAX_EXPANDED_ARCHIVE_BYTES: u64 = 1024 * 1024 * 1024;
const ZIP_LOCAL_FILE: &[u8] = b"PK\x03\x04";
const ZIP_EMPTY_ARCHIVE: &[u8] = b"PK\x05\x06";
const UNREAL_PAK_MAGIC: &[u8] = &[0xe1, 0x12, 0x6f, 0x5a];
const EICAR: &[u8] =
    b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

const BLOCKED_EXTENSIONS: &[&str] = &[
    "app", "bat", "cmd", "com", "cpl", "dll", "dylib", "elf", "exe", "hta", "jar", "jse",
    "lnk", "msi", "pif", "ps1", "scr", "sh", "so", "sys", "vb", "vbe", "vbs", "wsf",
];

const BLOCKED_NAMES: &[&str] = &["autorun.inf", "desktop.ini", "thumbs.db", "vbaProject.bin"];

pub fn verify_upload(bytes: Bytes) -> bool {
    let len = bytes.len();

    if !(MIN_UPLOAD_BYTES..=MAX_UPLOAD_BYTES).contains(&len) {
        return false;
    }

    if bytes.iter().all(|b| *b == 0) {
        return false;
    }

    if !scan_payload("upload", &bytes, 0) {
        return false;
    }

    has_zip_header(&bytes) || has_unreal_pak_footer(&bytes) || has_mixed_bytes(&bytes)
}

fn scan_payload(name: &str, bytes: &[u8], depth: usize) -> bool {
    if depth > MAX_ARCHIVE_DEPTH {
        return false;
    }

    if matches!(scan_with_clamav(bytes), Some(false)) {
        return false;
    }

    if bytes.windows(EICAR.len()).any(|window| window == EICAR) {
        return false;
    }

    if is_native_executable(bytes) {
        return false;
    }

    if is_blocked_name(name) {
        return false;
    }

    if has_zip_header(bytes) {
        return scan_zip(bytes, depth + 1);
    }

    true
}

fn scan_zip(bytes: &[u8], depth: usize) -> bool {
    let Ok(mut archive) = ZipArchive::new(Cursor::new(bytes)) else {
        return false;
    };

    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return false;
    }

    let mut expanded = 0_u64;

    for index in 0..archive.len() {
        let Ok(mut file) = archive.by_index(index) else {
            return false;
        };

        if file.is_dir() {
            continue;
        }

        let Some(path) = file.enclosed_name() else {
            return false;
        };

        expanded = expanded.saturating_add(file.size());

        if expanded > MAX_EXPANDED_ARCHIVE_BYTES {
            return false;
        }

        let path_text = path.to_string_lossy();

        if is_blocked_name(&path_text) {
            return false;
        }

        let mut data = Vec::new();

        if file.read_to_end(&mut data).is_err() {
            return false;
        }

        if !scan_payload(&path_text, &data, depth) {
            return false;
        }
    }

    true
}

fn scan_with_clamav(bytes: &[u8]) -> Option<bool> {
    let scanner = std::env::var("GREENMODS_CLAMSCAN")
        .ok()
        .filter(|value| !matches!(value.as_str(), "0" | "off" | "false" | "disabled"))
        .unwrap_or_else(|| "clamscan".into());

    let mut file = tempfile::Builder::new()
        .prefix("greenmods-upload-")
        .tempfile()
        .ok()?;

    file.write_all(bytes).ok()?;

    let output = Command::new(scanner)
        .arg("--no-summary")
        .arg(file.path())
        .output()
        .ok()?;

    match output.status.code() {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    }
}

fn is_blocked_name(name: &str) -> bool {
    let path = Path::new(name);
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    if BLOCKED_NAMES.iter().any(|blocked| *blocked == file_name) {
        return true;
    }

    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| {
            let ext = ext.to_ascii_lowercase();
            BLOCKED_EXTENSIONS.iter().any(|blocked| *blocked == ext)
        })
        .unwrap_or(false)
}

fn is_native_executable(bytes: &[u8]) -> bool {
    is_windows_pe(bytes) || bytes.starts_with(b"\x7fELF") || is_macho(bytes)
}

fn is_windows_pe(bytes: &[u8]) -> bool {
    if bytes.len() < 0x40 || !bytes.starts_with(b"MZ") {
        return false;
    }

    let offset = u32::from_le_bytes([bytes[0x3c], bytes[0x3d], bytes[0x3e], bytes[0x3f]]) as usize;

    bytes
        .get(offset..offset.saturating_add(4))
        .map(|magic| magic == b"PE\0\0")
        .unwrap_or(false)
}

fn is_macho(bytes: &[u8]) -> bool {
    matches!(
        bytes.get(..4),
        Some([0xfe, 0xed, 0xfa, 0xce])
            | Some([0xce, 0xfa, 0xed, 0xfe])
            | Some([0xfe, 0xed, 0xfa, 0xcf])
            | Some([0xcf, 0xfa, 0xed, 0xfe])
            | Some([0xca, 0xfe, 0xba, 0xbe])
    )
}

fn has_zip_header(bytes: &[u8]) -> bool {
    bytes.starts_with(ZIP_LOCAL_FILE) || bytes.starts_with(ZIP_EMPTY_ARCHIVE)
}

fn has_unreal_pak_footer(bytes: &[u8]) -> bool {
    let start = bytes.len().saturating_sub(512);

    bytes[start..]
        .windows(UNREAL_PAK_MAGIC.len())
        .any(|window| window == UNREAL_PAK_MAGIC)
}

fn has_mixed_bytes(bytes: &[u8]) -> bool {
    let sample_len = bytes.len().min(4096);
    let sample = &bytes[..sample_len];
    let mut seen = [false; 256];

    for byte in sample {
        seen[*byte as usize] = true;
    }

    seen.into_iter().filter(|v| *v).count() >= 16
}
