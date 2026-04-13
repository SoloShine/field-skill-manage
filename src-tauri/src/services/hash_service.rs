use md5::Md5;
use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

use crate::models::skill::{Checksum, FileEntry};

/// Calculate SHA256 hash of file contents
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let data =
        std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

/// Calculate MD5 hash of file contents
pub fn md5_file(path: &Path) -> Result<String, String> {
    let data =
        std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let mut hasher = Md5::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

/// Calculate aggregate SHA256 hash for a directory
/// Walks all files, computes per-file SHA256, then hashes the sorted concatenation
pub fn aggregate_sha256(dir: &Path) -> Result<Checksum, String> {
    if !dir.is_dir() {
        return Err(format!("{} is not a directory", dir.display()));
    }

    let mut entries: Vec<(String, String)> = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let relative = entry
            .path()
            .strip_prefix(dir)
            .map_err(|e| format!("Path error: {}", e))?
            .to_string_lossy()
            .replace('\\', "/");
        let hash = sha256_file(entry.path())?;
        entries.push((relative, hash));
    }

    // Sort by relative path for deterministic ordering
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Concatenate "path:hash\n" and hash the result
    let combined: String = entries
        .iter()
        .map(|(p, h)| format!("{}:{}", p, h))
        .collect::<Vec<_>>()
        .join("\n");

    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let value = hex::encode(hasher.finalize());

    Ok(Checksum {
        algorithm: "sha256".to_string(),
        value,
    })
}

/// List all files in a directory with their hashes
pub fn list_file_hashes(dir: &Path) -> Result<Vec<FileEntry>, String> {
    if !dir.is_dir() {
        return Err(format!("{} is not a directory", dir.display()));
    }

    let mut entries = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let relative = entry
            .path()
            .strip_prefix(dir)
            .map_err(|e| format!("Path error: {}", e))?
            .to_string_lossy()
            .replace('\\', "/");

        let hash = sha256_file(entry.path())?;
        let size = entry
            .metadata()
            .map(|m| m.len())
            .unwrap_or(0);

        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| {
                let dt: chrono::DateTime<chrono::Local> = t.into();
                Some(dt.to_rfc3339())
            })
            .unwrap_or_default();

        entries.push(FileEntry {
            path: relative,
            hash: format!("sha256:{}", hash),
            size,
            mtime,
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}
