//! Paired Bluetooth devices, as recorded by the Windows Bluetooth stack.

use serde::Serialize;

use crate::registry::{Hive, RegValue, Registry};

const DEVICES: &str = r"SYSTEM\CurrentControlSet\Services\BTHPORT\Parameters\Devices";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedDevice {
    pub address: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceState {
    Running,
    Stopped,
    Starting,
    Stopping,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BluetoothState {
    pub service: ServiceState,
    pub devices: Vec<PairedDevice>,
}

/// `001a7dda7113` -> `00:1A:7D:DA:71:13`
pub fn format_address(raw: &str) -> String {
    if raw.len() == 12 && raw.chars().all(|c| c.is_ascii_hexdigit()) {
        raw.as_bytes().chunks(2).map(|c| String::from_utf8_lossy(c).to_uppercase()).collect::<Vec<_>>().join(":")
    } else {
        raw.to_uppercase()
    }
}

fn decode_name(value: Option<RegValue>) -> Option<String> {
    let text = match value? {
        RegValue::Binary(bytes) => {
            let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
            String::from_utf8_lossy(&bytes[..end]).into_owned()
        }
        RegValue::String(s) | RegValue::ExpandString(s) => s,
        RegValue::Dword(_) => return None,
    };
    let text = text.trim().to_owned();
    (!text.is_empty()).then_some(text)
}

pub fn paired_devices(reg: &dyn Registry) -> Vec<PairedDevice> {
    let mut devices: Vec<PairedDevice> = reg
        .subkeys(Hive::LocalMachine, DEVICES)
        .into_iter()
        .map(|key| {
            let path = format!(r"{DEVICES}\{key}");
            let name = decode_name(reg.get(Hive::LocalMachine, &path, "FriendlyName"))
                .or_else(|| decode_name(reg.get(Hive::LocalMachine, &path, "Name")))
                .unwrap_or_else(|| "Unknown device".into());
            PairedDevice { address: format_address(&key), name }
        })
        .collect();
    devices.sort_by_key(|d| d.name.to_lowercase());
    devices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::MemoryRegistry;

    #[test]
    fn reads_binary_names_and_formats_addresses() {
        let reg = MemoryRegistry::new();
        reg.set(
            Hive::LocalMachine,
            &format!(r"{DEVICES}\001a7dda7113"),
            "Name",
            RegValue::Binary(b"WH-1000XM4\0\0".to_vec()),
        )
        .unwrap();
        reg.set(
            Hive::LocalMachine,
            &format!(r"{DEVICES}\a0b1c2d3e4f5"),
            "FriendlyName",
            RegValue::String("Keyboard".into()),
        )
        .unwrap();
        reg.create_key(Hive::LocalMachine, &format!(r"{DEVICES}\ffffffffffff"));
        let devices = paired_devices(&reg);
        assert_eq!(devices.len(), 3);
        assert_eq!(devices[0], PairedDevice { address: "A0:B1:C2:D3:E4:F5".into(), name: "Keyboard".into() });
        assert_eq!(devices[1].name, "Unknown device");
        assert_eq!(devices[2].address, "00:1A:7D:DA:71:13");
        assert_eq!(devices[2].name, "WH-1000XM4");
    }
}
