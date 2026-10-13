//! Installed desktop apps, read straight from the Uninstall registry keys
//! (the same list Settings > Apps shows), without PowerShell.

use serde::Serialize;

use crate::registry::{Hive, Registry};

const UNINSTALL: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
const UNINSTALL_WOW64: &str = r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AppCategory {
    Development,
    #[serde(rename = "Design & Creative")]
    DesignCreative,
    Gaming,
    #[serde(rename = "Office & General Use")]
    OfficeGeneral,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub publisher: Option<String>,
    pub install_location: Option<String>,
    /// Program to start, when the registry points at one.
    pub exe: Option<String>,
    pub category: AppCategory,
}

const DEV: &[&str] = &[
    "visual studio",
    "vs code",
    "git",
    "github",
    "flutter",
    "android studio",
    "python",
    "node.js",
    "java",
    "jdk",
    "xampp",
    "laragon",
    "docker",
    "postman",
    "jetbrains",
    "intellij",
    "pycharm",
    "webstorm",
    "rust",
    "wsl",
    "sql",
    "dbeaver",
    "putty",
    "winscp",
    "cursor",
    "windsurf",
    "antigravity",
    "herd",
    "composer",
];
const DESIGN: &[&str] = &[
    "adobe",
    "photoshop",
    "illustrator",
    "blender",
    "corel",
    "figma",
    "premiere",
    "autocad",
    "affinity",
    "aseprite",
    "gimp",
    "inkscape",
    "krita",
    "canva",
    "davinci",
    "clipchamp",
    "obs studio",
    "unity",
    "unreal",
    "sketchup",
    "capcut",
    "ansel",
    "darktable",
];
const GAMING: &[&str] = &[
    "steam",
    "epic games",
    "riot",
    "valorant",
    "league of legends",
    "gta",
    "game",
    "nintendo",
    "ubisoft",
    "games",
    "ea app",
    "electronic arts",
    "battle.net",
    "blizzard",
    "gog",
    "xbox",
    "rockstar",
    "diablo",
    "minecraft",
    "ds4windows",
    "red alert",
    "command and conquer",
];

pub fn categorize(name: &str, publisher: Option<&str>) -> AppCategory {
    // Whole-word matching so "community" doesn't match "unity" and "digital" doesn't match "git".
    let text = format!("{} {}", name, publisher.unwrap_or_default()).to_lowercase();
    let words: String = text.chars().map(|c| if c.is_alphanumeric() || c == '.' { c } else { ' ' }).collect();
    let haystack = format!(" {} ", words.split_whitespace().collect::<Vec<_>>().join(" "));
    let hit = |list: &[&str]| list.iter().any(|w| haystack.contains(&format!(" {w} ")));
    if hit(GAMING) {
        AppCategory::Gaming
    } else if hit(DESIGN) {
        AppCategory::DesignCreative
    } else if hit(DEV) {
        AppCategory::Development
    } else {
        AppCategory::OfficeGeneral
    }
}

fn is_system_noise(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("microsoft visual c++")
        || lower.contains("windows driver package")
        || lower.contains("update for")
        || lower.contains("security update")
        || lower.contains("redistributable")
        || lower.contains("windows software development kit")
        || (lower.contains("kb") && lower.chars().filter(char::is_ascii_digit).count() >= 6)
}

/// Turns a `DisplayIcon` value like `"C:\App\app.exe",0` into a launchable exe.
pub fn exe_from_display_icon(icon: &str) -> Option<String> {
    let trimmed = icon.trim().trim_matches('"');
    let path = match trimmed.rsplit_once(',') {
        Some((p, idx)) if idx.trim().trim_start_matches('-').chars().all(|c| c.is_ascii_digit()) => p,
        _ => trimmed,
    };
    let path = path.trim().trim_matches('"');
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path).to_lowercase();
    let is_exe = file.ends_with(".exe");
    let is_installer =
        file.starts_with("unins") || file.contains("uninstall") || file.starts_with("setup") || file == "msiexec.exe";
    (is_exe && !is_installer).then(|| path.to_owned())
}

pub fn list(reg: &dyn Registry) -> Vec<InstalledApp> {
    let mut apps: Vec<InstalledApp> = Vec::new();
    let sources = [
        (Hive::LocalMachine, UNINSTALL, "HKLM"),
        (Hive::LocalMachine, UNINSTALL_WOW64, "HKLM32"),
        (Hive::CurrentUser, UNINSTALL, "HKCU"),
    ];
    for (hive, root, tag) in sources {
        for key in reg.subkeys(hive, root) {
            let path = format!(r"{root}\{key}");
            let Some(name) = reg.get_string(hive, &path, "DisplayName").map(|n| n.trim().to_owned()) else { continue };
            if name.is_empty()
                || reg.get_string(hive, &path, "UninstallString").is_none()
                || reg.get_dword(hive, &path, "SystemComponent") == Some(1)
                || reg.get_string(hive, &path, "ParentKeyName").is_some()
                || reg
                    .get_string(hive, &path, "ReleaseType")
                    .is_some_and(|t| t.to_lowercase().contains("update") || t.eq_ignore_ascii_case("hotfix"))
                || is_system_noise(&name)
                || apps.iter().any(|a| a.name.eq_ignore_ascii_case(&name))
            {
                continue;
            }
            let non_empty = |v: Option<String>| v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
            let publisher = non_empty(reg.get_string(hive, &path, "Publisher"));
            let install_location = non_empty(reg.get_string(hive, &path, "InstallLocation"))
                .map(|p| p.trim_matches('"').trim_end_matches('\\').to_owned());
            let exe = reg.get_string(hive, &path, "DisplayIcon").as_deref().and_then(exe_from_display_icon);
            apps.push(InstalledApp {
                id: format!("{tag}:{key}"),
                category: categorize(&name, publisher.as_deref()),
                name,
                version: non_empty(reg.get_string(hive, &path, "DisplayVersion")),
                publisher,
                install_location,
                exe,
            });
        }
    }
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

/// Whether a running process (by its full image path) belongs to this app.
pub fn process_belongs_to(process_path: &str, app: &InstalledApp) -> bool {
    let process = process_path.to_lowercase().replace('/', "\\");
    if let Some(exe) = &app.exe {
        if process == exe.to_lowercase().replace('/', "\\") {
            return true;
        }
    }
    match &app.install_location {
        Some(dir) if dir.len() > 3 => {
            let dir = format!("{}\\", dir.to_lowercase().replace('/', "\\").trim_end_matches('\\'));
            process.starts_with(&dir)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{MemoryRegistry, RegValue};

    fn add(reg: &MemoryRegistry, hive: Hive, root: &str, key: &str, values: &[(&str, &str)]) {
        let path = format!(r"{root}\{key}");
        for (n, v) in values {
            reg.set(hive, &path, n, RegValue::String((*v).into())).unwrap();
        }
    }

    #[test]
    fn reads_and_filters_uninstall_keys() {
        let reg = MemoryRegistry::new();
        add(
            &reg,
            Hive::LocalMachine,
            UNINSTALL,
            "Steam",
            &[
                ("DisplayName", "Steam"),
                ("UninstallString", "x"),
                ("DisplayIcon", r"C:\Steam\steam.exe"),
                ("InstallLocation", r"C:\Steam"),
                ("Publisher", "Valve"),
            ],
        );
        add(
            &reg,
            Hive::CurrentUser,
            UNINSTALL,
            "Code",
            &[
                ("DisplayName", "Microsoft Visual Studio Code (User)"),
                ("UninstallString", "x"),
                ("DisplayIcon", r#""C:\VSCode\Code.exe",0"#),
            ],
        );
        add(
            &reg,
            Hive::LocalMachine,
            UNINSTALL_WOW64,
            "VC",
            &[("DisplayName", "Microsoft Visual C++ 2015-2022 Redistributable (x86)"), ("UninstallString", "x")],
        );
        add(&reg, Hive::LocalMachine, UNINSTALL, "NoUninstall", &[("DisplayName", "Ghost")]);
        add(
            &reg,
            Hive::LocalMachine,
            UNINSTALL,
            "Child",
            &[("DisplayName", "Patch"), ("UninstallString", "x"), ("ParentKeyName", "Office")],
        );
        reg.set(Hive::LocalMachine, &format!(r"{UNINSTALL}\Hidden"), "SystemComponent", RegValue::Dword(1)).unwrap();
        add(&reg, Hive::LocalMachine, UNINSTALL, "Hidden", &[("DisplayName", "Hidden"), ("UninstallString", "x")]);

        let apps = list(&reg);
        let names: Vec<_> = apps.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Microsoft Visual Studio Code (User)", "Steam"]);
        assert_eq!(apps[0].category, AppCategory::Development);
        assert_eq!(apps[0].exe.as_deref(), Some(r"C:\VSCode\Code.exe"));
        assert_eq!(apps[1].category, AppCategory::Gaming);
        assert_eq!(apps[1].id, "HKLM:Steam");
    }

    #[test]
    fn display_icon_parsing() {
        assert_eq!(exe_from_display_icon(r"C:\A\a.exe,-101").as_deref(), Some(r"C:\A\a.exe"));
        assert_eq!(exe_from_display_icon(r#""C:\A b\a.exe""#).as_deref(), Some(r"C:\A b\a.exe"));
        assert_eq!(exe_from_display_icon(r"C:\A\unins000.exe"), None);
        assert_eq!(exe_from_display_icon(r"C:\A\icon.ico"), None);
    }

    #[test]
    fn matches_processes_by_folder_or_exe() {
        let app = InstalledApp {
            id: "x".into(),
            name: "Tool".into(),
            version: None,
            publisher: None,
            install_location: Some(r"C:\Program Files\Tool".into()),
            exe: Some(r"D:\Elsewhere\tool.exe".into()),
            category: AppCategory::OfficeGeneral,
        };
        assert!(process_belongs_to(r"c:\program files\tool\bin\tool.exe", &app));
        assert!(process_belongs_to(r"D:\Elsewhere\tool.exe", &app));
        assert!(!process_belongs_to(r"C:\Program Files\ToolKit\x.exe", &app), "sibling folder must not match");
        let root = InstalledApp { install_location: Some(r"C:\".into()), exe: None, ..app };
        assert!(!process_belongs_to(r"C:\Windows\explorer.exe", &root), "drive roots never match");
    }

    #[test]
    fn categories() {
        assert_eq!(categorize("Aseprite", None), AppCategory::DesignCreative);
        assert_eq!(categorize("Riot Client", Some("Riot Games")), AppCategory::Gaming);
        assert_eq!(categorize("Zoom Workplace", Some("Zoom")), AppCategory::OfficeGeneral);
        assert_eq!(categorize("Visual Studio Community 2022", Some("Microsoft Corporation")), AppCategory::Development);
        assert_eq!(categorize("Digital Photo Viewer", None), AppCategory::OfficeGeneral);
        assert_eq!(categorize("Node.js", None), AppCategory::Development);
    }
}
