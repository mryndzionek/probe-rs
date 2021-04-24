use crate::{
    probe::{DebugProbeInfo, DebugProbeType, ProbeCreationError},
    DebugProbe, DebugProbeError, DebugProbeSelector, WireProtocol,
};
use rusb::{Device, UsbContext};
use serialport::{available_ports, SerialPort, SerialPortType};
use std::time::Duration;

pub struct BMPProbe {
    pub device: BMPDevice,
    protocol: WireProtocol,
    speed: u32,
}

impl std::fmt::Debug for BMPProbe {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Black Magic Probe")
            .field("protocol", &self.protocol)
            .field("speed", &self.speed)
            .finish()
    }
}

impl BMPProbe {
    pub fn new_from_device(device: BMPDevice) -> Self {
        Self {
            device,
            protocol: WireProtocol::Swd,
            speed: 1000,
        }
    }
}
pub struct BMPDevice {
    _port: Box<dyn SerialPort>,
}

fn open_device_from_selector(
    selector: impl Into<DebugProbeSelector>,
) -> Result<BMPDevice, ProbeCreationError> {
    let selector = selector.into();

    match available_ports() {
        Ok(ports) => {
            for p in ports {
                log::debug!("Trying serial port: {}", p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        if (info.vid == selector.vendor_id) & (info.pid == selector.product_id) {
                            log::debug!("Found matching serial port: {}", p.port_name);
                            let port = serialport::new(p.port_name, 115_200)
                                .timeout(Duration::from_millis(10))
                                .open();

                            match port {
                                Ok(port) => {
                                    log::debug!("Serial port opened successfuly");
                                    return Ok(BMPDevice { _port: port });
                                }
                                Err(_e) => {
                                    return Err(ProbeCreationError::NotFound);
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
            Err(ProbeCreationError::NotFound)
        }
        Err(_e) => Err(ProbeCreationError::NotFound),
    }
}

impl DebugProbe for BMPProbe {
    fn new_from_selector(
        selector: impl Into<DebugProbeSelector>,
    ) -> Result<Box<Self>, DebugProbeError>
    where
        Self: Sized,
    {
        Ok(Box::new(BMPProbe::new_from_device(
            open_device_from_selector(selector)?,
        )))
    }

    fn get_name(&self) -> &str {
        "Black Magic Probe"
    }

    fn speed(&self) -> u32 {
        self.speed
    }

    fn set_speed(&mut self, speed_khz: u32) -> Result<u32, DebugProbeError> {
        self.speed = speed_khz;

        Ok(speed_khz)
    }

    fn attach(&mut self) -> Result<(), DebugProbeError> {
        log::debug!("Attaching with protocol '{}'", self.protocol);
        Ok(())
    }

    fn select_protocol(&mut self, protocol: WireProtocol) -> Result<(), DebugProbeError> {
        if protocol != WireProtocol::Jtag {
            Err(DebugProbeError::UnsupportedProtocol(protocol))
        } else {
            Ok(())
        }
    }

    fn detach(&mut self) -> Result<(), DebugProbeError> {
        Ok(())
    }

    fn target_reset(&mut self) -> Result<(), DebugProbeError> {
        log::error!("BMP target_reset");
        unimplemented!()
    }

    fn target_reset_assert(&mut self) -> Result<(), DebugProbeError> {
        log::error!("BMP target_assert");
        unimplemented!()
    }

    fn target_reset_deassert(&mut self) -> Result<(), DebugProbeError> {
        log::error!("BMP target_reset_deassert");
        unimplemented!()
    }

    fn into_probe(self: Box<Self>) -> Box<dyn DebugProbe> {
        self
    }

    fn has_arm_interface(&self) -> bool {
        true
    }

    fn try_get_arm_interface<'probe>(
        self: Box<Self>,
    ) -> Result<
        Box<dyn crate::architecture::arm::communication_interface::ArmProbeInterface + 'probe>,
        (Box<dyn DebugProbe>, DebugProbeError),
    > {
        todo!()
    }
}

fn get_bmp_info(device: &Device<rusb::Context>) -> Option<DebugProbeInfo> {
    let timeout = Duration::from_millis(100);
    let d_desc = device.device_descriptor().ok()?;

    if d_desc.vendor_id() != 0x1d50 {
        None
    } else {
        let handle = device.open().ok()?;
        let language = handle.read_languages(timeout).ok()?.get(0).cloned()?;
        let prod_str = handle
            .read_product_string(language, &d_desc, timeout)
            .ok()?;
        let sn_str = handle
            .read_serial_number_string(language, &d_desc, timeout)
            .ok();

        if prod_str.starts_with("Black Magic Probe") {
            Some(DebugProbeInfo {
                identifier: prod_str,
                vendor_id: d_desc.vendor_id(),
                product_id: d_desc.product_id(),
                serial_number: sn_str,
                probe_type: DebugProbeType::BMP,
            })
        } else {
            None
        }
    }
}

pub(crate) fn list_bmp_devices() -> Vec<DebugProbeInfo> {
    match rusb::Context::new().and_then(|ctx| ctx.devices()) {
        Ok(devices) => devices
            .iter()
            .filter_map(|device| get_bmp_info(&device))
            .collect(),
        Err(_) => vec![],
    }
}
