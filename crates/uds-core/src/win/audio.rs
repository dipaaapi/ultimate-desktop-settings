//! Speakers, master volume and the per-app mixer through Windows Core Audio.
//! v1.x only had the master volume; its app mixer and device list were mock-ups.

use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    AudioSessionStateExpired, DEVICE_STATE_ACTIVE, IAudioSessionControl2, IAudioSessionManager2, IMMDevice,
    IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator, eConsole, eRender,
};
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PropVariantToBSTR};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree, STGM_READ};
use windows::core::{GUID, Interface};

use super::process::process_image_path;
use crate::system::{AudioDevice, AudioSession, AudioState};
use crate::{Error, Result};

/// Initializes COM for the current thread for the guard's lifetime.
pub(crate) struct ComGuard(bool);

impl ComGuard {
    pub(crate) fn new() -> Self {
        use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};
        // S_FALSE (already initialized) still needs a matching uninitialize.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        Self(hr.is_ok())
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.0 {
            unsafe { windows::Win32::System::Com::CoUninitialize() };
        }
    }
}

fn err(context: &str, e: windows::core::Error) -> Error {
    Error::msg(format!("{context}: {}", e.message()))
}

fn enumerator() -> Result<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }.map_err(|e| err("Audio is not available", e))
}

fn default_device(enumerator: &IMMDeviceEnumerator) -> Result<IMMDevice> {
    unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole) }
        .map_err(|e| err("No speakers or headphones found", e))
}

fn endpoint_volume(device: &IMMDevice) -> Result<IAudioEndpointVolume> {
    unsafe { device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) }
        .map_err(|e| err("Volume control unavailable", e))
}

fn device_id(device: &IMMDevice) -> String {
    unsafe {
        match device.GetId() {
            Ok(pwstr) => {
                let id = pwstr.to_string().unwrap_or_default();
                CoTaskMemFree(Some(pwstr.0 as *const _));
                id
            }
            Err(_) => String::new(),
        }
    }
}

fn device_name(device: &IMMDevice) -> String {
    unsafe {
        let Ok(store) = device.OpenPropertyStore(STGM_READ) else { return "Audio device".into() };
        let Ok(mut value) = store.GetValue(&PKEY_Device_FriendlyName) else { return "Audio device".into() };
        let name = PropVariantToBSTR(&value).map(|b| b.to_string()).unwrap_or_else(|_| "Audio device".into());
        let _ = PropVariantClear(&mut value);
        name
    }
}

fn friendly_process_name(pid: u32) -> String {
    let Some(path) = process_image_path(pid) else { return format!("Process {pid}") };
    let stem = std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or(path);
    let mut chars = stem.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => stem,
    }
}

fn sessions(device: &IMMDevice) -> Vec<AudioSession> {
    let mut out: Vec<AudioSession> = Vec::new();
    unsafe {
        let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else { return out };
        let Ok(list) = manager.GetSessionEnumerator() else { return out };
        let count = list.GetCount().unwrap_or(0);
        for i in 0..count {
            let Ok(control) = list.GetSession(i) else { continue };
            if control.GetState().map(|s| s == AudioSessionStateExpired).unwrap_or(true) {
                continue;
            }
            let Ok(control2) = control.cast::<IAudioSessionControl2>() else { continue };
            let Ok(volume) = control.cast::<ISimpleAudioVolume>() else { continue };
            let pid = control2.GetProcessId().unwrap_or(0);
            let is_system = control2.IsSystemSoundsSession().0 == 0; // S_OK
            if out.iter().any(|s| s.pid == pid) {
                continue;
            }
            out.push(AudioSession {
                pid,
                name: if is_system || pid == 0 { "System sounds".into() } else { friendly_process_name(pid) },
                volume: volume.GetMasterVolume().unwrap_or(0.0),
                muted: volume.GetMute().map(|b| b.as_bool()).unwrap_or(false),
                is_system,
            });
        }
    }
    out.sort_by(|a, b| b.is_system.cmp(&a.is_system).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    out
}

pub fn state() -> Result<AudioState> {
    let _com = ComGuard::new();
    let enumerator = enumerator()?;
    let default = default_device(&enumerator)?;
    let default_id = device_id(&default);
    let volume = endpoint_volume(&default)?;
    let (master, muted) = unsafe {
        (
            volume.GetMasterVolumeLevelScalar().map_err(|e| err("Could not read the volume", e))?,
            volume.GetMute().map(|b| b.as_bool()).unwrap_or(false),
        )
    };
    let mut devices = Vec::new();
    unsafe {
        if let Ok(collection) = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) {
            for i in 0..collection.GetCount().unwrap_or(0) {
                if let Ok(device) = collection.Item(i) {
                    let id = device_id(&device);
                    devices.push(AudioDevice { is_default: id == default_id, name: device_name(&device), id });
                }
            }
        }
    }
    Ok(AudioState { master, muted, devices, sessions: sessions(&default) })
}

pub fn set_master(level: Option<f32>, mute: Option<bool>) -> Result<()> {
    let _com = ComGuard::new();
    let volume = endpoint_volume(&default_device(&enumerator()?)?)?;
    unsafe {
        if let Some(level) = level {
            volume
                .SetMasterVolumeLevelScalar(level.clamp(0.0, 1.0), &GUID::zeroed())
                .map_err(|e| err("Could not set the volume", e))?;
        }
        if let Some(mute) = mute {
            volume.SetMute(mute, &GUID::zeroed()).map_err(|e| err("Could not mute", e))?;
        }
    }
    Ok(())
}

pub fn set_session(pid: u32, level: Option<f32>, mute: Option<bool>) -> Result<()> {
    let _com = ComGuard::new();
    let device = default_device(&enumerator()?)?;
    let mut found = false;
    unsafe {
        let manager =
            device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None).map_err(|e| err("Mixer unavailable", e))?;
        let list = manager.GetSessionEnumerator().map_err(|e| err("Mixer unavailable", e))?;
        for i in 0..list.GetCount().unwrap_or(0) {
            let Ok(control) = list.GetSession(i) else { continue };
            let Ok(control2) = control.cast::<IAudioSessionControl2>() else { continue };
            if control2.GetProcessId().unwrap_or(u32::MAX) != pid {
                continue;
            }
            let Ok(volume) = control.cast::<ISimpleAudioVolume>() else { continue };
            if let Some(level) = level {
                volume
                    .SetMasterVolume(level.clamp(0.0, 1.0), &GUID::zeroed())
                    .map_err(|e| err("Could not set app volume", e))?;
            }
            if let Some(mute) = mute {
                volume.SetMute(mute, &GUID::zeroed()).map_err(|e| err("Could not mute the app", e))?;
            }
            found = true;
        }
    }
    if found { Ok(()) } else { Err(Error::NotFound("That app is no longer playing sound.".into())) }
}
