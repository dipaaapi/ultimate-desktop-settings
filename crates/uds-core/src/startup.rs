//! Startup apps, kept compatible with Task Manager and Settings > Apps > Startup.
//!
//! Windows records whether a startup entry is enabled in the
//! `Explorer\StartupApproved` keys, so entries are disabled the same way
//! Windows does it and never deleted when the user only turns them off.
//!
//! Every place Windows starts programs from at sign-in is listed, including
//! the ones Task Manager hides or files elsewhere:
//!
//! - the `Run` keys of this user and of all users, including the 32-bit
//!   (`WOW6432Node`) key most older installers write to,
//! - the `RunOnce` keys (programs that run once at the next sign-in),
//! - `Run` keys set by group policy,
//! - this user's and the all-users Startup folders,
//! - Microsoft Store apps that registered a startup task,
//! - scheduled tasks that start at sign-in or boot (read on Windows by
//!   [`crate::system::scheduled_startup_tasks`]).

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::registry::{Hive, RegValue, Registry};
use crate::{Error, Result};

pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const RUN_ONCE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\RunOnce";
pub const RUN32_KEY: &str = r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run";
pub const POLICY_RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run";
pub const APPROVED_RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
pub const APPROVED_RUN32_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32";
pub const APPROVED_FOLDER_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder";
/// Startup tasks of Microsoft Store apps, one subkey per package and task.
pub const STORE_TASKS_KEY: &str =
    r"Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel\SystemAppData";
/// Where v1.x moved "disabled" entries. Migrated on first load.
pub const LEGACY_DISABLED_KEY: &str = r"Software\UltimateDesktopSettings\DisabledRun";
/// Value name this app uses for its own "launch at boot" entry.
pub const SELF_RUN_VALUE: &str = "UltimateWindowsDesktopSettings";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupSource {
    /// `HKCU\...\Run`: this user.
    UserRun,
    /// `HKCU\...\RunOnce`: runs once at this user's next sign-in.
    UserRunOnce,
    /// `HKLM\...\Run`: all users (changes need admin rights).
    MachineRun,
    /// `HKLM\Software\WOW6432Node\...\Run`: 32-bit programs for all users.
    MachineRun32,
    /// `HKLM\...\RunOnce`: runs once at the next sign-in of any user.
    MachineRunOnce,
    /// A `Run` key set by group policy for this user. Read-only.
    UserPolicy,
    /// A `Run` key set by group policy for all users. Read-only.
    MachinePolicy,
    /// Shortcuts in this user's Startup folder.
    UserFolder,
    /// Shortcuts in the all-users Startup folder.
    CommonFolder,
    /// A Microsoft Store app's startup task.
    StoreApp,
    /// A scheduled task that starts at sign-in or boot.
    ScheduledTask,
}

impl StartupSource {
    const ALL: [StartupSource; 11] = [
        StartupSource::UserRun,
        StartupSource::UserRunOnce,
        StartupSource::MachineRun,
        StartupSource::MachineRun32,
        StartupSource::MachineRunOnce,
        StartupSource::UserPolicy,
        StartupSource::MachinePolicy,
        StartupSource::UserFolder,
        StartupSource::CommonFolder,
        StartupSource::StoreApp,
        StartupSource::ScheduledTask,
    ];

    /// Id prefix. The first three match v2.0 so saved ids keep working.
    fn tag(self) -> &'static str {
        match self {
            StartupSource::UserRun => "user-run",
            StartupSource::UserRunOnce => "user-run-once",
            StartupSource::MachineRun => "machine-run",
            StartupSource::MachineRun32 => "machine-run32",
            StartupSource::MachineRunOnce => "machine-run-once",
            StartupSource::UserPolicy => "user-policy",
            StartupSource::MachinePolicy => "machine-policy",
            StartupSource::UserFolder => "user-folder",
            StartupSource::CommonFolder => "common-folder",
            StartupSource::StoreApp => "store-app",
            StartupSource::ScheduledTask => "task",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub command: String,
    pub source: StartupSource,
    pub enabled: bool,
    /// It can be turned on and off (possibly after a UAC prompt).
    pub editable: bool,
    /// It can be removed (possibly after a UAC prompt).
    pub removable: bool,
    /// Changing it needs admin rights, so Windows will show a UAC prompt.
    pub needs_admin: bool,
    /// Where Windows keeps it: a registry key, folder or task path.
    pub location: String,
}

/// The two Startup folders. Either may be missing.
#[derive(Debug, Clone, Default)]
pub struct StartupFolders {
    pub user: Option<PathBuf>,
    pub common: Option<PathBuf>,
}

impl StartupFolders {
    fn get(&self, source: StartupSource) -> Option<&Path> {
        match source {
            StartupSource::UserFolder => self.user.as_deref(),
            StartupSource::CommonFolder => self.common.as_deref(),
            _ => None,
        }
    }
}

/// A scheduled task that starts a program at sign-in or boot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledStartup {
    /// Full task path, such as `\GoogleUpdateTaskMachineCore`.
    pub path: String,
    pub command: String,
    pub enabled: bool,
}

/// A registry key Windows starts programs from.
struct RunKey {
    source: StartupSource,
    hive: Hive,
    path: &'static str,
    /// Where Task Manager records "disabled", if this key supports it.
    approved: Option<(Hive, &'static str)>,
    removable: bool,
}

const RUN_KEYS: [RunKey; 7] = [
    RunKey {
        source: StartupSource::UserRun,
        hive: Hive::CurrentUser,
        path: RUN_KEY,
        approved: Some((Hive::CurrentUser, APPROVED_RUN_KEY)),
        removable: true,
    },
    RunKey {
        source: StartupSource::UserRunOnce,
        hive: Hive::CurrentUser,
        path: RUN_ONCE_KEY,
        approved: None,
        removable: true,
    },
    RunKey {
        source: StartupSource::MachineRun,
        hive: Hive::LocalMachine,
        path: RUN_KEY,
        approved: Some((Hive::LocalMachine, APPROVED_RUN_KEY)),
        removable: true,
    },
    RunKey {
        source: StartupSource::MachineRun32,
        hive: Hive::LocalMachine,
        path: RUN32_KEY,
        approved: Some((Hive::LocalMachine, APPROVED_RUN32_KEY)),
        removable: true,
    },
    RunKey {
        source: StartupSource::MachineRunOnce,
        hive: Hive::LocalMachine,
        path: RUN_ONCE_KEY,
        approved: None,
        removable: true,
    },
    RunKey {
        source: StartupSource::UserPolicy,
        hive: Hive::CurrentUser,
        path: POLICY_RUN_KEY,
        approved: None,
        removable: false,
    },
    RunKey {
        source: StartupSource::MachinePolicy,
        hive: Hive::LocalMachine,
        path: POLICY_RUN_KEY,
        approved: None,
        removable: false,
    },
];

fn run_key(source: StartupSource) -> Option<&'static RunKey> {
    RUN_KEYS.iter().find(|k| k.source == source)
}

fn make_id(source: StartupSource, name: &str) -> String {
    format!("{}:{name}", source.tag())
}

/// Splits an id into its source and the name within that source.
pub fn parse_id(id: &str) -> Result<(StartupSource, &str)> {
    let (prefix, name) = id.split_once(':').ok_or_else(|| Error::Invalid(format!("startup id {id}")))?;
    let source = StartupSource::ALL
        .into_iter()
        .find(|s| s.tag() == prefix)
        .ok_or_else(|| Error::Invalid(format!("startup id {id}")))?;
    if name.is_empty() {
        return Err(Error::Invalid(format!("startup id {id}")));
    }
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

/// Store startup task states (`Windows.ApplicationModel.StartupTaskState`).
mod store_state {
    // 0 means disabled because the app never asked to start.
    pub const DISABLED_BY_USER: u32 = 1;
    pub const ENABLED: u32 = 2;
    pub const DISABLED_BY_POLICY: u32 = 3;
    pub const ENABLED_BY_POLICY: u32 = 4;
}

/// `SpotifyAB.SpotifyMusic_zpdnekdrzrea0` becomes `Spotify Music`.
pub fn store_app_name(package_family: &str) -> String {
    let package = package_family.split('_').next().unwrap_or(package_family);
    let last = package.rsplit('.').next().unwrap_or(package);
    let mut out = String::new();
    let chars: Vec<char> = last.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        let next_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
        let prev = i.checked_sub(1).map(|p| chars[p]);
        let boundary = c.is_uppercase()
            && prev.is_some_and(|p| p.is_lowercase() || p.is_ascii_digit() || (p.is_uppercase() && next_lower));
        if boundary {
            out.push(' ');
        }
        out.push(*c);
    }
    if out.is_empty() { package_family.to_owned() } else { out }
}

fn hive_location(hive: Hive, path: &str) -> String {
    format!(r"{}\{path}", hive.short_name())
}

/// Every startup entry, sorted by name. `tasks` comes from the Task Scheduler.
pub fn list(reg: &dyn Registry, folders: &StartupFolders, tasks: &[ScheduledStartup]) -> Vec<StartupItem> {
    let mut items = Vec::new();

    for key in &RUN_KEYS {
        for name in reg.value_names(key.hive, key.path) {
            let command = reg.get_string(key.hive, key.path, &name).unwrap_or_default();
            let enabled = key.approved.is_none_or(|(hive, path)| is_approved(reg.get(hive, path, &name).as_ref()));
            items.push(StartupItem {
                id: make_id(key.source, &name),
                name,
                command,
                source: key.source,
                enabled,
                editable: key.approved.is_some(),
                removable: key.removable,
                needs_admin: key.hive == Hive::LocalMachine,
                location: hive_location(key.hive, key.path),
            });
        }
    }

    for (source, approved_hive) in
        [(StartupSource::UserFolder, Hive::CurrentUser), (StartupSource::CommonFolder, Hive::LocalMachine)]
    {
        let Some(dir) = folders.get(source) else { continue };
        for file in folder_entries(dir) {
            let enabled = is_approved(reg.get(approved_hive, APPROVED_FOLDER_KEY, &file).as_ref());
            let display =
                Path::new(&file).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or(file.clone());
            items.push(StartupItem {
                id: make_id(source, &file),
                name: display,
                command: dir.join(&file).to_string_lossy().into_owned(),
                source,
                enabled,
                editable: true,
                removable: true,
                needs_admin: source == StartupSource::CommonFolder,
                location: dir.to_string_lossy().into_owned(),
            });
        }
    }

    for package in reg.subkeys(Hive::CurrentUser, STORE_TASKS_KEY) {
        let package_key = format!(r"{STORE_TASKS_KEY}\{package}");
        for task in reg.subkeys(Hive::CurrentUser, &package_key) {
            let task_key = format!(r"{package_key}\{task}");
            let Some(state) = reg.get_dword(Hive::CurrentUser, &task_key, "State") else { continue };
            let by_policy = matches!(state, store_state::DISABLED_BY_POLICY | store_state::ENABLED_BY_POLICY);
            items.push(StartupItem {
                id: make_id(StartupSource::StoreApp, &format!(r"{package}\{task}")),
                name: store_app_name(&package),
                command: format!("{package}!{task}"),
                source: StartupSource::StoreApp,
                enabled: matches!(state, store_state::ENABLED | store_state::ENABLED_BY_POLICY),
                editable: !by_policy,
                removable: false,
                needs_admin: false,
                location: "Microsoft Store app".into(),
            });
        }
    }

    for task in tasks {
        let name = task.path.rsplit('\\').next().filter(|n| !n.is_empty()).unwrap_or(&task.path).to_owned();
        items.push(StartupItem {
            id: make_id(StartupSource::ScheduledTask, &task.path),
            name,
            command: task.command.clone(),
            source: StartupSource::ScheduledTask,
            enabled: task.enabled,
            editable: true,
            removable: false,
            needs_admin: false,
            location: format!("Task Scheduler {}", task.path),
        });
    }

    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then(a.id.cmp(&b.id)));
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

/// Turns a registry, folder or Store entry on or off. Scheduled tasks go
/// through [`crate::system::set_task_enabled`] instead.
pub fn set_enabled(reg: &dyn Registry, folders: &StartupFolders, id: &str, enabled: bool) -> Result<()> {
    let (source, name) = parse_id(id)?;
    let bytes = RegValue::Binary(approval_bytes(enabled, filetime_now()));
    match source {
        StartupSource::UserFolder | StartupSource::CommonFolder => {
            let dir = folders.get(source).ok_or_else(|| Error::NotFound("Startup folder".into()))?;
            if !dir.join(name).is_file() {
                return Err(Error::NotFound(name.to_owned()));
            }
            let hive = if source == StartupSource::UserFolder { Hive::CurrentUser } else { Hive::LocalMachine };
            reg.set(hive, APPROVED_FOLDER_KEY, name, bytes)
        }
        StartupSource::StoreApp => {
            let key = format!(r"{STORE_TASKS_KEY}\{name}");
            match reg.get_dword(Hive::CurrentUser, &key, "State") {
                None => Err(Error::NotFound(name.to_owned())),
                Some(store_state::DISABLED_BY_POLICY | store_state::ENABLED_BY_POLICY) => {
                    Err(Error::msg("Your organization manages this app's startup setting."))
                }
                Some(_) => {
                    let state = if enabled { store_state::ENABLED } else { store_state::DISABLED_BY_USER };
                    reg.set(Hive::CurrentUser, &key, "State", RegValue::Dword(state))
                }
            }
        }
        StartupSource::ScheduledTask => {
            Err(Error::Invalid("Scheduled tasks are changed through Task Scheduler.".into()))
        }
        _ => {
            let key = run_key(source).ok_or_else(|| Error::Invalid(id.to_owned()))?;
            let Some((hive, approved)) = key.approved else {
                return Err(Error::msg(match source {
                    StartupSource::UserPolicy | StartupSource::MachinePolicy => {
                        "Group policy starts this program, so it can't be turned off here."
                    }
                    _ => "This entry runs only once. Remove it instead if you don't want it to run.",
                }));
            };
            if reg.get(key.hive, key.path, name).is_none() {
                return Err(Error::NotFound(name.to_owned()));
            }
            reg.set(hive, approved, name, bytes)
        }
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
        removable: true,
        needs_admin: false,
        location: hive_location(Hive::CurrentUser, RUN_KEY),
    })
}

/// Wraps a bare path that contains spaces in quotes so Windows runs the right file.
pub fn quote_command(command: &str) -> String {
    let looks_like_bare_path =
        command.contains(' ') && !command.starts_with('"') && command.to_ascii_lowercase().ends_with(".exe");
    if looks_like_bare_path { format!("\"{command}\"") } else { command.to_owned() }
}

pub fn remove(reg: &dyn Registry, folders: &StartupFolders, id: &str) -> Result<()> {
    let (source, name) = parse_id(id)?;
    match source {
        StartupSource::UserFolder | StartupSource::CommonFolder => {
            let dir = folders.get(source).ok_or_else(|| Error::NotFound("Startup folder".into()))?;
            let path: PathBuf = dir.join(name);
            if path.parent() != Some(dir) || name.contains(['/', '\\']) {
                return Err(Error::Invalid(name.to_owned()));
            }
            std::fs::remove_file(&path)?;
            let hive = if source == StartupSource::UserFolder { Hive::CurrentUser } else { Hive::LocalMachine };
            let _ = reg.delete_value(hive, APPROVED_FOLDER_KEY, name);
            Ok(())
        }
        StartupSource::StoreApp => {
            Err(Error::msg("Store apps can only be turned off. Uninstall the app to remove it."))
        }
        StartupSource::ScheduledTask => {
            Err(Error::msg("Scheduled tasks can only be turned off here. Delete them in Task Scheduler."))
        }
        _ => {
            let key = run_key(source).ok_or_else(|| Error::Invalid(id.to_owned()))?;
            if !key.removable {
                return Err(Error::msg("Group policy starts this program, so it can't be removed here."));
            }
            reg.delete_value(key.hive, key.path, name)?;
            if let Some((hive, approved)) = key.approved {
                let _ = reg.delete_value(hive, approved, name);
            }
            Ok(())
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

/// A change to a startup entry, as handed to the elevated helper process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Enable,
    Disable,
    Remove,
}

impl Change {
    pub fn as_arg(self) -> &'static str {
        match self {
            Change::Enable => "enable",
            Change::Disable => "disable",
            Change::Remove => "remove",
        }
    }

    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "enable" => Some(Change::Enable),
            "disable" => Some(Change::Disable),
            "remove" => Some(Change::Remove),
            _ => None,
        }
    }
}

/// Ids can hold any character, so they cross the command line as hex.
pub fn encode_arg(id: &str) -> String {
    id.bytes().map(|b| format!("{b:02x}")).collect()
}

pub fn decode_arg(hex: &str) -> Option<String> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let bytes: Option<Vec<u8>> =
        (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok()).collect();
    String::from_utf8(bytes?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::MemoryRegistry;

    const NO_FOLDERS: StartupFolders = StartupFolders { user: None, common: None };

    fn reg_with_entries() -> MemoryRegistry {
        let reg = MemoryRegistry::new();
        reg.set(Hive::CurrentUser, RUN_KEY, "Discord", RegValue::String(r"C:\Discord\Update.exe".into())).unwrap();
        reg.set(Hive::CurrentUser, RUN_KEY, "OneDrive", RegValue::String(r#""C:\OneDrive.exe" /background"#.into()))
            .unwrap();
        reg.set(Hive::LocalMachine, RUN_KEY, "SecurityHealth", RegValue::String("SecurityHealthSystray.exe".into()))
            .unwrap();
        reg
    }

    fn find<'a>(items: &'a [StartupItem], name: &str) -> &'a StartupItem {
        items.iter().find(|i| i.name == name).unwrap_or_else(|| panic!("{name} not listed"))
    }

    #[test]
    fn lists_user_and_machine_entries() {
        let reg = reg_with_entries();
        let items = list(&reg, &NO_FOLDERS, &[]);
        assert_eq!(items.len(), 3);
        let machine = find(&items, "SecurityHealth");
        assert_eq!(machine.source, StartupSource::MachineRun);
        assert!(machine.editable && machine.needs_admin);
        assert!(items.iter().all(|i| i.enabled));
    }

    #[test]
    fn lists_entries_task_manager_files_elsewhere() {
        let reg = reg_with_entries();
        reg.set(Hive::LocalMachine, RUN32_KEY, "Steam", RegValue::String("steam.exe -silent".into())).unwrap();
        reg.set(Hive::CurrentUser, RUN_ONCE_KEY, "Setup", RegValue::String("setup.exe /finish".into())).unwrap();
        reg.set(Hive::LocalMachine, RUN_ONCE_KEY, "Driver", RegValue::String("driver.exe".into())).unwrap();
        reg.set(Hive::CurrentUser, POLICY_RUN_KEY, "Agent", RegValue::String("agent.exe".into())).unwrap();
        reg.set(Hive::LocalMachine, POLICY_RUN_KEY, "Corp", RegValue::String("corp.exe".into())).unwrap();
        let items = list(&reg, &NO_FOLDERS, &[]);
        assert_eq!(find(&items, "Steam").source, StartupSource::MachineRun32);
        assert!(find(&items, "Steam").editable);
        assert_eq!(find(&items, "Setup").source, StartupSource::UserRunOnce);
        assert!(!find(&items, "Setup").editable && find(&items, "Setup").removable);
        assert_eq!(find(&items, "Driver").source, StartupSource::MachineRunOnce);
        assert_eq!(find(&items, "Agent").source, StartupSource::UserPolicy);
        assert!(!find(&items, "Corp").editable && !find(&items, "Corp").removable);
        assert_eq!(items.len(), 8);
    }

    #[test]
    fn machine_entries_read_approval_from_the_machine_key() {
        let reg = reg_with_entries();
        // A disabled flag in the user's key must not affect the all-users entry.
        reg.set(Hive::CurrentUser, APPROVED_RUN_KEY, "SecurityHealth", RegValue::Binary(vec![3; 12])).unwrap();
        assert!(find(&list(&reg, &NO_FOLDERS, &[]), "SecurityHealth").enabled);
        set_enabled(&reg, &NO_FOLDERS, "machine-run:SecurityHealth", false).unwrap();
        assert!(reg.get(Hive::LocalMachine, APPROVED_RUN_KEY, "SecurityHealth").is_some());
        assert!(!find(&list(&reg, &NO_FOLDERS, &[]), "SecurityHealth").enabled);

        reg.set(Hive::LocalMachine, RUN32_KEY, "Steam", RegValue::String("steam.exe".into())).unwrap();
        set_enabled(&reg, &NO_FOLDERS, "machine-run32:Steam", false).unwrap();
        assert!(reg.get(Hive::LocalMachine, APPROVED_RUN32_KEY, "Steam").is_some());
        assert!(!find(&list(&reg, &NO_FOLDERS, &[]), "Steam").enabled);
    }

    #[test]
    fn disabling_uses_startup_approved_and_keeps_entry() {
        let reg = reg_with_entries();
        set_enabled(&reg, &NO_FOLDERS, "user-run:Discord", false).unwrap();
        let bytes = reg.get(Hive::CurrentUser, APPROVED_RUN_KEY, "Discord").unwrap();
        let RegValue::Binary(b) = &bytes else { panic!("expected binary") };
        assert_eq!(b.len(), 12);
        assert_eq!(b[0], 0x03);
        assert!(reg.get(Hive::CurrentUser, RUN_KEY, "Discord").is_some(), "entry must not be deleted");
        assert!(!find(&list(&reg, &NO_FOLDERS, &[]), "Discord").enabled);

        set_enabled(&reg, &NO_FOLDERS, "user-run:Discord", true).unwrap();
        assert!(find(&list(&reg, &NO_FOLDERS, &[]), "Discord").enabled);
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
    fn run_once_and_policy_entries_cannot_be_toggled() {
        let reg = MemoryRegistry::new();
        reg.set(Hive::CurrentUser, RUN_ONCE_KEY, "Setup", RegValue::String("setup.exe".into())).unwrap();
        reg.set(Hive::LocalMachine, POLICY_RUN_KEY, "Corp", RegValue::String("corp.exe".into())).unwrap();
        assert!(set_enabled(&reg, &NO_FOLDERS, "user-run-once:Setup", false).is_err());
        assert!(set_enabled(&reg, &NO_FOLDERS, "machine-policy:Corp", false).is_err());
        assert!(remove(&reg, &NO_FOLDERS, "machine-policy:Corp").is_err());
        remove(&reg, &NO_FOLDERS, "user-run-once:Setup").unwrap();
        assert!(list(&reg, &NO_FOLDERS, &[]).iter().all(|i| i.name != "Setup"));
    }

    #[test]
    fn missing_entries_and_bad_ids_are_rejected() {
        let reg = reg_with_entries();
        assert!(set_enabled(&reg, &NO_FOLDERS, "user-run:Nope", false).is_err());
        assert!(set_enabled(&reg, &NO_FOLDERS, "bogus:Discord", false).is_err());
        assert!(set_enabled(&reg, &NO_FOLDERS, "user-run:", false).is_err());
        assert!(parse_id("no-colon").is_err());
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
        set_enabled(&reg, &NO_FOLDERS, "user-run:Discord", false).unwrap();
        remove(&reg, &NO_FOLDERS, "user-run:Discord").unwrap();
        assert!(reg.get(Hive::CurrentUser, RUN_KEY, "Discord").is_none());
        assert!(reg.get(Hive::CurrentUser, APPROVED_RUN_KEY, "Discord").is_none());
    }

    #[test]
    fn migrates_v1_disabled_entries() {
        let reg = MemoryRegistry::new();
        reg.set(Hive::CurrentUser, LEGACY_DISABLED_KEY, "Spotify", RegValue::String("spotify.exe".into())).unwrap();
        assert_eq!(migrate_legacy(&reg), 1);
        assert!(!find(&list(&reg, &NO_FOLDERS, &[]), "Spotify").enabled);
        assert!(reg.value_names(Hive::CurrentUser, LEGACY_DISABLED_KEY).is_empty());
        assert_eq!(migrate_legacy(&reg), 0, "idempotent");
    }

    #[test]
    fn startup_folder_entries() {
        let user = tempfile::tempdir().unwrap();
        let common = tempfile::tempdir().unwrap();
        std::fs::write(user.path().join("Notes.lnk"), b"").unwrap();
        std::fs::write(user.path().join("desktop.ini"), b"").unwrap();
        std::fs::write(common.path().join("Vendor Tray.lnk"), b"").unwrap();
        let folders = StartupFolders { user: Some(user.path().into()), common: Some(common.path().into()) };
        let reg = MemoryRegistry::new();
        let items = list(&reg, &folders, &[]);
        assert_eq!(items.len(), 2);
        let notes = find(&items, "Notes").clone();
        let vendor = find(&items, "Vendor Tray").clone();
        assert_eq!(vendor.source, StartupSource::CommonFolder);
        assert!(vendor.needs_admin && !notes.needs_admin);

        set_enabled(&reg, &folders, &notes.id, false).unwrap();
        set_enabled(&reg, &folders, &vendor.id, false).unwrap();
        assert!(reg.get(Hive::LocalMachine, APPROVED_FOLDER_KEY, "Vendor Tray.lnk").is_some());
        assert!(list(&reg, &folders, &[]).iter().all(|i| !i.enabled));

        remove(&reg, &folders, &notes.id).unwrap();
        remove(&reg, &folders, &vendor.id).unwrap();
        assert!(list(&reg, &folders, &[]).is_empty());
        assert!(remove(&reg, &folders, "user-folder:..\\x").is_err());
    }

    #[test]
    fn store_app_startup_tasks() {
        let reg = MemoryRegistry::new();
        let spotify = format!(r"{STORE_TASKS_KEY}\SpotifyAB.SpotifyMusic_zpdnekdrzrea0\SpotifyStartupTask");
        let teams = format!(r"{STORE_TASKS_KEY}\MSTeams_8wekyb3d8bbwe\TeamsTfwStartupTask");
        let managed = format!(r"{STORE_TASKS_KEY}\Contoso.Agent_abc\AgentTask");
        reg.set(Hive::CurrentUser, &spotify, "State", RegValue::Dword(2)).unwrap();
        reg.set(Hive::CurrentUser, &teams, "State", RegValue::Dword(1)).unwrap();
        reg.set(Hive::CurrentUser, &managed, "State", RegValue::Dword(4)).unwrap();
        // A package key without a startup task is ignored.
        reg.set(Hive::CurrentUser, &format!(r"{STORE_TASKS_KEY}\Other_x\Schemas"), "x", RegValue::Dword(1)).unwrap();

        let items = list(&reg, &NO_FOLDERS, &[]);
        assert_eq!(items.len(), 3);
        let s = find(&items, "Spotify Music").clone();
        assert!(s.enabled && s.editable && !s.removable);
        assert!(!find(&items, "MS Teams").enabled);
        assert!(!find(&items, "Agent").editable);

        set_enabled(&reg, &NO_FOLDERS, &s.id, false).unwrap();
        assert_eq!(reg.get_dword(Hive::CurrentUser, &spotify, "State"), Some(1));
        set_enabled(&reg, &NO_FOLDERS, &s.id, true).unwrap();
        assert_eq!(reg.get_dword(Hive::CurrentUser, &spotify, "State"), Some(2));
        assert!(set_enabled(&reg, &NO_FOLDERS, &find(&items, "Agent").id, false).is_err());
        assert!(remove(&reg, &NO_FOLDERS, &s.id).is_err());
    }

    #[test]
    fn store_names_are_readable() {
        assert_eq!(store_app_name("SpotifyAB.SpotifyMusic_zpdnekdrzrea0"), "Spotify Music");
        assert_eq!(store_app_name("Microsoft.WindowsTerminal_8wekyb3d8bbwe"), "Windows Terminal");
        assert_eq!(store_app_name("MSTeams_8wekyb3d8bbwe"), "MS Teams");
        assert_eq!(store_app_name("5319275A.WhatsAppDesktop_cv1g1gvanyjgm"), "Whats App Desktop");
    }

    #[test]
    fn scheduled_tasks_are_listed() {
        let reg = MemoryRegistry::new();
        let tasks = [ScheduledStartup {
            path: r"\Opera GX scheduled assistant Autoupdate".into(),
            command: r#""C:\Opera\launcher.exe" --scheduledautoupdate"#.into(),
            enabled: false,
        }];
        let items = list(&reg, &NO_FOLDERS, &tasks);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Opera GX scheduled assistant Autoupdate");
        assert_eq!(items[0].source, StartupSource::ScheduledTask);
        assert!(!items[0].enabled && items[0].editable);
        assert_eq!(parse_id(&items[0].id).unwrap(), (StartupSource::ScheduledTask, tasks[0].path.as_str()));
        assert!(set_enabled(&reg, &NO_FOLDERS, &items[0].id, true).is_err(), "handled by the system layer");
    }

    #[test]
    fn helper_arguments_round_trip() {
        let id = r#"user-run:Weird "name" ünïcode\x"#;
        assert_eq!(decode_arg(&encode_arg(id)).as_deref(), Some(id));
        assert!(decode_arg("abc").is_none());
        assert!(decode_arg("zz").is_none());
        assert_eq!(Change::from_arg(Change::Remove.as_arg()), Some(Change::Remove));
        assert_eq!(Change::from_arg("format-c"), None);
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
