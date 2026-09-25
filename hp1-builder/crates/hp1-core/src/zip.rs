use crate::{safe_relative_path, sha256_file, CoreError};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipEntry { pub path: String, pub size: u64, pub encrypted: bool }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipInspection { pub sha256: String, pub size: u64, pub entries: Vec<ZipEntry> }

/// Inspects the ZIP central directory without extracting or mounting it.
pub fn inspect_zip(path: &Path) -> Result<ZipInspection, CoreError> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() < 22 { return Err(CoreError::InvalidInput("Input is too small to be a ZIP file.".into())); }
    let mut file = std::fs::File::open(path)?;
    let tail_len = metadata.len().min(65_557) as usize;
    file.seek(SeekFrom::End(-(tail_len as i64)))?;
    let mut tail = vec![0; tail_len]; file.read_exact(&mut tail)?;
    let eocd = tail.windows(4).rposition(|window| window == b"PK\x05\x06").ok_or_else(|| CoreError::InvalidInput("ZIP end-of-central-directory record is missing.".into()))?;
    if eocd + 22 > tail.len() { return Err(CoreError::InvalidInput("ZIP end-of-central-directory record is truncated.".into())); }
    let entry_count = u16::from_le_bytes(tail[eocd + 10..eocd + 12].try_into().unwrap()) as usize;
    let central_size = u32::from_le_bytes(tail[eocd + 12..eocd + 16].try_into().unwrap()) as usize;
    let central_offset = u32::from_le_bytes(tail[eocd + 16..eocd + 20].try_into().unwrap()) as u64;
    if entry_count > 64 || central_size > 1024 * 1024 { return Err(CoreError::InvalidInput("ZIP directory exceeds builder safety limits.".into())); }
    file.seek(SeekFrom::Start(central_offset))?; let mut central = vec![0; central_size]; file.read_exact(&mut central)?;
    let mut entries = Vec::with_capacity(entry_count); let mut cursor = 0;
    for _ in 0..entry_count {
        if cursor + 46 > central.len() || &central[cursor..cursor + 4] != b"PK\x01\x02" { return Err(CoreError::InvalidInput("ZIP central directory is malformed.".into())); }
        let flags = u16::from_le_bytes(central[cursor + 8..cursor + 10].try_into().unwrap());
        let size = u32::from_le_bytes(central[cursor + 24..cursor + 28].try_into().unwrap()) as u64;
        let name_len = u16::from_le_bytes(central[cursor + 28..cursor + 30].try_into().unwrap()) as usize;
        let extra_len = u16::from_le_bytes(central[cursor + 30..cursor + 32].try_into().unwrap()) as usize;
        let comment_len = u16::from_le_bytes(central[cursor + 32..cursor + 34].try_into().unwrap()) as usize;
        let next = cursor.checked_add(46 + name_len + extra_len + comment_len).ok_or_else(|| CoreError::InvalidInput("ZIP entry size overflows.".into()))?;
        if next > central.len() { return Err(CoreError::InvalidInput("ZIP entry is truncated.".into())); }
        let name = std::str::from_utf8(&central[cursor + 46..cursor + 46 + name_len]).map_err(|_| CoreError::InvalidInput("ZIP has a non-UTF-8 path.".into()))?;
        safe_relative_path(Path::new(name)).map_err(|_| CoreError::InvalidInput(format!("ZIP contains an unsafe path: {name:?}")))?;
        entries.push(ZipEntry { path: name.into(), size, encrypted: flags & 1 != 0 }); cursor = next;
    }
    Ok(ZipInspection { sha256: sha256_file(path)?, size: metadata.len(), entries })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_a_safe_central_directory_without_extracting() {
        let path = std::env::temp_dir().join("hp1-core-safe-zip-test.zip");
        let name = b"top/file.bin";
        let mut central = vec![0; 46];
        central[..4].copy_from_slice(b"PK\x01\x02");
        central[24..28].copy_from_slice(&1u32.to_le_bytes());
        central[28..30].copy_from_slice(&(name.len() as u16).to_le_bytes());
        central.extend_from_slice(name);
        let mut eocd = vec![0; 22];
        eocd[..4].copy_from_slice(b"PK\x05\x06");
        eocd[8..10].copy_from_slice(&1u16.to_le_bytes());
        eocd[10..12].copy_from_slice(&1u16.to_le_bytes());
        eocd[12..16].copy_from_slice(&(central.len() as u32).to_le_bytes());
        eocd[16..20].copy_from_slice(&0u32.to_le_bytes());
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(&central).unwrap(); file.write_all(&eocd).unwrap();
        let zip = inspect_zip(&path).unwrap();
        assert_eq!(zip.entries, vec![ZipEntry { path: "top/file.bin".into(), size: 1, encrypted: false }]);
        std::fs::remove_file(path).unwrap();
    }
}
