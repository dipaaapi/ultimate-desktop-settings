//! Scrapbook notes: plain `.txt` files plus a `metadata.json` that lists the
//! priority (pinned) notes. Same layout as v1.x, so old notes can be copied in.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

pub const DEFAULT_NOTE: &str = "General Notes";
const WELCOME: &str = "Welcome to UDS Scrapbook Notes!\nWrite your notes here. They are saved on this PC only.";
const METADATA: &str = "metadata.json";
const MAX_TITLE: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteMeta {
    pub title: String,
    pub pinned: bool,
    pub modified_ms: u64,
    pub preview: String,
}

#[derive(Default, Serialize, Deserialize)]
struct Metadata {
    #[serde(default)]
    priority: BTreeSet<String>,
}

pub struct NotesStore {
    dir: PathBuf,
}

pub fn validate_title(title: &str) -> Result<String> {
    let title = title.trim();
    if title.is_empty() {
        return Err(Error::Invalid("Note title cannot be empty.".into()));
    }
    if title.chars().count() > MAX_TITLE {
        return Err(Error::Invalid(format!("Note title can be at most {MAX_TITLE} characters.")));
    }
    if title.chars().any(|c| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control()) {
        return Err(Error::Invalid(r#"Note title cannot contain \ / : * ? " < > |"#.into()));
    }
    if title.ends_with('.') || title.starts_with('.') {
        return Err(Error::Invalid("Note title cannot start or end with a dot.".into()));
    }
    let stem = title.split('.').next().unwrap_or(title).to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if reserved {
        return Err(Error::Invalid(format!("\"{title}\" is a reserved name on Windows.")));
    }
    Ok(title.to_owned())
}

impl NotesStore {
    /// Opens the store, creating the folder and a welcome note if needed.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        let store = Self { dir: dir.into() };
        fs::create_dir_all(&store.dir)?;
        if store.titles()?.is_empty() {
            fs::write(store.path_for(DEFAULT_NOTE), WELCOME)?;
        }
        Ok(store)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path_for(&self, title: &str) -> PathBuf {
        self.dir.join(format!("{title}.txt"))
    }

    fn existing_path(&self, title: &str) -> Result<PathBuf> {
        let title = validate_title(title)?;
        let path = self.path_for(&title);
        if path.is_file() { Ok(path) } else { Err(Error::NotFound(format!("note \"{title}\""))) }
    }

    fn titles(&self) -> Result<Vec<String>> {
        let mut titles = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path.is_file() && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("txt")) {
                if let Some(stem) = path.file_stem() {
                    titles.push(stem.to_string_lossy().into_owned());
                }
            }
        }
        Ok(titles)
    }

    fn metadata(&self) -> Metadata {
        fs::read_to_string(self.dir.join(METADATA)).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    fn save_metadata(&self, meta: &Metadata) -> Result<()> {
        write_atomic(&self.dir.join(METADATA), serde_json::to_string_pretty(meta)?.as_bytes())
    }

    pub fn list(&self) -> Result<Vec<NoteMeta>> {
        let meta = self.metadata();
        let mut notes: Vec<NoteMeta> = self
            .titles()?
            .into_iter()
            .map(|title| {
                let path = self.path_for(&title);
                let modified_ms = fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or_default();
                let preview: String = fs::read_to_string(&path)
                    .unwrap_or_default()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(90)
                    .collect();
                NoteMeta { pinned: meta.priority.contains(&title), title, modified_ms, preview }
            })
            .collect();
        notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(a.title.to_lowercase().cmp(&b.title.to_lowercase())));
        Ok(notes)
    }

    pub fn read(&self, title: &str) -> Result<String> {
        Ok(fs::read_to_string(self.existing_path(title)?)?)
    }

    pub fn save(&self, title: &str, content: &str) -> Result<()> {
        write_atomic(&self.existing_path(title)?, content.as_bytes())
    }

    pub fn create(&self, title: &str) -> Result<String> {
        let title = validate_title(title)?;
        if self.titles()?.iter().any(|t| t.eq_ignore_ascii_case(&title)) {
            return Err(Error::Invalid(format!("A note named \"{title}\" already exists.")));
        }
        fs::write(self.path_for(&title), "")?;
        Ok(title)
    }

    pub fn rename(&self, old: &str, new: &str) -> Result<String> {
        let from = self.existing_path(old)?;
        let new = validate_title(new)?;
        let same_name = old.trim().eq_ignore_ascii_case(&new);
        if !same_name && self.titles()?.iter().any(|t| t.eq_ignore_ascii_case(&new)) {
            return Err(Error::Invalid(format!("A note named \"{new}\" already exists.")));
        }
        fs::rename(from, self.path_for(&new))?;
        let mut meta = self.metadata();
        if meta.priority.remove(old.trim()) {
            meta.priority.insert(new.clone());
            self.save_metadata(&meta)?;
        }
        Ok(new)
    }

    pub fn delete(&self, title: &str) -> Result<()> {
        let path = self.existing_path(title)?;
        let meta = self.metadata();
        if meta.priority.contains(title.trim()) {
            return Err(Error::Invalid("Unpin this note before deleting it.".into()));
        }
        if self.titles()?.len() <= 1 {
            return Err(Error::Invalid("You must keep at least one note.".into()));
        }
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn set_pinned(&self, title: &str, pinned: bool) -> Result<()> {
        self.existing_path(title)?;
        let mut meta = self.metadata();
        if pinned {
            meta.priority.insert(title.trim().to_owned());
        } else {
            meta.priority.remove(title.trim());
        }
        self.save_metadata(&meta)
    }

    /// Copies notes from the v1.x folder, never overwriting existing notes.
    /// Returns how many notes were imported.
    pub fn import_from(&self, legacy_dir: &Path) -> Result<usize> {
        if !legacy_dir.is_dir() || legacy_dir == self.dir {
            return Ok(0);
        }
        let mut imported = 0;
        let mut meta = self.metadata();
        let legacy_meta: Metadata = fs::read_to_string(legacy_dir.join(METADATA))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        for entry in fs::read_dir(legacy_dir)? {
            let path = entry?.path();
            let is_txt = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("txt"));
            let Some(stem) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else { continue };
            if !is_txt || validate_title(&stem).is_err() {
                continue;
            }
            let target = self.path_for(&stem);
            if target.exists() {
                // Keep the welcome note replaceable by the user's real one.
                let is_untouched_welcome =
                    stem == DEFAULT_NOTE && fs::read_to_string(&target).ok().as_deref() == Some(WELCOME);
                if !is_untouched_welcome {
                    continue;
                }
            }
            fs::copy(&path, &target)?;
            if legacy_meta.priority.contains(&stem) {
                meta.priority.insert(stem);
            }
            imported += 1;
        }
        self.save_metadata(&meta)?;
        Ok(imported)
    }
}

/// Writes through a temp file so a crash never leaves a half-written file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp-write");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_welcome_note() {
        let dir = tempfile::tempdir().unwrap();
        let store = NotesStore::open(dir.path().join("notes")).unwrap();
        let notes = store.list().unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, DEFAULT_NOTE);
        assert!(notes[0].preview.starts_with("Welcome"));
    }

    #[test]
    fn crud_and_pinning() {
        let dir = tempfile::tempdir().unwrap();
        let store = NotesStore::open(dir.path()).unwrap();
        store.create("Shopping").unwrap();
        assert!(store.create("shopping").is_err(), "case-insensitive duplicate");
        store.save("Shopping", "eggs\nmilk").unwrap();
        assert_eq!(store.read("Shopping").unwrap(), "eggs\nmilk");

        store.set_pinned("Shopping", true).unwrap();
        assert_eq!(store.list().unwrap()[0].title, "Shopping");
        assert!(store.delete("Shopping").is_err(), "pinned notes are protected");

        let renamed = store.rename("Shopping", "Groceries").unwrap();
        assert_eq!(renamed, "Groceries");
        assert!(store.list().unwrap()[0].pinned, "pin follows rename");
        store.set_pinned("Groceries", false).unwrap();
        store.delete("Groceries").unwrap();
        assert!(store.delete(DEFAULT_NOTE).is_err(), "last note is kept");
    }

    #[test]
    fn rejects_unsafe_titles() {
        for bad in ["", "  ", "a/b", r"..\escape", "CON", "com1", "x?", "dots.", ".hidden"] {
            assert!(validate_title(bad).is_err(), "{bad:?} should be rejected");
        }
        assert_eq!(validate_title("  Ideas 2026  ").unwrap(), "Ideas 2026");
        assert!(validate_title("Console notes").is_ok());
    }

    #[test]
    fn imports_v1_notes_without_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("legacy");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("General Notes.txt"), "my real notes").unwrap();
        fs::write(legacy.join("simple note 1.txt"), "hello").unwrap();
        fs::write(legacy.join("metadata.json"), r#"{"priority":["simple note 1"]}"#).unwrap();

        let store = NotesStore::open(dir.path().join("notes")).unwrap();
        assert_eq!(store.import_from(&legacy).unwrap(), 2);
        assert_eq!(store.read(DEFAULT_NOTE).unwrap(), "my real notes");
        assert!(store.list().unwrap().iter().any(|n| n.title == "simple note 1" && n.pinned));

        store.save("simple note 1", "edited").unwrap();
        assert_eq!(store.import_from(&legacy).unwrap(), 0);
        assert_eq!(store.read("simple note 1").unwrap(), "edited");
    }
}
