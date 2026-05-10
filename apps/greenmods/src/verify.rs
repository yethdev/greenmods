use bytes::Bytes;
use std::{
    ffi::OsStr,
    io::{Cursor, Read, Write},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use zip::ZipArchive;

const MIN_UPLOAD_BYTES: usize = 32;
const MAX_UPLOAD_BYTES: usize = 512 * 1024 * 1024;
const MAX_ARCHIVE_DEPTH: usize = 4;
const MAX_ARCHIVE_ENTRIES: usize = 512;
const MAX_EXPANDED_ARCHIVE_BYTES: u64 = 1024 * 1024 * 1024;
const DEFAULT_CLAMSCAN_TIMEOUT: Duration = Duration::from_secs(60);
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

    let allow_archive_binary = is_allowed_archive_binary(name, depth);

    if matches!(scan_with_clamav(bytes), Some(false)) {
        return false;
    }

    if bytes.windows(EICAR.len()).any(|window| window == EICAR) {
        return false;
    }

    if is_native_executable(bytes) && !allow_archive_binary {
        return false;
    }

    if is_blocked_name(name) && !allow_archive_binary {
        return false;
    }

    if has_zip_header(bytes) {
        return scan_zip(bytes, depth + 1);
    }

    true
}

fn is_allowed_archive_binary(name: &str, depth: usize) -> bool {
    depth > 0
        && Path::new(name)
            .extension()
            .and_then(OsStr::to_str)
            .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "dll" | "so" | "dylib" | "exe"))
            .unwrap_or(false)
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

        if is_blocked_name(&path_text) && !is_allowed_archive_binary(&path_text, depth) {
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
    let timeout = std::env::var("GREENMODS_CLAMSCAN_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_CLAMSCAN_TIMEOUT);

    let mut file = tempfile::Builder::new()
        .prefix("greenmods-upload-")
        .tempfile()
        .ok()?;

    file.write_all(bytes).ok()?;

    let mut child = Command::new(scanner)
        .arg("--no-summary")
        .arg(file.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + timeout;

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(200)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };

    match status.code() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn fake_pe_bytes() -> Vec<u8> {
        let mut bytes = vec![0_u8; 0x100];
        bytes[0..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&(0x80_u32).to_le_bytes());
        bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
        bytes
    }

    fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());

        {
            let mut writer = ZipWriter::new(&mut cursor);
            let options = SimpleFileOptions::default();

            for (name, payload) in entries {
                writer.start_file(*name, options).unwrap();
                writer.write_all(payload).unwrap();
            }

            writer.finish().unwrap();
        }

        cursor.into_inner()
    }

    #[test]
    fn allows_archive_dlls() {
        let zip = build_zip(&[("mod/example.dll", &fake_pe_bytes())]);
        assert!(verify_upload(bytes::Bytes::from(zip)));
    }

    #[test]
    fn allows_archive_exes() {
        let zip = build_zip(&[("mod/example.exe", &fake_pe_bytes())]);
        assert!(verify_upload(bytes::Bytes::from(zip)));
    }

    #[test]
    fn blocks_archive_scripts() {
        let zip = build_zip(&[("mod/example.ps1", b"Write-Host 'hello'")]);
        assert!(!verify_upload(bytes::Bytes::from(zip)));
    }
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
