use bytes::Bytes;

const MIN_UPLOAD_BYTES: usize = 32;
const MAX_UPLOAD_BYTES: usize = 512 * 1024 * 1024;
const ZIP_LOCAL_FILE: &[u8] = b"PK\x03\x04";
const ZIP_EMPTY_ARCHIVE: &[u8] = b"PK\x05\x06";
const UNREAL_PAK_MAGIC: &[u8] = &[0xe1, 0x12, 0x6f, 0x5a];

pub fn verify_upload(bytes: Bytes) -> bool {
    let len = bytes.len();

    if !(MIN_UPLOAD_BYTES..=MAX_UPLOAD_BYTES).contains(&len) {
        return false;
    }

    if bytes.iter().all(|b| *b == 0) {
        return false;
    }

    has_zip_header(&bytes) || has_unreal_pak_footer(&bytes) || has_mixed_bytes(&bytes)
}

fn has_zip_header(bytes: &Bytes) -> bool {
    bytes.starts_with(ZIP_LOCAL_FILE) || bytes.starts_with(ZIP_EMPTY_ARCHIVE)
}

fn has_unreal_pak_footer(bytes: &Bytes) -> bool {
    let start = bytes.len().saturating_sub(512);

    bytes[start..]
        .windows(UNREAL_PAK_MAGIC.len())
        .any(|window| window == UNREAL_PAK_MAGIC)
}

fn has_mixed_bytes(bytes: &Bytes) -> bool {
    let sample_len = bytes.len().min(4096);
    let sample = &bytes[..sample_len];
    let mut seen = [false; 256];

    for byte in sample {
        seen[*byte as usize] = true;
    }

    seen.into_iter().filter(|v| *v).count() >= 16
}
