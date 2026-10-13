//! A small registry abstraction so the logic that reads and writes Windows
//! settings can be tested without Windows.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum Hive {
    CurrentUser,
    LocalMachine,
    ClassesRoot,
}

impl Hive {
    pub fn short_name(self) -> &'static str {
        match self {
            Hive::CurrentUser => "HKCU",
            Hive::LocalMachine => "HKLM",
            Hive::ClassesRoot => "HKCR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegValue {
    String(String),
    ExpandString(String),
    Dword(u32),
    Binary(Vec<u8>),
}

impl RegValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            RegValue::String(s) | RegValue::ExpandString(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_dword(&self) -> Option<u32> {
        match self {
            RegValue::Dword(v) => Some(*v),
            _ => None,
        }
    }
}

/// Registry operations used by the app. Paths use backslashes, relative to
/// the hive. An empty value name means the key's default value.
pub trait Registry: Send + Sync {
    fn get(&self, hive: Hive, path: &str, name: &str) -> Option<RegValue>;
    fn set(&self, hive: Hive, path: &str, name: &str, value: RegValue) -> Result<()>;
    fn delete_value(&self, hive: Hive, path: &str, name: &str) -> Result<()>;
    /// Value names (excluding the default value) of a key; empty if missing.
    fn value_names(&self, hive: Hive, path: &str) -> Vec<String>;
    /// Direct subkey names of a key; empty if missing.
    fn subkeys(&self, hive: Hive, path: &str) -> Vec<String>;

    fn get_string(&self, hive: Hive, path: &str, name: &str) -> Option<String> {
        self.get(hive, path, name).and_then(|v| v.as_str().map(str::to_owned))
    }

    fn get_dword(&self, hive: Hive, path: &str, name: &str) -> Option<u32> {
        self.get(hive, path, name).and_then(|v| v.as_dword())
    }
}

/// Lowercased value name -> (original name, value).
type Values = BTreeMap<String, (String, RegValue)>;

/// In-memory registry for tests and the non-Windows preview.
#[derive(Default)]
pub struct MemoryRegistry {
    // (hive, lowercased key path) -> values
    keys: Mutex<BTreeMap<(Hive, String), Values>>,
    // (hive, lowercased key path) -> key path with its original casing
    casing: Mutex<BTreeMap<(Hive, String), String>>,
}

fn norm(path: &str) -> String {
    path.trim_matches('\\').to_ascii_lowercase()
}

impl MemoryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a key (and its parents) with no values.
    pub fn create_key(&self, hive: Hive, path: &str) {
        let mut keys = self.keys.lock().unwrap();
        let mut casing = self.casing.lock().unwrap();
        let mut acc = String::new();
        for part in path.trim_matches('\\').split('\\') {
            if !acc.is_empty() {
                acc.push('\\');
            }
            acc.push_str(part);
            keys.entry((hive, norm(&acc))).or_default();
            casing.entry((hive, norm(&acc))).or_insert_with(|| acc.clone());
        }
    }
}

impl Registry for MemoryRegistry {
    fn get(&self, hive: Hive, path: &str, name: &str) -> Option<RegValue> {
        let keys = self.keys.lock().unwrap();
        keys.get(&(hive, norm(path))).and_then(|vals| vals.get(&name.to_ascii_lowercase())).map(|(_, v)| v.clone())
    }

    fn set(&self, hive: Hive, path: &str, name: &str, value: RegValue) -> Result<()> {
        self.create_key(hive, path);
        let mut keys = self.keys.lock().unwrap();
        keys.entry((hive, norm(path))).or_default().insert(name.to_ascii_lowercase(), (name.to_owned(), value));
        Ok(())
    }

    fn delete_value(&self, hive: Hive, path: &str, name: &str) -> Result<()> {
        let mut keys = self.keys.lock().unwrap();
        let removed = keys.get_mut(&(hive, norm(path))).and_then(|vals| vals.remove(&name.to_ascii_lowercase()));
        match removed {
            Some(_) => Ok(()),
            None => Err(Error::NotFound(format!("{}\\{path}\\{name}", hive.short_name()))),
        }
    }

    fn value_names(&self, hive: Hive, path: &str) -> Vec<String> {
        let keys = self.keys.lock().unwrap();
        keys.get(&(hive, norm(path)))
            .map(|vals| vals.values().filter(|(n, _)| !n.is_empty()).map(|(n, _)| n.clone()).collect())
            .unwrap_or_default()
    }

    fn subkeys(&self, hive: Hive, path: &str) -> Vec<String> {
        let prefix = format!("{}\\", norm(path));
        let keys = self.keys.lock().unwrap();
        let casing = self.casing.lock().unwrap();
        keys.keys()
            .filter(|(h, p)| *h == hive && p.starts_with(&prefix) && !p[prefix.len()..].contains('\\'))
            .map(|(h, p)| {
                let original = casing.get(&(*h, p.clone())).cloned().unwrap_or_else(|| p.clone());
                original.rsplit('\\').next().unwrap_or_default().to_owned()
            })
            .collect()
    }
}
