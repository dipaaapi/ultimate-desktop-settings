//! Startup apps, kept compatible with Task Manager and Settings > Apps > Startup.
//!
//! Windows records whether a startup entry is enabled in the
//! `Explorer\StartupApproved` keys, so entries are disabled the same way
//! Windows does it and never deleted when the user only turns them off.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::registry::{Hive, RegValue, Registry};
use crate::{Error, Result};

pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const APPROVED_RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
pub const APPROVED_FOLDER_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder";
/// Where v1.x moved "disabled" entries. Migrated on first load.
pub const LEGACY_DISABLED_KEY: &str = r"Software\UltimateDesktopSettings\DisabledRun";
/// Value name this app uses for its own "launch at boot" entry.
pub const SELF_RUN_VALUE: &str = "UltimateWindowsDesktopSettings";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupSource {
    /// `HKCU\...\Run`: this user, editable.
    UserRun,
    /// `HKLM\...\Run`: all users, needs admin rights so shown read-only.
    MachineRun,
    /// Shortcuts in the user's Startup folder.
    UserFolder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub command: String,
    pub source: StartupSource,
    pub enabled: bool,
    pub editable: bool,
}

fn make_id(source: StartupSource, name: &str) -> String {
    let prefix = match source {
        StartupSource::UserRun => "user-run",
        StartupSource::MachineRun => "machine-run",
        StartupSource::UserFolder => "user-folder",
    };
    format!("{prefix}:{name}")
}

fn parse_id(id: &str) -> Result<(StartupSource, &str)> {
    let (prefix, name) = id.split_once(':').ok_or_else(|| Error::Invalid(format!("startup id {id}")))?;
    let source = match prefix {
        "user-run" => StartupSource::UserRun,
        "machine-run" => StartupSource::MachineRun,
        "user-folder" => StartupSource::UserFolder,
        _ => return Err(Error::Invalid(format!("startup id {id}"))),
    };
    Ok((source, name))
}

/// Task Manager writes 12 bytes; an even first byte means enabled.
pub fn is_approved(value: Option<&RegValue>) -> bool {
    match value {
        Some(RegValue::Binary(bytes)) => bytes.first().is_none_or(|b| b & 1 == 0),
        _ => true,
    }
}

pub fn approval_bytes(enabled: bool, now_filetime: u64) -> Vec<u8> {
    let mut bytes = vec![0u8; 12];
    if enabled {
        bytes[0] = 0x02;
    } else {
        bytes[0] = 0x03;
        bytes[4..].copy_from_slice(&now_filetime.to_le_bytes());
    }
    bytes
}

pub fn filetime_now() -> u64 {
    const UNIX_TO_FILETIME_SECS: u64 = 11_644_473_600;
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    (since_epoch.as_secs() + UNIX_TO_FILETIME_SECS) * 10_000_000 + u64::from(since_epoch.subsec_nanos() / 100)
}

pub fn list(reg: &dyn Registry, startup_dir: Option<&Path>) -> Vec<StartupItem> {
    let mut items = Vec::new();
    for (hive, source) in [(Hive::CurrentUser, StartupSource::UserRun), (Hive::LocalMachine, StartupSource::MachineRun)]
    {
        for name in reg.value_names(hive, RUN_KEY) {
            let command = reg.get_string(hive, RUN_KEY, &name).unwrap_or_default();
            let enabled = is_approved(reg.get(hive, APPROVED_RUN_KEY, &name).as_ref());
            items.push(StartupItem {
                id: make_id(source, &name),
                name,
                command,
                source,
                enabled,
                editable: source == StartupSource::UserRun,
            });
        }
    }
    if let Some(dir) = startup_dir {
        for file in folder_entries(dir) {
            let enabled = is_approved(reg.get(Hive::CurrentUser, APPROVED_FOLDER_KEY, &file).as_ref());
            let display =
                Path::new(&file).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or(file.clone());
            items.push(StartupItem {
                id: make_id(StartupSource::UserFolder, &file),
                name: display,
                command: dir.join(&file).to_string_lossy().into_owned(),
                source: StartupSource::UserFolder,
                enabled,
                editable: true,
            });
        }
    }
    items.sort_by_key(|i| i.name.to_lowercase());
    items
}

fn folder_entries(dir: &Path) -> Vec<String> {
    let Ok(read) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut names: Vec<String> = read
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.eq_ignore_ascii_case("desktop.ini"))
        .collect();
    names.sort();
    names
}

pub fn set_enabled(reg: &dyn Registry, id: &str, enabled: bool) -> Result<()> {
    let (source, name) = parse_id(id)?;
    let bytes = RegValue::Binary(approval_bytes(enabled, filetime_now()));
    match source {
        StartupSource::UserRun => {
            if reg.get(Hive::CurrentUser, RUN_KEY, name).is_none() {
                return Err(Error::NotFound(name.to_owned()));
            }
            reg.set(Hive::CurrentUser, APPROVED_RUN_KEY, name, bytes)
        }
        StartupSource::UserFolder => reg.set(Hive::CurrentUser, APPROVED_FOLDER_KEY, name, bytes),
        StartupSource::MachineRun => Err(Error::msg(
            "This entry is installed for all users. Change it in Task Manager > Startup apps (needs admin rights).",
        )),
    }
}

pub fn add(reg: &dyn Registry, name: &str, command: &str) -> Result<StartupItem> {
    let name = name.trim();
    let command = command.trim();
    if name.is_empty() || command.is_empty() {
        return Err(Error::Invalid("Name and program path cannot be empty.".into()));
    }
    if name.contains('\\') {
        return Err(Error::Invalid("Name cannot contain a backslash.".into()));
    }
    if reg.get(Hive::CurrentUser, RUN_KEY, name).is_some() {
        return Err(Error::Invalid(format!("A startup entry named \"{name}\" already exists.")));
    }
    reg.set(Hive::CurrentUser, RUN_KEY, name, RegValue::String(quote_command(command)))?;
    let _ = reg.delete_value(Hive::CurrentUser, APPROVED_RUN_KEY, name);
    Ok(StartupItem {
        id: make_id(StartupSource::UserRun, name),
        name: name.to_owned(),
        command: quote_command(command),
        source: StartupSource::UserRun,
        enabled: true,
        editable: true,
    })
}

/// Wraps a bare path that contains spaces in quotes so Windows runs the right file.
pub fn quote_command(command: &str) -> String {
    let looks_like_bare_path =
        command.contains(' ') && !command.starts_with('"') && command.to_ascii_lowercase().ends_with(".exe");
    if looks_like_bare_path { format!("\"{command}\"") } else { command.to_owned() }
}

pub fn remove(reg: &dyn Registry, id: &str, startup_dir: Option<&Path>) -> Result<()> {
    let (source, name) = parse_id(id)?;
    match source {
        StartupSource::UserRun => {
            reg.delete_value(Hive::CurrentUser, RUN_KEY, name)?;
            let _ = reg.delete_value(Hive::CurrentUser, APPROVED_RUN_KEY, name);
            Ok(())
        }
        StartupSource::UserFolder => {
            let dir = startup_dir.ok_or_else(|| Error::NotFound("Startup folder".into()))?;
            let path: PathBuf = dir.join(name);
            if path.parent() != Some(dir) {
                return Err(Error::Invalid(name.to_owned()));
            }
            std::fs::remove_file(&path)?;
            let _ = reg.delete_value(Hive::CurrentUser, APPROVED_FOLDER_KEY, name);
            Ok(())
        }
        StartupSource::MachineRun => {
            Err(Error::msg("This entry is installed for all users and needs admin rights to remove."))
        }
    }
}

/// Moves entries that v1.x "disabled" into the Run key, marked disabled the
/// way Windows does it. Returns how many entries were migrated.
pub fn migrate_legacy(reg: &dyn Registry) -> usize {
    let mut moved = 0;
    for name in reg.value_names(Hive::CurrentUser, LEGACY_DISABLED_KEY) {
        let Some(command) = reg.get_string(Hive::CurrentUser, LEGACY_DISABLED_KEY, &name) else { continue };
        if reg.get(Hive::CurrentUser, RUN_KEY, &name).is_none()
            && reg.set(Hive::CurrentUser, RUN_KEY, &name, RegValue::String(command)).is_err()
        {
            continue;
        }
        let disabled = RegValue::Binary(approval_bytes(false, filetime_now()));
        if reg.set(Hive::CurrentUser, APPROVED_RUN_KEY, &name, disabled).is_ok()
            && reg.delete_value(Hive::CurrentUser, LEGACY_DISABLED_KEY, &name).is_ok()
        {
            moved += 1;
        }
    }
    moved
}

/// Launch-at-boot for this app. The path is quoted and absolute so the tray
/// icon and assets load no matter what folder Windows starts it in.
pub fn self_launch_enabled(reg: &dyn Registry) -> bool {
    reg.get(Hive::CurrentUser, RUN_KEY, SELF_RUN_VALUE).is_some()
        && is_approved(reg.get(Hive::CurrentUser, APPROVED_RUN_KEY, SELF_RUN_VALUE).as_ref())
}

pub fn set_self_launch(reg: &dyn Registry, exe_path: &Path, enabled: bool) -> Result<()> {
    if enabled {
        let command = format!("\"{}\" --minimized", exe_path.display());
        reg.set(Hive::CurrentUser, RUN_KEY, SELF_RUN_VALUE, RegValue::String(command))?;
        let _ = reg.delete_value(Hive::CurrentUser, APPROVED_RUN_KEY, SELF_RUN_VALUE);
    } else {
        let _ = reg.delete_value(Hive::CurrentUser, RUN_KEY, SELF_RUN_VALUE);
        let _ = reg.delete_value(Hive::CurrentUser, APPROVED_RUN_KEY, SELF_RUN_VALUE);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::MemoryRegistry;

    fn reg_with_entries() -> MemoryRegistry {
        let reg = MemoryRegistry::new();
        reg.set(Hive::CurrentUser, RUN_KEY, "Discord", RegValue::String(r"C:\Discord\Update.exe".into())).unwrap();
        reg.set(Hive::CurrentUser, RUN_KEY, "OneDrive", RegValue::String(r#""C:\OneDrive.exe" /background"#.into()))
            .unwrap();
        reg.set(Hive::LocalMachine, RUN_KEY, "SecurityHealth", RegValue::String("SecurityHealthSystray.exe".into()))
            .unwrap();
        reg
    }

    #[test]
    fn lists_user_and_machine_entries() {
        let reg = reg_with_entries();
        let items = list(&reg, None);
        assert_eq!(items.len(), 3);
        let machine = items.iter().find(|i| i.name == "SecurityHealth").unwrap();
        assert_eq!(machine.source, StartupSource::MachineRun);
        assert!(!machine.editable);
        assert!(items.iter().all(|i| i.enabled));
    }

    #[test]
    fn disabling_uses_startup_approved_and_keeps_entry() {
        let reg = reg_with_entries();
        set_enabled(&reg, "user-run:Discord", false).unwrap();
        let bytes = reg.get(Hive::CurrentUser, APPROVED_RUN_KEY, "Discord").unwrap();
        let RegValue::Binary(b) = &bytes else { panic!("expected binary") };
        assert_eq!(b.len(), 12);
        assert_eq!(b[0], 0x03);
        assert!(reg.get(Hive::CurrentUser, RUN_KEY, "Discord").is_some(), "entry must not be deleted");
        let item = list(&reg, None).into_iter().find(|i| i.name == "Discord").unwrap();
        assert!(!item.enabled);

        set_enabled(&reg, "user-run:Discord", true).unwrap();
        assert!(list(&reg, None).into_iter().find(|i| i.name == "Discord").unwrap().enabled);
    }

    #[test]
    fn approval_byte_parity() {
        assert!(is_approved(None));
        assert!(is_approved(Some(&RegValue::Binary(vec![0x02; 12]))));
        assert!(is_approved(Some(&RegValue::Binary(vec![0x06, 0, 0]))));
        assert!(!is_approved(Some(&RegValue::Binary(vec![0x03, 0, 0]))));
        assert!(!is_approved(Some(&RegValue::Binary(vec![0x07]))));
    }

    #[test]
    fn machine_entries_are_read_only() {
        let reg = reg_with_entries();
        assert!(set_enabled(&reg, "machine-run:SecurityHealth", false).is_err());
        assert!(remove(&reg, "machine-run:SecurityHealth", None).is_err());
    }

    #[test]
    fn add_validates_and_quotes() {
        let reg = MemoryRegistry::new();
        assert!(add(&reg, "", "x.exe").is_err());
        assert!(add(&reg, "a\\b", "x.exe").is_err());
        let item = add(&reg, "My Tool", r"C:\Program Files\Tool\tool.exe").unwrap();
        assert_eq!(item.command, r#""C:\Program Files\Tool\tool.exe""#);
        assert!(add(&reg, "My Tool", "other.exe").is_err(), "duplicates rejected");
    }

    #[test]
    fn remove_deletes_value_and_approval() {
        let reg = reg_with_entries();
        set_enabled(&reg, "user-run:Discord", false).unwrap();
        remove(&reg, "user-run:Discord", None).unwrap();
        assert!(reg.get(Hive::CurrentUser, RUN_KEY, "Discord").is_none());
        assert!(reg.get(Hive::CurrentUser, APPROVED_RUN_KEY, "Discord").is_none());
    }

    #[test]
    fn migrates_v1_disabled_entries() {
        let reg = MemoryRegistry::new();
        reg.set(Hive::CurrentUser, LEGACY_DISABLED_KEY, "Spotify", RegValue::String("spotify.exe".into())).unwrap();
        assert_eq!(migrate_legacy(&reg), 1);
        let item = list(&reg, None).into_iter().find(|i| i.name == "Spotify").unwrap();
        assert!(!item.enabled);
        assert!(reg.value_names(Hive::CurrentUser, LEGACY_DISABLED_KEY).is_empty());
        assert_eq!(migrate_legacy(&reg), 0, "idempotent");
    }

    #[test]
    fn startup_folder_entries() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Notes.lnk"), b"").unwrap();
        std::fs::write(dir.path().join("desktop.ini"), b"").unwrap();
        let reg = MemoryRegistry::new();
        let items = list(&reg, Some(dir.path()));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Notes");
        set_enabled(&reg, &items[0].id, false).unwrap();
        assert!(!list(&reg, Some(dir.path()))[0].enabled);
        remove(&reg, &items[0].id, Some(dir.path())).unwrap();
        assert!(list(&reg, Some(dir.path())).is_empty());
    }

    #[test]
    fn self_launch_round_trip() {
        let reg = MemoryRegistry::new();
        assert!(!self_launch_enabled(&reg));
        set_self_launch(&reg, Path::new(r"C:\Apps\uds.exe"), true).unwrap();
        assert!(self_launch_enabled(&reg));
        assert_eq!(
            reg.get_string(Hive::CurrentUser, RUN_KEY, SELF_RUN_VALUE).unwrap(),
            r#""C:\Apps\uds.exe" --minimized"#
        );
        set_self_launch(&reg, Path::new(r"C:\Apps\uds.exe"), false).unwrap();
        assert!(!self_launch_enabled(&reg));
    }
}
