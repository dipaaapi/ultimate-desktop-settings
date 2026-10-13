//! Windows service status (read without admin rights) and start/stop through
//! the standard UAC prompt.

use windows::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatus, SC_MANAGER_CONNECT, SERVICE_QUERY_STATUS,
    SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STATUS, SERVICE_STOP_PENDING, SERVICE_STOPPED,
};
use windows::core::{PCWSTR, w};

use super::wide;
use crate::bluetooth::ServiceState;
use crate::{Error, Result};

pub fn state(name: &str) -> ServiceState {
    let name_w = wide(name);
    unsafe {
        let Ok(scm) = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT) else {
            return ServiceState::Unknown;
        };
        let result = match OpenServiceW(scm, PCWSTR(name_w.as_ptr()), SERVICE_QUERY_STATUS) {
            Ok(service) => {
                let mut status = SERVICE_STATUS::default();
                let state = if QueryServiceStatus(service, &mut status).is_ok() {
                    match status.dwCurrentState {
                        SERVICE_RUNNING => ServiceState::Running,
                        SERVICE_STOPPED => ServiceState::Stopped,
                        SERVICE_START_PENDING => ServiceState::Starting,
                        SERVICE_STOP_PENDING => ServiceState::Stopping,
                        _ => ServiceState::Unknown,
                    }
                } else {
                    ServiceState::Unknown
                };
                let _ = CloseServiceHandle(service);
                state
            }
            Err(_) => ServiceState::Unknown,
        };
        let _ = CloseServiceHandle(scm);
        result
    }
}

/// Starts or stops a service via `sc.exe`, elevated with the UAC prompt.
pub fn control_elevated(name: &str, start: bool) -> Result<()> {
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(Error::Invalid(name.to_owned()));
    }
    let args = format!("{} {name}", if start { "start" } else { "stop" });
    super::shell::shell_execute(w!("runas"), "sc.exe", Some(&args), true)
        .map_err(|_| Error::msg("The change was cancelled or Windows didn't allow it."))
}
