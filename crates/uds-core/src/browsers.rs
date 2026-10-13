//! Installed web browsers and the current default browser.

use serde::Serialize;

use crate::registry::{Hive, Registry};

const CLIENTS: &str = r"SOFTWARE\Clients\StartMenuInternet";
const USER_CHOICE: &str = r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\{}\UserChoice";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowserKind {
    Chrome,
    Edge,
    Firefox,
    Brave,
    Opera,
    Vivaldi,
    Other,
}

impl BrowserKind {
    pub fn from_exe(exe: &str) -> Self {
        let file = exe.rsplit(['\\', '/']).next().unwrap_or(exe).to_ascii_lowercase();
        let lower = exe.to_ascii_lowercase();
        match file.as_str() {
            "chrome.exe" => Self::Chrome,
            "msedge.exe" => Self::Edge,
            "firefox.exe" => Self::Firefox,
            "brave.exe" => Self::Brave,
            "vivaldi.exe" => Self::Vivaldi,
            "opera.exe" => Self::Opera,
            "launcher.exe" if lower.contains("opera") => Self::Opera,
            _ => Self::Other,
        }
    }

    /// Command-line switch that opens a private window, if the browser has one.
    pub fn private_flag(self) -> Option<&'static str> {
        match self {
            Self::Chrome | Self::Brave | Self::Vivaldi => Some("--incognito"),
            Self::Edge => Some("-inprivate"),
            Self::Firefox => Some("-private-window"),
            Self::Opera => Some("--private"),
            Self::Other => None,
        }
    }

    fn matches_prog_id(self, prog_id: &str) -> bool {
        let p = prog_id.to_ascii_lowercase();
        match self {
            Self::Chrome => p.starts_with("chromehtml"),
            Self::Edge => p.starts_with("msedgehtm"),
            Self::Firefox => p.starts_with("firefoxurl"),
            Self::Brave => p.starts_with("bravehtml"),
            Self::Opera => p.starts_with("opera"),
            Self::Vivaldi => p.starts_with("vivaldihtm"),
            Self::Other => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Browser {
    pub id: String,
    pub name: String,
    pub exe: String,
    pub kind: BrowserKind,
    pub is_default: bool,
    pub supports_private: bool,
}

/// Extracts the program path from a shell command such as
/// `"C:\Program Files\App\app.exe" --flag "%1"`.
pub fn exe_from_command(command: &str) -> String {
    let command = command.trim();
    if let Some(rest) = command.strip_prefix('"') {
        return rest.split('"').next().unwrap_or_default().to_owned();
    }
    let lower = command.to_ascii_lowercase();
    match lower.find(".exe") {
        Some(end) => command[..end + 4].to_owned(),
        None => command.split_whitespace().next().unwrap_or_default().to_owned(),
    }
}

pub fn default_prog_id(reg: &dyn Registry) -> Option<String> {
    ["https", "http"]
        .iter()
        .find_map(|scheme| reg.get_string(Hive::CurrentUser, &USER_CHOICE.replace("{}", scheme), "ProgId"))
}

pub fn list(reg: &dyn Registry) -> Vec<Browser> {
    let prog_id = default_prog_id(reg);
    let default_exe = prog_id
        .as_deref()
        .and_then(|id| reg.get_string(Hive::ClassesRoot, &format!(r"{id}\shell\open\command"), ""))
        .map(|cmd| exe_from_command(&cmd).to_ascii_lowercase());

    let mut browsers: Vec<Browser> = Vec::new();
    for hive in [Hive::CurrentUser, Hive::LocalMachine] {
        for key in reg.subkeys(hive, CLIENTS) {
            let base = format!(r"{CLIENTS}\{key}");
            let name = reg.get_string(hive, &base, "").filter(|n| !n.trim().is_empty()).unwrap_or_else(|| key.clone());
            if name.to_ascii_lowercase().contains("internet explorer") || key.eq_ignore_ascii_case("IEXPLORE.EXE") {
                continue;
            }
            let Some(command) = reg.get_string(hive, &format!(r"{base}\shell\open\command"), "") else { continue };
            let exe = exe_from_command(&command);
            if exe.is_empty() || browsers.iter().any(|b| b.exe.eq_ignore_ascii_case(&exe)) {
                continue;
            }
            let kind = BrowserKind::from_exe(&exe);
            let is_default = match (&default_exe, &prog_id) {
                (Some(def), _) if !def.is_empty() => *def == exe.to_ascii_lowercase(),
                (_, Some(id)) => kind.matches_prog_id(id),
                _ => false,
            };
            browsers.push(Browser {
                id: key,
                name,
                exe,
                kind,
                is_default,
                supports_private: kind.private_flag().is_some(),
            });
        }
    }
    browsers.sort_by(|a, b| b.is_default.cmp(&a.is_default).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    browsers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{MemoryRegistry, RegValue};

    fn add_browser(reg: &MemoryRegistry, hive: Hive, key: &str, name: &str, command: &str) {
        let base = format!(r"{CLIENTS}\{key}");
        reg.set(hive, &base, "", RegValue::String(name.into())).unwrap();
        reg.set(hive, &format!(r"{base}\shell\open\command"), "", RegValue::String(command.into())).unwrap();
    }

    #[test]
    fn parses_commands() {
        assert_eq!(
            exe_from_command(r#""C:\Program Files\Google\Chrome\Application\chrome.exe""#),
            r"C:\Program Files\Google\Chrome\Application\chrome.exe"
        );
        assert_eq!(exe_from_command(r"C:\Opera\launcher.exe --single"), r"C:\Opera\launcher.exe");
        assert_eq!(exe_from_command("  "), "");
    }

    #[test]
    fn detects_browsers_and_default() {
        let reg = MemoryRegistry::new();
        add_browser(&reg, Hive::LocalMachine, "Google Chrome", "Google Chrome", r#""C:\Chrome\chrome.exe""#);
        add_browser(&reg, Hive::LocalMachine, "Microsoft Edge", "Microsoft Edge", r#""C:\Edge\msedge.exe""#);
        add_browser(&reg, Hive::LocalMachine, "IEXPLORE.EXE", "Internet Explorer", r#""C:\IE\iexplore.exe""#);
        add_browser(&reg, Hive::CurrentUser, "Firefox-308046B0AF4A39CB", "Firefox", r#""C:\Firefox\firefox.exe""#);
        reg.set(Hive::CurrentUser, &USER_CHOICE.replace("{}", "https"), "ProgId", RegValue::String("MSEdgeHTM".into()))
            .unwrap();
        reg.set(
            Hive::ClassesRoot,
            r"MSEdgeHTM\shell\open\command",
            "",
            RegValue::String(r#""C:\Edge\msedge.exe" --single-argument %1"#.into()),
        )
        .unwrap();

        let list = list(&reg);
        assert_eq!(list.len(), 3, "Internet Explorer is skipped");
        assert_eq!(list[0].kind, BrowserKind::Edge);
        assert!(list[0].is_default);
        assert_eq!(list.iter().filter(|b| b.is_default).count(), 1);
        assert!(list.iter().all(|b| b.supports_private));
    }

    #[test]
    fn falls_back_to_prog_id_prefix() {
        let reg = MemoryRegistry::new();
        add_browser(&reg, Hive::LocalMachine, "Brave", "Brave", r#""C:\Brave\brave.exe""#);
        reg.set(Hive::CurrentUser, &USER_CHOICE.replace("{}", "http"), "ProgId", RegValue::String("BraveHTML".into()))
            .unwrap();
        assert!(list(&reg)[0].is_default);
    }

    #[test]
    fn private_flags() {
        assert_eq!(BrowserKind::Edge.private_flag(), Some("-inprivate"));
        assert_eq!(BrowserKind::Firefox.private_flag(), Some("-private-window"));
        assert_eq!(BrowserKind::from_exe(r"C:\x\Opera\launcher.exe"), BrowserKind::Opera);
        assert_eq!(BrowserKind::Other.private_flag(), None);
    }
}
