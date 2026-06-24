use serde::{Deserialize, Serialize};
use serialport::{available_ports, SerialPortType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_id: Option<String>,
    pub available: bool,
}

impl PortInfo {
    pub fn list_ports() -> Result<Vec<PortInfo>, serialport::Error> {
        let ports = available_ports()?;

        Ok(ports
            .into_iter()
            .map(|port| {
                let hardware_id = match &port.port_type {
                    SerialPortType::UsbPort(info) => {
                        Some(format!("USB VID:{:04X} PID:{:04X}", info.vid, info.pid))
                    }
                    SerialPortType::PciPort => Some("PCI".to_string()),
                    SerialPortType::BluetoothPort => Some("Bluetooth".to_string()),
                    SerialPortType::Unknown => None,
                };

                PortInfo {
                    name: port.port_name.clone(),
                    description: get_port_description(&port),
                    hardware_id,
                    available: true,
                }
            })
            .collect())
    }
}

fn get_port_description(port: &serialport::SerialPortInfo) -> String {
    match &port.port_type {
        SerialPortType::UsbPort(info) => {
            format!(
                "{} {}",
                info.manufacturer.as_ref().unwrap_or(&"Unknown".to_string()),
                info.product
                    .as_ref()
                    .unwrap_or(&"USB Serial Device".to_string())
            )
        }
        SerialPortType::PciPort => "PCI Serial Port".to_string(),
        SerialPortType::BluetoothPort => "Bluetooth Serial Port".to_string(),
        SerialPortType::Unknown => "Serial Port".to_string(),
    }
}
