//! Scheduled tasks that start programs at sign-in or boot. Task Manager's
//! Startup page doesn't show these, so updaters and helpers often hide here.

use windows::Win32::Foundation::{VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::TaskScheduler::{
    IExecAction, IRegisteredTask, ITaskFolder, ITaskService, TASK_ACTION_EXEC, TASK_ENUM_HIDDEN, TASK_TRIGGER_BOOT,
    TASK_TRIGGER_LOGON, TASK_TRIGGER_TYPE2, TaskScheduler,
};
use windows::Win32::System::Variant::{VARIANT, VT_I4};
use windows::core::{BSTR, Interface};

use super::audio::ComGuard;
use crate::startup::ScheduledStartup;
use crate::{Error, Result};

/// Windows' own maintenance tasks live here; they are not apps the user installed.
const SYSTEM_FOLDER: &str = r"\Microsoft\Windows";
/// Stops a damaged task store from making the walk run forever.
const MAX_FOLDERS: usize = 2000;

fn index(i: i32) -> VARIANT {
    let mut v = VARIANT::default();
    unsafe {
        let inner = &mut *v.Anonymous.Anonymous;
        inner.vt = VT_I4;
        inner.Anonymous.lVal = i;
    }
    v
}

fn service() -> Result<ITaskService> {
    unsafe {
        let service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| Error::msg(format!("Task Scheduler is not available: {}", e.message())))?;
        let empty = VARIANT::default();
        service
            .Connect(&empty, &empty, &empty, &empty)
            .map_err(|e| Error::msg(format!("Task Scheduler is not available: {}", e.message())))?;
        Ok(service)
    }
}

fn starts_at_sign_in(task: &IRegisteredTask) -> Option<String> {
    unsafe {
        let definition = task.Definition().ok()?;
        let triggers = definition.Triggers().ok()?;
        let mut count = 0;
        triggers.Count(&mut count).ok()?;
        let at_start = (1..=count).any(|i| {
            let Ok(trigger) = triggers.get_Item(i) else { return false };
            let mut kind = TASK_TRIGGER_TYPE2::default();
            trigger.Type(&mut kind).is_ok() && (kind == TASK_TRIGGER_LOGON || kind == TASK_TRIGGER_BOOT)
        });
        if !at_start {
            return None;
        }
        let actions = definition.Actions().ok()?;
        actions.Count(&mut count).ok()?;
        let mut commands = Vec::new();
        for i in 1..=count {
            let Ok(action) = actions.get_Item(i) else { continue };
            let mut kind = Default::default();
            if action.Type(&mut kind).is_err() || kind != TASK_ACTION_EXEC {
                continue;
            }
            let Ok(exec) = action.cast::<IExecAction>() else { continue };
            let mut path = BSTR::new();
            let mut args = BSTR::new();
            let _ = exec.Path(&mut path);
            let _ = exec.Arguments(&mut args);
            let path = path.to_string();
            let args = args.to_string();
            let path = if path.contains(' ') && !path.starts_with('"') { format!("\"{path}\"") } else { path };
            commands.push(if args.is_empty() { path } else { format!("{path} {args}") });
        }
        // Tasks with only COM handlers still start something; say so.
        Some(if commands.is_empty() { "(runs a built-in Windows handler)".into() } else { commands.join(" & ") })
    }
}

fn walk(folder: &ITaskFolder, out: &mut Vec<ScheduledStartup>, visited: &mut usize) {
    *visited += 1;
    if *visited > MAX_FOLDERS {
        return;
    }
    unsafe {
        if let Ok(tasks) = folder.GetTasks(TASK_ENUM_HIDDEN.0) {
            let count = tasks.Count().unwrap_or(0);
            for i in 1..=count {
                let Ok(task) = tasks.get_Item(&index(i)) else { continue };
                let Some(command) = starts_at_sign_in(&task) else { continue };
                let Ok(path) = task.Path() else { continue };
                let enabled = task.Enabled().map(|e| e.as_bool()).unwrap_or(false);
                out.push(ScheduledStartup { path: path.to_string(), command, enabled });
            }
        }
        if let Ok(folders) = folder.GetFolders(0) {
            let count = folders.Count().unwrap_or(0);
            for i in 1..=count {
                let Ok(child) = folders.get_Item(&index(i)) else { continue };
                let is_system =
                    child.Path().map(|p| p.to_string().eq_ignore_ascii_case(SYSTEM_FOLDER)).unwrap_or(false);
                if !is_system {
                    walk(&child, out, visited);
                }
            }
        }
    }
}

/// Every non-Windows scheduled task with a sign-in or boot trigger.
pub fn startup_tasks() -> Vec<ScheduledStartup> {
    let _com = ComGuard::new();
    let Ok(service) = service() else { return Vec::new() };
    let Ok(root) = (unsafe { service.GetFolder(&BSTR::from(r"\")) }) else { return Vec::new() };
    let mut out = Vec::new();
    let mut visited = 0;
    walk(&root, &mut out, &mut visited);
    out
}

pub fn set_enabled(path: &str, enabled: bool) -> Result<()> {
    let _com = ComGuard::new();
    let service = service()?;
    let (folder, name) = path.rsplit_once('\\').ok_or_else(|| Error::Invalid(path.to_owned()))?;
    let folder = if folder.is_empty() { r"\" } else { folder };
    unsafe {
        let folder = service.GetFolder(&BSTR::from(folder)).map_err(|_| Error::NotFound(path.to_owned()))?;
        let task = folder.GetTask(&BSTR::from(name)).map_err(|_| Error::NotFound(path.to_owned()))?;
        task.SetEnabled(if enabled { VARIANT_TRUE } else { VARIANT_FALSE }).map_err(|e| {
            if e.code() == windows::Win32::Foundation::E_ACCESSDENIED {
                Error::AccessDenied
            } else {
                Error::msg(format!("Could not change the task: {}", e.message()))
            }
        })
    }
}
