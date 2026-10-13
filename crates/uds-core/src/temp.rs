//! %TEMP% cleanup. v1.x deleted everything without asking; v2 shows the size
//! first, cleans only after the user confirms, and skips files in use.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TempSummary {
    pub items: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub removed: u64,
    pub skipped: u64,
    pub freed_bytes: u64,
}

fn size_of(path: &Path) -> u64 {
    let Ok(meta) = fs::symlink_metadata(path) else { return 0 };
    if meta.is_dir() {
        fs::read_dir(path).map(|r| r.filter_map(|e| e.ok()).map(|e| size_of(&e.path())).sum()).unwrap_or(0)
    } else {
        meta.len()
    }
}

pub fn summarize(dir: &Path) -> TempSummary {
    let Ok(read) = fs::read_dir(dir) else { return TempSummary::default() };
    read.filter_map(|e| e.ok()).fold(TempSummary::default(), |acc, e| TempSummary {
        items: acc.items + 1,
        bytes: acc.bytes + size_of(&e.path()),
    })
}

/// Removes top-level items in `dir` older than `min_age` (recently created
/// files usually belong to installers or apps that are running right now).
pub fn clean(dir: &Path, min_age: Duration) -> CleanResult {
    let mut result = CleanResult::default();
    let Ok(read) = fs::read_dir(dir) else { return result };
    let now = SystemTime::now();
    for entry in read.filter_map(|e| e.ok()) {
        let path = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else { continue };
        let age = meta.modified().ok().and_then(|m| now.duration_since(m).ok()).unwrap_or_default();
        if age < min_age {
            result.skipped += 1;
            continue;
        }
        let bytes = size_of(&path);
        let removed = if meta.is_dir() { fs::remove_dir_all(&path) } else { fs::remove_file(&path) };
        match removed {
            Ok(()) => {
                result.removed += 1;
                result.freed_bytes += bytes;
            }
            Err(_) => result.skipped += 1,
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_and_cleans() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.tmp"), [0u8; 100]).unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("b.tmp"), [0u8; 50]).unwrap();
        assert_eq!(summarize(dir.path()), TempSummary { items: 2, bytes: 150 });

        let fresh = clean(dir.path(), Duration::from_secs(3600));
        assert_eq!(fresh, CleanResult { removed: 0, skipped: 2, freed_bytes: 0 }, "new files are kept");

        let all = clean(dir.path(), Duration::ZERO);
        assert_eq!(all, CleanResult { removed: 2, skipped: 0, freed_bytes: 150 });
        assert_eq!(summarize(dir.path()).items, 0);
    }
}
