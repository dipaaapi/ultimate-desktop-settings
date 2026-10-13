use windows_registry::{CLASSES_ROOT, CURRENT_USER, Key, LOCAL_MACHINE, Type, Value};

use crate::registry::{Hive, RegValue, Registry};
use crate::{Error, Result};

/// The real Windows registry.
pub struct WindowsRegistry;

fn root(hive: Hive) -> &'static Key {
    match hive {
        Hive::CurrentUser => CURRENT_USER,
        Hive::LocalMachine => LOCAL_MACHINE,
        Hive::ClassesRoot => CLASSES_ROOT,
    }
}

fn open_read(hive: Hive, path: &str) -> Option<Key> {
    root(hive).options().read().open(path).ok()
}

fn to_error(err: windows::core::Error, what: &str) -> Error {
    Error::msg(format!("Could not change {what}: {}", err.message()))
}

fn decode_wide(bytes: &[u8]) -> String {
    let wide: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    let end = wide.iter().position(|c| *c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..end])
}

impl Registry for WindowsRegistry {
    fn get(&self, hive: Hive, path: &str, name: &str) -> Option<RegValue> {
        let value: Value = open_read(hive, path)?.get_value(name).ok()?;
        Some(match value.ty() {
            Type::String => RegValue::String(decode_wide(&value)),
            Type::ExpandString => RegValue::ExpandString(decode_wide(&value)),
            Type::U32 => RegValue::Dword(u32::try_from(value).ok()?),
            Type::Bytes => RegValue::Binary(value.to_vec()),
            _ => return None,
        })
    }

    fn set(&self, hive: Hive, path: &str, name: &str, value: RegValue) -> Result<()> {
        let what = format!(r"{}\{path}\{name}", hive.short_name());
        let key = root(hive).options().read().write().create().open(path).map_err(|e| to_error(e, &what))?;
        match value {
            RegValue::String(s) => key.set_string(name, s),
            RegValue::ExpandString(s) => key.set_expand_string(name, s),
            RegValue::Dword(v) => key.set_u32(name, v),
            RegValue::Binary(b) => key.set_bytes(name, Type::Bytes, &b),
        }
        .map_err(|e| to_error(e, &what))
    }

    fn delete_value(&self, hive: Hive, path: &str, name: &str) -> Result<()> {
        let what = format!(r"{}\{path}\{name}", hive.short_name());
        let key = root(hive).options().read().write().open(path).map_err(|e| to_error(e, &what))?;
        key.remove_value(name).map_err(|e| to_error(e, &what))
    }

    fn value_names(&self, hive: Hive, path: &str) -> Vec<String> {
        open_read(hive, path)
            .and_then(|key| {
                key.values().ok().map(|values| values.map(|(name, _)| name).filter(|n| !n.is_empty()).collect())
            })
            .unwrap_or_default()
    }

    fn subkeys(&self, hive: Hive, path: &str) -> Vec<String> {
        open_read(hive, path).and_then(|key| key.keys().ok().map(|keys| keys.collect())).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = r"Software\UltimateWindowsDesktopSettings\Tests";

    #[test]
    fn real_registry_round_trip() {
        let reg = WindowsRegistry;
        reg.set(Hive::CurrentUser, TEST_KEY, "Text", RegValue::String("héllo".into())).unwrap();
        reg.set(Hive::CurrentUser, TEST_KEY, "Number", RegValue::Dword(42)).unwrap();
        reg.set(Hive::CurrentUser, TEST_KEY, "Bytes", RegValue::Binary(vec![3, 0, 0, 0])).unwrap();
        reg.set(Hive::CurrentUser, &format!(r"{TEST_KEY}\Child"), "", RegValue::String("default".into())).unwrap();

        assert_eq!(reg.get_string(Hive::CurrentUser, TEST_KEY, "Text").as_deref(), Some("héllo"));
        assert_eq!(reg.get_dword(Hive::CurrentUser, TEST_KEY, "Number"), Some(42));
        assert_eq!(reg.get(Hive::CurrentUser, TEST_KEY, "Bytes"), Some(RegValue::Binary(vec![3, 0, 0, 0])));
        assert_eq!(reg.get_string(Hive::CurrentUser, &format!(r"{TEST_KEY}\Child"), "").as_deref(), Some("default"));

        let mut names = reg.value_names(Hive::CurrentUser, TEST_KEY);
        names.sort();
        assert_eq!(names, ["Bytes", "Number", "Text"]);
        assert_eq!(reg.subkeys(Hive::CurrentUser, TEST_KEY), ["Child"]);

        reg.delete_value(Hive::CurrentUser, TEST_KEY, "Text").unwrap();
        assert!(reg.get(Hive::CurrentUser, TEST_KEY, "Text").is_none());
        assert!(reg.delete_value(Hive::CurrentUser, TEST_KEY, "Text").is_err());
        assert!(reg.value_names(Hive::CurrentUser, r"Software\Does\Not\Exist").is_empty());
        let _ = CURRENT_USER.remove_tree(r"Software\UltimateWindowsDesktopSettings");
    }

    #[test]
    fn startup_and_tweaks_against_real_registry() {
        use crate::{startup, tweaks};
        let reg = WindowsRegistry;
        let item = startup::add(&reg, "UWDS Test Entry", r"C:\Windows\notepad.exe").unwrap();
        startup::set_enabled(&reg, &item.id, false).unwrap();
        let listed = startup::list(&reg, None).into_iter().find(|i| i.id == item.id).unwrap();
        assert!(!listed.enabled);
        startup::remove(&reg, &item.id, None).unwrap();
        assert!(startup::list(&reg, None).iter().all(|i| i.id != item.id));

        let before = tweaks::list(&reg).into_iter().find(|t| t.id == "compact-view").unwrap().enabled;
        tweaks::set(&reg, "compact-view", !before).unwrap();
        assert_eq!(tweaks::list(&reg).into_iter().find(|t| t.id == "compact-view").unwrap().enabled, !before);
        tweaks::set(&reg, "compact-view", before).unwrap();
    }
}
